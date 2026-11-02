use super::*;
use serde_json::{Value, json};
use theia_core::time::TickRate;
const SWORD: &str = "theia.dev.training-sword";
fn data() -> Vec<u8> {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-status-abilities.json"
    ))
    .unwrap();
    value["schema"] = json!(4);
    value["abilities"][0] = json!({"id":"theia.dev.sword-slash", "target":"hostile_alive", "range_metres":3., "cast_ms":500, "cooldown_ms":1500, "mana_cost":0, "requires_main_hand":SWORD, "animation":"sword_attack", "effect":{"kind":"damage","amount":20}});
    serde_json::to_vec(&value).unwrap()
}
fn fixture(rate: TickRate) -> (World, u64, EntityKey) {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(world.set_tick_rate(rate));
    let target = world.spawn_development_target([8002., 8000.]).unwrap();
    world
        .enable_status_combat(
            &data(),
            100,
            5000,
            include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
            include_bytes!("../../../../../content/gameplay/development-statuses.json"),
        )
        .unwrap();
    let actor = world.join().unwrap();
    let p = world.global_position(actor).unwrap();
    let chunks = world
        .procedural
        .as_ref()
        .unwrap()
        .patch(p)
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
    (world, actor, target)
}
fn gear(world: &mut World, actor: u64, revision: u64, item: Option<&str>) {
    world.observe_combat_equipment(
        world.entity_key(actor).unwrap(),
        Some((
            [7; 32],
            revision,
            item.map(|id| ContentId::new(id).unwrap()),
        )),
    );
}
fn cast(world: &mut World, actor: u64, target: EntityKey, sequence: u64) -> CastResult {
    world.cast(
        actor,
        Request {
            epoch: world.epoch(),
            sequence,
            target,
            ability: ContentId::new("theia.dev.sword-slash").unwrap(),
        },
    )
}
#[test]
fn melee_requires_trusted_committed_item_and_rejects_stale_or_conflicting_observations() {
    let (mut world, actor, target) = fixture(TickRate::Hz30);
    assert_eq!(
        cast(&mut world, actor, target, 1).reply.outcome,
        Outcome::Rejected(Reason::EquipmentRequired)
    );
    gear(&mut world, actor, 1, Some("theia.dev.training-wand"));
    assert_eq!(
        cast(&mut world, actor, target, 2).reply.outcome,
        Outcome::Rejected(Reason::EquipmentRequired)
    );
    gear(&mut world, actor, 2, None);
    gear(&mut world, actor, 1, Some(SWORD));
    assert_eq!(
        cast(&mut world, actor, target, 3).reply.outcome,
        Outcome::Rejected(Reason::EquipmentRequired)
    );
    gear(&mut world, actor, 2, Some(SWORD)); // Same revision cannot change committed contents.
    assert_eq!(
        cast(&mut world, actor, target, 4).reply.outcome,
        Outcome::Rejected(Reason::EquipmentRequired)
    );
    gear(&mut world, actor, 3, Some(SWORD));
    let key = world.entity_key(actor).unwrap();
    world.observe_combat_equipment(
        key,
        Some(([8; 32], 4, Some(ContentId::new(SWORD).unwrap()))),
    );
    assert_eq!(
        cast(&mut world, actor, target, 5).reply.outcome,
        Outcome::Rejected(Reason::EquipmentRequired)
    );
    gear(&mut world, actor, 3, Some(SWORD));
    assert!(matches!(
        cast(&mut world, actor, target, 6).reply.outcome,
        Outcome::Started { .. }
    ));
}
#[test]
fn unequip_or_backend_loss_cancels_windup_before_damage_at_every_supported_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for unavailable in [false, true] {
            let (mut world, actor, target) = fixture(rate);
            gear(&mut world, actor, 1, Some(SWORD));
            assert!(matches!(
                cast(&mut world, actor, target, 1).reply.outcome,
                Outcome::Started { .. }
            ));
            world.step();
            if unavailable {
                world.observe_combat_equipment(world.entity_key(actor).unwrap(), None);
            } else {
                gear(&mut world, actor, 2, None);
            }
            world.step();
            assert!(matches!(
                world.combat_events(),
                [Event::Cancelled {
                    reason: Reason::EquipmentRequired,
                    ..
                }]
            ));
            gear(&mut world, actor, 3, Some(SWORD));
            for _ in 0..rate.hz() {
                world.step();
            }
            assert_eq!(
                world.combat_snapshot(actor).unwrap().targets[0].current,
                100
            );
        }
    }
}
#[test]
fn melee_rechecks_range_at_completion_and_success_spends_no_mana() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for out_of_range in [false, true] {
            let (mut world, actor, target) = fixture(rate);
            gear(&mut world, actor, 1, Some(SWORD));
            let before = world.combat_snapshot(actor).unwrap().vitals.unwrap();
            assert!(matches!(
                cast(&mut world, actor, target, 1).reply.outcome,
                Outcome::Started { .. }
            ));
            if out_of_range {
                let mut p = world.combat.as_ref().unwrap().creatures[0]
                    .position
                    .metres();
                p[0] += 4.;
                world.combat.as_mut().unwrap().creatures[0].position =
                    GlobalPosition::new(p).unwrap();
            }
            for _ in 0..(rate.hz() / 2) {
                world.step();
            }
            let after = world.combat_snapshot(actor).unwrap();
            assert_eq!(
                after.targets[0].current,
                if out_of_range { 100 } else { 80 }
            );
            assert_eq!(after.vitals.unwrap(), before);
            if out_of_range {
                assert!(matches!(
                    world.combat_events(),
                    [Event::Cancelled {
                        reason: Reason::OutOfRange,
                        ..
                    }]
                ));
            } else {
                assert!(matches!(
                    world.combat_events(),
                    [Event::Damage { amount: 20, .. }]
                ));
            }
        }
    }
}
#[test]
fn queued_melee_cannot_execute_after_committed_unequip() {
    let (mut world, actor, target) = fixture(TickRate::Hz30);
    gear(&mut world, actor, 1, Some(SWORD));
    assert!(matches!(
        cast(&mut world, actor, target, 1).reply.outcome,
        Outcome::Started { .. }
    ));
    for _ in 0..37 {
        world.step();
    }
    assert!(matches!(
        cast(&mut world, actor, target, 2).reply.outcome,
        Outcome::Queued { .. }
    ));
    gear(&mut world, actor, 2, None);
    for _ in 0..8 {
        world.step();
    }
    assert!(matches!(
        world.combat_events(),
        [Event::Cancelled {
            sequence: 2,
            reason: Reason::EquipmentRequired,
            ..
        }]
    ));
    assert!(world.combat_snapshot(actor).unwrap().cast.is_none());
    assert_eq!(world.combat_snapshot(actor).unwrap().targets[0].current, 80);
}

