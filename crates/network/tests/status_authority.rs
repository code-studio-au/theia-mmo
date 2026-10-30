//! Actual headless Avian World status/cast timing, not network/native gameplay.
use serde_json::{Value, json};
use theia_core::{content::ContentId, identity::EntityKey, time::TickRate};
use theia_network::{
    simulation::{
        World,
        combat::{self, Event, Outcome, Reason, Request},
    },
    wire::{ChunkReady, VERSION},
};
const ABILITIES: &[u8] =
    include_bytes!("../../../content/gameplay/development-status-abilities.json");
const STATUSES: &[u8] = include_bytes!("../../../content/gameplay/development-statuses.json");
const STATS: &[u8] = include_bytes!("../../../content/gameplay/development-character-stats.json");
const ID: &str = "theia.dev.scorch";
fn fast() -> Value {
    let mut value: Value = serde_json::from_slice(ABILITIES).unwrap();
    value["global_cooldown_ms"] = json!(0);
    value["abilities"][0]["cast_ms"] = json!(0);
    value["abilities"][0]["cooldown_ms"] = json!(0);
    value["abilities"][0]["mana_cost"] = json!(0);
    value
}
fn fixture(
    rate: TickRate,
    abilities: &[u8],
    statuses: &[u8],
    health: u32,
    players: usize,
    targets: usize,
) -> (World, Vec<u64>, Vec<EntityKey>) {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(world.set_tick_rate(rate));
    let targets: Vec<_> = (0..targets)
        .map(|i| {
            world
                .spawn_development_target([8008. + 4. * i as f64, 8000.])
                .unwrap()
        })
        .collect();
    world
        .enable_status_combat(abilities, health, 1000, STATS, statuses)
        .unwrap();
    let players: Vec<_> = (0..players)
        .map(|_| {
            let actor = world.join().unwrap();
            let position = world.global_position(actor).unwrap();
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
                actor,
                &ChunkReady {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence: 1,
                    chunks
                }
            ));
            actor
        })
        .collect();
    (world, players, targets)
}
fn cast(world: &mut World, actor: u64, sequence: u64, target: EntityKey, ability: &str) -> Outcome {
    world
        .cast(
            actor,
            Request {
                epoch: world.epoch(),
                sequence,
                target,
                ability: ContentId::new(ability).unwrap(),
            },
        )
        .reply
        .outcome
}
fn hp(world: &World, actor: u64, target: EntityKey) -> u32 {
    world
        .combat_snapshot(actor)
        .unwrap()
        .targets
        .iter()
        .find(|c| c.key == target)
        .unwrap()
        .current
}
fn mana(world: &World, actor: u64) -> u32 {
    world.combat_snapshot(actor).unwrap().vitals.unwrap().mana()
}
fn steps(world: &mut World, n: u32) {
    for _ in 0..n {
        world.step();
    }
}
#[test]
fn completion_applies_no_early_damage_then_six_exact_ticks_expire_at_every_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, actors, targets) = fixture(rate, ABILITIES, STATUSES, 100, 1, 1);
        let actor = actors[0];
        let target = targets[0];
        let hz = rate.hz();
        assert!(matches!(
            cast(&mut world, actor, 1, target, ID),
            Outcome::Started { .. }
        ));
        assert_eq!(mana(&world, actor), 75);
        steps(&mut world, hz - 1);
        assert!(world.combat_status_snapshot(actor).unwrap().is_empty());
        assert_eq!(hp(&world, actor, target), 100);
        world.step();
        let status = world.combat_status_snapshot(actor).unwrap().pop().unwrap();
        assert_eq!(status.source, world.entity_key(actor).unwrap());
        assert_eq!(status.target, target);
        assert_eq!(status.next_tick, Some(2 * u64::from(hz)));
        assert_eq!(status.expires_at, 7 * u64::from(hz));
        assert!(world.combat_events().is_empty());
        assert_eq!(hp(&world, actor, target), 100);
        assert_eq!(cast(&mut world, actor, 1, target, ID), Outcome::Ignored);
        assert_eq!(mana(&world, actor), 80);
        let mut damage = Vec::new();
        for offset in 1..=6 * hz {
            world.step();
            for event in world.combat_events() {
                if let Event::Damage {
                    amount,
                    sequence,
                    target: hit,
                    ..
                } = event
                {
                    assert_eq!((*amount, *sequence, *hit), (5, 1, target));
                    damage.push(world.completed_tick());
                }
            }
            assert_eq!(hp(&world, actor, target), 100 - 5 * (offset / hz));
            assert_eq!(
                world.combat_status_snapshot(actor).unwrap().len(),
                usize::from(offset < 6 * hz)
            );
        }
        assert_eq!(
            damage,
            (2..=7).map(|n| n * u64::from(hz)).collect::<Vec<_>>()
        );
    }
}
#[test]
fn frequent_refresh_and_same_deadline_refresh_never_add_extra_ticks_or_instances() {
    let data = serde_json::to_vec(&fast()).unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, actors, targets) = fixture(rate, &data, STATUSES, 1000, 1, 1);
        let actor = actors[0];
        let target = targets[0];
        let hz = rate.hz();
        cast(&mut world, actor, 1, target, ID);
        world.step();
        let mut due = Vec::new();
        for offset in 1..=10 * hz {
            if offset <= 4 * hz {
                assert!(matches!(
                    cast(&mut world, actor, u64::from(offset) + 1, target, ID),
                    Outcome::Started { .. }
                ));
            }
            world.step();
            due.extend(world.combat_events().iter().filter_map(|e| {
                matches!(e, Event::Damage { .. }).then_some(world.completed_tick())
            }));
            assert!(world.combat_status_snapshot(actor).unwrap().len() <= 1);
        }
        assert_eq!(
            due,
            (1..=10).map(|n| 1 + n * u64::from(hz)).collect::<Vec<_>>()
        );
        assert_eq!(hp(&world, actor, target), 950);
        assert!(world.combat_status_snapshot(actor).unwrap().is_empty());
        assert_eq!(mana(&world, actor), 100);
    }
}
#[test]
fn sources_remain_independent_and_retirement_death_respawn_and_removal_clear_old_lives() {
    let data = serde_json::to_vec(&fast()).unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let hz = rate.hz();
        let (mut world, actors, targets) = fixture(rate, &data, STATUSES, 1000, 2, 1);
        let target = targets[0];
        for actor in &actors {
            cast(&mut world, *actor, 1, target, ID);
        }
        world.step();
        assert_eq!(world.combat_status_snapshot(actors[0]).unwrap().len(), 2);
        steps(&mut world, hz);
        assert_eq!(hp(&world, actors[0], target), 990);
        world.leave(actors[0]);
        let statuses = world.combat_status_snapshot(actors[1]).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].source, world.entity_key(actors[1]).unwrap());
        steps(&mut world, hz);
        assert_eq!(hp(&world, actors[1], target), 985);
        assert!(world.remove_development_target(target));
        assert!(world.combat_status_snapshot(actors[1]).unwrap().is_empty());
        steps(&mut world, 2 * hz);
        assert!(
            !world
                .combat_events()
                .iter()
                .any(|e| matches!(e, Event::Damage { .. } | Event::Respawned { .. }))
        );
        let (mut world, actors, targets) = fixture(rate, &data, STATUSES, 5, 2, 1);
        let target = targets[0];
        for actor in &actors {
            cast(&mut world, *actor, 1, target, ID);
        }
        world.step();
        steps(&mut world, hz);
        assert_eq!(
            world
                .combat_events()
                .iter()
                .filter(|e| matches!(e, Event::Damage { .. }))
                .count(),
            1
        );
        assert_eq!(
            world
                .combat_events()
                .iter()
                .filter(|e| matches!(e, Event::Died { .. }))
                .count(),
            1
        );
        assert!(world.combat_status_snapshot(actors[0]).unwrap().is_empty());
        steps(&mut world, hz);
        let fresh = world.combat_snapshot(actors[0]).unwrap().targets[0].key;
        assert_ne!(fresh, target);
        steps(&mut world, hz);
        assert_eq!(hp(&world, actors[0], fresh), 5);
        assert!(world.combat_status_snapshot(actors[0]).unwrap().is_empty());
    }
}
fn two_statuses() -> (Value, Value) {
    let mut abilities = fast();
    let mut second = abilities["abilities"][0].clone();
    second["id"] = json!("theia.test.other");
    second["effect"]["status"] = json!("theia.test.other");
    abilities["abilities"].as_array_mut().unwrap().push(second);
    let mut statuses: Value = serde_json::from_slice(STATUSES).unwrap();
    let mut second = statuses["statuses"][0].clone();
    second["id"] = json!("theia.test.other");
    statuses["statuses"].as_array_mut().unwrap().push(second);
    (abilities, statuses)
}
#[test]
fn target_and_world_caps_reject_before_cost_but_allow_refresh_and_recheck_completion() {
    let (mut abilities, statuses) = two_statuses();
    for ability in abilities["abilities"].as_array_mut().unwrap() {
        ability["mana_cost"] = json!(25);
    }
    let data = serde_json::to_vec(&abilities).unwrap();
    let statuses = serde_json::to_vec(&statuses).unwrap();
    let (mut world, actors, targets) = fixture(TickRate::Hz30, &data, &statuses, 1_000_000, 8, 3);
    for actor in &actors {
        cast(&mut world, *actor, 1, targets[0], ID);
    }
    world.step();
    assert_eq!(
        world.combat_status_snapshot(actors[0]).unwrap().len(),
        combat::MAX_TARGET_STATUSES
    );
    let before = mana(&world, actors[0]);
    assert_eq!(
        cast(&mut world, actors[0], 2, targets[0], "theia.test.other"),
        Outcome::Rejected(Reason::StatusCapacity)
    );
    assert_eq!(mana(&world, actors[0]), before);
    for (i, actor) in actors.iter().enumerate() {
        cast(
            &mut world,
            *actor,
            if i == 0 { 3 } else { 2 },
            targets[1],
            ID,
        );
    }
    world.step();
    assert_eq!(
        world.combat_status_snapshot(actors[0]).unwrap().len(),
        combat::MAX_ACTIVE_STATUSES
    );
    let before = mana(&world, actors[0]);
    assert_eq!(
        cast(&mut world, actors[0], 4, targets[2], ID),
        Outcome::Rejected(Reason::StatusCapacity)
    );
    assert_eq!(mana(&world, actors[0]), before);
    assert!(matches!(
        cast(&mut world, actors[0], 5, targets[0], ID),
        Outcome::Started { .. }
    ));
    world.step();
    assert_eq!(world.combat_status_snapshot(actors[0]).unwrap().len(), 16);
    let (mut abilities, statuses) = two_statuses();
    abilities["abilities"][0]["cast_ms"] = json!(1000);
    abilities["abilities"][0]["mana_cost"] = json!(25);
    let mut fast = abilities["abilities"][0].clone();
    fast["id"] = json!("theia.test.fast-scorch");
    fast["cast_ms"] = json!(0);
    fast["mana_cost"] = json!(0);
    abilities["abilities"].as_array_mut().unwrap().push(fast);
    let data = serde_json::to_vec(&abilities).unwrap();
    let statuses = serde_json::to_vec(&statuses).unwrap();
    let (mut world, actors, targets) = fixture(TickRate::Hz30, &data, &statuses, 1_000_000, 8, 1);
    for actor in &actors[1..] {
        cast(&mut world, *actor, 1, targets[0], "theia.test.other");
    }
    world.step();
    assert_eq!(world.combat_status_snapshot(actors[0]).unwrap().len(), 7);
    assert!(matches!(
        cast(&mut world, actors[0], 1, targets[0], ID),
        Outcome::Started { .. }
    ));
    assert_eq!(mana(&world, actors[0]), 75);
    // Another source fills the final slot before the slow cast completes.
    assert!(matches!(
        cast(
            &mut world,
            actors[1],
            2,
            targets[0],
            "theia.test.fast-scorch"
        ),
        Outcome::Started { .. }
    ));
    steps(&mut world, 30);
    assert_eq!(world.combat_status_snapshot(actors[0]).unwrap().len(), 8);
    assert!(
        !world
            .combat_status_snapshot(actors[0])
            .unwrap()
            .iter()
            .any(|s| s.source == world.entity_key(actors[0]).unwrap())
    );
    assert!(world.combat_events().iter().any(|e|matches!(e,Event::Cancelled {actor,sequence:1,reason:Reason::StatusCapacity} if actor.entity.value()==actors[0])));
    assert_eq!(
        mana(&world, actors[0]),
        80,
        "Failed completion retains its actual-start cost, plus one second of regeneration"
    );
    world.leave(actors[1]);
    assert_eq!(world.combat_status_snapshot(actors[0]).unwrap().len(), 6);
    assert!(matches!(
        cast(&mut world, actors[0], 2, targets[0], ID),
        Outcome::Started { .. }
    ));
    steps(&mut world, 30);
    assert!(
        world
            .combat_status_snapshot(actors[0])
            .unwrap()
            .iter()
            .any(|s| s.source == world.entity_key(actors[0]).unwrap())
    );
    assert_eq!(mana(&world, actors[0]), 60);
}
#[test]
fn catalogs_are_bound_and_schema_three_cannot_silently_run_without_status_definitions() {
    let (world, actors, _) = fixture(TickRate::Hz30, ABILITIES, STATUSES, 100, 1, 1);
    assert!(world.combat_status_catalog().is_some());
    assert!(world.combat_status_snapshot(actors[0]).unwrap().is_empty());
    assert!(combat::profile_identity([1; 32], ABILITIES, 100, 1000).is_err());
    assert!(combat::resource_profile_identity([1; 32], ABILITIES, 100, 1000, STATS).is_err());
    let digest =
        combat::status_profile_identity([1; 32], ABILITIES, 100, 1000, STATS, STATUSES).unwrap();
    let mut changed = STATUSES.to_vec();
    changed.push(b' ');
    assert_ne!(
        digest,
        combat::status_profile_identity([1; 32], ABILITIES, 100, 1000, STATS, &changed).unwrap()
    );
    let mut invalid: Value = serde_json::from_slice(ABILITIES).unwrap();
    invalid["abilities"][0]["effect"]["status"] = json!("theia.missing.status");
    assert!(
        combat::status_profile_identity(
            [1; 32],
            &serde_json::to_vec(&invalid).unwrap(),
            100,
            1000,
            STATS,
            STATUSES
        )
        .is_err()
    );
}
