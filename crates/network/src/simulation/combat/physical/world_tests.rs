use super::*;
use crate::{
    commands::Command,
    prediction_wire::Commands,
    simulation::combat::{
        Reason,
        autoattack::{self, Outcome, Pause, Phase},
    },
    wire::{ChunkReady, Intent, VERSION},
};
use theia_core::{facing::Facing, time::TickRate};
const DEFINITIONS: &[u8] = include_bytes!(
    "../../../../../../content/gameplay/development-physical-damage-definitions.json"
);
const DAMAGE_STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-physical-damage-stats.json");
const ITEMS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-equipment-items.json");
const WEAPONS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-melee-weapons.json");
const VITALS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-character-stats.json");
const CREATURE: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-creature-patrol.json");
fn inactive(rate: TickRate, modern: bool) -> (World, EntityKey) {
    let (mut w, npc) = creatures::tests::fixture(rate);
    w.enable_prediction().unwrap();
    w.enable_classic().unwrap();
    w.enable_status_combat(
        include_bytes!("../../../../../../content/gameplay/development-classic-abilities.json"),
        1000,
        5000,
        VITALS,
        include_bytes!("../../../../../../content/gameplay/development-statuses.json"),
    )
    .unwrap();
    let life = crate::player_life_wire::Profile::from_json(
        VITALS,
        CREATURE,
        include_bytes!("../../../../../../content/gameplay/development-creature-retaliation.json"),
        include_bytes!("../../../../../../content/gameplay/development-player-respawn.json"),
    )
    .unwrap();
    w.enable_player_life(Arc::new(life)).unwrap();
    w.enable_autoattack(WEAPONS, ITEMS).unwrap();
    if modern {
        w.enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
            .unwrap();
        w.combat.as_mut().unwrap().physical.as_mut().unwrap().random =
            outcomes::Random::seed_for_test([7; 32]);
    }
    (w, npc)
}
fn join(w: &mut World) -> u64 {
    let xz = [8008., 8002.];
    let position = GlobalPosition::new([xz[0], 0., xz[1]]).unwrap();
    let patch = w.procedural.as_ref().unwrap().patch(position).unwrap();
    let mut p = patch.try_spawn(xz[0], xz[1]).unwrap().metres();
    p[1] += f64::from(crate::world::PAWN_Y);
    let id = w.join_restored(GlobalPosition::new(p).unwrap()).unwrap();
    ready(w, id, 1, true);
    gear(w, id, 1, Some("theia.dev.training-sword"));
    id
}
fn ready(w: &mut World, id: u64, sequence: u64, present: bool) {
    let chunks = if present {
        w.procedural
            .as_ref()
            .unwrap()
            .patch(w.global_position(id).unwrap())
            .unwrap()
            .ids
            .iter()
            .map(|id| [id.x, id.z])
            .collect()
    } else {
        vec![]
    };
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
            item.map(|s| ContentId::new(s).unwrap()),
        )),
    );
}
fn engage(w: &mut World, id: u64, target: EntityKey, sequence: u64) -> autoattack::Reply {
    w.request_autoattack(
        id,
        autoattack::Request {
            epoch: w.epoch(),
            stream_epoch: w.correction(id).unwrap().state.input.stream_epoch,
            sequence,
            target: Some(target),
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
fn facts(w: &World) -> Vec<Fact> {
    w.combat_events()
        .iter()
        .filter_map(|e| {
            if let Event::PhysicalDamage(f) = e {
                Some(f.clone())
            } else {
                None
            }
        })
        .collect()
}
fn health(w: &World, key: EntityKey) -> u32 {
    w.combat
        .as_ref()
        .unwrap()
        .creatures
        .iter()
        .find(|c| c.key == key)
        .unwrap()
        .current
}
#[test]
fn world_binding_is_atomic_requires_inactive_known_roles_and_matches_identity_order() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, _) = inactive(rate, false);
        let before = w.identity();
        let prior = w
            .combat
            .as_ref()
            .unwrap()
            .identity(w.patrol_identity.unwrap());
        let mut wrong = ITEMS.to_vec();
        wrong.push(b' ');
        assert!(
            w.enable_physical_damage(DEFINITIONS, DAMAGE_STATS, &wrong)
                .is_err()
        );
        assert_eq!(w.identity(), before);
        assert!(w.physical_damage_profile().is_none());
        w.enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
            .unwrap();
        let profile = w.physical_damage_profile().unwrap();
        assert_ne!(w.identity(), before);
        assert_eq!(
            w.identity(),
            crate::classic::profile_identity(profile.identity(prior))
        );
        let after = w.identity();
        assert!(
            w.enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
                .is_err()
        );
        assert_eq!(w.identity(), after);
        let (mut late, _) = inactive(rate, false);
        join(&mut late);
        let before = late.identity();
        assert!(
            late.enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
                .is_err()
        );
        assert_eq!(late.identity(), before);
        let (mut late, _) = inactive(rate, false);
        late.step();
        assert!(
            late.enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
                .is_err()
        );
    }
}
#[test]
fn real_fixed_steps_resolve_both_directions_at_every_rate_without_legacy_melee_events() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        let owner = w.entity_key(id).unwrap();
        let profile = w.physical_damage_profile().unwrap().clone();
        assert_eq!(engage(&mut w, id, npc, 1).outcome, Outcome::Engaged);
        let mut seen = Vec::new();
        for _ in 0..6 * u64::from(rate.hz()) {
            w.step();
            assert!(
                !w.combat_events()
                    .iter()
                    .any(|e| matches!(e, Event::MeleeDamage { .. }))
            );
            seen.extend(facts(&w));
            if w.physical_damage_metrics().unwrap().incoming_hits >= 3 {
                break;
            }
        }
        let metrics = w.physical_damage_metrics().unwrap();
        assert!(metrics.outgoing_hits >= 2 && metrics.incoming_hits >= 3);
        assert_eq!(
            metrics.base_draws,
            metrics.outgoing_hits + metrics.incoming_hits
        );
        for fact in &seen {
            let before = fact.remaining + fact.result.health_lost;
            if fact.actor == owner {
                assert_eq!(fact.target, npc);
                assert!(fact.actor_generation.is_some());
                assert!(fact.target_generation.is_none());
                assert_eq!(
                    fact.result,
                    profile
                        .weapon_result(
                            &ContentId::new("theia.dev.training-sword").unwrap(),
                            fact.roll,
                            before
                        )
                        .unwrap()
                );
            } else {
                assert_eq!(fact.actor, npc);
                assert_eq!(fact.target, owner);
                assert!(fact.actor_generation.is_none());
                assert_eq!(fact.target_generation.unwrap().get(), 1);
                assert_eq!(
                    fact.result,
                    profile.creature_result(fact.roll, before).unwrap()
                );
            }
        }
        assert_eq!(1000 - health(&w, npc), metrics.outgoing_damage as u32);
        assert_eq!(
            100 - w.player_life_snapshot(id).unwrap().health,
            metrics.incoming_damage as u32
        );
        assert!(w.physical_damage_error().is_none());
        assert!(w.player_life_error().is_none());
    }
}
#[test]
fn facing_gear_and_readiness_rejections_never_roll_or_damage() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        turn(&mut w, id, 32768);
        engage(&mut w, id, npc, 1);
        w.step();
        assert_eq!(
            w.autoattack_snapshot(id).unwrap().phase,
            Phase::Paused(Pause::NotFacing)
        );
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, 0);
        assert_eq!(health(&w, npc), 1000);
        turn(&mut w, id, 0);
        assert_eq!(w.physical_damage_metrics().unwrap().outgoing_hits, 1);
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        w.observe_combat_equipment(w.entity_key(id).unwrap(), None);
        engage(&mut w, id, npc, 1);
        w.step();
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, 0);
        assert_eq!(health(&w, npc), 1000);
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        ready(&mut w, id, 2, false);
        engage(&mut w, id, npc, 1);
        w.step();
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, 0);
        assert_eq!(health(&w, npc), 1000);
    }
}
#[test]
fn replay_and_failed_preview_preserve_the_owned_random_stream() {
    let mut rolls = Vec::new();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        engage(&mut w, id, npc, 1);
        w.step();
        let before = w.physical_damage_metrics().unwrap();
        let old = facts(&w);
        assert_eq!(engage(&mut w, id, npc, 1).outcome, Outcome::Ignored);
        assert_eq!(
            w.physical_damage_metrics().unwrap().base_draws,
            before.base_draws
        );
        assert_eq!(facts(&w), old);
        rolls.push(old[0].roll);
        let runtime = w.combat.as_ref().unwrap().physical.as_ref().unwrap();
        let item = ContentId::new("theia.dev.training-sword").unwrap();
        let a = runtime.weapon(&item, 1000).unwrap();
        assert!(runtime.weapon(&item, 1001).is_err());
        let b = runtime.weapon(&item, 1000).unwrap();
        assert_eq!(a.roll, b.roll);
        assert_eq!(runtime.random.base_draws(), 1);
    }
    assert!(rolls.windows(2).all(|r| r[0] == r[1]));
}
#[test]
fn legacy_profiles_stay_fixed_and_legacy_event_codecs_refuse_even_small_new_results() {
    let (mut w, npc) = inactive(TickRate::Hz30, false);
    let id = join(&mut w);
    engage(&mut w, id, npc, 1);
    w.step();
    assert_eq!(health(&w, npc), 985);
    assert!(
        w.combat_events()
            .iter()
            .any(|e| matches!(e, Event::MeleeDamage { amount: 15, .. }))
    );
    assert!(w.physical_damage_metrics().is_none());
    let (mut w, npc) = inactive(TickRate::Hz30, true);
    let id = join(&mut w);
    engage(&mut w, id, npc, 1);
    w.step();
    let batch = crate::combat_wire::Batch {
        epoch: w.epoch(),
        sequence: 1,
        tick: w.tick,
        events: w.combat_events().to_vec(),
    };
    assert!(batch.encode(&w.combat.as_ref().unwrap().catalog).is_err());
    assert!(
        batch
            .encode_melee(
                &w.combat.as_ref().unwrap().catalog,
                w.autoattack_catalog().unwrap()
            )
            .is_err()
    );
}
#[test]
fn lethal_creature_hits_preserve_player_life_generation_and_do_not_attack_the_replacement() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        let key = w.player_life_snapshot(id).unwrap().key;
        let mut authority = w.combat.take().unwrap();
        authority.damage_player(&mut w, key, 99).unwrap();
        w.combat = Some(authority);
        engage(&mut w, id, npc, 1);
        let mut lethal = None;
        for _ in 0..2 * rate.hz() {
            w.step();
            lethal = facts(&w)
                .into_iter()
                .find(|f| f.target == key.actor && f.remaining == 0);
            if lethal.is_some() {
                break;
            }
        }
        let lethal = lethal.unwrap();
        assert_eq!(lethal.target_generation, Some(key.generation));
        assert_eq!(lethal.result.health_lost, 1);
        assert!(lethal.result.overkill > 0);
        let life = w.player_life_snapshot(id).unwrap();
        let respawn = life.respawn_at.unwrap();
        assert_eq!(life.health, 0);
        let before = w.physical_damage_metrics().unwrap().base_draws;
        while w.tick < respawn + u64::from(rate.hz()) {
            w.step();
        }
        let replacement = w.player_life_snapshot(id).unwrap();
        assert_eq!(replacement.key.generation.get(), 2);
        assert_eq!(replacement.health, 100);
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, before);
        assert_eq!(w.autoattack_snapshot(id).unwrap().phase, Phase::Idle);
    }
}
#[test]
fn outgoing_event_reservation_covers_pending_cast_cancellation_before_any_roll() {
    let (mut w, npc) = inactive(TickRate::Hz30, true);
    let id = join(&mut w);
    let caster = join(&mut w);
    gear(&mut w, caster, 2, Some("theia.dev.training-wand"));
    let reply = w.cast(
        caster,
        super::super::Request {
            epoch: w.epoch(),
            sequence: 1,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target: npc,
        },
    );
    assert!(matches!(
        reply.reply.outcome,
        super::super::Outcome::Started { .. }
    ));
    engage(&mut w, id, npc, 1);
    let mut authority = w.combat.take().unwrap();
    authority.creatures[0].current = 1;
    let placeholder = Event::Cancelled {
        actor: w.entity_key(id).unwrap(),
        sequence: 1,
        reason: Reason::NotReady,
    };
    authority.events = vec![placeholder; authority.step_event_budget() - 2];
    let original = authority.events.clone();
    assert_eq!(authority.physical_melee_event_slots(npc), 3);
    authority.advance_autoattack(&mut w);
    assert_eq!(authority.events, original);
    assert_eq!(authority.creatures[0].current, 1);
    assert_eq!(authority.physical.as_ref().unwrap().random.base_draws(), 0);
}
#[test]
fn failed_death_deadlines_and_stale_lives_do_not_commit_previewed_randomness_or_health() {
    let (mut w, npc) = inactive(TickRate::Hz30, true);
    let id = join(&mut w);
    let owner = w.entity_key(id).unwrap();
    let mut authority = w.combat.take().unwrap();
    authority.creatures[0].current = 1;
    w.tick = u64::MAX - 1;
    let weapon = authority
        .autoattack
        .as_ref()
        .unwrap()
        .catalog()
        .weapons()
        .next()
        .unwrap()
        .clone();
    assert!(
        authority
            .apply_physical_melee(&mut w, id, npc, &weapon, 1)
            .is_err()
    );
    assert_eq!(authority.creatures[0].current, 1);
    assert!(authority.events.is_empty());
    assert_eq!(authority.physical.as_ref().unwrap().random.base_draws(), 0);
    w.tick = 0;
    let key = authority.actors[&id].life.as_ref().unwrap().state.key();
    authority.damage_player(&mut w, key, 99).unwrap();
    w.tick = u64::MAX - 1;
    assert!(
        authority
            .apply_physical_retaliation(&mut w, npc, key, 5)
            .is_err()
    );
    assert_eq!(
        authority.actors[&id].life.as_ref().unwrap().state.health(),
        1
    );
    assert_eq!(authority.physical.as_ref().unwrap().random.base_draws(), 0);
    assert!(authority.events.is_empty());
    let stale = LifeKey {
        actor: owner,
        generation: NonZeroU64::new(2).unwrap(),
    };
    assert!(
        authority
            .apply_physical_retaliation(&mut w, npc, stale, 5)
            .unwrap()
            .is_none()
    );
    assert_eq!(authority.physical.as_ref().unwrap().random.base_draws(), 0);
}

