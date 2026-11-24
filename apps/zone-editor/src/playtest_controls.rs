//! Offline playtest uses the game's actual binding sampler and orbit math.
//! Pointer ownership stays with the editor; this never emits network commands.
use super::*;
use bevy::{
    input::mouse::MouseScrollUnit,
    window::{CursorGrabMode, CursorOptions},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut windows: Query<(&Window, &mut CursorOptions)>,
    mut session: ResMut<Session>,
    mut captured: Local<bool>,
    focus: Res<bevy::input_focus::InputFocus>,
    menu_items: Query<(), With<bevy::ui_widgets::MenuItem>>,
) {
    let Ok((window, mut cursor)) = windows.single_mut() else {
        return;
    };
    let allowed = window.focused
        && !session.has_modal()
        && !session.inline_editing
        && !session.ui.input_owned
        && !session.ui.overlay_owned
        && !focus
            .get()
            .is_some_and(|entity| menu_items.contains(entity));
    let over_canvas = canvas_cursor(window, &session).is_some();
    let Some(play) = session.play.as_mut() else {
        if *captured {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
            *captured = false;
        }
        return;
    };
    for (index, button) in [MouseButton::Left, MouseButton::Right]
        .into_iter()
        .enumerate()
    {
        if !allowed || !buttons.pressed(button) {
            play.look[index] = false;
        } else if buttons.just_pressed(button) && over_canvas {
            play.look[index] = true;
            play.look_travel = 0.;
        }
    }
    if play.look.iter().any(|v| *v) {
        play.look_travel += motion.delta.length();
    } else {
        play.look_travel = 0.;
    }
    let rotating = play.look.iter().any(|v| *v) && play.look_travel > 6.;
    let transition = rotating != *captured;
    if rotating || *captured {
        cursor.visible = !rotating;
        cursor.grab_mode = if rotating {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
    *captured = rotating;
    let wheel = if allowed && over_canvas {
        scroll.delta.y
            * if scroll.unit == MouseScrollUnit::Pixel {
                1. / 40.
            } else {
                1.
            }
    } else {
        0.
    };
    play.camera.adjust(
        if rotating && !transition {
            motion.delta
        } else {
            Vec2::ZERO
        },
        wheel,
    );
    let both = play.look[0] && play.look[1];
    let sample = play.controls.sample(
        &keys,
        time.delta_secs_f64(),
        play.camera.angles().x,
        play.look[1] && (rotating || both),
        both,
        allowed,
    );
    if !play.look[0] && sample.keyboard_turn != 0. {
        let angles = play.camera.angles();
        play.camera
            .set_angles(angles.x + sample.keyboard_turn, angles.y);
    }
    play.input = theia_network::wire::Intent {
        axes: sample.axes,
        jump: allowed && (play.input.jump || sample.jump),
        sprint: sample.sprint,
        facing: Some(play.controls.facing()),
        ..default()
    };
    session.heading = play.controls.yaw();
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> (App, Entity, Zone) {
        let zone = Zone::default();
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            "unused.zone.json".into(),
            Some(zone.clone()),
        );
        session.play = Some(playtest::Playtest::new(&zone).unwrap());
        let mut app = App::new();
        app.insert_resource(session)
            .init_resource::<Time>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .init_resource::<bevy::input_focus::InputFocus>()
            .add_systems(Update, input);
        let mut window = Window {
            focused: true,
            resolution: (1280, 720).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(800., 300.)));
        let entity = app
            .world_mut()
            .spawn((window, CursorOptions::default()))
            .id();
        (app, entity, zone)
    }
    fn frame(app: &mut App, motion: Vec2, wheel: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(1. / 60.));
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = motion;
        app.world_mut()
            .resource_mut::<AccumulatedMouseScroll>()
            .delta
            .y = wheel;
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
    }
    #[test]
    fn game_orbit_preserves_facing_then_rmb_steers_and_both_buttons_walk_without_document_changes()
    {
        let (mut app, window, zone) = app();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        frame(&mut app, Vec2::new(10., 0.), 0.);
        let camera = app
            .world()
            .resource::<Session>()
            .play
            .as_ref()
            .unwrap()
            .camera
            .angles();
        assert_eq!(camera.x, 0., "Capture transition must ignore warp motion");
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        frame(&mut app, Vec2::new(80., 20.), 0.);
        let session = app.world().resource::<Session>();
        let play = session.play.as_ref().unwrap();
        assert!(play.camera.angles().x < -0.3);
        assert_eq!(session.heading, 0.);
        assert_eq!(play.input.axes, [0, -1000]);
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::Locked
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyW);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        frame(&mut app, Vec2::ZERO, 0.);
        let session = app.world().resource::<Session>();
        let play = session.play.as_ref().unwrap();
        assert!(play.input.axes[0] > 200 && play.input.axes[1] < -800);
        assert!((session.heading - play.controls.yaw()).abs() < 1e-6);
        assert_eq!(session.editor.zone(), &zone);
        assert_eq!(session.editor.revision(), 0);
    }
    #[test]
    fn ui_focus_loss_and_return_release_mouse_and_do_not_replay_held_gestures() {
        let (mut app, window, zone) = app();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        frame(&mut app, Vec2::new(10., 0.), 0.);
        frame(&mut app, Vec2::new(30., 0.), 0.);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        frame(&mut app, Vec2::new(900., 900.), 10.);
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::None
        );
        let before = app
            .world()
            .resource::<Session>()
            .play
            .as_ref()
            .unwrap()
            .camera
            .angles();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        frame(&mut app, Vec2::new(100., 100.), 0.);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .camera
                .angles(),
            before
        );
        app.world_mut().resource_mut::<Session>().play = None;
        frame(&mut app, Vec2::ZERO, 0.);
        assert!(app.world().get::<CursorOptions>(window).unwrap().visible);
        assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
    }
    #[test]
    fn wheel_and_bindings_match_game_and_ui_cannot_start_a_world_drag() {
        let (mut app, window, _) = app();
        frame(&mut app, Vec2::ZERO, 100.);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .camera
                .distance(),
            2.5
        );
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(30., 20.)));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        frame(&mut app, Vec2::new(100., 100.), -100.);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .camera
                .distance(),
            2.5
        );
        assert!(
            !app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .look[1]
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::NumpadDivide);
        frame(&mut app, Vec2::ZERO, 0.);
        assert!(
            !app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .input
                .sprint
        );
        app.world_mut().resource_mut::<Session>().exit_confirmation = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        frame(&mut app, Vec2::ZERO, 0.);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .input
                .axes,
            [0, 0]
        );
    }
    #[test]
    fn short_jump_press_survives_render_frames_until_fixed_tick_consumes_it() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut app, _, _) = app();
        let start = app
            .world()
            .resource::<Session>()
            .play
            .as_ref()
            .unwrap()
            .feet;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        frame(&mut app, Vec2::ZERO, 0.);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Space);
        frame(&mut app, Vec2::ZERO, 0.);
        assert!(
            app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .input
                .jump
        );
        app.world_mut().run_system_once(playtest::step).unwrap();
        let play = app.world().resource::<Session>().play.as_ref().unwrap();
        assert!(!play.input.jump);
        assert!(play.feet.metres()[1] > start.metres()[1]);
        frame(&mut app, Vec2::ZERO, 0.);
        assert!(
            !app.world()
                .resource::<Session>()
                .play
                .as_ref()
                .unwrap()
                .input
                .jump
        );
    }
    #[test]
    fn playtest_camera_retracts_against_actual_world_collision() {
        let mut zone = Zone::default();
        zone.objects.push(Object {
            id: 1,
            name: "Wall".into(),
            primitive: Primitive::Box,
            pose: Pose {
                position: [0., 3., 10.],
                yaw_degrees: 0.,
                size: [10., 6., 0.5],
            },
        });
        let play = playtest::Playtest::new(&zone).unwrap();
        let target = Vec3::from_array(play.feet.metres()) + Vec3::Y;
        assert!(play.camera_fraction(target, play.camera.unobstructed_eye(target)) < 0.6);
    }
}
