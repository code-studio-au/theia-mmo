//! Pure attack policy only: no health mutation, navigation, networking or PvE proof.
use serde_json::{Value, json};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    retaliation::{Error, Event, MAX_PROFILE_BYTES, Profile, State, Target},
    time::TickRate,
};
const DATA: &[u8] =
    include_bytes!("../../../content/gameplay/development-creature-retaliation.json");
fn key(epoch: u64, id: u64) -> LifeKey {
    LifeKey {
        actor: EntityKey {
            epoch: WorldEpoch::new(epoch).unwrap(),
            entity: EntityId::new(id).unwrap(),
        },
        generation: std::num::NonZeroU64::MIN,
    }
}
fn target(k: LifeKey, reachable: bool) -> Option<Target> {
    Some(Target {
        key: k,
        alive: true,
        reachable,
    })
}
fn state(rate: TickRate) -> State {
    State::new(Profile::from_json(DATA).unwrap(), rate)
}
#[test]
fn bounded_recipe_rejects_invalid_ids_fields_timing_and_duplicate_keys() {
    let profile = Profile::from_json(DATA).unwrap();
    assert_eq!(profile.id().as_str(), "theia.dev.training-retaliation");
    assert_eq!(profile.creature().as_str(), "theia.dev.training-creature");
    assert_eq!((profile.damage(), profile.reach_metres()), (5, 3.));
    let raw: Value = serde_json::from_slice(DATA).unwrap();
    for (field, bad) in [
        ("schema", json!(2)),
        ("id", json!("../invalid")),
        ("creature", json!("Invalid")),
        ("damage", json!(0)),
        ("damage", json!(1001)),
        ("reach_metres", json!(0.)),
        ("reach_metres", json!(5.1)),
        ("reach_metres", json!(null)),
        ("windup_ms", json!(0)),
        ("windup_ms", json!(5001)),
        ("recovery_ms", json!(0)),
        ("recovery_ms", json!(10001)),
        ("memory_ms", json!(1500)),
        ("memory_ms", json!(60001)),
        ("unknown", json!(true)),
    ] {
        let mut bad_raw = raw.clone();
        bad_raw[field] = bad;
        assert!(
            Profile::from_json(&serde_json::to_vec(&bad_raw).unwrap()).is_err(),
            "{field}"
        );
    }
    for field in raw.as_object().unwrap().keys() {
        let extra = format!("{{\"{field}\":{},", raw[field]);
        let duplicate = String::from_utf8(DATA.to_vec())
            .unwrap()
            .replacen('{', &extra, 1);
        assert!(
            Profile::from_json(duplicate.as_bytes()).is_err(),
            "duplicate {field}"
        );
    }
    assert!(Profile::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1]).is_err());
}
#[test]
fn fixed_rates_round_up_windup_and_recovery_without_early_or_duplicate_strikes() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut raw: Value = serde_json::from_slice(DATA).unwrap();
        raw["windup_ms"] = json!(333);
        raw["recovery_ms"] = json!(777);
        let mut s = State::new(
            Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap(),
            rate,
        );
        let k = key(1, 1);
        let windup = (333 * u64::from(rate.hz())).div_ceil(1000);
        let recovery = (777 * u64::from(rate.hz())).div_ceil(1000);
        assert!(s.record_hit(0, k).unwrap());
        assert_eq!(
            s.advance(0, true, target(k, true)).unwrap(),
            Event::Windup {
                target: k,
                completes: windup
            }
        );
        for tick in 1..windup {
            assert_eq!(s.advance(tick, true, target(k, true)).unwrap(), Event::None)
        }
        assert_eq!(
            s.advance(windup, true, target(k, true)).unwrap(),
            Event::Strike {
                target: k,
                amount: 5
            }
        );
        assert_eq!(
            s.advance(windup, true, target(k, true)).unwrap(),
            Event::None
        );
        for tick in windup + 1..windup + recovery {
            assert_eq!(s.advance(tick, true, target(k, true)).unwrap(), Event::None)
        }
        assert_eq!(
            s.advance(windup + recovery, true, target(k, true)).unwrap(),
            Event::Windup {
                target: k,
                completes: windup * 2 + recovery
            }
        );
    }
}
#[test]
fn same_attacker_refreshes_memory_without_restarting_windup_and_foreign_hits_do_not_steal() {
    let (a, b) = (key(1, 1), key(1, 2));
    let mut s = state(TickRate::Hz30);
    s.record_hit(0, a).unwrap();
    s.advance(0, true, target(a, true)).unwrap();
    assert!(!s.record_hit(1, b).unwrap());
    assert_eq!(s.target(), Some(a));
    assert_eq!(s.windup_until(), Some(15));
    assert!(s.record_hit(2, a).unwrap());
    assert_eq!(s.windup_until(), Some(15));
    assert_eq!(
        s.advance(15, true, target(a, true)).unwrap(),
        Event::Strike {
            target: a,
            amount: 5
        }
    );
    assert!(!s.record_hit(301, b).unwrap());
    assert_eq!(s.target(), Some(a));
    assert!(s.record_hit(302, b).unwrap());
    assert_eq!(s.target(), Some(b));
    assert_eq!(s.windup_until(), None);
}
#[test]
fn losing_reach_or_los_cancels_and_requires_recovery_before_another_windup() {
    let k = key(1, 1);
    let mut s = state(TickRate::Hz30);
    s.record_hit(0, k).unwrap();
    s.advance(0, true, target(k, true)).unwrap();
    assert_eq!(
        s.advance(15, true, target(k, false)).unwrap(),
        Event::Cancelled { target: k }
    );
    assert_eq!(s.target(), Some(k));
    assert_eq!(s.advance(16, true, target(k, true)).unwrap(), Event::None);
    assert_eq!(s.advance(59, true, target(k, true)).unwrap(), Event::None);
    assert_eq!(
        s.advance(60, true, target(k, true)).unwrap(),
        Event::Windup {
            target: k,
            completes: 75
        }
    );
}
#[test]
fn missing_dead_foreign_life_and_dead_creature_clear_the_engagement() {
    let k = key(1, 1);
    for (alive, observation) in [
        (true, None),
        (
            true,
            Some(Target {
                key: k,
                alive: false,
                reachable: true,
            }),
        ),
        (true, target(key(2, 1), true)),
        (
            true,
            target(
                LifeKey {
                    generation: std::num::NonZeroU64::new(2).unwrap(),
                    ..k
                },
                true,
            ),
        ),
        (true, target(key(1, 2), true)),
        (false, target(k, true)),
    ] {
        let mut s = state(TickRate::Hz30);
        s.record_hit(0, k).unwrap();
        s.advance(0, true, target(k, true)).unwrap();
        assert_eq!(
            s.advance(15, alive, observation).unwrap(),
            Event::Cancelled { target: k }
        );
        assert_eq!(s.target(), None);
        assert_eq!(s.advance(16, true, target(k, true)).unwrap(), Event::None);
    }
}
#[test]
fn late_steps_emit_one_strike_and_expired_memory_never_releases_an_old_swing() {
    let k = key(1, 1);
    let mut s = state(TickRate::Hz30);
    s.record_hit(0, k).unwrap();
    s.advance(0, true, target(k, true)).unwrap();
    assert_eq!(
        s.advance(100, true, target(k, true)).unwrap(),
        Event::Strike {
            target: k,
            amount: 5
        }
    );
    assert_eq!(s.advance(100, true, target(k, true)).unwrap(), Event::None);
    assert_eq!(s.advance(101, true, target(k, true)).unwrap(), Event::None);
    assert_eq!(
        s.advance(145, true, target(k, true)).unwrap(),
        Event::Windup {
            target: k,
            completes: 160
        }
    );
    assert_eq!(
        s.advance(300, true, target(k, true)).unwrap(),
        Event::Cancelled { target: k }
    );
    assert_eq!(s.target(), None);
}
#[test]
fn stale_time_and_exhaustion_refuse_without_partial_mutation() {
    let k = key(1, 1);
    let mut s = state(TickRate::Hz30);
    let before = s.clone();
    assert_eq!(s.record_hit(u64::MAX, k), Err(Error::TimeExhausted));
    assert_eq!(s, before);
    s.record_hit(10, k).unwrap();
    let before = s.clone();
    assert_eq!(s.advance(9, true, target(k, true)), Err(Error::StaleTick));
    assert_eq!(s, before);
    assert_eq!(s.record_hit(9, k), Err(Error::StaleTick));
    assert_eq!(s, before);
    let mut s = state(TickRate::Hz30);
    s.record_hit(u64::MAX - 300, k).unwrap();
    s.advance(u64::MAX - 300, true, target(k, true)).unwrap();
    let before = s.clone();
    assert_eq!(
        s.advance(u64::MAX - 1, true, target(k, true)),
        Err(Error::TimeExhausted)
    );
    assert_eq!(s, before);
}
#[test]
fn recording_a_hit_after_advance_never_evaluates_twice_in_the_same_tick() {
    let k = key(1, 1);
    let mut s = state(TickRate::Hz30);
    assert_eq!(s.advance(0, true, None).unwrap(), Event::None);
    s.record_hit(0, k).unwrap();
    assert_eq!(s.advance(0, true, target(k, true)).unwrap(), Event::None);
    assert_eq!(
        s.advance(1, true, target(k, true)).unwrap(),
        Event::Windup {
            target: k,
            completes: 16
        }
    );
}

