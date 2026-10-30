//! Status data/timing only. No live damage, entity ownership or network claim.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    statuses::{
        Catalog, Effect, Error, MAX_CATALOG_BYTES, MAX_DEFINITIONS, Schedule, Step, Timing,
    },
    time::TickRate,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-statuses.json");
#[test]
fn definitions_reject_unbounded_unknown_or_ambiguous_status_data() {
    let catalog = Catalog::from_json(DATA).unwrap();
    assert_eq!(catalog.id().as_str(), "theia.dev.combat-statuses");
    let status = catalog
        .definition(&ContentId::new("theia.dev.scorch").unwrap())
        .unwrap();
    assert_eq!(status.name(), "Scorch");
    assert_eq!(status.effect(), Effect::PeriodicDamage { amount: 5 });
    let original: Value = serde_json::from_slice(DATA).unwrap();
    for (field, value) in [
        ("schema", json!(2)),
        ("id", json!("../statuses")),
        ("statuses", json!([])),
        ("unknown", json!(1)),
    ] {
        let mut bad = original.clone();
        bad[field] = value;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (field, value) in [
        ("id", json!("Bad/Status")),
        ("name", json!("")),
        ("name", json!(" Scorch")),
        ("name", json!("Scorch\n")),
        ("name", json!("x".repeat(49))),
        ("duration_ms", json!(0)),
        ("duration_ms", json!(60_001)),
        ("interval_ms", json!(0)),
        ("interval_ms", json!(49)),
        ("interval_ms", json!(6001)),
        ("interval_ms", json!(1.5)),
        ("unknown", json!(1)),
        ("effect", json!({"kind":"periodic_damage","amount":0})),
        (
            "effect",
            json!({"kind":"periodic_damage","amount":1_000_001}),
        ),
        ("effect", json!({"kind":"periodic_damage","amount":-1})),
        ("effect", json!({"kind":"periodic_damage","amount":1.5})),
        ("effect", json!({"kind":"damage","amount":5})),
        (
            "effect",
            json!({"kind":"periodic_damage","amount":5,"unknown":true}),
        ),
    ] {
        let mut bad = original.clone();
        bad["statuses"][0][field] = value;
        assert!(
            Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut duplicate = original.clone();
    let copy = duplicate["statuses"][0].clone();
    duplicate["statuses"].as_array_mut().unwrap().push(copy);
    assert!(Catalog::from_json(&serde_json::to_vec(&duplicate).unwrap()).is_err());
    let mut oversized = original.clone();
    oversized["statuses"]=json!((0..=MAX_DEFINITIONS).map(|i|json!({"id":format!("theia.test.status-{i}"),"name":"Status","duration_ms":1000,"interval_ms":1000,"effect":{"kind":"periodic_damage","amount":1}})).collect::<Vec<_>>());
    assert!(Catalog::from_json(&serde_json::to_vec(&oversized).unwrap()).is_err());
    assert!(Catalog::from_json(&vec![b' '; MAX_CATALOG_BYTES + 1]).is_err());
    for field in ["id", "name", "duration_ms", "interval_ms", "effect"] {
        let mut bad = original.clone();
        bad["statuses"][0].as_object_mut().unwrap().remove(field);
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}
#[test]
fn exact_six_second_cadence_expires_once_and_includes_the_last_tick_at_every_rate() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let status = catalog.definitions().next().unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let timing = status.timing(rate);
        let hz = u64::from(rate.hz());
        assert_eq!(
            timing,
            Timing {
                duration_ticks: 6 * hz,
                interval_ticks: hz
            }
        );
        let mut schedule = Schedule::start(100, timing).unwrap();
        assert_eq!(schedule.advance(100), Ok(Step::default()));
        let mut due = Vec::new();
        let mut expired = Vec::new();
        for now in 101..=100 + 6 * hz {
            let step = schedule.advance(now).unwrap();
            if step.damage_due {
                due.push(now);
            }
            if step.expired {
                expired.push(now);
            } else {
                assert_eq!(
                    schedule.advance(now),
                    Ok(Step::default()),
                    "No duplicate damage at one tick"
                );
            }
        }
        assert_eq!(due, (1..=6).map(|n| 100 + n * hz).collect::<Vec<_>>());
        assert_eq!(expired, vec![100 + 6 * hz]);
        assert!(schedule.finished());
        assert_eq!(schedule.next_tick(), None);
        assert_eq!(schedule.advance(100 + 6 * hz), Err(Error::Finished));
        assert_eq!(schedule.refresh(100 + 6 * hz), Err(Error::Finished));
    }
}
#[test]
fn refresh_preserves_cadence_including_exact_due_ticks_and_fractional_duration_boundaries() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let status = catalog.definitions().next().unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let hz = u64::from(rate.hz());
        let mut schedule = Schedule::start(0, status.timing(rate)).unwrap();
        let mut due = Vec::new();
        for now in 1..=10 * hz {
            if now <= 4 * hz {
                let next = schedule.next_tick();
                schedule.refresh(now).unwrap();
                assert_eq!(schedule.next_tick(), next);
                assert_eq!(schedule.expires_at(), now + 6 * hz);
            }
            let step = schedule.advance(now).unwrap();
            if step.damage_due {
                due.push(now);
            }
            assert_eq!(step.expired, now == 10 * hz);
        }
        assert_eq!(due, (1..=10).map(|n| n * hz).collect::<Vec<_>>());
    }
    // Retain the nominal next tick even when it was beyond the old expiration.
    let mut schedule = Schedule::start(
        0,
        Timing {
            duration_ticks: 5,
            interval_ticks: 3,
        },
    )
    .unwrap();
    assert!(schedule.advance(3).unwrap().damage_due);
    assert_eq!(schedule.next_tick(), None);
    schedule.advance(4).unwrap();
    schedule.refresh(5).unwrap();
    assert_eq!(schedule.next_tick(), Some(6));
    let mut due = Vec::new();
    for now in 5..=10 {
        let step = schedule.advance(now).unwrap();
        if step.damage_due {
            due.push(now);
        }
        assert_eq!(step.expired, now == 10);
    }
    assert_eq!(due, vec![6, 9]);
    let mut raw: Value = serde_json::from_slice(DATA).unwrap();
    raw["statuses"][0]["duration_ms"] = json!(251);
    raw["statuses"][0]["interval_ms"] = json!(101);
    let catalog = Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let status = catalog.definitions().next().unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let timing = status.timing(rate);
        let hz = u64::from(rate.hz());
        assert_eq!(timing.duration_ticks, (251 * hz).div_ceil(1000));
        assert_eq!(timing.interval_ticks, (101 * hz).div_ceil(1000));
    }
}
#[test]
fn clock_regression_missed_ticks_and_overflow_fail_without_mutating_schedule() {
    for timing in [
        Timing {
            duration_ticks: 0,
            interval_ticks: 0,
        },
        Timing {
            duration_ticks: 1,
            interval_ticks: 2,
        },
    ] {
        assert_eq!(Schedule::start(0, timing), Err(Error::InvalidTiming));
    }
    let timing = Timing {
        duration_ticks: 6,
        interval_ticks: 2,
    };
    assert_eq!(
        Schedule::start(u64::MAX - 5, timing),
        Err(Error::TimeExhausted)
    );
    let mut schedule = Schedule::start(100, timing).unwrap();
    schedule.advance(102).unwrap();
    for (now, error) in [(101, Error::TimeRegressed), (105, Error::MissedTick)] {
        let before = schedule.clone();
        assert_eq!(schedule.advance(now), Err(error));
        assert_eq!(schedule, before);
        assert_eq!(schedule.refresh(now), Err(error));
        assert_eq!(schedule, before);
    }
    schedule.refresh(103).unwrap();
    let before = schedule.clone();
    assert_eq!(schedule.refresh(102), Err(Error::TimeRegressed));
    assert_eq!(schedule.advance(102), Err(Error::TimeRegressed));
    assert_eq!(schedule, before);
    assert!(!schedule.advance(103).unwrap().damage_due);
    assert!(schedule.advance(104).unwrap().damage_due);
    let mut edge = Schedule::start(u64::MAX - 6, timing).unwrap();
    let before = edge.clone();
    assert_eq!(edge.refresh(u64::MAX - 5), Err(Error::TimeExhausted));
    assert_eq!(edge, before);
    for offset in 1..=6 {
        let now = u64::MAX - 6 + offset;
        let step = edge.advance(now).unwrap();
        assert_eq!(step.damage_due, offset % 2 == 0);
        assert_eq!(step.expired, offset == 6);
    }
    assert!(edge.finished());
    assert_eq!(edge.next_tick(), None);
}
