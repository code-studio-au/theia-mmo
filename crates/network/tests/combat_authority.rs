//! Actual World + Avian authority; no mocked health or transport claims.
use std::time::{Duration, Instant};
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    simulation::{
        World,
        combat::{Event, Outcome, Reason, Request},
    },
    wire::{ChunkReady, Input, Intent, VERSION},
};
const BOLT: &str = "theia.dev.arcane-bolt";
fn data() -> Vec<u8> {
    include_bytes!("../../../content/gameplay/development-abilities.json").to_vec()
}
fn request(world: &World, sequence: u64, target: EntityKey, ability: &str) -> Request {
    Request {
        epoch: world.epoch(),
        sequence,
        target,
        ability: ContentId::new(ability).unwrap(),
    }
}
fn ready(world: &mut World, id: u64, sequence: u64) {
    let p = world.global_position(id).unwrap();
    let chunks = world
        .procedural()
        .unwrap()
        .patch(p)
        .unwrap()
        .ids
        .iter()
        .map(|id| [id.x, id.z])
        .collect();
    assert!(world.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence,
            chunks
        }
    ));
}
fn fixture(rate: TickRate, bytes: &[u8], health: u32) -> (World, EntityKey, u64) {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(world.set_tick_rate(rate));
    let target = world.spawn_development_target([8008., 8000.]).unwrap();
    world.enable_combat(bytes, health, 1000).unwrap();
    let actor = world.join().unwrap();
    ready(&mut world, actor, 1);
    (world, target, actor)
}
fn health(world: &World, actor: u64) -> u32 {
    world.combat_snapshot(actor).unwrap().targets[0].current
}
fn step(world: &mut World, n: u64) {
    for _ in 0..n {
        world.step();
        assert!(world.combat_events().len() <= 64);
    }
}
#[test]
fn mobile_cast_damage_and_cooldowns_use_exact_simulation_ticks_at_every_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, target, actor) = fixture(rate, &data(), 100);
        let hz = u64::from(rate.hz());
        let result = world.cast(actor, request(&world, 1, target, BOLT));
        assert_eq!(result.reply.outcome, Outcome::Started { completes: hz });
        assert_eq!(result.events.len(), 1);
        assert!(
            matches!(&result.events[0],Event::Started {position,..} if *position==world.global_position(actor).unwrap())
        );
        for sequence in 1..=hz {
            assert!(world.input(
                actor,
                Input {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence,
                    intent: Intent {
                        axes: [1000, 0],
                        sprint: false,
                        jump: false
                    }
                }
            ));
            world.step();
            assert_eq!(health(&world, actor), if sequence < hz { 100 } else { 80 });
        }
        assert!(
            world.global_position(actor).unwrap().metres()[0] > 8004.,
            "Mobile cast froze the actual controller"
        );
        assert_eq!(world.movement_recoveries(), 0);
        assert!(world.combat_snapshot(actor).unwrap().cast.is_none());
        assert!(matches!(
            world.combat_events(),
            [Event::Damage {
                amount: 20,
                remaining: 80,
                ..
            }]
        ));
        assert_eq!(
            world
                .cast(actor, request(&world, 2, target, BOLT))
                .reply
                .outcome,
            Outcome::Rejected(Reason::Unavailable)
        );
        step(&mut world, hz);
        assert_eq!(
            world
                .cast(actor, request(&world, 3, target, BOLT))
                .reply
                .outcome,
            Outcome::Started { completes: 3 * hz }
        );
    }
}
#[test]
fn single_queue_replacement_and_off_gcd_do_not_create_unbounded_or_packet_rate_damage() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut json: serde_json::Value = serde_json::from_slice(&data()).unwrap();
        let mut instant = json["abilities"][0].clone();
        instant["id"] = "theia.development.instant".into();
        instant["cast_ms"] = 0.into();
        instant["cooldown_ms"] = 0.into();
        instant["uses_global_cooldown"] = false.into();
        json["abilities"].as_array_mut().unwrap().push(instant);
        let bytes = serde_json::to_vec(&json).unwrap();
        let (mut world, target, actor) = fixture(rate, &bytes, 1000);
        let hz = u64::from(rate.hz());
        assert!(matches!(
            world
                .cast(actor, request(&world, 1, target, BOLT))
                .reply
                .outcome,
            Outcome::Started { .. }
        ));
        step(&mut world, hz);
        let instant = "theia.development.instant";
        assert_eq!(
            world
                .cast(actor, request(&world, 2, target, instant))
                .reply
                .outcome,
            Outcome::Started { completes: hz + 1 }
        );
        assert_eq!(
            health(&world, actor),
            980,
            "Instant must await a fixed step"
        );
        for sequence in 3..=100 {
            let result = world.cast(actor, request(&world, sequence, target, instant));
            assert_eq!(result.reply.outcome, Outcome::Queued { at: hz + 1 });
            assert!(result.events.len() <= 2);
        }
        world.step();
        assert_eq!(health(&world, actor), 960);
        let snapshot = world.combat_snapshot(actor).unwrap();
        assert_eq!(snapshot.cast.unwrap().sequence, 100);
        assert!(snapshot.queued.is_none());
        world.step();
        assert_eq!(
            health(&world, actor),
            940,
            "Only the latest queued spell can hit"
        );
        let window = rate.ticks_for(Duration::from_millis(300)).unwrap();
        step(&mut world, hz - window - 2);
        assert_eq!(
            world
                .cast(actor, request(&world, 101, target, BOLT))
                .reply
                .outcome,
            Outcome::Queued { at: 2 * hz }
        );
        step(&mut world, window);
        assert_eq!(
            world.combat_snapshot(actor).unwrap().cast.unwrap().started,
            2 * hz
        );
    }
}
#[test]
fn stale_forged_unknown_and_unready_requests_do_not_damage_or_poison_future_sequences() {
    let (mut world, target, actor) = fixture(TickRate::Hz30, &data(), 100);
    let mut foreign = request(&world, 1, target, BOLT);
    foreign.epoch = if foreign.epoch == 1 { 2 } else { 1 };
    assert_eq!(world.cast(actor, foreign).reply.outcome, Outcome::Ignored);
    assert_eq!(
        world
            .cast(actor, request(&world, u64::MAX, target, BOLT))
            .reply
            .outcome,
        Outcome::Ignored
    );
    let mut foreign_target = request(&world, 1, target, BOLT);
    foreign_target.target.epoch = WorldEpoch::new(if world.epoch() == 1 { 2 } else { 1 }).unwrap();
    assert_eq!(
        world.cast(actor, foreign_target).reply.outcome,
        Outcome::Ignored
    );
    assert_eq!(
        world
            .cast(u64::MAX, request(&world, 1, target, BOLT))
            .reply
            .outcome,
        Outcome::Ignored
    );
    let player = world.entity_key(actor).unwrap();
    assert_eq!(
        world
            .cast(actor, request(&world, 1, player, BOLT))
            .reply
            .outcome,
        Outcome::Rejected(Reason::UnknownTarget)
    );
    assert_eq!(
        world
            .cast(
                actor,
                request(&world, 2, target, "theia.development.unknown")
            )
            .reply
            .outcome,
        Outcome::Rejected(Reason::UnknownAbility)
    );
    let mut missing = target;
    missing.entity = EntityId::new(9999).unwrap();
    assert_eq!(
        world
            .cast(actor, request(&world, 3, missing, BOLT))
            .reply
            .outcome,
        Outcome::Rejected(Reason::UnknownTarget)
    );
    assert!(world.client_ready(
        actor,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 2,
            chunks: vec![]
        }
    ));
    assert_eq!(
        world
            .cast(actor, request(&world, 4, target, BOLT))
            .reply
            .outcome,
        Outcome::Rejected(Reason::NotReady)
    );
    ready(&mut world, actor, 3);
    assert!(matches!(
        world
            .cast(actor, request(&world, 5, target, BOLT))
            .reply
            .outcome,
        Outcome::Started { .. }
    ));
    assert_eq!(
        world
            .cast(actor, request(&world, 5, target, BOLT))
            .reply
            .outcome,
        Outcome::Ignored
    );
    step(&mut world, 30);
    assert_eq!(health(&world, actor), 80);
}
#[test]
fn competing_lethal_hits_clear_selection_and_respawn_with_a_fresh_identity_at_every_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, target, a) = fixture(rate, &data(), 20);
        let b = world.join().unwrap();
        ready(&mut world, b, 1);
        assert!(world.select_target(a, Some(target)) && world.select_target(b, Some(target)));
        assert!(matches!(
            world
                .cast(a, request(&world, 1, target, BOLT))
                .reply
                .outcome,
            Outcome::Started { .. }
        ));
        assert!(matches!(
            world
                .cast(b, request(&world, 1, target, BOLT))
                .reply
                .outcome,
            Outcome::Started { .. }
        ));
        let hz = u64::from(rate.hz());
        step(&mut world, hz);
        assert_eq!(health(&world, a), 0);
        assert_eq!(
            world
                .combat_events()
                .iter()
                .filter(|e| matches!(e, Event::Died { .. }))
                .count(),
            1
        );
        assert_eq!(
            world
                .combat_events()
                .iter()
                .filter(|e| matches!(e, Event::Damage { .. }))
                .count(),
            1
        );
        assert!(world.targeting_snapshot(a).unwrap().targets.is_empty());
        assert!(
            world.targeting_snapshot(a).unwrap().selected.is_none()
                && world.targeting_snapshot(b).unwrap().selected.is_none()
        );
        assert_eq!(
            world
                .cast(b, request(&world, 2, target, BOLT))
                .reply
                .outcome,
            Outcome::Rejected(Reason::Dead)
        );
        step(&mut world, hz - 1);
        assert_eq!(health(&world, a), 0);
        world.step();
        let new = world.targeting_snapshot(a).unwrap().targets[0].key;
        assert_ne!(new, target);
        assert_eq!(new.epoch, target.epoch);
        assert!(new.entity > target.entity);
        assert_eq!(health(&world, a), 20);
        assert_eq!(
            world
                .cast(a, request(&world, 2, target, BOLT))
                .reply
                .outcome,
            Outcome::Rejected(Reason::UnknownTarget)
        );
        assert!(matches!(
            world.cast(a, request(&world, 3, new, BOLT)).reply.outcome,
            Outcome::Started { .. }
        ));
        world.leave(a);
        let replacement = world.join().unwrap();
        ready(&mut world, replacement, 1);
        assert!(world.combat_snapshot(replacement).unwrap().cast.is_none());
        step(&mut world, hz);
        assert_eq!(
            health(&world, replacement),
            20,
            "Disconnected caster cannot hit"
        );
        assert!(world.remove_development_target(new));
        step(&mut world, 2 * hz);
        assert!(world.combat_snapshot(b).unwrap().targets.is_empty());
    }
}
#[test]
fn restricted_movement_readiness_loss_and_teleport_interrupt_without_cooldown_refund() {
    let mut json: serde_json::Value = serde_json::from_slice(&data()).unwrap();
    json["abilities"][0]["cast_while_moving"] = false.into();
    let bytes = serde_json::to_vec(&json).unwrap();
    let (mut world, target, actor) = fixture(TickRate::Hz30, &bytes, 100);
    step(&mut world, 2);
    let starts = world.combat_snapshot(actor).unwrap().tick;
    assert!(matches!(
        world
            .cast(actor, request(&world, 1, target, BOLT))
            .reply
            .outcome,
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
                ..Intent::default()
            }
        }
    ));
    world.step();
    assert!(matches!(
        world.combat_events(),
        [Event::Cancelled {
            reason: Reason::Moving,
            ..
        }]
    ));
    assert_eq!(health(&world, actor), 100);
    assert_eq!(
        world.combat_snapshot(actor).unwrap().cooldowns[0].1,
        starts + 60
    );
    step(&mut world, 60);
    assert!(matches!(
        world
            .cast(actor, request(&world, 2, target, BOLT))
            .reply
            .outcome,
        Outcome::Started { .. }
    ));
    assert!(world.client_ready(
        actor,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 2,
            chunks: vec![]
        }
    ));
    world.step();
    assert!(matches!(
        world.combat_events(),
        [Event::Cancelled {
            reason: Reason::NotReady,
            ..
        }]
    ));
    ready(&mut world, actor, 3);
    step(&mut world, 60);
    assert!(matches!(
        world
            .cast(actor, request(&world, 3, target, BOLT))
            .reply
            .outcome,
        Outcome::Started { .. }
    ));
    assert!(world.teleport_development(actor, [8000., 8000.]));
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        world.poll_terrain();
        if world.global_position(actor).unwrap().metres()[0] == 8000. {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    ready(&mut world, actor, 4);
    world.step();
    assert!(matches!(
        world.combat_events(),
        [Event::Cancelled {
            reason: Reason::Interrupted,
            ..
        }]
    ));
    assert_eq!(health(&world, actor), 100);
}
#[test]
fn combat_configuration_is_inactive_bounded_and_identity_bound() {
    let mut base = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(base.enable_combat(&data(), 100, 1000).is_err());
    base.spawn_development_target([8008., 8000.]).unwrap();
    let before = base.identity();
    assert!(base.enable_combat(&data(), 0, 1000).is_err());
    assert!(base.enable_combat(&data(), 100, 99).is_err());
    assert!(base.enable_combat(b"{}", 100, 1000).is_err());
    base.enable_combat(&data(), 100, 1000).unwrap();
    assert_ne!(before, base.identity());
    assert_eq!(
        base.identity(),
        theia_network::simulation::combat::profile_identity(before, &data(), 100, 1000).unwrap()
    );
    assert!(!base.set_tick_rate(TickRate::Hz60));
    assert!(base.spawn_development_target([8009., 8000.]).is_err());
    assert!(base.enable_combat(&data(), 100, 1000).is_err());
    let mut other = World::with_procedural([8000., 8000.], 1.).unwrap();
    other.spawn_development_target([8008., 8000.]).unwrap();
    other.enable_combat(&data(), 101, 1000).unwrap();
    assert_ne!(base.identity(), other.identity());
}

