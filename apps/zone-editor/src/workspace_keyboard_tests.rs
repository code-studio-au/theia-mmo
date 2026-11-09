//! Standard focus/slider/input dispatch must not also edit the selected object.
use super::*;
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};
use bevy::input_focus::{AcquireFocus, InputFocus};

#[test]
fn focused_workspace_slider_owns_arrow_keys_and_preserves_selected_source_and_redo() {
    for placement in [false, true] {
        let zone = Zone {
            objects: vec![Object {
                id: 1,
                name: "Selected block".into(),
                primitive: Primitive::Box,
                pose: Pose {
                    position: [8., 1., -8.],
                    yaw_degrees: 0.,
                    size: [2.; 3],
                },
            }],
            ..default()
        };
        let mut editor = Editor::new(zone.clone()).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Temporary".into(),
                }],
            })
            .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        let mut session = Session::new(editor, "unused.json".into(), Some(zone.clone()));
        session.selected = Some(1);
        session.palette = placement.then_some(Primitive::Box);
        let mut app = App::new();
        app.add_plugins((
            bevy::app::TaskPoolPlugin::default(),
            bevy::asset::AssetPlugin::default(),
            bevy::mesh::MeshPlugin,
            bevy::transform::TransformPlugin,
            bevy::camera::visibility::VisibilityPlugin,
            bevy::input::InputPlugin,
            bevy::input_focus::InputFocusPlugin,
            bevy::input_focus::pointer_focus::PointerFocusPlugin,
            bevy::input_focus::InputDispatchPlugin,
            bevy::ui_widgets::SliderPlugin,
        ))
        .insert_resource(session)
        .insert_resource(View {
            target: Vec3::ZERO,
            yaw: 0.6,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        })
        .init_resource::<outliner::Content>()
        .add_observer(change)
        .add_systems(Startup, setup)
        .add_systems(Update, (crate::keyboard, sync).chain());
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                bevy::window::PrimaryWindow,
            ))
            .id();
        for _ in 0..3 {
            app.update();
        }
        let slider = app
            .world_mut()
            .query::<(Entity, &Setting)>()
            .iter(app.world())
            .find(|(_, setting)| {
                if placement {
                    matches!(setting, Setting::PlacementSize(0))
                } else {
                    matches!(setting, Setting::MoveGrid)
                }
            })
            .unwrap()
            .0;
        app.world_mut().trigger(AcquireFocus {
            focused_entity: slider,
            window,
        });
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(slider));
        press(&mut app, window, KeyCode::ArrowRight, Key::ArrowRight);
        let session = app.world().resource::<Session>();
        assert_eq!(
            session.editor.zone(),
            &zone,
            "placement {placement}: slider input must not nudge the selection"
        );
        assert_eq!(session.editor.revision(), 2);
        assert_eq!(session.editor.history(), (0, 1));
        if placement {
            assert_eq!(session.placement_size, [1.25, 1., 1.]);
        } else {
            assert_eq!(session.move_step, 2.);
        }
        for (code, key) in [
            (KeyCode::End, Key::End),
            (KeyCode::Home, Key::Home),
            (KeyCode::ArrowLeft, Key::ArrowLeft),
        ] {
            press(&mut app, window, code, key);
            let session = app.world().resource::<Session>();
            assert_eq!(session.editor.zone(), &zone);
            assert_eq!(session.editor.revision(), 2);
            assert_eq!(session.editor.history(), (0, 1));
        }
        let step = app.world().resource::<Session>().move_step;
        app.world_mut().resource_mut::<InputFocus>().clear();
        press(&mut app, window, KeyCode::ArrowRight, Key::ArrowRight);
        let session = app.world().resource::<Session>();
        assert_eq!(
            session.editor.zone().objects[0].pose.position,
            [8. + step, 1., -8.]
        );
        assert_eq!(session.editor.revision(), 3);
        assert_eq!(session.editor.history(), (1, 0));
    }
}

fn press(app: &mut App, window: Entity, code: KeyCode, key: Key) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code: code,
            logical_key: key.clone(),
            state,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    }
}
