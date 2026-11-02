use serde_json::{Value, json};
use theia_core::{
    abilities,
    damage::{
        self, MAX_COEFFICIENT_BP, MAX_VALUE, Range, Weapon,
        ability_starting::{
            ApSpeed, WeaponSpecial,
            catalog::{Catalog, MAX_PROFILE_BYTES, MAX_RECIPES},
        },
        stats,
    },
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-ability-starting.json");
fn weapon(speed: u32) -> Weapon {
    Weapon::new(Range::new(80, 100).unwrap(), speed).unwrap()
}
fn contexts() -> (damage::catalog::Catalog, abilities::Catalog, stats::Profile) {
    (
        damage::catalog::Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-physical-damage-definitions.json"
        ))
        .unwrap(),
        abilities::Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap(),
        stats::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-physical-damage-stats.json"
        ))
        .unwrap(),
    )
}
fn data() -> Value {
    serde_json::from_slice(DATA).unwrap()
}
fn parse(v: &Value) -> Result<Catalog, &'static str> {
    let (w, a, _) = contexts();
    Catalog::from_json(&serde_json::to_vec(v).unwrap(), &w, &a)
}
#[test]
fn normalization_changes_ap_only_and_does_not_mutate_weapon_or_white_swings() {
    let w = weapon(2800);
    let actual = WeaponSpecial::new(w, ApSpeed::weapon_base(), 10000, 5)
        .unwrap()
        .starting_damage(90, 140)
        .unwrap();
    let normalized = WeaponSpecial::new(w, ApSpeed::normalized(2400).unwrap(), 10000, 5)
        .unwrap()
        .starting_damage(90, 140)
        .unwrap();
    assert_eq!(actual.starting.floor(), 123);
    assert_eq!(normalized.weapon_with_ap.floor(), 114);
    assert_eq!(normalized.starting.floor(), 119);
    assert_eq!(normalized.ap_speed_ms, 2400);
    assert_eq!(w.base_speed_ms(), 2800);
    assert_eq!(w.range(), Range::new(80, 100).unwrap());
    assert_eq!(w.starting_damage(90, 140).unwrap().floor(), 118);
    // At zero AP, normalization cannot change the base weapon roll.
    for speed in [100, 1700, 2400, 2800, 3300, 10000] {
        assert_eq!(
            WeaponSpecial::new(w, ApSpeed::normalized(speed).unwrap(), 10000, 0)
                .unwrap()
                .starting_damage(90, 0)
                .unwrap()
                .starting
                .floor(),
            90
        );
    }
}
#[test]
fn percentage_precedes_flat_bonus_and_fractions_survive_until_caller_rounding() {
    let result = WeaponSpecial::new(weapon(2800), ApSpeed::weapon_base(), 15000, 5)
        .unwrap()
        .starting_damage(90, 140)
        .unwrap();
    assert_eq!(result.weapon_with_ap.floor(), 118);
    assert_eq!(result.scaled_weapon.floor(), 177);
    assert_eq!(result.starting.floor(), 182);
    let w = Weapon::new(Range::new(0, 0).unwrap(), 1000).unwrap();
    let result = WeaponSpecial::new(w, ApSpeed::weapon_base(), 5000, 1)
        .unwrap()
        .starting_damage(0, 7)
        .unwrap();
    assert_eq!(result.weapon_with_ap.exact_ratio(), (1, 2));
    assert_eq!(result.scaled_weapon.exact_ratio(), (1, 4));
    assert_eq!(result.starting.exact_ratio(), (5, 4));
    assert_eq!(result.starting.floor(), 1);
    assert_eq!(
        WeaponSpecial::new(w, ApSpeed::weapon_base(), 0, 7)
            .unwrap()
            .starting_damage(0, 0)
            .unwrap()
            .starting
            .floor(),
        7
    );
}
#[test]
fn invalid_rolls_speeds_scalars_and_each_overflow_stage_fail_closed() {
    for speed in [0, 99, 10001, u32::MAX] {
        assert!(ApSpeed::normalized(speed).is_none());
    }
    assert!(
        WeaponSpecial::new(
            weapon(2800),
            ApSpeed::weapon_base(),
            MAX_COEFFICIENT_BP + 1,
            0
        )
        .is_none()
    );
    assert!(
        WeaponSpecial::new(weapon(2800), ApSpeed::weapon_base(), 10000, MAX_VALUE + 1).is_none()
    );
    let s = WeaponSpecial::new(weapon(2800), ApSpeed::weapon_base(), 10000, 0).unwrap();
    for roll in [0, 79, 101, u32::MAX] {
        assert!(s.starting_damage(roll, 0).is_err());
    }
    assert!(s.starting_damage(90, MAX_VALUE + 1).is_err());
    let w = Weapon::new(Range::new(MAX_VALUE, MAX_VALUE).unwrap(), 10000).unwrap();
    assert!(
        WeaponSpecial::new(w, ApSpeed::weapon_base(), 0, 0)
            .unwrap()
            .starting_damage(MAX_VALUE, 1)
            .is_err()
    );
    assert!(
        WeaponSpecial::new(w, ApSpeed::weapon_base(), 10001, 0)
            .unwrap()
            .starting_damage(MAX_VALUE, 0)
            .is_err()
    );
    assert!(
        WeaponSpecial::new(w, ApSpeed::weapon_base(), 10000, 1)
            .unwrap()
            .starting_damage(MAX_VALUE, 0)
            .is_err()
    );
    assert_eq!(
        WeaponSpecial::new(w, ApSpeed::weapon_base(), 10000, 0)
            .unwrap()
            .starting_damage(MAX_VALUE, 0)
            .unwrap()
            .starting
            .floor(),
        MAX_VALUE
    );
}
#[test]
fn candidate_recipes_cover_both_abilities_and_use_only_their_declared_stat() {
    let (w, a, s) = contexts();
    let c = Catalog::from_json(DATA, &w, &a).unwrap();
    c.validate_stats(s.player()).unwrap();
    assert_eq!(c.id().as_str(), "theia.dev.ability-starting-fixture-v1");
    assert_eq!(c.recipes().len(), 2);
    let mut raw: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-physical-damage-stats.json"
    ))
    .unwrap();
    raw["player"]["spell_bonus_damage"] = json!(100);
    let spell_stats = stats::Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    for (id, r) in c.recipes() {
        if id.as_str() == "theia.dev.sword-slash" {
            assert_eq!(r.range(), Range::new(10, 12).unwrap());
            assert_eq!(
                r.starting_damage(11, s.player()).unwrap().exact_ratio(),
                (104, 5)
            );
            assert_eq!(
                r.starting_damage(11, s.player()).unwrap(),
                r.starting_damage(11, spell_stats.player()).unwrap()
            );
        } else {
            assert_eq!(r.range(), Range::new(18, 22).unwrap());
            assert_eq!(r.starting_damage(20, s.player()).unwrap().floor(), 20);
            assert_eq!(
                r.starting_damage(20, spell_stats.player()).unwrap().floor(),
                80
            );
        }
        assert!(
            r.starting_damage(r.range().maximum() + 1, s.player())
                .is_err()
        );
    }
}
#[test]
fn missing_null_unknown_and_unsupported_schema_fields_are_rejected() {
    for key in ["schema", "id", "ruleset", "weapon_definitions", "recipes"] {
        let mut d = data();
        d.as_object_mut().unwrap().remove(key);
        assert!(parse(&d).is_err(), "{key}");
        let mut d = data();
        d[key] = Value::Null;
        assert!(parse(&d).is_err(), "{key}");
    }
    for schema in [0, 2, u32::MAX] {
        let mut d = data();
        d["schema"] = json!(schema);
        assert!(parse(&d).is_err());
    }
    for index in 0..2 {
        for key in data()["recipes"][index].as_object().unwrap().keys() {
            let mut d = data();
            d["recipes"][index].as_object_mut().unwrap().remove(key);
            assert!(parse(&d).is_err(), "{key}");
            let mut d = data();
            d["recipes"][index][key] = Value::Null;
            assert!(parse(&d).is_err(), "{key}");
        }
        let mut d = data();
        d["recipes"][index]["unexpected"] = json!(true);
        assert!(parse(&d).is_err());
    }
    let mut d = data();
    d["unexpected"] = json!(true);
    assert!(parse(&d).is_err());
    let mut d = data();
    d["ruleset"] = json!("classic_exact");
    assert!(parse(&d).is_err());
    for v in [
        json!({"kind":"normalized"}),
        json!({"kind":"normalized","milliseconds":null}),
        json!({"kind":"weapon_base","milliseconds":2400}),
    ] {
        let mut d = data();
        d["recipes"][0]["ap_speed"] = v;
        assert!(parse(&d).is_err());
    }
}
#[test]
fn references_categories_duplicates_and_missing_coverage_are_checked() {
    for (key, val) in [
        ("weapon_definitions", "theia.test.other"),
        ("id", "../escape"),
    ] {
        let mut d = data();
        d[key] = json!(val);
        assert!(parse(&d).is_err());
    }
    for attack in ["theia.test.unknown", "theia.dev.wolf-bite"] {
        let mut d = data();
        d["recipes"][0]["weapon_attack"] = json!(attack);
        assert!(parse(&d).is_err());
    }
    for index in 0..2 {
        let mut d = data();
        d["recipes"][index]["ability"] = json!("theia.test.unknown");
        assert!(parse(&d).is_err());
    }
    let mut d = data();
    d["recipes"][0]["ability"] = json!("theia.dev.arcane-bolt");
    assert!(parse(&d).is_err());
    let mut d = data();
    d["recipes"][1]["ability"] = json!("theia.dev.sword-slash");
    assert!(parse(&d).is_err());
    let mut d = data();
    let duplicate = d["recipes"][1].clone();
    d["recipes"].as_array_mut().unwrap().push(duplicate);
    assert!(parse(&d).is_err());
    for index in 0..2 {
        let mut d = data();
        d["recipes"].as_array_mut().unwrap().remove(index);
        assert!(parse(&d).is_err());
    }
    let (w, a, _) = contexts();
    let mut abilities: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    abilities["schema"] = json!(6);
    for a in abilities["abilities"].as_array_mut().unwrap() {
        a.as_object_mut().unwrap().remove("combat");
    }
    assert!(
        Catalog::from_json(
            DATA,
            &w,
            &abilities::Catalog::from_json(&serde_json::to_vec(&abilities).unwrap()).unwrap()
        )
        .is_err()
    );
    assert_eq!(a.schema(), 7);
}
#[test]
fn byte_count_numeric_and_combined_stat_bounds_are_enforced_before_activation() {
    let (w, a, stats) = contexts();
    assert!(Catalog::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1], &w, &a).is_err());
    let mut d = data();
    d["recipes"] = json!([]);
    assert!(parse(&d).is_err());
    let mut d = data();
    let r = d["recipes"][1].clone();
    d["recipes"] = json!(vec![r; MAX_RECIPES + 1]);
    assert!(parse(&d).is_err());
    for (i, key, val) in [
        (0, "flat_bonus", MAX_VALUE + 1),
        (0, "weapon_percent_bp", MAX_COEFFICIENT_BP + 1),
        (1, "minimum", 23),
        (1, "maximum", MAX_VALUE + 1),
        (1, "coefficient_bp", MAX_COEFFICIENT_BP + 1),
    ] {
        let mut d = data();
        d["recipes"][i][key] = json!(val);
        assert!(parse(&d).is_err());
    }
    let mut d = data();
    d["recipes"][0]["flat_bonus"] = json!(MAX_VALUE);
    let c = parse(&d).unwrap();
    assert!(c.validate_stats(stats.player()).is_err());
    let mut d = data();
    d["recipes"][0]["ap_speed"] = json!({"kind":"normalized","milliseconds":3300});
    assert!(parse(&d).is_ok());
}
