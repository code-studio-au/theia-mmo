//! Bevy presentation: offline sandbox or server-owned multiplayer walking.
mod movement;
mod network;
mod scene;
mod smoke;

use bevy::{app::AppExit, prelude::*, time::TimeUpdateStrategy, window::WindowResolution};
use movement::{LocalPawn, MoveIntent};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let connected = args
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "--connect" | "--network-smoke"));
    let net = if connected {
        Some(network::Network::from_args(&args)?)
    } else {
        None
    };
    let smoke = if connected {
        None
    } else {
        smoke::Run::from_args(args.into_iter())?
    };
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
        .add_systems(FixedUpdate, movement::move_local_pawn);
    if let Some(net) = net {
        app.insert_resource(net)
            .add_systems(
                Update,
                (network::send_input, network::present, follow_camera).chain(),
            )
            .add_systems(Last, network::capture);
    } else {
        app.add_systems(Update, follow_camera);
    }
    if let Some(run) = smoke {
        app.insert_resource(run)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )))
            .add_systems(Last, smoke::observe);
    }
    info!("Theia; connected mode: {connected}; offline smoke mode: {scripted}");
    match app.run() {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => Err(format!("Client exited with code {code}").into()),
    }
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<MoveIntent>,
    smoke: Option<Res<smoke::Run>>,
    network: Option<Res<network::Network>>,
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
    if network.is_none()
        && keys.just_pressed(KeyCode::KeyR)
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
