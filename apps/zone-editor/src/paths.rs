//! Visual road/stream guides. Every committed edit uses the shared zone commands.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
};
use std::collections::{BTreeMap, BTreeSet};
use theia_zone::paths::{GuidePath, PathKind};
pub(super) struct Draft {
    pub path: GuidePath,
    revision: u64,
}
#[derive(Resource)]
pub(super) struct Scene {
    road: Handle<StandardMaterial>,
    stream: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
    cache: BTreeMap<u64, (Entity, Handle<Mesh>)>,
    revision: Option<u64>,
    selection: Option<u64>,
    draft_entity: Entity,
    draft_mesh: Handle<Mesh>,
    draft: Option<GuidePath>,
    anchor: Entity,
}
pub(super) fn actions() -> [(Action, &'static str); 12] {
    [
        (Action::NewPath(PathKind::Road), "New road guide"),
        (Action::NewPath(PathKind::Stream), "New stream guide"),
        (Action::FinishPath, "Finish guide [Enter]"),
        (Action::GradePath, "Grade terrain to guide"),
        (Action::NextPath, "Select next guide"),
        (Action::DeletePath, "Delete guide [Del]"),
        (Action::PathWidth(1), "Wider guide (+0.5 m)"),
        (Action::PathWidth(-1), "Narrower guide (-0.5 m)"),
        (Action::Undo, "Undo [Ctrl Z]"),
        (Action::Redo, "Redo [Ctrl Y]"),
        (Action::Save, "Save zone [Ctrl S]"),
        (Action::Play, "Enter / leave playtest [F5]"),
    ]
}
/// Straight, explicit-height guide ribbon. It is render-only, never world collision.
pub(super) fn ribbon(zone: &Zone, path: &GuidePath) -> Mesh {
    let mut vertices = vec![];
    let mut indices = vec![];
    for segment in path.points.windows(2) {
        let a = Vec3::from_array(zone.local(segment[0]).expect("Validated guide point"));
        let b = Vec3::from_array(zone.local(segment[1]).expect("Validated guide point"));
        let delta = b - a;
        let side = Vec3::new(delta.z, 0., -delta.x).normalize() * (path.width_metres as f32 / 2.);
        let start = vertices.len() as u32;
        vertices.extend([a - side, a + side, b - side, b + side].map(|v| v.to_array()));
        indices.extend([start, start + 2, start + 1, start + 1, start + 2, start + 3]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();
    mesh
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let road = materials.add(Color::srgb(0.48, 0.31, 0.15));
    let stream = materials.add(StandardMaterial {
        base_color: Color::srgba(0.04, 0.36, 0.75, 0.8),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let selected = materials.add(StandardMaterial {
        base_color: Color::srgba(1., 0.68, 0.16, 0.8),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let draft_mesh = meshes.add(Cuboid::new(1., 0.01, 1.));
    let draft_entity = commands
        .spawn((
            Mesh3d(draft_mesh.clone()),
            MeshMaterial3d(selected.clone()),
            Transform::default(),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
        ))
        .id();
    let anchor = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(selected.clone()),
            Transform::default(),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
        ))
        .id();
    commands.insert_resource(Scene {
        road,
        stream,
        selected,
        cache: BTreeMap::new(),
        revision: None,
        selection: None,
        draft_entity,
        draft_mesh,
        draft: None,
        anchor,
    });
}
pub(super) fn sync(
    mut commands: Commands,
    session: Res<Session>,
    mut scene: ResMut<Scene>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if scene.revision != Some(session.editor.revision()) {
        let keep: BTreeSet<_> = session.editor.zone().paths.iter().map(|p| p.id).collect();
        let removed: Vec<_> = scene
            .cache
            .keys()
            .filter(|id| !keep.contains(id))
            .copied()
            .collect();
        for id in removed {
            if let Some((entity, handle)) = scene.cache.remove(&id) {
                commands.entity(entity).despawn();
                meshes.remove(handle.id());
            }
        }
        for path in &session.editor.zone().paths {
            let mesh = ribbon(session.editor.zone(), path);
            if let Some((_, handle)) = scene.cache.get(&path.id) {
                *meshes.get_mut(handle).expect("Resident guide mesh") = mesh;
            } else {
                let handle = meshes.add(mesh);
                let material = if path.kind == PathKind::Road {
                    scene.road.clone()
                } else {
                    scene.stream.clone()
                };
                let entity = commands
                    .spawn((
                        Mesh3d(handle.clone()),
                        MeshMaterial3d(material),
                        Transform::default(),
                        bevy::light::NotShadowCaster,
                    ))
                    .id();
                scene.cache.insert(path.id, (entity, handle));
            }
        }
    }
    let selected = session.path_selected.filter(|_| session.path_mode);
    if scene.revision != Some(session.editor.revision()) || scene.selection != selected {
        for path in &session.editor.zone().paths {
            let material = if session.path_selected == Some(path.id) && session.path_mode {
                scene.selected.clone()
            } else if path.kind == PathKind::Road {
                scene.road.clone()
            } else {
                scene.stream.clone()
            };
            let entity = scene.cache[&path.id].0;
            commands.entity(entity).insert(MeshMaterial3d(material));
        }
        scene.revision = Some(session.editor.revision());
        scene.selection = selected;
    }
    let draft = session.path_draft.as_ref().map(|d| d.path.clone());
    if scene.draft != draft {
        if let Some(path) = &draft {
            if path.points.len() >= 2 {
                *meshes.get_mut(&scene.draft_mesh).expect("Draft guide mesh") =
                    ribbon(session.editor.zone(), path);
                commands
                    .entity(scene.draft_entity)
                    .insert(Visibility::Visible);
            } else {
                commands
                    .entity(scene.draft_entity)
                    .insert(Visibility::Hidden);
            }
            if let Some(&point) = path.points.last() {
                commands.entity(scene.anchor).insert((
                    Transform::from_translation(Vec3::from_array(
                        session.editor.zone().local(point).unwrap(),
                    )),
                    Visibility::Visible,
                ));
            } else {
                commands.entity(scene.anchor).insert(Visibility::Hidden);
            }
        } else {
            commands
                .entity(scene.draft_entity)
                .insert(Visibility::Hidden);
            commands.entity(scene.anchor).insert(Visibility::Hidden);
        }
        scene.draft = draft;
    }
}
pub(super) fn perform(action: Action, session: &mut Session) {
    match action {
        Action::NewPath(kind) => {
            let id = session.next_id;
            let Some(next) = id.checked_add(1) else {
                session.status = "Authoring IDs exhausted".into();
                return;
            };
            session.next_id = next;
            session.path_selected = None;
            session.path_draft = Some(Draft {
                path: GuidePath {
                    id,
                    name: format!("{kind:?} {id}"),
                    kind,
                    width_metres: if kind == PathKind::Road { 4. } else { 2. },
                    points: vec![],
                },
                revision: session.editor.revision(),
            });
            session.status = "Click ground points, Enter to finish, Esc to cancel".into();
        }
        Action::FinishPath => {
            if let Some(draft) = session.path_draft.take() {
                if draft.path.points.len() < 2 {
                    session.status = "Add at least two guide points".into();
                    session.path_draft = Some(draft);
                    return;
                }
                let id = draft.path.id;
                session.execute(Command::Apply {
                    expected_revision: draft.revision,
                    edits: vec![Edit::PutPath { path: draft.path }],
                });
                if session.editor.zone().paths.iter().any(|p| p.id == id) {
                    session.path_selected = Some(id);
                }
            }
        }
        Action::GradePath => {
            if session.path_draft.is_some() {
                session.status = "Finish or cancel guide draft before grading".into();
                return;
            }
            if let Some(path_id) = session.path_selected {
                session.edit(vec![Edit::GradePath {
                    grade: theia_zone::grading::Grade {
                        path_id,
                        surface_offset_metres: -0.03,
                        falloff_metres: 4.,
                        blend: 1.,
                    },
                }]);
            } else {
                session.status = "Select a committed guide to grade terrain".into();
            }
        }
        Action::PopPath => {
            if let Some(draft) = &mut session.path_draft {
                draft.path.points.pop();
            }
        }
        Action::NextPath => {
            session.path_draft = None;
            let paths = &session.editor.zone().paths;
            session.path_selected = if paths.is_empty() {
                None
            } else {
                let index = paths
                    .iter()
                    .position(|p| Some(p.id) == session.path_selected)
                    .map_or(0, |i| (i + 1) % paths.len());
                Some(paths[index].id)
            };
        }
        Action::DeletePath => {
            session.path_draft = None;
            if let Some(id) = session.path_selected {
                session.edit(vec![Edit::DeletePath { id }]);
            }
        }
        Action::PathWidth(direction) => {
            if let Some(draft) = &mut session.path_draft {
                let mut candidate = draft.path.clone();
                candidate.width_metres =
                    (candidate.width_metres + f64::from(direction) * 0.5).clamp(0.25, 32.);
                let result = if candidate.points.len() >= 2 {
                    Editor::new(session.editor.zone().clone()).and_then(|mut e| {
                        e.execute(Command::Apply {
                            expected_revision: 0,
                            edits: vec![Edit::PutPath {
                                path: candidate.clone(),
                            }],
                        })
                    })
                } else {
                    Ok(())
                };
                match result {
                    Ok(()) => draft.path = candidate,
                    Err(error) => session.status = error,
                }
            } else if let Some(path) = session
                .editor
                .zone()
                .paths
                .iter()
                .find(|p| Some(p.id) == session.path_selected)
            {
                let mut path = path.clone();
                path.width_metres =
                    (path.width_metres + f64::from(direction) * 0.5).clamp(0.25, 32.);
                session.edit(vec![Edit::PutPath { path }]);
            }
        }
        _ => unreachable!("Guide action dispatch"),
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ground: Query<(), With<terrain::GroundPart>>,
    mut raycast: MeshRayCast,
    mut session: ResMut<Session>,
) {
    if !session.path_mode || session.play.is_some() {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused {
        session.path_draft = None;
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) || session.path_draft.is_none() {
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
    let filter = |entity| ground.contains(entity);
    let settings = MeshRayCastSettings::default().with_filter(&filter);
    let point = raycast
        .cast_ray(ray, &settings)
        .first()
        .map(|(_, hit)| ray.get_point(hit.distance));
    let Some(point) = point else {
        return;
    };
    let mut position = session.snapped(
        session
            .editor
            .zone()
            .global(point.to_array())
            .expect("Bounded guide ray"),
    );
    let height = session
        .editor
        .zone()
        .terrain
        .as_ref()
        .and_then(|p| p.height(position[0], position[2]).ok().flatten())
        .unwrap_or(0.);
    position[1] = session.editor.zone().origin[1] + f64::from(height) + 0.03;
    let mut draft = session.path_draft.take().unwrap();
    let mut candidate = draft.path.clone();
    candidate.points.push(position);
    let result = if candidate.points.len() >= 2 {
        Editor::new(session.editor.zone().clone()).and_then(|mut editor| {
            editor.execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::PutPath {
                    path: candidate.clone(),
                }],
            })
        })
    } else {
        Ok(())
    };
    match result {
        Ok(()) => {
            draft.path = candidate;
            session.status = format!("{} guide points - Enter finishes", draft.path.points.len());
        }
        Err(error) => session.status = error,
    }
    session.path_draft = Some(draft);
}

pub(super) fn summary(session: &Session) -> String {
    let guide = session.path_draft.as_ref().map(|d| &d.path).or_else(|| {
        session
            .editor
            .zone()
            .paths
            .iter()
            .find(|p| Some(p.id) == session.path_selected)
    });
    format!(
        "{} guides\n{}",
        session.editor.zone().paths.len(),
        guide.map_or("No guide selected".into(), |p| format!(
            "#{} {:?} | {} points\nwidth {:.1} m{}",
            p.id,
            p.kind,
            p.points.len(),
            p.width_metres,
            if session.path_draft.is_some() {
                " | draft"
            } else {
                ""
            }
        ))
    )
}
pub(super) fn translate(keys: &ButtonInput<KeyCode>, session: &mut Session) {
    if session.play.is_some()
        || session.path_draft.is_some()
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
    {
        return;
    }
    let Some(path) = session
        .editor
        .zone()
        .paths
        .iter()
        .find(|p| Some(p.id) == session.path_selected)
    else {
        return;
    };
    let mut delta = [0.; 3];
    let step = if session.snap { 1. } else { 0.1 };
    for (key, axis, sign) in [
        (KeyCode::ArrowLeft, 0, -1.),
        (KeyCode::ArrowRight, 0, 1.),
        (KeyCode::ArrowUp, 2, -1.),
        (KeyCode::ArrowDown, 2, 1.),
        (KeyCode::PageUp, 1, 1.),
        (KeyCode::PageDown, 1, -1.),
    ] {
        if keys.just_pressed(key) {
            delta[axis] += step * sign;
        }
    }
    if delta != [0.; 3] {
        let mut path = path.clone();
        for point in &mut path.points {
            for i in 0..3 {
                point[i] += delta[i];
            }
        }
        session.edit(vec![Edit::PutPath { path }]);
    }
}
