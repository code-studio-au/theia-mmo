//! Native menu events and CPU layout only; no OS UI/input or file writes.
use super::*;
pub(in crate::menus) fn fixture(origin: [f64; 3]) -> (App, Entity) {
    let (mut app, window) = outliner::tests::app(origin);
    app.init_resource::<bevy::input_focus::InputFocusVisible>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_plugins((
            bevy::app::TaskPoolPlugin::default(),
            bevy::asset::AssetPlugin::default(),
            bevy::mesh::MeshPlugin,
            bevy::transform::TransformPlugin,
            bevy::camera::visibility::VisibilityPlugin,
            super::super::plugin,
            bevy::ui_widgets::MenuPlugin,
        ));
    app.update();
    (app, window)
}
fn root(app: &mut App, index: usize) -> Entity {
    app.world_mut()
        .query::<(Entity, &State)>()
        .iter(app.world())
        .find(|(_, state)| state.row == index)
        .unwrap()
        .0
}
fn open(app: &mut App, index: usize, secondary: bool) -> Entity {
    let owner = root(app, index);
    let trigger = app
        .world()
        .get::<Children>(owner)
        .unwrap()
        .iter()
        .find(|child| app.world().get::<More>(*child).is_some())
        .unwrap();
    if secondary {
        let window = app
            .world_mut()
            .query_filtered::<Entity, With<Window>>()
            .single(app.world())
            .unwrap();
        let row = app
            .world_mut()
            .query::<(Entity, &outliner::Control)>()
            .iter(app.world())
            .find(|(_, control)| **control == outliner::Control::Row(index))
            .unwrap()
            .0;
        app.world_mut()
            .trigger(bevy::picking::events::PointerPress {
                entity: row,
                button: PointerButton::Secondary,
                count: 1,
                pointer: bevy::picking::events::Pointer::new(
                    bevy::picking::pointer::PointerId::Mouse,
                    bevy::picking::pointer::Location {
                        target: bevy::camera::RenderTarget::Window(
                            bevy::window::WindowRef::Entity(window),
                        )
                        .normalize(Some(window))
                        .unwrap(),
                        position: Vec2::new(150., 300.),
                    },
                ),
                hit: bevy::picking::backend::HitData::new(window, 0., None, None),
            });
    } else {
        app.world_mut().trigger(Activate { entity: trigger });
    }
    app.update();
    app.world()
        .get::<Children>(owner)
        .unwrap()
        .iter()
        .find(|child| app.world().get::<MenuPopup>(*child).is_some())
        .unwrap()
}
fn item(app: &mut App, popup: Entity, control: workspace::Control) -> Entity {
    app.world()
        .get::<Children>(popup)
        .unwrap()
        .iter()
        .find(|child| {
            app.world()
                .get::<workspace::Control>(*child)
                .is_some_and(|found| matches!((found, control), (workspace::Control::Command(a), workspace::Control::Command(b)) if std::mem::discriminant(a) == std::mem::discriminant(&b)))
        })
        .unwrap()
}
#[test]
fn particle_row_menu_deletes_placement_and_preserves_other_content() {
    let (mut app, _) = fixture([0.; 3]);
    app.world_mut()
        .resource_mut::<Session>()
        .edit(vec![Edit::PutParticle {
            particle: theia_zone::particles::ParticlePlacement {
                id: 999,
                name: "New particle".into(),
                effect: "vfx.unassigned".into(),
                position: [0.; 3],
                yaw_degrees: 0.,
                scale: 1.,
            },
        }]);
    app.world_mut().resource_mut::<outliner::Content>().filter = outliner::Filter::Particles;
    app.update();
    let popup = open(&mut app, 0, true);
    assert_eq!(
        app.world().resource::<Session>().particle_selected,
        Some(999)
    );
    let before = app
        .world()
        .resource::<Session>()
        .editor
        .zone()
        .objects
        .clone();
    let delete = item(&mut app, popup, workspace::Control::Command(Action::Delete));
    app.world_mut().trigger(Activate { entity: delete });
    app.update();
    let session = app.world().resource::<Session>();
    assert!(session.editor.zone().particles.is_empty());
    assert_eq!(session.editor.zone().objects, before);
}

