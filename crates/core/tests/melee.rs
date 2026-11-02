use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    facing::Facing,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory,
    melee::{
        Catalog,
        timer::{Clock, MAX_DELAY_TICKS, MAX_PERIOD_TICKS, Step},
    },
    time::TickRate,
};
const FIXTURE: &[u8] = include_bytes!("../../../content/gameplay/development-melee-weapons.json");
fn key(n: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(n).unwrap(),
    }
}
fn sword() -> ContentId {
    ContentId::new("theia.dev.training-sword").unwrap()
}
#[test]
fn fixture_is_metric_item_bound_and_ceil_timed_at_every_supported_rate() {
    let catalog = Catalog::from_json(FIXTURE).unwrap();
    assert_eq!(catalog.id().as_str(), "theia.dev.classic-melee-fixture");
    let weapon = catalog.weapon(&sword()).unwrap();
    assert_eq!(weapon.item(), &sword());
    assert_eq!(weapon.period_ms(), 2400);
    assert_eq!(weapon.damage(), 15);
    assert_eq!(weapon.reach_metres(), 3.);
    let from = GlobalPosition::new([9e11, 12., 9e11]).unwrap();
    let front = GlobalPosition::new([9e11, 12., 9e11 - 2.]).unwrap();
    let behind = GlobalPosition::new([9e11, 12., 9e11 + 2.]).unwrap();
    assert!(weapon.arc().contains(Facing::default(), from, front));
    assert!(!weapon.arc().contains(Facing::default(), from, behind));
    let items = inventory::Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    catalog.validate_items(&items).unwrap();
    let old = inventory::Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-items.json"
    ))
    .unwrap();
    assert!(catalog.validate_items(&old).is_err());
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let period = weapon.period_ticks(rate);
        assert_eq!(period, u64::from(rate.hz()) * 24 / 10);
        let mut clock = Clock::new(key(1), period, 0).unwrap();
        clock.engage(key(2), 0).unwrap();
        let mut hits = vec![];
        for tick in 0..u64::from(rate.hz()) * 10 {
            if let Step::Swing { target, .. } = clock.step(tick, true).unwrap() {
                assert_eq!(target, key(2));
                hits.push(tick);
            }
        }
        assert_eq!(hits.len(), 5);
        assert!(hits.windows(2).all(|v| v[1] - v[0] == period));
    }
    let mut raw: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    raw["weapons"][0]["period_ms"] = serde_json::json!(1001);
    let catalog = Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert_eq!(
        catalog
            .weapon(&sword())
            .unwrap()
            .period_ticks(TickRate::Hz30),
        31
    );
}
#[test]
fn profile_faults_are_bounded_and_fail_closed() {
    let raw: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    for (field, value) in [
        ("period_ms", serde_json::json!(0)),
        ("period_ms", serde_json::json!(10001)),
        ("damage", serde_json::json!(0)),
        ("reach_metres", serde_json::json!(-1)),
        ("reach_metres", serde_json::json!(10.1)),
        ("frontal_arc_degrees", serde_json::json!(0)),
        ("frontal_arc_degrees", serde_json::json!(361)),
        ("item", serde_json::json!("")),
        ("unknown", serde_json::json!(true)),
    ] {
        let mut bad = raw.clone();
        bad["weapons"][0][field] = value;
        assert!(
            Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut duplicate = raw.clone();
    duplicate["weapons"]
        .as_array_mut()
        .unwrap()
        .push(raw["weapons"][0].clone());
    assert!(Catalog::from_json(&serde_json::to_vec(&duplicate).unwrap()).is_err());
    let mut empty = raw.clone();
    empty["weapons"] = serde_json::json!([]);
    assert!(Catalog::from_json(&serde_json::to_vec(&empty).unwrap()).is_err());
    assert!(Catalog::from_json(&vec![b' '; theia_core::melee::MAX_PROFILE_BYTES + 1]).is_err());
    let mut bad = raw;
    bad["weapons"][0]["item"] = serde_json::json!("theia.dev.training-token");
    let catalog = Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).unwrap();
    let items = inventory::Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    assert!(catalog.validate_items(&items).is_err());
}
#[test]
fn stop_reengage_retarget_and_repeated_enable_cannot_accelerate_swing() {
    let mut clock = Clock::new(key(1), 30, 0).unwrap();
    assert_eq!(clock.step(0, true).unwrap(), Step::Idle);
    clock.engage(key(2), 0).unwrap();
    assert!(matches!(clock.step(0, true).unwrap(), Step::Swing { .. }));
    for tick in 0..30 {
        clock.engage(key(2), tick).unwrap();
        clock.stop(tick).unwrap();
        clock.engage(key(3), tick).unwrap();
        assert_eq!(
            clock.step(tick, true).unwrap(),
            Step::Waiting { ready_at: 30 }
        );
    }
    assert_eq!(
        clock.step(30, true).unwrap(),
        Step::Swing {
            target: key(3),
            next_at: 60
        }
    );
    assert_eq!(
        clock.step(30, true).unwrap(),
        Step::Waiting { ready_at: 60 }
    );
}
#[test]
fn range_facing_or_los_pause_keeps_target_without_banked_bursts() {
    let mut clock = Clock::new(key(1), 30, 0).unwrap();
    clock.engage(key(2), 0).unwrap();
    clock.step(0, true).unwrap();
    for tick in [1, 30, 3000] {
        assert_eq!(clock.step(tick, false).unwrap(), Step::Paused);
        assert_eq!(clock.target(), Some(key(2)));
    }
    assert_eq!(
        clock.step(3001, true).unwrap(),
        Step::Swing {
            target: key(2),
            next_at: 3031
        }
    );
    assert_eq!(
        clock.step(3001, true).unwrap(),
        Step::Waiting { ready_at: 3031 }
    );
    assert_eq!(
        clock.step(3010, true).unwrap(),
        Step::Waiting { ready_at: 3031 }
    );
}
#[test]
fn ability_suspend_delay_and_reset_have_explicit_separate_timer_effects() {
    let mut clock = Clock::new(key(1), 30, 0).unwrap();
    clock.engage(key(2), 0).unwrap();
    clock.step(0, true).unwrap();
    clock.suspend(10).unwrap();
    clock.suspend(50).unwrap();
    assert!(clock.suspended());
    assert_eq!(clock.step(60, true).unwrap(), Step::Paused);
    clock.delay(60, 12).unwrap();
    clock.resume(100).unwrap();
    assert_eq!(
        clock.step(100, true).unwrap(),
        Step::Waiting { ready_at: 132 }
    );
    assert!(matches!(
        clock.step(132, true).unwrap(),
        Step::Swing { next_at: 162, .. }
    ));
    clock.suspend(140).unwrap();
    clock.reset(150, 20).unwrap();
    clock.resume(200).unwrap();
    assert_eq!(clock.ready_at(), 220);
    assert_eq!(
        clock.step(219, true).unwrap(),
        Step::Waiting { ready_at: 220 }
    );
    assert!(matches!(
        clock.step(220, true).unwrap(),
        Step::Swing { next_at: 240, .. }
    ));
    clock.reset(225, 40).unwrap();
    assert_eq!(clock.ready_at(), 265);
}
#[test]
fn incarnation_invalidation_and_invalid_inputs_do_not_corrupt_the_clock() {
    let mut clock = Clock::new(key(1), 30, 10).unwrap();
    clock.engage(key(2), 10).unwrap();
    let initial = clock;
    for target in [
        key(1),
        EntityKey {
            epoch: WorldEpoch::new(8).unwrap(),
            ..key(2)
        },
    ] {
        assert!(clock.engage(target, 10).is_err());
        assert_eq!(clock, initial);
    }
    assert!(clock.step(9, true).is_err());
    assert_eq!(clock, initial);
    assert!(clock.reset(10, 0).is_err());
    assert_eq!(clock, initial);
    assert!(clock.reset(10, MAX_PERIOD_TICKS + 1).is_err());
    assert_eq!(clock, initial);
    assert!(clock.delay(10, MAX_DELAY_TICKS + 1).is_err());
    assert_eq!(clock, initial);
    assert!(!clock.invalidate(key(3), 10).unwrap());
    assert_eq!(clock.target(), Some(key(2)));
    assert!(clock.invalidate(key(2), 10).unwrap());
    assert_eq!(clock.step(10, true).unwrap(), Step::Idle);
}
#[test]
fn arithmetic_exhaustion_is_atomic_and_never_authorizes_damage() {
    let mut clock = Clock::new(key(1), 30, u64::MAX - 10).unwrap();
    clock.engage(key(2), u64::MAX - 10).unwrap();
    let before = clock;
    assert!(clock.step(u64::MAX - 10, true).is_err());
    assert_eq!(clock, before);
    assert!(clock.reset(u64::MAX, 30).is_err());
    assert_eq!(clock, before);
    let mut clock = Clock::new(key(1), 20, u64::MAX - 20).unwrap();
    clock.engage(key(2), u64::MAX - 20).unwrap();
    clock.step(u64::MAX - 20, true).unwrap();
    clock.suspend(u64::MAX - 10).unwrap();
    let before = clock;
    assert!(clock.resume(u64::MAX - 5).is_err());
    assert_eq!(clock, before);
    assert!(clock.delay(u64::MAX - 5, 1).is_err());
    assert_eq!(clock, before);
}