#[test]
fn incoming_event_pressure_cancels_before_damage_or_random_commit() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        engage(&mut w, id, npc, 1);
        w.step();
        let due = w
            .player_life_snapshot(id)
            .unwrap()
            .windup
            .unwrap()
            .completes;
        while w.tick < due - 1 {
            w.step();
        }
        let mut authority = w.combat.take().unwrap();
        let placeholder = Event::Cancelled {
            actor: w.entity_key(id).unwrap(),
            sequence: 1,
            reason: Reason::NotReady,
        };
        authority.events = vec![placeholder; authority.step_event_budget() - 2];
        let before = authority.events.clone();
        w.tick = due;
        authority.advance_retaliation(&mut w);
        assert_eq!(authority.events, before);
        assert_eq!(
            authority.actors[&id].life.as_ref().unwrap().state.health(),
            100
        );
        let runtime = authority.physical.as_ref().unwrap();
        assert_eq!(runtime.metrics.base_draws, 1);
        assert_eq!(runtime.metrics.incoming_hits, 0);
        assert_eq!(runtime.npc_sequence, 0);
    }
}
#[test]
fn exhausted_npc_sequence_stops_before_any_new_roll_or_life_change() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        engage(&mut w, id, npc, 1);
        w.step();
        let due = w
            .player_life_snapshot(id)
            .unwrap()
            .windup
            .unwrap()
            .completes;
        while w.tick < due - 1 {
            w.step();
        }
        w.combat
            .as_mut()
            .unwrap()
            .physical
            .as_mut()
            .unwrap()
            .npc_sequence = u64::MAX;
        w.step();
        assert_eq!(w.player_life_snapshot(id).unwrap().health, 100);
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, 1);
        assert_eq!(w.physical_damage_metrics().unwrap().incoming_hits, 0);
        assert!(w.player_life_error().unwrap().contains("sequence"));
        assert!(facts(&w).is_empty());
    }
}
#[test]
fn creature_death_respawn_and_reengagement_use_the_new_entity_incarnation() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        w.combat.as_mut().unwrap().creatures[0].current = 1;
        engage(&mut w, id, npc, 1);
        w.step();
        let result = facts(&w).pop().unwrap();
        assert_eq!(result.target, npc);
        assert_eq!(result.result.health_lost, 1);
        assert!(result.result.overkill > 0);
        let due = w.combat.as_ref().unwrap().creatures[0].respawn_at.unwrap();
        let before = w.physical_damage_metrics().unwrap().base_draws;
        while w.tick < due {
            w.step();
        }
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, before);
        let replacement = w.patrol.as_ref().unwrap().key;
        assert_ne!(replacement, npc);
        assert_eq!(health(&w, replacement), 1000);
        assert_eq!(engage(&mut w, id, replacement, 2).outcome, Outcome::Engaged);
        let mut new = None;
        for _ in 0..3 * rate.hz() {
            w.step();
            new = facts(&w).into_iter().find(|f| f.actor.entity.value() == id);
            if new.is_some() {
                break;
            }
        }
        assert_eq!(new.unwrap().target, replacement);
        let before = w.physical_damage_metrics().unwrap().base_draws;
        assert_eq!(
            engage(&mut w, id, npc, 3).outcome,
            Outcome::Rejected(Pause::InvalidTarget)
        );
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, before);
    }
}

