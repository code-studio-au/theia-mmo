//! Transient viewport selection; committed transforms remain ordinary atomic zone edits.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone)]
pub(super) struct Marquee {
    pub start: Vec2,
    pub end: Vec2,
    pub base: BTreeSet<u64>,
    pub toggle: bool,
    pub clicked: Option<u64>,
}

pub(super) fn assets(zone: &Zone) -> Vec<(u64, Pose)> {
    let mut result: Vec<_> = zone
        .objects
        .iter()
        .map(|o| (o.id, o.pose.clone()))
        .collect();
    result.extend(
        zone.scenery
            .iter()
            .map(|p| (p.id, transform_target::scenery_pose(p))),
    );
    result.extend(
        zone.plants
            .iter()
            .map(|p| (p.id, pose(p.position, p.yaw_degrees, p.scale))),
    );
    result.extend(
        zone.markers
            .iter()
            .map(|p| (p.id, pose(p.position, p.yaw_degrees, p.radius_metres))),
    );
    result.extend(
        zone.particles
            .iter()
            .map(|p| (p.id, pose(p.position, p.yaw_degrees, p.scale))),
    );
    result.extend(
        zone.test_players
            .iter()
            .map(|p| (p.id, pose(p.position, 0., 1.))),
    );
    result
}
pub(super) fn pose(position: [f64; 3], yaw_degrees: f64, scale: f64) -> Pose {
    Pose {
        position,
        yaw_degrees,
        size: [scale; 3],
    }
}
pub(super) fn pivot(session: &Session) -> Option<Pose> {
    if session.group.is_empty() {
        return None;
    }
    let selected: Vec<_> = assets(session.editor.zone())
        .into_iter()
        .filter(|(id, _)| session.group.contains(id))
        .collect();
    let first = selected.first()?.1.position;
    let mut min = first;
    let mut max = first;
    for (_, p) in selected {
        for i in 0..3 {
            min[i] = min[i].min(p.position[i]);
            max[i] = max[i].max(p.position[i]);
        }
    }
    Some(pose(
        std::array::from_fn(|i| min[i] + (max[i] - min[i]) * 0.5),
        0.,
        1.,
    ))
}
/// Rotate in the same right-handed Y-up convention as the scene transforms.
fn transformed(original: &Pose, before: &Pose, after: &Pose) -> Pose {
    let factor = after.size[0] / before.size[0];
    let angle = after.yaw_degrees - before.yaw_degrees;
    let (sin, cos) = angle.to_radians().sin_cos();
    let offset: [f64; 3] =
        std::array::from_fn(|i| (original.position[i] - before.position[i]) * factor);
    let offset = [
        cos * offset[0] + sin * offset[2],
        offset[1],
        -sin * offset[0] + cos * offset[2],
    ];
    Pose {
        position: std::array::from_fn(|i| after.position[i] + offset[i]),
        yaw_degrees: original.yaw_degrees + angle,
        size: original.size.map(|v| v * factor),
    }
}
pub(super) fn preview(session: &Session, id: u64, original: &Pose) -> Pose {
    match session.drag.as_ref() {
        Some(d) if d.id == id => d.preview.clone(),
        Some(d) if d.id == 0 && session.group.contains(&id) => {
            transformed(original, &d.before, &d.preview)
        }
        _ => original.clone(),
    }
}
pub(super) fn active(session: &Session) -> bool {
    session.drag.as_ref().is_some_and(|d| d.id == 0)
}
pub(super) fn edits(
    session: &Session,
    id: u64,
    after: &Pose,
    library: Option<&scenery_library::Library>,
) -> Result<Vec<Edit>, String> {
    if id != 0 {
        return transform_target::edit(session.editor.zone(), id, after, library).map(|e| vec![e]);
    }
    if !after.size[0].is_finite() || after.size[0] <= 0. || after.size != [after.size[0]; 3] {
        return Err("Group scale must be positive and uniform".into());
    }
    let before = pivot(session).ok_or("Select assets first")?;
    assets(session.editor.zone())
        .into_iter()
        .filter(|(id, _)| session.group.contains(id))
        .map(|(id, p)| {
            let mut p = transformed(&p, &before, after);
            // Player anchors have no authored size/facing: transform their position only.
            if session
                .editor
                .zone()
                .test_players
                .iter()
                .any(|v| v.id == id)
            {
                p.yaw_degrees = 0.;
                p.size = [1.; 3];
            }
            transform_target::edit(session.editor.zone(), id, &p, library)
        })
        .collect()
}
pub(super) fn current(session: &Session) -> BTreeSet<u64> {
    let mut ids = session.group.clone();
    ids.extend(
        session
            .selected
            .or(session.scenery_selected)
            .or(session.particle_selected)
            .or(session.test_player_selected)
            .or(session.plant_selected)
            .or(session.marker_selected),
    );
    ids
}
pub(super) fn select(session: &mut Session, ids: BTreeSet<u64>) {
    session.set_mode(EditorMode::Objects);
    session.group = ids;
    session.status = format!(
        "{} assets selected · Move / Rotate / Scale · F frames · Esc clears",
        session.group.len()
    );
}
pub(super) fn begin(session: &mut Session, cursor: Vec2, add: bool, toggle: bool) {
    let base = if add || toggle {
        current(session)
    } else {
        BTreeSet::new()
    };
    session.marquee = Some(Marquee {
        start: cursor,
        end: cursor,
        base,
        toggle,
        clicked: None,
    });
}
fn box_ids(marquee: &Marquee, projected: impl Iterator<Item = (u64, Vec2)>) -> BTreeSet<u64> {
    let rect = Rect::from_corners(marquee.start, marquee.end);
    let mut ids = marquee.base.clone();
    for (id, point) in projected {
        if rect.contains(point) && !(marquee.toggle && ids.remove(&id)) {
            ids.insert(id);
        }
    }
    ids
}
/// Called before regular picking so a held marquee also releases outside the viewport.
pub(super) fn update_marquee(
    session: &mut Session,
    window: &Window,
    mouse: &ButtonInput<MouseButton>,
    camera: &Camera,
    transform: &GlobalTransform,
) -> bool {
    let Some(mut marquee) = session.marquee.take() else {
        return false;
    };
    if let Some(p) = window.cursor_position() {
        let min = Vec2::new(session.ui.canvas_left(), workspace::HEADER);
        let max = Vec2::new(
            window.width(),
            window.height() - session.ui.canvas_bottom(window.height(), true),
        );
        marquee.end = session
            .ui
            .window_to_viewport(p.clamp(min, max.max(min)), window.scale_factor());
    }
    if mouse.just_released(MouseButton::Left) {
        // Clicking empty ground clears; moving at least 4 logical pixels selects a box.
        let ids = if marquee.start.distance(marquee.end) < 4. * window.scale_factor() {
            let mut ids = marquee.base.clone();
            if let Some(id) = marquee.clicked
                && !(marquee.toggle && ids.remove(&id))
            {
                ids.insert(id);
            }
            ids
        } else {
            box_ids(
                &marquee,
                assets(session.editor.zone())
                    .into_iter()
                    .filter_map(|(id, p)| {
                        let local = Vec3::from_array(session.editor.zone().local(p.position).ok()?);
                        let point = camera.world_to_viewport(transform, local).ok()?;
                        Some((id, point))
                    }),
            )
        };
        select(session, ids);
    } else if mouse.pressed(MouseButton::Left) {
        session.marquee = Some(marquee);
    }
    true
}
#[derive(Component)]
pub(super) struct Overlay;
pub(super) fn draw(
    mut commands: Commands,
    session: Res<Session>,
    windows: Query<&Window>,
    mut overlay: Query<(&mut Node, &mut Visibility), With<Overlay>>,
    mut gizmos: Gizmos,
    library: Option<Res<scenery_library::Library>>,
) {
    if overlay.is_empty() {
        commands.spawn((
            Overlay,
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(px(1.)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.65, 1., 0.12)),
            BorderColor::all(Color::srgb(0.3, 0.75, 1.)),
            Visibility::Hidden,
            GlobalZIndex(40),
            Pickable::IGNORE,
        ));
    }
    if let Ok((mut node, mut visibility)) = overlay.single_mut() {
        *visibility = Visibility::Hidden;
        if let Some(m) = &session.marquee
            && let Ok(window) = windows.single()
        {
            let min = session
                .ui
                .viewport_to_window(m.start.min(m.end), window.scale_factor());
            let size = (m.end - m.start).abs() / window.scale_factor();
            node.left = px(min.x);
            node.top = px(min.y);
            node.width = px(size.x);
            node.height = px(size.y);
            *visibility = Visibility::Visible;
        }
    }
    if session.play.is_some() {
        return;
    }
    for (id, p) in assets(session.editor.zone())
        .into_iter()
        .filter(|(id, _)| session.group.contains(id))
    {
        let p = preview(&session, id, &p);
        if let Ok(local) = session.editor.zone().local(p.position) {
            let extent =
                transform_target::extent(session.editor.zone(), id, &p, library.as_deref())
                    .max(0.5);
            gizmos.cube(
                Transform::from_translation(Vec3::from_array(local))
                    .with_rotation(Quat::from_rotation_y(p.yaw_degrees.to_radians() as f32))
                    .with_scale(Vec3::splat(extent * 2.)),
                Color::srgb(1., 0.7, 0.12),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(origin: [f64; 3]) -> (Session, scenery_library::Library) {
        let (session, library) = transform_target::tests::fixture(origin);
        let mut zone = session.editor.zone().clone();
        zone.objects.push(Object {
            id: 2,
            name: "Block".into(),
            primitive: Primitive::Box,
            pose: pose([origin[0] - 2.125, origin[1], origin[2] + 3.25], 12., 2.),
        });
        zone.markers.push(theia_zone::markers::Marker {
            id: 3,
            name: "Guide".into(),
            kind: theia_zone::markers::MarkerKind::Npc,
            position: origin,
            yaw_degrees: 10.,
            radius_metres: 1.,
            definition: Some("npc.guide".into()),
        });
        zone.plants.push(theia_zone::vegetation::Plant {
            id: 4,
            name: "Tree".into(),
            kind: theia_zone::vegetation::PlantKind::Tree,
            position: origin,
            yaw_degrees: 30.,
            scale: 1.,
        });
        zone.particles
            .push(theia_zone::particles::ParticlePlacement {
                id: 5,
                name: "Fire".into(),
                effect: "vfx.fire".into(),
                position: origin,
                yaw_degrees: 20.,
                scale: 1.,
            });
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            "unused.json".into(),
            Some(zone),
        );
        select(&mut session, (1..=5).collect());
        (session, library)
    }
    #[test]
    fn group_move_rotate_scale_share_pivot_preserve_bindings_and_undo_exactly() {
        for origin in [[0.; 3], [128_000_000_000.125, 0.25, -128_000_000_000.5]] {
            for operation in 0..3 {
                let (mut session, library) = fixture(origin);
                let source = session.editor.zone().clone();
                let before = pivot(&session).unwrap();
                assert_eq!(before.position, origin);
                let mut after = before.clone();
                match operation {
                    0 => after.position = std::array::from_fn(|i| origin[i] + [8., 2., -4.][i]),
                    1 => after.yaw_degrees = 90.,
                    _ => after.size = [2.; 3],
                }
                let batch = edits(&session, 0, &after, Some(&library)).unwrap();
                session.edit(batch);
                assert_eq!(session.editor.revision(), 1, "{}", session.status);
                let result = session.editor.zone().clone();
                let block = &result.objects[0].pose;
                let expected = match operation {
                    0 => [origin[0] + 5.875, origin[1] + 2., origin[2] - 0.75],
                    1 => [origin[0] + 3.25, origin[1], origin[2] + 2.125],
                    _ => [origin[0] - 4.25, origin[1], origin[2] + 6.5],
                };
                for (actual, expected) in block.position.into_iter().zip(expected) {
                    assert!((actual - expected).abs() < 0.0001);
                }
                assert_eq!(block.size, if operation == 2 { [4.; 3] } else { [2.; 3] });
                assert_eq!(block.yaw_degrees, if operation == 1 { 102. } else { 12. });
                assert_eq!(result.markers[0].definition, source.markers[0].definition);
                assert_eq!(result.scenery[0].asset, source.scenery[0].asset);
                assert_eq!(result.particles[0].effect, source.particles[0].effect);
                session.execute(Command::Undo {
                    expected_revision: 1,
                });
                assert_eq!(session.editor.zone(), &source);
                session.execute(Command::Redo {
                    expected_revision: 2,
                });
                assert_eq!(session.editor.zone(), &result);
                assert_eq!(
                    Zone::from_json(&serde_json::to_vec(&result).unwrap()).unwrap(),
                    result
                );
            }
        }
    }
    #[test]
    fn invalid_member_rejects_whole_group_without_history_or_partial_movement() {
        let (mut session, library) = fixture([0.; 3]);
        let source = session.editor.zone().clone();
        let mut after = pivot(&session).unwrap();
        after.size = [20.; 3]; // Plant scale exceeds its limit; other members could be valid.
        if let Ok(batch) = edits(&session, 0, &after, Some(&library)) {
            session.edit(batch);
        }
        assert_eq!(session.editor.zone(), &source);
        assert_eq!(session.editor.history(), (0, 0));
        after = pivot(&session).unwrap();
        after.position[0] += 2.;
        assert!(edits(&session, 0, &after, None).is_err());
        assert_eq!(session.editor.zone(), &source);
    }
    #[test]
    fn preview_cancel_is_transient_and_modes_clear_the_group() {
        let (mut session, _) = fixture([0.; 3]);
        let source = session.editor.zone().clone();
        let before = pivot(&session).unwrap();
        let mut after = before.clone();
        after.position[0] += 5.;
        let ray = Ray3d::new(
            Vec3::new(0., 10., 10.),
            Dir3::new(Vec3::new(0., -1., -1.)).unwrap(),
        );
        session.drag = Some(Drag {
            id: 0,
            before,
            preview: after,
            revision: 0,
            gesture: gizmo::Gesture::plane(ray, Vec3::ZERO).unwrap(),
        });
        let original = source.objects[0].pose.clone();
        assert_eq!(
            preview(&session, 2, &original).position[0],
            original.position[0] + 5.
        );
        assert_eq!(session.editor.zone(), &source);
        session.drag = None;
        assert_eq!(preview(&session, 2, &original), original);
        session.set_mode(EditorMode::Sculpt);
        assert!(session.group.is_empty());
        assert_eq!(session.editor.history(), (0, 0));
    }
    #[test]
    fn group_exposes_viewport_tools_and_rejects_nonpositive_or_nonuniform_scale() {
        let (session, library) = fixture([0.; 3]);
        assert!(outliner::selection(&session).is_none());
        assert!(
            viewport_tools::bounds(
                &Window {
                    resolution: (1440, 900).into(),
                    ..default()
                },
                &session
            )
            .is_some()
        );
        for tool in [gizmo::Tool::Move, gizmo::Tool::Rotate, gizmo::Tool::Scale] {
            assert!(workspace::Control::GizmoTool(tool).enabled(&session));
        }
        assert!(workspace::Control::Frame.enabled(&session));
        for size in [[0.; 3], [-1.; 3], [1., 2., 1.], [f64::NAN; 3]] {
            let mut after = pivot(&session).unwrap();
            after.size = size;
            assert!(edits(&session, 0, &after, Some(&library)).is_err());
        }
    }
    #[test]
    fn rectangle_both_directions_and_add_toggle_are_deterministic() {
        for (start, end) in [
            (Vec2::ZERO, Vec2::splat(20.)),
            (Vec2::splat(20.), Vec2::ZERO),
        ] {
            let points = [
                (1, Vec2::splat(10.)),
                (2, Vec2::splat(20.)),
                (3, Vec2::splat(30.)),
            ];
            let mut m = Marquee {
                start,
                end,
                base: default(),
                toggle: false,
                clicked: None,
            };
            assert_eq!(box_ids(&m, points.into_iter()), [1, 2].into());
            m.base.insert(3);
            assert_eq!(box_ids(&m, points.into_iter()), [1, 2, 3].into());
            m.base.insert(1);
            m.toggle = true;
            assert_eq!(box_ids(&m, points.into_iter()), [2, 3].into());
        }
    }
    #[test]
    fn large_selection_over_old_128_limit_is_one_transaction() {
        let mut zone = Zone::default();
        for id in 1..=256 {
            zone.objects.push(Object {
                id,
                name: format!("Block {id}"),
                primitive: Primitive::Box,
                pose: pose([id as f64, 1., 0.], 0., 1.),
            });
        }
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            "unused.json".into(),
            Some(zone.clone()),
        );
        select(&mut session, (1..=256).collect());
        let mut after = pivot(&session).unwrap();
        after.position[2] += 3.;
        let batch = edits(&session, 0, &after, None).unwrap();
        session.edit(batch);
        assert_eq!(session.editor.revision(), 1, "{}", session.status);
        assert!(
            session
                .editor
                .zone()
                .objects
                .iter()
                .all(|o| o.pose.position[2] == 3.)
        );
        session.execute(Command::Undo {
            expected_revision: 1,
        });
        assert_eq!(session.editor.zone(), &zone);
    }
}
