//! Whole-inspector CPU font/layout and native scroll requests; no OS UI or renderer.
use super::*;
use bevy::ecs::system::RunSystemOnce;
fn fixture(target: u8) -> App {
    let (mut app, _) = super::super::yaw_tests::fixture_with_input(
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
        target,
        true,
        true,
    );
    {
        let mut session = app.world_mut().resource_mut::<Session>();
        session.transform_edit = None;
        session.set_mode(match target {
            1 => EditorMode::Plants,
            2 => EditorMode::Markers,
            3 | 5 => EditorMode::Guides,
            _ => EditorMode::Objects,
        });
        match target {
            0 => session.selected = Some(1),
            1 => session.plant_selected = Some(1001),
            2 => session.marker_selected = Some(1000),
            _ => {
                session.path_selected = Some(1002);
                session.path_point = (target == 3).then_some(0);
            }
        }
    }
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.insert_resource(View {
        target: Vec3::ZERO,
        yaw: 0.6,
        pitch: 0.8,
        distance: 32.,
        preset: camera_views::Preset::Perspective,
        show_grid: true,
    })
    .init_resource::<outliner::Content>()
    .add_plugins((
        menus::plugin,
        theme::plugin,
        plugin,
        scrolling::plugin,
        bevy::ui_widgets::ButtonPlugin,
        bevy::ui_widgets::CheckboxPlugin,
        bevy::ui_widgets::MenuPlugin,
        bevy::ui_widgets::ScrollbarPlugin,
        bevy::ui_widgets::popover::PopoverPlugin,
    ));
    app.world_mut().run_system_once(workspace::setup).unwrap();
    app.world_mut().run_system_once(outliner::setup).unwrap();
    app.world_mut().run_system_once(crate::setup_ui).unwrap();
    app.add_systems(
        Update,
        (
            panels::apply,
            workspace::sync,
            outliner::sync,
            crate::present,
        )
            .chain(),
    );
    crate::headless_ui::add_layout(&mut app);
    app
}
#[test]
fn complete_selected_inspector_reveals_each_control_at_short_full_and_narrow_wide_layouts() {
    for target in [0, 1, 2, 3, 5] {
        let mut app = fixture(target);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let window = app
            .world_mut()
            .query_filtered::<Entity, With<Window>>()
            .single(app.world())
            .unwrap();
        for width in [300., 600.] {
            app.world_mut().resource_mut::<Session>().ui.inspector = width;
            for height in [420, 1440] {
                app.world_mut()
                    .get_mut::<Window>(window)
                    .unwrap()
                    .resolution
                    .set_physical_resolution(3440, height);
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
                        .y = height;
                    camera.viewport.as_mut().unwrap().physical_size.y = height;
                }
                for _ in 0..3 {
                    app.update();
                }
                let pane = app
                    .world_mut()
                    .query::<(Entity, &scrolling::Pane)>()
                    .iter(app.world())
                    .find(|(_, scope)| matches!(scope, scrolling::Pane::Workspace))
                    .unwrap()
                    .0;
                let candidates: Vec<_> = app
                    .world_mut()
                    .query::<(Entity, &ComputedNode, Has<Button>, Has<Slider>, Has<Info>)>()
                    .iter(app.world())
                    .filter(|(_, node, button, slider, info)| {
                        node.size().min_element() > 0. && (*button || *slider || *info)
                    })
                    .map(|(entity, ..)| entity)
                    .collect();
                let controls: Vec<_> = candidates
                    .into_iter()
                    .filter(|entity| {
                        std::iter::successors(Some(*entity), |current| {
                            app.world().get::<ChildOf>(*current).map(ChildOf::parent)
                        })
                        .any(|ancestor| ancestor == pane)
                    })
                    .collect();
                assert!(controls.len() > 10);
                for entity in controls {
                    app.world_mut()
                        .trigger(bevy::ui_widgets::ScrollIntoView { entity });
                    for _ in 0..3 {
                        app.update();
                    }
                    let node = app.world().get::<ComputedNode>(entity).unwrap();
                    let pose = app
                        .world()
                        .get::<bevy::ui::UiGlobalTransform>(entity)
                        .unwrap();
                    let first = pose.affine().translation - node.size() / 2.;
                    let last = pose.affine().translation + node.size() / 2.;
                    assert!(
                        first.x >= sidebar::WIDTH + outliner::WIDTH
                            && last.x <= sidebar::WIDTH + outliner::WIDTH + width,
                        "target {target}, width {width}, entity {entity:?}: {first:?}..{last:?}"
                    );
                    assert!(
                        first.y >= workspace::HEADER && last.y <= height as f32,
                        "target {target}, {width}x{height}, entity {entity:?}: {first:?}..{last:?}"
                    );
                }
                let session = app.world().resource::<Session>();
                assert_eq!(session.editor.zone(), &source);
                assert_eq!(session.editor.history(), (0, 1));
                assert_eq!(session.editor.revision(), 2);
                assert!(!session.is_dirty());
            }
        }
    }
}
