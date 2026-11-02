use super::*;
use crate::{
    commands::Command,
    prediction_wire::Commands,
    wire::{ChunkReady, Intent, VERSION},
};
use std::sync::Arc;
use theia_core::{coordinates::GlobalPosition, facing::Facing, time::TickRate};
const WEAPONS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-melee-weapons.json");
const ITEMS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-equipment-items.json");
const ABILITIES: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-abilities.json");
fn inactive(rate: TickRate) -> (World, EntityKey) {
    let mut w = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(w.set_tick_rate(rate));
    w.enable_prediction().unwrap();
    w.enable_classic().unwrap();
    let key = w.spawn_development_target([8000., 7998.]).unwrap();
    w.enable_combat(ABILITIES, 100, 5000).unwrap();
    (w, key)
}
fn fixture(rate: TickRate) -> (World, u64, EntityKey) {
    let (mut w, key) = inactive(rate);
    w.enable_autoattack(WEAPONS, ITEMS).unwrap();
    let id = w.join().unwrap();
    ready(&mut w, id, 1);
    gear(&mut w, id, 1, Some("theia.dev.training-sword"));
    (w, id, key)
}
fn ready(w: &mut World, id: u64, sequence: u64) {
    let chunks = w
        .procedural
        .as_ref()
        .unwrap()
        .patch(w.global_position(id).unwrap())
        .unwrap()
        .ids
        .iter()
        .map(|c| [c.x, c.z])
        .collect();
    assert!(w.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: w.epoch(),
            sequence,
            chunks
        }
    ));
}
fn gear(w: &mut World, id: u64, revision: u64, item: Option<&str>) {
    w.observe_combat_equipment(
        w.entity_key(id).unwrap(),
        Some((
            crate::terrain::sha256(ITEMS),
            revision,
            item.map(|id| ContentId::new(id).unwrap()),
        )),
    );
}
fn request(w: &mut World, id: u64, sequence: u64, target: Option<EntityKey>) -> Reply {
    w.request_autoattack(
        id,
        Request {
            stream_epoch: w.correction(id).map_or(1, |c| c.state.input.stream_epoch),
            epoch: w.epoch(),
            sequence,
            target,
        },
    )
}
fn turn(w: &mut World, id: u64, facing: u16) {
    let c = w.correction(id).unwrap().state;
    assert_eq!(
        w.command(
            id,
            &Commands {
                epoch: w.epoch(),
                stream_epoch: c.input.stream_epoch,
                ready_sequence: c.ready_sequence,
                commands: vec![Command {
                    sequence: c.input.applied_sequence + 1,
                    tick: c.input.tick + 1,
                    intent: Intent {
                        facing: Some(Facing::from_units(facing)),
                        ..Intent::default()
                    }
                }]
            }
        ),
        1
    );
    w.step();
}
fn health(w: &World, target: EntityKey) -> u32 {
    w.combat
        .as_ref()
        .unwrap()
        .creatures
        .iter()
        .find(|c| c.key == target)
        .unwrap()
        .current
}
#[test]
fn actual_world_swings_follow_weapon_period_not_gcd_or_repeated_engage_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = fixture(rate);
        let initial = w.correction(id).unwrap().state.pose;
        assert_eq!(
            request(&mut w, id, 1, Some(target)).outcome,
            Outcome::Engaged
        );
        w.step();
        assert_eq!(health(&w, target), 85);
        assert!(matches!(
            w.combat_events(),
            [Event::MeleeDamage {
                sequence: 1,
                amount: 15,
                ..
            }]
        ));
        let deadline = w.autoattack_snapshot(id).unwrap().ready_at.unwrap();
        let period = u64::from(rate.hz()) * 24 / 10;
        assert_eq!(deadline, w.tick + period);
        assert_eq!(w.combat_snapshot(id).unwrap().gcd_until, 0);
        assert!(w.combat_snapshot(id).unwrap().cast.is_none());
        assert_eq!(
            request(&mut w, id, 1, Some(target)).outcome,
            Outcome::Ignored
        );
        assert_eq!(request(&mut w, id, 2, None).outcome, Outcome::Stopped);
        assert_eq!(w.targeting_snapshot(id).unwrap().selected, Some(target));
        assert_eq!(
            request(&mut w, id, 3, Some(target)).outcome,
            Outcome::Engaged
        );
        assert_eq!(w.autoattack_snapshot(id).unwrap().ready_at, Some(deadline));
        while w.tick + 1 < deadline {
            w.step();
            assert_eq!(health(&w, target), 85);
        }
        w.step();
        assert_eq!(health(&w, target), 70);
        assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 2);
        assert_eq!(w.correction(id).unwrap().state.pose.facing, initial.facing);
    }
}
#[test]
fn heading_range_los_and_readiness_pause_without_clearing_selection_or_engagement() {
    let (mut w, id, target) = fixture(TickRate::Hz30);
    request(&mut w, id, 1, Some(target));
    w.step();
    turn(&mut w, id, 32768);
    assert_eq!(
        w.autoattack_snapshot(id).unwrap().phase,
        Phase::Paused(Pause::NotFacing)
    );
    for _ in 0..100 {
        w.step();
    }
    assert_eq!(health(&w, target), 85);
    assert_eq!(w.autoattack_snapshot(id).unwrap().target, Some(target));
    assert_eq!(w.targeting_snapshot(id).unwrap().selected, Some(target));
    turn(&mut w, id, 0);
    assert_eq!(health(&w, target), 70);
    let original = w.combat.as_ref().unwrap().creatures[0].position;
    let mut far = original.metres();
    far[2] -= 8.;
    let far = GlobalPosition::new(far).unwrap();
    w.combat.as_mut().unwrap().move_target(target, far);
    w.targets.get_mut(&target.entity.value()).unwrap().position = far;
    for _ in 0..100 {
        w.step();
    }
    assert_eq!(
        w.autoattack_snapshot(id).unwrap().phase,
        Phase::Paused(Pause::OutOfRange)
    );
    assert_eq!(health(&w, target), 70);
    w.combat.as_mut().unwrap().move_target(target, original);
    w.targets.get_mut(&target.entity.value()).unwrap().position = original;
    w.step();
    assert_eq!(health(&w, target), 55);
    // Replace only the fixture's immutable collision patch, keeping its floor.
    let position = w.global_position(id).unwrap();
    let runtime = w.procedural.as_mut().unwrap();
    let prior = runtime.patch(position).unwrap().clone();
    let mut colliders = prior.scene.colliders.clone();
    let a = position.metres();
    let b = original.metres();
    let centre = GlobalPosition::new(std::array::from_fn(|i| (a[i] + b[i]) * 0.5)).unwrap();
    colliders.push(crate::physics::PlacedCollider {
        shape: avian3d::prelude::Collider::cuboid(4., 5., 0.05),
        center: bevy::prelude::Vec3::from_array(prior.frame.to_local(centre).unwrap().metres()),
        layer: crate::physics::SOLID,
    });
    runtime.patches.insert(
        prior.owner,
        Arc::new(crate::procedural::Patch {
            owner: prior.owner,
            frame: prior.frame,
            ids: prior.ids.clone(),
            scene: crate::physics::StaticScene::new(colliders),
        }),
    );
    for _ in 0..100 {
        w.step();
    }
    assert_eq!(
        w.autoattack_snapshot(id).unwrap().phase,
        Phase::Paused(Pause::Obstructed)
    );
    assert_eq!(health(&w, target), 55);
    w.procedural
        .as_mut()
        .unwrap()
        .patches
        .insert(prior.owner, prior);
    w.step();
    assert_eq!(health(&w, target), 40);
    assert!(w.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: w.epoch(),
            sequence: 2,
            chunks: vec![]
        }
    ));
    for _ in 0..100 {
        w.step();
    }
    assert_eq!(
        w.autoattack_snapshot(id).unwrap().phase,
        Phase::Paused(Pause::NotReady)
    );
    assert_eq!(health(&w, target), 40);
    ready(&mut w, id, 3);
    w.step();
    assert_eq!(health(&w, target), 25);
    assert_eq!(w.autoattack_snapshot(id).unwrap().target, Some(target));
}
#[test]
fn committed_equipment_and_profile_hash_are_required_and_unavailable_gear_pauses() {
    let (mut w, id, target) = fixture(TickRate::Hz30);
    gear(&mut w, id, 2, None);
    assert_eq!(
        request(&mut w, id, 1, Some(target)).outcome,
        Outcome::Rejected(Pause::NoWeapon)
    );
    gear(&mut w, id, 3, Some("theia.dev.training-sword"));
    assert_eq!(
        request(&mut w, id, 1, Some(target)).outcome,
        Outcome::Ignored
    );
    request(&mut w, id, 2, Some(target));
    w.step();
    assert_eq!(health(&w, target), 85);
    w.observe_combat_equipment(w.entity_key(id).unwrap(), None);
    for _ in 0..100 {
        w.step();
    }
    assert_eq!(
        w.autoattack_snapshot(id).unwrap().phase,
        Phase::Paused(Pause::NoWeapon)
    );
    assert_eq!(health(&w, target), 85);
    gear(&mut w, id, 4, Some("theia.dev.training-sword"));
    w.step();
    assert_eq!(health(&w, target), 70);
    // A conflicting/stale observation cannot authorize a basic swing.
    w.observe_combat_equipment(
        w.entity_key(id).unwrap(),
        Some((
            [3; 32],
            5,
            Some(ContentId::new("theia.dev.training-sword").unwrap()),
        )),
    );
    for _ in 0..100 {
        w.step();
    }
    assert_eq!(health(&w, target), 70);
    gear(&mut w, id, 5, Some("theia.dev.training-sword"));
    w.step();
    assert_eq!(health(&w, target), 55);
}
#[test]
fn explicit_target_clear_removal_and_rejoin_end_engagement_and_replays_cannot_restore_it() {
    let (mut w, id, target) = fixture(TickRate::Hz30);
    request(&mut w, id, 1, Some(target));
    w.step();
    assert!(w.select_target(id, None));
    assert_eq!(w.autoattack_snapshot(id).unwrap().target, None);
    for _ in 0..100 {
        w.step();
    }
    assert_eq!(health(&w, target), 85);
    request(&mut w, id, 2, Some(target));
    w.step();
    assert_eq!(health(&w, target), 70);
    assert!(w.remove_development_target(target));
    assert_eq!(w.autoattack_snapshot(id).unwrap().target, None);
    assert_eq!(
        request(&mut w, id, 2, Some(target)).outcome,
        Outcome::Ignored
    );
    assert_eq!(
        request(&mut w, id, 3, Some(target)).outcome,
        Outcome::Rejected(Pause::InvalidTarget)
    );
    w.leave(id);
    let replacement = w.join().unwrap();
    assert_ne!(id, replacement);
    assert_eq!(w.autoattack_snapshot(replacement).unwrap().target, None);
    assert_eq!(
        request(&mut w, id, 4, Some(target)).outcome,
        Outcome::Ignored
    );
}
#[test]
fn target_death_clears_engagement_and_legacy_event_codecs_refuse_weapon_hits() {
    let (mut w, id, target) = fixture(TickRate::Hz60);
    request(&mut w, id, 1, Some(target));
    w.step();
    let event = w.combat_events()[0].clone();
    let batch = crate::combat_wire::Batch {
        epoch: w.epoch(),
        sequence: 1,
        tick: w.tick,
        events: vec![event],
    };
    assert!(batch.encode(w.combat_catalog().unwrap()).is_err());
    for _ in 0..1200 {
        w.step();
        if health(&w, target) == 0 {
            break;
        }
    }
    assert_eq!(health(&w, target), 0);
    assert_eq!(w.autoattack_snapshot(id).unwrap().target, None);
    assert_eq!(w.targeting_snapshot(id).unwrap().selected, None);
    assert!(matches!(
        w.combat_events(),
        [
            Event::MeleeDamage {
                amount: 10,
                remaining: 0,
                ..
            },
            Event::Died { .. }
        ]
    ));
}
#[test]
fn adoption_requires_inactive_classic_combat_and_binds_item_and_weapon_bytes() {
    let (mut w, _) = inactive(TickRate::Hz30);
    let baseline = w.identity();
    let combat_source = w
        .combat
        .as_ref()
        .unwrap()
        .identity(crate::targeting::profile_identity(
            crate::procedural::IDENTITY,
        ));
    let expected =
        crate::classic::profile_identity(profile_identity(combat_source, WEAPONS, ITEMS).unwrap());
    w.enable_autoattack(WEAPONS, ITEMS).unwrap();
    assert_ne!(w.identity(), baseline);
    assert_eq!(w.identity(), expected);
    assert!(w.enable_autoattack(WEAPONS, ITEMS).is_err());
    assert_ne!(
        profile_identity([7; 32], WEAPONS, ITEMS).unwrap(),
        profile_identity([7; 32], &[WEAPONS, b" "].concat(), ITEMS).unwrap()
    );
    let (mut late, _) = inactive(TickRate::Hz30);
    late.join().unwrap();
    assert!(late.enable_autoattack(WEAPONS, ITEMS).is_err());
    let mut legacy = World::with_procedural([8000., 8000.], 1.).unwrap();
    legacy.spawn_development_target([8002., 8000.]).unwrap();
    legacy.enable_combat(ABILITIES, 100, 5000).unwrap();
    assert!(legacy.enable_autoattack(WEAPONS, ITEMS).is_err());
}
#[test]
fn active_cast_pauses_basic_swings_but_gcd_does_not_own_the_weapon_clock() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = fixture(rate);
        let cast = w.cast(
            id,
            super::super::Request {
                epoch: w.epoch(),
                sequence: 1,
                ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
                target,
            },
        );
        assert!(matches!(
            cast.reply.outcome,
            super::super::Outcome::Started { .. }
        ));
        request(&mut w, id, 1, Some(target));
        for _ in 0..rate.hz() - 1 {
            w.step();
            assert_eq!(
                w.autoattack_snapshot(id).unwrap().phase,
                Phase::Paused(Pause::Casting)
            );
            assert_eq!(health(&w, target), 100);
        }
        w.step();
        assert_eq!(health(&w, target), 65);
        assert!(w.combat_snapshot(id).unwrap().gcd_until > w.tick);
        assert!(matches!(
            w.combat_events(),
            [
                Event::Damage { amount: 20, .. },
                Event::MeleeDamage { amount: 15, .. }
            ]
        ));
        assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 1);
    }
}
#[test]
fn committed_weapon_swap_resets_period_while_same_item_revision_preserves_readiness() {
    let (mut w, target) = inactive(TickRate::Hz30);
    let mut profile: serde_json::Value = serde_json::from_slice(WEAPONS).unwrap();
    let mut second = profile["weapons"][0].clone();
    second["item"] = serde_json::json!("theia.dev.training-wand");
    second["period_ms"] = serde_json::json!(1000);
    profile["weapons"].as_array_mut().unwrap().push(second);
    w.enable_autoattack(&serde_json::to_vec(&profile).unwrap(), ITEMS)
        .unwrap();
    let id = w.join().unwrap();
    ready(&mut w, id, 1);
    gear(&mut w, id, 1, Some("theia.dev.training-sword"));
    request(&mut w, id, 1, Some(target));
    w.step();
    let deadline = w.autoattack_snapshot(id).unwrap().ready_at;
    gear(&mut w, id, 2, Some("theia.dev.training-sword"));
    w.step();
    assert_eq!(w.autoattack_snapshot(id).unwrap().ready_at, deadline);
    gear(&mut w, id, 3, Some("theia.dev.training-wand"));
    w.step();
    assert_eq!(
        w.autoattack_snapshot(id).unwrap().ready_at,
        Some(w.tick + 30)
    );
    assert_eq!(health(&w, target), 85);
    for _ in 0..29 {
        w.step();
        assert_eq!(health(&w, target), 85);
    }
    w.step();
    assert_eq!(health(&w, target), 70);
}
#[test]
fn real_basic_kill_produces_bounded_reward_and_experience_facts_for_the_same_victim() {
    let (mut w, target) = crate::simulation::creatures::tests::fixture(TickRate::Hz30);
    w.enable_prediction().unwrap();
    w.enable_classic().unwrap();
    w.enable_combat(ABILITIES, 100, 5000).unwrap();
    let mut weapons: serde_json::Value = serde_json::from_slice(WEAPONS).unwrap();
    weapons["weapons"][0]["damage"] = serde_json::json!(100);
    w.enable_autoattack(&serde_json::to_vec(&weapons).unwrap(), ITEMS)
        .unwrap();
    let items = crate::renet_runtime::inventory::Profile::from_json(ITEMS).unwrap();
    w.enable_rewards(Arc::new(
        crate::reward_wire::Profile::from_json(
            include_bytes!("../../../../../../content/gameplay/development-rewards.json"),
            &items,
        )
        .unwrap(),
    ))
    .unwrap();
    w.enable_progression(Arc::new(
        crate::progression_wire::Profile::from_json(include_bytes!(
            "../../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap(),
    ))
    .unwrap();
    let p = w
        .procedural
        .as_ref()
        .unwrap()
        .patch(GlobalPosition::new([8007., 0., 8000.]).unwrap())
        .unwrap()
        .try_spawn(8007., 8000.)
        .unwrap();
    let mut p = p.metres();
    p[1] += f64::from(crate::world::PAWN_Y);
    let id = w.join_restored(GlobalPosition::new(p).unwrap()).unwrap();
    ready(&mut w, id, 1);
    gear(&mut w, id, 1, Some("theia.dev.training-sword"));
    turn(&mut w, id, 49152);
    request(&mut w, id, 1, Some(target));
    w.step();
    assert_eq!(health(&w, target), 0);
    let events = w.combat_events().to_vec();
    let rewards = w.reward_opportunities(&events).unwrap();
    let xp = w.experience_opportunities(&events).unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(xp.len(), 1);
    assert_eq!(rewards[0].actor, w.entity_key(id).unwrap());
    assert_eq!(xp[0].event.id.victim(), target);
    let fatal = events
        .iter()
        .find(|e| matches!(e, Event::MeleeDamage { remaining: 0, .. }))
        .unwrap()
        .clone();
    assert_eq!(
        w.reward_opportunities(&[fatal.clone(), fatal.clone()])
            .unwrap(),
        rewards
    );
    assert_eq!(
        w.experience_opportunities(&[fatal.clone(), fatal]).unwrap(),
        xp
    );
    // Facts only: persistence must still commit awards before acknowledgement.
}
#[test]
fn foreign_request_context_cannot_poison_live_sequences_or_allocate_unknown_owners() {
    let (mut w, id, target) = fixture(TickRate::Hz30);
    let c = w.correction(id).unwrap().state;
    let valid = Request {
        epoch: w.epoch(),
        stream_epoch: c.input.stream_epoch,
        sequence: 1,
        target: Some(target),
    };
    let actors = w.combat.as_ref().unwrap().actors.len();
    assert_eq!(
        w.request_autoattack(u64::MAX, valid).outcome,
        Outcome::Ignored
    );
    for request in [
        Request {
            epoch: if valid.epoch == 1 { 2 } else { 1 },
            sequence: u64::MAX,
            ..valid
        },
        Request {
            stream_epoch: c.input.stream_epoch + 1,
            sequence: u64::MAX,
            ..valid
        },
    ] {
        assert_eq!(w.request_autoattack(id, request).outcome, Outcome::Ignored);
    }
    assert_eq!(w.combat.as_ref().unwrap().actors.len(), actors);
    assert!(w.autoattack_snapshot(id).unwrap().reply.is_none());
    assert_eq!(w.request_autoattack(id, valid).outcome, Outcome::Engaged);
    w.step();
    assert_eq!(health(&w, target), 85);
}
#[test]
fn full_event_budget_pauses_without_damage_or_dropped_events_and_resumes_once() {
    let (mut w, id, target) = fixture(TickRate::Hz30);
    request(&mut w, id, 1, Some(target));
    let mut authority = w.combat.take().unwrap();
    let placeholder = Event::Died {
        target,
        position: authority.creatures[0].position,
        respawn_at: 100,
    };
    authority.events = vec![placeholder; authority.step_event_budget() - 1];
    let before = authority.events.clone();
    authority.advance_autoattack(&mut w);
    assert_eq!(authority.events, before);
    assert_eq!(authority.creatures[0].current, 100);
    assert_eq!(
        authority.actors[&id].autoattack.phase,
        Phase::Paused(Pause::EventBudget)
    );
    authority.actors.get_mut(&id).unwrap().autoattack.swings = u64::MAX;
    authority.events.clear();
    authority.advance_autoattack(&mut w);
    assert!(authority.events.is_empty());
    assert_eq!(authority.creatures[0].current, 100);
    assert_eq!(
        authority.actors[&id].autoattack.phase,
        Phase::Paused(Pause::TimeExhausted)
    );
    // Restore the trusted fixture counter; the retained engagement emits one hit.
    authority.actors.get_mut(&id).unwrap().autoattack.swings = 0;
    w.combat = Some(authority);
    w.step();
    assert_eq!(health(&w, target), 85);
    assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 1);
}

#[path = "ability_rules_tests.rs"]
mod ability_rules;
