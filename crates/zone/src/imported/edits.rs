//! Version-17 deterministic brush journal over immutable source tiles.
//! Only affected sample buffers are copied; snapshots share all other buffers.
use super::*;
use crate::{
    painting::{GRASS, Paint},
    terrain::{Brush, BrushOperation},
};
pub const MAX_STAMPS: usize = 65536;
const PAGE: usize = 128;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Stamp {
    Sculpt { brush: Brush },
    Paint { paint: Paint },
}
#[derive(Clone, Debug)]
pub(super) struct Edited {
    pub heights: Vec<Arc<[f32]>>,
    pub weights: Vec<Option<Arc<[[u8; 4]]>>>,
    pub stamps: usize,
}
impl Imported {
    pub(crate) fn extended_brushes(&self) -> bool {
        self.journal
            .iter()
            .flat_map(|p| p.iter())
            .any(|s| matches!(s, Stamp::Sculpt { brush } if brush.extended()))
    }
    pub fn stamp_count(&self) -> usize {
        self.journal.iter().map(|p| p.len()).sum()
    }
    pub(super) fn replay(&mut self) -> Result<()> {
        require(
            self.stamp_count() <= MAX_STAMPS
                && self
                    .journal
                    .iter()
                    .all(|p| !p.is_empty() && p.len() <= PAGE),
            "Imported terrain edit journal budget exceeded",
        )?;
        self.edited = None;
        let journal = self.journal.clone();
        for stamp in journal.iter().flat_map(|p| p.iter()) {
            self.apply_stamp(stamp)?;
        }
        Ok(())
    }
    fn record(&mut self, stamp: Stamp) -> Result<()> {
        self.validate()?;
        require(
            self.stamp_count() < MAX_STAMPS,
            "Imported terrain edit journal is full (65536 stamps)",
        )?;
        self.apply_stamp(&stamp)?;
        if self.journal.last().is_none_or(|p| p.len() == PAGE) {
            self.journal.push(Arc::new(vec![]));
        }
        Arc::make_mut(self.journal.last_mut().unwrap()).push(stamp);
        Ok(())
    }
    pub(crate) fn sculpt(&mut self, brush: &Brush) -> Result<()> {
        self.record(Stamp::Sculpt {
            brush: brush.clone(),
        })
    }
    pub(crate) fn paint(&mut self, paint: &Paint) -> Result<()> {
        self.record(Stamp::Paint {
            paint: paint.clone(),
        })
    }
    pub(super) fn heights(&self) -> &[Arc<[f32]>] {
        self.edited.as_ref().map_or_else(
            || self.surface.as_ref().unwrap().heights.as_slice(),
            |s| s.heights.as_slice(),
        )
    }
    fn tile_index(&self, id: ChunkId) -> Option<usize> {
        let s = self.surface.as_ref()?;
        let x = usize::try_from(id.x - s.origin[0]).ok()?;
        let z = usize::try_from(id.z - s.origin[1]).ok()?;
        (x < s.counts[0] && z < s.counts[1]).then_some(z * s.counts[0] + x)
    }
    pub fn tile_samples(&self, id: ChunkId) -> Option<Arc<[f32]>> {
        {
            let i = self.tile_index(id)?;
            Some(self.heights()[i].clone())
        }
    }
    pub fn tile_weights(&self, id: ChunkId) -> Option<Arc<[[u8; 4]]>> {
        self.edited.as_ref()?.weights[self.tile_index(id)?].clone()
    }
    // Global sample indices within this package; no duplicated seam ownership.
    fn sample(&self, x: usize, z: usize) -> f32 {
        let s = self.surface.as_ref().unwrap();
        let tx = (x / self.cells).min(s.counts[0] - 1);
        let tz = (z / self.cells).min(s.counts[1] - 1);
        self.heights()[tz * s.counts[0] + tx]
            [(z - tz * self.cells) * (self.cells + 1) + x - tx * self.cells]
    }
    fn apply_stamp(&mut self, stamp: &Stamp) -> Result<()> {
        let (centre, radius) = match stamp {
            Stamp::Sculpt { brush } => (brush.centre, brush.radius_metres),
            Stamp::Paint { paint } => (paint.centre, paint.radius_metres),
        };
        let maximum = match stamp {
            Stamp::Sculpt { brush } => brush.maximum_radius(),
            Stamp::Paint { .. } => 32.,
        };
        require(
            centre.iter().all(|v| v.is_finite())
                && radius.is_finite()
                && (4. ..=maximum).contains(&radius),
            "Brush needs finite centre and valid operation radius",
        )?;
        require(
            (0..2).all(|i| (self.bounds[0][i]..=self.bounds[1][i]).contains(&centre[i])),
            "Brush centre outside imported terrain",
        )?;
        match stamp {
            Stamp::Sculpt { brush } => match brush.operation {
                BrushOperation::Raise { metres } | BrushOperation::Lower { metres } => require(
                    metres.is_finite() && (0.05..=8.).contains(&metres),
                    "Brush amount must be 0.05..8 metres",
                )?,
                BrushOperation::Flatten { height, blend } => require(
                    height.is_finite()
                        && height.abs() <= 1024.
                        && blend.is_finite()
                        && (0.05..=1.).contains(&blend),
                    "Invalid imported flatten height/blend",
                )?,
                BrushOperation::FlattenArea {
                    height,
                    edge_metres,
                    noise,
                } => crate::terrain::area::validate(height, edge_metres, noise, 1024.)?,
                BrushOperation::Smooth { blend } => require(
                    blend.is_finite() && (0.05..=1.).contains(&blend),
                    "Invalid smooth blend",
                )?,
            },
            Stamp::Paint { paint } => require(
                paint.blend.is_finite() && (0.05..=1.).contains(&paint.blend),
                "Paint blend must be 0.05..1",
            )?,
        }
        let radius = match stamp {
            Stamp::Sculpt { brush } => brush.support_radius(),
            Stamp::Paint { .. } => radius,
        };
        let s = self
            .surface
            .as_ref()
            .ok_or("Imported terrain is not loaded")?;
        let mut next = self.edited.as_ref().map_or_else(
            || Edited {
                heights: s.heights.clone(),
                weights: vec![None; s.heights.len()],
                stamps: 0,
            },
            |e| (**e).clone(),
        );
        let spacing = 128. / self.cells as f64;
        let max = [s.counts[0] * self.cells, s.counts[1] * self.cells];
        let low: [usize; 2] = std::array::from_fn(|i| {
            ((centre[i] - radius - self.bounds[0][i]) / spacing)
                .floor()
                .max(0.) as usize
        });
        let high: [usize; 2] = std::array::from_fn(|i| {
            (((centre[i] + radius - self.bounds[0][i]) / spacing).ceil() as usize).min(max[i])
        });
        let mut changed = false;
        for tz in
            (low[1] / self.cells).saturating_sub(1)..=(high[1] / self.cells).min(s.counts[1] - 1)
        {
            for tx in (low[0] / self.cells).saturating_sub(1)
                ..=(high[0] / self.cells).min(s.counts[0] - 1)
            {
                let tile = tz * s.counts[0] + tx;
                for z in low[1].max(tz * self.cells)..=high[1].min((tz + 1) * self.cells) {
                    for x in low[0].max(tx * self.cells)..=high[0].min((tx + 1) * self.cells) {
                        let dx = self.bounds[0][0] + x as f64 * spacing - centre[0];
                        let dz = self.bounds[0][1] + z as f64 * spacing - centre[1];
                        let influence = (1. - dx.hypot(dz) / radius).max(0.) as f32;
                        if influence == 0. {
                            continue;
                        }
                        let i = (z - tz * self.cells) * (self.cells + 1) + x - tx * self.cells;
                        match stamp {
                            Stamp::Sculpt { brush } => {
                                let old = self.sample(x, z);
                                let value = match brush.operation {
                                    BrushOperation::Raise { metres } => old + metres * influence,
                                    BrushOperation::Lower { metres } => old - metres * influence,
                                    BrushOperation::Flatten { height, blend } => {
                                        old + (height - old) * blend * influence
                                    }
                                    BrushOperation::FlattenArea { .. } => brush.area_value(
                                        [
                                            self.bounds[0][0] + x as f64 * spacing,
                                            self.bounds[0][1] + z as f64 * spacing,
                                        ],
                                        old,
                                    ),
                                    BrushOperation::Smooth { blend } => {
                                        let average = (self.sample(x.saturating_sub(1), z)
                                            + self.sample((x + 1).min(max[0]), z)
                                            + self.sample(x, z.saturating_sub(1))
                                            + self.sample(x, (z + 1).min(max[1])))
                                            / 4.;
                                        old + (average - old) * blend * influence
                                    }
                                };
                                require(
                                    value.is_finite() && value.abs() <= 1024.,
                                    "Brush would exceed imported height limits (+/-1024 m)",
                                )?;
                                if value != old {
                                    Arc::make_mut(&mut next.heights[tile])[i] = value;
                                    changed = true;
                                }
                            }
                            Stamp::Paint { paint } => {
                                let old = next.weights[tile].as_ref().map_or(GRASS, |w| w[i]);
                                let mut value = old;
                                let target = paint.surface.index();
                                let mut others = 0_u16;
                                for (channel, w) in value.iter_mut().enumerate() {
                                    if channel != target {
                                        *w = (f32::from(old[channel])
                                            * (1. - influence * paint.blend))
                                            .round()
                                            as u8;
                                        others += u16::from(*w);
                                    }
                                }
                                value[target] = (255 - others) as u8;
                                if value != old {
                                    let weights = next.weights[tile].get_or_insert_with(|| {
                                        vec![GRASS; (self.cells + 1).pow(2)].into()
                                    });
                                    Arc::make_mut(weights)[i] = value;
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        require(changed, "Brush did not change any samples")?;
        next.stamps += 1;
        self.edited = Some(Arc::new(next));
        Ok(())
    }
}
