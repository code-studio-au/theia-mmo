//! Whole-inspector CPU font/layout and native scroll requests; no OS UI or renderer.
use super::*;
use crate::particle_workspace::ZoneAppExt;
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
            4 => session.inspect_spawn = true,
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
        property_tabs::plugin,
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
    app.add_zone_systems(
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
    for target in [0, 1, 2, 3, 4, 5] {
        let mut app = fixture(target);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let window = app
            .world_mut()
            .query_filtered::<Entity, With<Window>>()
            .single(app.world())
            .unwrap();
        for width in [300., 600.] {
            app.world_mut().resource_mut::<Session>().ui.width = width;
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
                for tab in [
                    property_tabs::Tab::Attributes,
                    property_tabs::Tab::Tools,
                    property_tabs::Tab::View,
                ] {
                    app.world_mut()
                        .resource_mut::<property_tabs::State>()
                        .active = tab;
                    for _ in 0..3 {
                        app.update();
                    }
                    let candidates: Vec<_> = app
                        .world_mut()
                        .query::<(
                            Entity,
                            &ComputedNode,
                            Has<Button>,
                            Has<Slider>,
                            Has<Info>,
                            Has<TextInput>,
                        )>()
                        .iter(app.world())
                        .filter(|(_, node, button, slider, info, input)| {
                            node.size().min_element() > 0.
                                && (*button || *slider || *info || *input)
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
                    assert!(!controls.is_empty());
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
                            first.x >= sidebar::WIDTH && last.x <= sidebar::WIDTH + width,
                            "target {target}, width {width}, entity {entity:?}: {first:?}..{last:?}"
                        );
                        assert!(
                            first.y
                                >= app
                                    .world()
                                    .resource::<Session>()
                                    .ui
                                    .properties_top(height as f32)
                                && last.y <= height as f32 - status_bar::HEIGHT,
                            "target {target}, {width}x{height}, entity {entity:?}: {first:?}..{last:?}"
                        );
                    }
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

#[test]
fn opening_property_actions_commits_the_focused_field_before_menu_snapshot() {
    use bevy::input_focus::FocusCause;
    use bevy::text::EditableText;
    use bevy::ui_widgets::{MenuAction, MenuEvent, MenuPopup};
    let mut app = fixture(0);
    for _ in 0..3 {
        app.update();
    }
    let source = app.world().resource::<Session>().editor.zone().clone();
    let field = app
        .world_mut()
        .query::<(Entity, &Value)>()
        .iter(app.world())
        .find(|(_, field)| field.0 == 0)
        .unwrap()
        .0;
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(field, FocusCause::Navigated);
    app.update();
    let value = source.origin[0] + 9.375;
    app.world_mut()
        .get_mut::<EditableText>(field)
        .unwrap()
        .editor_mut()
        .set_text(&value.to_string());
    app.update();
    let trigger = app
        .world_mut()
        .query::<(Entity, &menus::Trigger)>()
        .iter(app.world())
        .find(|(_, trigger)| trigger.0 == "Property actions")
        .unwrap()
        .0;
    app.world_mut().trigger(MenuEvent {
        source: trigger,
        action: MenuAction::Open(NavAction::First),
    });
    for _ in 0..4 {
        app.update();
    }
    let session = app.world().resource::<Session>();
    assert_eq!(session.editor.zone().objects[0].pose.position[0], value);
    assert_eq!(session.editor.history(), (1, 0));
    assert!(!session.inline_editing);
    let popup = app
        .world_mut()
        .query::<(Entity, &menus::Popup, &Node, Has<MenuPopup>)>()
        .iter(app.world())
        .find(|(_, p, _, menu)| p.0 == "Property actions" && *menu)
        .unwrap();
    assert_eq!(popup.2.display, Display::Flex);
    let duplicate = app
        .world()
        .get::<Children>(popup.0)
        .unwrap()
        .iter()
        .find(|child| {
            app.world()
                .get::<workspace::Control>(*child)
                .is_some_and(|control| {
                    matches!(control, workspace::Control::Command(Action::Duplicate))
                })
        })
        .unwrap();
    app.world_mut().trigger(Activate { entity: duplicate });
    app.update();
    assert_eq!(
        app.world()
            .resource::<Session>()
            .editor
            .zone()
            .objects
            .len(),
        source.objects.len() + 1
    );
    assert_eq!(app.world().resource::<Session>().editor.history(), (2, 0));
}

#[test]
fn property_action_popover_fits_a_short_window_and_keeps_one_frame() {
    use bevy::ui::OverrideClip;
    use bevy::ui_widgets::{MenuAction, MenuEvent, MenuItem, MenuPopup};
    let mut app = fixture(0);
    app.world_mut().resource_mut::<Session>().ui.width = 300.;
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
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
    for _ in 0..4 {
        app.update();
    }
    let trigger = app
        .world_mut()
        .query::<(Entity, &menus::Trigger)>()
        .iter(app.world())
        .find(|(_, trigger)| trigger.0 == "Property actions")
        .unwrap()
        .0;
    app.world_mut().trigger(MenuEvent {
        source: trigger,
        action: MenuAction::Open(NavAction::First),
    });
    for _ in 0..5 {
        app.update();
    }
    let popup = app
        .world_mut()
        .query::<(Entity, &menus::Popup, &Node, Has<MenuPopup>)>()
        .iter(app.world())
        .find(|(_, popup, _, menu)| popup.0 == "Property actions" && *menu)
        .unwrap()
        .0;
    assert!(app.world().get::<OverrideClip>(popup).is_some());
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().border,
        UiRect::all(px(1))
    );
    let computed = app.world().get::<ComputedNode>(popup).unwrap();
    let centre = app
        .world()
        .get::<bevy::ui::UiGlobalTransform>(popup)
        .unwrap()
        .affine()
        .translation;
    let first = centre - computed.size() / 2.;
    let last = centre + computed.size() / 2.;
    assert!(
        first.x >= 0. && first.y >= 0. && last.x <= 3440. && last.y <= 420.,
        "{first:?}..{last:?}"
    );
    let children: Vec<_> = app.world().get::<Children>(popup).unwrap().iter().collect();
    for child in children {
        assert!(app.world().get::<MenuItem>(child).is_some());
        assert_eq!(app.world().get::<Node>(child).unwrap().border, UiRect::ZERO);
        assert_eq!(
            app.world().get::<BackgroundColor>(child).unwrap().0,
            theme::Variant::Ghost.background(false)
        );
    }
}

#[test]
fn rename_from_property_actions_focuses_the_permanent_name_field() {
    use bevy::ui_widgets::{MenuAction, MenuEvent, MenuPopup};
    let mut app = fixture(0);
    app.add_zone_systems(naming::keyboard);
    for _ in 0..4 {
        app.update();
    }
    let source = app.world().resource::<Session>().editor.zone().clone();
    let trigger = app
        .world_mut()
        .query::<(Entity, &menus::Trigger)>()
        .iter(app.world())
        .find(|(_, trigger)| trigger.0 == "Property actions")
        .unwrap()
        .0;
    app.world_mut().trigger(MenuEvent {
        source: trigger,
        action: MenuAction::Open(NavAction::First),
    });
    for _ in 0..3 {
        app.update();
    }
    let popup = app
        .world_mut()
        .query::<(Entity, &menus::Popup, Has<MenuPopup>)>()
        .iter(app.world())
        .find(|(_, popup, menu)| popup.0 == "Property actions" && *menu)
        .unwrap()
        .0;
    let rename = app
        .world()
        .get::<Children>(popup)
        .unwrap()
        .iter()
        .find(|child| {
            app.world()
                .get::<workspace::Control>(*child)
                .is_some_and(|control| matches!(control, workspace::Control::Rename))
        })
        .unwrap();
    app.world_mut().trigger(Activate { entity: rename });
    for _ in 0..4 {
        app.update();
    }
    let focused = app.world().resource::<InputFocus>().get().unwrap();
    assert!(app.world().get::<Name>(focused).is_some());
    assert!(app.world().resource::<Session>().name_edit.is_none());
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
}

#[test]
fn guide_point_controls_follow_the_selected_content() {
    for (target, expected) in [(0, Display::None), (3, Display::Flex), (4, Display::None)] {
        let mut app = fixture(target);
        for _ in 0..3 {
            app.update();
        }
        let node = app
            .world_mut()
            .query_filtered::<&Node, With<menus::GuideControls>>()
            .single(app.world())
            .unwrap();
        assert_eq!(node.display, expected);
    }
}

#[test]
fn property_menu_rejects_a_changed_guide_point_without_editing_the_guide() {
    use bevy::ui_widgets::{MenuAction, MenuEvent, MenuPopup};
    let mut app = fixture(3);
    for _ in 0..3 {
        app.update();
    }
    let source = app.world().resource::<Session>().editor.zone().clone();
    let trigger = app
        .world_mut()
        .query::<(Entity, &menus::Trigger)>()
        .iter(app.world())
        .find(|(_, trigger)| trigger.0 == "Property actions")
        .unwrap()
        .0;
    app.world_mut().trigger(MenuEvent {
        source: trigger,
        action: MenuAction::Open(NavAction::First),
    });
    for _ in 0..3 {
        app.update();
    }
    let popup = app
        .world_mut()
        .query::<(Entity, &menus::Popup, Has<MenuPopup>)>()
        .iter(app.world())
        .find(|(_, popup, menu)| popup.0 == "Property actions" && *menu)
        .unwrap()
        .0;
    let delete = app
        .world()
        .get::<Children>(popup)
        .unwrap()
        .iter()
        .find(|child| {
            app.world()
                .get::<workspace::Control>(*child)
                .is_some_and(|control| {
                    matches!(control, workspace::Control::Command(Action::Delete))
                })
        })
        .unwrap();
    app.world_mut().resource_mut::<Session>().path_point = Some(1);
    app.world_mut().trigger(Activate { entity: delete });
    app.update();
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::None
    );
}
