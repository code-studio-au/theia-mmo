use super::*;
use serde_json::{Value, json};
use std::sync::Arc;
use theia_core::{
    hit_outcomes::{Eligibility, Outcome as Resolution, Table},
    time::TickRate,
};
const ABILITIES: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-melee-abilities.json");
const STATS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-character-stats.json");
const STATUSES: &[u8] = include_bytes!("../../../../../content/gameplay/development-statuses.json");
fn fixture(
    rate: TickRate,
    table: Option<Table>,
    deflect: bool,
    fast: bool,
) -> (World, u64, EntityKey) {
    let mut profile: Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-creature-patrol.json"
    ))
    .unwrap();
    profile["schema"] = json!(2);
    profile["can_deflect"] = json!(deflect);
    profile["wait_ms"] = json!(60000);
    let (mut world, target) = crate::simulation::creatures::tests::fixture_profile_at(
        rate,
        [8006.5, 8000.],
        &serde_json::to_vec(&profile).unwrap(),
    );
    let mut data: Value = serde_json::from_slice(ABILITIES).unwrap();
    if let Some(table) = table {
        data["schema"] = json!(5);
        data["abilities"][1]["hit_table"] = json!({"hit_bp":table.hit_bp(),"miss_bp":table.miss_bp(),"deflect_bp":table.deflect_bp()});
    }
    data["abilities"][1]["mana_cost"] = json!(if fast { 0 } else { 25 });
    if fast {
        data["abilities"][1]["cast_ms"] = json!(0);
        data["abilities"][1]["cooldown_ms"] = json!(0);
        data["abilities"][1]["uses_global_cooldown"] = json!(false);
        data["abilities"][1]["effect"]["amount"] = json!(1);
    }
    let mut stats: Value = serde_json::from_slice(STATS).unwrap();
    stats["mana_regeneration_per_second"] = json!(0);
    world
        .enable_status_combat(
            &serde_json::to_vec(&data).unwrap(),
            if fast { 1000 } else { 100 },
            5000,
            &serde_json::to_vec(&stats).unwrap(),
            STATUSES,
        )
        .unwrap();
    let items = crate::renet_runtime::inventory::Profile::from_json(include_bytes!(
        "../../../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    world
        .enable_rewards(Arc::new(
            crate::reward_wire::Profile::from_json(
                include_bytes!("../../../../../content/gameplay/development-rewards.json"),
                &items,
            )
            .unwrap(),
        ))
        .unwrap();
    world
        .enable_progression(Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        ))
        .unwrap();
    let actor = world.join().unwrap();
    let position = world.global_position(actor).unwrap();
    let chunks = world
        .procedural
        .as_ref()
        .unwrap()
        .patch(position)
        .unwrap()
        .ids
        .iter()
        .map(|id| [id.x, id.z])
        .collect();
    assert!(world.client_ready(
        actor,
        &crate::wire::ChunkReady {
            version: crate::wire::VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks
        }
    ));
    world.observe_combat_equipment(
        world.entity_key(actor).unwrap(),
        Some((
            items.hash(),
            1,
            Some(ContentId::new("theia.dev.training-sword").unwrap()),
        )),
    );
    (world, actor, target)
}
fn request(world: &World, target: EntityKey, sequence: u64) -> Request {
    Request {
        epoch: world.epoch(),
        sequence,
        ability: ContentId::new("theia.dev.sword-slash").unwrap(),
        target,
    }
}
fn advance_cast(world: &mut World, rate: TickRate) {
    for _ in 0..rate.hz() / 2 {
        world.step();
    }
}
#[test]
fn valid_hit_miss_deflect_and_ineligible_deflect_keep_started_costs_at_every_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for (table, deflect, expected) in [
            (Table::certain_hit(), true, Resolution::Hit),
            (Table::new(0, 10000, 0).unwrap(), true, Resolution::Miss),
            (Table::new(0, 0, 10000).unwrap(), true, Resolution::Deflect),
            (Table::new(0, 0, 10000).unwrap(), false, Resolution::Hit),
        ] {
            let (mut world, actor, target) = fixture(rate, Some(table), deflect, false);
            assert!(matches!(
                world.cast(actor, request(&world, target, 1)).reply.outcome,
                Outcome::Started { .. }
            ));
            assert_eq!(world.combat_outcome_metrics().unwrap().random_draws, 0);
            assert_eq!(
                world.cast(actor, request(&world, target, 1)).reply.outcome,
                Outcome::Ignored
            );
            advance_cast(&mut world, rate);
            let metrics = world.combat_outcome_metrics().unwrap();
            assert_eq!(metrics.random_draws, 1);
            assert_eq!(metrics.resolved, 1);
            let snapshot = world.combat_snapshot(actor).unwrap();
            assert_eq!(snapshot.vitals.unwrap().mana(), 75);
            assert!(snapshot.gcd_until > world.tick && !snapshot.cooldowns.is_empty());
            let event = world.combat_events();
            match expected {
                Resolution::Hit => {
                    assert_eq!(snapshot.targets[0].current, 80);
                    assert!(matches!(event, [Event::Damage { amount: 20, .. }]));
                    assert_eq!(metrics.hits, 1);
                }
                Resolution::Miss => {
                    assert_eq!(snapshot.targets[0].current, 100);
                    assert!(matches!(
                        event,
                        [Event::Avoided {
                            kind: Avoidance::Miss,
                            ..
                        }]
                    ));
                    assert_eq!(metrics.misses, 1);
                }
                Resolution::Deflect => {
                    assert_eq!(snapshot.targets[0].current, 100);
                    assert!(matches!(
                        event,
                        [Event::Avoided {
                            kind: Avoidance::Deflect,
                            ..
                        }]
                    ));
                    assert_eq!(metrics.deflects, 1);
                }
            }
            assert!(world.reward_opportunities(event).unwrap().is_empty());
            assert!(world.experience_opportunities(event).unwrap().is_empty());
            for _ in 0..3 {
                world.step();
            }
            assert_eq!(world.combat_outcome_metrics().unwrap().random_draws, 1);
        }
    }
}
#[test]
fn rejected_start_and_cancelled_windup_do_not_advance_randomness_or_resolve_damage() {
    let (mut world, actor, target) = fixture(
        TickRate::Hz30,
        Some(Table::new(8500, 1000, 500).unwrap()),
        true,
        false,
    );
    let original = world.combat.as_ref().unwrap().creatures[0].position;
    let mut point = original.metres();
    point[0] += 10.;
    world.combat.as_mut().unwrap().creatures[0].position = GlobalPosition::new(point).unwrap();
    assert_eq!(
        world.cast(actor, request(&world, target, 1)).reply.outcome,
        Outcome::Rejected(Reason::OutOfRange)
    );
    assert_eq!(world.combat_outcome_metrics().unwrap().random_draws, 0);
    world.combat.as_mut().unwrap().creatures[0].position = original;
    assert!(matches!(
        world.cast(actor, request(&world, target, 2)).reply.outcome,
        Outcome::Started { .. }
    ));
    world.observe_combat_equipment(world.entity_key(actor).unwrap(), None);
    world.step();
    assert!(matches!(
        world.combat_events(),
        [Event::Cancelled {
            reason: Reason::EquipmentRequired,
            ..
        }]
    ));
    let metrics = world.combat_outcome_metrics().unwrap();
    assert_eq!(metrics.resolved, 0);
    assert_eq!(metrics.random_draws, 0);
    assert_eq!(
        world.combat_snapshot(actor).unwrap().targets[0].current,
        100
    );
}
#[test]
fn legacy_damage_remains_guaranteed_and_allocates_no_random_stream() {
    let (mut world, actor, target) = fixture(TickRate::Hz30, None, true, false);
    assert!(world.combat.as_ref().unwrap().random.is_none());
    assert!(world.combat_outcome_metrics().is_none());
    world.cast(actor, request(&world, target, 1));
    advance_cast(&mut world, TickRate::Hz30);
    assert!(matches!(
        world.combat_events(),
        [Event::Damage { amount: 20, .. }]
    ));
    assert_eq!(world.combat_snapshot(actor).unwrap().targets[0].current, 80);
}
#[test]
fn seeded_mixed_completions_match_private_stream_and_duplicates_cannot_select_a_roll() {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    let table = Table::new(8500, 1000, 500).unwrap();
    let (mut world, actor, target) = fixture(TickRate::Hz30, Some(table), true, true);
    world.combat.as_mut().unwrap().random = Some(outcomes::Random::seed_for_test([9; 32]));
    let mut oracle = ChaCha20Rng::from_seed([9; 32]);
    let mut health = 1000;
    for sequence in 1..=100 {
        let draw = oracle.random_range(0..theia_core::hit_outcomes::BASIS_POINTS);
        let expected = table.resolve(
            theia_core::hit_outcomes::Roll::new(draw).unwrap(),
            Eligibility { deflect: true },
        );
        world.cast(actor, request(&world, target, sequence));
        assert_eq!(
            world
                .cast(actor, request(&world, target, sequence))
                .reply
                .outcome,
            Outcome::Ignored
        );
        world.step();
        match expected {
            Resolution::Hit => {
                health -= 1;
                assert!(matches!(
                    world.combat_events(),
                    [Event::Damage { amount: 1, .. }]
                ));
            }
            Resolution::Miss => assert!(matches!(
                world.combat_events(),
                [Event::Avoided {
                    kind: Avoidance::Miss,
                    ..
                }]
            )),
            Resolution::Deflect => assert!(matches!(
                world.combat_events(),
                [Event::Avoided {
                    kind: Avoidance::Deflect,
                    ..
                }]
            )),
        }
        assert_eq!(
            world.combat_snapshot(actor).unwrap().targets[0].current,
            health
        );
        assert_eq!(
            world.combat_outcome_metrics().unwrap().random_draws,
            sequence
        );
    }
}
