use super::*;
fn complete() -> Checks {
    Checks {
        connection: true,
        world: true,
        characters: true,
        renderer: true,
        authority: true,
    }
}
#[test]
fn entry_requires_every_actual_dependency_and_never_reopens_over_play() {
    for missing in 0..5 {
        let mut checks = complete();
        match missing {
            0 => checks.connection = false,
            1 => checks.world = false,
            2 => checks.characters = false,
            3 => checks.renderer = false,
            _ => checks.authority = false,
        }
        let mut state = State::default();
        state.advance(checks, None, false);
        assert!(state.blocks_input());
        assert!(state.entered_with.is_none());
        for _ in 0..4 {
            state.advance(complete(), None, false);
        }
        assert_eq!(state.phase, Phase::Playing);
        assert_eq!(state.entered_with, Some(complete()));
        state.advance(Checks::default(), Some("Lost readiness"), true);
        assert_eq!(state.phase, Phase::Playing);
        assert!(!state.fail("Post-entry fault"));
    }
}
#[test]
fn failure_wins_over_readiness_and_remains_visible_until_close() {
    for (error, timeout) in [(Some("Connection failed"), false), (None, true)] {
        let mut state = State::default();
        state.advance(complete(), error, timeout);
        assert_eq!(state.phase, Phase::Failed);
        assert!(state.blocks_input());
        let label = state.label().to_owned();
        state.advance(complete(), None, false);
        state.fail("Later error");
        assert_eq!(state.label(), label);
        assert!(state.entered_with.is_none());
    }
}
fn admitted() -> theia_network::client::State {
    use theia_network::wire::*;
    let report = ChunkReady {
        version: VERSION,
        epoch: 7,
        sequence: 2,
        chunks: vec![[0, 0]],
    };
    theia_network::client::State {
        id: Some(3),
        welcome: Some(Welcome {
            terrain_stream: true,
            id: 3,
            epoch: 7,
            world: [1; 32],
            origin: [0.; 3],
        }),
        snapshot: Some(Snapshot {
            version: VERSION,
            epoch: 7,
            tick: 8,
            tick_hz: 30,
            origin: [0.; 3],
            players: vec![Player {
                facing: None,
                id: 3,
                xz: [3., 4.],
                y: 0.9,
                applied_sequence: 0,
            }],
        }),
        chunk_ack: Some(ChunkAck::new(&report, 8)),
        chunk_report: Some(report),
        ..default()
    }
}
#[test]
fn authority_gate_rejects_stale_epoch_report_tick_empty_or_wrong_owner() {
    assert!(authority_ready(&admitted(), false));
    assert!(!authority_ready(&admitted(), true)); // Predicted owners also need their correction.
    for variant in 0..7 {
        let mut state = admitted();
        match variant {
            0 => state.chunk_ack = None,
            1 => state.chunk_ack.as_mut().unwrap().epoch += 1,
            2 => state.chunk_report.as_mut().unwrap().sequence += 1,
            3 => state.snapshot.as_mut().unwrap().tick -= 1,
            4 => state.chunk_report.as_mut().unwrap().chunks.clear(),
            5 => state.snapshot.as_mut().unwrap().players[0].xz = [128., 4.],
            _ => state.welcome.as_mut().unwrap().epoch += 1,
        }
        assert!(!authority_ready(&state, false), "variant {variant}");
    }
}
#[test]
fn keyboard_is_owned_during_loading_and_cancel_exits_without_gameplay_escape() {
    let mut app = App::new();
    app.init_resource::<State>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<AppExit>()
        .add_message::<bevy::input::keyboard::KeyboardInput>()
        .add_systems(Update, shield_keyboard);
    app.world_mut().spawn(Window {
        focused: true,
        ..default()
    });
    for key in [
        KeyCode::KeyW,
        KeyCode::KeyO,
        KeyCode::Digit1,
        KeyCode::Escape,
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .get_pressed()
            .next()
            .is_none()
    );
    assert_eq!(app.world().resource::<State>().blocked_key_frames, 1);
    assert!(
        app.world()
            .resource::<Messages<AppExit>>()
            .iter_current_update_messages()
            .any(|e| matches!(e, AppExit::Success))
    );
    for _ in 0..4 {
        app.world_mut()
            .resource_mut::<State>()
            .advance(complete(), None, false);
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyA);
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::KeyA)
    );
}
#[test]
fn asset_failure_is_visible_only_for_interactive_startup() {
    let mut world = World::new();
    world.init_resource::<Messages<AppExit>>();
    fail_or_exit(&mut world, "No overlay");
    assert_eq!(world.resource::<Messages<AppExit>>().len(), 1);
    world.resource_mut::<Messages<AppExit>>().clear();
    world.init_resource::<State>();
    fail_or_exit(&mut world, "Missing characters");
    assert!(world.resource::<Messages<AppExit>>().is_empty());
    assert_eq!(world.resource::<State>().phase, Phase::Failed);
    world.resource_mut::<State>().phase = Phase::Playing;
    fail_or_exit(&mut world, "Later asset fault");
    assert_eq!(world.resource::<Messages<AppExit>>().len(), 1);
}

