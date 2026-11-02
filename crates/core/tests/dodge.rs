use theia_core::{
    dodge::{Direction, Gate, Profile, Rejection, State},
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
fn direction() -> Direction {
    Direction::from_axes([1000, 0]).unwrap()
}
fn rates() -> [TickRate; 3] {
    [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60]
}
#[test]
fn profile_is_strict_finite_and_bounds_speed_duration_cooldown_and_bytes() {
    let base: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-dodge.json"
    ))
    .unwrap();
    for (key, value) in [
        ("schema", serde_json::json!(2)),
        ("id", serde_json::json!("bad id")),
        ("distance_metres", serde_json::json!(0.)),
        ("distance_metres", serde_json::json!(6.1)),
        ("duration_ms", serde_json::json!(99)),
        ("duration_ms", serde_json::json!(751)),
        ("cooldown_ms", serde_json::json!(249)),
        ("cooldown_ms", serde_json::json!(10001)),
        ("extra", serde_json::json!(true)),
    ] {
        let mut data = base.clone();
        data[key] = value;
        assert!(
            Profile::from_json(&serde_json::to_vec(&data).unwrap()).is_err(),
            "{key}"
        );
    }
    let mut fast = base;
    fast["distance_metres"] = serde_json::json!(6.);
    fast["duration_ms"] = serde_json::json!(100);
    assert!(Profile::from_json(&serde_json::to_vec(&fast).unwrap()).is_err());
    assert!(Profile::from_json(&vec![b' '; 4097]).is_err());
    let p = profile();
    assert_eq!(p.id().as_str(), "theia.dev.dodge");
    assert_eq!(p.distance_metres(), 3.);
}
#[test]
fn direction_rejects_zero_and_invalid_axes_and_normalizes_diagonals() {
    for axes in [[0, 0], [1001, 0], [0, -1001], [i16::MIN, 1]] {
        assert!(Direction::from_axes(axes).is_none());
    }
    let [x, z] = Direction::from_axes([1000, -1000]).unwrap().components();
    assert!((x.hypot(z) - 1.).abs() < 1e-12);
    assert!(x > 0. && z < 0.);
}
#[test]
fn equal_distance_at_all_rates_starts_next_tick_and_finishes_once() {
    for rate in rates() {
        let p = profile();
        let mut state = State::default();
        state
            .start(
                &p,
                rate,
                10,
                Direction::from_axes([1000, 1000]).unwrap(),
                gate(),
            )
            .unwrap();
        assert!(state.advance(10).unwrap().is_none());
        let mut sum = [0.; 2];
        let mut finished = 0;
        let steps = rate.ticks_for(p.duration()).unwrap();
        for tick in 11..=10 + steps {
            let step = state.advance(tick).unwrap().unwrap();
            assert!(
                step.horizontal_metres[0].hypot(step.horizontal_metres[1])
                    <= 12. / f64::from(rate.hz()) + 1e-12
            );
            for (total, delta) in sum.iter_mut().zip(step.horizontal_metres) {
                *total += delta;
            }
            finished += u32::from(step.finished);
            assert!(state.advance(tick).unwrap().is_none());
        }
        assert_eq!(finished, 1);
        assert!(!state.active());
        assert!((sum[0].hypot(sum[1]) - 3.).abs() < 1e-12);
        assert!(state.advance(10000).unwrap().is_none());
    }
}
#[test]
fn cooldown_edge_and_busy_refusals_cannot_extend_or_restart_a_motion() {
    for rate in rates() {
        let p = profile();
        let mut state = State::default();
        state.start(&p, rate, 10, direction(), gate()).unwrap();
        let original = state;
        assert_eq!(
            state.start(&p, rate, 10, direction(), gate()),
            Err(Rejection::Busy)
        );
        assert_eq!(state, original);
        for tick in 11..=10 + rate.ticks_for(p.duration()).unwrap() {
            state.advance(tick).unwrap();
        }
        let ready = state.ready_at();
        assert_eq!(ready, 10 + u64::from(rate.hz()) * 3);
        let before = state;
        assert_eq!(
            state.start(&p, rate, ready - 1, direction(), gate()),
            Err(Rejection::Cooldown)
        );
        assert_eq!(state, before);
        state.start(&p, rate, ready, direction(), gate()).unwrap();
        assert!(state.active());
    }
}
#[test]
fn unavailable_airborne_and_unprepared_requests_do_not_spend_cooldown() {
    for (gate, reason) in [
        (
            Gate {
                alive: false,
                ..gate()
            },
            Rejection::Unavailable,
        ),
        (
            Gate {
                grounded: false,
                ..gate()
            },
            Rejection::Airborne,
        ),
        (
            Gate {
                terrain_ready: false,
                ..gate()
            },
            Rejection::Unprepared,
        ),
    ] {
        let mut state = State::default();
        assert_eq!(
            state.start(&profile(), TickRate::Hz30, 10, direction(), gate),
            Err(reason)
        );
        assert_eq!(state, State::default());
    }
}
#[test]
fn cancellation_is_idempotent_and_keeps_the_accepted_cooldown() {
    let p = profile();
    let mut state = State::default();
    state
        .start(&p, TickRate::Hz30, 10, direction(), gate())
        .unwrap();
    state.advance(11).unwrap();
    let ready = state.ready_at();
    assert!(state.cancel());
    assert!(!state.cancel());
    assert_eq!(state.ready_at(), ready);
    assert!(state.advance(12).unwrap().is_none());
    assert_eq!(
        state.start(&p, TickRate::Hz30, 13, direction(), gate()),
        Err(Rejection::Cooldown)
    );
}
#[test]
fn skipped_ticks_fail_closed_without_catchup_and_stale_start_cannot_rewind() {
    let p = profile();
    let mut state = State::default();
    state
        .start(&p, TickRate::Hz30, 10, direction(), gate())
        .unwrap();
    let ready = state.ready_at();
    assert_eq!(state.advance(13), Err(Rejection::SkippedTick));
    assert!(!state.active());
    assert_eq!(state.ready_at(), ready);
    assert!(state.advance(14).unwrap().is_none());
    assert_eq!(
        state.start(&p, TickRate::Hz30, 12, direction(), gate()),
        Err(Rejection::StaleTick)
    );
}
#[test]
fn tick_overflow_refuses_without_mutation_at_every_rate() {
    for rate in rates() {
        let mut state = State::default();
        assert_eq!(
            state.start(&profile(), rate, u64::MAX - 1, direction(), gate()),
            Err(Rejection::TimeExhausted)
        );
        assert_eq!(state, State::default());
    }
}
