//! Standard Bevy greybox trees/shrubs. Export reuses these exact meshes/transforms.
use super::*;
use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings};
use std::collections::BTreeMap;
use theia_zone::vegetation::{Plant, PlantKind, Scatter};
#[derive(Component)]
pub(super) struct AuthoredPlant(u64);
#[derive(Resource)]
pub(super) struct Scene {
    meshes: [Handle<Mesh>; 2],
    trunk: Handle<StandardMaterial>,
    leaf: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
    cache: BTreeMap<u64, Vec<Entity>>,
    signature: Option<(u64, Option<u64>)>,
    cursor: Entity,
}
pub(super) fn mesh(index: usize) -> Mesh {
    if index == 0 {
        Cylinder::new(1., 1.)
            .mesh()
            .resolution(8)
            .segments(1)
            .build()
    } else {
        Sphere::new(1.)
            .mesh()
            .ico(1)
            .expect("Fixed low-poly sphere")
    }
}
pub(super) fn parts(zone: &Zone, plant: &Plant) -> Vec<(usize, Transform)> {
    let p = Vec3::from_array(zone.local(plant.position).unwrap());
    let s = plant.scale as f32;
    let yaw = Quat::from_rotation_y(plant.yaw_degrees.to_radians() as f32);
    let part = |index, offset, size| {
        (
            index,
            Transform {
                translation: p + Vec3::Y * offset * s,
                rotation: yaw,
                scale: size * s,
            },
        )
    };
    if plant.kind == PlantKind::Tree {
        vec![
            part(0, 1.5, Vec3::new(0.2, 3., 0.2)),
            part(1, 3.2, Vec3::splat(1.8)),
        ]
    } else {
        vec![part(1, 0.6, Vec3::new(0.9, 0.6, 0.9))]
    }
}
pub(super) fn actions() -> [(Action, &'static str); 12] {
    [
        (Action::SelectPlant, "Select plant [Esc]"),
        (Action::Plant(PlantKind::Tree, false), "Place tree proxy"),
        (Action::Plant(PlantKind::Shrub, false), "Place shrub proxy"),
        (Action::Plant(PlantKind::Tree, true), "Scatter trees"),
        (Action::Plant(PlantKind::Shrub, true), "Scatter shrubs"),
        (Action::NextPlant, "Select next plant"),
        (Action::DeletePlant, "Delete plant [Del]"),
        (Action::Undo, "Undo [Ctrl Z]"),
        (Action::Redo, "Redo [Ctrl Y]"),
        (Action::Save, "Save zone [Ctrl S]"),
        (Action::Load, "Reload zone [Ctrl Shift O]"),
        (Action::Play, "Enter / leave playtest [F5]"),
    ]
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let selected = materials.add(Color::srgb(0.95, 0.65, 0.15));
    let cursor_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.9, 0.4, 0.15),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let cursor = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(1., 0.03))),
            MeshMaterial3d(cursor_material),
            Transform::default(),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
        ))
        .id();
    commands.insert_resource(Scene {
        meshes: [meshes.add(mesh(0)), meshes.add(mesh(1))],
        trunk: materials.add(Color::srgb(0.3, 0.16, 0.07)),
        leaf: materials.add(Color::srgb(0.16, 0.39, 0.13)),
        selected,
        cache: BTreeMap::new(),
        signature: None,
        cursor,
    });
}
pub(super) fn sync(mut commands: Commands, session: Res<Session>, mut scene: ResMut<Scene>) {
    let signature = (
        session.editor.revision(),
        session.plant_selected.filter(|_| session.vegetation_mode),
    );
    if scene.signature != Some(signature) {
        let removed: Vec<_> = scene
            .cache
            .keys()
            .filter(|id| !session.editor.zone().plants.iter().any(|p| p.id == **id))
            .copied()
            .collect();
        for id in removed {
            for e in scene.cache.remove(&id).unwrap() {
                commands.entity(e).despawn();
            }
        }
        for plant in &session.editor.zone().plants {
            let parts = parts(session.editor.zone(), plant);
            if scene
                .cache
                .get(&plant.id)
                .is_some_and(|entities| entities.len() != parts.len())
            {
                for e in scene.cache.remove(&plant.id).unwrap() {
                    commands.entity(e).despawn();
                }
            }
            if let Some(entities) = scene.cache.get(&plant.id) {
                for (entity, (index, transform)) in entities.iter().zip(parts) {
                    let material = if signature.1 == Some(plant.id) {
                        scene.selected.clone()
                    } else if index == 0 {
                        scene.trunk.clone()
                    } else {
                        scene.leaf.clone()
                    };
                    commands.entity(*entity).insert((
                        Mesh3d(scene.meshes[index].clone()),
                        MeshMaterial3d(material),
                        transform,
                    ));
                }
            } else {
                let entities = parts
                    .into_iter()
                    .map(|(index, transform)| {
                        let material = if signature.1 == Some(plant.id) {
                            scene.selected.clone()
                        } else if index == 0 {
                            scene.trunk.clone()
                        } else {
                            scene.leaf.clone()
                        };
                        commands
                            .spawn((
                                AuthoredPlant(plant.id),
                                Mesh3d(scene.meshes[index].clone()),
                                MeshMaterial3d(material),
                                transform,
                            ))
                            .id()
                    })
                    .collect();
                scene.cache.insert(plant.id, entities);
            }
        }
        scene.signature = Some(signature);
    }
    if session.vegetation_mode
        && session.play.is_none()
        && session.plant_palette.is_some()
        && let Some(point) = session.plant_point
    {
        let radius = if session.plant_scatter {
            session.scatter_radius as f32
        } else {
            1.
        };
        commands.entity(scene.cursor).insert((
            Visibility::Visible,
            Transform::from_translation(
                Vec3::from_array(session.editor.zone().local(point).unwrap()) + Vec3::Y * 0.04,
            )
            .with_scale(Vec3::new(radius, 1., radius)),
        ));
    } else {
        commands.entity(scene.cursor).insert(Visibility::Hidden);
    }
}
pub(super) fn perform(action: Action, session: &mut Session) {
    match action {
        Action::SelectPlant => session.plant_palette = None,
        Action::Plant(kind, scatter) => {
            session.plant_palette = Some(kind);
            session.plant_scatter = scatter;
            session.plant_selected = None;
            session.status = "Click ground to place vegetation".into();
        }
        Action::NextPlant => {
            session.plant_palette = None;
            let plants = &session.editor.zone().plants;
            session.plant_selected = if plants.is_empty() {
                None
            } else {
                Some(
                    plants[plants
                        .iter()
                        .position(|p| Some(p.id) == session.plant_selected)
                        .map_or(0, |i| (i + 1) % plants.len())]
                    .id,
                )
            };
        }
        Action::DeletePlant => {
            if let Some(id) = session.plant_selected {
                session.edit(vec![Edit::DeletePlant { id }]);
            }
        }
        _ => unreachable!("Vegetation action dispatch"),
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<crate::viewport::ZoneCamera>>,
    plants: Query<&AuthoredPlant>,
    objects: Query<(), With<viewport::Authored>>,
    ground: Query<(), With<terrain::GroundPart>>,
    mut raycast: MeshRayCast,
    mut session: ResMut<Session>,
) {
    session.plant_point = None;
    if session.has_modal() {
        return;
    }
    if !session.vegetation_mode || session.play.is_some() {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused {
        return;
    }
    let Some(cursor) = canvas_cursor(window, &session) else {
        return;
    };
    let Ok((camera, transform)) = camera.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };
    if let Some(kind) = session.plant_palette {
        let filter = |e| ground.contains(e);
        let settings = MeshRayCastSettings::default().with_filter(&filter);
        let Some(point) = raycast
            .cast_ray(ray, &settings)
            .first()
            .map(|(_, hit)| ray.get_point(hit.distance))
        else {
            return;
        };
        let point = session.snapped(session.editor.zone().global(point.to_array()).unwrap());
        let Ok(position) = session.editor.zone().surface_position([point[0], point[2]]) else {
            return;
        };
        session.plant_point = Some(position);
        if !mouse.just_pressed(MouseButton::Left) {
            return;
        }
        let id = session.next_id;
        let count = if session.plant_scatter {
            session.scatter_count
        } else {
            1
        };
        let Some(next) = id.checked_add(u64::from(count)) else {
            session.status = "Authoring IDs exhausted".into();
            return;
        };
        session.next_id = next;
        let edit = if session.plant_scatter {
            Edit::ScatterVegetation {
                scatter: Scatter {
                    first_id: id,
                    seed: id,
                    kind,
                    centre: [position[0], position[2]],
                    radius_metres: session.scatter_radius,
                    count,
                },
            }
        } else {
            Edit::PutPlant {
                plant: Plant {
                    id,
                    name: format!("{kind:?} {id}"),
                    kind,
                    position,
                    yaw_degrees: 0.,
                    scale: 1.,
                },
            }
        };
        session.edit(vec![edit]);
        if session.editor.zone().plants.iter().any(|p| p.id == id) {
            session.plant_selected = Some(id);
        }
    } else if mouse.just_pressed(MouseButton::Left) {
        let filter = |e| plants.contains(e) || objects.contains(e) || ground.contains(e);
        let settings = MeshRayCastSettings::default().with_filter(&filter);
        session.plant_selected = raycast
            .cast_ray(ray, &settings)
            .first()
            .and_then(|(e, _)| plants.get(*e).ok())
            .map(|p| p.0);
    }
}
pub(super) fn keyboard(keys: &ButtonInput<KeyCode>, session: &mut Session) {
    if session.play.is_some()
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
    {
        return;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        session.scatter_radius = (session.scatter_radius - 1.).max(1.);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        session.scatter_radius = (session.scatter_radius + 1.).min(32.);
    }
    if keys.just_pressed(KeyCode::Comma) {
        session.scatter_count = session.scatter_count.saturating_sub(4).max(1);
    }
    if keys.just_pressed(KeyCode::Period) {
        session.scatter_count = (session.scatter_count + 4).min(64);
    }
    let Some(plant) = session
        .editor
        .zone()
        .plants
        .iter()
        .find(|p| Some(p.id) == session.plant_selected)
    else {
        return;
    };
    let mut p = plant.clone();
    let increment = if session.snap { session.move_step } else { 0.1 };
    for (key, axis, sign) in [
        (KeyCode::ArrowLeft, 0, -1.),
        (KeyCode::ArrowRight, 0, 1.),
        (KeyCode::ArrowUp, 2, -1.),
        (KeyCode::ArrowDown, 2, 1.),
        (KeyCode::PageUp, 1, 1.),
        (KeyCode::PageDown, 1, -1.),
    ] {
        if keys.just_pressed(key) {
            p.position[axis] += increment * sign;
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        p.yaw_degrees += if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            -15.
        } else {
            15.
        };
    }
    if keys.just_pressed(KeyCode::Equal) {
        p.scale *= 1.25;
    }
    if keys.just_pressed(KeyCode::Minus) {
        p.scale /= 1.25;
    }
    if p != *plant {
        session.edit(vec![Edit::PutPlant { plant: p }]);
    }
}
pub(super) fn summary(session: &Session) -> String {
    let p = session
        .editor
        .zone()
        .plants
        .iter()
        .find(|p| Some(p.id) == session.plant_selected);
    format!(
        "{} plants | scatter {} / {:.1} m\n{}",
        session.editor.zone().plants.len(),
        session.scatter_count,
        session.scatter_radius,
        p.map_or("No plant selected".into(), |p| format!(
            "#{} {:?} | scale {:.2}\npos {:.2?} | yaw {:.0}",
            p.id, p.kind, p.scale, p.position, p.yaw_degrees
        ))
    )
}