#[test]
fn committed_weapon_changes_pause_and_reset_before_another_physical_hit() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
        let id = join(&mut w);
        engage(&mut w, id, npc, 1);
        w.step();
        assert_eq!(w.physical_damage_metrics().unwrap().outgoing_hits, 1);
        gear(&mut w, id, 2, Some("theia.dev.training-wand"));
        w.step();
        assert_eq!(
            w.autoattack_snapshot(id).unwrap().phase,
            Phase::Paused(Pause::NoWeapon)
        );
        gear(&mut w, id, 3, Some("theia.dev.training-sword"));
        let due = w.autoattack_snapshot(id).unwrap().ready_at.unwrap();
        assert!(due > w.tick + 1);
        while w.tick < due - 1 {
            w.step();
            assert_eq!(w.physical_damage_metrics().unwrap().outgoing_hits, 1);
        }
        w.step();
        assert_eq!(w.physical_damage_metrics().unwrap().outgoing_hits, 2);
        let result = facts(&w)
            .into_iter()
            .find(|f| f.actor.entity.value() == id)
            .unwrap();
        assert_eq!(result.attack.as_str(), "theia.dev.training-sword-swing");
        assert!(w.physical_damage_error().is_none());
    }
}

#[test]
fn physical_fatal_hits_feed_existing_xp_and_reward_boundaries_once_per_victim() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, true);
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
        let id = join(&mut w);
        w.combat.as_mut().unwrap().creatures[0].current = 1;
        engage(&mut w, id, npc, 1);
        w.step();
        let fatal = facts(&w).pop().unwrap();
        let events = vec![
            Event::PhysicalDamage(fatal.clone()),
            Event::PhysicalDamage(fatal.clone()),
        ];
        let xp = w.experience_opportunities(&events).unwrap();
        let loot = w.reward_opportunities(&events).unwrap();
        assert_eq!(xp.len(), 1);
        assert_eq!(loot.len(), 1);
        assert_eq!(xp[0].actor, w.entity_key(id).unwrap());
        assert_eq!(loot[0].actor, xp[0].actor);
        let mut incoming = fatal.clone();
        incoming.attack = w
            .physical_damage_profile()
            .unwrap()
            .creature_attack()
            .0
            .clone();
        incoming.actor = npc;
        incoming.target = w.entity_key(id).unwrap();
        incoming.actor_generation = None;
        incoming.target_generation = NonZeroU64::new(1);
        incoming.roll = 5;
        incoming.result = w
            .physical_damage_profile()
            .unwrap()
            .creature_result(5, 1)
            .unwrap();
        let incoming = vec![Event::PhysicalDamage(incoming)];
        assert!(w.experience_opportunities(&incoming).unwrap().is_empty());
        assert!(w.reward_opportunities(&incoming).unwrap().is_empty());
        let due = w.combat.as_ref().unwrap().creatures[0].respawn_at.unwrap();
        while w.tick < due {
            w.step();
        }
        assert!(w.experience_opportunities(&events).unwrap().is_empty());
        assert!(w.reward_opportunities(&events).unwrap().is_empty());
    }
}
