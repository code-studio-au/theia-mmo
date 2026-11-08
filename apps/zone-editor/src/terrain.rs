//! Bounded height-patch authoring through the same command engine as agent edits.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
};
use theia_zone::terrain::{Brush, BrushOperation, HeightPatch};
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Tool {
    Raise,
    Lower,
    Flatten,
    Smooth,
}
pub(super) struct Stroke {
    pub draft: Editor,
    revision: u64,
    edits: Vec<Edit>,
    last: Option<Vec3>,
}
#[derive(Component)]
pub(super) struct GroundPart(pub usize);
#[derive(Component)]
pub(super) struct BrushCursor;
#[derive(Resource)]
pub(super) struct GroundMesh {
    surface: Handle<Mesh>,
    lines: Handle<Mesh>,
}
#[derive(Component)]
pub(super) struct TerrainGrid;
pub(super) fn actions() -> [(Action, &'static str); 12] {
    [
        (Action::TerrainToggle, "Create / remove terrain"),
        (Action::Brush(Tool::Raise), "Raise (2 m / stamp)"),
        (Action::Brush(Tool::Lower), "Lower (2 m / stamp)"),
        (Action::Brush(Tool::Flatten), "Flatten to 0 m"),
        (Action::Brush(Tool::Smooth), "Smooth heights"),
        (Action::Radius(1), "Larger brush (+4 m)"),
        (Action::Radius(-1), "Smaller brush (-4 m)"),
        (Action::Undo, "Undo [Ctrl Z]"),
        (Action::Redo, "Redo [Ctrl Y]"),
        (Action::Save, "Save zone [Ctrl S]"),
        (Action::Load, "Load zone [Ctrl O]"),
        (Action::Play, "Enter / leave playtest [F5]"),
    ]
}
pub(super) fn geometry(zone: &Zone) -> Option<theia_network::procedural::Geometry> {
    let patch = zone.terrain.as_ref()?;
    let start = patch.origin().expect("Validated patch");
    let offset = zone
        .local([start[0], zone.origin[1], start[2]])
        .expect("Validated local patch");
    let mut geometry = theia_network::procedural::grid(32);
    for (i, point) in geometry.positions.iter_mut().enumerate() {
        point[0] += offset[0];
        point[1] = patch.heights[i];
        point[2] += offset[2];
    }
    Some(geometry)
}
/// Flat surroundings have a real cutout; negative sculpting has no hidden floor.
pub(super) fn ground_boxes(zone: &Zone) -> Vec<(Vec3, Vec3)> {
    let Some(patch) = &zone.terrain else {
        return vec![(Vec3::new(0., -0.5, 0.), Vec3::new(1024., 1., 1024.))];
    };
    let start = patch.origin().expect("Validated patch");
    let start = zone
        .local([start[0], zone.origin[1], start[2]])
        .expect("Validated patch frame");
    let (x, z) = (start[0], start[2]);
    [
        (-512., x, -512., 512.),
        (x + 128., 512., -512., 512.),
        (x, x + 128., -512., z),
        (x, x + 128., z + 128., 512.),
    ]
    .into_iter()
    .filter(|(a, b, c, d)| b - a > 0. && d - c > 0.)
    .map(|(a, b, c, d)| {
        (
            Vec3::new((a + b) / 2., -0.5, (c + d) / 2.),
            Vec3::new(b - a, 1., d - c),
        )
    })
    .collect()
}
fn mesh(geometry: theia_network::procedural::Geometry) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, geometry.positions);
    mesh.insert_indices(Indices::U32(
        geometry.triangles.into_iter().flatten().collect(),
    ));
    mesh.compute_normals();
    mesh
}
fn lines(geometry: &theia_network::procedural::Geometry) -> Mesh {
    let mut positions = Vec::new();
    let mut edge = |a: usize, b: usize| {
        for index in [a, b] {
            let mut point = geometry.positions[index];
            point[1] += 0.018;
            positions.push(point);
        }
    };
    for z in 0..33 {
        for x in 0..33 {
            let index = z * 33 + x;
            if x < 32 {
                edge(index, index + 1);
            }
            if z < 32 {
                edge(index, index + 33);
            }
        }
    }
    let normals = vec![[0., 1., 0.]; positions.len()];
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(Color::srgb(0.18, 0.24, 0.20));
    let cube = meshes.add(Cuboid::new(1., 1., 1.));
    for i in 0..4 {
        commands.spawn((
            GroundPart(i),
            Mesh3d(cube.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    let terrain = meshes.add(mesh(theia_network::procedural::grid(32)));
    commands.spawn((
        GroundPart(4),
        Mesh3d(terrain.clone()),
        MeshMaterial3d(material),
        Transform::default(),
        Visibility::Hidden,
    ));
    let grid = meshes.add(lines(&theia_network::procedural::grid(32)));
    commands.spawn((
        TerrainGrid,
        Mesh3d(grid.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.34, 0.45, 0.37),
            unlit: true,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
    ));
    commands.insert_resource(GroundMesh {
        surface: terrain,
        lines: grid,
    });
    commands.spawn((
        BrushCursor,
        Mesh3d(meshes.add(Cylinder::new(1., 0.025))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1., 0.65, 0.12, 0.3),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
    ));
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn sync(
    session: Res<Session>,
    handle: Res<GroundMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ground: Query<(&GroundPart, &mut Transform, &mut Visibility), Without<BrushCursor>>,
    mut cursor: Query<(&mut Transform, &mut Visibility), (With<BrushCursor>, Without<GroundPart>)>,
    mut grid: Query<
        &mut Visibility,
        (
            With<viewport::GridLine>,
            Without<GroundPart>,
            Without<BrushCursor>,
            Without<TerrainGrid>,
        ),
    >,
    mut terrain_grid: Query<
        &mut Visibility,
        (
            With<TerrainGrid>,
            Without<GroundPart>,
            Without<BrushCursor>,
            Without<viewport::GridLine>,
        ),
    >,
    mut last: Local<Option<(u64, Option<u64>)>>,
) {
    for mut visibility in &mut grid {
        *visibility = if session.editor.zone().terrain.is_some() {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    for mut visibility in &mut terrain_grid {
        *visibility = if session.editor.zone().terrain.is_some() && session.play.is_none() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let key = (
        session.editor.revision(),
        session.stroke.as_ref().map(|s| s.draft.revision()),
    );
    if *last != Some(key) {
        let zone = session
            .stroke
            .as_ref()
            .map_or(session.editor.zone(), |s| s.draft.zone());
        let boxes = ground_boxes(zone);
        let geometry = geometry(zone);
        let has_terrain = geometry.is_some();
        if let Some(geometry) = geometry {
            *meshes
                .get_mut(&handle.lines)
                .expect("Resident terrain grid") = lines(&geometry);
            *meshes
                .get_mut(&handle.surface)
                .expect("Resident terrain mesh") = mesh(geometry);
        }
        for (part, mut transform, mut visibility) in &mut ground {
            if part.0 == 4 {
                *transform = Transform::default();
                *visibility = if has_terrain {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            } else if let Some((centre, size)) = boxes.get(part.0) {
                *transform = Transform::from_translation(*centre).with_scale(*size);
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
        *last = Some(key);
    }
    if let Ok((mut transform, mut visibility)) = cursor.single_mut() {
        *visibility =
            if session.terrain_mode
                && session.play.is_none()
                && let Some(point) = session.brush_point
            {
                *transform = Transform::from_translation(point + Vec3::Y * 0.08).with_scale(
                    Vec3::new(session.brush_radius as f32, 1., session.brush_radius as f32),
                );
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
    }
}
pub(super) fn palette(
    session: Res<Session>,
    mut buttons: Query<(
        &PaletteSlot,
        &mut Action,
        &Children,
        &Hovered,
        &mut BackgroundColor,
    )>,
    mut labels: Query<&mut Text, With<PaletteLabel>>,
    mut help: Query<&mut Text, (With<Help>, Without<PaletteLabel>)>,
) {
    for mut text in &mut help {
        **text = if session.marker_mode {
            "F4: NPC / quest markers\nClick to place or select\nArrows/PgUp/Dn: move\nR / Shift R: yaw +/-15 deg\n+/- changes footprint radius\nCtrl O loads saved zone\nMarkers are authoring notes;\nplaytest hides them".into()
        } else if session.path_mode {
            "F3: guide paths / streams\nClick ground to add points\nEnter finishes | Esc cancels\nBksp removes last point\nArrows/PgUp/Dn move guide\n+/- changes width\nRoad/stream guides do not\nadd collision or water rules".into()
        } else if session.terrain_mode {
            "F1: object tools | F2: terrain
Click/drag: brush stroke
Release: one undo entry
Esc: cancel preview
WASD pan | RMB orbit
Wheel zoom | F5 playtest
Borders locked flat
Offline collision sandbox"
                .into()
        } else {
            "WASD pan | RMB orbit | wheel zoom
Drag selected: X/Z
Arrows: X/Z | PgUp/Dn: Y
R / Shift R: yaw +/-15 deg
+ / -: uniform size
Play: W/S forward/back | A/D turn
Q/E strafe | Space jump
Offline collision sandbox"
                .into()
        };
    }
    let actions = if session.marker_mode {
        markers::actions()
    } else if session.path_mode {
        paths::actions()
    } else if session.terrain_mode {
        actions()
    } else {
        object_actions()
    };
    for (slot, mut action, children, hovered, mut color) in &mut buttons {
        *action = actions[slot.0].0;
        let active = match *action {
            Action::Brush(tool) => session.terrain_mode && tool == session.brush_tool,
            Action::NewPath(kind) => session
                .path_draft
                .as_ref()
                .is_some_and(|draft| draft.path.kind == kind),
            Action::PlaceMarker(kind) => session.marker_palette == Some(kind),
            Action::SelectMarker => session.marker_palette.is_none(),
            Action::Place(primitive) => session.palette == Some(primitive),
            Action::Select => session.palette.is_none(),
            _ => false,
        };
        color.0 = if active || hovered.0 {
            Color::srgb(0.20, 0.30, 0.38)
        } else {
            Color::srgb(0.10, 0.16, 0.22)
        };
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                **text = actions[slot.0].1.into();
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ground: Query<(), With<GroundPart>>,
    mut raycast: MeshRayCast,
    mut session: ResMut<Session>,
) {
    session.brush_point = None;
    if !session.terrain_mode {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused || session.play.is_some() {
        session.stroke = None;
        return;
    }
    let Some(cursor) = window.cursor_position().filter(|p| p.x > PANEL) else {
        if mouse.just_released(MouseButton::Left) {
            session.stroke = None;
        }
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
    session.brush_point = point;
    if mouse.just_pressed(MouseButton::Left) && session.editor.zone().terrain.is_some() {
        session.stroke = Some(Stroke {
            draft: Editor::new(session.editor.zone().clone()).expect("Validated draft"),
            revision: session.editor.revision(),
            edits: vec![],
            last: None,
        });
    }
    if let Some(mut stroke) = session.stroke.take() {
        if mouse.pressed(MouseButton::Left)
            && let Some(point) = point
            && stroke.edits.len() < theia_zone::MAX_EDITS
            && stroke
                .last
                .is_none_or(|last| (last.xz() - point.xz()).length() >= 2.)
        {
            let zone = stroke.draft.zone();
            let global = zone.global(point.to_array()).expect("Bounded terrain hit");
            let centre = [global[0], global[2]];
            let operation = match session.brush_tool {
                Tool::Raise => BrushOperation::Raise { metres: 2. },
                Tool::Lower => BrushOperation::Lower { metres: 2. },
                Tool::Flatten => BrushOperation::Flatten {
                    height: 0.,
                    blend: 0.5,
                },
                Tool::Smooth => BrushOperation::Smooth { blend: 0.5 },
            };
            let edit = Edit::Brush {
                brush: Brush {
                    centre,
                    radius_metres: session.brush_radius,
                    operation,
                },
            };
            match stroke.draft.execute(Command::Apply {
                expected_revision: stroke.draft.revision(),
                edits: vec![edit.clone()],
            }) {
                Ok(()) => {
                    stroke.edits.push(edit);
                    stroke.last = Some(point);
                    session.status = "Terrain preview - release to commit, Esc to cancel".into();
                }
                Err(error) => {
                    stroke.last = Some(point);
                    session.status = error;
                }
            }
        }
        if mouse.just_released(MouseButton::Left) {
            if !stroke.edits.is_empty() {
                session.execute(Command::Apply {
                    expected_revision: stroke.revision,
                    edits: stroke.edits,
                });
            }
        } else if mouse.pressed(MouseButton::Left) {
            session.stroke = Some(stroke);
        }
    }
}
pub(super) fn toggle(session: &mut Session) {
    let terrain = if session.editor.zone().terrain.is_some() {
        None
    } else {
        let zone = session.editor.zone();
        let chunk = theia_core::coordinates::GlobalPosition::new(zone.origin)
            .unwrap()
            .chunk(128)
            .unwrap();
        Some(HeightPatch::flat([chunk.x, chunk.z]))
    };
    session.edit(vec![Edit::SetTerrain { terrain }]);
}
