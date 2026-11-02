use theia_core::{
    dodge::{Gate, Rejection},
    time::TickRate,
};
use theia_network::dodging::{self, Controller, Movement, Outcome, Profile, Request};
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
fn request(nonce: u64) -> Request {
    Request {
        nonce,
        axes: [1000, 0],
    }
}
#[test]
fn repeated_nonce_never_restarts_even_after_cooldown_at_every_rate() {
    let p = profile();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut c = Controller::new(&p, rate, 0);
        let mut starts = 0;
        let mut total = 0.;
        for tick in 1..=u64::from(rate.hz()) * 4 {
            let applied = c.advance(&p, rate, tick, Some(request(1)), gate()).unwrap();
            starts += u32::from(applied.outcome == Outcome::Started);
            if let Movement::Dodge(Some(step)) = applied.movement {
                total += step.horizontal_metres[0];
            }
        }
        assert_eq!(starts, 1);
        assert!((total - 3.).abs() < 1e-12);
        assert_eq!(c.snapshot().last_nonce, 1);
    }
}
#[test]
fn refused_nonce_is_consumed_so_landing_or_cooldown_cannot_activate_it_later() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut c = Controller::new(&p, rate, 0);
    assert_eq!(
        c.advance(
            &p,
            rate,
            1,
            Some(request(1)),
            Gate {
                grounded: false,
                ..gate()
            }
        )
        .unwrap()
        .outcome,
        Outcome::Refused(Rejection::Airborne)
    );
    assert_eq!(
        c.advance(&p, rate, 2, Some(request(1)), gate())
            .unwrap()
            .outcome,
        Outcome::Duplicate
    );
    assert_eq!(
        c.advance(&p, rate, 3, Some(request(2)), gate())
            .unwrap()
            .outcome,
        Outcome::Started
    );
    for tick in 4..=12 {
        c.advance(&p, rate, tick, None, gate()).unwrap();
    }
    assert_eq!(
        c.advance(&p, rate, 13, Some(request(3)), gate())
            .unwrap()
            .outcome,
        Outcome::Refused(Rejection::Cooldown)
    );
    assert_eq!(
        c.advance(&p, rate, 100, Some(request(3)), gate())
            .unwrap()
            .outcome,
        Outcome::Duplicate
    );
    assert_eq!(
        c.advance(&p, rate, 101, Some(request(4)), gate())
            .unwrap()
            .outcome,
        Outcome::Started
    );
}
#[test]
fn death_or_unavailable_collision_retires_motion_without_refund() {
    let p = profile();
    let rate = TickRate::Hz30;
    for unavailable in [
        Gate {
            alive: false,
            ..gate()
        },
        Gate {
            terrain_ready: false,
            ..gate()
        },
    ] {
        let mut c = Controller::new(&p, rate, 0);
        c.advance(&p, rate, 1, Some(request(1)), gate()).unwrap();
        let ready = c.snapshot().policy.ready_at;
        let result = c.advance(&p, rate, 2, None, unavailable).unwrap();
        assert!(result.retired);
        assert_eq!(result.movement, Movement::Ordinary);
        assert!(c.snapshot().policy.active.is_none());
        assert_eq!(c.snapshot().policy.ready_at, ready);
        assert!(!c.advance(&p, rate, 3, None, unavailable).unwrap().retired);
    }
}
#[test]
fn duplicate_ticks_cannot_add_solver_steps_or_consume_new_edges() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut c = Controller::new(&p, rate, 0);
    c.advance(&p, rate, 1, Some(request(1)), gate()).unwrap();
    let before = c;
    assert_eq!(
        c.advance(&p, rate, 1, Some(request(99)), gate())
            .unwrap()
            .movement,
        Movement::None
    );
    assert_eq!(c, before);
    assert!(c.advance(&p, rate, 0, None, gate()).is_err());
    assert_eq!(c, before);
}
#[test]
fn live_profile_or_rate_change_and_malformed_requests_leave_state_unchanged() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut c = Controller::new(&p, rate, 0);
    let before = c;
    let other=Profile::from_json(br#"{"schema":1,"id":"theia.dev.dodge","distance_metres":4.0,"duration_ms":250,"cooldown_ms":3000}"#).unwrap();
    assert!(c.advance(&other, rate, 1, None, gate()).is_err());
    assert_eq!(c, before);
    assert!(c.advance(&p, TickRate::Hz60, 1, None, gate()).is_err());
    assert_eq!(c, before);
    for r in [
        Request {
            nonce: 0,
            axes: [1000, 0],
        },
        Request {
            nonce: 1,
            axes: [0, 0],
        },
        Request {
            nonce: 1,
            axes: [1001, 0],
        },
    ] {
        assert!(c.advance(&p, rate, 1, Some(r), gate()).is_err());
        assert_eq!(c, before);
    }
    assert_ne!(
        dodging::profile_identity([1; 32], &p),
        dodging::profile_identity([1; 32], &other)
    );
}
#[test]
fn correction_roundtrip_preserves_nonce_motion_and_next_result_at_every_rate() {
    let p = profile();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut c = Controller::new(&p, rate, 0);
        for tick in 1..=u64::from(rate.hz()) {
            let snapshot = c.snapshot();
            let mut replica = Controller::restore(&p, rate, tick - 1, snapshot).unwrap();
            assert_eq!(
                replica
                    .advance(&p, rate, tick, Some(request(1)), gate())
                    .unwrap(),
                c.advance(&p, rate, tick, Some(request(1)), gate()).unwrap()
            );
            assert_eq!(replica, c);
        }
    }
}
#[test]
fn correction_requires_exact_outer_tick_and_a_nonce_for_noninitial_cooldown() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut c = Controller::new(&p, rate, 0);
    c.advance(&p, rate, 1, Some(request(1)), gate()).unwrap();
    let good = c.snapshot();
    assert!(Controller::restore(&p, rate, 2, good).is_err());
    let mut bad = good;
    bad.last_nonce = 0;
    assert!(Controller::restore(&p, rate, 1, bad).is_err());
    assert!(Controller::restore(&p, rate, 1, good).is_ok());
}
#[test]
fn skipped_tick_cancels_without_catchup_and_nonce_exhaustion_is_bounded() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut c = Controller::new(&p, rate, 0);
    c.advance(&p, rate, 1, Some(request(u64::MAX)), gate())
        .unwrap();
    let gap = c
        .advance(&p, rate, 3, Some(request(u64::MAX)), gate())
        .unwrap();
    assert!(gap.retired);
    assert_eq!(gap.movement, Movement::Ordinary);
    assert_eq!(c.snapshot().last_nonce, u64::MAX);
    assert_eq!(
        c.advance(&p, rate, 100, Some(request(1)), gate())
            .unwrap()
            .outcome,
        Outcome::Duplicate
    );
}
