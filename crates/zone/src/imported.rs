//! Immutable terrain source with a saved sculpt/paint journal for whole-zone authoring.
//! Samples are verified once at document admission and shared across undo snapshots.
use crate::{Result, height_tiles::Store, require};
mod edits;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use theia_core::coordinates::ChunkId;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Imported {
    pub package: String,
    pub identity: [u8; 32],
    pub bounds: [[f64; 2]; 2],
    pub cells: usize,
    #[serde(skip)]
    surface: Option<Arc<Surface>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    journal: Vec<Arc<Vec<edits::Stamp>>>,
    #[serde(skip)]
    edited: Option<Arc<edits::Edited>>,
}
impl PartialEq for Imported {
    fn eq(&self, other: &Self) -> bool {
        self.package == other.package
            && self.identity == other.identity
            && self.bounds == other.bounds
            && self.cells == other.cells
            && self.journal == other.journal
    }
}
#[derive(Debug)]
struct Surface {
    package: String,
    identity: [u8; 32],
    bounds: [[f64; 2]; 2],
    origin: [i64; 2],
    counts: [usize; 2],
    cells: usize,
    heights: Vec<Arc<[f32]>>,
}
impl Imported {
    pub(crate) fn share_samples_from(&mut self, other: &Self) {
        if self == other {
            self.surface.clone_from(&other.surface);
            self.edited.clone_from(&other.edited);
        }
    }
    pub fn open(path: &Path) -> Result<Self> {
        let store = Store::open(path).map_err(|e| e.to_string())?;
        let mut result = Self {
            package: store
                .root()
                .to_str()
                .ok_or("Terrain path must be UTF-8")?
                .into(),
            identity: store.identity(),
            bounds: store.bounds(),
            cells: store.cells(),
            surface: None,
            journal: vec![],
            edited: None,
        };
        result.load_from(&store)?;
        Ok(result)
    }
    pub fn hydrate(&mut self) -> Result<()> {
        self.validate_descriptor()?;
        let store = Store::open(Path::new(&self.package)).map_err(|e| e.to_string())?;
        self.load_from(&store)?;
        self.replay()
    }

