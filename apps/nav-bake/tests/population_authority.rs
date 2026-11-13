//! Actual authored collision/preparation and authoritative combat, without a renderer.
use std::{path::PathBuf, sync::Arc};
use theia_core::{content::ContentId, time::TickRate};
use theia_network::{
    authored_world,
    navigation::{self, spawn_candidates::Request, spawn_preparation},
    procedural,
    simulation::{World, combat},
    wire::{ChunkReady, VERSION},
};
const ABILITIES: &[u8] = include_bytes!("../../../content/gameplay/development-abilities.json");

#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn checked_population_uses_real_world_combat_and_fresh_respawn_ids_at_all_rates() {
    let world_path = PathBuf::from(std::env::var_os("THEIA_AUTHORED_WORLD").unwrap());
    let nav_path = PathBuf::from(std::env::var_os("THEIA_AUTHORED_NAV").unwrap());
    let request: Request = serde_json::from_slice(
        &std::fs::read(std::env::var_os("THEIA_AUTHORED_SPAWNS").unwrap()).unwrap(),
    )
    .unwrap();
    let store = Arc::new(authored_world::Store::open(&world_path).unwrap());
    let navigation = navigation::Store::open(&nav_path, store.identity()).unwrap();
    let prepare = |request: &Request| {
        spawn_preparation::check_authored(&store, &navigation, request)
            .unwrap()
            .into_prepared()
            .unwrap()
    };
    assert_eq!(
        request.count, 5,
        "This qualification uses five independent actors/targets"
    );
    let saved_spawn = store.spawn(0).unwrap().metres();
    let spawn = [saved_spawn[0], saved_spawn[2]];
    let definition = theia_network::simulation::creatures::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-creature-patrol.json"
    ))
    .unwrap()
    .definition()
    .clone();
    // Load a real serialized editor draft: runtime inputs are not copied into
    // a separate candidate request or hard-coded lifecycle binding.
    let mut zone = theia_zone::Zone::load(&world_path.join("authoring.zone.json")).unwrap();
    zone.schema = 14;
    let group_id = zone.next_id().unwrap();
    zone.spawn_groups
        .push(theia_zone::spawn_groups::SpawnGroup {
            id: group_id,
            name: "Passive qualification grove".into(),
            enabled: true,
            creature: definition.id().as_str().into(),
            model: None,
            volume: request.volume.clone(),
            target_alive: request.count as u16,
            respawn_seconds: 30,
            seed: request.seed,
        });
    zone.validate().unwrap();
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "theia-population-config-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    )));
    std::fs::create_dir(&scratch.0).unwrap();
    std::fs::write(
        scratch.0.join("zone.json"),
        serde_json::to_vec(&zone).unwrap(),
    )
    .unwrap();
    std::fs::write(
        scratch.0.join("creature.json"),
        include_bytes!("../../../content/gameplay/development-creature-patrol.json"),
    )
    .unwrap();
    std::fs::create_dir(scratch.0.join("navigation")).unwrap();
    for file in std::fs::read_dir(&nav_path).unwrap() {
        let file = file.unwrap();
        if file.file_type().unwrap().is_file() {
            std::fs::copy(
                file.path(),
                scratch.0.join("navigation").join(file.file_name()),
            )
            .unwrap();
        }
    }
    let config = serde_json::json!({
        "schema": 1, "zone": "zone.json", "group": group_id,
        "creature": "creature.json", "navigation": "navigation",
        "minimum_spacing_metres": request.minimum_spacing_metres,
        "access_from": request.access_from,
    });
    let config_path = scratch.0.join("population.json");
    std::fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    for (field, value) in [
        ("schema", serde_json::json!(2)),
        ("unknown", serde_json::json!(true)),
        ("zone", serde_json::json!("../zone.json")),
        ("group", serde_json::json!(u64::MAX)),
        ("minimum_spacing_metres", serde_json::json!(0.)),
        (
            "creature",
            serde_json::json!("navigation/navigation.manifest.json"),
        ),
    ] {
        let mut invalid = config.clone();
        invalid[field] = value;
        std::fs::write(&config_path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(
            theia_network::simulation::population::configuration::Loaded::open(
                &config_path,
                &procedural::Source::Authored(store.clone())
            )
            .is_err(),
            "{field}"
        );
    }
    std::fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let loaded = theia_network::simulation::population::configuration::Loaded::open(
            &config_path,
            &procedural::Source::Authored(store.clone()),
        )
        .unwrap();
        assert_eq!(loaded.count(), 5);
        assert_eq!(loaded.respawn_ms(), 30_000);
        let loaded_identity =
            loaded.profile_identity(theia_network::targeting::profile_identity(store.identity()));
        let (prepared, binding) = loaded.into_parts();
        let feet = prepared.points().to_vec();
        let expected_identity =
            prepared.profile_identity(theia_network::targeting::profile_identity(store.identity()));
        let mut world =
            World::with_terrain_source(spawn, 1., procedural::Source::Authored(store.clone()))
                .unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        let expected_identity = binding.profile_identity(expected_identity);
        assert_eq!(loaded_identity, expected_identity);
        let keys = world.enable_defined_population(prepared, binding).unwrap();
        assert!(
            keys.iter()
                .all(|&key| world.creature_definition(key) == Some(&definition))
        );
        assert_eq!(keys.len(), 5);
        assert_eq!(
            world.identity(),
            theia_network::classic::profile_identity(expected_identity)
        );
        assert!(!world.set_tick_rate(rate));
        let identity = world.identity();
        assert!(world.spawn_development_target(spawn).is_err());
        assert!(world.enable_prepared_population(prepare(&request)).is_err());
        assert_eq!(world.identity(), identity);
        assert!(
            world
                .enable_combat(ABILITIES, combat::DEVELOPMENT_HEALTH, 5000)
                .is_err()
        );
        assert_eq!(world.identity(), identity);
        world
            .enable_combat(ABILITIES, combat::DEVELOPMENT_HEALTH, 30_000)
            .unwrap();
        world
            .enable_progression(Arc::new(
                theia_network::progression_wire::Profile::from_json(include_bytes!(
                    "../../../content/gameplay/development-progression.json"
                ))
                .unwrap(),
            ))
            .unwrap();
        world
            .spawn_placed_guide(
                Arc::new(
                    theia_network::dialogue::Profile::from_json(include_bytes!(
                        "../../../content/quests/development-journal-dialogue.json"
                    ))
                    .unwrap(),
                ),
                theia_network::dialogue::Placement::Feet(feet[0]),
            )
            .unwrap();
        let quest_bytes = include_bytes!("../../../content/quests/development-journal.json");
        let catalog =
            Arc::new(theia_core::quests::catalog::Catalog::from_json(quest_bytes).unwrap());
        let prepared_journal = world.prepare_quest_journal(catalog).unwrap();
        let foreign = String::from_utf8(quest_bytes.to_vec())
            .unwrap()
            .replace("theia.dev.training-creature", "theia.dev.foreign-creature");
        let foreign =
            Arc::new(theia_core::quests::catalog::Catalog::from_json(foreign.as_bytes()).unwrap());
        let before = world.identity();
        assert!(world.prepare_quest_journal(foreign).is_err());
        assert_eq!(world.identity(), before);
        let mut actors = Vec::new();
        for key in &keys {
            let mut body = feet[0].metres();
            body[1] += f64::from(theia_network::world::PAWN_Y);
            let id = world
                .join_restored(theia_core::coordinates::GlobalPosition::new(body).unwrap())
                .unwrap();
            let position = world.global_position(id).unwrap();
            let chunks = world
                .procedural()
                .unwrap()
                .patch(position)
                .unwrap()
                .ids
                .iter()
                .map(|c| [c.x, c.z])
                .collect();
            assert!(world.client_ready(
                id,
                &ChunkReady {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence: 1,
                    chunks
                }
            ));
            assert!(world.select_target(id, Some(*key)));
            actors.push(id);
        }
        let initial = world.combat_snapshot(actors[0]).unwrap();
        assert_eq!(initial.targets.len(), 5);
        for (health, feet) in initial.targets.iter().zip(&feet) {
            let mut body = feet.metres();
            body[1] += f64::from(theia_network::world::PAWN_Y);
            assert_eq!(health.position.metres(), body);
        }
        let hz = u64::from(rate.hz());
        let mut fatal_events = Vec::new();
        let mut facts = Vec::new();
        let mut quest_facts = Vec::new();
        for sequence in 1..=5 {
            for (&actor, &target) in actors.iter().zip(&keys) {
                let result = world.cast(
                    actor,
                    combat::Request {
                        epoch: world.epoch(),
                        sequence,
                        target,
                        ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
                    },
                );
                assert!(
                    matches!(result.reply.outcome, combat::Outcome::Started { .. }),
                    "{:?}",
                    result.reply
                );
            }
            for _ in 0..2 * hz {
                world.step();
                let events = world.combat_events();
                facts.extend(world.experience_opportunities(events).unwrap());
                quest_facts.extend(
                    world
                        .quest_kill_occurrences(&prepared_journal, events)
                        .unwrap(),
                );
                fatal_events.extend(
                    events
                        .iter()
                        .filter(|event| matches!(event, combat::Event::Damage { remaining: 0, .. }))
                        .cloned(),
                );
            }
        }
        assert_eq!(facts.len(), 5);
        assert_eq!(quest_facts, facts);
        assert_eq!(fatal_events.len(), 5);
        for (fact, (&actor, &victim)) in facts.iter().zip(actors.iter().zip(&keys)) {
            assert_eq!(fact.actor, world.entity_key(actor).unwrap());
            assert_eq!(fact.event.id.victim(), victim);
            assert_eq!(&fact.event.creature, definition.id());
            assert!(
                world.creature_definition(victim).is_none(),
                "Dead actors remain untargetable"
            );
        }
        let doubled = [fatal_events.clone(), fatal_events.clone()].concat();
        assert_eq!(world.experience_opportunities(&doubled).unwrap(), facts);
        assert_eq!(
            world
                .quest_kill_occurrences(&prepared_journal, &doubled)
                .unwrap(),
            facts
        );
        let dead = world.combat_snapshot(actors[0]).unwrap();
        assert!(dead.targets.iter().all(|t| t.current == 0));
        let deadline = dead.targets[0].respawn_at.unwrap();
        assert_eq!(deadline, 39 * hz);
        assert!(dead.targets.iter().all(|t| t.respawn_at == Some(deadline)));
        for &actor in &actors {
            let targets = world.targeting_snapshot(actor).unwrap();
            assert!(targets.targets.is_empty());
            assert!(targets.selected.is_none());
        }
        while world.completed_tick() + 1 < deadline {
            world.step();
        }
        assert!(
            world
                .combat_snapshot(actors[0])
                .unwrap()
                .targets
                .iter()
                .all(|t| t.current == 0)
        );
        world.step();
        assert!(
            world
                .experience_opportunities(&fatal_events)
                .unwrap()
                .is_empty(),
            "Old death facts must not resolve after fresh-ID respawn"
        );
        assert!(
            world
                .quest_kill_occurrences(&prepared_journal, &fatal_events)
                .unwrap()
                .is_empty()
        );
        let alive = world.combat_snapshot(actors[0]).unwrap();
        assert_eq!(alive.targets.len(), 5);
        for (i, health) in alive.targets.iter().enumerate() {
            assert_eq!(health.current, combat::DEVELOPMENT_HEALTH);
            assert_eq!(health.respawn_at, None);
            assert!(!keys.contains(&health.key));
            assert_eq!(world.creature_definition(health.key), Some(&definition));
            assert!(
                keys.iter()
                    .all(|&old| world.creature_definition(old).is_none())
            );
            assert_eq!(health.position, initial.targets[i].position);
        }
        for (&actor, &old) in actors.iter().zip(&keys) {
            assert!(!world.select_target(actor, Some(old)));
        }
        assert_eq!(
            world.targeting_snapshot(actors[0]).unwrap().targets.len(),
            5
        );
        world.poll_terrain();
        for point in &feet {
            assert!(world.procedural().unwrap().patch(*point).is_some());
        }
        println!(
            "checked_population hz={} classic=true actors=5 targets=5 full_health=100 damage_per_cast=20 casts_per_target=5 respawn_seconds=30 stale_ids_rejected=true",
            rate.hz()
        );
    }
    let mut foreign = World::with_procedural([8000., 8000.], 1.).unwrap();
    let identity = foreign.identity();
    assert!(
        foreign
            .enable_prepared_population(prepare(&request))
            .is_err()
    );
    assert_eq!(foreign.identity(), identity);
    assert!(!foreign.targeting_enabled());
    let mut excessive = request.clone();
    excessive.count = theia_network::targeting::MAX_TARGETS + 1;
    excessive.minimum_spacing_metres = 0.25;
    let mut world =
        World::with_terrain_source(spawn, 1., procedural::Source::Authored(store.clone())).unwrap();
    let identity = world.identity();
    let missing = ContentId::new("theia.creature.missing").unwrap();
    assert!(
        theia_network::simulation::population::Binding::new(&missing, definition.clone(), 30)
            .is_err()
    );
    assert_eq!(world.identity(), identity);
    let aggressive = theia_network::simulation::creatures::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-creature-pursuit.json"
    ))
    .unwrap()
    .definition()
    .clone();
    assert!(
        theia_network::simulation::population::Binding::new(
            aggressive.id(),
            aggressive.clone(),
            30
        )
        .is_err()
    );
    for seconds in [0, 86401, u32::MAX] {
        assert!(
            theia_network::simulation::population::Binding::new(
                definition.id(),
                definition.clone(),
                seconds
            )
            .is_err()
        );
    }
    let short =
        theia_network::simulation::population::Binding::new(definition.id(), definition.clone(), 1)
            .unwrap();
    let long = theia_network::simulation::population::Binding::new(
        definition.id(),
        definition.clone(),
        86400,
    )
    .unwrap();
    assert_eq!(long.respawn_ms(), 86_400_000);
    assert_ne!(
        short.profile_identity(identity),
        long.profile_identity(identity)
    );
    assert_eq!(world.identity(), identity);
    assert!(
        world
            .enable_prepared_population(prepare(&excessive))
            .is_err()
    );
    assert_eq!(world.identity(), identity);
    assert!(!world.targeting_enabled());
    assert_eq!(
        world.enable_prepared_population(prepare(&request)).unwrap()[0]
            .entity
            .value(),
        1
    );
}
