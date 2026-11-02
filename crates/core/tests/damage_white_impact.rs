use theia_core::damage::{
    Amount, MAX_VALUE,
    white_melee::{
        Outcome,
        impact::{GlancePercent, GlanceRange, Impact, physical_reference_v1 as resolve},
    },
};
fn fixed(n: u32) -> Amount {
    Amount::fixed(n).unwrap()
}
#[test]
fn selected_outcomes_preserve_armour_stage_and_distinguish_avoidance_from_block() {
    for (impact, expected) in [
        (Impact::Miss, 0),
        (Impact::Dodge, 0),
        (Impact::Parry, 0),
        (Impact::Hit, 12),
        (Impact::Critical, 24),
        (Impact::Crushing, 18),
        (Impact::Block { value: 5 }, 7),
        (Impact::Glancing(GlancePercent::new(65).unwrap()), 7),
    ] {
        let r = resolve(fixed(15), 1, 100, 0, 100, impact).unwrap();
        assert_eq!(r.outcome, impact.outcome());
        assert_eq!(r.resolved_damage, expected);
        assert_eq!(r.health_lost, expected);
        if matches!(impact, Impact::Miss | Impact::Dodge | Impact::Parry) {
            assert!(r.armour.is_none());
        } else {
            let a = r.armour.unwrap();
            assert_eq!((a.effective, a.prevented, a.remaining), (100, 3, 12));
        }
        assert_eq!(
            r.blocked,
            if matches!(impact, Impact::Block { .. }) {
                5
            } else {
                0
            }
        );
    }
}
#[test]
fn reference_floor_is_applied_before_outcome_scaling_not_after_recalculated_armour() {
    let r = resolve(fixed(2), 1, 100, 0, 100, Impact::Critical).unwrap();
    assert_eq!(r.armour.unwrap().remaining, 1);
    assert_eq!(r.resolved_damage, 2);
    let multiplied_first = resolve(fixed(4), 1, 100, 0, 100, Impact::Hit).unwrap();
    assert_eq!(multiplied_first.resolved_damage, 3);
    let crush = resolve(fixed(3), 1, 0, 0, 100, Impact::Crushing).unwrap();
    assert_eq!(crush.resolved_damage, 4);
}
#[test]
fn killing_critical_reports_full_damage_separately_from_health_loss_and_overkill() {
    let r = resolve(fixed(15), 1, 100, 0, 3, Impact::Critical).unwrap();
    assert_eq!(
        (
            r.outcome_damage,
            r.resolved_damage,
            r.health_lost,
            r.overkill
        ),
        (24, 24, 3, 21)
    );
    assert_eq!(r.outcome, Outcome::Critical);
}
#[test]
fn full_block_and_small_glance_can_reduce_positive_normal_contact_to_zero() {
    let block = resolve(fixed(1), 1, MAX_VALUE, 0, 100, Impact::Block { value: 100 }).unwrap();
    assert_eq!(block.armour.unwrap().remaining, 1);
    assert_eq!((block.blocked, block.resolved_damage), (1, 0));
    let glance = resolve(
        fixed(1),
        1,
        0,
        0,
        100,
        Impact::Glancing(GlancePercent::new(99).unwrap()),
    )
    .unwrap();
    assert_eq!(glance.resolved_damage, 0);
    for impact in [
        Impact::Hit,
        Impact::Critical,
        Impact::Crushing,
        Impact::Block { value: 1 },
        Impact::Glancing(GlancePercent::new(50).unwrap()),
    ] {
        assert_eq!(
            resolve(fixed(0), 1, 0, 0, 100, impact)
                .unwrap()
                .resolved_damage,
            0
        );
    }
}
#[test]
fn glance_damage_uses_uncapped_skill_bonus_independently_of_chance_cap() {
    for (skill, low, high) in [
        (300, 55, 75),
        (305, 80, 90),
        (310, 91, 99),
        (315, 91, 99),
        (316, 100, 100),
    ] {
        let r = GlanceRange::melee_reference_v1(skill, 315).unwrap();
        assert_eq!((r.minimum(), r.maximum()), (low, high));
        assert!(r.pick(low).is_some());
        assert!(r.pick(high).is_some());
        assert!(r.pick(low - 1).is_none());
        assert!(r.pick(high + 1).is_none());
    }
}
#[test]
fn glance_ranges_stay_ordered_and_checked_over_extreme_skill_differences() {
    for skill in [0, 1, 300, 315, 1000] {
        for defense in 0..=1000 {
            let r = GlanceRange::melee_reference_v1(skill, defense).unwrap();
            assert!(1 <= r.minimum() && r.minimum() <= r.maximum() && r.maximum() <= 100);
        }
    }
    assert!(GlancePercent::new(0).is_none());
    assert!(GlancePercent::new(101).is_none());
    assert!(GlanceRange::melee_reference_v1(1001, 315).is_err());
    assert!(GlanceRange::melee_reference_v1(300, 1001).is_err());
}
#[test]
fn malformed_defenses_or_levels_fail_even_for_avoidance_and_overflow_never_clips_to_health() {
    for impact in [Impact::Miss, Impact::Hit] {
        assert!(resolve(fixed(15), 0, 0, 0, 100, impact).is_err());
        assert!(resolve(fixed(15), 64, 0, 0, 100, impact).is_err());
        assert!(resolve(fixed(15), 1, MAX_VALUE + 1, 0, 100, impact).is_err());
        assert!(resolve(fixed(15), 1, 0, MAX_VALUE + 1, 100, impact).is_err());
        assert!(resolve(fixed(15), 1, 0, 0, MAX_VALUE + 1, impact).is_err());
    }
    assert!(
        resolve(
            fixed(15),
            1,
            0,
            0,
            100,
            Impact::Block {
                value: MAX_VALUE + 1
            }
        )
        .is_err()
    );
    assert!(resolve(fixed(500001), 1, 0, 0, 1, Impact::Critical).is_err());
    assert!(resolve(fixed(MAX_VALUE), 1, 0, 0, 1, Impact::Crushing).is_err());
}
