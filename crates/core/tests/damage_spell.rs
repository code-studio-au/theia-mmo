use theia_core::{
    damage::spell::{Chances, Outcome, PveHit},
    hit_outcomes::Roll,
};
fn roll(n: u16) -> Roll {
    Roll::new(n).unwrap()
}
#[test]
fn creature_level_landing_has_a_separate_pve_curve_and_unavoidable_floor() {
    for (target, base, hit) in [
        (57, 100, 9900),
        (58, 200, 9800),
        (59, 300, 9700),
        (60, 400, 9600),
        (61, 500, 9500),
        (62, 600, 9400),
        (63, 1700, 8300),
    ] {
        let value = PveHit::reference_v1(60, target, 0).unwrap();
        assert_eq!(value.level_difference, i16::from(target) - 60);
        assert_eq!(value.raw_base_miss_bp, base);
        assert_eq!(value.hit_bp(), hit);
    }
    assert_eq!(PveHit::reference_v1(60, 63, 1600).unwrap().hit_bp(), 9900);
    assert_eq!(PveHit::reference_v1(60, 63, 1700).unwrap().hit_bp(), 9900);
    assert_eq!(PveHit::reference_v1(60, 63, 10000).unwrap().hit_bp(), 9900);
    assert_eq!(PveHit::reference_v1(60, 1, 0).unwrap().hit_bp(), 9900);
    assert_eq!(PveHit::reference_v1(1, 63, 0).unwrap().hit_bp(), 0);
    // Apply the hit bonus before clamping a large raw level penalty.
    let value = PveHit::reference_v1(1, 20, 10000).unwrap();
    assert_eq!(value.raw_base_miss_bp, 19300);
    assert_eq!(value.miss_bp(), 9300);
    for (a, b, bonus) in [
        (0, 1, 0),
        (1, 0, 0),
        (64, 1, 0),
        (1, 64, 0),
        (1, 1, 10001),
        (255, 255, 65535),
    ] {
        assert!(PveHit::reference_v1(a, b, bonus).is_err());
    }
}
#[test]
fn all_supported_levels_preserve_bounded_monotonic_landing_and_bonus() {
    for caster in 1..=63 {
        let mut previous = 10000;
        for target in 1..=63 {
            let base = PveHit::reference_v1(caster, target, 0).unwrap();
            assert!((100..=10000).contains(&base.miss_bp()));
            assert_eq!(base.miss_bp() + base.hit_bp(), 10000);
            assert!(base.hit_bp() <= previous);
            previous = base.hit_bp();
            let mut hit = base.hit_bp();
            for bonus in [1, 100, 300, 1600, 10000] {
                let boosted = PveHit::reference_v1(caster, target, bonus).unwrap();
                assert!(boosted.hit_bp() >= hit && boosted.hit_bp() <= 9900);
                hit = boosted.hit_bp();
            }
        }
    }
}
#[test]
fn second_critical_domain_is_conditional_and_not_pushed_off_by_misses() {
    let chances = Chances::new(PveHit::reference_v1(60, 63, 0).unwrap(), 10000).unwrap();
    assert_eq!(
        chances.resolve(roll(1699), None, true).unwrap(),
        Outcome::Miss
    );
    assert_eq!(
        chances.resolve(roll(1700), Some(roll(9999)), true).unwrap(),
        Outcome::Critical
    );
    assert!(chances.resolve(roll(1699), Some(roll(0)), true).is_err());
    assert!(chances.resolve(roll(1700), None, true).is_err());
    assert_eq!(
        chances.resolve(roll(1700), None, false).unwrap(),
        Outcome::Hit
    );
    assert!(chances.resolve(roll(1700), Some(roll(0)), false).is_err());
    let chances = Chances::new(PveHit::reference_v1(60, 60, 0).unwrap(), 0).unwrap();
    assert!(!chances.needs_critical_roll(roll(9999), true));
    assert_eq!(
        chances.resolve(roll(400), None, true).unwrap(),
        Outcome::Hit
    );
    assert!(chances.resolve(roll(400), Some(roll(0)), true).is_err());
    assert!(Chances::new(PveHit::reference_v1(1, 1, 0).unwrap(), 10001).is_none());
}
#[test]
fn exhaustive_domains_give_conditional_critical_probability_without_a_white_table() {
    let c = Chances::new(PveHit::reference_v1(60, 60, 0).unwrap(), 2500).unwrap();
    let mut misses = 0;
    let mut landed = 0;
    for n in 0..10000 {
        if c.needs_critical_roll(roll(n), true) {
            landed += 1;
            assert!(c.resolve(roll(n), None, true).is_err());
        } else {
            misses += 1;
            assert_eq!(c.resolve(roll(n), None, true).unwrap(), Outcome::Miss);
        }
    }
    let mut criticals = 0;
    let mut normal = 0;
    for n in 0..10000 {
        match c.resolve(roll(400), Some(roll(n)), true).unwrap() {
            Outcome::Critical => criticals += 1,
            Outcome::Hit => normal += 1,
            Outcome::Miss => panic!("Landed spell became a miss"),
        }
    }
    assert_eq!((misses, landed, criticals, normal), (400, 9600, 2500, 7500));
    // Exact joint frequencies over two independent 10,000-value domains.
    assert_eq!(landed * criticals, 24_000_000);
    assert_eq!(landed * normal, 72_000_000);
    assert_eq!(misses * 10000, 4_000_000);
    assert_eq!(
        c.resolve(roll(400), Some(roll(2499)), true).unwrap(),
        Outcome::Critical
    );
    assert_eq!(
        c.resolve(roll(400), Some(roll(2500)), true).unwrap(),
        Outcome::Hit
    );
}
