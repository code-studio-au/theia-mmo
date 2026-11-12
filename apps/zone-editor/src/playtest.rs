//! Offline capsule sandbox. Uses the existing gameplay solver; no network session.
use super::*;
use avian3d::prelude::Collider;
use theia_core::{coordinates::LocalPosition, time::TickRate};
use theia_network::{
    kinematic::{self, Motion},
    physics::{CAMERA, GROUND, PlacedCollider, SOLID, StaticScene},
    wire::Intent,
};

pub(super) struct Playtest {
    scene: StaticScene,
    pub feet: LocalPosition,
    motion: Motion,
    spawn: LocalPosition,
}
#[cfg(test)]
pub(super) fn collision(zone: &Zone) -> Result<StaticScene, String> {
    collision_with_library(zone, &scenery_library::Library::default())
}
fn collision_with_library(
    zone: &Zone,
    library: &scenery_library::Library,
) -> Result<StaticScene, String> {
    zone.validate()?;
    let scenery = scenery_collision::colliders(zone, library)?;
    let mut colliders: Vec<_> = terrain::ground_boxes(zone)
        .into_iter()
        .map(|(center, size)| PlacedCollider {
            shape: Collider::cuboid(size.x, size.y, size.z),
            center,
            layer: SOLID | GROUND | CAMERA,
        })
        .collect();
    if let Some(geometry) = terrain::geometry(zone) {
        colliders.push(PlacedCollider {
            shape: Collider::trimesh(
                geometry
                    .positions
                    .into_iter()
                    .map(Vec3::from_array)
                    .collect(),
                geometry.triangles,
            ),
            center: Vec3::ZERO,
            layer: SOLID | GROUND | CAMERA,
        });
    }
    for object in &zone.objects {
        let [x, y, z] = object.pose.size.map(|v| v as f32);
        let shape = match object.primitive {
            Primitive::Box => Collider::cuboid(x, y, z),
            Primitive::Sphere => Collider::sphere(x / 2.),
            Primitive::Cylinder => Collider::cylinder(x / 2., y),
        };
        let shape = Collider::compound(vec![(
            Vec3::ZERO,
            Quat::from_rotation_y(object.pose.yaw_degrees.to_radians() as f32),
            shape,
        )]);
        colliders.push(PlacedCollider {
            shape,
            center: Vec3::from_array(zone.local(object.pose.position)?),
            layer: SOLID | GROUND | CAMERA,
        });
    }
    colliders.extend(scenery);
    Ok(StaticScene::new(colliders))
}
impl Playtest {
    pub fn new(zone: &Zone) -> Result<Self, String> {
        Self::with_library(zone, &scenery_library::Library::default())
    }
    pub fn with_library(zone: &Zone, library: &scenery_library::Library) -> Result<Self, String> {
        let scene = collision_with_library(zone, library)?;
        let guess = LocalPosition::new(zone.local(zone.spawn)?).ok_or("Invalid spawn")?;
        let spawn =
            kinematic::spawn(&scene, guess).map_err(|e| format!("Playtest spawn refused: {e}"))?;
        Ok(Self {
            scene,
            feet: spawn,
            spawn,
            motion: Motion::default(),
        })
    }
}
pub(super) fn step(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
) {
    if !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if session.play.is_none() {
        return;
    }
    let mut heading = session.heading;
    heading += (i32::from(keys.pressed(KeyCode::KeyA)) - i32::from(keys.pressed(KeyCode::KeyD)))
        as f32
        * 2.
        / 30.;
    session.heading = heading;
    let Some(play) = session.play.as_mut() else {
        return;
    };
    let forward =
        (i32::from(keys.pressed(KeyCode::KeyW)) - i32::from(keys.pressed(KeyCode::KeyS))) as f32;
    let right =
        (i32::from(keys.pressed(KeyCode::KeyE)) - i32::from(keys.pressed(KeyCode::KeyQ))) as f32;
    let delta = Quat::from_rotation_y(heading) * Vec3::new(right, 0., -forward);
    let intent = Intent {
        axes: [
            (delta.x * 1000.).round() as i16,
            (delta.z * 1000.).round() as i16,
        ],
        jump: keys.pressed(KeyCode::Space),
        ..default()
    };
    match kinematic::advance(
        &play.scene,
        &mut play.motion,
        play.feet,
        intent,
        TickRate::Hz30,
    ) {
        Ok(feet) if feet.metres().iter().all(|v| v.abs() < 500.) && feet.metres()[1] > -100. => {
            play.feet = feet
        }
        _ => {
            play.feet = play.spawn;
            play.motion = Motion::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_rotated_shapes_block_shared_capsule_and_bad_spawn_refuses() {
        let mut zone = Zone::default();
        zone.objects.push(Object {
            id: 1,
            name: "Wall".into(),
            primitive: Primitive::Box,
            pose: Pose {
                position: [0., 1.5, 0.],
                size: [12., 3., 1.],
                yaw_degrees: 15.,
            },
        });
        zone.objects.push(Object {
            id: 2,
            name: "Sphere".into(),
            primitive: Primitive::Sphere,
            pose: Pose {
                position: [8., 1., 0.],
                size: [2.; 3],
                yaw_degrees: 0.,
            },
        });
        zone.objects.push(Object {
            id: 3,
            name: "Pillar".into(),
            primitive: Primitive::Cylinder,
            pose: Pose {
                position: [-8., 1.5, 0.],
                size: [2., 3., 2.],
                yaw_degrees: 0.,
            },
        });
        let mut play = Playtest::new(&zone).unwrap();
        for _ in 0..90 {
            play.feet = kinematic::advance(
                &play.scene,
                &mut play.motion,
                play.feet,
                Intent {
                    axes: [0, -1000],
                    ..default()
                },
                TickRate::Hz30,
            )
            .unwrap();
        }
        let feet = play.feet.metres();
        let wall_local = Quat::from_rotation_y(-15_f32.to_radians()) * Vec3::from_array(feet);
        assert!(
            (0.83..0.9).contains(&wall_local.z),
            "Rotated wall contact {feet:?} -> {wall_local:?}"
        );
        assert!(feet[0] > 0.5, "Expected sliding along the authored wall");
        assert!(feet[1].abs() < 0.02);
        assert!(
            !play
                .scene
                .line_clear(Vec3::new(8., 1., 4.), Vec3::new(8., 1., -4.))
        );
        assert!(
            !play
                .scene
                .line_clear(Vec3::new(-8., 1., 4.), Vec3::new(-8., 1., -4.))
        );
        zone.spawn = [0., 0.1, 0.];
        assert!(Playtest::new(&zone).is_err());
    }
}

#[cfg(test)]
mod terrain_tests {
    use super::*;
    #[test]
    fn every_terrain_triangle_matches_avian_including_depressions_and_far_frames() {
        for sign in [-1., 1.] {
            let chunk = [(sign as i64) * 1_000_000, -(sign as i64) * 1_000_000];
            let mut patch = theia_zone::terrain::HeightPatch::flat(chunk);
            let origin = patch.origin().unwrap();
            for z in 0..33 {
                for x in 0..33 {
                    patch.heights[z * 33 + x] = sign
                        * 4.
                        * (1. - ((x as f32 - 16.).abs().max((z as f32 - 16.).abs())) / 16.);
                }
            }
            let zone = Zone {
                schema: 2,
                origin,
                spawn: [origin[0] + 64., f64::from(sign * 4.) + 0.1, origin[2] + 64.],
                terrain: Some(patch),
                ..Zone::default()
            };
            let scene = collision(&zone).unwrap();
            let geometry = terrain::geometry(&zone).unwrap();
            let patch = zone.terrain.as_ref().unwrap();
            for triangle in geometry.triangles {
                let centre = triangle
                    .map(|index| Vec3::from_array(geometry.positions[index as usize]))
                    .into_iter()
                    .sum::<Vec3>()
                    / 3.;
                let height = scene.ground([centre.x, centre.z], 64.).unwrap().0;
                assert!(
                    (height - centre.y).abs() < 0.0001,
                    "Actual Avian triangle mismatch {centre:?} vs {height}"
                );
                let sampled = patch
                    .height(
                        origin[0] + f64::from(centre.x),
                        origin[2] + f64::from(centre.z),
                    )
                    .unwrap()
                    .unwrap();
                assert!(
                    (sampled - centre.y).abs() < 0.0001,
                    "Authored interpolation mismatch"
                );
            }
            let play = Playtest::new(&zone).unwrap();
            assert!(
                (play.feet.metres()[1] - sign * 4.).abs() < 0.08,
                "Shared capsule terrain spawn {:?}",
                play.feet.metres()
            );
        }
    }
}

#[cfg(test)]
mod grading_tests {
    use super::*;
    #[test]
    fn graded_path_mesh_matches_all_avian_triangles_and_actual_capsule_walk() {
        use theia_zone::{
            Command, Edit, Editor,
            grading::Grade,
            paths::{GuidePath, PathKind},
            terrain::HeightPatch,
        };
        for sign in [-1_i64, 1] {
            let patch = HeightPatch::flat([sign * 1_000_000_000, -sign * 1_000_000_000]);
            let mut origin = patch.origin().unwrap();
            origin[1] = 24.;
            let mut editor = Editor::new(Zone {
                schema: 3,
                origin,
                terrain: Some(patch),
                spawn: [
                    origin[0] + 64.,
                    origin[1] + sign as f64 * 4. + 0.1,
                    origin[2] + 64.,
                ],
                paths: vec![GuidePath {
                    id: 1,
                    name: "Grade".into(),
                    kind: PathKind::Road,
                    width_metres: 8.,
                    points: vec![
                        [
                            origin[0] + 32.,
                            origin[1] + sign as f64 * 2. + 0.03,
                            origin[2] + 64.,
                        ],
                        [
                            origin[0] + 96.,
                            origin[1] + sign as f64 * 6. + 0.03,
                            origin[2] + 64.,
                        ],
                    ],
                }],
                ..Zone::default()
            })
            .unwrap();
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::GradePath {
                        grade: Grade {
                            path_id: 1,
                            surface_offset_metres: -0.03,
                            falloff_metres: 8.,
                            blend: 1.,
                        },
                    }],
                })
                .unwrap();
            let zone = editor.zone();
            let scene = collision(zone).unwrap();
            let geometry = terrain::geometry(zone).unwrap();
            for triangle in geometry.triangles {
                let centre = triangle
                    .map(|index| Vec3::from_array(geometry.positions[index as usize]))
                    .into_iter()
                    .sum::<Vec3>()
                    / 3.;
                let actual = scene.ground([centre.x, centre.z], 64.).unwrap().0;
                assert!(
                    (actual - centre.y).abs() < 0.0001,
                    "Graded Avian mismatch {centre:?} vs {actual}"
                );
            }
            let mut play = Playtest::new(zone).unwrap();
            let start = play.feet.metres();
            for _ in 0..30 {
                play.feet = kinematic::advance(
                    &play.scene,
                    &mut play.motion,
                    play.feet,
                    Intent {
                        axes: [1000, 0],
                        ..default()
                    },
                    TickRate::Hz30,
                )
                .unwrap();
            }
            let feet = play.feet.metres();
            assert!(feet[0] > start[0] + 4.);
            assert!(
                (feet[1] - start[1]) * sign as f32 > 0.2,
                "Capsule must traverse graded slope {start:?} -> {feet:?}"
            );
            let sampled = zone
                .surface_position([
                    origin[0] + f64::from(feet[0]),
                    origin[2] + f64::from(feet[2]),
                ])
                .unwrap()[1]
                - origin[1];
            assert!((f64::from(feet[1]) - sampled).abs() < 0.08);
        }
    }
}

