//! Real document/file controller flows, with owned temporary files and no OS UI.
use super::*;
use bevy::picking::{
    backend::HitData,
    events::Pointer,
    pointer::{Location, PointerId},
};

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "theia-zone-document-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn fixture(root: &Sandbox, origin: [f64; 3]) -> App {
    let zone = Zone {
        name: "Original zone".into(),
        origin,
        spawn: [origin[0], origin[1] + 0.0625, origin[2] + 6.],
        objects: vec![Object {
            id: 41,
            name: "Exact old box".into(),
            primitive: Primitive::Box,
            pose: Pose {
                position: [origin[0] + 8.125, origin[1] + 1.25, origin[2] - 6.5],
                yaw_degrees: 17.25,
                size: [2., 2.5, 3.],
            },
        }],
        ..default()
    };
    let path = root.0.join("original.zone.json");
    zone.save(&path).unwrap();
    let mut session = Session::new(Editor::new(zone.clone()).unwrap(), path, Some(zone));
    session.selected = Some(41);
    session.snap = false;
    session.move_step = 0.125;
    let mut app = App::new();
    app.insert_resource(session)
        .insert_resource(View {
            target: Vec3::new(9., 2., 3.),
            yaw: 0.6,
            pitch: 0.8,
            distance: 100.,
            preset: camera_views::Preset::Top,
            show_grid: false,
        })
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<InputFocus>()
        .init_resource::<outliner::Content>()
        .add_observer(workspace::activate)
        .add_observer(activate)
        .add_systems(Startup, (workspace::setup, setup))
        .add_systems(Update, (keyboard, workspace::sync, present).chain())
        .add_systems(PostUpdate, finish);
    app.world_mut().spawn(Window {
        resolution: (3440, 1440).into(),
        focused: true,
        ..default()
    });
    app.world_mut().spawn((Summary, Text::new("")));
    app.update();
    app
}
fn click(app: &mut App, entity: Entity) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    app.world_mut().trigger(PointerPress {
        entity,
        button: PointerButton::Primary,
        count: 1,
        pointer: Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                    .normalize(Some(window))
                    .unwrap(),
                position: Vec2::ZERO,
            },
        ),
        hit: HitData::new(window, 0., None, None),
    });
    app.update();
}
fn new_zone(app: &mut App) {
    let button = app
        .world_mut()
        .query::<(Entity, &workspace::Control)>()
        .iter(app.world())
        .find(|(_, control)| matches!(control, workspace::Control::New))
        .unwrap()
        .0;
    click(app, button);
}
fn action(app: &mut App, action: Action) {
    app.world_mut()
        .resource_scope(|world, mut session: Mut<Session>| {
            let mut view = world.resource_mut::<View>();
            perform(action, &mut session, &mut view);
        });
    app.update();
}
fn apply(app: &mut App) {
    let button = app
        .world_mut()
        .query::<(Entity, &Control)>()
        .iter(app.world())
        .find(|(_, control)| matches!(control, Control::Apply))
        .unwrap()
        .0;
    click(app, button);
    app.update();
}
fn filename(app: &mut App, name: &str) {
    let mut fields = app
        .world_mut()
        .query_filtered::<&mut EditableText, With<Field>>();
    fields
        .single_mut(app.world_mut())
        .unwrap()
        .editor_mut()
        .set_text(name);
    app.update();
}
fn cancel(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.update();
}
#[test]
fn new_untitled_document_keeps_old_unsaved_content_undoable_and_first_save_uses_save_as() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let root = Sandbox::new();
        let mut app = fixture(&root, origin);
        let original_path = root.0.join("original.zone.json");
        let original_bytes = std::fs::read(&original_path).unwrap();
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Unsaved courtyard".into(),
                }],
            })
            .unwrap();
        let prior = app.world().resource::<Session>().editor.zone().clone();
        {
            let mut content = app.world_mut().resource_mut::<outliner::Content>();
            content.filter = outliner::Filter::Objects;
            content.set_query("old query").unwrap();
        }
        new_zone(&mut app);
        app.update();
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &Zone::default());
        assert_eq!(session.editor.revision(), 2);
        assert!(
            session.path.is_none() && session.saved_zone.is_none() && session.selected.is_none()
        );
        assert!(session.is_dirty());
        assert_eq!(session.last_directory, root.0);
        assert!(!session.snap);
        assert_eq!(session.move_step, 0.125);
        assert_eq!(
            app.world().resource::<outliner::Content>().filter,
            outliner::Filter::All
        );
        assert!(app.world().resource::<outliner::Content>().query.is_empty());
        assert_eq!(
            app.world().resource::<View>().preset,
            camera_views::Preset::Top
        );
        assert_eq!(app.world().resource::<View>().target, Vec3::ZERO);
        assert!(
            app.world_mut()
                .query::<&Window>()
                .single(app.world())
                .unwrap()
                .title
                .contains("Untitled")
        );
        action(&mut app, Action::Undo);
        assert_eq!(app.world().resource::<Session>().editor.zone(), &prior);
        assert!(app.world().resource::<Session>().path.is_none());
        action(&mut app, Action::Redo);
        assert_eq!(
            app.world().resource::<Session>().editor.zone(),
            &Zone::default()
        );
        let history = app.world().resource::<Session>().editor.history();
        action(&mut app, Action::Save);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .file_dialog
                .as_ref()
                .unwrap()
                .mode,
            Mode::SaveAs
        );
        assert_eq!(
            app.world()
                .resource::<Session>()
                .file_dialog
                .as_ref()
                .unwrap()
                .folder,
            root.0
        );
        assert_eq!(
            app.world()
                .resource::<Session>()
                .file_dialog
                .as_ref()
                .unwrap()
                .text,
            "untitled.zone.json"
        );
        assert!(!root.0.join("untitled.zone.json").exists());
        assert_eq!(std::fs::read(&original_path).unwrap(), original_bytes);
        cancel(&mut app);
        assert!(app.world().resource::<Session>().path.is_none());
        assert_eq!(app.world().resource::<Session>().editor.history(), history);
        action(&mut app, Action::Save);
        filename(&mut app, "new zone");
        apply(&mut app);
        let destination = root.0.join("new zone.zone.json");
        assert_eq!(
            app.world().resource::<Session>().path.as_ref(),
            Some(&destination)
        );
        assert_eq!(Zone::load(&destination).unwrap(), Zone::default());
        assert!(!app.world().resource::<Session>().is_dirty());
        assert_eq!(app.world().resource::<Session>().editor.history(), history);
        assert_eq!(std::fs::read(&original_path).unwrap(), original_bytes);
        assert!(
            app.world_mut()
                .query::<&Window>()
                .single(app.world())
                .unwrap()
                .title
                .contains("new zone.zone.json")
        );
    }
}
#[test]
fn failed_and_overwrite_saves_preserve_untitled_identity_until_success() {
    let root = Sandbox::new();
    let mut app = fixture(&root, [0.; 3]);
    new_zone(&mut app);
    let source = app.world().resource::<Session>().editor.zone().clone();
    let history = app.world().resource::<Session>().editor.history();
    std::fs::create_dir(root.0.join("blocked.json")).unwrap();
    action(&mut app, Action::Save);
    filename(&mut app, "blocked.json");
    apply(&mut app);
    assert!(app.world().resource::<Session>().file_dialog.is_some());
    assert!(app.world().resource::<Session>().path.is_none());
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    let retained = root.0.join("untitled.zone.json");
    std::fs::write(&retained, b"retain existing destination exactly").unwrap();
    filename(&mut app, "untitled.zone.json");
    apply(&mut app);
    assert!(
        app.world()
            .resource::<Session>()
            .file_dialog
            .as_ref()
            .unwrap()
            .overwrite
    );
    assert_eq!(
        std::fs::read(&retained).unwrap(),
        b"retain existing destination exactly"
    );
    assert!(app.world().resource::<Session>().path.is_none());
    apply(&mut app);
    assert_eq!(
        app.world().resource::<Session>().path.as_ref(),
        Some(&retained)
    );
    assert_eq!(Zone::load(&retained).unwrap(), source);
    assert_eq!(app.world().resource::<Session>().editor.history(), history);
    assert!(!app.world().resource::<Session>().is_dirty());
}
#[test]
fn repeated_blank_new_preserves_redo_and_playtest_or_drafts_refuse_new() {
    let root = Sandbox::new();
    let mut app = fixture(&root, [0.; 3]);
    new_zone(&mut app);
    app.world_mut()
        .resource_mut::<Session>()
        .editor
        .execute(Command::Apply {
            expected_revision: 1,
            edits: vec![Edit::Place {
                object: Object {
                    id: 99,
                    name: "Temporary box".into(),
                    primitive: Primitive::Box,
                    pose: Pose {
                        position: [8., 1., 0.],
                        yaw_degrees: 0.,
                        size: [2.; 3],
                    },
                },
            }],
        })
        .unwrap();
    action(&mut app, Action::Undo);
    let history = app.world().resource::<Session>().editor.history();
    let revision = app.world().resource::<Session>().editor.revision();
    new_zone(&mut app);
    assert_eq!(app.world().resource::<Session>().editor.history(), history);
    assert_eq!(
        app.world().resource::<Session>().editor.revision(),
        revision
    );
    action(&mut app, Action::Load);
    assert!(app.world().resource::<Session>().path.is_none());
    action(&mut app, Action::Play);
    assert!(app.world().resource::<Session>().play.is_some());
    new_zone(&mut app);
    assert!(app.world().resource::<Session>().play.is_some());
    assert_eq!(
        app.world().resource::<Session>().editor.revision(),
        revision
    );
    action(&mut app, Action::Play);
    paths::perform(
        Action::NewPath(theia_zone::paths::PathKind::Road),
        &mut app.world_mut().resource_mut::<Session>(),
    );
    new_zone(&mut app);
    assert!(app.world().resource::<Session>().path_draft.is_some());
    assert_eq!(
        app.world().resource::<Session>().editor.revision(),
        revision
    );
}
