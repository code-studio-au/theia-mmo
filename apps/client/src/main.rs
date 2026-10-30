//! Bevy presentation: offline sandbox or server-owned multiplayer walking.
mod asset_preview;
mod camera;
mod capture_ready;
mod chunk_scene;
mod chunk_world;
mod combat;
mod dialogue;
mod equipment;
mod floating_origin;
mod graybox;
mod inventory;
mod movement;
mod movement_metrics;
mod network;
mod physics;
mod progression;
mod quests;
mod reference_scene;
mod reference_smoke;
mod runtime_materials;
mod scene;
mod shared_images;
mod smoke;
mod spell_feedback;
mod status_effects;
mod targeting;
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
    let fullscreen = if let Some(i) = args.iter().position(|a| a == "--fullscreen") {
        args.remove(i);
        true
    } else {
        false
    };
    let window_size = if let Some(i) = args.iter().position(|a| a == "--window-size") {
        args.remove(i);
        if i >= args.len() {
            return Err("Missing window size (WIDTHxHEIGHT)".into());
        }
        args.remove(i)
    } else {
        std::env::var("THEIA_WINDOW_SIZE").unwrap_or_else(|_| "1920x1200".into())
    };
    let (width, height) = window_size
        .split_once('x')
        .ok_or("Expected window size WIDTHxHEIGHT")?;
    let (width, height): (u32, u32) = (width.parse()?, height.parse()?);
    if !(640..=7680).contains(&width) || !(480..=4320).contains(&height) {
        return Err("Window size outside 640x480 to 7680x4320".into());
    }
    let saved = if let Some(i) = args.iter().position(|a| a == "--saved-terrain") {
        args.remove(i);
        if i >= args.len() {
            return Err("Missing saved terrain directory".into());
        }
        Some(std::sync::Arc::new(
            theia_network::saved_terrain::Store::open(std::path::Path::new(&args.remove(i)))
                .map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let graybox_flag = if let Some(i) = args.iter().position(|a| a == "--graybox-world") {
        args.remove(i);
        true
    } else {
        false
    };
    if graybox_flag && saved.is_some() {
        return Err("Choose generated or saved terrain".into());
    }
    let graybox = graybox_flag || saved.is_some();
    let source = saved.map_or(
        theia_network::procedural::Source::Procedural,
        theia_network::procedural::Source::Saved,
    );
    let prediction = if let Some(i) = args.iter().position(|a| a == "--prediction") {
        args.remove(i);
        true
    } else {
        false
    };
    if prediction && !graybox {
        return Err("Prediction requires the graybox world".into());
    }
    let combat_catalog = if let Some(i) = args.iter().position(|a| a == "--combat-catalog") {
        args.remove(i);
        if i >= args.len() {
            return Err("Missing combat catalog".into());
        }
        Some(combat::load_catalog(std::path::Path::new(&args.remove(i)))?)
    } else {
        None
    };
    let character_stats = if let Some(i) = args.iter().position(|a| a == "--character-stats") {
        args.remove(i);
        if i >= args.len() {
            return Err("Missing character stats profile".into());
        }
        if combat_catalog.is_none() {
            return Err("Character stats require combat".into());
        }
        Some(combat::load_resources(std::path::Path::new(
            &args.remove(i),
        ))?)
    } else {
        None
    };
    let status_catalog = if let Some(i) = args.iter().position(|a| a == "--status-catalog") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing status catalog".into());
        }
        if character_stats.is_none() {
            return Err("Statuses require combat and character stats".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::statuses::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let catalog = std::sync::Arc::new(theia_core::statuses::Catalog::from_json(&bytes)?);
        Some((bytes, catalog))
    } else {
        None
    };
    let patrol_binding = if let Some(i) = args.iter().position(|a| a == "--creature-profile") {
        use std::io::Read;
        use theia_network::simulation::creatures;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing creature profile".into());
        }
        if !graybox || !prediction || combat_catalog.is_none() {
            return Err("Creature patrol requires predicted graybox combat".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(creatures::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let profile = creatures::Profile::from_json(&bytes).map_err(|e| e.to_string())?;
        let i = args
            .iter()
            .position(|a| a == "--navigation")
            .ok_or("Creature patrol requires navigation package")?;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing navigation package".into());
        }
        let navigation = theia_network::navigation::Store::open(
            std::path::Path::new(&args.remove(i)),
            source.identity(),
        )
        .map_err(|e| e.to_string())?;
        Some((profile, navigation.identity()))
    } else {
        if args.iter().any(|a| a == "--navigation") {
            return Err("Navigation package requires a creature profile".into());
        }
        None
    };
    let progression_profile = if let Some(i) =
        args.iter().position(|a| a == "--progression-catalog")
    {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing progression catalog".into());
        }
        if !graybox || !prediction || patrol_binding.is_none() || combat_catalog.is_none() {
            return Err("Progression requires predicted saved terrain and creature combat".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::progression::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Some(std::sync::Arc::new(
            theia_network::progression_wire::Profile::from_json(&bytes)
                .map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let dialogue_profile = if let Some(i) = args.iter().position(|a| a == "--dialogue-catalog") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() || !graybox || !prediction {
            return Err("Dialogue requires catalog and predicted graybox".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::dialogue::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let profile = std::sync::Arc::new(
            theia_network::dialogue::Profile::from_json(&bytes).map_err(|e| e.to_string())?,
        );
        let i = args
            .iter()
            .position(|a| a == "--guide-position")
            .ok_or("Dialogue requires explicit guide position")?;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing guide position".into());
        }
        let values: Vec<f64> = args
            .remove(i)
            .split(',')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let xz: [f64; 2] = values.try_into().map_err(|_| "Expected guide x,z")?;
        let focus = theia_core::coordinates::GlobalPosition::new([xz[0], 0., xz[1]])
            .ok_or("Invalid guide position")?;
        if theia_network::procedural::owner(focus).is_none() {
            return Err("Guide outside graybox".into());
        }
        Some((profile, xz))
    } else {
        if args.iter().any(|a| a == "--guide-position") {
            return Err("Guide position requires dialogue".into());
        }
        None
    };
    let quest_profile = if let Some(i) = args.iter().position(|a| a == "--quest-catalog") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing quest catalog".into());
        }
        let xp = progression_profile
            .as_ref()
            .ok_or("Objectives require progression")?;
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::quests::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let profile = std::sync::Arc::new(
            theia_network::quest_wire::Profile::from_json(&bytes).map_err(|e| e.to_string())?,
        );
        if xp
            .profile()
            .kill_experience(profile.profile().creature())
            .is_none()
        {
            return Err("Objective/progression creature mismatch".into());
        }
        Some(profile)
    } else {
        None
    };
    let mut inventory_profile = if let Some(i) = args.iter().position(|a| a == "--item-catalog") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing item catalog".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::inventory::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Some(
            theia_network::renet_runtime::inventory::Profile::from_json(&bytes)
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    if let Some(i) = args.iter().position(|a| a == "--reward-catalog") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() {
            return Err("Missing reward catalog".into());
        }
        if patrol_binding.is_none() || combat_catalog.is_none() {
            return Err("Rewards require authoritative creature combat".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::rewards::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let profile = inventory_profile.take().ok_or("Rewards require items")?;
        inventory_profile = Some(profile.with_rewards(&bytes).map_err(|e| e.to_string())?);
    }
    if let Some(i) = args.iter().position(|a| a == "--vendor-catalog") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() || dialogue_profile.is_none() {
            return Err("Vendor requires catalog and dialogue".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::vendors::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let profile = inventory_profile
            .take()
            .ok_or("Vendor requires items")?
            .with_vendor(&bytes)
            .map_err(|e| e.to_string())?;
        if dialogue_profile
            .as_ref()
            .is_none_or(|(d, _)| d.profile().npc() != profile.vendor().unwrap().catalog().npc())
        {
            return Err("Vendor NPC differs from dialogue guide".into());
        }
        inventory_profile = Some(profile);
    }
    if inventory_profile.is_some() && (!graybox || !prediction) {
        return Err("Inventory requires predicted saved terrain".into());
    }
    let feedback = if let Some(i) = args.iter().position(|a| a == "--spell-feedback") {
        args.remove(i);
        if i >= args.len() {
            return Err("Missing spell feedback definition".into());
        }
        let (_, catalog) = combat_catalog
            .as_ref()
            .ok_or("Spell feedback requires combat")?;
        Some(spell_feedback::data::Definition::load(
            std::path::Path::new(&args.remove(i)),
            catalog,
        )?)
    } else {
        None
    };
    let targeting_flag = if let Some(i) = args.iter().position(|a| a == "--targeting") {
        args.remove(i);
        true
    } else {
        false
    };
    let targeted = targeting_flag || combat_catalog.is_some();
    if targeted && !graybox {
        return Err("Targeting requires a graybox world".into());
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
    if inventory_profile.is_some() && !connected {
        return Err("Inventory requires a network connection".into());
    }
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
                let base = if targeted {
                    theia_network::targeting::profile_identity(source.identity())
                } else {
                    source.identity()
                };
                let base = patrol_binding
                    .as_ref()
                    .map_or(base, |(profile, navigation)| {
                        theia_network::simulation::creatures::profile_identity(
                            base,
                            profile,
                            *navigation,
                        )
                    });
                let base = if let Some((bytes, _)) = &combat_catalog {
                    use theia_network::simulation::combat::{
                        self, DEVELOPMENT_HEALTH, DEVELOPMENT_RESPAWN_MS,
                    };
                    if let Some((stats, _)) = &character_stats {
                        if let Some((statuses, _)) = &status_catalog {
                            combat::status_profile_identity(
                                base,
                                bytes,
                                DEVELOPMENT_HEALTH,
                                DEVELOPMENT_RESPAWN_MS,
                                stats,
                                statuses,
                            )
                        } else {
                            combat::resource_profile_identity(
                                base,
                                bytes,
                                DEVELOPMENT_HEALTH,
                                DEVELOPMENT_RESPAWN_MS,
                                stats,
                            )
                        }
                    } else {
                        combat::profile_identity(
                            base,
                            bytes,
                            DEVELOPMENT_HEALTH,
                            DEVELOPMENT_RESPAWN_MS,
                        )
                    }
                    .map_err(|e| e.to_string())?
                } else {
                    base
                };
                let base = inventory_profile
                    .as_ref()
                    .and_then(|p| p.rewards())
                    .map_or(base, |r| {
                        theia_network::simulation::rewards::profile_identity(base, r.hash())
                    });
                let base = progression_profile.as_ref().map_or(base, |p| {
                    theia_network::progression_wire::profile_identity(base, p.hash())
                });
                let base = quest_profile.as_ref().map_or(base, |p| {
                    theia_network::quest_wire::profile_identity(base, p.hash())
                });
                let base = dialogue_profile.as_ref().map_or(base, |(profile, xz)| {
                    theia_network::dialogue::profile_identity(base, profile.hash(), *xz)
                });
                inventory_profile
                    .as_ref()
                    .and_then(|p| p.vendor())
                    .map_or(base, |v| {
                        theia_network::vendor_wire::profile_identity(base, v.hash())
                    })
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
            targeted,
            combat_catalog.as_ref().map(|(_, catalog)| catalog.clone()),
            inventory_profile.clone(),
            status_catalog.as_ref().map(|(_, catalog)| catalog.clone()),
            progression_profile.clone(),
            quest_profile.clone(),
            dialogue_profile
                .as_ref()
                .map(|(profile, _)| profile.clone()),
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
        graybox::install(&mut app, source);
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
            resolution: WindowResolution::new(width, height).with_scale_factor_override(1.),
            mode: if fullscreen {
                bevy::window::WindowMode::BorderlessFullscreen(
                    bevy::window::MonitorSelection::Current,
                )
            } else {
                bevy::window::WindowMode::Windowed
            },
            present_mode: if performance_uncapped {
                bevy::window::PresentMode::AutoNoVsync
            } else {
                bevy::window::PresentMode::default()
            },
            ..default()
        }),
        ..default()
    });
    if feedback.is_some() {
        plugins = plugins.set(spell_feedback::audio_plugin());
    } else {
        plugins = plugins.disable::<bevy::audio::AudioPlugin>();
    }
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
    if targeted {
        targeting::install(
            &mut app,
            patrol_binding
                .as_ref()
                .map(|(profile, _)| profile.speed_metres_per_second()),
        );
    }
    if let Some((_, catalog)) = status_catalog {
        status_effects::install(&mut app, catalog);
    }
    if let Some((profile, _)) = dialogue_profile {
        dialogue::install(&mut app, profile);
    }
    if let Some(profile) = quest_profile {
        quests::install(&mut app, profile);
    }
    if let Some(profile) = progression_profile {
        progression::install(&mut app, profile);
    }
    if let Some(profile) = inventory_profile {
        if profile.catalog().equipment_enabled() {
            equipment::install(&mut app, profile.catalog())?;
        }
        inventory::install(&mut app, profile);
    }
    if let Some((_, catalog)) = combat_catalog {
        combat::install(
            &mut app,
            catalog,
            character_stats.map(|(_, profile)| profile),
        );
    }
    if let Some(definition) = feedback {
        spell_feedback::install(&mut app, definition);
    }
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
    app.add_systems(Update, toggle_fullscreen)
        .add_systems(
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

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::F11)
        && let Ok(mut window) = windows.single_mut()
    {
        window.mode = match window.mode {
            bevy::window::WindowMode::Windowed => bevy::window::WindowMode::BorderlessFullscreen(
                bevy::window::MonitorSelection::Current,
            ),
            _ => bevy::window::WindowMode::Windowed,
        };
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct InputMode<'w> {
    dialogue: Option<Res<'w, dialogue::Ui>>,
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
    let talking = mode
        .dialogue
        .as_ref()
        .is_some_and(|ui| ui.open || ui.pending || ui.close_when_open)
        || mode.network.as_ref().is_some_and(|n| {
            let s = n.state();
            s.error.is_none()
                && (s.dialogue_pending || s.dialogue.is_some_and(|v| v.open.is_some()))
        });
    if keys.just_pressed(KeyCode::Escape) && !talking {
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
