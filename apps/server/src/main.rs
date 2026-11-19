mod persistence;
use std::{net::SocketAddr, path::PathBuf};
use theia_network::{Result, server, tls};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let bind: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:5000".into())
        .parse()?;
    let cert = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "build/server-cert.der".into()),
    );
    let mut tickets = None;
    let mut dev_relays: Option<Vec<SocketAddr>> = None;
    let mut chunk_stream = false;
    let mut world_selected = false;
    let mut graybox = false;
    let mut terrain_source = theia_network::procedural::Source::Procedural;
    let mut predicted = false;
    let mut classic = false;
    let mut dodge_profile = None;
    let mut melee_weapons = None;
    let mut physical_definitions = None;
    let mut physical_stats = None;
    let mut white_melee = None;
    let mut ability_starting = None;
    let mut ability_damage = None;
    let mut combat_membership = None;
    let mut creature_credit = None;
    let mut creature_reset = None;
    let mut creature_threat = None;
    let mut public_creature_attacks = false;
    let mut public_avatar_life = false;
    let mut combat_bytes = None;
    let mut targeted = false;
    let mut combat_catalog = None;
    let mut character_stats = None;
    let mut creature_retaliation = None;
    let mut player_respawn = None;
    let mut player_stats_bytes = None;
    let mut patrol_bytes = None;
    let mut status_catalog = None;
    let mut patrol_profile = None;
    let mut navigation = None;
    let mut item_catalog = None;
    let mut reward_catalog = None;
    let mut progression_catalog = None;
    let mut quest_catalog = None;
    let mut quest_areas = None;
    let mut explicit_quest = false;
    let mut journal_mode = false;
    let mut quest_turn_in_xp = None;
    let mut dialogue_catalog = None;
    let mut vendor_catalog = None;
    let mut guide_position: Option<[f64; 2]> = None;
    let mut dev_persistence = false;
    let mut account_login = false;
    let mut dev_tour = false;
    let mut dev_speed = 1.;
    let mut dev_spawn = [8000., 8000.];
    let mut origin = None;
    let mut replication_span = None;
    let mut world = theia_network::simulation::World::default();
    while let Some(option) = args.next() {
        match option.as_str() {
            "--dev-relays" => dev_relays=Some(args.next().ok_or("Missing relay endpoints")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?),
            "--prediction" => predicted=true,
            "--classic" => classic=true,
            "--ability-starting" => {
                if ability_starting.is_some(){return Err("Choose one ability starting profile".into());}
                ability_starting=Some(PathBuf::from(args.next().ok_or("Missing ability starting profile")?));
            }
            "--public-avatar-life" => {
                if std::mem::replace(&mut public_avatar_life,true){return Err("Choose public avatar life once".into());}
            }
            "--public-creature-attacks" => {
                if std::mem::replace(&mut public_creature_attacks, true) {return Err("Choose public creature attacks once".into());}
            }
            "--creature-threat" => {
                if creature_threat.is_some(){return Err("Choose one creature threat policy".into());}
                creature_threat=Some(PathBuf::from(args.next().ok_or("Missing creature threat policy")?));
            }
            "--creature-reset" => {
                if creature_reset.is_some(){return Err("Choose one creature reset policy".into());}
                creature_reset=Some(PathBuf::from(args.next().ok_or("Missing creature reset policy")?));
            }
            "--creature-credit" => {
                if creature_credit.is_some(){return Err("Choose one creature credit policy".into());}
                creature_credit=Some(PathBuf::from(args.next().ok_or("Missing creature credit policy")?));
            }
            "--combat-membership" => {
                if combat_membership.is_some(){return Err("Choose one combat membership policy".into());}
                combat_membership=Some(PathBuf::from(args.next().ok_or("Missing combat membership policy")?));
            }
            "--ability-damage" => {
                if ability_damage.is_some(){return Err("Choose one ability damage profile".into());}
                ability_damage=Some(PathBuf::from(args.next().ok_or("Missing ability damage profile")?));
            }
            "--white-melee" => {
                if white_melee.is_some() {return Err("Choose one ordinary-melee profile".into());}
                white_melee=Some(PathBuf::from(args.next().ok_or("Missing ordinary-melee profile")?));
            }
            "--physical-definitions" => {
                if physical_definitions.is_some() {return Err("Choose one physical definition profile".into());}
                physical_definitions=Some(PathBuf::from(args.next().ok_or("Missing physical definitions")?));
            }
            "--physical-stats" => {
                if physical_stats.is_some() {return Err("Choose one physical stats profile".into());}
                physical_stats=Some(PathBuf::from(args.next().ok_or("Missing physical stats")?));
            }
            "--melee-weapons" => {
                if melee_weapons.is_some() {return Err("Choose one melee weapon profile".into());}
                melee_weapons=Some(PathBuf::from(args.next().ok_or("Missing melee weapon profile")?));
            }
            "--dodge-profile" => {
                use std::io::Read;
                if dodge_profile.is_some() { return Err("Choose one dodge profile".into()); }
                let mut bytes=Vec::new();
                std::fs::File::open(args.next().ok_or("Missing dodge profile")?)?.take(theia_core::dodge::MAX_PROFILE_BYTES as u64+1).read_to_end(&mut bytes)?;
                dodge_profile=Some(std::sync::Arc::new(theia_network::dodging::Profile::from_json(&bytes)?));
            }
            "--creature-retaliation" => {
                if creature_retaliation.is_some() { return Err("Choose one retaliation profile".into()); }
                creature_retaliation=Some(PathBuf::from(args.next().ok_or("Missing retaliation profile")?));
            }
            "--player-respawn" => {
                if player_respawn.is_some() { return Err("Choose one player respawn profile".into()); }
                player_respawn=Some(PathBuf::from(args.next().ok_or("Missing player respawn profile")?));
            }
            "--dev-persistence" => dev_persistence=true,
            "--account-login" => account_login=true,
            "--vendor-catalog" => {
                if vendor_catalog.is_some() { return Err("Choose one vendor catalog".into()); }
                vendor_catalog = Some(PathBuf::from(args.next().ok_or("Missing vendor catalog")?));
            }
            "--dialogue-catalog" => {
                if dialogue_catalog.is_some() {return Err("Choose one dialogue catalog".into());}
                dialogue_catalog=Some(PathBuf::from(args.next().ok_or("Missing dialogue catalog")?));
            }
            "--guide-position" => {
                if guide_position.is_some() {return Err("Choose one guide position".into());}
                let values:Vec<f64>=args.next().ok_or("Missing guide position")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?;
                guide_position=Some(values.try_into().map_err(|_|"Expected guide x,z")?);
            }
            "--quest-lifecycle" => explicit_quest=true,
            "--quest-journal" => journal_mode=true,
            "--quest-turn-in-xp" => {
                if quest_turn_in_xp.is_some() {return Err("Choose one completion XP recipe".into());}
                quest_turn_in_xp=Some(PathBuf::from(args.next().ok_or("Missing completion XP recipe")?));
            }
            "--quest-areas" => {
                if quest_areas.is_some() {return Err("Choose one quest area catalogue".into());}
                quest_areas=Some(PathBuf::from(args.next().ok_or("Missing quest areas")?));
            }
            "--quest-catalog" => {
                if quest_catalog.is_some() {return Err("Choose one quest catalog".into());}
                quest_catalog=Some(PathBuf::from(args.next().ok_or("Missing quest catalog")?));
            }
            "--progression-catalog" => {
                if progression_catalog.is_some() {return Err("Choose one progression catalog".into());}
                progression_catalog=Some(PathBuf::from(args.next().ok_or("Missing progression catalog")?));
            }
            "--reward-catalog" => {
                if reward_catalog.is_some() {return Err("Choose one reward catalog".into());}
                reward_catalog=Some(PathBuf::from(args.next().ok_or("Missing reward catalog")?));
            }
            "--item-catalog" => {
                if item_catalog.is_some() {return Err("Choose one item catalog".into());}
                item_catalog=Some(PathBuf::from(args.next().ok_or("Missing item catalog")?));
            }
            "--creature-profile" => {
                if patrol_profile.is_some() {return Err("Choose one creature profile".into());}
                patrol_profile=Some(PathBuf::from(args.next().ok_or("Missing creature profile")?));targeted=true;
            }
            "--navigation" => {
                if navigation.is_some() {return Err("Choose one navigation package".into());}
                navigation=Some(PathBuf::from(args.next().ok_or("Missing navigation package")?));
            }
            "--targeting" => targeted=true,
            "--status-catalog" => {
                if status_catalog.is_some() {return Err("Choose one status catalog".into());}
                status_catalog=Some(PathBuf::from(args.next().ok_or("Missing status catalog")?));
            }
            "--character-stats" => {
                if character_stats.is_some() {return Err("Choose one character stats profile".into());}
                character_stats=Some(PathBuf::from(args.next().ok_or("Missing character stats profile")?));
            }
            "--combat-catalog" => {
                if combat_catalog.is_some() {return Err("Choose one combat catalog".into());}
                combat_catalog=Some(PathBuf::from(args.next().ok_or("Missing combat catalog")?));
                targeted=true;
            }
            "--dev-tour" => {dev_tour=true;}
            "--saved-terrain" => { if world_selected {return Err("Choose one world".into());} world_selected=true; graybox=true;
                terrain_source=theia_network::procedural::Source::Saved(std::sync::Arc::new(theia_network::saved_terrain::Store::open(&PathBuf::from(args.next().ok_or("Missing saved terrain directory")?))?)); }
            "--graybox-world" => { if world_selected {return Err("Choose one world".into());} world_selected=true; graybox=true; }
            "--dev-speed" => {dev_speed=args.next().ok_or("Missing developer speed")?.parse()?;}
            "--dev-spawn" => {let values:Vec<f64>=args.next().ok_or("Missing developer spawn")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?;dev_spawn=values.try_into().map_err(|_|"Expected x,z")?;}
            "--chunk-stream"=>chunk_stream=true,
            "--world-origin" => {
                let xyz: Vec<f64> = args.next().ok_or("Missing world origin")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?;
                let xyz: [f64;3] = xyz.try_into().map_err(|_|"World origin must be x,y,z")?;
                origin = Some(theia_core::coordinates::GlobalPosition::new(xyz).ok_or("Invalid global origin")?);
            }
            "--replication-span" => {replication_span = Some(args.next().ok_or("Missing replication span")?.parse()?);}

            "--renet-tickets" => {
                tickets = Some(PathBuf::from(
                    args.next().ok_or("Missing ticket directory")?,
                ));
            }
            "--tick-hz" => {
                let hz: u32 = args.next().ok_or("Missing tick rate")?.parse()?;
                let rate = theia_core::time::TickRate::from_hz(hz)
                    .ok_or("Tick rate must be 20, 30 or 60")?;
                if !world.set_tick_rate(rate) {
                    return Err("Cannot change active tick rate".into());
                }
            }
            "--chunk-world" => {
                if world_selected { return Err("Choose one world descriptor".into()); }
                world_selected = true;
                let rate = world.tick_rate();
                let directory = PathBuf::from(args.next().ok_or("Missing chunk directory")?);
                let config = theia_network::chunk_world::Config::load(&directory)?;
                world = theia_network::simulation::World::with_chunk_config(&config)?;
                assert!(world.set_tick_rate(rate));
                println!("chunk_world_startup resident={} spawns={}", config.initial().len(), config.spawns().len());
            }
            "--terrain-world" => {
                if world_selected { return Err("Choose one world descriptor".into()); }
                world_selected = true;
                let rate = world.tick_rate();
                let directory = PathBuf::from(args.next().ok_or("Missing terrain directory")?);
                let terrain =
                    theia_network::terrain::Terrain::load(&directory.join("terrain.json"))?;
                if !terrain.has_spawns() { return Err("World requires an explicit validated spawn; chunk-only packages cannot host sessions".into()); }
                world = theia_network::simulation::World::with_terrain(terrain);
                assert!(world.set_tick_rate(rate));
            }
            _ => return Err("Expected --terrain-world or --chunk-world <directory>, --tick-hz <20|30|60> or --renet-tickets <fresh-directory>".into()),
        }
    }
    if status_catalog.is_some() && (character_stats.is_none() || combat_catalog.is_none()) {
        return Err("Statuses require combat and character stats".into());
    }
    if creature_retaliation.is_some() != player_respawn.is_some() {
        return Err("Retaliation and player respawn profiles must be selected together".into());
    }
    if creature_retaliation.is_some()
        && (!predicted
            || patrol_profile.is_none()
            || character_stats.is_none()
            || combat_catalog.is_none())
    {
        return Err("Player life requires predicted creature combat and character stats".into());
    }
    if patrol_profile.is_some() != navigation.is_some() {
        return Err("Creature profile and navigation package must be selected together".into());
    }
    if patrol_profile.is_some() && (combat_catalog.is_none() || !predicted || !graybox) {
        return Err("Creature patrol requires predicted graybox combat".into());
    }
    if character_stats.is_some() && combat_catalog.is_none() {
        return Err("Character stats require combat".into());
    }
    if graybox {
        if (tickets.is_none() && !account_login) || origin.is_some() || replication_span.is_some() {
            return Err("Graybox requires Renet and manages its own frames".into());
        }
        let rate = world.tick_rate();
        world = theia_network::simulation::World::with_terrain_source(
            dev_spawn,
            dev_speed,
            terrain_source,
        )?;
        world.set_tick_rate(rate);
        world.dev_tour = dev_tour;
    } else if dev_tour || dev_speed != 1. || dev_spawn != [8000., 8000.] {
        return Err("Developer travel requires --graybox-world".into());
    }
    if predicted {
        world.enable_prediction()?;
    }
    if let Some(profile) = dodge_profile {
        world.enable_dodge(profile)?;
    }
    if classic {
        if tickets.is_none() && !account_login {
            return Err("Classic controls require secure Renet admission".into());
        }
        world.enable_classic()?;
    }
    if targeted {
        if !graybox || (tickets.is_none() && !account_login) {
            return Err("Targeting requires a graybox world and secure Renet".into());
        }
        let target = if let Some(path) = patrol_profile {
            use std::io::Read;
            use theia_network::simulation::creatures;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(creatures::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = creatures::Profile::from_json(&bytes)?;
            if profile.aggro().is_some() && creature_retaliation.is_none() {
                return Err(
                    "Creature pursuit requires player life and retaliation profiles".into(),
                );
            }
            let source = world
                .procedural()
                .ok_or("Creature patrol requires terrain")?
                .source
                .clone();
            let navigation =
                theia_network::navigation::Store::open(&navigation.unwrap(), source.identity())?;
            println!("development_creature profile={}", profile.id().as_str());
            let target =
                world.enable_patrol(creatures::Prepared::load(profile, &navigation, &source)?)?;
            patrol_bytes = Some(bytes);
            target
        } else {
            let x = if dev_spawn[0] + 8. <= 15999. {
                dev_spawn[0] + 8.
            } else {
                dev_spawn[0] - 8.
            };
            world.spawn_development_target([x, dev_spawn[1]])?
        };
        println!("development_target id={}", target.entity.value());
    }
    if let Some(path) = combat_catalog {
        use std::io::Read;
        use theia_network::simulation::combat::{DEVELOPMENT_HEALTH, DEVELOPMENT_RESPAWN_MS};
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(theia_core::abilities::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        combat_bytes = Some(bytes.clone());
        if let Some(path) = character_stats {
            let mut stats = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::resources::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut stats)?;
            if let Some(path) = status_catalog {
                let mut statuses = Vec::new();
                std::fs::File::open(path)?
                    .take(theia_core::statuses::MAX_CATALOG_BYTES as u64 + 1)
                    .read_to_end(&mut statuses)?;
                world.enable_status_combat(
                    &bytes,
                    DEVELOPMENT_HEALTH,
                    DEVELOPMENT_RESPAWN_MS,
                    &stats,
                    &statuses,
                )?;
            } else {
                world.enable_resource_combat(
                    &bytes,
                    DEVELOPMENT_HEALTH,
                    DEVELOPMENT_RESPAWN_MS,
                    &stats,
                )?;
            }
            player_stats_bytes = Some(stats);
        } else {
            world.enable_combat(&bytes, DEVELOPMENT_HEALTH, DEVELOPMENT_RESPAWN_MS)?;
        }
        println!(
            "development_combat abilities={}",
            world.combat_catalog().unwrap().abilities().len()
        );
    }
    if let (Some(policy_path), Some(respawn_path)) = (creature_retaliation, player_respawn) {
        use std::io::Read;
        let mut policy = Vec::new();
        std::fs::File::open(policy_path)?
            .take(theia_core::retaliation::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut policy)?;
        let mut respawn = Vec::new();
        std::fs::File::open(respawn_path)?
            .take(theia_core::player_life::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut respawn)?;
        let profile = std::sync::Arc::new(theia_network::player_life_wire::Profile::from_json(
            player_stats_bytes
                .as_deref()
                .ok_or("Missing player life stats")?,
            patrol_bytes
                .as_deref()
                .ok_or("Missing player life creature")?,
            &policy,
            &respawn,
        )?);
        world.enable_player_life(profile)?;
        println!("development_player_life enabled=true transient=true");
    }
    if let Some(path) = melee_weapons {
        use std::io::Read;
        if !classic || !account_login {
            return Err("Melee requires classic account-backed combat".into());
        }
        let mut weapons = Vec::new();
        std::fs::File::open(path)?
            .take(theia_core::melee::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut weapons)?;
        let mut items = Vec::new();
        std::fs::File::open(item_catalog.as_ref().ok_or("Melee requires items")?)?
            .take(theia_core::inventory::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut items)?;
        world.enable_autoattack(&weapons, &items)?;
        println!("development_melee enabled=true");
    }
    if physical_definitions.is_some() != physical_stats.is_some() {
        return Err("Physical definitions and stats must be selected together".into());
    }
    if let (Some(definitions_path), Some(stats_path)) = (physical_definitions, physical_stats) {
        use std::io::Read;
        if !classic || !account_login {
            return Err("Physical damage requires classic account-backed combat".into());
        }
        let mut definitions = Vec::new();
        std::fs::File::open(definitions_path)?
            .take(theia_core::damage::catalog::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut definitions)?;
        let mut stats = Vec::new();
        std::fs::File::open(stats_path)?
            .take(theia_core::damage::stats::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut stats)?;
        let mut items = Vec::new();
        std::fs::File::open(
            item_catalog
                .as_ref()
                .ok_or("Physical damage requires items")?,
        )?
        .take(theia_core::inventory::MAX_CATALOG_BYTES as u64 + 1)
        .read_to_end(&mut items)?;
        world.enable_physical_damage(&definitions, &stats, &items)?;
        println!("development_physical_damage enabled=true policy=normal_reference_v1");
    }
    if let Some(path) = white_melee {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(theia_network::simulation::combat::white_melee::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        world.enable_white_melee(&bytes)?;
        println!("development_white_melee enabled=true policy=white_physical_reference_v1");
    }
    if ability_starting.is_some() != ability_damage.is_some() {
        return Err("Ability starting and damage policy must be selected together".into());
    }
    if let (Some(starting), Some(policy)) = (ability_starting, ability_damage) {
        use std::io::Read;
        let mut starting_bytes = Vec::new();
        let mut policy_bytes = Vec::new();
        std::fs::File::open(starting)?
            .take(theia_core::damage::ability_starting::catalog::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut starting_bytes)?;
        std::fs::File::open(policy)?
            .take(theia_network::simulation::combat::ability_damage::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut policy_bytes)?;
        world.enable_ability_damage(
            &starting_bytes,
            &policy_bytes,
            combat_bytes
                .as_deref()
                .ok_or("Ability damage requires active abilities")?,
        )?;
        println!(
            "development_ability_damage enabled=true policy=ability_damage_reference_fixture_v1"
        );
    }
    if let Some(path) = combat_membership {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(theia_core::combat_membership::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        world.enable_combat_membership(&bytes)?;
        println!("development_combat_membership enabled=true policy=combat_membership_fixture_v1");
    }
    if let Some(path) = creature_credit {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(theia_core::creature_credit::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let profile = theia_core::creature_credit::Profile::from_json(&bytes)?;
        if profile.recipient() != Some(theia_core::creature_credit::RecipientPolicy::SoloTagOwner) {
            return Err("Normal credit requires explicit solo tag-owner recipient policy".into());
        }
        world.enable_creature_credit(&bytes)?;
        println!("development_creature_credit enabled=true policy=solo_tag_credit_fixture_v1");
    }
    if let Some(path) = creature_reset {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(theia_network::simulation::combat::reset::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        world.enable_creature_reset(&bytes)?;
        println!("development_creature_reset enabled=true policy=creature_return_reset_fixture_v1");
    }
    if let Some(path) = creature_threat {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(theia_core::threat::MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        world.enable_creature_threat(&bytes)?;
        println!(
            "development_creature_threat enabled=true policy=classic_damage_threat_empirical_v1"
        );
    }
    if public_creature_attacks {
        world.enable_public_creature_attacks()?;
        println!("development_public_creature_attacks enabled=true contract=NA01");
    }
    if public_avatar_life {
        world.enable_public_avatar_life()?;
        println!("development_public_avatar_life enabled=true contract=AV01");
    }
    if let Some(origin) = origin
        && !world.set_origin(origin)
    {
        return Err("Unsafe world origin".into());
    }
    if let Some(span) = replication_span
        && !world.set_replication_span(span)
    {
        return Err("Replication span must be finite, 1..1024 m".into());
    }
    if world.chunks().is_some() && tickets.is_none() {
        return Err("Chunk worlds require the Renet readiness transport".into());
    }
    if chunk_stream && (world.chunks().is_none() || tickets.is_none()) {
        return Err("--chunk-stream requires --chunk-world and Renet".into());
    }
    if dev_relays.is_some() && tickets.is_none() && !account_login {
        return Err("Loopback relay qualification requires Renet tickets".into());
    }
    if progression_catalog.is_some()
        && (!account_login || world.patrol_snapshot().is_none() || !world.combat_enabled())
    {
        return Err("Progression requires account login and authoritative creature combat".into());
    }
    if quest_areas.is_some() && (!journal_mode || quest_catalog.is_none()) {
        return Err("Quest areas require expanded journal mode and a catalogue".into());
    }
    if journal_mode && (explicit_quest || quest_turn_in_xp.is_some()) {
        return Err(
            "Journal mode excludes legacy quest modes and external completion recipes".into(),
        );
    }
    if journal_mode
        && (quest_catalog.is_none() || dialogue_catalog.is_none() || !account_login || !predicted)
    {
        return Err("Journal requires catalog, dialogue and predicted account login".into());
    }
    if quest_catalog.is_some() && progression_catalog.is_none() {
        return Err("Quest objectives require progression".into());
    }
    if quest_turn_in_xp.is_some() && !explicit_quest {
        return Err("Completion XP requires explicit quest mode".into());
    }
    if explicit_quest
        && (quest_catalog.is_none() || dialogue_catalog.is_none() || !account_login || !predicted)
    {
        return Err(
            "Explicit quests require quest catalog, guide dialogue and predicted account login"
                .into(),
        );
    }
    // Validate world options before opening an endpoint.
    match (dialogue_catalog, guide_position) {
        (Some(path), Some(xz)) if graybox && (account_login || tickets.is_some()) => {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::dialogue::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = std::sync::Arc::new(theia_network::dialogue::Profile::from_json(&bytes)?);
            let guide = theia_server::development::prepare_guide(&mut world, profile, xz).await?;
            println!(
                "guide_ready epoch={} entity={} position={:?}",
                guide.key.epoch.value(),
                guide.key.entity.value(),
                guide.position.metres()
            );
        }
        (None, None) => {}
        _ => {
            return Err(
                "Dialogue requires secure graybox, catalog and explicit guide position".into(),
            );
        }
    }
    if !bind.ip().is_loopback() {
        return Err("Local prototype only: bind must be loopback".into());
    }
    if reward_catalog.is_some()
        && (!account_login
            || item_catalog.is_none()
            || world.patrol_snapshot().is_none()
            || !world.combat_enabled())
    {
        return Err(
            "Rewards require account login, items and authoritative creature combat".into(),
        );
    }
    if item_catalog.is_some() && !account_login {
        return Err("Persistent inventory requires account login".into());
    }
    if !account_login && let Some(catalog) = world.combat_catalog() {
        catalog.validate_equipment(None)?;
    }
    if account_login {
        let mut wire_items = None;
        let inventory = if let Some(path) = item_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::inventory::MAX_CATALOG_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            wire_items = Some(theia_network::renet_runtime::inventory::Profile::from_json(
                &bytes,
            )?);
            Some(theia_persistence::InventoryCatalog::from_json(&bytes)?)
        } else {
            None
        };
        if let Some(catalog) = world.combat_catalog() {
            catalog.validate_equipment(wire_items.as_ref().map(|p| p.catalog()))?;
        }
        let vendor = if let Some(path) = vendor_catalog.take() {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::vendors::MAX_CATALOG_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = theia_persistence::VendorCatalog::from_json(
                &bytes,
                inventory.as_ref().ok_or("Vendor requires items")?,
            )?;
            world.enable_vendor(std::sync::Arc::new(
                theia_network::vendor_wire::Profile::from_json(
                    &bytes,
                    wire_items.as_ref().ok_or("Vendor requires item profile")?,
                )?,
            ))?;
            Some(profile)
        } else {
            None
        };
        let rewards = if let Some(path) = reward_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::rewards::MAX_CATALOG_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let inventory = inventory.as_ref().ok_or("Rewards require items")?;
            let profile = theia_persistence::RewardCatalog::from_json(&bytes, inventory)?;
            let wire = wire_items.as_ref().ok_or("Reward wire items missing")?;
            world.enable_rewards(std::sync::Arc::new(
                theia_network::reward_wire::Profile::from_json(&bytes, wire)?,
            ))?;
            Some(profile)
        } else {
            None
        };
        let mut journal_progression = None;
        let progression = if let Some(path) = progression_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::progression::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = theia_persistence::ProgressionCatalog::from_json(&bytes)?;
            let wire =
                std::sync::Arc::new(theia_network::progression_wire::Profile::from_json(&bytes)?);
            if journal_mode {
                journal_progression = Some(wire);
            } else {
                world.enable_progression(wire)?;
            }
            Some(profile)
        } else {
            None
        };
        let (quest, quest_lifecycle, quest_journal) = if let Some(path) = quest_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(
                    if journal_mode {
                        theia_core::quests::catalog::MAX_CATALOG_BYTES
                    } else {
                        theia_core::quests::MAX_PROFILE_BYTES
                    } as u64
                        + 1,
                )
                .read_to_end(&mut bytes)?;
            if journal_mode {
                let xp = progression.as_ref().ok_or("Journal requires progression")?;
                let wire_xp = journal_progression
                    .take()
                    .ok_or("Journal requires world XP policy")?;
                let (profile, wire) = if let Some(path) = &quest_areas {
                    let mut areas = Vec::new();
                    std::fs::File::open(path)?
                        .take(theia_core::quests::areas::MAX_CATALOG_BYTES as u64 + 1)
                        .read_to_end(&mut areas)?;
                    (
                        theia_persistence::QuestJournalCatalog::from_json_with_areas(
                            &bytes, &areas, xp,
                        )?,
                        theia_network::quest_journal::Profile::from_json_with_areas(
                            &bytes, &areas, &wire_xp,
                        )?,
                    )
                } else {
                    (
                        theia_persistence::QuestJournalCatalog::from_json(&bytes, xp)?,
                        theia_network::quest_journal::Profile::from_json(&bytes, &wire_xp)?,
                    )
                };
                let wire = std::sync::Arc::new(wire);
                if profile.hash() != wire.hash() {
                    return Err("Journal authority/persistence identity mismatch".into());
                }
                world.enable_quest_journal(wire, wire_xp)?;
                (None, None, Some(profile))
            } else if explicit_quest {
                let recipe = if let Some(path) = &quest_turn_in_xp {
                    let mut recipe = Vec::new();
                    std::fs::File::open(path)?
                        .take(theia_core::quests::turn_in_xp::MAX_PROFILE_BYTES as u64 + 1)
                        .read_to_end(&mut recipe)?;
                    Some(recipe)
                } else {
                    None
                };
                let (profile, wire) = if let Some(recipe) = recipe {
                    (
                        theia_persistence::QuestLifecycleCatalog::from_json_with_turn_in_xp(
                            &bytes,
                            &recipe,
                            progression
                                .as_ref()
                                .ok_or("Completion XP requires progression")?,
                        )?,
                        theia_network::quest_lifecycle_wire::Profile::from_json_with_turn_in_xp(
                            &bytes,
                            &recipe,
                            world
                                .progression_profile()
                                .ok_or("Completion XP requires world progression")?,
                        )?,
                    )
                } else {
                    (
                        theia_persistence::QuestLifecycleCatalog::from_json(&bytes)?,
                        theia_network::quest_lifecycle_wire::Profile::from_json(&bytes)?,
                    )
                };
                world.enable_quest_lifecycle(std::sync::Arc::new(wire))?;
                (None, Some(profile), None)
            } else {
                let profile = theia_persistence::QuestCatalog::from_json(&bytes)?;
                world.enable_quest(std::sync::Arc::new(
                    theia_network::quest_wire::Profile::from_json(&bytes)?,
                ))?;
                (Some(profile), None, None)
            }
        } else {
            (None, None, None)
        };
        if tickets.is_some() || dev_persistence || chunk_stream || !graybox || !predicted {
            return Err("Account login requires saved predicted terrain and excludes development ticket/persistence overrides".into());
        }
        let realm = theia_persistence::RealmId::parse(
            &std::env::var("THEIA_DEV_REALM_ID")
                .unwrap_or_else(|_| "74686569-6100-0000-0000-000000000007".into()),
        )
        .ok_or("Invalid account realm UUID")?;
        let url = std::env::var("THEIA_DATABASE_URL")
            .map_err(|_| "Account login requires THEIA_DATABASE_URL")?;
        let store = theia_persistence::Store::connect_loopback(&url, realm).await?;
        let config = theia_server::accounts::Config {
            listen: std::env::var("THEIA_ACCOUNT_BIND")
                .unwrap_or_else(|_| "127.0.0.1:5050".into())
                .parse()?,
            issuer: std::env::var("THEIA_OIDC_ISSUER")
                .map_err(|_| "Account login requires THEIA_OIDC_ISSUER")?,
            client_id: std::env::var("THEIA_OIDC_CLIENT_ID")
                .map_err(|_| "Account login requires THEIA_OIDC_CLIENT_ID")?,
        };
        let socket = std::net::UdpSocket::bind(bind)?;
        let addresses = dev_relays.unwrap_or_else(|| vec![socket.local_addr().unwrap()]);
        let (stop, shutdown) = tokio::sync::watch::channel(false);
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            stop.send_replace(true);
        });
        return theia_server::authenticated::run_with_profiles(
            world,
            socket,
            addresses,
            store,
            config,
            shutdown,
            theia_server::authenticated::GameplayProfiles {
                quest_journal,
                quest_lifecycle,
                inventory,
                rewards,
                progression,
                quest,
                vendor,
            },
        )
        .await;
    }
    if vendor_catalog.is_some() {
        return Err("Vendor requires account login".into());
    }
    let persistent = if dev_persistence {
        if tickets.is_none() || !graybox || !predicted || chunk_stream {
            return Err("Development persistence requires saved terrain, predicted Renet and no chunk-stream override".into());
        }
        Some(persistence::prepare(&mut world).await?)
    } else {
        None
    };
    if let Some(directory) = tickets {
        let socket = std::net::UdpSocket::bind(bind)?;
        let address = socket.local_addr()?;
        let transport = if let Some(addresses) = dev_relays {
            theia_network::renet_runtime::issue_local_advertised(socket, &directory, addresses)?
        } else {
            theia_network::renet_runtime::issue_local(socket, &directory)?
        };
        println!("listening={address} local_tickets={}", directory.display());
        let (shutdown, stop) = tokio::sync::watch::channel(false);
        let signal = shutdown.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = signal.send(true);
        });
        if let Some(persistent) = persistent {
            let (bridge, manager) = persistent.into_parts(shutdown);
            let (game, saved) = tokio::join!(
                theia_network::renet_runtime::run_server_with_checkpoints(
                    transport, stop, world, bridge
                ),
                manager
            );
            game?;
            saved?;
            return Ok(());
        }
        return if chunk_stream {
            theia_network::renet_runtime::run_server_streaming(transport, stop, world)
                .await
                .map(|_| ())
        } else {
            theia_network::renet_runtime::run_server(transport, stop, world).await
        };
    }
    let (endpoint, certificate) = tls::server(bind)?;
    if let Some(parent) = cert.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cert, certificate)?;
    println!(
        "listening={} public_certificate={}",
        endpoint.local_addr()?,
        cert.display()
    );
    let (shutdown, stop) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = shutdown.send(true);
    });
    server::run_world(endpoint, stop, world).await
}