#[test]
fn held_key_repeat_cannot_escape_entry_until_physical_release() {
    use bevy::input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    };
    let mut app = App::new();
    app.init_resource::<State>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<AppExit>()
        .add_message::<KeyboardInput>()
        .add_systems(Update, shield_keyboard);
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            ..default()
        })
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();
    for _ in 0..4 {
        app.world_mut()
            .resource_mut::<State>()
            .advance(complete(), None, false);
    }
    let event = |state, repeat| KeyboardInput {
        key_code: KeyCode::KeyW,
        logical_key: Key::Character("w".into()),
        state,
        text: None,
        repeat,
        window,
    };
    app.world_mut()
        .write_message(event(ButtonState::Pressed, true));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::KeyW)
    );
    app.world_mut()
        .write_message(event(ButtonState::Released, false));
    app.update();
    app.world_mut()
        .write_message(event(ButtonState::Pressed, false));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::KeyW)
    );
}

/// Widget-only native acceptance. `fail.request` deliberately exercises the error
/// presentation; real admission/readiness is qualified separately with two accounts.
#[test]
#[ignore = "requires owned desktop and THEIA_LOADING_WIDGET_QUALIFY output directory"]
fn native_loading_cancel_and_failure_widgets() {
    let output =
        std::path::PathBuf::from(std::env::var_os("THEIA_LOADING_WIDGET_QUALIFY").unwrap());
    let height = std::env::var("THEIA_LOADING_WIDGET_HEIGHT")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Theia loading widget qualification".into(),
                    visible: false,
                    resolution: (1720, height).into(),
                    decorations: false,
                    position: bevy::window::WindowPosition::At(IVec2::ZERO),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::winit::WinitPlugin {
                run_on_any_thread: true,
            })
            .disable::<bevy::audio::AudioPlugin>(),
    )
    .add_plugins(crate::capture_ready::CaptureReadyPlugin)
    .insert_resource(Artwork::Millhaven)
    .init_resource::<State>()
    .add_systems(
        Startup,
        (ui::setup, |mut commands: Commands| {
            // A conspicuous unfinished world must never leak through startup.
            commands.spawn((
                Camera3d::default(),
                bevy::core_pipeline::tonemapping::Tonemapping::None,
                Camera {
                    clear_color: bevy::camera::ClearColorConfig::Custom(Color::srgb(1., 0., 1.)),
                    ..default()
                },
            ));
        }),
    )
    .add_systems(PostStartup, presentation::sync)
    .add_systems(Last, presentation::reveal)
    .add_systems(
        PreUpdate,
        shield_keyboard
            .after(bevy::input::InputSystems)
            .before(bevy::picking::PickingSystems::ProcessInput),
    )
    .add_systems(Update, (presentation::sync, ui::present).chain())
    .add_observer(ui::cancel);
    let path = output.clone();
    let mut frame = 0;
    app.add_systems(Last, move |world: &mut World| {
        frame += 1;
        if frame <= 8 {
            world
                .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(
                    path.join(format!("startup-{frame:02}.png")),
                ));
        }
        assert!(
            world.resource::<State>().started.elapsed().as_secs() < 60,
            "Owned widget check timed out"
        );
        if path.join("fail.request").exists() {
            world
                .resource_mut::<State>()
                .fail("World assets could not be prepared.");
        }
        let mut value = snapshot(world);
        value["pid"] = std::process::id().into();
        std::fs::write(
            path.join("widget.json"),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
        if path.join("capture.request").exists() {
            std::fs::remove_file(path.join("capture.request")).unwrap();
            world
                .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(
                    path.join("widget.png"),
                ));
        }
    });
    let exit = app.run();
    assert_eq!(
        matches!(exit, AppExit::Error(_)),
        output.join("fail.request").exists()
    );
}

#[test]
fn entry_requires_stable_complete_observations() {
    let mut state = State::default();
    state.advance(complete(), None, false);
    state.advance(complete(), None, false);
    assert!(state.blocks_input());
    state.advance(Checks::default(), None, false);
    assert_eq!(state.settled_frames, 0);
    for _ in 0..4 {
        state.advance(complete(), None, false);
    }
    assert_eq!(state.phase, Phase::Playing);
}

