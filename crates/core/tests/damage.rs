use theia_core::damage::{Amount, MAX_COEFFICIENT_BP, MAX_VALUE, Range, Spell, Weapon};

#[test]
fn main_hand_example_uses_ap_divisor_and_declared_speed_without_a_tick_clock() {
    let range = Range::new(80, 100).unwrap();
    let amount = Weapon::new(range, 2800)
        .unwrap()
        .starting_damage(90, 140)
        .unwrap();
    assert_eq!(amount.floor(), 118);
    assert_eq!(amount.nearest_half_up(), 118);
    let (numerator, denominator) = amount.exact_ratio();
    assert_eq!(numerator, 118 * denominator);
    assert_eq!(
        Weapon::new(range, 1400)
            .unwrap()
            .starting_damage(90, 140)
            .unwrap()
            .floor(),
        104
    );
    assert_eq!(
        Weapon::new(range, 5600)
            .unwrap()
            .starting_damage(90, 140)
            .unwrap()
            .floor(),
        146
    );
    // The same AP contributes 10 damage per second at either base speed.
}

#[test]
fn spell_example_and_coefficients_above_one_are_explicit() {
    let spell = Spell::new(Range::new(200, 200).unwrap(), 6000).unwrap();
    assert_eq!(spell.starting_damage(200, 100).unwrap().floor(), 260);
    let spell = Spell::new(Range::new(25, 25).unwrap(), 12500).unwrap();
    assert_eq!(spell.starting_damage(25, 100).unwrap().floor(), 150);
    assert_eq!(
        Spell::new(Range::new(25, 25).unwrap(), 0)
            .unwrap()
            .starting_damage(25, MAX_VALUE)
            .unwrap()
            .floor(),
        25
    );
}

#[test]
fn exact_fractions_survive_until_a_named_rounding_boundary() {
    let range = Range::new(0, 0).unwrap();
    let half = Weapon::new(range, 1000)
        .unwrap()
        .starting_damage(0, 7)
        .unwrap();
    let (n, d) = half.exact_ratio();
    assert_eq!(n * 2, d);
    assert_eq!(half.floor(), 0);
    assert_eq!(half.nearest_half_up(), 1);
    for (bp, floor, nearest) in [(4999, 0, 0), (5000, 0, 1), (5001, 0, 1), (15000, 1, 2)] {
        let amount = Spell::new(range, bp)
            .unwrap()
            .starting_damage(0, 1)
            .unwrap();
        assert_eq!(amount.floor(), floor);
        assert_eq!(amount.nearest_half_up(), nearest);
    }
}

#[test]
fn creature_and_zero_starting_amounts_do_not_require_player_equipment() {
    assert_eq!(Amount::fixed(5).unwrap().floor(), 5);
    assert_eq!(Amount::fixed(0).unwrap().exact_ratio(), (0, 1));
    let range = Range::new(0, 0).unwrap();
    assert_eq!(
        Weapon::new(range, 100)
            .unwrap()
            .starting_damage(0, 0)
            .unwrap()
            .floor(),
        0
    );
}

#[test]
fn invalid_ranges_rolls_stats_speeds_and_coefficients_fail_closed() {
    assert!(Range::new(100, 80).is_none());
    assert!(Range::new(0, MAX_VALUE + 1).is_none());
    let range = Range::new(80, 100).unwrap();
    for speed in [0, 99, 10_001, u32::MAX] {
        assert!(Weapon::new(range, speed).is_none());
    }
    let weapon = Weapon::new(range, 2800).unwrap();
    let spell = Spell::new(range, 6000).unwrap();
    for roll in [0, 79, 101, u32::MAX] {
        assert!(weapon.starting_damage(roll, 0).is_err());
        assert!(spell.starting_damage(roll, 0).is_err());
    }
    assert!(weapon.starting_damage(90, u32::MAX).is_err());
    assert!(spell.starting_damage(90, u32::MAX).is_err());
    assert!(Spell::new(range, MAX_COEFFICIENT_BP + 1).is_none());
    assert!(Amount::fixed(u32::MAX).is_err());
}

#[test]
fn result_bounds_are_checked_before_rounding_and_never_saturate() {
    let range = Range::new(MAX_VALUE, MAX_VALUE).unwrap();
    assert!(
        Weapon::new(range, 10_000)
            .unwrap()
            .starting_damage(MAX_VALUE, MAX_VALUE)
            .is_err()
    );
    assert!(
        Spell::new(range, MAX_COEFFICIENT_BP)
            .unwrap()
            .starting_damage(MAX_VALUE, MAX_VALUE)
            .is_err()
    );
    // A fractional amount just beyond the limit is invalid even if floor would fit.
    assert!(
        Weapon::new(range, 100)
            .unwrap()
            .starting_damage(MAX_VALUE, 1)
            .is_err()
    );
    assert_eq!(
        Amount::fixed(MAX_VALUE).unwrap().nearest_half_up(),
        MAX_VALUE
    );
}