#[test]
fn wrong_actor_epoch_and_retired_session_cannot_supply_equipment_to_a_new_session() {
    let (mut world, actor, target) = fixture(TickRate::Hz30);
    let key = world.entity_key(actor).unwrap();
    let mut foreign = key;
    foreign.epoch = theia_core::identity::WorldEpoch::new(world.epoch() + 1).unwrap();
    world.observe_combat_equipment(
        foreign,
        Some(([7; 32], 1, Some(ContentId::new(SWORD).unwrap()))),
    );
    assert_eq!(
        cast(&mut world, actor, target, 1).reply.outcome,
        Outcome::Rejected(Reason::EquipmentRequired)
    );
    gear(&mut world, actor, 1, Some(SWORD));
    world.leave(actor);
    let replacement = world.join().unwrap();
    world.observe_combat_equipment(
        key,
        Some(([7; 32], 2, Some(ContentId::new(SWORD).unwrap()))),
    );
    assert!(
        world
            .combat
            .as_ref()
            .unwrap()
            .actors
            .get(&replacement)
            .is_none_or(|a| !a.equipment.ready)
    );
}
#[test]
fn equipped_melee_rejects_a_target_beyond_three_metres_before_starting_a_cooldown() {
    let (mut world, actor, target) = fixture(TickRate::Hz30);
    gear(&mut world, actor, 1, Some(SWORD));
    let mut p = world.combat.as_ref().unwrap().creatures[0]
        .position
        .metres();
    p[0] += 4.;
    world.combat.as_mut().unwrap().creatures[0].position = GlobalPosition::new(p).unwrap();
    assert_eq!(
        cast(&mut world, actor, target, 1).reply.outcome,
        Outcome::Rejected(Reason::OutOfRange)
    );
    let state = world.combat_snapshot(actor).unwrap();
    assert!(state.cast.is_none() && state.cooldowns.is_empty());
    assert_eq!(state.gcd_until, 0);
}
