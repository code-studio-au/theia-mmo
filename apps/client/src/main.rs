//! Bevy presentation: offline sandbox or server-owned multiplayer walking.
mod asset_preview;
mod camera;
mod capture_ready;
mod chunk_scene;
mod chunk_world;
mod classic_controls;
mod combat;
mod combat_feedback;
mod creature_visual;
mod dialogue;
mod dodge;
mod equipment;
mod floating_origin;
mod graybox;
mod inventory;
mod melee_qualification;
mod minimap;
mod movement;
mod movement_metrics;
mod network;
mod physics;
mod player_life;
mod progression;
mod projectile_qualification;
mod projectile_visual;
mod pursuit_qualification;
mod quest_journal;
mod quest_lifecycle;
mod quest_marker;
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
mod world_input;

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
    let classic = if let Some(i) = args.iter().position(|a| a == "--classic") {
        args.remove(i);
        true
    } else {
        false
    };
    let classic_controller = classic_controls::Controller::from_args(&mut args, classic)?;
    if classic && !prediction {
        return Err("Classic controls require predicted terrain".into());
    }
    let dodge_profile = if let Some(i) = args.iter().position(|a| a == "--dodge-profile") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() || !prediction {
            return Err("Dodge requires a profile and predicted terrain".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::dodge::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Some(std::sync::Arc::new(
            theia_network::dodging::Profile::from_json(&bytes).map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let combat_catalog = if let Some(i) = args.iter().position(|a| a == "--combat-catalog") {
        args.remove(i);
        if i >= args.len() {
            return Err("Missing combat catalog".into());
        }
        Some(combat::load_catalog(std::path::Path::new(&args.remove(i)))?)
    } else {
        None
    };
    if combat_catalog
        .as_ref()
        .is_some_and(|(_, catalog)| catalog.schema() >= 7)
        && !classic
    {
        return Err("Explicit ability rules require classic controls".into());
    }
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
        Some((profile, navigation.identity(), bytes))
    } else {
        if args.iter().any(|a| a == "--navigation") {
            return Err("Navigation package requires a creature profile".into());
        }
        None
    };
    let creature_asset = creature_visual::parse(&mut args, patrol_binding.is_some())?;
    let player_life_profile = player_life::parse(
        &mut args,
        character_stats.as_ref().map(|(bytes, _)| bytes.as_slice()),
        patrol_binding
            .as_ref()
            .map(|(_, _, bytes)| bytes.as_slice()),
    )?;
    if patrol_binding
        .as_ref()
        .is_some_and(|(profile, _, _)| profile.aggro().is_some())
        && player_life_profile.is_none()
    {
        return Err("Creature pursuit requires player life and retaliation profiles".into());
    }
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
    let explicit_quest = if let Some(i) = args.iter().position(|a| a == "--quest-lifecycle") {
        args.remove(i);
        true
    } else {
        false
    };
    let journal_mode = if let Some(i) = args.iter().position(|a| a == "--quest-journal") {
        args.remove(i);
        true
    } else {
        false
    };
    if journal_mode && explicit_quest {
        return Err("Choose expanded journal or legacy quest mode".into());
    }
    let quest_turn_in_xp = if let Some(i) = args.iter().position(|a| a == "--quest-turn-in-xp") {
        use std::io::Read;
        if !explicit_quest {
            return Err("Completion XP requires explicit quest mode".into());
        }
        args.remove(i);
        if i >= args.len() {
            return Err("Missing completion XP recipe".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::quests::turn_in_xp::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Some(bytes)
    } else {
        None
    };
    let quest_area_bytes = if let Some(i) = args.iter().position(|a| a == "--quest-areas") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() || !journal_mode || !args.iter().any(|a| a == "--quest-catalog") {
            return Err("Quest areas require expanded journal mode and a catalogue".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::quests::areas::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Some(bytes)
    } else {
        None
    };
    let (quest_profile, lifecycle_profile, journal_profile) =
        if let Some(i) = args.iter().position(|a| a == "--quest-catalog") {
            use std::io::Read;
            args.remove(i);
            if i >= args.len() {
                return Err("Missing quest catalog".into());
            }
            let xp = progression_profile
                .as_ref()
                .ok_or("Quests require progression")?;
            let mut bytes = Vec::new();
            let budget = if journal_mode {
                theia_core::quests::catalog::MAX_CATALOG_BYTES
            } else {
                theia_core::quests::MAX_PROFILE_BYTES
            };
            std::fs::File::open(args.remove(i))?
                .take(budget as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profiles = if journal_mode {
                if dialogue_profile.is_none() {
                    return Err("Journal requires guide dialogue".into());
                }
                (
                    None,
                    None,
                    Some(std::sync::Arc::new(
                        if let Some(areas) = &quest_area_bytes {
                            theia_network::quest_journal::Profile::from_json_with_areas(
                                &bytes, areas, xp,
                            )
                            .map_err(|e| e.to_string())?
                        } else {
                            theia_network::quest_journal::Profile::from_json(&bytes, xp)
                                .map_err(|e| e.to_string())?
                        },
                    )),
                )
            } else if explicit_quest {
                if dialogue_profile.is_none() {
                    return Err("Explicit quests require guide dialogue".into());
                }
                let profile = if let Some(recipe) = &quest_turn_in_xp {
                    theia_network::quest_lifecycle_wire::Profile::from_json_with_turn_in_xp(
                        &bytes, recipe, xp,
                    )
                    .map_err(|e| e.to_string())?
                } else {
                    theia_network::quest_lifecycle_wire::Profile::from_json(&bytes)
                        .map_err(|e| e.to_string())?
                };
                (None, Some(std::sync::Arc::new(profile)), None)
            } else {
                (
                    Some(std::sync::Arc::new(
                        theia_network::quest_wire::Profile::from_json(&bytes)
                            .map_err(|e| e.to_string())?,
                    )),
                    None,
                    None,
                )
            };
            if let Some(creature) = profiles
                .0
                .as_ref()
                .map(|p| p.profile().creature())
                .or_else(|| profiles.1.as_ref().map(|p| p.profile().creature()))
                && xp.profile().kill_experience(creature).is_none()
            {
                return Err("Objective/progression creature mismatch".into());
            }
            profiles
        } else {
            if explicit_quest || journal_mode {
                return Err("Quest mode requires quest catalog".into());
            }
            (None, None, None)
        };
    let mut item_bytes = None;
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
        item_bytes = Some(bytes.clone());
        Some(
            theia_network::renet_runtime::inventory::Profile::from_json(&bytes)
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    let melee_profile = if let Some(i) = args.iter().position(|a| a == "--melee-weapons") {
        use std::io::Read;
        args.remove(i);
        if i >= args.len() || !classic || combat_catalog.is_none() {
            return Err("Melee requires a weapon profile and classic combat".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_core::melee::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let catalog = theia_core::melee::Catalog::from_json(&bytes)?;
        catalog.validate_items(
            inventory_profile
                .as_ref()
                .ok_or("Melee requires equipment")?
                .catalog(),
        )?;
        Some((bytes, std::sync::Arc::new(catalog)))
    } else {
        None
    };
    let physical_selected = args.iter().any(|a| a == "--physical-definitions");
    if physical_selected != args.iter().any(|a| a == "--physical-stats") {
        return Err("Physical definitions and stats must be selected together".into());
    }
    let physical_profile = if physical_selected {
        use std::io::Read;
        if !classic || melee_profile.is_none() || player_life_profile.is_none() {
            return Err("Physical damage requires classic melee and life profiles".into());
        }
        let mut data = Vec::new();
        for (flag, limit) in [
            (
                "--physical-definitions",
                theia_core::damage::catalog::MAX_PROFILE_BYTES,
            ),
            (
                "--physical-stats",
                theia_core::damage::stats::MAX_PROFILE_BYTES,
            ),
        ] {
            let i = args
                .iter()
                .position(|a| a == flag)
                .ok_or("Missing physical profile flag")?;
            args.remove(i);
            if i >= args.len() {
                return Err("Missing physical profile path".into());
            }
            let mut bytes = Vec::new();
            std::fs::File::open(args.remove(i))?
                .take(limit as u64 + 1)
                .read_to_end(&mut bytes)?;
            data.push(bytes);
        }
        Some(std::sync::Arc::new(
            theia_network::simulation::combat::physical::Profile::new(
                &data[0],
                &data[1],
                theia_network::simulation::combat::physical::Bindings {
                    weapons: &melee_profile.as_ref().unwrap().1,
                    items: inventory_profile
                        .as_ref()
                        .ok_or("Physical damage requires items")?
                        .catalog(),
                    abilities: &combat_catalog
                        .as_ref()
                        .ok_or("Physical damage requires abilities")?
                        .1,
                    creature: &patrol_binding
                        .as_ref()
                        .ok_or("Physical damage requires creature")?
                        .0,
                    player_maximum_health: character_stats
                        .as_ref()
                        .ok_or("Physical damage requires vitals")?
                        .1
                        .maximum_health(),
                    creature_maximum_health: theia_network::simulation::combat::DEVELOPMENT_HEALTH,
                },
            )
            .map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let white_profile = if let Some(i) = args.iter().position(|arg| arg == "--white-melee") {
        use std::io::Read;
        if args.iter().filter(|arg| *arg == "--white-melee").count() != 1 {
            return Err("Choose one ordinary-melee profile".into());
        }
        args.remove(i);
        if i >= args.len() {
            return Err("Missing ordinary-melee profile path".into());
        }
        let physical = physical_profile
            .clone()
            .ok_or("Ordinary outcomes require physical damage")?;
        if patrol_binding
            .as_ref()
            .is_none_or(|p| p.0.aggro().is_none())
        {
            return Err("Ordinary outcomes require pursuit".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(args.remove(i))?
            .take(theia_network::simulation::combat::white_melee::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let profile =
            theia_network::simulation::combat::white_melee::Profile::new(&bytes, physical)
                .map_err(|e| e.to_string())?;
        profile
            .check_supported_abilities(
                &combat_catalog
                    .as_ref()
                    .ok_or("Ordinary outcomes require abilities")?
                    .1,
            )
            .map_err(|e| e.to_string())?;
        Some(std::sync::Arc::new(profile))
    } else {
        None
    };
    let ability_profile = {
        let mut load =
            |flag: &str, limit: usize| -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
                use std::io::Read;
                let Some(i) = args.iter().position(|arg| arg == flag) else {
                    return Ok(None);
                };
                if args.iter().filter(|arg| arg.as_str() == flag).count() != 1 {
                    return Err("Choose one ability damage/starting profile".into());
                }
                args.remove(i);
                if i >= args.len() {
                    return Err("Missing ability damage/starting profile path".into());
                }
                let mut bytes = Vec::new();
                std::fs::File::open(args.remove(i))?
                    .take(limit as u64 + 1)
                    .read_to_end(&mut bytes)?;
                Ok(Some(bytes))
            };
        let starting = load(
            "--ability-starting",
            theia_core::damage::ability_starting::catalog::MAX_PROFILE_BYTES,
        )?;
        let policy = load(
            "--ability-damage",
            theia_network::simulation::combat::ability_damage::MAX_PROFILE_BYTES,
        )?;
        match (starting, policy) {
            (None, None) => None,
            (Some(starting), Some(policy)) => Some(std::sync::Arc::new(
                theia_network::simulation::combat::ability_damage::Profile::new(
                    &starting,
                    &policy,
                    &combat_catalog
                        .as_ref()
                        .ok_or("Ability damage requires abilities")?
                        .0,
                    white_profile
                        .clone()
                        .ok_or("Ability damage requires ordinary physical combat")?,
                )
                .map_err(|e| e.to_string())?,
            )),
            _ => return Err("Ability starting and damage policy must be selected together".into()),
        }
    };
    let membership_profile = player_life::parse_membership(
        &mut args,
        ability_profile.is_some() && player_life_profile.is_some(),
    )?;
    let credit_bytes = player_life::parse_credit(&mut args, membership_profile.is_some())?;
    let reset_profile = player_life::parse_reset(
        &mut args,
        credit_bytes.is_some(),
        patrol_binding.as_ref().map(|p| &p.0),
    )?;
    let threat_bytes = player_life::parse_threat(&mut args, reset_profile.is_some())?;
    let public_attack_profile = player_life::parse_public_attacks(
        &mut args,
        classic
            && patrol_binding
                .as_ref()
                .is_some_and(|p| p.0.aggro().is_some()),
        player_life_profile.as_ref(),
    )?;
    let public_avatar_profile =
        player_life::parse_public_avatar(&mut args, classic, player_life_profile.as_ref())?;
    if let Some((_, catalog)) = &combat_catalog {
        catalog.validate_equipment(inventory_profile.as_ref().map(|p| p.catalog()))?;
    }
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
                    .map_or(base, |(profile, navigation, _)| {
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
                let base = player_life_profile.as_ref().map_or(base, |profile| {
                    theia_network::player_life_wire::profile_identity(base, profile)
                });
                let base = if let Some((weapons, _)) = &melee_profile {
                    theia_network::simulation::combat::autoattack::profile_identity(
                        base,
                        weapons,
                        item_bytes.as_deref().ok_or("Missing melee item bytes")?,
                    )
                    .map_err(|e| e.to_string())?
                } else {
                    base
                };
                let base = physical_profile
                    .as_ref()
                    .map_or(base, |profile| profile.identity(base));
                let base = white_profile
                    .as_ref()
                    .map_or(base, |profile| profile.identity(base));
                let base = ability_profile
                    .as_ref()
                    .map_or(base, |profile| profile.identity(base));
                let base = membership_profile.as_ref().map_or(base, |profile| {
                    theia_network::simulation::combat::membership::identity_for_profile(
                        base, profile,
                    )
                });
                let base = if let Some(bytes) = &credit_bytes {
                    theia_network::simulation::combat::credit::profile_identity(base, bytes)
                        .map_err(|error| error.to_string())?
                } else {
                    base
                };
                let base = reset_profile
                    .as_ref()
                    .map_or(base, |profile| profile.identity(base));
                let base = if let Some(bytes) = &threat_bytes {
                    theia_network::simulation::combat::threat::profile_identity(base, bytes)
                        .map_err(|error| error.to_string())?
                } else {
                    base
                };
                let base = public_attack_profile
                    .as_ref()
                    .map_or(base, |profile| profile.identity(base));
                let base = public_avatar_profile
                    .as_ref()
                    .map_or(base, |p| p.identity(base));
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
                let base = lifecycle_profile.as_ref().map_or(base, |p| {
                    theia_network::quest_lifecycle_wire::profile_identity(base, p.hash())
                });
                let base = journal_profile.as_ref().map_or(base, |p| {
                    theia_network::quest_journal::profile_identity(base, p.hash())
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
            player_life_profile.clone(),
            public_attack_profile.clone(),
            public_avatar_profile.clone(),
            combat_catalog.as_ref().map(|(_, catalog)| catalog.clone()),
            inventory_profile.clone(),
            status_catalog.as_ref().map(|(_, catalog)| catalog.clone()),
            progression_profile.clone(),
            quest_profile.clone(),
            lifecycle_profile.clone(),
            journal_profile.clone(),
            dialogue_profile
                .as_ref()
                .map(|(profile, _)| profile.clone()),
            dodge_profile.clone(),
            classic,
            melee_profile.as_ref().map(|(_, catalog)| catalog.clone()),
            physical_profile.clone(),
            white_profile.clone(),
            ability_profile.clone(),
            membership_profile.clone(),
        )?)
    } else {
        None
    };
    if membership_profile.is_some() && net.is_none() {
        return Err("Combat membership requires network admission".into());
    }
    if ability_profile.is_some() && net.is_none() {
        return Err("Ability damage requires network admission".into());
    }
    if dodge_profile.is_some() && net.is_none() {
        return Err("Dodge requires network admission".into());
    }
    if classic && (net.is_none() || dodge_profile.is_some()) {
        return Err("Classic controls require network admission and exclude dodge".into());
    }
    if journal_profile.is_some() && net.is_none() {
        return Err("Journal requires network admission".into());
    }
    if lifecycle_profile.is_some() && net.is_none() {
        return Err("Explicit quests require network admission".into());
    }
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
    pursuit_qualification::install(&mut app)?;
    melee_qualification::install(&mut app)?;
    projectile_qualification::install(&mut app)?;
    app.add_systems(Last, reference_scene::grip::observe);
    app.add_systems(Last, reference_scene::attack::observe);
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
    if let Some(path) = creature_asset {
        creature_visual::install(&mut app, path);
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
                .after(bevy::input::InputSystems)
                .after(bevy::picking::PickingSystems::Last),
        )
        .add_systems(FixedUpdate, movement::move_local_pawn);
    if let Some(controller) = classic_controller {
        app.insert_resource(controller);
    }
    if let Some(profile) = dodge_profile {
        app.insert_resource(dodge::animation::Policy(profile))
            .init_resource::<dodge::animation::State>()
            .add_systems(
                Update,
                dodge::animation::present
                    .after(network::present)
                    .before(reference_scene::tick),
            );
    }
    if targeted {
        targeting::install(
            &mut app,
            patrol_binding
                .as_ref()
                .map(|(profile, _, _)| profile.speed_metres_per_second()),
        );
    }
    if let Some((_, catalog)) = status_catalog {
        status_effects::install(&mut app, catalog);
    }
    if let Some((profile, _)) = dialogue_profile {
        dialogue::install(&mut app, profile);
    }
    if let Some(profile) = lifecycle_profile {
        quest_lifecycle::install(&mut app, profile);
    }
    if let Some(profile) = journal_profile {
        quest_journal::install(&mut app, profile);
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
            player_life_profile.is_some(),
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
                    dodge::input,
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
    inventory: Option<Res<'w, inventory::Ui>>,
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

#[allow(clippy::too_many_arguments)]
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<MoveIntent>,
    mode: InputMode,
    mut pawn: Query<(&mut Transform, &mut movement::Body), With<LocalPawn>>,
    mut exit: MessageWriter<AppExit>,
    mut orbit: ResMut<camera::Orbit>,
    camera_mode: Option<Res<camera::mode::Mode>>,
    windows: Query<&Window>,
    mut classic: Option<ResMut<classic_controls::Controller>>,
    buttons: Res<ButtonInput<MouseButton>>,
    gesture: Option<Res<world_input::Gesture>>,
    time: Res<Time>,
) {
    intent.jump = false;
    if windows.single().is_ok_and(|window| !window.focused) {
        intent.direction = Vec2::ZERO;
        intent.sprint = false;
        return;
    }
    let talking = mode
        .dialogue
        .as_ref()
        .is_some_and(|ui| ui.open || ui.pending || ui.close_when_open)
        || mode.network.as_ref().is_some_and(|n| {
            let s = n.state();
            s.error.is_none()
                && (s.dialogue_pending || s.dialogue.is_some_and(|v| v.open.is_some()))
        });
    if let Some(classic) = classic.as_mut() {
        classic.escape_owned_by_panel = keys.just_pressed(KeyCode::Escape)
            && (talking || mode.inventory.as_ref().is_some_and(|u| u.open));
    }
    if keys.just_pressed(KeyCode::Escape)
        && !talking
        && !mode.inventory.as_ref().is_some_and(|ui| ui.open)
        && !camera_mode
            .as_ref()
            .is_some_and(|mode| mode.consumed_escape)
        && classic.is_none()
    {
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
    if let Some(classic) = classic.as_mut() {
        let correction = mode
            .network
            .as_ref()
            .and_then(|n| n.state().correction.map(|c| c.state));
        classic.sync(correction);
        let allowed = !talking
            && mode
                .network
                .as_ref()
                .is_some_and(|n| n.connected() && n.can_act());
        let world_mouse = gesture.as_ref().is_none_or(|g| g.world_origin());
        let both = world_mouse
            && buttons.pressed(MouseButton::Left)
            && buttons.pressed(MouseButton::Right);
        let steering = world_mouse
            && buttons.pressed(MouseButton::Right)
            && (both || gesture.as_ref().is_none_or(|g| g.camera_dragging()));
        let sample = classic.sample(
            &keys,
            time.delta_secs_f64(),
            orbit.angles().x,
            steering,
            both,
            allowed,
        );
        intent.direction = Vec2::new(
            f32::from(sample.axes[0]) / 1000.,
            f32::from(sample.axes[1]) / 1000.,
        );
        intent.jump = sample.jump;
        intent.sprint = sample.sprint;
        if !buttons.pressed(MouseButton::Left) && sample.keyboard_turn != 0. {
            let angles = orbit.angles();
            orbit.set_angles(angles.x + sample.keyboard_turn, angles.y);
        }
        if let Ok((mut pawn, _)) = pawn.single_mut() {
            pawn.rotation = Quat::from_rotation_y(classic.yaw());
        }
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

#[cfg(test)]
mod control_tests {
    use super::*;
    #[test]
    fn escape_closes_dialogue_then_backpack_before_allowing_quit() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<Time>()
            .init_resource::<MoveIntent>()
            .init_resource::<camera::Orbit>()
            .init_resource::<inventory::Ui>()
            .init_resource::<dialogue::Ui>()
            .add_message::<AppExit>()
            .add_systems(Update, (input, inventory::close_panel).chain());
        app.world_mut().resource_mut::<inventory::Ui>().open = true;
        app.world_mut().resource_mut::<dialogue::Ui>().open = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
        assert!(app.world().resource::<inventory::Ui>().open);
        // Dialogue itself still closes through its authoritative request path.
        // Observe that closure before the next independent Escape press.
        app.world_mut().resource_mut::<dialogue::Ui>().open = false;
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.clear();
            keys.press(KeyCode::Escape);
        }
        app.update();
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
        assert!(!app.world().resource::<inventory::Ui>().open);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.clear();
            keys.press(KeyCode::Escape);
        }
        app.update();
        assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
    }
}
