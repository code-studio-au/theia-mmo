use theia_core::damage::white_melee::chances::*;
fn fives() -> BaseDefences {
    BaseDefences::new([500; 3]).unwrap()
}
#[test]
fn creature_defensive_reference_matches_equal_and_plus_three_examples() {
    let equal = creature_defences_reference_v1(fives(), 300, 300).unwrap();
    assert_eq!(
        (equal.dodge_bp, equal.parry_bp, equal.block_bp),
        (500, 500, 500)
    );
    let boss = creature_defences_reference_v1(fives(), 300, 315).unwrap();
    assert_eq!(
        (boss.dodge_bp, boss.parry_bp, boss.block_bp),
        (650, 1400, 500)
    );
    let lower = creature_defences_reference_v1(fives(), 305, 300).unwrap();
    assert_eq!(
        (lower.dodge_bp, lower.parry_bp, lower.block_bp),
        (480, 480, 480)
    );
}
#[test]
fn creature_parry_reference_breakpoint_is_explicit_and_block_does_not_grow() {
    for (gap, parry) in [(5, 550), (10, 600), (11, 1160), (15, 1400)] {
        let r = creature_defences_reference_v1(fives(), 300, 300 + gap).unwrap();
        assert_eq!(r.parry_bp, parry);
        assert_eq!(r.block_bp, 500);
    }
}
#[test]
fn player_defense_uses_its_own_skill_factor_and_preserves_absent_chances() {
    let player = player_defences_reference_v1(fives(), 315, 300).unwrap();
    assert_eq!(
        (player.dodge_bp, player.parry_bp, player.block_bp),
        (440, 440, 440)
    );
    for rates in [
        player_defences_reference_v1(BaseDefences::new([0; 3]).unwrap(), 0, 1000).unwrap(),
        creature_defences_reference_v1(BaseDefences::new([0; 3]).unwrap(), 0, 1000).unwrap(),
    ] {
        assert_eq!((rates.dodge_bp, rates.parry_bp, rates.block_bp), (0, 0, 0));
    }
}
#[test]
fn melee_glancing_caps_bonus_skill_for_chance_but_retains_training_deficits() {
    for (level, expected) in [(60, 1000), (61, 2000), (62, 3000), (63, 4000)] {
        assert_eq!(
            player_melee_glancing_research_v1(60, 300, level).unwrap(),
            expected
        );
    }
    assert_eq!(
        player_melee_glancing_research_v1(60, 305, 63).unwrap(),
        4000
    );
    assert_eq!(
        player_melee_glancing_research_v1(60, 295, 63).unwrap(),
        5000
    );
    assert_eq!(
        player_melee_glancing_research_v1(60, 0, 63).unwrap(),
        10_000
    );
}
#[test]
fn crushing_starts_at_fifteen_deficit_and_uses_capped_defense() {
    assert_eq!(creature_crushing_reference_v1(314, 60, 300).unwrap(), 0);
    assert_eq!(creature_crushing_reference_v1(315, 60, 300).unwrap(), 1500);
    assert_eq!(creature_crushing_reference_v1(316, 60, 300).unwrap(), 1700);
    assert_eq!(creature_crushing_reference_v1(315, 60, 320).unwrap(), 1500);
    assert_eq!(creature_crushing_reference_v1(315, 60, 295).unwrap(), 2500);
    assert_eq!(creature_crushing_reference_v1(1000, 1, 0).unwrap(), 10_000);
}
#[test]
fn isolated_critical_adjustments_do_not_invent_class_auras_or_lower_level_bonus() {
    assert_eq!(
        higher_creature_critical_reference_v1(400, 60, 63).unwrap(),
        100
    );
    assert_eq!(
        higher_creature_critical_reference_v1(200, 60, 63).unwrap(),
        0
    );
    assert_eq!(
        higher_creature_critical_reference_v1(400, 60, 60).unwrap(),
        400
    );
    assert!(higher_creature_critical_reference_v1(400, 60, 59).is_err());
    assert_eq!(creature_critical_reference_v1(500, 315, 300).unwrap(), 560);
    assert_eq!(creature_critical_reference_v1(500, 300, 315).unwrap(), 440);
    assert_eq!(creature_critical_reference_v1(0, 1000, 0).unwrap(), 0);
}
#[test]
fn bounded_defensive_inputs_cannot_wrap_or_create_rates_outside_the_roll_domain() {
    for skill in [0, 315, 1000] {
        for defense in [0, 315, 1000] {
            for base in [0, 1, 500, 10_000] {
                for r in [
                    player_defences_reference_v1(
                        BaseDefences::new([base; 3]).unwrap(),
                        skill,
                        defense,
                    )
                    .unwrap(),
                    creature_defences_reference_v1(
                        BaseDefences::new([base; 3]).unwrap(),
                        skill,
                        defense,
                    )
                    .unwrap(),
                ] {
                    assert!(r.dodge_bp <= 10_000 && r.parry_bp <= 10_000 && r.block_bp <= 10_000);
                }
            }
        }
    }
}
#[test]
fn unsupported_levels_skills_and_base_probabilities_fail_before_adjustment() {
    assert!(BaseDefences::new([10_001, 0, 0]).is_none());
    assert!(player_defences_reference_v1(fives(), 1001, 300).is_err());
    assert!(creature_defences_reference_v1(fives(), 300, 1001).is_err());
    for level in [0, 64, 80, u8::MAX] {
        assert!(player_melee_glancing_research_v1(level, 300, 60).is_err());
        assert!(player_melee_glancing_research_v1(60, 300, level).is_err());
        assert!(creature_crushing_reference_v1(315, level, 300).is_err());
        assert!(higher_creature_critical_reference_v1(500, level, 63).is_err());
    }
    assert!(player_melee_glancing_research_v1(60, 1001, 63).is_err());
    assert!(creature_crushing_reference_v1(1001, 60, 300).is_err());
    assert!(higher_creature_critical_reference_v1(10_001, 60, 63).is_err());
    assert!(creature_critical_reference_v1(10_001, 315, 300).is_err());
}

#[test]
fn creature_player_miss_uses_defense_difference_without_creature_target_suppression() {
    assert_eq!(creature_miss_reference_v1(500, 300, 315).unwrap(), 560);
    assert_eq!(creature_miss_reference_v1(500, 315, 300).unwrap(), 440);
    assert_eq!(creature_miss_reference_v1(0, 0, 1000).unwrap(), 0);
    assert!(creature_miss_reference_v1(10001, 300, 300).is_err());
    assert!(creature_miss_reference_v1(500, 1001, 300).is_err());
}
