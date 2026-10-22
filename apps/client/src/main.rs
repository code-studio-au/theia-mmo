//! Bevy presentation: offline sandbox or server-owned multiplayer walking.
mod asset_preview;
mod camera;
mod capture_ready;
mod chunk_scene;
mod chunk_world;
mod floating_origin;
mod graybox;
mod movement;
mod network;
mod physics;
mod reference_scene;
mod reference_smoke;
mod runtime_materials;
mod scene;
mod shared_images;
mod smoke;
mod terrain_scene;
mod triplanar;

use bevy::{app::AppExit, prelude::*, time::TimeUpdateStrategy, window::WindowResolution};
use movement::{LocalPawn, MoveIntent};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--inspect-asset") {
        return asset_preview::run(&args);
    }
    if args.first().is_some_and(|a| a == "--chunk-smoke") {
        return chunk_scene::run(&args);
    }
    let graybox = if let Some(i) = args.iter().position(|a| a == "--graybox-world") {
        args.remove(i);
        true
    } else {
        false
    };
    let prediction = if let Some(i) = args.iter().position(|a| a == "--prediction") {
        args.remove(i);
        true
    } else {
        false
    };
    if prediction && !graybox {
        return Err("Prediction requires the graybox world".into());
    }
    let camera_collision = camera::CollisionMode::parse(&mut args)?;
    let chunks = chunk_world::Config::parse(&mut args)?;
    let terrain = terrain_scene::Config::parse(&mut args)?;
    if chunks.is_some() && terrain.is_some() {
        return Err("Choose either chunk or monolithic terrain world".into());
    }
    if graybox && (chunks.is_some() || terrain.is_some()) {
        return Err("Choose one development world".into());
    }
    let content_world = graybox || chunks.is_some() || terrain.is_some();
    let references = reference_scene::Config::parse(&mut args)?;
    if let Some(references) = &references
        && content_world != references.players_only
    {
        return Err(
            "Use --player-assets only after --terrain-world or --chunk-world; --reference-scene is a separate world"
                .into(),
        );
    }
    if chunks.is_some() && args.first().is_none_or(|a| !a.starts_with("--renet-")) {
        return Err("Chunk worlds require the Renet readiness transport".into());
    }
    let connected = args.first().is_some_and(|arg| {
        matches!(
            arg.as_str(),
            "--connect" | "--network-smoke" | "--renet-connect" | "--renet-smoke"
        )
    });
    if content_world && !connected {
        return Err(
            "World walking requires --connect, --renet-connect or a network smoke mode".into(),
        );
    }
    if graybox && args.first().is_none_or(|a| !a.starts_with("--renet-")) {
        return Err("Graybox requires secure Renet".into());
    }
    let mut net = if connected {
        Some(network::Network::from_args(
            &args,
            prediction,
            if graybox {
                theia_network::procedural::IDENTITY
            } else {
                chunks.as_ref().map_or_else(
                    || {
                        terrain
                            .as_ref()
                            .map_or(theia_network::terrain::GRAYBOX_ID, |t| t.terrain.identity)
                    },
                    |chunks| chunks.data.identity(),
                )
            },
        )?)
    } else {
        None
    };
    if let (Some(net), Some(terrain)) = (&mut net, &terrain) {
        net.configure_map_access(&terrain.directory)?;
    }
    let smoke = if connected {
        None
    } else {
        smoke::Run::from_args(args.into_iter())?
    };
    let scripted = smoke.is_some();
    let mut app = App::new();
    app.insert_resource(camera_collision.unwrap_or(if graybox || chunks.is_some() {
        camera::CollisionMode::World
    } else {
        camera::CollisionMode::Visual
    }));
    if graybox {
        graybox::install(&mut app);
    }
    let material_diagnostic = std::env::var("THEIA_RUNTIME_TRIPLANAR_DIAGNOSTIC")
        .ok()
        .map(|v| v.parse::<u32>())
        .transpose()?
        .unwrap_or(0);
    if material_diagnostic > 2
        || (material_diagnostic != 0 && net.as_ref().is_none_or(|n| n.smoke.is_none()))
    {
        return Err(
            "Runtime material diagnostics require a network smoke and mode 0, 1 or 2".into(),
        );
    }
    app.insert_resource(runtime_materials::Diagnostic(material_diagnostic));
    app.insert_resource(floating_origin::Frame::from_env()?);
    let performance_uncapped = std::env::var("THEIA_PERFORMANCE_UNCAPPED").as_deref() == Ok("1");
    if performance_uncapped {
        app.insert_resource(bevy::winit::WinitSettings::continuous());
    }
    let mut plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: if content_world {
                "Theia MMO — development map"
            } else {
                "Theia MMO — movement sandbox"
            }
            .into(),
            resolution: WindowResolution::new(1280, 720).with_scale_factor_override(1.),
            present_mode: if performance_uncapped {
                bevy::window::PresentMode::AutoNoVsync
            } else {
                bevy::window::PresentMode::default()
            },
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
            .add_systems(
                Update,
                (terrain_scene::tick, terrain_scene::mark_camera_scenery),
            );
    }
    if let Some(chunks) = chunks {
        plugins = plugins.set(AssetPlugin {
            // Prepared packages bind source bytes and explicit loader variants;
            // unbound editor metadata must not change those image settings.
            meta_check: bevy::asset::AssetMetaCheck::Never,
            file_path: chunks
                .directory
                .to_str()
                .ok_or("Chunk directory must be UTF-8")?
                .to_owned(),
            ..default()
        });
        app.insert_resource(chunks)
            .add_systems(Startup, chunk_world::load)
            .add_systems(
                Update,
                (chunk_world::tick, terrain_scene::mark_camera_scenery),
            )
            .add_systems(
                Update,
                chunk_world::validate_initial.after(chunk_world::tick),
            )
            .add_systems(
                Update,
                (chunk_world::drive, chunk_world::remote_visibility)
                    .chain()
                    .after(network::present)
                    .after(chunk_world::validate_initial),
            )
            .add_systems(
                Last,
                chunk_world::report_ready
                    .after(physics::probe)
                    .before(network::capture),
            );
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
        .init_resource::<camera::SceneryCameraStats>()
        .add_plugins(plugins)
        .add_plugins(physics::WorldPhysicsPlugin)
        .add_plugins(capture_ready::CaptureReadyPlugin)
        .add_plugins(graybox::ReadinessPlugin)
        .add_plugins((
            triplanar::TriplanarPlugin,
            runtime_materials::RuntimeMaterialsPlugin,
        ))
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
                (
                    floating_origin::select,
                    network::send_input,
                    network::present,
                )
                    .chain(),
            )
            .add_systems(Last, network::capture);
    }
    app.add_systems(
        Update,
        floating_origin::place_static
            .after(network::present)
            .after(reference_scene::tick)
            .after(terrain_scene::tick)
            .after(chunk_world::tick)
            .after(chunk_world::drive),
    )
    .add_systems(
        PostUpdate,
        camera::follow
            .after(avian3d::prelude::PhysicsSystems::Last)
            .before(bevy::transform::TransformSystems::Propagate),
    )
    .add_systems(
        PostUpdate,
        (camera::scenery_clearance, camera::sync_camera_global)
            .chain()
            .after(bevy::transform::TransformSystems::Propagate),
    );
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
    chunks: Option<Res<'w, chunk_world::Config>>,
    chunks_ready: Option<Res<'w, chunk_world::Ready>>,
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<MoveIntent>,
    mode: InputMode,
    mut pawn: Query<(&mut Transform, &mut movement::Body), With<LocalPawn>>,
    mut exit: MessageWriter<AppExit>,
    orbit: Res<camera::Orbit>,
) {
    intent.jump = false;
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if (mode.references.is_some() && mode.ready.is_none())
        || (mode.terrain.is_some() && mode.terrain_ready.is_none())
        || (mode.chunks.is_some() && mode.chunks_ready.is_none())
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
    intent.jump = keys.pressed(KeyCode::Space);
    intent.sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if mode.network.is_none()
        && keys.just_pressed(KeyCode::KeyR)
        && let Ok((mut pawn, mut body)) = pawn.single_mut()
    {
        pawn.translation = movement::START;
        *body = Default::default();
    }
}
