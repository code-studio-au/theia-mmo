use theia_core::{
    dodge::{Active, Direction, Gate, Profile, Rejection, Snapshot, State},
    time::TickRate,
};
fn profile() -> Profile {
    Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-dodge.json"
    ))
    .unwrap()
}
fn gate() -> Gate {
    Gate {
        alive: true,
        grounded: true,
        terrain_ready: true,
    }
}
fn rates() -> [TickRate; 3] {
    [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60]
}
#[test]
fn every_active_terminal_and_cooldown_tick_restores_exact_policy_and_next_step() {
    let p = profile();
    for rate in rates() {
        let mut original = State::default();
        original
            .start(
                &p,
                rate,
                10,
                Direction::from_axes([123, -999]).unwrap(),
                gate(),
            )
            .unwrap();
        for tick in 10..=12 + rate.ticks_for(p.duration()).unwrap() {
            let snapshot = original.snapshot().unwrap();
            assert_eq!(snapshot.tick, tick);
            let mut restored = State::restore(&p, rate, snapshot).unwrap();
            assert_eq!(restored, original);
            assert_eq!(restored.advance(tick + 1), original.advance(tick + 1));
            assert_eq!(restored, original);
        }
    }
}
#[test]
fn restoring_partial_thirty_hz_motion_cannot_refund_distance_or_cooldown() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut state = State::default();
    state
        .start(
            &p,
            rate,
            10,
            Direction::from_axes([1000, 0]).unwrap(),
            gate(),
        )
        .unwrap();
    let mut travelled = 0.;
    for tick in 11..=14 {
        travelled += state.advance(tick).unwrap().unwrap().horizontal_metres[0];
    }
    let mut restored = State::restore(&p, rate, state.snapshot().unwrap()).unwrap();
    let mut remaining = 0.;
    for tick in 15..=18 {
        remaining += restored.advance(tick).unwrap().unwrap().horizontal_metres[0];
    }
    assert!((travelled + remaining - 3.).abs() < 1e-12);
    assert!(remaining < 1.5);
    assert_eq!(restored.ready_at(), 100);
    assert_eq!(
        restored.start(
            &p,
            rate,
            19,
            Direction::from_axes([1000, 0]).unwrap(),
            gate()
        ),
        Err(Rejection::Cooldown)
    );
}
#[test]
fn canceled_motion_restores_idle_without_refunding_its_deadline() {
    let p = profile();
    let rate = TickRate::Hz60;
    let mut state = State::default();
    state
        .start(
            &p,
            rate,
            10,
            Direction::from_axes([-1000, 0]).unwrap(),
            gate(),
        )
        .unwrap();
    state.advance(11).unwrap();
    state.cancel();
    let snapshot = state.snapshot().unwrap();
    assert!(snapshot.active.is_none());
    let mut restored = State::restore(&p, rate, snapshot).unwrap();
    assert_eq!(restored, state);
    assert!(restored.advance(12).unwrap().is_none());
    assert_eq!(restored.ready_at(), 190);
}
#[test]
fn impossible_active_timing_direction_and_cooldown_snapshots_are_rejected() {
    let p = profile();
    let rate = TickRate::Hz30;
    let good = Snapshot {
        tick: 12,
        ready_at: 100,
        active: Some(Active {
            started_at: 10,
            axes: [1000, 0],
        }),
    };
    assert!(State::restore(&p, rate, good).is_ok());
    let invalid = [
        Snapshot { tick: 9, ..good },
        Snapshot { tick: 18, ..good },
        Snapshot {
            ready_at: 99,
            ..good
        },
        Snapshot {
            ready_at: 0,
            ..good
        },
        Snapshot {
            active: Some(Active {
                axes: [0, 0],
                ..good.active.unwrap()
            }),
            ..good
        },
        Snapshot {
            active: Some(Active {
                axes: [1001, 0],
                ..good.active.unwrap()
            }),
            ..good
        },
        Snapshot {
            ready_at: 200,
            active: None,
            ..good
        },
        Snapshot {
            ready_at: 89,
            active: None,
            ..good
        },
    ];
    for value in invalid {
        assert!(State::restore(&p, rate, value).is_err(), "{value:?}");
    }
}
#[test]
fn unobserved_default_is_distinct_from_an_anchored_enabled_idle_state() {
    let p = profile();
    let mut state = State::default();
    assert!(state.snapshot().is_none());
    for tick in [0, 1000] {
        state.advance(tick).unwrap();
        let snapshot = state.snapshot().unwrap();
        assert_eq!(snapshot.tick, tick);
        assert_eq!(snapshot.ready_at, 0);
        assert!(snapshot.active.is_none());
        assert_eq!(State::restore(&p, TickRate::Hz30, snapshot).unwrap(), state);
    }
}
#[test]
fn snapshot_json_rejects_extra_position_fields_bad_axis_shape_and_missing_ticks() {
    for raw in [
        r#"{"tick":1,"ready_at":0,"active":null,"position":[0,0,0]}"#,
        r#"{"ready_at":0,"active":null}"#,
        r#"{"tick":1,"ready_at":90,"active":{"started_at":0,"axes":[1,2,3]}}"#,
    ] {
        assert!(serde_json::from_str::<Snapshot>(raw).is_err());
    }
    let snapshot = Snapshot {
        tick: 5,
        ready_at: 95,
        active: Some(Active {
            started_at: 5,
            axes: [-123, 999],
        }),
    };
    assert_eq!(
        serde_json::from_slice::<Snapshot>(&serde_json::to_vec(&snapshot).unwrap()).unwrap(),
        snapshot
    );
}
#[test]
fn near_clock_exhaustion_roundtrips_without_overflow_or_fresh_cooldown() {
    let p = profile();
    for rate in rates() {
        let start = u64::MAX - rate.ticks_for(p.cooldown()).unwrap();
        let mut state = State::default();
        state
            .start(
                &p,
                rate,
                start,
                Direction::from_axes([1, 1]).unwrap(),
                gate(),
            )
            .unwrap();
        let mut restored = State::restore(&p, rate, state.snapshot().unwrap()).unwrap();
        assert_eq!(restored, state);
        for tick in start + 1..=start + rate.ticks_for(p.duration()).unwrap() {
            restored.advance(tick).unwrap();
        }
        restored.advance(u64::MAX).unwrap();
        let snapshot = restored.snapshot().unwrap();
        assert_eq!(State::restore(&p, rate, snapshot).unwrap(), restored);
        let before = restored;
        assert_eq!(
            restored.start(
                &p,
                rate,
                u64::MAX,
                Direction::from_axes([1, 0]).unwrap(),
                gate()
            ),
            Err(Rejection::TimeExhausted)
        );
        assert_eq!(restored, before);
    }
}
#[test]
fn restored_motion_still_suppresses_duplicates_and_retires_skipped_ticks() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut state = State::default();
    state
        .start(&p, rate, 10, Direction::from_axes([1, 0]).unwrap(), gate())
        .unwrap();
    let mut state = State::restore(&p, rate, state.snapshot().unwrap()).unwrap();
    assert!(state.advance(10).unwrap().is_none());
    assert_eq!(state.advance(12), Err(Rejection::SkippedTick));
    assert!(!state.active());
    assert_eq!(state.ready_at(), 100);
}
