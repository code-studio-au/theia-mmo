//! Transient health/deadline contracts only; no runtime pose/replication/PvE claim.
use serde_json::{Value, json};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::{Error, MAX_PROFILE_BYTES, Profile, State},
    resources, retaliation,
    time::TickRate,
};
const STATS: &[u8] = include_bytes!("../../../content/gameplay/development-character-stats.json");
const RESPAWN: &[u8] = include_bytes!("../../../content/gameplay/development-player-respawn.json");
fn profile() -> Profile {
    Profile::from_json(RESPAWN, &resources::Profile::from_json(STATS).unwrap()).unwrap()
}
fn actor(epoch: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(epoch).unwrap(),
        entity: EntityId::new(1).unwrap(),
    }
}
#[test]
fn respawn_recipe_checks_stats_reference_bounds_and_exact_structure() {
    let stats = resources::Profile::from_json(STATS).unwrap();
    let p = profile();
    assert_eq!(p.id().as_str(), "theia.dev.player-respawn");
    assert_eq!(p.stats(), &stats);
    let raw: Value = serde_json::from_slice(RESPAWN).unwrap();
    for (field, bad) in [
        ("schema", json!(2)),
        ("id", json!("../bad")),
        ("stats", json!("theia.other.stats")),
        ("delay_ms", json!(0)),
        ("delay_ms", json!(99)),
        ("delay_ms", json!(60001)),
        ("delay_ms", json!(100.5)),
        ("unknown", json!(true)),
    ] {
        let mut changed = raw.clone();
        changed[field] = bad;
        assert!(
            Profile::from_json(&serde_json::to_vec(&changed).unwrap(), &stats).is_err(),
            "{field}"
        );
    }
    for field in raw.as_object().unwrap().keys() {
        let extra = format!("{{\"{field}\":{},", raw[field]);
        let duplicate = String::from_utf8(RESPAWN.to_vec())
            .unwrap()
            .replacen('{', &extra, 1);
        assert!(Profile::from_json(duplicate.as_bytes(), &stats).is_err());
    }
    assert!(Profile::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1], &stats).is_err());
}
#[test]
fn fixed_rates_clip_overkill_and_schedule_exactly_one_death_and_respawn() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut s = State::new(profile(), rate, actor(1));
        let first = s.key();
        let partial = s.damage(0, s.key(), 25).unwrap();
        assert_eq!(
            (
                partial.applied,
                partial.remaining,
                partial.died,
                partial.respawn_at
            ),
            (25, 75, false, None)
        );
        let death = s.damage(1, s.key(), u32::MAX).unwrap();
        let due = 1 + 3 * u64::from(rate.hz());
        assert_eq!(
            (death.applied, death.remaining, death.died, death.respawn_at),
            (75, 0, true, Some(due))
        );
        assert!(!s.alive());
        let repeat = s.damage(2, s.key(), 100).unwrap();
        assert_eq!(
            (repeat.applied, repeat.died, repeat.respawn_at),
            (0, false, Some(due))
        );
        assert!(!s.advance(due - 1).unwrap());
        assert_eq!(s.health(), 0);
        assert_eq!(s.key(), first);
        assert!(s.advance(due).unwrap());
        assert!(s.alive());
        assert_eq!(s.health(), 100);
        assert_eq!(s.respawn_at(), None);
        assert_eq!(s.key().generation.get(), 2);
        assert!(!s.advance(due).unwrap());
        assert_eq!(s.key().generation.get(), 2);
    }
}
#[test]
fn elapsed_time_neither_regenerates_living_health_nor_implicitly_respawns_during_damage() {
    let mut s = State::new(profile(), TickRate::Hz30, actor(1));
    s.damage(0, s.key(), 20).unwrap();
    assert!(!s.advance(1000).unwrap());
    assert_eq!(s.health(), 80);
    s.damage(1000, s.key(), 80).unwrap();
    let repeat = s.damage(2000, s.key(), 0).unwrap();
    assert_eq!(
        (
            repeat.applied,
            repeat.died,
            repeat.remaining,
            repeat.respawn_at
        ),
        (0, false, 0, Some(1090))
    );
    assert!(s.advance(2000).unwrap());
    assert_eq!(s.health(), 100);
}
#[test]
fn respawned_player_life_refuses_a_pending_swing_even_when_runtime_actor_is_the_same() {
    let mut player = State::new(profile(), TickRate::Hz30, actor(1));
    let old = player.key();
    let recipe = retaliation::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-creature-retaliation.json"
    ))
    .unwrap();
    let mut npc = retaliation::State::new(recipe, TickRate::Hz30);
    npc.record_hit(0, old).unwrap();
    npc.advance(
        0,
        true,
        Some(retaliation::Target {
            key: old,
            alive: true,
            reachable: true,
        }),
    )
    .unwrap();
    player.damage(0, player.key(), 100).unwrap();
    player.advance(90).unwrap();
    let new = player.key();
    assert_ne!(old, new);
    assert_eq!(old.actor, new.actor);
    assert_ne!(
        new,
        theia_core::player_life::LifeKey {
            actor: actor(2),
            ..new
        }
    );
    // Even a creature not evaluated during the whole dead interval must cancel.
    assert_eq!(
        npc.advance(
            90,
            true,
            Some(retaliation::Target {
                key: new,
                alive: true,
                reachable: true
            })
        )
        .unwrap(),
        retaliation::Event::Cancelled { target: old }
    );
    assert_eq!(npc.target(), None);
    assert_eq!(player.health(), 100);
}
#[test]
fn stale_ticks_and_lethal_deadline_overflow_leave_health_and_all_metadata_unchanged() {
    let mut s = State::new(profile(), TickRate::Hz30, actor(1));
    let before = s.clone();
    assert_eq!(s.damage(u64::MAX, s.key(), 100), Err(Error::TimeExhausted));
    assert_eq!(s, before);
    s.damage(10, s.key(), 1).unwrap();
    let before = s.clone();
    assert_eq!(s.damage(9, s.key(), 1), Err(Error::StaleTick));
    assert_eq!(s, before);
    assert_eq!(s.advance(9), Err(Error::StaleTick));
    assert_eq!(s, before);
    assert_eq!(s.damage(u64::MAX, s.key(), 100), Err(Error::TimeExhausted));
    assert_eq!(s, before);
}
#[test]
fn a_late_step_restores_one_life_and_never_catches_up_additional_respawns() {
    let mut s = State::new(profile(), TickRate::Hz30, actor(1));
    s.damage(0, s.key(), 100).unwrap();
    assert!(s.advance(1000000).unwrap());
    let key = s.key();
    assert_eq!(key.generation.get(), 2);
    assert!(!s.advance(1000000).unwrap());
    assert!(!s.advance(2000000).unwrap());
    assert_eq!(s.key(), key);
}
#[test]
fn configured_health_is_bound_to_the_checked_stats_and_fresh_admission_starts_a_new_session_life() {
    let mut raw: Value = serde_json::from_slice(STATS).unwrap();
    raw["maximum_health"] = json!(250);
    let stats = resources::Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let p = Profile::from_json(RESPAWN, &stats).unwrap();
    let mut s = State::new(p.clone(), TickRate::Hz30, actor(1));
    assert_eq!((s.health(), s.maximum_health()), (250, 250));
    s.damage(0, s.key(), 200).unwrap();
    let fresh = State::new(p, TickRate::Hz30, actor(2));
    assert_eq!(fresh.health(), 250);
    assert_eq!(fresh.key().generation.get(), 1);
    assert_ne!(s.key(), fresh.key());
}

#[test]
fn foreign_or_previous_life_hits_cannot_damage_or_advance_the_current_life_clock() {
    let mut s = State::new(profile(), TickRate::Hz30, actor(1));
    let old = s.key();
    let before = s.clone();
    let foreign = theia_core::player_life::LifeKey {
        actor: actor(2),
        ..old
    };
    assert_eq!(s.damage(u64::MAX, foreign, 100), Err(Error::StaleLife));
    assert_eq!(s, before);
    s.damage(0, old, 100).unwrap();
    s.advance(90).unwrap();
    let before = s.clone();
    assert_eq!(s.damage(91, old, 100), Err(Error::StaleLife));
    assert_eq!(s, before);
    assert_eq!(s.damage(91, s.key(), 5).unwrap().remaining, 95);
}