#[test]
fn native_row_menu_selects_context_and_duplicates_exactly_once_through_shared_commands() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let (mut app, _) = fixture(origin);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let popup = open(&mut app, 7, origin[0].abs() > 1.);
        assert_eq!(app.world().resource::<Session>().selected, Some(7));
        assert!(
            app.world()
                .resource::<InputFocus>()
                .get()
                .is_some_and(|entity| app.world().get::<MenuItem>(entity).is_some())
        );
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::Flex
        );
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
        let duplicate = item(
            &mut app,
            popup,
            workspace::Control::Command(Action::Duplicate),
        );
        let mut expected = Session::new(
            Editor::new(source.clone()).unwrap(),
            PathBuf::new(),
            Some(source.clone()),
        );
        expected.selected = Some(7);
        selection::duplicate(&mut expected);
        app.world_mut().trigger(Activate { entity: duplicate });
        // A replay against the old context must not duplicate the new selection.
        app.world_mut().trigger(Activate { entity: duplicate });
        app.update();
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), expected.editor.zone());
        assert_eq!(session.editor.revision(), 3);
        assert_eq!(session.editor.history(), (1, 0));
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::None
        );
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Undo {
                expected_revision: 3,
            })
            .unwrap();
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    }
}
#[test]
fn row_context_refuses_stale_selection_revision_and_unfocused_or_owned_gestures() {
    for cause in 0..8 {
        let (mut app, window) = fixture([0.; 3]);
        let popup = open(&mut app, 7, cause % 2 == 0);
        let delete = item(&mut app, popup, workspace::Control::Command(Action::Delete));
        match cause {
            0 => app.world_mut().resource_mut::<Session>().selected = Some(8),
            1 => {
                app.world_mut()
                    .resource_mut::<Session>()
                    .editor
                    .execute(Command::Apply {
                        expected_revision: 2,
                        edits: vec![Edit::RenameZone {
                            name: "External update".into(),
                        }],
                    })
                    .unwrap();
            }
            2 => app.world_mut().get_mut::<Window>(window).unwrap().focused = false,
            3 => app.world_mut().resource_mut::<Session>().ui.input_owned = true,
            4 => app.world_mut().resource_mut::<outliner::Content>().page = 1,
            5 => {
                app.world_mut()
                    .resource_mut::<outliner::Content>()
                    .set_query("Block 8")
                    .unwrap();
                app.update();
            }
            6 => app.world_mut().resource_mut::<Session>().document_id += 1,
            _ => app.world_mut().resource_mut::<Session>().path_point = Some(1),
        }
        let source = app.world().resource::<Session>().editor.zone().clone();
        let history = app.world().resource::<Session>().editor.history();
        let revision = app.world().resource::<Session>().editor.revision();
        app.world_mut().trigger(Activate { entity: delete });
        app.update();
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &source);
        assert_eq!(session.editor.history(), history);
        assert_eq!(session.editor.revision(), revision);
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::None
        );
    }
}
#[test]
fn row_popup_uses_native_unclipping_and_window_edge_placement_inside_a_short_browser() {
    let (mut app, window) = fixture([128_000_000_000.125, 20.25, -128_000_000_000.5]);
    app.add_plugins((scrolling::plugin, bevy::ui_widgets::popover::PopoverPlugin));
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
    for _ in 0..3 {
        app.update();
    }
    let row = app
        .world_mut()
        .query::<(Entity, &outliner::Control)>()
        .iter(app.world())
        .find(|(_, control)| **control == outliner::Control::Row(17))
        .unwrap()
        .0;
    app.world_mut()
        .trigger(bevy::ui_widgets::ScrollIntoView { entity: row });
    for _ in 0..3 {
        app.update();
    }
    let popup = open(&mut app, 17, true);
    for _ in 0..3 {
        app.update();
    }
    let node = app.world().get::<ComputedNode>(popup).unwrap();
    let pose = app
        .world()
        .get::<bevy::ui::UiGlobalTransform>(popup)
        .unwrap();
    assert_eq!(node.size().x, 240.);
    let first = pose.affine().translation - node.size() / 2.;
    let last = pose.affine().translation + node.size() / 2.;
    assert!(first.x >= 8. && last.x <= 3432.);
    assert!(first.y >= 8. && last.y <= 412., "{first:?}..{last:?}");
    assert!(app.world().get::<bevy::ui::CalculatedClip>(row).is_some());
    assert!(
        app.world()
            .get::<bevy::ui::CalculatedClip>(popup)
            .is_none_or(|clip| clip.rects().is_some_and(|rects| rects.is_empty()))
    );
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
    // A popup crossing the panel edge must own that click before raw resize input.
    let browser = app.world().resource::<Session>().ui.width;
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(sidebar::WIDTH + browser, 300.)));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    use bevy::ecs::system::RunSystemOnce;
    app.world_mut().run_system_once(panels::resize).unwrap();
    assert!(!app.world().resource::<Session>().ui.input_owned);
    assert_eq!(app.world().resource::<Session>().ui.width, browser);
}