    pub(crate) fn hydrate_export(&mut self, store: &Store) -> Result<()> {
        self.package = store
            .root()
            .to_str()
            .ok_or("Terrain path must be UTF-8")?
            .into();
        self.load_from(store)?;
        self.replay()
    }
    fn validate_descriptor(&self) -> Result<()> {
        require(
            self.package.len() <= 4096 && Path::new(&self.package).is_absolute(),
            "Imported terrain needs an absolute local package path",
        )?;
        require(
            matches!(self.cells, 32 | 64 | 128),
            "Unsupported imported sample spacing",
        )?;
        require(
            (0..2).all(|i| {
                self.bounds[0][i].is_finite()
                    && self.bounds[1][i].is_finite()
                    && self.bounds[0][i] >= 0.
                    && self.bounds[1][i] <= 16000.
                    && (128. ..=8192.).contains(&(self.bounds[1][i] - self.bounds[0][i]))
            }),
            "Imported authoring supports up to 8192 m per axis",
        )
    }
    fn load_from(&mut self, store: &Store) -> Result<()> {
        self.validate_descriptor()?;
        require(
            store.identity() == self.identity
                && store.bounds() == self.bounds
                && store.cells() == self.cells,
            "Imported terrain package identity or layout changed",
        )?;
        let (origin, counts) = store.chunk_layout();
        require(
            counts[0] * counts[1] <= 4096
                && counts[0] * counts[1] * (self.cells + 1).pow(2) * 4 <= 128 * 1024 * 1024,
            "Imported tile or 128 MiB source sample budget exceeded",
        )?;
        let mut heights: Vec<Arc<[f32]>> = Vec::with_capacity(counts[0] * counts[1]);
        let samples = self.cells + 1;
        for z in 0..counts[1] {
            for x in 0..counts[0] {
                let h = store
                    .heights(ChunkId {
                        x: origin[0] + x as i64,
                        z: origin[1] + z as i64,
                    })
                    .map_err(|e| e.to_string())?;
                if x > 0 {
                    require(
                        (0..samples).all(|i| {
                            heights[z * counts[0] + x - 1][i * samples + self.cells].to_bits()
                                == h[i * samples].to_bits()
                        }),
                        "Imported X seam mismatch",
                    )?;
                }
                if z > 0 {
                    require(
                        (0..samples).all(|i| {
                            heights[(z - 1) * counts[0] + x][self.cells * samples + i].to_bits()
                                == h[i].to_bits()
                        }),
                        "Imported Z seam mismatch",
                    )?;
                }
                heights.push(h.into());
            }
        }
        self.surface = Some(Arc::new(Surface {
            package: self.package.clone(),
            identity: self.identity,
            bounds: self.bounds,
            origin,
            counts,
            cells: self.cells,
            heights,
        }));
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.validate_descriptor()?;
        require(
            self.edited.as_ref().map_or(0, |e| e.stamps) == self.stamp_count(),
            "Imported edit journal is not hydrated",
        )?;
        let s = self
            .surface
            .as_ref()
            .ok_or("Imported terrain is not loaded")?;
        require(
            s.package == self.package
                && s.identity == self.identity
                && s.bounds == self.bounds
                && s.cells == self.cells,
            "Imported terrain descriptor differs from loaded samples",
        )
    }
    pub fn tile_count(&self) -> usize {
        self.surface.as_ref().map_or(0, |s| s.heights.len())
    }
    pub fn sample_bytes(&self) -> usize {
        self.tile_count() * (self.cells + 1).pow(2) * 4
    }
    pub fn tiles(&self) -> impl Iterator<Item = (ChunkId, &[f32])> {
        self.surface.iter().flat_map(|s| {
            self.heights().iter().enumerate().map(move |(i, h)| {
                (
                    ChunkId {
                        x: s.origin[0] + (i % s.counts[0]) as i64,
                        z: s.origin[1] + (i / s.counts[0]) as i64,
                    },
                    h.as_ref(),
                )
            })
        })
    }
    pub fn height(&self, xz: [f64; 2]) -> Result<f32> {
        self.validate()?;
        require(
            (0..2).all(|i| {
                xz[i].is_finite() && (self.bounds[0][i]..=self.bounds[1][i]).contains(&xz[i])
            }),
            "Outside imported terrain",
        )?;
        let s = self.surface.as_ref().unwrap();
        let relative = [xz[0] - self.bounds[0][0], xz[1] - self.bounds[0][1]];
        let tile: [usize; 2] =
            std::array::from_fn(|i| ((relative[i] / 128.).floor() as usize).min(s.counts[i] - 1));
        let uv: [f64; 2] = std::array::from_fn(|i| {
            (relative[i] - tile[i] as f64 * 128.) * self.cells as f64 / 128.
        });
        let cell: [usize; 2] = uv.map(|v| (v.floor() as usize).min(self.cells - 1));
        let u = (uv[0] - cell[0] as f64) as f32;
        let v = (uv[1] - cell[1] as f64) as f32;
        let h = &self.heights()[tile[1] * s.counts[0] + tile[0]];
        let stride = self.cells + 1;
        let i = cell[1] * stride + cell[0];
        let [a, b, c, d] = [h[i], h[i + 1], h[i + stride], h[i + stride + 1]];
        Ok(if u + v <= 1. {
            a + u * (b - a) + v * (c - a)
        } else {
            d + (1. - u) * (c - d) + (1. - v) * (b - d)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor, Object, Pose, Primitive, Zone};
    use ring::digest::{SHA256, digest};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn fixture(counts: [usize; 2], cells: usize) -> Fixture {
        let root = std::env::temp_dir().join(format!(
            "theia-imported-authoring-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let mut hashes = vec![];
        for z in 0..counts[1] {
            for x in 0..counts[0] {
                let mut bytes = b"THTILE01".to_vec();
                bytes.extend((x as i64).to_le_bytes());
                bytes.extend((z as i64).to_le_bytes());
                for row in 0..=cells {
                    for col in 0..=cells {
                        let gx = x * 128 + col * 128 / cells;
                        let gz = z * 128 + row * 128 / cells;
                        let height = 100. + gx as f32 / 128. + gz as f32 / 64.;
                        bytes.extend(height.to_le_bytes());
                    }
                }
                hashes.push(digest(&SHA256, &bytes).as_ref().to_vec());
                fs::write(root.join(format!("tile_{x}_{z}.height")), bytes).unwrap();
            }
        }
        let manifest = serde_json::json!({"schema":2,"content_id":"theia.test.imported","revision":1,"chunk_metres":128,"chunks_xz":counts,"chunk_origin":[0,0],"samples_per_axis":cells+1,"tiles":hashes});
        fs::write(
            root.join("terrain.manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        Fixture(root)
    }
    fn zone(imported: Imported) -> Zone {
        let x = imported.bounds[1][0] * 0.5;
        let z = imported.bounds[1][1] * 0.5;
        Zone {
            schema: 16,
            origin: [x, 0., z],
            spawn: [x, f64::from(imported.height([x, z]).unwrap()) + 0.1, z],
            imported_terrain: Some(imported),
            ..Default::default()
        }
    }
    #[test]
    fn imported_samples_support_seams_both_triangles_and_outer_edges_at_all_resolutions() {
        for cells in [32, 64, 128] {
            let f = fixture([2, 2], cells);
            let t = Imported::open(&f.0).unwrap();
            for [x, z] in [
                [0., 0.],
                [256., 256.],
                [128., 127.5],
                [127.5, 128.],
                [40.25, 32.75],
                [41.75, 35.75],
            ] {
                assert!(
                    (f64::from(t.height([x, z]).unwrap()) - (100. + x / 128. + z / 64.)).abs()
                        < 0.00002
                );
            }
            assert!(t.height([256.01, 20.]).is_err());
            assert!(t.height([f64::NAN, 20.]).is_err());
            assert_eq!(t.sample_bytes(), 4 * (cells + 1).pow(2) * 4);
        }
    }
    #[test]
    fn full_zone_placements_save_reload_undo_and_share_immutable_sample_storage() {
        let f = fixture([32, 24], 32);
        let z = zone(Imported::open(&f.0).unwrap());
        z.validate().unwrap();
        let source = z
            .imported_terrain
            .as_ref()
            .unwrap()
            .surface
            .as_ref()
            .unwrap()
            .clone();
        let mut e = Editor::new(z.clone()).unwrap();
        for (i, xz) in [[64., 64.], [4000., 3000.], [2560., 2540.]]
            .into_iter()
            .enumerate()
        {
            let mut p = z.surface_position(xz).unwrap();
            p[1] += 1.;
            e.execute(Command::Apply {
                expected_revision: e.revision(),
                edits: vec![Edit::Place {
                    object: Object {
                        id: i as u64 + 1,
                        name: format!("Placed {i}"),
                        primitive: Primitive::Box,
                        pose: Pose {
                            position: p,
                            yaw_degrees: 0.,
                            size: [2.; 3],
                        },
                    },
                }],
            })
            .unwrap();
            assert!(Arc::ptr_eq(
                &source,
                e.zone()
                    .imported_terrain
                    .as_ref()
                    .unwrap()
                    .surface
                    .as_ref()
                    .unwrap()
            ));
        }
        let placed = e.zone().clone();
        e.execute(Command::Undo {
            expected_revision: e.revision(),
        })
        .unwrap();
        assert_eq!(e.zone().objects.len(), 2);
        e.execute(Command::Redo {
            expected_revision: e.revision(),
        })
        .unwrap();
        assert_eq!(e.zone(), &placed);
        let path = f.0.join("editor.zone.json");
        e.zone().save(&path).unwrap();
        assert!(fs::metadata(&path).unwrap().len() < 4096);
        assert_eq!(Zone::load(&path).unwrap(), placed);
        e.replace(Zone::load(&path).unwrap(), e.revision()).unwrap();
        assert!(Arc::ptr_eq(
            &source,
            e.zone()
                .imported_terrain
                .as_ref()
                .unwrap()
                .surface
                .as_ref()
                .unwrap()
        ));
        assert!(z.check_position([4097., 100., 64.], 0.).is_err());
        assert!(z.check_position([0.5, 100., 64.], 1.).is_err());
        let before = e.zone().clone();
        assert!(
            e.execute(Command::Apply {
                expected_revision: e.revision(),
                edits: vec![Edit::SetTerrain { terrain: None }]
            })
            .is_err()
        );
        assert_eq!(e.zone(), &before);
    }
    #[test]
    fn regional_area_reaches_checked_frame_edges_and_retains_edit_history() {
        let f = fixture([57, 64], 32);
        let z = zone(Imported::open(&f.0).unwrap());
        z.validate().unwrap();
        assert_eq!(z.local([0., 0., 0.]).unwrap(), [-3648., 0., -4096.]);
        assert_eq!(z.local([7296., 0., 8192.]).unwrap(), [3648., 0., 4096.]);
        let mut e = Editor::new(z.clone()).unwrap();
        let mut p = z.surface_position([7200., 8100.]).unwrap();
        p[1] += 1.;
        apply(
            &mut e,
            vec![Edit::Place {
                object: Object {
                    id: 1,
                    name: "Regional corner".into(),
                    primitive: Primitive::Box,
                    pose: Pose {
                        position: p,
                        yaw_degrees: 0.,
                        size: [2.; 3],
                    },
                },
            }],
        );
        let path = f.0.join("region.zone.json");
        e.zone().save(&path).unwrap();
        assert_eq!(Zone::load(&path).unwrap(), *e.zone());
        e.execute(Command::Undo {
            expected_revision: e.revision(),
        })
        .unwrap();
        assert_eq!(*e.zone(), z);
        let mut oversized = z.imported_terrain.unwrap();
        oversized.bounds[1][1] = 8320.;
        assert!(oversized.validate_descriptor().is_err());
        // Reject excessive source storage before reading any tile payloads.
        let manifest_path = f.0.join("terrain.manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["samples_per_axis"] = 129.into();
        fs::write(manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(Imported::open(&f.0).unwrap_err().contains("128 MiB"));
    }
    #[test]
    fn changed_or_missing_package_and_unhydrated_or_mutated_descriptors_are_rejected() {
        let f = fixture([2, 2], 128);
        let z = zone(Imported::open(&f.0).unwrap());
        let bytes = serde_json::to_vec(&z).unwrap();
        let raw: Zone = serde_json::from_slice(&bytes).unwrap();
        assert!(raw.validate().is_err());
        let mut mutated = z.clone();
        mutated.imported_terrain.as_mut().unwrap().identity[0] ^= 1;
        assert!(mutated.validate().is_err());
        mutated = z.clone();
        mutated.schema = 15;
        assert!(mutated.validate().is_err());
        let tile = f.0.join("tile_1_1.height");
        let mut changed = fs::read(&tile).unwrap();
        changed[25] ^= 1;
        fs::write(&tile, changed).unwrap();
        assert!(Zone::from_json(&bytes).is_err());
        fs::remove_file(tile).unwrap();
        assert!(Zone::from_json(&bytes).is_err());
        // The already admitted immutable snapshot stays valid and unchanged.
        z.validate().unwrap();
        assert!((z.surface_position([200., 200.]).unwrap()[1] - 104.6875).abs() < 1e-8);
    }
    fn apply(e: &mut Editor, edits: Vec<Edit>) {
        e.execute(Command::Apply {
            expected_revision: e.revision(),
            edits,
        })
        .unwrap();
    }
    fn sculpt(centre: [f64; 2], operation: crate::terrain::BrushOperation) -> Edit {
        Edit::Brush {
            brush: crate::terrain::Brush {
                centre,
                radius_metres: 12.,
                operation,
            },
        }
    }
    fn seams(t: &Imported) {
        let tiles: Vec<_> = t.tiles().collect();
        let n = t.cells + 1;
        for (id, h) in &tiles {
            if let Some((_, right)) = tiles
                .iter()
                .find(|(other, _)| other.x == id.x + 1 && other.z == id.z)
            {
                for i in 0..n {
                    assert_eq!(h[i * n + n - 1].to_bits(), right[i * n].to_bits());
                }
            }
            if let Some((_, above)) = tiles
                .iter()
                .find(|(other, _)| other.x == id.x && other.z == id.z + 1)
            {
                for i in 0..n {
                    assert_eq!(h[(n - 1) * n + i].to_bits(), above[i].to_bits());
                }
            }
        }
    }
    #[test]
    fn imported_strokes_cross_seams_and_round_trip_all_tools_with_shared_history() {
        use crate::{
            painting::{Paint, Surface as Colour},
            terrain::BrushOperation as Op,
        };
        for cells in [32, 64, 128] {
            let f = fixture([3, 3], cells);
            let original = zone(Imported::open(&f.0).unwrap());
            let bytes = fs::read(f.0.join("tile_1_1.height")).unwrap();
            let untouched = original
                .imported_terrain
                .as_ref()
                .unwrap()
                .tile_samples(ChunkId { x: 2, z: 2 })
                .unwrap();
            let mut e = Editor::new(original.clone()).unwrap();
            let initial = original.surface_position([128., 128.]).unwrap()[1];
            apply(
                &mut e,
                vec![
                    sculpt([128., 128.], Op::Raise { metres: 8. }),
                    sculpt([128., 128.], Op::Lower { metres: 2. }),
                ],
            );
            assert_eq!(e.history(), (1, 0));
            assert_eq!(
                e.zone().surface_position([128., 128.]).unwrap()[1],
                initial + 6.
            );
            let t = e.zone().imported_terrain.as_ref().unwrap();
            seams(t);
            assert!(Arc::ptr_eq(
                &untouched,
                &t.tile_samples(ChunkId { x: 2, z: 2 }).unwrap()
            ));
            let step = 128. / cells as f64;
            let average = ([[-step, 0.], [step, 0.], [0., -step], [0., step]]
                .map(|d| t.height([128. + d[0], 128. + d[1]]).unwrap())
                .into_iter()
                .sum::<f32>())
                / 4.;
            apply(&mut e, vec![sculpt([128., 128.], Op::Smooth { blend: 1. })]);
            assert_eq!(
                e.zone().surface_position([128., 128.]).unwrap()[1],
                f64::from(average)
            );
            apply(
                &mut e,
                vec![
                    sculpt(
                        [128., 128.],
                        Op::Flatten {
                            height: 250.,
                            blend: 1.,
                        },
                    ),
                    Edit::PaintTerrain {
                        paint: Paint {
                            centre: [128., 128.],
                            radius_metres: 12.,
                            surface: Colour::Rock,
                            blend: 1.,
                        },
                    },
                ],
            );
            let changed = e.zone().clone();
            let t = changed.imported_terrain.as_ref().unwrap();
            seams(t);
            assert_eq!(changed.schema, 17);
            assert_eq!(t.height([128., 128.]).unwrap(), 250.);
            for (id, index) in [
                (ChunkId { x: 0, z: 0 }, (cells + 1).pow(2) - 1),
                (ChunkId { x: 1, z: 0 }, cells * (cells + 1)),
                (ChunkId { x: 0, z: 1 }, cells),
                (ChunkId { x: 1, z: 1 }, 0),
            ] {
                assert_eq!(t.tile_weights(id).unwrap()[index], [0, 0, 255, 0]);
            }
            e.execute(Command::Undo {
                expected_revision: e.revision(),
            })
            .unwrap();
            assert_ne!(e.zone(), &changed);
            e.execute(Command::Redo {
                expected_revision: e.revision(),
            })
            .unwrap();
            assert_eq!(e.zone(), &changed);
            let path = f.0.join("edited.zone.json");
            changed.save(&path).unwrap();
            let loaded = Zone::load(&path).unwrap();
            assert_eq!(loaded, changed);
            let restored = loaded.imported_terrain.as_ref().unwrap();
            for ((id, a), (_, b)) in t.tiles().zip(restored.tiles()) {
                assert_eq!(a, b);
                assert_eq!(t.tile_weights(id), restored.tile_weights(id));
            }
            assert!(fs::metadata(&path).unwrap().len() < 8192);
            assert_eq!(fs::read(f.0.join("tile_1_1.height")).unwrap(), bytes);
        }
    }
    #[test]
    fn area_flatten_large_smooth_seams_replay_history_and_rejection() {
        use crate::terrain::{Brush, BrushOperation as Op, area::AreaNoise};
        let f = fixture([4, 4], 64);
        let original = zone(Imported::open(&f.0).unwrap());
        let centre = [255.25, 256.75];
        let level = original.surface_position(centre).unwrap()[1] as f32;
        let mut e = Editor::new(original.clone()).unwrap();
        let edit = |noise| Edit::Brush {
            brush: Brush {
                centre,
                radius_metres: 160.,
                operation: Op::FlattenArea {
                    height: level,
                    edge_metres: 40.,
                    noise,
                },
            },
        };
        apply(&mut e, vec![edit(None)]);
        let t = e.zone().imported_terrain.as_ref().unwrap();
        assert_eq!(e.zone().schema, 18);
        seams(t);
        for xz in [[128., 256.], [256., 128.], [256., 256.], [400., 256.]] {
            assert_eq!(t.height(xz).unwrap(), level);
        }
        assert_eq!(e.zone().surface_position(centre).unwrap()[1] as f32, level);
        assert_eq!(
            t.height([460., 256.]).unwrap(),
            original
                .imported_terrain
                .as_ref()
                .unwrap()
                .height([460., 256.])
                .unwrap()
        );
        let old = original
            .imported_terrain
            .as_ref()
            .unwrap()
            .height([432., 256.])
            .unwrap();
        let band = t.height([432., 256.]).unwrap();
        assert!(band > level && band < old);
        let plateau = e.zone().clone();
        e.execute(Command::Undo {
            expected_revision: e.revision(),
        })
        .unwrap();
        assert_eq!(e.zone(), &original);
        e.execute(Command::Redo {
            expected_revision: e.revision(),
        })
        .unwrap();
        assert_eq!(e.zone(), &plateau);
        apply(
            &mut e,
            vec![edit(Some(AreaNoise {
                height_metres: 3.,
                width_metres: 19.,
                anchor: centre,
            }))],
        );
        let noisy = e.zone().clone();
        seams(noisy.imported_terrain.as_ref().unwrap());
        assert_ne!(
            noisy
                .imported_terrain
                .as_ref()
                .unwrap()
                .height([256., 256.]),
            plateau
                .imported_terrain
                .as_ref()
                .unwrap()
                .height([256., 256.])
        );
        apply(
            &mut e,
            vec![Edit::Brush {
                brush: Brush {
                    centre,
                    radius_metres: 160.,
                    operation: Op::Smooth { blend: 1. },
                },
            }],
        );
        assert_eq!(e.zone().schema, 18);
        seams(e.zone().imported_terrain.as_ref().unwrap());
        // Subsequent paint and old tools must not downgrade the new journal.
        apply(&mut e, vec![sculpt(centre, Op::Raise { metres: 1. })]);
        assert_eq!(e.zone().schema, 18);
        let path = f.0.join("area.zone.json");
        e.zone().save(&path).unwrap();
        let restored = Zone::load(&path).unwrap();
        for ((_, a), (_, b)) in e
            .zone()
            .imported_terrain
            .as_ref()
            .unwrap()
            .tiles()
            .zip(restored.imported_terrain.as_ref().unwrap().tiles())
        {
            assert_eq!(a, b);
        }
        let mut raw = serde_json::to_value(e.zone()).unwrap();
        raw["schema"] = 17.into();
        assert!(Zone::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
        for (radius, operation) in [
            (160.1, Op::Smooth { blend: 1. }),
            (160., Op::Raise { metres: 1. }),
            (
                160.,
                Op::FlattenArea {
                    height: level,
                    edge_metres: f64::NAN,
                    noise: None,
                },
            ),
            (
                160.,
                Op::FlattenArea {
                    height: level,
                    edge_metres: 40.,
                    noise: Some(AreaNoise {
                        height_metres: 3.,
                        width_metres: 0.,
                        anchor: centre,
                    }),
                },
            ),
        ] {
            let before = e.zone().clone();
            let revision = e.revision();
            let history = e.history();
            assert!(
                e.execute(Command::Apply {
                    expected_revision: revision,
                    edits: vec![Edit::Brush {
                        brush: Brush {
                            centre,
                            radius_metres: radius,
                            operation
                        }
                    }]
                })
                .is_err()
            );
            assert_eq!(e.zone(), &before);
            assert_eq!(e.revision(), revision);
            assert_eq!(e.history(), history);
        }
    }
    #[test]
    fn imported_edge_strokes_and_invalid_transactions_preserve_document_and_redo() {
        use crate::terrain::BrushOperation as Op;
        let f = fixture([2, 2], 128);
        let mut e = Editor::new(zone(Imported::open(&f.0).unwrap())).unwrap();
        apply(&mut e, vec![sculpt([256., 256.], Op::Raise { metres: 2. })]);
        assert_eq!(
            e.zone()
                .imported_terrain
                .as_ref()
                .unwrap()
                .height([256., 256.])
                .unwrap(),
            108.
        );
        e.execute(Command::Undo {
            expected_revision: e.revision(),
        })
        .unwrap();
        for edit in [
            sculpt([257., 0.], Op::Raise { metres: 2. }),
            sculpt(
                [128., 128.],
                Op::Flatten {
                    height: 1025.,
                    blend: 1.,
                },
            ),
            sculpt([f64::NAN, 0.], Op::Smooth { blend: 1. }),
        ] {
            let old = e.zone().clone();
            let rev = e.revision();
            let history = e.history();
            assert!(
                e.execute(Command::Apply {
                    expected_revision: rev,
                    edits: vec![sculpt([128., 128.], Op::Raise { metres: 2. }), edit]
                })
                .is_err()
            );
            assert_eq!(e.zone(), &old);
            assert_eq!(e.revision(), rev);
            assert_eq!(e.history(), history);
        }
        e.execute(Command::Redo {
            expected_revision: e.revision(),
        })
        .unwrap();
        let bytes = serde_json::to_vec(e.zone()).unwrap();
        let raw: Zone = serde_json::from_slice(&bytes).unwrap();
        assert!(raw.validate().is_err());
        let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        json["schema"] = 16.into();
        assert!(Zone::from_json(&serde_json::to_vec(&json).unwrap()).is_err());
        let base = e
            .zone()
            .imported_terrain
            .as_ref()
            .unwrap()
            .tile_samples(ChunkId { x: 1, z: 1 })
            .unwrap();
        e.replace(Zone::from_json(&bytes).unwrap(), e.revision())
            .unwrap();
        assert!(Arc::ptr_eq(
            &base,
            &e.zone()
                .imported_terrain
                .as_ref()
                .unwrap()
                .tile_samples(ChunkId { x: 1, z: 1 })
                .unwrap()
        ));
    }
}