#[test]
fn transfer_cancels_old_windup_and_new_victim_gets_fresh_telegraph_after_recovery() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (a, b) = (key(1, 1), key(1, 2));
        let mut s = state(rate);
        s.record_hit(0, a).unwrap();
        s.advance(0, true, target(a, true)).unwrap();
        assert_eq!(s.transfer(1, b), Ok(Event::Cancelled { target: a }));
        assert_eq!(s.target(), Some(b));
        assert_eq!(s.windup_until(), None);
        let ready = 1 + (1500 * u64::from(rate.hz())).div_ceil(1000);
        let windup = (500 * u64::from(rate.hz())).div_ceil(1000);
        for now in 1..ready {
            assert_eq!(s.advance(now, true, target(b, true)), Ok(Event::None));
        }
        assert_eq!(
            s.advance(ready, true, target(b, true)),
            Ok(Event::Windup {
                target: b,
                completes: ready + windup
            })
        );
        assert_eq!(
            s.advance(ready + windup, true, target(b, true)),
            Ok(Event::Strike {
                target: b,
                amount: 5
            })
        );
    }
}
#[test]
fn transfer_during_recovery_cannot_shorten_the_existing_attack_timer() {
    let (a, b) = (key(1, 1), key(1, 2));
    let mut s = state(TickRate::Hz30);
    s.record_hit(0, a).unwrap();
    s.advance(0, true, target(a, true)).unwrap();
    assert!(matches!(
        s.advance(15, true, target(a, true)),
        Ok(Event::Strike { .. })
    ));
    assert_eq!(s.transfer(16, b), Ok(Event::None));
    for now in 16..60 {
        assert_eq!(s.advance(now, true, target(b, true)), Ok(Event::None));
    }
    assert_eq!(
        s.advance(60, true, target(b, true)),
        Ok(Event::Windup {
            target: b,
            completes: 75
        })
    );
}
#[test]
fn transfer_refuses_inactive_foreign_postevaluation_and_overflow_without_partial_mutation() {
    let (a, b) = (key(1, 1), key(1, 2));
    let mut s = state(TickRate::Hz30);
    let before = s.clone();
    assert_eq!(s.transfer(0, b), Err(Error::InvalidTransfer));
    assert_eq!(s, before);
    s.record_hit(0, a).unwrap();
    s.advance(0, true, target(a, true)).unwrap();
    let before = s.clone();
    assert_eq!(s.transfer(0, b), Err(Error::StaleTick));
    assert_eq!(s, before);
    assert_eq!(s.transfer(1, key(2, 2)), Err(Error::InvalidTransfer));
    assert_eq!(s, before);
    assert_eq!(s.transfer(1, a), Ok(Event::None));
    assert_eq!(s, before);
    let mut s = state(TickRate::Hz30);
    s.record_hit(u64::MAX - 400, a).unwrap();
    s.advance(u64::MAX - 400, true, target(a, true)).unwrap();
    let before = s.clone();
    assert_eq!(s.transfer(u64::MAX - 101, b), Err(Error::TimeExhausted));
    assert_eq!(s, before);
}
