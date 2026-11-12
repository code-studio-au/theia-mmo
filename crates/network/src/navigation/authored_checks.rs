//! Shared loader/worker capsule checks for authored routes and spawn candidates.
//! May load collision patches; never call from the fixed simulation step.
use super::query::Query;
use crate::{Result, authored_world::Store, procedural};
use std::collections::BTreeMap;
use theia_core::coordinates::GlobalPosition;

pub struct Grounding<'a> {
    store: &'a Store,
    patches: BTreeMap<theia_core::coordinates::ChunkId, procedural::Patch>,
}
impl<'a> Grounding<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self {
            store,
            patches: BTreeMap::new(),
        }
    }
    pub fn at(&mut self, point: GlobalPosition) -> Result<GlobalPosition> {
        let id = procedural::owner(point).ok_or("Route outside prediction domain")?;
        if let std::collections::btree_map::Entry::Vacant(entry) = self.patches.entry(id) {
            entry.insert(self.store.patch(id)?);
        }
        let patch = &self.patches[&id];
        let mut local = patch
            .frame
            .to_local(point)
            .ok_or("Route outside local frame")?
            .metres();
        let (height, walkable) = patch
            .scene
            .ground([local[0], local[2]], local[1] + 0.3)
            .ok_or("Route has no close collision surface")?;
        if !walkable || (height - local[1]).abs() > 0.3 {
            return Err(format!(
                "Navigation route at {:?} disagrees with collision height {} (walkable={walkable})",
                point.metres(),
                height
            )
            .into());
        }
        local[1] = height + 0.03;
        let local =
            theia_core::coordinates::LocalPosition::new(local).ok_or("Invalid route pose")?;
        let local = crate::kinematic::spawn(&patch.scene, local)?;
        crate::kinematic::validate_pose(&patch.scene, local, true).map_err(|error| {
            format!(
                "Route grounding at {:?}, local feet {:?}: {error}",
                point.metres(),
                local.metres()
            )
        })?;
        patch
            .frame
            .to_global(local)
            .ok_or("Invalid global route".into())
    }
}
pub fn clearance(
    query: &Query,
    grounding: &mut Grounding<'_>,
    points: &[[f64; 3]],
    samples: &mut usize,
) -> Result<()> {
    for segment in points.windows(2) {
        let distance = (0..3)
            .map(|i| (segment[1][i] - segment[0][i]).powi(2))
            .sum::<f64>()
            .sqrt();
        let steps = (distance / 0.2).ceil().max(1.) as usize;
        *samples = samples
            .checked_add(steps + 1)
            .filter(|n| *n <= 8192)
            .ok_or("Authored route clearance sample budget exceeded")?;
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            let point = GlobalPosition::new(std::array::from_fn(|i| {
                segment[0][i] + (segment[1][i] - segment[0][i]) * fraction
            }))
            .ok_or("Invalid route interpolation")?;
            query
                .sample(point)
                .map_err(|error| format!("Route clearance at {:?}: {error}", point.metres()))?;
            grounding.at(point)?;
        }
    }
    Ok(())
}