#[test]
fn completion_rechecks_range_and_owner_removal_cancels_a_dead_creatures_future_respawn() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut json: serde_json::Value = serde_json::from_slice(&data()).unwrap();
        json["abilities"][0]["range_metres"] = 10.into();
        let (mut world, target, actor) = fixture(rate, &serde_json::to_vec(&json).unwrap(), 20);
        assert!(matches!(
            world
                .cast(actor, request(&world, 1, target, BOLT))
                .reply
                .outcome,
            Outcome::Started { .. }
        ));
        for sequence in 1..=u64::from(rate.hz()) {
            assert!(world.input(
                actor,
                Input {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence,
                    intent: Intent {
                        axes: [-1000, 0],
                        ..Intent::default()
                    }
                }
            ));
            world.step();
        }
        assert!(matches!(
            world.combat_events(),
            [Event::Cancelled {
                reason: Reason::OutOfRange,
                ..
            }]
        ));
        assert_eq!(health(&world, actor), 20);
        assert_eq!(
            world
                .cast(actor, request(&world, 2, target, BOLT))
                .reply
                .outcome,
            Outcome::Rejected(Reason::OutOfRange)
        );
        // A separate authority proves owner deletion cancels a scheduled respawn.
        let (mut world, target, actor) = fixture(rate, &data(), 20);
        world.cast(actor, request(&world, 1, target, BOLT));
        step(&mut world, u64::from(rate.hz()));
        assert_eq!(health(&world, actor), 0);
        assert!(world.remove_development_target(target));
        step(&mut world, 2 * u64::from(rate.hz()));
        assert!(world.combat_snapshot(actor).unwrap().targets.is_empty());
        assert!(world.targeting_snapshot(actor).unwrap().targets.is_empty());
    }
}

