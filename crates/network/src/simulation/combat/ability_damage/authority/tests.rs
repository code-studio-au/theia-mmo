use super::*;
use crate::simulation::combat::{Outcome as CastOutcome, Request};
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
const STARTING: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-ability-starting.json");
const POLICY: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-ability-damage.json");
const ABILITIES: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-classic-abilities.json");
fn fixture(rate: TickRate) -> (World, EntityKey) {
    let (mut w, npc) = inactive(rate, Some(DATA));
    w.enable_ability_damage(STARTING, POLICY, ABILITIES)
        .unwrap();
    w.combat
        .as_mut()
        .unwrap()
        .ability_damage
        .as_mut()
        .unwrap()
        .random = outcomes::Random::seed_for_test([41; 32]);
    (w, npc)
}
fn facts(w: &World) -> Vec<Fact> {
    w.combat_events()
        .iter()
        .filter_map(|e| {
            if let Event::AbilityDamage(f) = e {
                Some(f.clone())
            } else {
                None
            }
        })
        .collect()
}
fn cast(
    w: &mut World,
    id: u64,
    npc: EntityKey,
    sequence: u64,
    name: &str,
) -> crate::simulation::combat::CastResult {
    w.cast(
        id,
        Request {
            epoch: w.epoch(),
            sequence,
            ability: ContentId::new(name).unwrap(),
            target: npc,
        },
    )
}
fn hit(w: &World, id: u64, npc: EntityKey) -> Hit {
    Hit {
        credit_encounter: w.creature_credit_encounter(),
        actor: w.entity_key(id).unwrap(),
        target: npc,
        sequence: 1,
        ability: ContentId::new("theia.dev.sword-slash").unwrap(),
        amount: 999999,
    }
}
#[test]
fn explicit_activation_binds_actual_ability_bytes_and_is_atomic_before_admission() {
    let (mut w, _) = inactive(TickRate::Hz30, Some(DATA));
    let before = w.identity();
    assert!(w.enable_ability_damage(STARTING, b"{}", ABILITIES).is_err());
    assert_eq!(w.identity(), before);
    assert!(w.ability_damage_profile().is_none());
    let mut a = ABILITIES.to_vec();
    a.push(b' ');
    assert!(w.enable_ability_damage(STARTING, POLICY, &a).is_err());
    assert_eq!(w.identity(), before);
    w.enable_ability_damage(STARTING, POLICY, ABILITIES)
        .unwrap();
    assert_ne!(w.identity(), before);
    let after = w.identity();
    assert!(
        w.enable_ability_damage(STARTING, POLICY, ABILITIES)
            .is_err()
    );
    assert_eq!(w.identity(), after);
    let id = join(&mut w);
    assert!(w.ability_damage_profile().is_some());
    assert!(
        w.enable_ability_damage(STARTING, POLICY, ABILITIES)
            .is_err()
    );
    assert_eq!(w.entity_key(id).unwrap().epoch.value(), w.epoch());
    let (mut old, _) = inactive(TickRate::Hz30, None);
    assert!(
        old.enable_ability_damage(STARTING, POLICY, ABILITIES)
            .is_err()
    );
}
#[test]
fn instant_specials_use_typed_authoritative_damage_at_each_rate_and_preserve_white_swings() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = fixture(rate);
        let id = join(&mut w);
        let start = cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
        assert!(matches!(start.reply.outcome, CastOutcome::Started { .. }));
        assert!(facts(&w).is_empty());
        w.step();
        let f = facts(&w).pop().unwrap();
        assert_eq!(f.ability.as_str(), "theia.dev.sword-slash");
        assert_eq!(
            f.actor_generation,
            w.combat.as_ref().unwrap().actors[&id]
                .life
                .as_ref()
                .unwrap()
                .state
                .key()
                .generation
        );
        assert_eq!(1000 - f.remaining, f.result.damage.health_lost);
        assert!(f.result.damage.resolved_damage < 100);
        let metrics = w.ability_damage_metrics().unwrap();
        assert_eq!(
            (
                metrics.base_draws,
                metrics.landing_draws,
                metrics.special.attempts
            ),
            (1, 1, 1)
        );
        assert_eq!(w.combat_outcome_metrics().unwrap().random_draws, 0);
        engage(&mut w, id, Some(npc), 1);
        for _ in 0..rate.hz() * 3 {
            w.step();
            if w.white_melee_metrics().unwrap().outgoing.attempts > 0 {
                break;
            }
        }
        assert!(w.white_melee_metrics().unwrap().outgoing.attempts > 0);
        assert_eq!(w.ability_damage_metrics().unwrap().special.attempts, 1);
    }
}
#[test]
fn arcane_draws_occur_only_on_projectile_impact_after_cast_and_survive_equipment_swap() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, npc) = fixture(rate);
        let id = join(&mut w);
        gear(&mut w, id, 2, "theia.dev.training-wand");
        let reply = cast(&mut w, id, npc, 1, "theia.dev.arcane-bolt");
        assert!(matches!(reply.reply.outcome, CastOutcome::Started { .. }));
        let mut launched = false;
        let mut impacted = false;
        for _ in 0..rate.hz() * 3 {
            w.step();
            if w.combat_events()
                .iter()
                .any(|e| matches!(e, Event::ProjectileLaunched { .. }))
            {
                launched = true;
                assert_eq!(w.ability_damage_metrics().unwrap().base_draws, 0);
                gear(&mut w, id, 3, "theia.dev.training-sword");
            }
            if let Some(f) = facts(&w).pop() {
                assert!(launched);
                assert!(matches!(f.result.outcome, Outcome::Arcane(_)));
                assert_eq!(f.result.damage.armour_prevented, 0);
                assert_eq!(1000 - f.remaining, f.result.damage.health_lost);
                impacted = true;
                break;
            }
        }
        assert!(impacted);
        assert_eq!(w.ability_damage_metrics().unwrap().arcane.attempts, 1);
    }
}
#[test]
fn invalid_facing_equipment_and_interrupted_casts_consume_no_ability_rolls() {
    let (mut w, npc) = fixture(TickRate::Hz30);
    let id = join(&mut w);
    gear(&mut w, id, 2, "theia.dev.training-wand");
    assert!(matches!(
        cast(&mut w, id, npc, 1, "theia.dev.sword-slash")
            .reply
            .outcome,
        CastOutcome::Rejected(Reason::EquipmentRequired)
    ));
    turn(&mut w, id, 32768);
    assert!(matches!(
        cast(&mut w, id, npc, 2, "theia.dev.arcane-bolt")
            .reply
            .outcome,
        CastOutcome::Rejected(Reason::NotFacing)
    ));
    turn(&mut w, id, 0);
    assert!(matches!(
        cast(&mut w, id, npc, 3, "theia.dev.arcane-bolt")
            .reply
            .outcome,
        CastOutcome::Started { .. }
    ));
    w.combat.as_mut().unwrap().interrupt(id);
    for _ in 0..30 {
        w.step();
    }
    assert_eq!(w.ability_damage_metrics().unwrap().base_draws, 0);
    assert!(facts(&w).is_empty());
}
#[test]
fn preview_failure_capacity_and_deadline_checks_leave_damage_and_rng_uncommitted() {
    let (mut w, npc) = fixture(TickRate::Hz30);
    let id = join(&mut w);
    let h = hit(&w, id, npc);
    let mut a = w.combat.take().unwrap();
    let hp = a.creatures[0].current;
    let marker = Event::Cancelled {
        actor: h.actor,
        sequence: 1,
        reason: Reason::Unavailable,
    };
    a.events = vec![marker; a.step_event_budget()];
    assert!(a.apply_ability_damage(&mut w, &h).is_err());
    assert_eq!(a.creatures[0].current, hp);
    assert_eq!(a.ability_damage.as_ref().unwrap().random.base_draws(), 0);
    a.events.clear();
    let state = a.ordinary_creature_state(&w, npc, h.actor).unwrap();
    let runtime = a.ability_damage.as_ref().unwrap();
    let p = runtime.preview(&h.ability, state, hp).unwrap();
    let p2 = runtime.preview(&h.ability, state, hp).unwrap();
    assert_eq!(p.draws, p2.draws);
    assert_eq!(runtime.random.base_draws(), 0);
    a.creatures[0].current = 1;
    w.tick = u64::MAX;
    assert!(a.apply_ability_damage(&mut w, &h).is_err());
    assert_eq!(a.creatures[0].current, 1);
    assert_eq!(a.ability_damage.as_ref().unwrap().random.base_draws(), 0);
    assert!(a.events.is_empty());
}
#[test]
fn ordinary_and_older_codecs_refuse_candidate_ability_facts_instead_of_downgrading() {
    let (mut w, npc) = fixture(TickRate::Hz30);
    let id = join(&mut w);
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    let batch = crate::combat_wire::Batch {
        epoch: w.epoch(),
        sequence: 1,
        tick: w.tick,
        events: vec![Event::AbilityDamage(facts(&w).pop().unwrap())],
    };
    let c = &w.combat.as_ref().unwrap().catalog;
    let m = w.autoattack_catalog().unwrap();
    assert!(batch.encode(c).is_err());
    assert!(batch.encode_melee(c, m).is_err());
    assert!(
        batch
            .encode_physical(c, m, w.physical_damage_profile().unwrap())
            .is_err()
    );
    assert!(
        batch
            .encode_white(c, m, w.white_melee_profile().unwrap())
            .is_err()
    );
}
#[test]
fn departed_projectile_sources_end_without_drawing_or_damaging_the_creature() {
    let (mut w, npc) = fixture(TickRate::Hz30);
    let id = join(&mut w);
    gear(&mut w, id, 2, "theia.dev.training-wand");
    cast(&mut w, id, npc, 1, "theia.dev.arcane-bolt");
    let mut launched = false;
    for _ in 0..30 {
        w.step();
        if w.combat_events()
            .iter()
            .any(|e| matches!(e, Event::ProjectileLaunched { .. }))
        {
            launched = true;
            break;
        }
    }
    assert!(launched);
    assert_eq!(w.ability_damage_metrics().unwrap().base_draws, 0);
    w.leave(id);
    let mut ended = false;
    for _ in 0..30 {
        w.step();
        ended |= w.combat_events().iter().any(|e| {
            matches!(
                e,
                Event::ProjectileEnded {
                    reason: crate::simulation::combat::ProjectileEnd::SourceGone,
                    ..
                }
            )
        });
        assert!(facts(&w).is_empty());
    }
    assert!(ended);
    assert_eq!(w.ability_damage_metrics().unwrap().base_draws, 0);
    assert_eq!(w.combat.as_ref().unwrap().creatures[0].current, 1000);
}
#[test]
fn fatal_special_facts_feed_once_only_xp_loot_and_do_not_repeat_after_creature_respawn() {
    let (mut w, npc) = inactive(TickRate::Hz30, Some(DATA));
    let mut p: Value = serde_json::from_slice(POLICY).unwrap();
    p["abilities"][0]["miss_bp"] = json!(0);
    p["abilities"][0]["dodge_bp"] = json!(0);
    p["abilities"][0]["critical_bp"] = json!(10000);
    w.enable_ability_damage(STARTING, &serde_json::to_vec(&p).unwrap(), ABILITIES)
        .unwrap();
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
    cast(&mut w, id, npc, 1, "theia.dev.sword-slash");
    w.step();
    let f = facts(&w).pop().unwrap();
    assert_eq!(f.result.damage.health_lost, 1);
    assert_eq!(f.remaining, 0);
    assert!(f.result.damage.overkill > 0);
    let events = vec![Event::AbilityDamage(f.clone()), Event::AbilityDamage(f)];
    assert_eq!(w.experience_opportunities(&events).unwrap().len(), 1);
    assert_eq!(w.reward_opportunities(&events).unwrap().len(), 1);
    let due = w.combat.as_ref().unwrap().creatures[0].respawn_at.unwrap();
    while w.tick < due {
        w.step();
        assert!(facts(&w).is_empty());
    }
    assert!(w.experience_opportunities(&events).unwrap().is_empty());
    assert!(w.reward_opportunities(&events).unwrap().is_empty());
    assert_eq!(w.ability_damage_metrics().unwrap().special.attempts, 1);
}
#[test]
fn source_identity_sequence_and_checked_counters_refuse_before_committing_draws() {
    let (mut w, npc) = fixture(TickRate::Hz30);
    let id = join(&mut w);
    let mut h = hit(&w, id, npc);
    let mut a = w.combat.take().unwrap();
    h.sequence = 0;
    assert!(a.apply_ability_damage(&mut w, &h).is_err());
    h.sequence = 1;
    a.ability_damage.as_mut().unwrap().metrics.landing_draws = u64::MAX;
    assert!(a.apply_ability_damage(&mut w, &h).is_err());
    assert_eq!(a.creatures[0].current, 1000);
    assert_eq!(a.ability_damage.as_ref().unwrap().random.base_draws(), 0);
    assert!(a.events.is_empty());
    a.ability_damage.as_mut().unwrap().metrics.landing_draws = 0;
    w.leave(id);
    assert!(a.apply_ability_damage(&mut w, &h).is_err());
    assert_eq!(a.creatures[0].current, 1000);
    assert_eq!(a.ability_damage.as_ref().unwrap().random.base_draws(), 0);
}

mod credit;
mod membership;

mod recipient;

mod reset;
