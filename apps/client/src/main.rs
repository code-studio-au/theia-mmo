//! Bevy presentation: offline sandbox or server-owned multiplayer walking.
mod asset_preview;
mod camera;
mod capture_ready;
mod movement;
mod network;
mod reference_scene;
mod reference_smoke;
mod scene;
mod smoke;
mod terrain_scene;

use bevy::{app::AppExit, prelude::*, time::TimeUpdateStrategy, window::WindowResolution};
use movement::{LocalPawn, MoveIntent};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--inspect-asset") {
        return asset_preview::run(&args);
    }
    let terrain = terrain_scene::Config::parse(&mut args)?;
    let references = reference_scene::Config::parse(&mut args)?;
    if let Some(references) = &references
        && terrain.is_some() != references.players_only
    {
        return Err(
            "Use --player-assets only after --terrain-world; --reference-scene is a separate world"
                .into(),
        );
    }
    let connected = args
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "--connect" | "--network-smoke"));
    if terrain.is_some() && !connected {
        return Err("Terrain walking currently requires --connect or --network-smoke".into());
    }
    let net = if connected {
        Some(network::Network::from_args(
            &args,
            terrain
                .as_ref()
                .map_or(theia_network::terrain::GRAYBOX_ID, |t| t.terrain.identity),
        )?)
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
    let mut plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Theia MMO — movement sandbox".into(),
            resolution: WindowResolution::new(1280, 720).with_scale_factor_override(1.),
            ..default()
        }),
        ..default()
    });
    if let Some(terrain) = terrain {
        plugins = plugins.set(AssetPlugin {
            file_path: terrain
                .directory
                .to_str()
                .ok_or("Terrain directory must be UTF-8")?
                .to_owned(),
            ..default()
        });
        app.insert_resource(terrain)
            .add_systems(Startup, terrain_scene::load)
            .add_systems(Update, terrain_scene::tick);
    }
    if let Some(references) = references {
        if references.players_only {
            use bevy::asset::{
                AssetApp,
                io::{AssetSourceBuilder, file::FileAssetReader},
            };
            let directory = references.directory.clone();
            app.register_asset_source(
                "players",
                AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(directory.clone()))),
            );
        } else {
            plugins = plugins.set(AssetPlugin {
                file_path: references
                    .directory
                    .to_str()
                    .ok_or("Asset directory must be UTF-8")?
                    .to_owned(),
                ..default()
            });
        }
        if let Some(output) = &references.capture {
            app.insert_resource(reference_smoke::Run::new(output.clone()))
                .add_systems(Last, reference_smoke::observe);
        }
        app.insert_resource(references)
            .add_systems(Startup, reference_scene::load)
            .add_systems(Update, reference_scene::tick.after(network::present));
    }
    app.insert_resource(ClearColor(Color::srgb(0.16, 0.23, 0.33)))
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .init_resource::<MoveIntent>()
        .init_resource::<camera::Orbit>()
        .add_plugins(plugins)
        .add_plugins(capture_ready::CaptureReadyPlugin)
        .add_systems(Startup, scene::setup)
        .add_systems(
            PreUpdate,
            (camera::controls, input)
                .chain()
                .after(bevy::input::InputSystems),
        )
        .add_systems(FixedUpdate, movement::move_local_pawn);
    if let Some(net) = net {
        app.insert_resource(net)
            .add_systems(
                Update,
                (network::send_input, network::present, camera::follow).chain(),
            )
            .add_systems(Last, network::capture);
    } else {
        app.add_systems(Update, camera::follow);
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

#[derive(bevy::ecs::system::SystemParam)]
struct InputMode<'w> {
    smoke: Option<Res<'w, smoke::Run>>,
    network: Option<Res<'w, network::Network>>,
    references: Option<Res<'w, reference_scene::Config>>,
    ready: Option<Res<'w, reference_scene::Ready>>,
    reference_smoke: Option<Res<'w, reference_smoke::Run>>,
    terrain: Option<Res<'w, terrain_scene::Config>>,
    terrain_ready: Option<Res<'w, terrain_scene::Ready>>,
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<MoveIntent>,
    mode: InputMode,
    mut pawn: Query<&mut Transform, With<LocalPawn>>,
    mut exit: MessageWriter<AppExit>,
    orbit: Res<camera::Orbit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if (mode.references.is_some() && mode.ready.is_none())
        || (mode.terrain.is_some() && mode.terrain_ready.is_none())
    {
        intent.direction = Vec2::ZERO;
        intent.sprint = false;
        return;
    }
    if let Some(run) = mode.reference_smoke {
        (intent.direction, intent.sprint) = run.intent();
        return;
    }
    if let Some(run) = mode.smoke {
        intent.direction = if run.world() {
            match run.frame {
                60..144 => Vec2::NEG_X,
                144..264 => Vec2::NEG_Y,
                _ => Vec2::ZERO,
            }
        } else {
            match run.frame {
                60..120 => Vec2::NEG_Y,
                120..180 => Vec2::X,
                _ => Vec2::ZERO,
            }
        };
        intent.sprint = false;
        return;
    }
    let held = |a, b| f32::from(u8::from(keys.pressed(a) || keys.pressed(b)));
    intent.direction = orbit.world_direction(Vec2::new(
        held(KeyCode::KeyD, KeyCode::ArrowRight) - held(KeyCode::KeyA, KeyCode::ArrowLeft),
        held(KeyCode::KeyS, KeyCode::ArrowDown) - held(KeyCode::KeyW, KeyCode::ArrowUp),
    ));
    intent.sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if mode.network.is_none()
        && keys.just_pressed(KeyCode::KeyR)
        && let Ok(mut pawn) = pawn.single_mut()
    {
        pawn.translation = movement::START;
    }
}
