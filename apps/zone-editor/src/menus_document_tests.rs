//! Document selector over actual shared controllers; no OS UI or file writes.
use super::*;
use crate::particle_workspace::ZoneAppExt;
use bevy::{ecs::system::RunSystemOnce, text::EditableText};
fn fixture(origin: [f64; 3]) -> App {
    let (mut app, _) = super::super::context::tests::fixture(origin);
    app.world_mut().run_system_once(workspace::setup).unwrap();
    app.add_zone_systems(workspace::sync)
        .add_systems(PostUpdate, naming::finish);
    app.update();
    app
}
fn action(app: &mut App, wanted: impl Fn(&workspace::Control) -> bool) {
    let (trigger, popup) = app
        .world_mut()
        .query::<(Entity, &SelectTrigger)>()
        .iter(app.world())
        .find(|(_, selector)| selector.kind == Kind::Document)
        .map(|(entity, selector)| (entity, selector.popup))
        .unwrap();
    app.world_mut().trigger(Activate { entity: trigger });
    app.update();
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
                .is_some_and(&wanted)
        })
        .unwrap();
    app.world_mut().trigger(Activate { entity: item });
    app.update();
}
fn caption(app: &mut App) -> String {
    app.world_mut()
        .query::<(&Label, &Text)>()
        .iter(app.world())
        .find(|(label, _)| label.0 == Kind::Document)
        .unwrap()
        .1
        .0
        .clone()
}
#[test]
fn document_menu_renames_zone_with_selection_then_new_and_undo_preserve_exact_content() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let mut app = fixture(origin);
        app.world_mut().resource_mut::<Session>().selected = Some(7);
        let source = app.world().resource::<Session>().editor.zone().clone();
        assert_eq!(caption(&mut app), source.name);
        action(&mut app, |control| {
            matches!(control, workspace::Control::RenameZone)
        });
        assert!(app.world().resource::<Session>().name_edit.is_some());
        assert_eq!(
            app.world()
                .resource::<Session>()
                .name_edit
                .as_ref()
                .unwrap()
                .text,
            source.name
        );
        let field = app
            .world_mut()
            .query_filtered::<Entity, With<naming::Field>>()
            .single(app.world())
            .unwrap();
        let mut text = app.world_mut().get_mut::<EditableText>(field).unwrap();
        text.clear();
        text.editor_mut().set_text("Élan zone");
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.update();
        let mut expected = source.clone();
        expected.name = "Élan zone".into();
        assert_eq!(app.world().resource::<Session>().editor.zone(), &expected);
        assert_eq!(caption(&mut app), "* Élan zone");
        assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
        assert_eq!(app.world().resource::<Session>().editor.revision(), 3);
        assert_eq!(app.world().resource::<Session>().selected, Some(7));
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Undo {
                expected_revision: 3,
            })
            .unwrap();
        app.update();
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(caption(&mut app), source.name);
        assert!(!app.world().resource::<Session>().is_dirty());
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Redo {
                expected_revision: 4,
            })
            .unwrap();
        app.update();
        action(&mut app, |control| {
            matches!(control, workspace::Control::New)
        });
        assert_eq!(
            app.world().resource::<Session>().editor.zone(),
            &Zone::default()
        );
        assert_eq!(app.world().resource::<Session>().editor.history(), (2, 0));
        assert_eq!(app.world().resource::<Session>().editor.revision(), 6);
        assert!(app.world().resource::<Session>().path.is_none());
        assert_eq!(caption(&mut app), "* Untitled zone");
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Undo {
                expected_revision: 6,
            })
            .unwrap();
        app.update();
        assert_eq!(app.world().resource::<Session>().editor.zone(), &expected);
        assert_eq!(caption(&mut app), "* Élan zone");
        assert_eq!(app.world().resource::<Session>().editor.history(), (1, 1));
        assert!(app.world().resource::<Session>().is_dirty());
    }
}
#[test]
fn long_unicode_document_name_stays_bounded_inside_the_top_bar_with_actual_theme() {
    let mut app = fixture([0.; 3]);
    let name = "Élan ".repeat(12) + "studio";
    app.world_mut()
        .resource_mut::<Session>()
        .editor
        .execute(Command::Apply {
            expected_revision: 2,
            edits: vec![Edit::RenameZone { name: name.clone() }],
        })
        .unwrap();
    crate::headless_ui::add_layout(&mut app);
    app.init_resource::<bevy::input::touch::Touches>()
        .add_plugins(theme::plugin);
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(caption(&mut app), format!("* {name}"));
    let (entity, _, node, pose) = app
        .world_mut()
        .query::<(
            Entity,
            &SelectTrigger,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
        )>()
        .iter(app.world())
        .find(|(_, selector, _, _)| selector.kind == Kind::Document)
        .unwrap();
    let first = pose.affine().translation - node.size() / 2.;
    let last = pose.affine().translation + node.size() / 2.;
    assert!(node.size().x > 0. && node.size().x <= 320.);
    assert!(first.x >= 0. && last.x <= 3440. && first.y >= 0. && last.y <= workspace::HEADER);
    assert!(app.world().get::<Button>(entity).is_some());
    assert_eq!(app.world().resource::<Session>().editor.revision(), 3);
    assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
}
