//! Native menu selection and CPU window-edge layout; no OS input or renderer.
use super::*;
use bevy::ecs::system::RunSystemOnce;
fn setup(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: px(400),
            top: px(280),
            width: px(300),
            overflow: Overflow::clip(),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        })
        .with_children(|root| {
            spawn(root, Kind::Camera);
            spawn(root, Kind::Gizmo);
        });
}
fn fixture(origin: [f64; 3]) -> (App, Entity) {
    let (mut app, window) = super::super::context::tests::fixture(origin);
    app.world_mut().run_system_once(setup).unwrap();
    app.update();
    (app, window)
}
fn open(app: &mut App, kind: Kind) -> Entity {
    let (trigger, popup) = app
        .world_mut()
        .query::<(Entity, &SelectTrigger)>()
        .iter(app.world())
        .find(|(_, trigger)| trigger.kind == kind)
        .map(|(entity, selector)| (entity, selector.popup))
        .unwrap();
    app.world_mut().trigger(Activate { entity: trigger });
    app.update();
    popup
}
fn choose(app: &mut App, kind: Kind, control: workspace::Control) {
    let popup = open(app, kind);
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::Flex
    );
    let item = app
        .world()
        .get::<Children>(popup)
        .unwrap()
        .iter()
        .find(|child| {
            app.world()
                .get::<workspace::Control>(*child)
                .is_some_and(|found| match (found, control) {
                    (workspace::Control::View(a), workspace::Control::View(b)) => *a == b,
                    (workspace::Control::GizmoTool(a), workspace::Control::GizmoTool(b)) => *a == b,
                    _ => false,
                })
        })
        .unwrap();
    app.world_mut().trigger(Activate { entity: item });
    app.update();
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::None
    );
}
#[test]
fn native_selectors_show_current_camera_and_gizmo_without_source_or_history_edits() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let (mut app, _) = fixture(origin);
        let source = app.world().resource::<Session>().editor.zone().clone();
        for preset in [
            camera_views::Preset::Top,
            camera_views::Preset::Front,
            camera_views::Preset::Right,
            camera_views::Preset::Perspective,
        ] {
            choose(&mut app, Kind::Camera, workspace::Control::View(preset));
            assert_eq!(app.world().resource::<View>().preset, preset);
        }
        for tool in [gizmo::Tool::Rotate, gizmo::Tool::Scale, gizmo::Tool::Move] {
            choose(&mut app, Kind::Gizmo, workspace::Control::GizmoTool(tool));
            assert_eq!(app.world().resource::<Session>().gizmo_tool, tool);
        }
        for (label, text) in app.world_mut().query::<(&Label, &Text)>().iter(app.world()) {
            assert_eq!(
                text.0,
                label.0.value(
                    app.world().resource::<Session>(),
                    app.world().resource::<View>()
                )
            );
        }
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &source);
        assert_eq!(session.editor.history(), (0, 1));
        assert_eq!(session.editor.revision(), 2);
        assert!(!session.is_dirty());
    }
}
#[test]
fn selector_guards_close_disabled_or_unfocused_popups_and_preserve_new_modal_focus() {
    let (mut app, window) = fixture([0.; 3]);
    app.world_mut()
        .resource_mut::<Session>()
        .set_mode(EditorMode::Plants);
    app.update();
    let popup = open(&mut app, Kind::Gizmo);
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::None
    );
    let popup = open(&mut app, Kind::Camera);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.update();
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::None
    );
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    let popup = open(&mut app, Kind::Camera);
    app.world_mut().resource_mut::<workspace::Requests>().find = true;
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().resource::<Session>().name_edit.is_some());
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::None
    );
    assert!(
        app.world()
            .resource::<InputFocus>()
            .get()
            .is_some_and(|entity| app.world().get::<naming::Field>(entity).is_some())
    );
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    assert!(!app.world().resource::<Session>().is_dirty());
}
#[test]
fn selector_popups_flip_above_the_field_and_escape_parent_clipping_in_a_short_window() {
    let (mut app, window) = fixture([0.; 3]);
    app.add_plugins(bevy::ui_widgets::popover::PopoverPlugin);
    crate::headless_ui::add_layout(&mut app);
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .resolution
        .set_physical_resolution(3440, 420);
    for mut camera in app
        .world_mut()
        .query::<&mut Camera>()
        .iter_mut(app.world_mut())
    {
        camera
            .computed
            .target_info
            .as_mut()
            .unwrap()
            .physical_size
            .y = 420;
        camera.viewport.as_mut().unwrap().physical_size.y = 420;
    }
    for kind in [Kind::Camera, Kind::Gizmo] {
        let popup = open(&mut app, kind);
        for _ in 0..3 {
            app.update();
        }
        let node = app.world().get::<ComputedNode>(popup).unwrap();
        let pose = app
            .world()
            .get::<bevy::ui::UiGlobalTransform>(popup)
            .unwrap();
        let first = pose.affine().translation - node.size() / 2.;
        let last = pose.affine().translation + node.size() / 2.;
        assert_eq!(node.size().x, 300.);
        assert!(
            app.world()
                .get::<bevy::ui::CalculatedClip>(popup)
                .is_none_or(|clip| clip.rects().is_some_and(|rects| rects.is_empty()))
        );
        assert!(first.x >= 8. && last.x <= 3432.);
        assert!(
            first.y >= 8. && last.y <= 412.,
            "{kind:?}: {first:?}..{last:?}"
        );
        assert!(last.y <= 280. + if kind == Kind::Camera { 0. } else { 60. });
        app.world_mut().trigger(MenuEvent {
            source: popup,
            action: MenuAction::CloseAll,
        });
        app.update();
    }
}