#[test]
fn a_new_cast_cannot_hold_an_older_queued_spell_beyond_its_original_deadline() {
    let mut json: serde_json::Value = serde_json::from_slice(&data()).unwrap();
    let mut second = json["abilities"][0].clone();
    second["id"] = "theia.dev.off-gcd-cast".into();
    second["uses_global_cooldown"] = false.into();
    second["cooldown_ms"] = 0.into();
    json["abilities"].as_array_mut().unwrap().push(second);
    let (mut world, target, actor) =
        fixture(TickRate::Hz30, &serde_json::to_vec(&json).unwrap(), 100);
    world.cast(actor, request(&world, 1, target, BOLT));
    step(&mut world, 51);
    assert_eq!(
        world
            .cast(actor, request(&world, 2, target, BOLT))
            .reply
            .outcome,
        Outcome::Queued { at: 60 }
    );
    assert!(matches!(
        world
            .cast(actor, request(&world, 3, target, "theia.dev.off-gcd-cast"))
            .reply
            .outcome,
        Outcome::Started { completes: 81 }
    ));
    step(&mut world, 9);
    assert!(world.combat_snapshot(actor).unwrap().queued.is_none());
    assert!(world.combat_events().iter().any(|e| matches!(
        e,
        Event::Cancelled {
            sequence: 2,
            reason: Reason::Unavailable,
            ..
        }
    )));
    step(&mut world, 60);
    assert_eq!(health(&world, actor), 60);
}

