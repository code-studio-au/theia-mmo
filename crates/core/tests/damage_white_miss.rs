use theia_core::damage::white_melee::miss::{
    MAX_WEAPON_SKILL, Weapons, player_creature_research_v1 as resolve,
};
#[test]
fn confirmed_boss_example_separates_base_miss_and_first_hit_bonus_suppression() {
    for (bonus, expected) in [
        (0, 800),
        (50, 800),
        (100, 800),
        (101, 799),
        (800, 100),
        (899, 1),
        (900, 0),
    ] {
        let r = resolve(63, 300, bonus, Weapons::Single).unwrap();
        assert_eq!(r.defense_skill, 315);
        assert_eq!(r.skill_deficit, 15);
        assert_eq!(r.base_miss_bp, 800);
        assert_eq!(r.hit_suppression_bp, 100);
        assert_eq!(r.miss_bp, expected);
    }
}
#[test]
fn research_skill_breakpoints_are_explicit_at_ten_and_eleven_deficit() {
    for (skill, base, suppression) in [
        (315, 500, 0),
        (310, 550, 0),
        (305, 600, 0),
        (304, 720, 20),
        (303, 740, 40),
        (302, 760, 60),
        (301, 780, 80),
        (300, 800, 100),
    ] {
        let r = resolve(63, skill, 0, Weapons::Single).unwrap();
        assert_eq!((r.base_miss_bp, r.hit_suppression_bp), (base, suppression));
        assert_eq!(
            resolve(63, skill, base + suppression, Weapons::Single)
                .unwrap()
                .miss_bp,
            0
        );
        assert_eq!(
            resolve(63, skill, base + suppression - 1, Weapons::Single)
                .unwrap()
                .miss_bp,
            1
        );
    }
}
#[test]
fn dual_weapon_penalty_is_added_to_base_before_effective_hit_bonus() {
    assert_eq!(resolve(60, 300, 0, Weapons::Dual).unwrap().miss_bp, 2400);
    for (bonus, expected) in [(0, 2700), (2700, 100), (2800, 0)] {
        let r = resolve(63, 300, bonus, Weapons::Dual).unwrap();
        assert_eq!(r.dual_penalty_bp, 1900);
        assert_eq!(r.miss_bp, expected);
    }
}
#[test]
fn low_level_research_adjustment_does_not_install_raid_chances_on_level_one() {
    for level in 1..=10 {
        let r = resolve(level, u16::from(level) * 5, 0, Weapons::Single).unwrap();
        assert_eq!(r.base_miss_bp, u16::from(level) * 50);
        assert_eq!(r.hit_suppression_bp, 0);
    }
    assert_eq!(resolve(1, 5, 0, Weapons::Single).unwrap().miss_bp, 50);
    assert_eq!(resolve(1, 5, 0, Weapons::Dual).unwrap().miss_bp, 1950);
}
#[test]
fn strong_skill_and_hit_bonus_cannot_create_negative_or_wrapped_chances() {
    let strong = resolve(60, 400, 0, Weapons::Dual).unwrap();
    assert_eq!(strong.raw_base_bp, -500);
    assert_eq!(strong.base_miss_bp, 0);
    assert_eq!(strong.miss_bp, 1400);
    assert_eq!(resolve(60, 1000, 0, Weapons::Dual).unwrap().miss_bp, 0);
    for level in 1..=63 {
        for skill in [0, u16::from(level) * 5, MAX_WEAPON_SKILL] {
            for bonus in [0, 10_000] {
                for mode in [Weapons::Single, Weapons::Dual] {
                    let r = resolve(level, skill, bonus, mode).unwrap();
                    assert!(r.miss_bp <= 10_000);
                    assert!(r.effective_hit_bonus_bp <= bonus);
                    if skill == MAX_WEAPON_SKILL && mode == Weapons::Single {
                        assert_eq!(r.miss_bp, 0);
                    }
                }
            }
        }
    }
}
#[test]
fn unsupported_levels_skills_and_hit_magnitudes_fail_before_probability_math() {
    for level in [0, 64, 80, u8::MAX] {
        assert!(resolve(level, 300, 0, Weapons::Single).is_err());
    }
    for skill in [MAX_WEAPON_SKILL + 1, u16::MAX] {
        assert!(resolve(60, skill, 0, Weapons::Single).is_err());
    }
    for hit in [10_001, u16::MAX] {
        assert!(resolve(60, 300, hit, Weapons::Single).is_err());
    }
}
