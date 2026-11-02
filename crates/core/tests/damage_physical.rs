use theia_core::damage::{Amount, MAX_VALUE, Range, Weapon, physical::normal_reference_v1};
#[test]
fn armour_uses_attacker_level_and_reports_separate_integer_facts() {
    let amount = Amount::fixed(118).unwrap();
    for (level, expected) in [(30, 41), (60, 59), (63, 60)] {
        let result = normal_reference_v1(amount, level, 5500, 0, 1000).unwrap();
        assert_eq!(result.resolved_damage, expected);
        assert_eq!(result.armour_prevented, 118 - expected);
        assert_eq!(result.health_lost, expected);
        assert_eq!(result.overkill, 0);
    }
}
#[test]
fn penetration_is_bounded_and_cannot_create_negative_armour() {
    let amount = Amount::fixed(118).unwrap();
    assert_eq!(
        normal_reference_v1(amount, 60, 5500, 2750, 1000)
            .unwrap()
            .resolved_damage,
        78
    );
    for penetration in [5500, 5501, MAX_VALUE] {
        let result = normal_reference_v1(amount, 60, 5500, penetration, 1000).unwrap();
        assert_eq!(result.effective_armour, 0);
        assert_eq!(result.resolved_damage, 118);
    }
    assert_eq!(
        normal_reference_v1(amount, 60, 0, 0, 1000)
            .unwrap()
            .resolved_damage,
        118
    );
}
#[test]
fn cap_positive_minimum_and_zero_are_distinct_policies() {
    for armour in [16500, 16501, MAX_VALUE] {
        assert_eq!(
            normal_reference_v1(Amount::fixed(100).unwrap(), 60, armour, 0, 1000)
                .unwrap()
                .resolved_damage,
            25
        );
        assert_eq!(
            normal_reference_v1(Amount::fixed(1).unwrap(), 60, armour, 0, 1000)
                .unwrap()
                .resolved_damage,
            1
        );
        assert_eq!(
            normal_reference_v1(Amount::fixed(0).unwrap(), 60, armour, 0, 1000)
                .unwrap()
                .resolved_damage,
            0
        );
    }
}
#[test]
fn starting_and_armour_quantization_are_explicit_and_overkill_is_not_clipped_damage() {
    let half = Weapon::new(Range::new(0, 0).unwrap(), 1000)
        .unwrap()
        .starting_damage(0, 7)
        .unwrap();
    let result = normal_reference_v1(half, 60, 5500, 0, 100).unwrap();
    assert_eq!(result.starting_exact.exact_ratio(), (1, 2));
    assert_eq!(result.starting_damage, 0);
    assert_eq!(result.resolved_damage, 0);
    let result = normal_reference_v1(Amount::fixed(118).unwrap(), 60, 5500, 0, 40).unwrap();
    assert_eq!(
        (result.resolved_damage, result.health_lost, result.overkill),
        (59, 40, 19)
    );
}
#[test]
fn unsupported_levels_and_magnitudes_fail_without_extrapolating_a_ruleset() {
    for level in [0, 64, 80, u8::MAX] {
        assert!(normal_reference_v1(Amount::fixed(1).unwrap(), level, 0, 0, 100).is_err());
    }
    for (armour, penetration, health) in [
        (MAX_VALUE + 1, 0, 100),
        (0, MAX_VALUE + 1, 100),
        (0, 0, MAX_VALUE + 1),
    ] {
        assert!(
            normal_reference_v1(Amount::fixed(1).unwrap(), 1, armour, penetration, health).is_err()
        );
    }
}
#[test]
fn reference_bounds_conserve_damage_and_armour_never_amplifies_a_hit() {
    for level in [1, 10, 30, 60, 63] {
        for raw in [0, 1, 3, 118, MAX_VALUE] {
            for penetration in [0, 100, MAX_VALUE] {
                let mut previous = raw;
                for armour in [0, 1, 100, 5500, 16500, MAX_VALUE] {
                    let result = normal_reference_v1(
                        Amount::fixed(raw).unwrap(),
                        level,
                        armour,
                        penetration,
                        40,
                    )
                    .unwrap();
                    assert!(result.resolved_damage <= previous);
                    previous = result.resolved_damage;
                    assert_eq!(result.resolved_damage + result.armour_prevented, raw);
                    assert_eq!(result.health_lost + result.overkill, result.resolved_damage);
                    assert!(result.health_lost <= 40);
                    if raw > 0 {
                        assert!(result.resolved_damage > 0);
                        assert!(result.resolved_damage >= raw / 4);
                    }
                }
            }
        }
    }
}