#[test]
fn a_later_actors_lethal_hit_cancels_an_earlier_actors_unfinished_cast_in_the_same_step() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, target, earlier) = fixture(rate, &data(), 20);
        let later = world.join().unwrap();
        ready(&mut world, later, 1);
        world.cast(later, request(&world, 1, target, BOLT));
        world.step();
        world.cast(earlier, request(&world, 1, target, BOLT));
        step(&mut world, u64::from(rate.hz()) - 1);
        assert_eq!(health(&world, earlier), 0);
        assert!(world.combat_snapshot(earlier).unwrap().cast.is_none());
        assert!(world.combat_snapshot(later).unwrap().cast.is_none());
        assert!(world.combat_events().iter().any(|event|matches!(event,Event::Cancelled {actor,reason:Reason::Dead,..} if actor.entity.value()==earlier)));
    }
}

#[test]
fn impact_death_and_respawn_positions_survive_entity_retirement() {
    let (mut world, target, actor) = fixture(TickRate::Hz30, &data(), 20);
    let position = world.targeting_snapshot(actor).unwrap().targets[0].position;
    world.cast(actor, request(&world, 1, target, BOLT));
    step(&mut world, 30);
    assert!(world.targeting_snapshot(actor).unwrap().targets.is_empty());
    assert!(
        world
            .combat_events()
            .iter()
            .any(|e| matches!(e,Event::Damage {position:p,..} if *p==position))
    );
    assert!(
        world
            .combat_events()
            .iter()
            .any(|e| matches!(e,Event::Died {position:p,..} if *p==position))
    );
    step(&mut world, 30);
    assert!(
        world
            .combat_events()
            .iter()
            .any(|e| matches!(e,Event::Respawned {position:p,..} if *p==position))
    );
}
