//! Offline movement sandbox. Gameplay authority belongs to the future server.
mod movement;
mod scene;
mod smoke;

use bevy::{app::AppExit, prelude::*, time::TimeUpdateStrategy, window::WindowResolution};
use movement::{LocalPawn, MoveIntent};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = smoke::Run::from_args(std::env::args().skip(1))?;
    let scripted = smoke.is_some();
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.16, 0.23, 0.33)))
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .init_resource::<MoveIntent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Theia MMO — movement sandbox".into(),
                resolution: WindowResolution::new(1280, 720).with_scale_factor_override(1.),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, scene::setup)
        .add_systems(PreUpdate, input.after(bevy::input::InputSystems))
        .add_systems(FixedUpdate, movement::move_local_pawn)
        .add_systems(Update, follow_camera);
    if let Some(run) = smoke {
        app.insert_resource(run)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )))
            .add_systems(Last, smoke::observe);
    }
    info!("Theia offline sandbox; scripted smoke mode: {scripted}; no server connection");
    match app.run() {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => Err(format!("Client exited with code {code}").into()),
    }
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<MoveIntent>,
    smoke: Option<Res<smoke::Run>>,
    mut pawn: Query<&mut Transform, With<LocalPawn>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if let Some(run) = smoke {
        intent.direction = match run.frame {
            60..120 => Vec2::NEG_Y,
            120..180 => Vec2::X,
            _ => Vec2::ZERO,
        };
        intent.sprint = false;
        return;
    }
    let held = |a, b| f32::from(u8::from(keys.pressed(a) || keys.pressed(b)));
    intent.direction = Vec2::new(
        held(KeyCode::KeyD, KeyCode::ArrowRight) - held(KeyCode::KeyA, KeyCode::ArrowLeft),
        held(KeyCode::KeyS, KeyCode::ArrowDown) - held(KeyCode::KeyW, KeyCode::ArrowUp),
    );
    intent.sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if keys.just_pressed(KeyCode::KeyR)
        && let Ok(mut pawn) = pawn.single_mut()
    {
        pawn.translation = movement::START;
    }
}

fn follow_camera(
    pawn: Query<&Transform, With<LocalPawn>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<LocalPawn>)>,
) {
    if let (Ok(pawn), Ok(mut camera)) = (pawn.single(), camera.single_mut()) {
        *camera = Transform::from_translation(pawn.translation + Vec3::new(0., 7., 12.))
            .looking_at(pawn.translation, Vec3::Y);
    }
}
