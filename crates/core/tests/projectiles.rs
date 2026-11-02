use std::time::Duration;
use theia_core::{
    coordinates::{GlobalPosition, MAX_GLOBAL_METRES},
    projectiles::{Flight, Profile, State},
    time::TickRate,
};

fn point(x: f64, y: f64, z: f64) -> GlobalPosition {
    GlobalPosition::new([x, y, z]).unwrap()
}
fn profile(speed: f64, distance: f64, lifetime_ms: u32, radius: f64) -> Profile {
    serde_json::from_value(serde_json::json!({
        "speed_metres_per_second":speed,
        "maximum_distance_metres":distance,
        "lifetime_ms":lifetime_ms,
        "target_radius_metres":radius,
    }))
    .unwrap()
}
fn distance(a: GlobalPosition, b: GlobalPosition) -> f64 {
    a.metres()
        .into_iter()
        .zip(b.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
fn rates() -> [TickRate; 3] {
    [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60]
}

#[test]
fn finite_speed_produces_swept_segments_and_a_single_candidate_impact_at_all_rates() {
    for rate in rates() {
        let start = point(15040., 25., 15040.);
        let target = point(15050.1, 25., 15040.);
        let mut flight = Flight::launch(profile(10., 30., 3000, 0.1), start, rate);
        let mut prior = start;
        let mut arrivals = 0;
        for tick in 1..=rate.hz() + 1 {
            let Some(segment) = flight.advance(target) else {
                break;
            };
            assert_eq!(segment.from, prior);
            assert!(distance(segment.from, segment.to) <= 10. / f64::from(rate.hz()) + 1e-8);
            if segment.state == State::Arrived {
                arrivals += 1;
                assert!(flight.elapsed().as_secs_f64() >= 1. - 1e-8);
                assert!(flight.elapsed().as_secs_f64() <= 1. + 1. / f64::from(rate.hz()) + 1e-8);
                assert!(distance(segment.to, target) <= 0.1 + 1e-8);
            } else {
                assert_eq!(segment.state, State::Travelling);
                assert!(tick <= rate.hz());
            }
            prior = segment.to;
        }
        assert_eq!(arrivals, 1);
        assert!((flight.travelled_metres() - 10.).abs() < 1e-8);
        let saved = (
            flight.position(),
            flight.elapsed(),
            flight.travelled_metres(),
        );
        for _ in 0..100 {
            assert_eq!(flight.advance(start), None);
        }
        assert_eq!(
            saved,
            (
                flight.position(),
                flight.elapsed(),
                flight.travelled_metres()
            )
        );
    }
}

#[test]
fn moving_target_cannot_extend_accumulated_travel_budget_or_reactivate_expired_flight() {
    for rate in rates() {
        let start = point(0., 0., 0.);
        let mut flight = Flight::launch(profile(10., 5., 3000, 0.1), start, rate);
        let mut length = 0.;
        let mut turned = false;
        for tick in 0..rate.hz() * 3 {
            let target = if tick < rate.hz() / 4 {
                point(30., 0., 0.)
            } else {
                turned = true;
                point(-30., 10., 0.)
            };
            let Some(segment) = flight.advance(target) else {
                break;
            };
            length += distance(segment.from, segment.to);
            assert_ne!(segment.state, State::Arrived);
        }
        assert!(turned);
        assert_eq!(flight.state(), State::Expired);
        assert!((flight.travelled_metres() - 5.).abs() < 1e-8);
        assert!((length - 5.).abs() < 1e-8);
        assert!(distance(start, flight.position()) < length);
        assert_eq!(flight.advance(flight.position()), None);
    }
}

#[test]
fn lifetime_clips_the_final_segment_even_when_it_is_between_ticks() {
    for rate in rates() {
        let mut flight = Flight::launch(profile(10., 100., 125, 0.1), point(0., 0., 0.), rate);
        let target = point(30., 0., 0.);
        let mut steps = 0;
        while let Some(segment) = flight.advance(target) {
            assert_ne!(segment.state, State::Arrived);
            steps += 1;
            assert!(steps <= 8);
        }
        assert_eq!(flight.state(), State::Expired);
        assert_eq!(flight.elapsed(), Duration::from_millis(125));
        assert!((flight.travelled_metres() - 1.25).abs() < 1e-9);
        assert!((flight.position().metres()[0] - 1.25).abs() < 1e-9);
    }
}

#[test]
fn arrival_at_budget_boundary_wins_and_initial_overlap_is_one_zero_length_candidate() {
    for rate in rates() {
        let start = point(0., 0., 0.);
        let mut flight = Flight::launch(profile(10., 1., 100, 0.25), start, rate);
        while let Some(segment) = flight.advance(point(1.25, 0., 0.)) {
            assert_ne!(segment.state, State::Expired);
        }
        assert_eq!(flight.state(), State::Arrived);
        assert!((flight.travelled_metres() - 1.).abs() < 1e-9);
        let mut overlap = Flight::launch(profile(10., 1., 100, 0.25), start, rate);
        let result = overlap.advance(point(0.1, 0., 0.)).unwrap();
        assert_eq!(result.from, result.to);
        assert_eq!(result.state, State::Arrived);
        assert_eq!(overlap.advance(start), None);
    }
}

#[test]
fn checked_profile_rejects_incomplete_unknown_nonfinite_and_out_of_budget_values() {
    let valid = serde_json::json!({"speed_metres_per_second":10.,"maximum_distance_metres":30.,"lifetime_ms":3000,"target_radius_metres":0.1});
    for field in [
        "speed_metres_per_second",
        "maximum_distance_metres",
        "lifetime_ms",
        "target_radius_metres",
    ] {
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Profile>(missing).is_err());
        let mut null = valid.clone();
        null[field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<Profile>(null).is_err());
    }
    for (field, invalid) in [
        ("speed_metres_per_second", -1.),
        ("speed_metres_per_second", 0.),
        ("speed_metres_per_second", 100.1),
        ("maximum_distance_metres", 0.09),
        ("maximum_distance_metres", 256.1),
        ("lifetime_ms", 99.),
        ("lifetime_ms", 10001.),
        ("target_radius_metres", 0.),
        ("target_radius_metres", 1.1),
    ] {
        let mut raw = valid.clone();
        raw[field] = serde_json::json!(invalid);
        assert!(
            serde_json::from_value::<Profile>(raw).is_err(),
            "{field}: {invalid}"
        );
    }
    let mut unknown = valid.clone();
    unknown["client_impact"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Profile>(unknown).is_err());
    let mut too_wide = valid.clone();
    too_wide["maximum_distance_metres"] = serde_json::json!(0.1);
    too_wide["target_radius_metres"] = serde_json::json!(0.2);
    assert!(serde_json::from_value::<Profile>(too_wide).is_err());
    for value in ["1e999", "NaN", "Infinity"] {
        let raw = format!(
            "{{\"speed_metres_per_second\":{value},\"maximum_distance_metres\":30,\"lifetime_ms\":3000,\"target_radius_metres\":0.1}}"
        );
        assert!(serde_json::from_str::<Profile>(&raw).is_err());
    }
    let duplicate = r#"{"speed_metres_per_second":10,"speed_metres_per_second":20,"maximum_distance_metres":30,"lifetime_ms":3000,"target_radius_metres":0.1}"#;
    assert!(serde_json::from_str::<Profile>(duplicate).is_err());
    for speed in [0.1, 100.] {
        for distance in [0.1, 256.] {
            for life in [100, 10_000] {
                let p = profile(speed, distance, life, 0.01);
                assert_eq!(p.speed_metres_per_second(), speed);
                assert_eq!(p.maximum_distance_metres(), distance);
                assert_eq!(p.lifetime(), Duration::from_millis(u64::from(life)));
                assert_eq!(p.target_radius_metres(), 0.01);
            }
        }
    }
}

#[test]
fn replay_and_far_global_boundaries_preserve_finite_bounded_segments() {
    for rate in rates() {
        for sign in [-1., 1.] {
            let start = point(sign * MAX_GLOBAL_METRES, 0., sign * MAX_GLOBAL_METRES);
            let p = profile(10., 30., 3000, 0.1);
            let mut a = Flight::launch(p, start, rate);
            let mut b = a.clone();
            for tick in 0..rate.hz() * 4 {
                let target = point(
                    sign * (MAX_GLOBAL_METRES - 20.),
                    f64::from(tick % 7),
                    sign * (MAX_GLOBAL_METRES - 20.),
                );
                let first = a.advance(target);
                assert_eq!(first, b.advance(target));
                if let Some(segment) = first {
                    assert!(distance(segment.from, segment.to) < 0.501);
                    assert!(
                        segment
                            .to
                            .metres()
                            .iter()
                            .all(|v| v.is_finite() && v.abs() <= MAX_GLOBAL_METRES)
                    );
                }
                assert_eq!(a.position(), b.position());
                assert!(a.travelled_metres() <= 30.);
            }
            assert_ne!(a.state(), State::Travelling);
        }
    }
}