#[cfg(test)]
mod resolution_tests {
    use super::*;
    #[test]
    fn fine_sculpt_paint_and_all_avian_triangles_use_the_same_bounded_grid() {
        use theia_zone::{
            Command, Edit, Editor,
            painting::{Paint, Surface},
            terrain::{Brush, BrushOperation, HeightPatch, Resolution},
        };
        for sign in [-1_i64, 1] {
            let patch = HeightPatch::flat([sign * 1_000_000_000, -sign * 1_000_000_000]);
            let origin = patch.origin().unwrap();
            let mut e = Editor::new(Zone {
                schema: 2,
                origin,
                spawn: [origin[0] + 64., 4.6, origin[2] + 64.],
                terrain: Some(patch),
                ..Zone::default()
            })
            .unwrap();
            e.execute(Command::Apply {
                expected_revision: 0,
                edits: vec![
                    Edit::Brush {
                        brush: Brush {
                            centre: [origin[0] + 64., origin[2] + 64.],
                            radius_metres: 16.,
                            operation: BrushOperation::Raise { metres: 4. },
                        },
                    },
                    Edit::ResampleTerrain {
                        resolution: Resolution::TwoMetres,
                    },
                    Edit::Brush {
                        brush: Brush {
                            centre: [origin[0] + 66., origin[2] + 64.],
                            radius_metres: 4.,
                            operation: BrushOperation::Raise { metres: 1. },
                        },
                    },
                    Edit::PaintTerrain {
                        paint: Paint {
                            centre: [origin[0] + 64., origin[2] + 64.],
                            radius_metres: 8.,
                            surface: Surface::Rock,
                            blend: 0.7,
                        },
                    },
                ],
            })
            .unwrap();
            let zone = e.zone();
            let geometry = terrain::geometry(zone).unwrap();
            assert_eq!(geometry.positions.len(), 65 * 65);
            assert_eq!(geometry.triangles.len(), 8192);
            let mesh = terrain::surface_mesh(zone).unwrap();
            assert_eq!(mesh.count_vertices(), 65 * 65);
            let scene = collision(zone).unwrap();
            for triangle in geometry.triangles {
                let centre = triangle
                    .map(|i| Vec3::from_array(geometry.positions[i as usize]))
                    .into_iter()
                    .sum::<Vec3>()
                    / 3.;
                assert!(
                    (scene.ground([centre.x, centre.z], 64.).unwrap().0 - centre.y).abs() < 0.0001
                );
            }
            let mut play = Playtest::new(zone).unwrap();
            let start = play.feet.metres();
            for _ in 0..30 {
                play.feet = kinematic::advance(
                    &play.scene,
                    &mut play.motion,
                    play.feet,
                    Intent {
                        axes: [1000, 0],
                        ..default()
                    },
                    TickRate::Hz30,
                )
                .unwrap();
            }
            let feet = play.feet.metres();
            assert!(feet[0] > start[0] + 4.);
            let height = zone
                .surface_position([
                    origin[0] + f64::from(feet[0]),
                    origin[2] + f64::from(feet[2]),
                ])
                .unwrap()[1];
            assert!((f64::from(feet[1]) - height).abs() < 0.1);
        }
    }
}

