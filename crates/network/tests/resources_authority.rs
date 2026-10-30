//! Actual Avian World resource/cast timing; no mocked sessions or native claims.
use serde_json::{Value, json};
use theia_core::{content::ContentId, identity::EntityKey, time::TickRate};
use theia_network::{
    simulation::{
        World,
        combat::{self, Event, Outcome, Reason, Request},
    },
    wire::{ChunkReady, Input, Intent, VERSION},
};
const ABILITIES: &[u8] =
    include_bytes!("../../../content/gameplay/development-resource-abilities.json");
const STATS: &[u8] = include_bytes!("../../../content/gameplay/development-character-stats.json");
const ID: &str = "theia.dev.arcane-bolt";
fn fixture(rate: TickRate, abilities: &[u8], stats: &[u8]) -> (World, u64, EntityKey) {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.set_tick_rate(rate);
    let target = world.spawn_development_target([8008., 8000.]).unwrap();
    world
        .enable_resource_combat(abilities, 1_000_000, 1000, stats)
        .unwrap();
    let actor = world.join().unwrap();
    let chunks = world
        .procedural()
        .unwrap()
        .patch(world.global_position(actor).unwrap())
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
    (world, actor, target)
}
fn request(world: &World, sequence: u64, target: EntityKey) -> Request {
    Request {
        epoch: world.epoch(),
        sequence,
        ability: ContentId::new(ID).unwrap(),
        target,
    }
}
fn cast(world: &mut World, actor: u64, sequence: u64, target: EntityKey) -> Outcome {
    world
        .cast(actor, request(world, sequence, target))
        .reply
        .outcome
}
fn mana(world: &World, actor: u64) -> u32 {
    world.combat_snapshot(actor).unwrap().vitals.unwrap().mana()
}
fn steps(world: &mut World, n: u32) {
    for _ in 0..n {
        world.step();
    }
}
fn stats(mana: u32, regen: u32) -> Vec<u8> {
    let mut value: Value = serde_json::from_slice(STATS).unwrap();
    value["maximum_mana"] = json!(mana);
    value["mana_regeneration_per_second"] = json!(regen);
    serde_json::to_vec(&value).unwrap()
}
#[test]
fn actual_costs_are_once_only_failed_requests_do_not_spend_and_regeneration_uses_server_ticks() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, actor, target) = fixture(rate, ABILITIES, STATS);
        let hz = rate.hz();
        assert_eq!(mana(&world, actor), 100);
        assert!(matches!(
            cast(&mut world, actor, 1, target),
            Outcome::Started { .. }
        ));
        assert_eq!(mana(&world, actor), 75);
        assert_eq!(cast(&mut world, actor, 1, target), Outcome::Ignored);
        assert_eq!(mana(&world, actor), 75);
        let mut bad = request(&world, 2, target);
        bad.ability = ContentId::new("theia.invalid.ability").unwrap();
        assert_eq!(
            world.cast(actor, bad).reply.outcome,
            Outcome::Rejected(Reason::UnknownAbility)
        );
        assert_eq!(mana(&world, actor), 75);
        steps(&mut world, hz);
        assert_eq!(mana(&world, actor), 80);
        assert_eq!(
            world.combat_snapshot(actor).unwrap().targets[0].current,
            999_980
        );
        assert_eq!(
            cast(&mut world, actor, 3, target),
            Outcome::Rejected(Reason::Unavailable)
        );
        assert_eq!(mana(&world, actor), 80);
        steps(&mut world, hz);
        assert_eq!(mana(&world, actor), 85);
        assert!(matches!(
            cast(&mut world, actor, 4, target),
            Outcome::Started { .. }
        ));
        assert_eq!(mana(&world, actor), 60);
    }
}
#[test]
fn exhaustion_rejects_without_cooldown_mutation_then_simulated_regeneration_enables_a_cast() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, actor, target) = fixture(rate, ABILITIES, STATS);
        for sequence in 1..=6 {
            assert!(matches!(
                cast(&mut world, actor, sequence, target),
                Outcome::Started { .. }
            ));
            steps(&mut world, rate.hz() * 2);
        }
        assert_eq!(mana(&world, actor), 10);
        let before = world.combat_snapshot(actor).unwrap();
        assert_eq!(
            cast(&mut world, actor, 7, target),
            Outcome::Rejected(Reason::InsufficientMana)
        );
        let after = world.combat_snapshot(actor).unwrap();
        assert_eq!(after.cooldowns, before.cooldowns);
        assert_eq!(after.gcd_until, before.gcd_until);
        assert!(after.cast.is_none());
        assert_eq!(mana(&world, actor), 10);
        steps(&mut world, rate.hz() * 3);
        assert_eq!(mana(&world, actor), 25);
        assert!(matches!(
            cast(&mut world, actor, 8, target),
            Outcome::Started { .. }
        ));
        assert_eq!(mana(&world, actor), 0);
    }
}
#[test]
fn queue_replacement_spends_only_on_actual_start_and_queued_resource_failure_has_no_new_cooldown() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, actor, target) = fixture(rate, ABILITIES, &stats(60, 0));
        let hz = rate.hz();
        assert!(matches!(
            cast(&mut world, actor, 1, target),
            Outcome::Started { .. }
        ));
        assert_eq!(mana(&world, actor), 35);
        let window = world
            .combat_catalog()
            .unwrap()
            .timing(ID, rate)
            .unwrap()
            .queue_window_ticks as u32;
        steps(&mut world, 2 * hz - window);
        assert!(matches!(
            cast(&mut world, actor, 2, target),
            Outcome::Queued { .. }
        ));
        assert_eq!(mana(&world, actor), 35);
        assert!(matches!(
            cast(&mut world, actor, 3, target),
            Outcome::Queued { .. }
        ));
        assert_eq!(mana(&world, actor), 35);
        steps(&mut world, window);
        assert_eq!(mana(&world, actor), 10);
        assert!(
            world
                .combat_events()
                .iter()
                .any(|e| matches!(e,Event::Started{cast,..} if cast.sequence==3))
        );
        steps(&mut world, 2 * hz - window);
        assert!(matches!(
            cast(&mut world, actor, 4, target),
            Outcome::Queued { .. }
        ));
        assert_eq!(mana(&world, actor), 10);
        steps(&mut world, window);
        assert!(world.combat_events().iter().any(|e| matches!(
            e,
            Event::Cancelled {
                sequence: 4,
                reason: Reason::InsufficientMana,
                ..
            }
        )));
        let snapshot = world.combat_snapshot(actor).unwrap();
        assert!(
            snapshot.cast.is_none() && snapshot.queued.is_none() && snapshot.cooldowns.is_empty()
        );
        assert!(snapshot.gcd_until <= world.completed_tick());
        assert_eq!(mana(&world, actor), 10);
    }
}
#[test]
fn interruption_keeps_the_started_cost_and_fresh_actor_resources_are_explicitly_transient() {
    let mut abilities: Value = serde_json::from_slice(ABILITIES).unwrap();
    abilities["abilities"][0]["cast_while_moving"] = json!(false);
    let (mut world, actor, target) = fixture(
        TickRate::Hz30,
        &serde_json::to_vec(&abilities).unwrap(),
        &stats(100, 0),
    );
    assert!(matches!(
        cast(&mut world, actor, 1, target),
        Outcome::Started { .. }
    ));
    assert!(world.input(
        actor,
        Input {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 1,
            intent: Intent {
                axes: [1000, 0],
                sprint: false,
                jump: false
            }
        }
    ));
    world.step();
    assert_eq!(mana(&world, actor), 75);
    assert!(world.combat_events().iter().any(|e| matches!(
        e,
        Event::Cancelled {
            reason: Reason::Moving,
            ..
        }
    )));
    world.leave(actor);
    let fresh = world.join().unwrap();
    assert_ne!(fresh, actor);
    assert_eq!(
        mana(&world, fresh),
        100,
        "This opt-in development profile does not claim durable vitals yet"
    );
}
#[test]
fn resource_configuration_is_content_bound_and_never_silently_enabled_as_zero_cost_legacy_combat() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.spawn_development_target([8008., 8000.]).unwrap();
    let base = world.identity();
    assert!(world.enable_combat(ABILITIES, 100, 1000).is_err());
    assert!(!world.combat_enabled());
    assert!(
        world
            .enable_resource_combat(ABILITIES, 100, 1000, &stats(20, 5))
            .is_err()
    );
    assert!(!world.combat_enabled());
    let expected = combat::resource_profile_identity(base, ABILITIES, 100, 1000, STATS).unwrap();
    world
        .enable_resource_combat(ABILITIES, 100, 1000, STATS)
        .unwrap();
    assert_eq!(world.identity(), expected);
    assert_ne!(
        expected,
        combat::resource_profile_identity(base, ABILITIES, 100, 1000, &stats(200, 5)).unwrap()
    );
}