#[test]
fn keyboard_shield_precedes_panel_shortcuts_in_preupdate() {
    let mut app = App::new();
    app.init_resource::<State>()
        .init_resource::<crate::party::Ui>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<AppExit>()
        .add_message::<bevy::input::keyboard::KeyboardInput>()
        .add_systems(PreUpdate, crate::party::keyboard);
    install_input(&mut app);
    app.world_mut().spawn(Window {
        focused: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyO);
    app.update();
    assert!(!app.world().resource::<crate::party::Ui>().open);
}

#[test]
fn f10_reopens_interface_after_loading_and_physical_release() {
    use crate::fantasy_ui::settings;
    use bevy::input::{
        ButtonState, InputPlugin,
        keyboard::{Key, KeyboardInput, NativeKey},
    };
    let mut app = App::new();
    app.add_plugins(InputPlugin)
        .init_resource::<State>()
        .init_resource::<settings::Ui>()
        .add_message::<AppExit>()
        .add_systems(
            PreUpdate,
            settings::keyboard.after(bevy::input::InputSystems),
        );
    install_input(&mut app);
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            ..default()
        })
        .id();
    let event = |state, repeat| KeyboardInput {
        key_code: KeyCode::F10,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat,
        window,
    };
    app.world_mut()
        .write_message(event(ButtonState::Pressed, false));
    app.update();
    assert!(!app.world().resource::<settings::Ui>().open);
    for _ in 0..4 {
        app.world_mut()
            .resource_mut::<State>()
            .advance(complete(), None, false);
    }
    app.world_mut()
        .write_message(event(ButtonState::Pressed, true));
    app.update();
    assert!(!app.world().resource::<settings::Ui>().open);
    app.world_mut()
        .write_message(event(ButtonState::Released, false));
    app.update();
    app.world_mut()
        .write_message(event(ButtonState::Pressed, false));
    app.update();
    assert!(app.world().resource::<settings::Ui>().open);
    app.update();
    assert!(
        app.world().resource::<settings::Ui>().open,
        "Holding F10 must not toggle twice"
    );
    app.world_mut()
        .write_message(event(ButtonState::Released, false));
    app.update();
    app.world_mut()
        .write_message(event(ButtonState::Pressed, false));
    app.update();
    assert!(!app.world().resource::<settings::Ui>().open);
}

#[test]
fn progress_counts_verified_stages_and_reserves_completion_for_stability() {
    let mut state = State::default();
    assert_eq!(state.progress_percent(), 0);
    let mut checks = complete();
    checks.connection = false;
    state.advance(checks, None, false);
    assert_eq!(state.progress_percent(), 0);
    for missing in 1..6 {
        checks = complete();
        match missing {
            1 => checks.world = false,
            2 => checks.characters = false,
            3 => checks.authority = false,
            4 => checks.renderer = false,
            _ => (),
        }
        state.advance(checks, None, false);
        assert_eq!(state.progress_percent(), missing * 100 / 6);
    }
    state.advance(complete(), None, false);
    assert_eq!(state.progress_percent(), 83);
    state.advance(complete(), None, false);
    assert_eq!(state.progress_percent(), 100);
    assert_eq!(
        state.phase,
        Phase::Loading,
        "Show the full bar before entry"
    );
    state.advance(Checks::default(), None, false);
    assert_eq!(state.progress_percent(), 0);
    for _ in 0..3 {
        state.advance(complete(), None, false);
    }
    state.advance(complete(), Some("Lost connection"), false);
    assert_eq!(state.phase, Phase::Failed);
    assert!(state.progress_percent() < 100);
}

#[test]
fn unfinished_world_never_writes_the_window_but_keeps_preparing() {
    use bevy::camera::CameraOutputMode;
    let mut app = App::new();
    app.init_resource::<State>()
        .add_systems(PostStartup, presentation::sync)
        .add_systems(Update, presentation::sync);
    let world_camera = app.world_mut().spawn(Camera3d::default()).id();
    let cover = app
        .world_mut()
        .spawn((Camera2d, presentation::LoadingCamera))
        .id();
    for phase in [Phase::Loading, Phase::Failed, Phase::Playing] {
        app.world_mut().resource_mut::<State>().phase = phase;
        app.update();
        let camera = app.world().get::<Camera>(world_camera).unwrap();
        assert!(camera.is_active, "World preparation must keep running");
        assert_eq!(
            matches!(camera.output_mode, CameraOutputMode::Skip),
            phase != Phase::Playing
        );
        assert_eq!(
            app.world().get::<Camera>(cover).unwrap().is_active,
            phase != Phase::Playing
        );
        assert!(
            app.world().get::<IsDefaultUiCamera>(world_camera).is_some(),
            "Gameplay UI retains its original camera after entry"
        );
    }
    assert!(
        app.world()
            .get::<presentation::WorldOutput>(world_camera)
            .is_none()
    );
}

#[test]
fn native_window_waits_for_rendered_loading_then_relinquishes_visibility() {
    let mut app = App::new();
    app.init_resource::<State>()
        .init_resource::<crate::capture_ready::CaptureReady>()
        .add_systems(Last, presentation::reveal);
    let window = app
        .world_mut()
        .spawn(Window {
            visible: false,
            ..default()
        })
        .id();
    app.update();
    assert!(!app.world().get::<Window>(window).unwrap().visible);
    app.world_mut()
        .resource_mut::<crate::capture_ready::CaptureReady>()
        .clear_frames = 2;
    app.update();
    assert!(!app.world().get::<Window>(window).unwrap().visible);
    app.world_mut()
        .resource_mut::<crate::capture_ready::CaptureReady>()
        .clear_frames = 3;
    app.update();
    assert!(app.world().get::<Window>(window).unwrap().visible);
    app.world_mut().get_mut::<Window>(window).unwrap().visible = false;
    app.update();
    assert!(!app.world().get::<Window>(window).unwrap().visible);
}