#[cfg(test)]
mod example_tests {
    use super::*;
    #[test]
    fn wayfarer_hollow_route_and_shelter_use_actual_avian_collision() {
        let zone = Zone::from_json(include_bytes!(
            "../../../content/zones/wayfarer-hollow.base.json"
        ))
        .unwrap();
        let mut editor = Editor::new(zone).unwrap();
        for line in include_str!("../../../content/zones/wayfarer-hollow.commands.jsonl").lines() {
            editor.execute(serde_json::from_str(line).unwrap()).unwrap();
        }
        let zone = editor.zone();
        assert_eq!(zone.schema, 7);
        assert_eq!(
            (
                zone.objects.len(),
                zone.paths.len(),
                zone.plants.len(),
                zone.markers.len()
            ),
            (11, 3, 48, 2)
        );
        let mut play = Playtest::new(zone).unwrap();
        let route = &zone.paths.iter().find(|p| p.id == 100).unwrap().points;
        for target in route.iter().skip(1) {
            let target = zone.local(*target).unwrap();
            let mut reached = false;
            for _ in 0..400 {
                let feet = play.feet.metres();
                let delta = Vec2::new(target[0] - feet[0], target[2] - feet[2]);
                if delta.length() < 0.35 {
                    reached = true;
                    break;
                }
                let axes = delta.normalize() * 1000.;
                play.feet = kinematic::advance(
                    &play.scene,
                    &mut play.motion,
                    play.feet,
                    Intent {
                        axes: [axes.x.round() as i16, axes.y.round() as i16],
                        ..default()
                    },
                    TickRate::Hz30,
                )
                .unwrap();
                let global = zone.global(play.feet.metres()).unwrap();
                let surface = zone.surface_position([global[0], global[2]]).unwrap();
                assert!(
                    (global[1] - surface[1]).abs() < 0.15,
                    "Road contact {global:?} vs {surface:?}"
                );
            }
            assert!(
                reached,
                "Road blocked before {target:?}, feet {:?}",
                play.feet.metres()
            );
        }
        let feet = play.feet.metres();
        assert!((feet[1] - 6.).abs() < 0.15);
        let mut shelter = zone.clone();
        shelter.spawn = [42., 0.15, 97.];
        let mut play = Playtest::new(&shelter).unwrap();
        for _ in 0..120 {
            play.feet = kinematic::advance(
                &play.scene,
                &mut play.motion,
                play.feet,
                Intent {
                    axes: [0, 1000],
                    ..default()
                },
                TickRate::Hz30,
            )
            .unwrap();
        }
        let feet = play.feet.metres();
        assert!(
            feet[2] > 100.7 && feet[2] < 101.3,
            "Shelter wall failed {feet:?}"
        );
        assert!(
            (feet[1] - 0.05).abs() < 0.08,
            "Shelter floor failed {feet:?}"
        );
        shelter.spawn = [36., 0.1, 97.];
        assert!(Playtest::new(&shelter).is_err());
    }
}

