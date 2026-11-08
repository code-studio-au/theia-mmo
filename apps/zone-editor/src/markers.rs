//! Nonblocking authoring annotations. No runtime NPCs, quests or triggers are spawned.
use super::*;
use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings};
use std::collections::BTreeMap;
use theia_zone::markers::{Marker, MarkerKind};
#[derive(Component)]
pub(super) struct AuthoredMarker(u64);
#[derive(Resource)]
pub(super) struct Scene {
    meshes: [Handle<Mesh>; 3],
    npc: Handle<StandardMaterial>,
    quest: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
    cache: BTreeMap<u64, [Entity; 3]>,
    signature: Option<(u64, Option<u64>, bool)>,
}
pub(super) fn actions() -> [(Action, &'static str); 12] {
    [
        (Action::SelectMarker, "Select marker [Esc]"),
        (Action::PlaceMarker(MarkerKind::Npc), "Place NPC marker"),
        (Action::PlaceMarker(MarkerKind::Quest), "Place quest marker"),
        (Action::NextMarker, "Select next marker"),
        (Action::DeleteMarker, "Delete marker [Del]"),
        (Action::MarkerRadius(1), "Larger footprint (+0.5 m)"),
        (Action::MarkerRadius(-1), "Smaller footprint (-0.5 m)"),
        (Action::Snap, "Toggle 1 m snap [G]"),
        (Action::Undo, "Undo [Ctrl Z]"),
        (Action::Redo, "Redo [Ctrl Y]"),
        (Action::Save, "Save zone [Ctrl S]"),
        (Action::Play, "Enter / leave playtest [F5]"),
    ]
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = |color| StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };
    commands.insert_resource(Scene {
        meshes: [
            meshes.add(Cylinder::new(1., 0.03)),
            meshes.add(Capsule3d::new(0.3, 1.2)),
            meshes.add(Cuboid::new(0.12, 0.12, 0.8)),
        ],
        npc: materials.add(material(Color::srgba(0.05, 0.7, 0.9, 0.5))),
        quest: materials.add(material(Color::srgba(0.9, 0.2, 0.65, 0.5))),
        selected: materials.add(material(Color::srgba(1., 0.65, 0.1, 0.7))),
        cache: BTreeMap::new(),
        signature: None,
    });
}
pub(super) fn sync(mut commands: Commands, session: Res<Session>, mut scene: ResMut<Scene>) {
    let signature = (
        session.editor.revision(),
        session.marker_selected.filter(|_| session.marker_mode),
        session.play.is_some(),
    );
    if scene.signature == Some(signature) {
        return;
    }
    let removed: Vec<_> = scene
        .cache
        .keys()
        .filter(|id| !session.editor.zone().markers.iter().any(|m| m.id == **id))
        .copied()
        .collect();
    for id in removed {
        for entity in scene.cache.remove(&id).unwrap() {
            commands.entity(entity).despawn();
        }
    }
    for marker in &session.editor.zone().markers {
        let material = if signature.1 == Some(marker.id) {
            scene.selected.clone()
        } else if marker.kind == MarkerKind::Npc {
            scene.npc.clone()
        } else {
            scene.quest.clone()
        };
        let p = Vec3::from_array(session.editor.zone().local(marker.position).unwrap());
        let yaw = Quat::from_rotation_y(marker.yaw_degrees.to_radians() as f32);
        let footprint = Transform::from_translation(p + Vec3::Y * 0.03).with_scale(Vec3::new(
            marker.radius_metres as f32,
            1.,
            marker.radius_metres as f32,
        ));
        // A capsule is an authoring badge; the quest badge is shorter and wider.
        let badge = Transform::from_translation(
            p + Vec3::Y
                * if marker.kind == MarkerKind::Npc {
                    0.9
                } else {
                    0.4
                },
        )
        .with_scale(if marker.kind == MarkerKind::Npc {
            Vec3::ONE
        } else {
            Vec3::new(1.5, 0.4, 1.5)
        });
        let arrow = Transform::from_translation(p + Vec3::Y * 0.6 + yaw * Vec3::new(0., 0., -0.7))
            .with_rotation(yaw);
        let visibility = if signature.2 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if let Some(entities) = scene.cache.get(&marker.id) {
            for (entity, transform) in entities.iter().zip([footprint, badge, arrow]) {
                commands.entity(*entity).insert((
                    transform,
                    MeshMaterial3d(material.clone()),
                    visibility,
                ));
            }
        } else {
            let entities = std::array::from_fn(|i| {
                commands
                    .spawn((
                        AuthoredMarker(marker.id),
                        Mesh3d(scene.meshes[i].clone()),
                        MeshMaterial3d(material.clone()),
                        [footprint, badge, arrow][i],
                        visibility,
                        bevy::light::NotShadowCaster,
                    ))
                    .id()
            });
            scene.cache.insert(marker.id, entities);
        }
    }
    scene.signature = Some(signature);
}
pub(super) fn perform(action: Action, session: &mut Session) {
    match action {
        Action::SelectMarker => session.marker_palette = None,
        Action::PlaceMarker(kind) => {
            session.marker_palette = Some(kind);
            session.marker_selected = None;
            session.status = "Click ground to place an authoring marker".into();
        }
        Action::NextMarker => {
            session.marker_palette = None;
            let markers = &session.editor.zone().markers;
            session.marker_selected = if markers.is_empty() {
                None
            } else {
                Some(
                    markers[markers
                        .iter()
                        .position(|m| Some(m.id) == session.marker_selected)
                        .map_or(0, |i| (i + 1) % markers.len())]
                    .id,
                )
            };
        }
        Action::DeleteMarker => {
            if let Some(id) = session.marker_selected {
                session.edit(vec![Edit::DeleteMarker { id }]);
            }
        }
        Action::MarkerRadius(sign) => {
            if let Some(m) = session
                .editor
                .zone()
                .markers
                .iter()
                .find(|m| Some(m.id) == session.marker_selected)
            {
                let mut marker = m.clone();
                marker.radius_metres =
                    (marker.radius_metres + f64::from(sign) * 0.5).clamp(0.25, 64.);
                session.edit(vec![Edit::PutMarker { marker }]);
            }
        }
        _ => unreachable!("Marker action dispatch"),
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    markers: Query<&AuthoredMarker>,
    objects: Query<(), With<viewport::Authored>>,
    ground: Query<(), With<terrain::GroundPart>>,
    mut raycast: MeshRayCast,
    mut session: ResMut<Session>,
) {
    if !session.marker_mode || session.play.is_some() || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused {
        return;
    }
    let Some(cursor) = window.cursor_position().filter(|p| p.x > PANEL) else {
        return;
    };
    let Ok((camera, transform)) = camera.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };
    if let Some(kind) = session.marker_palette {
        let filter = |e| ground.contains(e);
        let settings = MeshRayCastSettings::default().with_filter(&filter);
        let Some(point) = raycast
            .cast_ray(ray, &settings)
            .first()
            .map(|(_, hit)| ray.get_point(hit.distance))
        else {
            return;
        };
        let mut position = session.snapped(session.editor.zone().global(point.to_array()).unwrap());
        let height = session
            .editor
            .zone()
            .terrain
            .as_ref()
            .and_then(|p| p.height(position[0], position[2]).ok().flatten())
            .unwrap_or(0.);
        position[1] = session.editor.zone().origin[1] + f64::from(height);
        let id = session.next_id;
        let Some(next) = id.checked_add(1) else {
            session.status = "Authoring IDs exhausted".into();
            return;
        };
        session.next_id = next;
        session.edit(vec![Edit::PutMarker {
            marker: Marker {
                id,
                name: format!("{kind:?} marker {id}"),
                kind,
                position,
                yaw_degrees: 0.,
                radius_metres: if kind == MarkerKind::Npc { 1. } else { 4. },
                definition: None,
            },
        }]);
        if session.editor.zone().markers.iter().any(|m| m.id == id) {
            session.marker_selected = Some(id);
        }
    } else {
        let filter = |e| markers.contains(e) || ground.contains(e) || objects.contains(e);
        let settings = MeshRayCastSettings::default().with_filter(&filter);
        session.marker_selected = raycast
            .cast_ray(ray, &settings)
            .first()
            .and_then(|(e, _)| markers.get(*e).ok())
            .map(|m| m.0);
    }
}
pub(super) fn transform(keys: &ButtonInput<KeyCode>, session: &mut Session) {
    if session.play.is_some()
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
    {
        return;
    }
    let Some(marker) = session
        .editor
        .zone()
        .markers
        .iter()
        .find(|m| Some(m.id) == session.marker_selected)
    else {
        return;
    };
    let mut next = marker.clone();
    let increment = if session.snap { 1. } else { 0.1 };
    for (key, axis, sign) in [
        (KeyCode::ArrowLeft, 0, -1.),
        (KeyCode::ArrowRight, 0, 1.),
        (KeyCode::ArrowUp, 2, -1.),
        (KeyCode::ArrowDown, 2, 1.),
        (KeyCode::PageUp, 1, 1.),
        (KeyCode::PageDown, 1, -1.),
    ] {
        if keys.just_pressed(key) {
            next.position[axis] += increment * sign;
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        next.yaw_degrees += if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
        {
            -15.
        } else {
            15.
        };
    }
    if next != *marker {
        session.edit(vec![Edit::PutMarker { marker: next }]);
    }
}
pub(super) fn summary(session: &Session) -> String {
    let selected = session
        .editor
        .zone()
        .markers
        .iter()
        .find(|m| Some(m.id) == session.marker_selected);
    format!("{} markers\n{}",session.editor.zone().markers.len(),selected.map_or("No marker selected".into(), |m| format!("#{} {:?}: {}\npos {:.2?} | yaw {:.0}\nradius {} m\nDefinition: {}\nAuthoring annotation",m.id,m.kind,m.name,m.position,m.yaw_degrees,m.radius_metres,m.definition.as_deref().unwrap_or("unbound"))))
}
