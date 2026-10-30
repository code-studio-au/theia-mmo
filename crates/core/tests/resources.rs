//! Resource definition/arithmetic contracts only; no working combat claim.
use serde_json::{Value, json};
use theia_core::{
    resources::{MAX_PROFILE_BYTES, MAX_VALUE, Pool, Profile},
    time::TickRate,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-character-stats.json");
#[test]
fn actual_development_profile_is_bounded_and_rejects_unsupported_data() {
    let profile = Profile::from_json(DATA).unwrap();
    assert_eq!(profile.id().as_str(), "theia.dev.player-vitals");
    assert_eq!(
        (
            profile.maximum_health(),
            profile.maximum_mana(),
            profile.mana_regeneration_per_second()
        ),
        (100, 100, 5)
    );
    let baseline: Value = serde_json::from_slice(DATA).unwrap();
    for (field, bad) in [
        ("schema", json!(2)),
        ("id", json!("../private")),
        ("maximum_health", json!(0)),
        ("maximum_health", json!(MAX_VALUE + 1)),
        ("maximum_mana", json!(0)),
        ("maximum_mana", json!(-1)),
        ("maximum_mana", json!(1.5)),
        ("mana_regeneration_per_second", json!(MAX_VALUE + 1)),
        ("unknown", json!(true)),
    ] {
        let mut value = baseline.clone();
        value[field] = bad;
        assert!(
            Profile::from_json(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{field}"
        );
    }
    assert!(Profile::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1]).is_err());
    assert!(Profile::from_json(b"{}").is_err());
    for name in [
        "schema",
        "id",
        "maximum_health",
        "maximum_mana",
        "mana_regeneration_per_second",
    ] {
        let mut value = baseline.clone();
        value.as_object_mut().unwrap().remove(name);
        assert!(Profile::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}
#[test]
fn deterministic_regeneration_matches_every_qualified_tick_rate_and_fractional_ticks() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for regen in [0, 1, 5, 17, 1_000_000] {
            let mut pool = Pool::new(0, MAX_VALUE).unwrap();
            for tick in 1..=rate.hz() * 7 {
                pool.regenerate(rate, regen);
                let expected = (u64::from(tick) * u64::from(regen) / u64::from(rate.hz()))
                    .min(u64::from(MAX_VALUE));
                assert_eq!(
                    u64::from(pool.current()),
                    expected,
                    "{rate:?} {regen} {tick}"
                );
            }
        }
    }
}
#[test]
fn capped_pools_do_not_bank_hidden_regeneration_and_failed_costs_are_atomic() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut pool = Pool::new(99, 100).unwrap();
        for _ in 0..rate.hz() * 5 {
            pool.regenerate(rate, 17);
        }
        assert_eq!(pool.current(), 100);
        assert!(pool.spend(100));
        pool.regenerate(rate, 17);
        assert_eq!(
            pool.current(),
            0,
            "No capped fractional or whole surplus survives spending"
        );
        let before = pool;
        assert!(!pool.spend(1));
        assert_eq!(pool, before);
        for _ in 1..rate.hz() {
            pool.regenerate(rate, 17);
        }
        assert_eq!(pool.current(), 17);
        assert!(pool.spend(0));
        assert_eq!(pool.current(), 17);
    }
    for (current, maximum) in [(0, 0), (101, 100), (1, MAX_VALUE + 1), (u32::MAX, u32::MAX)] {
        assert!(Pool::new(current, maximum).is_none());
    }
    let mut pool = Pool::new(1, MAX_VALUE).unwrap();
    pool.regenerate(TickRate::Hz20, u32::MAX);
    assert_eq!(
        pool.current(),
        MAX_VALUE,
        "Public arithmetic cannot overflow even on extreme caller input"
    );
}

#[test]
fn vitals_views_cannot_represent_excess_or_unbounded_health_or_mana() {
    use theia_core::resources::Vitals;
    let view = Vitals::new(0, 100, 25, 100).unwrap();
    assert_eq!(
        (
            view.health(),
            view.maximum_health(),
            view.mana(),
            view.maximum_mana()
        ),
        (0, 100, 25, 100)
    );
    for values in [
        [101, 100, 25, 100],
        [0, 0, 25, 100],
        [1, 100, 101, 100],
        [1, 100, 1, 0],
        [1, MAX_VALUE + 1, 1, 100],
    ] {
        assert!(Vitals::new(values[0], values[1], values[2], values[3]).is_none());
    }
}