#[cfg(test)]
mod scenery_tests {
    use super::*;
    fn walk(play: &mut Playtest, axes: [i16; 2], ticks: usize) {
        for _ in 0..ticks {
            play.feet = kinematic::advance(
                &play.scene,
                &mut play.motion,
                play.feet,
                Intent { axes, ..default() },
                TickRate::Hz30,
            )
            .unwrap();
        }
    }
    #[test]
    #[ignore = "Requires the local proprietary Synty catalog via THEIA_SCENERY_CATALOG"]
    fn actual_synty_scenery_arch_passage_and_pillar_contact() {
        let path = std::env::var_os("THEIA_SCENERY_CATALOG").expect("Set THEIA_SCENERY_CATALOG");
        let library = scenery_library::Library::load(std::path::Path::new(&path)).unwrap();
        let entry = &library.assets["synty.forest.archway"].entry;
        for origin in [[0.; 3], [128_000_000_000., 20., -128_000_000_000.]] {
            let mut zone = Zone {
                schema: 10,
                origin,
                spawn: [origin[0], origin[1] + 0.1, origin[2] + 4.],
                scenery: vec![theia_zone::scenery::SceneryPlacement {
                    id: 1,
                    name: "Archway route".into(),
                    asset: entry.id.clone(),
                    position: [origin[0], origin[1] - entry.bounds[0][1], origin[2]],
                    yaw_degrees: 0.,
                    scale: 1.,
                }],
                ..default()
            };
            let mut play = Playtest::with_library(&zone, &library).unwrap();
            walk(&mut play, [0, -1000], 120);
            let through = play.feet.metres();
            println!("scenery arch passage origin={origin:?} end={through:?}");
            assert!(
                through[2] < -2.,
                "Arch opening must remain passable: {through:?}"
            );
            zone.spawn[0] = origin[0] + 1.5;
            let mut play = Playtest::with_library(&zone, &library).unwrap();
            walk(&mut play, [0, -1000], 120);
            let pillar = play.feet.metres();
            println!("scenery pillar origin={origin:?} end={pillar:?}");
            assert!(pillar[2] > 0., "Solid pillar must stop passage: {pillar:?}");
        }
        let zone = Zone::load(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../content/zones/forest-scenery.zone.json"
        )))
        .unwrap();
        let colliders = scenery_collision::colliders(&zone, &library).unwrap();
        assert_eq!(
            colliders.len(),
            6,
            "Six solid placements; lantern and pebbles decorative"
        );
        assert!(Playtest::with_library(&zone, &library).is_ok());
        assert!(
            Playtest::new(&zone).is_err(),
            "No silent missing-library omission"
        );
    }
}
