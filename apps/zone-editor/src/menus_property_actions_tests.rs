//! Native Bevy menu events; no operating-system input or display.
use super::*;
use bevy::ecs::system::RunSystemOnce;

fn setup(mut commands: Commands) {
    commands
        .spawn(Node::default())
        .with_children(|parent| spawn(parent, None));
}
fn fixture(origin: [f64; 3]) -> (App, Entity, Entity, Entity) {
    let (mut app, window) = super::super::context::tests::fixture(origin);
    app.world_mut().resource_mut::<Session>().selected = Some(7);
    app.world_mut().run_system_once(setup).unwrap();
    app.update();
    let owner = app
        .world_mut()
        .query_filtered::<Entity, With<State>>()
        .single(app.world())
        .unwrap();
    let children = app.world().get::<Children>(owner).unwrap();
    let button = children
        .iter()
        .find(|e| app.world().get::<MenuButton>(*e).is_some())
        .unwrap();
    let popup = children
        .iter()
        .find(|e| app.world().get::<MenuPopup>(*e).is_some())
        .unwrap();
    (app, window, button, popup)
}
fn open(app: &mut App, button: Entity, popup: Entity) {
    app.world_mut().trigger(MenuEvent {
        source: button,
        action: MenuAction::Open(NavAction::First),
    });
    app.update();
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::Flex
    );
}
fn item(app: &App, popup: Entity, action: Action) -> Entity {
    app.world().get::<Children>(popup).unwrap().iter().find(|child| {
        app.world().get::<workspace::Control>(*child).is_some_and(|control|
            matches!(control, workspace::Control::Command(found) if std::mem::discriminant(found) == std::mem::discriminant(&action)))
    }).unwrap()
}
#[test]
fn selected_actions_duplicate_once_and_refuse_stale_or_closed_contexts() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let (mut app, _, button, popup) = fixture(origin);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let duplicate = item(&app, popup, Action::Duplicate);
        app.world_mut().trigger(Activate { entity: duplicate });
        app.update();
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        open(&mut app, button, popup);
        app.world_mut().trigger(Activate { entity: duplicate });
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
        assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::None
        );
        app.world_mut()
            .resource_scope(|world, mut session: Mut<Session>| {
                perform(
                    Action::Undo,
                    &mut session,
                    &mut world.resource_mut::<View>(),
                );
            });
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    }
    for cause in 0..6 {
        let (mut app, window, button, popup) = fixture([0.; 3]);
        open(&mut app, button, popup);
        let delete = item(&app, popup, Action::Delete);
        match cause {
            0 => app.world_mut().resource_mut::<Session>().selected = Some(8),
            1 => {
                app.world_mut()
                    .resource_mut::<Session>()
                    .editor
                    .execute(Command::Apply {
                        expected_revision: 2,
                        edits: vec![Edit::RenameZone {
                            name: "New revision".into(),
                        }],
                    })
                    .unwrap();
            }
            2 => app.world_mut().resource_mut::<Session>().document_id += 1,
            3 => app.world_mut().get_mut::<Window>(window).unwrap().focused = false,
            4 => {
                app.world_mut()
                    .resource_mut::<Session>()
                    .ui
                    .properties_collapsed = true
            }
            _ => {
                app.init_resource::<property_tabs::State>();
                app.world_mut()
                    .resource_mut::<property_tabs::State>()
                    .active = property_tabs::Tab::View;
            }
        }
        let source = app.world().resource::<Session>().editor.zone().clone();
        let history = app.world().resource::<Session>().editor.history();
        let revision = app.world().resource::<Session>().editor.revision();
        app.world_mut().trigger(Activate { entity: delete });
        app.update();
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), history);
        assert_eq!(
            app.world().resource::<Session>().editor.revision(),
            revision
        );
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::None
        );
    }
}

#[test]
fn spawn_and_empty_selection_do_not_show_or_open_content_actions() {
    let (mut app, _, button, popup) = fixture([0.; 3]);
    for spawn in [false, true] {
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            session.selected = None;
            session.inspect_spawn = spawn;
        }
        app.update();
        app.world_mut().trigger(MenuEvent {
            source: button,
            action: MenuAction::Open(NavAction::First),
        });
        app.update();
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::None
        );
        let owner = app.world().get::<ChildOf>(button).unwrap().parent();
        assert_eq!(
            app.world().get::<Node>(owner).unwrap().display,
            Display::None
        );
    }
}

#[test]
fn a_deferred_open_is_cancelled_if_focus_moves_elsewhere() {
    let (mut app, _, button, popup) = fixture([0.; 3]);
    app.world_mut().resource_mut::<Session>().inline_editing = true;
    app.world_mut().trigger(MenuEvent {
        source: button,
        action: MenuAction::Open(NavAction::First),
    });
    let other = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(other, FocusCause::Navigated);
    app.world_mut().resource_mut::<Session>().inline_editing = false;
    app.update();
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::None
    );
}
