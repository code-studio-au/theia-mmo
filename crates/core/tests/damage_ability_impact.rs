use theia_core::{
    damage::{
        Amount, MAX_VALUE, Range, Weapon,
        ability_impact::{
            magic_zero_resistance_reference_v1 as magic, physical_special_reference_v1 as physical,
        },
        melee_special::{self, Chances, Eligibility, Outcome},
        spell,
        white_melee::{
            eligibility::DefenderFacts,
            impact::{self, Impact},
        },
    },
    hit_outcomes::Roll,
};
fn roll(n: u16) -> Roll {
    Roll::new(n).unwrap()
}
fn eligible() -> Eligibility {
    Eligibility {
        dodge: true,
        parry: true,
        critical: true,
    }
}
#[test]
fn special_avoidance_uses_exact_eligible_order_and_conditional_crit_domain() {
    let c = Chances::new(500, 1000, 1500, 2500).unwrap();
    let e = eligible();
    assert_eq!(c.weights(e), [500, 1000, 1500, 7000]);
    let mut counts = [0; 4];
    for n in 0..10000 {
        let r = roll(n);
        let second = c.needs_critical_roll(r, e).then(|| roll(2500));
        let i = match c.resolve(r, second, e).unwrap() {
            Outcome::Miss => 0,
            Outcome::Dodge => 1,
            Outcome::Parry => 2,
            Outcome::Hit => 3,
            Outcome::Critical => panic!("Boundary draw should be normal"),
        };
        counts[i] += 1;
    }
    assert_eq!(counts, [500, 1000, 1500, 7000]);
    for (n, outcome) in [
        (499, Outcome::Miss),
        (500, Outcome::Dodge),
        (1499, Outcome::Dodge),
        (1500, Outcome::Parry),
        (2999, Outcome::Parry),
        (3000, Outcome::Hit),
    ] {
        assert_eq!(
            c.resolve(
                roll(n),
                c.needs_critical_roll(roll(n), e).then(|| roll(2500)),
                e
            )
            .unwrap(),
            outcome
        );
    }
    assert_eq!(
        c.resolve(roll(3000), Some(roll(2499)), e).unwrap(),
        Outcome::Critical
    );
    assert!(c.resolve(roll(2999), Some(roll(0)), e).is_err());
    assert!(c.resolve(roll(3000), None, e).is_err());
    let no = Eligibility {
        dodge: false,
        parry: false,
        critical: false,
    };
    assert_eq!(c.weights(no), [500, 0, 0, 9500]);
    assert_eq!(c.resolve(roll(500), None, no).unwrap(), Outcome::Hit);
    assert!(c.resolve(roll(500), Some(roll(0)), no).is_err());
    assert!(Chances::new(10001, 0, 0, 0).is_none());
    assert!(Chances::new(0, 10001, 0, 0).is_none());
    assert!(Chances::new(0, 0, 10001, 0).is_none());
    assert!(Chances::new(0, 0, 0, 10001).is_none());
}
#[test]
fn saturated_avoidance_and_zero_or_full_crit_have_no_extra_or_missing_rolls() {
    let e = eligible();
    let c = Chances::new(6000, 6000, 6000, 10000).unwrap();
    assert_eq!(c.weights(e), [6000, 4000, 0, 0]);
    for n in 0..10000 {
        assert!(!c.needs_critical_roll(roll(n), e));
        assert!(matches!(
            c.resolve(roll(n), None, e).unwrap(),
            Outcome::Miss | Outcome::Dodge
        ));
    }
    let c = Chances::new(500, 0, 0, 10000).unwrap();
    assert_eq!(
        c.resolve(roll(500), Some(roll(9999)), e).unwrap(),
        Outcome::Critical
    );
    let c = Chances::new(0, 0, 0, 0).unwrap();
    assert_eq!(c.resolve(roll(0), None, e).unwrap(), Outcome::Hit);
    assert!(c.resolve(roll(0), Some(roll(0)), e).is_err());
}
#[test]
fn standing_creature_defences_are_directional_and_unsupported_block_is_explicit() {
    let facts = DefenderFacts {
        standing: true,
        can_react: true,
        in_front: true,
        can_dodge: true,
        can_parry: true,
        ..Default::default()
    };
    assert_eq!(
        melee_special::creature_eligibility_reference_v1(facts, true).unwrap(),
        eligible()
    );
    let e = melee_special::creature_eligibility_reference_v1(
        DefenderFacts {
            in_front: false,
            ..facts
        },
        true,
    )
    .unwrap();
    assert!(e.dodge && !e.parry);
    for changed in [
        DefenderFacts {
            can_react: false,
            ..facts
        },
        DefenderFacts {
            casting_non_melee: true,
            ..facts
        },
    ] {
        let e = melee_special::creature_eligibility_reference_v1(changed, true).unwrap();
        assert!(!e.dodge && !e.parry && e.critical);
    }
    let e = melee_special::creature_eligibility_reference_v1(
        DefenderFacts {
            disarmed: true,
            ..facts
        },
        false,
    )
    .unwrap();
    assert!(e.dodge && !e.parry && !e.critical);
    assert!(
        melee_special::creature_eligibility_reference_v1(
            DefenderFacts {
                standing: false,
                ..facts
            },
            true
        )
        .is_err()
    );
    assert!(
        melee_special::creature_eligibility_reference_v1(
            DefenderFacts {
                can_block: true,
                ..facts
            },
            true
        )
        .is_err()
    );
}
#[test]
fn special_crit_precedes_armour_and_differs_from_white_swing_boundaries() {
    // 20.8 -> 20 -> critical 40 -> armour floor 33; ordinary 20 -> 16 -> 32.
    let start = Weapon::new(Range::new(16, 16).unwrap(), 2400)
        .unwrap()
        .starting_damage(16, 28)
        .unwrap();
    assert_eq!(start.exact_ratio(), (104, 5));
    let r = physical(start, Outcome::Critical, 1, 100, 0, 3).unwrap();
    assert_eq!(
        (
            r.starting_damage,
            r.critical_bonus,
            r.outcome_damage,
            r.armour_prevented,
            r.resolved_damage,
            r.health_lost,
            r.overkill
        ),
        (20, 20, 40, 7, 33, 3, 30)
    );
    let white = impact::physical_reference_v1(start, 1, 100, 0, 100, Impact::Critical).unwrap();
    assert_eq!(white.resolved_damage, 32);
    let normal = physical(start, Outcome::Hit, 1, 100, 0, 100).unwrap();
    assert_eq!((normal.critical_bonus, normal.resolved_damage), (0, 16));
    assert_eq!(
        physical(start, Outcome::Critical, 1, 100, 100, 100)
            .unwrap()
            .resolved_damage,
        40
    );
}
#[test]
fn magic_uses_its_own_multiplier_with_zero_resistance_and_explicit_health_clipping() {
    let start = Amount::fixed(21).unwrap();
    let r = magic(start, spell::Outcome::Critical, 3).unwrap();
    assert_eq!(
        (
            r.outcome_damage,
            r.critical_bonus,
            r.armour_prevented,
            r.health_lost,
            r.overkill
        ),
        (31, 10, 0, 3, 28)
    );
    assert_eq!(
        magic(start, spell::Outcome::Hit, 100)
            .unwrap()
            .resolved_damage,
        21
    );
    assert_eq!(
        magic(start, spell::Outcome::Miss, 100)
            .unwrap()
            .resolved_damage,
        0
    );
    assert_eq!(magic(start, spell::Outcome::Hit, 0).unwrap().overkill, 21);
    assert_eq!(
        magic(Amount::fixed(0).unwrap(), spell::Outcome::Critical, 100)
            .unwrap()
            .resolved_damage,
        0
    );
}
#[test]
fn avoidance_zero_minimum_overflow_and_invalid_mitigation_never_change_health() {
    for outcome in [Outcome::Miss, Outcome::Dodge, Outcome::Parry] {
        let r = physical(Amount::fixed(20).unwrap(), outcome, 1, 100, 0, 100).unwrap();
        assert_eq!(
            (
                r.outcome_damage,
                r.armour_prevented,
                r.health_lost,
                r.overkill
            ),
            (0, 0, 0, 0)
        );
    }
    assert_eq!(
        physical(Amount::fixed(0).unwrap(), Outcome::Critical, 1, 100, 0, 100)
            .unwrap()
            .resolved_damage,
        0
    );
    assert_eq!(
        physical(
            Amount::fixed(1).unwrap(),
            Outcome::Hit,
            1,
            MAX_VALUE,
            0,
            100
        )
        .unwrap()
        .resolved_damage,
        1
    );
    for outcome in [Outcome::Miss, Outcome::Hit, Outcome::Critical] {
        for (level, armour, pen, hp) in [
            (0, 0, 0, 100),
            (64, 0, 0, 100),
            (1, MAX_VALUE + 1, 0, 100),
            (1, 0, MAX_VALUE + 1, 100),
            (1, 0, 0, MAX_VALUE + 1),
        ] {
            assert!(physical(Amount::fixed(1).unwrap(), outcome, level, armour, pen, hp).is_err());
        }
    }
    assert!(
        physical(
            Amount::fixed(500001).unwrap(),
            Outcome::Critical,
            1,
            MAX_VALUE,
            0,
            100
        )
        .is_err()
    );
    assert_eq!(
        physical(
            Amount::fixed(500000).unwrap(),
            Outcome::Critical,
            1,
            0,
            0,
            1
        )
        .unwrap()
        .outcome_damage,
        MAX_VALUE
    );
    assert!(
        magic(
            Amount::fixed(666668).unwrap(),
            spell::Outcome::Critical,
            100
        )
        .is_err()
    );
    assert_eq!(
        magic(
            Amount::fixed(666667).unwrap(),
            spell::Outcome::Critical,
            100
        )
        .unwrap()
        .outcome_damage,
        MAX_VALUE
    );
    assert!(
        magic(
            Amount::fixed(1).unwrap(),
            spell::Outcome::Miss,
            MAX_VALUE + 1
        )
        .is_err()
    );
}
