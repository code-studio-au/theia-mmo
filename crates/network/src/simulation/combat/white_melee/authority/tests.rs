use super::*;
use crate::{
    commands::Command,
    prediction_wire::Commands,
    simulation::combat::autoattack,
    simulation::creatures,
    wire::{ChunkReady, Intent, VERSION},
};
use serde_json::{Value, json};
use theia_core::{facing::Facing, time::TickRate};
const DATA: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-white-melee.json");
const ITEMS: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-equipment-items.json");
const WEAPONS: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-melee-weapons.json");
const VITALS: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-character-stats.json");
fn data(edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut r = serde_json::from_slice(DATA).unwrap();
    edit(&mut r);
    serde_json::to_vec(&r).unwrap()
}
fn inactive(rate: TickRate, profile: Option<&[u8]>) -> (World, EntityKey) {
    inactive_stats(
        rate,
        profile,
        include_bytes!(
            "../../../../../../../content/gameplay/development-physical-damage-stats.json"
        ),
    )
}
fn inactive_stats(rate: TickRate, profile: Option<&[u8]>, stats: &[u8]) -> (World, EntityKey) {
    let mut creature: Value = serde_json::from_slice(include_bytes!(
        "../../../../../../../content/gameplay/development-creature-patrol.json"
    ))
    .unwrap();
    creature["speed_metres_per_second"] = json!(0.1);
    creature["aggro"] = json!({"perception_metres":1.0,"leash_metres":35.0,"chase_speed":5.0,"return_speed":5.0,"los_grace_ms":2000,"engagement_ms":30000,"reacquire_ms":2000});
    let creature = serde_json::to_vec(&creature).unwrap();
    let (mut w, npc) = creatures::tests::fixture_profile_at(rate, [8000., 8000.], &creature);
    w.enable_prediction().unwrap();
    w.enable_classic().unwrap();
    w.enable_status_combat(
        include_bytes!("../../../../../../../content/gameplay/development-classic-abilities.json"),
        1000,
        5000,
        VITALS,
        include_bytes!("../../../../../../../content/gameplay/development-statuses.json"),
    )
    .unwrap();
    w.enable_player_life(Arc::new(
        crate::player_life_wire::Profile::from_json(
            VITALS,
            &creature,
            include_bytes!(
                "../../../../../../../content/gameplay/development-creature-retaliation.json"
            ),
            include_bytes!("../../../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    ))
    .unwrap();
    w.enable_autoattack(WEAPONS, ITEMS).unwrap();
    w.enable_physical_damage(
        include_bytes!(
            "../../../../../../../content/gameplay/development-physical-damage-definitions.json"
        ),
        stats,
        ITEMS,
    )
    .unwrap();
    if let Some(profile) = profile {
        w.enable_white_melee(profile).unwrap();
        w.combat
            .as_mut()
            .unwrap()
            .white_melee
            .as_mut()
            .unwrap()
            .random = outcomes::Random::seed_for_test([19; 32]);
    }
    (w, npc)
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
fn gear(w: &mut World, id: u64, revision: u64, item: &str) {
    w.observe_combat_equipment(
        w.entity_key(id).unwrap(),
        Some((
            crate::terrain::sha256(ITEMS),
            revision,
            Some(ContentId::new(item).unwrap()),
        )),
    );
}
fn join(w: &mut World) -> u64 {
    let point = GlobalPosition::new([8008., 0., 8002.]).unwrap();
    let patch = w.procedural.as_ref().unwrap().patch(point).unwrap();
    let mut p = patch.try_spawn(8008., 8002.).unwrap().metres();
    p[1] += f64::from(crate::world::PAWN_Y);
    let id = w.join_restored(GlobalPosition::new(p).unwrap()).unwrap();
    ready(w, id, 1, true);
    gear(w, id, 1, "theia.dev.training-sword");
    id
}
fn engage(w: &mut World, id: u64, target: Option<EntityKey>, sequence: u64) -> autoattack::Reply {
    w.request_autoattack(
        id,
        autoattack::Request {
            epoch: w.epoch(),
            stream_epoch: w.correction(id).unwrap().state.input.stream_epoch,
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
                        ..Default::default()
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
        .filter_map(|event| {
            if let Event::WhiteMelee(f) = event {
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
fn weapon(authority: &Authority) -> melee::Weapon {
    authority
        .autoattack
        .as_ref()
        .unwrap()
        .catalog()
        .weapons()
        .next()
        .unwrap()
        .clone()
}
fn critical_data() -> Vec<u8> {
    data(|r| {
        r["player"]["weapons"][0]["hit_bonus_bp"] = json!(10000);
        r["player"]["critical_bp"] = json!(10000);
        r["creature"]["defences"]["dodge_bp"] = json!(0);
    })
}
#[test]
fn activation_is_atomic_before_admission_and_wraps_the_physical_identity() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, _) = inactive(rate, None);
        let before = w.identity();
        assert!(w.enable_white_melee(b"{}").is_err());
        assert_eq!(w.identity(), before);
        assert!(w.white_melee_profile().is_none());
        let prior = w
            .combat
            .as_ref()
            .unwrap()
            .identity(w.patrol_identity.unwrap());
        w.enable_white_melee(DATA).unwrap();
        assert_eq!(
            w.identity(),
            crate::classic::profile_identity(w.white_melee_profile().unwrap().identity(prior))
        );
        let after = w.identity();
        assert_ne!(before, after);
        assert!(w.enable_white_melee(DATA).is_err());
        assert_eq!(w.identity(), after);
        let (mut late, _) = inactive(rate, None);
        join(&mut late);
        assert!(late.enable_white_melee(DATA).is_err());
        let (mut late, _) = inactive(rate, None);
        late.step();
        assert!(late.enable_white_melee(DATA).is_err());
    }
}
#[test]
fn real_fixed_steps_resolve_both_directions_without_legacy_damage_events_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, Some(DATA));
        let id = join(&mut w);
        let owner = w.entity_key(id).unwrap();
        let profile = w.white_melee_profile().unwrap().clone();
        assert_eq!(
            engage(&mut w, id, Some(npc), 1).outcome,
            autoattack::Outcome::Engaged
        );
        let mut seen = Vec::new();
        for _ in 0..rate.hz() * 6 {
            w.step();
            assert!(
                !w.combat_events()
                    .iter()
                    .any(|e| matches!(e, Event::MeleeDamage { .. } | Event::PhysicalDamage(_)))
            );
            seen.extend(facts(&w));
            if w.white_melee_metrics().unwrap().incoming.attempts >= 3 {
                break;
            }
        }
        let m = w.white_melee_metrics().unwrap();
        assert!(m.incoming.attempts >= 3 && m.outgoing.attempts >= 2);
        assert_eq!(m.base_draws, m.incoming.attempts + m.outgoing.attempts);
        assert_eq!(m.outcome_draws, m.base_draws);
        assert_eq!(m.glance_draws, 0);
        for f in seen {
            let before = f.remaining + f.result.health_lost;
            if f.actor == owner {
                assert_eq!(f.target, npc);
                assert!(f.actor_generation.is_some() && f.target_generation.is_none());
                assert_eq!(
                    f.result,
                    profile
                        .evaluate_outgoing(
                            &ContentId::new("theia.dev.training-sword").unwrap(),
                            f.defender,
                            f.draws,
                            before
                        )
                        .unwrap()
                );
            } else {
                assert_eq!((f.actor, f.target), (npc, owner));
                assert!(f.actor_generation.is_none());
                assert_eq!(f.target_generation.unwrap().get(), 1);
                assert_eq!(
                    f.result,
                    profile
                        .evaluate_incoming(f.defender, f.draws, before)
                        .unwrap()
                );
            }
        }
        assert_eq!(1000 - health(&w, npc), m.outgoing.health_lost as u32);
        assert_eq!(
            100 - w.player_life_snapshot(id).unwrap().health,
            m.incoming.health_lost as u32
        );
        assert_eq!(w.physical_damage_metrics().unwrap().base_draws, 0);
        assert!(w.white_melee_error().is_none() && w.player_life_error().is_none());
    }
}
#[test]
fn a_valid_avoided_swing_provokes_outside_passive_perception_without_health_loss() {
    let profile = data(|r| {
        r["player"]["weapons"][0]["hit_bonus_bp"] = json!(10000);
        r["creature"]["defences"]["dodge_bp"] = json!(10000);
    });
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = inactive(rate, Some(&profile));
        let id = join(&mut w);
        assert_eq!(w.creature_pursuit_metrics().unwrap().acquisitions, 0);
        engage(&mut w, id, Some(npc), 1);
        w.step();
        let result = facts(&w).pop().unwrap();
        assert_eq!(result.result.outcome, Outcome::Dodge);
        assert_eq!(health(&w, npc), 1000);
        assert_eq!(result.result.health_lost, 0);
        assert_eq!(w.white_melee_metrics().unwrap().outgoing.dodges, 1);
        w.step();
        assert_eq!(w.creature_pursuit_metrics().unwrap().acquisitions, 1);
        assert_eq!(
            w.creature_pursuit_directive()
                .unwrap()
                .target
                .unwrap()
                .actor,
            w.entity_key(id).unwrap()
        );
    }
}
#[test]
fn facing_readiness_gear_and_replay_do_not_consume_an_extra_roll() {
    let (mut w, npc) = inactive(TickRate::Hz30, Some(DATA));
    let id = join(&mut w);
    turn(&mut w, id, 32768);
    engage(&mut w, id, Some(npc), 1);
    w.step();
    assert_eq!(
        w.autoattack_snapshot(id).unwrap().phase,
        autoattack::Phase::Paused(autoattack::Pause::NotFacing)
    );
    assert_eq!(w.white_melee_metrics().unwrap().base_draws, 0);
    engage(&mut w, id, None, 2);
    turn(&mut w, id, 0);
    ready(&mut w, id, 2, false);
    engage(&mut w, id, Some(npc), 3);
    w.step();
    assert_eq!(w.white_melee_metrics().unwrap().base_draws, 0);
    engage(&mut w, id, None, 4);
    ready(&mut w, id, 3, true);
    gear(&mut w, id, 2, "theia.dev.training-wand");
    engage(&mut w, id, Some(npc), 5);
    w.step();
    assert_eq!(w.white_melee_metrics().unwrap().base_draws, 0);
    gear(&mut w, id, 3, "theia.dev.training-sword");
    engage(&mut w, id, Some(npc), 6);
    for _ in 0..100 {
        w.step();
        if w.white_melee_metrics().unwrap().outgoing.attempts > 0 {
            break;
        }
    }
    let before = w.white_melee_metrics().unwrap().base_draws;
    assert!(
        before > 0,
        "phase={:?}, error={:?}, position={:?}",
        w.autoattack_snapshot(id).unwrap().phase,
        w.white_melee_error(),
        w.patrol_snapshot().unwrap().position
    );
    assert_eq!(
        engage(&mut w, id, Some(npc), 6).outcome,
        autoattack::Outcome::Ignored
    );
    assert_eq!(w.white_melee_metrics().unwrap().base_draws, before);
}
#[test]
fn event_pressure_rejects_both_directions_before_health_or_randomness_changes() {
    let (mut w, npc) = inactive(TickRate::Hz30, Some(DATA));
    let id = join(&mut w);
    let mut a = w.combat.take().unwrap();
    let weapon = weapon(&a);
    let life = a.actors[&id].life.as_ref().unwrap().state.key();
    let event = Event::Died {
        target: npc,
        position: a.creatures[0].position,
        respawn_at: 100,
    };
    a.events.resize(a.step_event_budget(), event);
    assert!(a.apply_white_melee(&mut w, id, npc, &weapon, 1).is_err());
    assert!(a.apply_white_retaliation(&mut w, npc, life).is_err());
    assert_eq!(a.creatures[0].current, 1000);
    assert_eq!(a.actors[&id].life.as_ref().unwrap().state.health(), 100);
    assert_eq!(a.white_melee.as_ref().unwrap().random.base_draws(), 0);
    assert_eq!(a.white_melee.as_ref().unwrap().random.draws(), 0);
}
#[test]
fn failed_death_and_provocation_deadlines_and_stale_lives_preserve_health_and_rng() {
    let profile = critical_data();
    let (mut w, npc) = inactive(TickRate::Hz30, Some(&profile));
    let id = join(&mut w);
    let mut a = w.combat.take().unwrap();
    let weapon = weapon(&a);
    let life = a.actors[&id].life.as_ref().unwrap().state.key();
    w.tick = u64::MAX - 1;
    a.creatures[0].current = 1;
    assert!(a.apply_white_melee(&mut w, id, npc, &weapon, 1).is_err());
    assert_eq!(a.creatures[0].current, 1);
    a.creatures[0].current = 1000;
    assert!(a.apply_white_melee(&mut w, id, npc, &weapon, 1).is_err());
    assert_eq!(a.creatures[0].current, 1000);
    assert_eq!(a.white_melee.as_ref().unwrap().random.base_draws(), 0);
    assert!(a.events.is_empty());
    w.tick = 0;
    a.damage_player(&mut w, life, 99).unwrap();
    let state = a.ordinary_player_state(&w, npc, life).unwrap();
    let runtime = a.white_melee.as_mut().unwrap();
    for seed in 0..=255 {
        runtime.random = outcomes::Random::seed_for_test([seed; 32]);
        if runtime.creature(state, 1).unwrap().result.health_lost > 0 {
            break;
        }
    }
    assert!(runtime.creature(state, 1).unwrap().result.health_lost > 0);
    w.tick = u64::MAX - 1;
    assert!(a.apply_white_retaliation(&mut w, npc, life).is_err());
    assert_eq!(a.actors[&id].life.as_ref().unwrap().state.health(), 1);
    assert_eq!(a.white_melee.as_ref().unwrap().random.base_draws(), 0);
    let stale = LifeKey {
        generation: NonZeroU64::new(2).unwrap(),
        ..life
    };
    assert!(
        a.apply_white_retaliation(&mut w, npc, stale)
            .unwrap()
            .is_none()
    );
    assert_eq!(a.white_melee.as_ref().unwrap().random.draws(), 0);
}
#[test]
fn failed_previews_and_exhausted_npc_sequence_do_not_advance_owned_randomness() {
    let (mut w, npc) = inactive(TickRate::Hz30, Some(DATA));
    let id = join(&mut w);
    let mut a = w.combat.take().unwrap();
    let life = a.actors[&id].life.as_ref().unwrap().state.key();
    let state = a.ordinary_creature_state(&w, npc, life.actor).unwrap();
    let item = ContentId::new("theia.dev.training-sword").unwrap();
    let runtime = a.white_melee.as_mut().unwrap();
    let first = runtime.weapon(&item, state, 1000).unwrap();
    assert!(runtime.weapon(&item, state, 1001).is_err());
    let second = runtime.weapon(&item, state, 1000).unwrap();
    assert_eq!(first.draws, second.draws);
    assert_eq!(runtime.random.base_draws(), 0);
    assert_eq!(runtime.random.draws(), 0);
    runtime.npc_sequence = u64::MAX;
    assert!(a.apply_white_retaliation(&mut w, npc, life).is_err());
    assert_eq!(a.actors[&id].life.as_ref().unwrap().state.health(), 100);
    assert_eq!(a.white_melee.as_ref().unwrap().random.base_draws(), 0);
}
#[test]
fn ordinary_fatal_facts_feed_existing_xp_loot_and_retire_with_the_creature_life() {
    let profile = critical_data();
    let (mut w, npc) = inactive(TickRate::Hz30, Some(&profile));
    let items = crate::renet_runtime::inventory::Profile::from_json(ITEMS).unwrap();
    w.enable_rewards(Arc::new(
        crate::reward_wire::Profile::from_json(
            include_bytes!("../../../../../../../content/gameplay/development-rewards.json"),
            &items,
        )
        .unwrap(),
    ))
    .unwrap();
    w.enable_progression(Arc::new(
        crate::progression_wire::Profile::from_json(include_bytes!(
            "../../../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap(),
    ))
    .unwrap();
    let id = join(&mut w);
    w.combat.as_mut().unwrap().creatures[0].current = 1;
    engage(&mut w, id, Some(npc), 1);
    w.step();
    let fact = facts(&w).pop().unwrap();
    assert_eq!(fact.result.outcome, Outcome::Critical);
    let events = vec![
        Event::WhiteMelee(fact.clone()),
        Event::WhiteMelee(fact.clone()),
    ];
    assert_eq!(w.experience_opportunities(&events).unwrap().len(), 1);
    assert_eq!(w.reward_opportunities(&events).unwrap().len(), 1);
    let mut incoming = fact;
    incoming.actor = npc;
    incoming.target = w.entity_key(id).unwrap();
    incoming.actor_generation = None;
    incoming.target_generation = NonZeroU64::new(1);
    assert!(
        w.experience_opportunities(&[Event::WhiteMelee(incoming)])
            .unwrap()
            .is_empty()
    );
    let due = w.combat.as_ref().unwrap().creatures[0].respawn_at.unwrap();
    while w.tick < due {
        w.step();
    }
    assert!(w.experience_opportunities(&events).unwrap().is_empty());
    assert!(w.reward_opportunities(&events).unwrap().is_empty());
    assert!(w.patrol_facing(npc).is_none());
}
#[test]
fn older_codecs_reject_ordinary_facts_instead_of_downgrading_to_normal_damage() {
    let (mut w, npc) = inactive(TickRate::Hz30, Some(DATA));
    let id = join(&mut w);
    engage(&mut w, id, Some(npc), 1);
    w.step();
    let event = Event::WhiteMelee(facts(&w).pop().unwrap());
    let batch = crate::combat_wire::Batch {
        epoch: w.epoch(),
        sequence: 1,
        tick: w.tick,
        events: vec![event],
    };
    let catalog = &w.combat.as_ref().unwrap().catalog;
    let weapons = w.autoattack_catalog().unwrap();
    assert!(batch.encode(catalog).is_err());
    assert!(batch.encode_melee(catalog, weapons).is_err());
    assert!(
        batch
            .encode_physical(catalog, weapons, w.physical_damage_profile().unwrap())
            .is_err()
    );
}

#[test]
fn reference_level_fixture_commits_conditional_glance_and_creature_crushing_draws() {
    let mut stats: Value = serde_json::from_slice(include_bytes!(
        "../../../../../../../content/gameplay/development-physical-damage-stats.json"
    ))
    .unwrap();
    stats["player"]["physical"]["level"] = json!(60);
    stats["creatures"][0]["physical"]["level"] = json!(63);
    let stats = serde_json::to_vec(&stats).unwrap();
    let profile = data(|r| {
        r["player"]["weapons"][0]["skill"] = json!(300);
        r["player"]["defense_skill"] = json!(300);
    });
    for incoming in [false, true] {
        let (mut w, npc) = inactive_stats(TickRate::Hz30, Some(&profile), &stats);
        let id = join(&mut w);
        let mut a = w.combat.take().unwrap();
        let life = a.actors[&id].life.as_ref().unwrap().state.key();
        let weapon = weapon(&a);
        let desired = if incoming {
            Outcome::Crushing
        } else {
            Outcome::Glancing
        };
        let state = if incoming {
            a.ordinary_player_state(&w, npc, life).unwrap()
        } else {
            a.ordinary_creature_state(&w, npc, life.actor).unwrap()
        };
        let runtime = a.white_melee.as_mut().unwrap();
        let mut found = false;
        // Test-only seeded contexts choose an eligible outcome. No production
        // seed control or outcome override is added to authority or commands.
        for seed in 0..=255 {
            runtime.random = outcomes::Random::seed_for_test([seed; 32]);
            let preview = if incoming {
                runtime.creature(state, 100).unwrap()
            } else {
                runtime.weapon(weapon.item(), state, 1000).unwrap()
            };
            if preview.result.outcome == desired {
                found = true;
                break;
            }
        }
        assert!(found);
        assert_eq!(runtime.random.base_draws(), 0);
        if incoming {
            a.apply_white_retaliation(&mut w, npc, life)
                .unwrap()
                .unwrap();
        } else {
            a.apply_white_melee(&mut w, id, npc, &weapon, 1).unwrap();
        }
        let result = a
            .events
            .iter()
            .find_map(|e| {
                if let Event::WhiteMelee(f) = e {
                    Some(f)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(result.result.outcome, desired);
        assert_eq!(result.draws.glancing.is_some(), !incoming);
        let m = a.white_melee.as_ref().unwrap().metrics;
        assert_eq!(
            (m.base_draws, m.outcome_draws, m.glance_draws),
            (1, 1, u64::from(!incoming))
        );
    }
}
#[test]
fn defender_casting_state_comes_from_actual_cast_timing_and_committed_gear() {
    for (item, ability, casting) in [
        ("theia.dev.training-sword", "theia.dev.sword-slash", false),
        ("theia.dev.training-wand", "theia.dev.arcane-bolt", true),
    ] {
        let (mut w, npc) = inactive(TickRate::Hz30, Some(DATA));
        let id = join(&mut w);
        gear(&mut w, id, 2, item);
        let reply = w.cast(
            id,
            crate::simulation::combat::Request {
                epoch: w.epoch(),
                sequence: 1,
                target: npc,
                ability: ContentId::new(ability).unwrap(),
            },
        );
        assert!(matches!(
            reply.reply.outcome,
            crate::simulation::combat::Outcome::Started { .. }
        ));
        let a = w.combat.take().unwrap();
        let life = a.actors[&id].life.as_ref().unwrap().state.key();
        let state = a.ordinary_player_state(&w, npc, life).unwrap();
        assert!(state.in_front);
        assert_eq!(state.casting_non_melee, casting);
        assert_eq!(state.usable_melee_weapon, !casting);
        let distribution = a
            .white_melee
            .as_ref()
            .unwrap()
            .profile
            .incoming_distribution(state)
            .unwrap();
        assert_eq!(distribution.weights()[1], if casting { 0 } else { 500 });
    }
}
