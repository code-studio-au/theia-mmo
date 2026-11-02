use serde_json::{Value, json};
use theia_core::{
    abilities::{Catalog, Effect, MAX_ABILITIES, MAX_CATALOG_BYTES, Target, Timing},
    time::TickRate,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-abilities.json");
const ID: &str = "theia.dev.arcane-bolt";
#[test]
fn schema_four_requires_explicit_valid_equipment_for_sword_animation() {
    use theia_core::abilities::AnimationCue;
    let mut value = data();
    value["schema"] = json!(4);
    value["abilities"][0]["mana_cost"] = json!(0);
    value["abilities"][0]["requires_main_hand"] = json!("theia.dev.training-sword");
    value["abilities"][0]["animation"] = json!("sword_attack");
    let catalog = parse(&value).unwrap();
    let ability = catalog.ability(ID).unwrap();
    assert_eq!(
        ability.requires_main_hand().unwrap().as_str(),
        "theia.dev.training-sword"
    );
    assert_eq!(ability.animation(), Some(AnimationCue::SwordAttack));
    let items = theia_core::inventory::Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    assert!(catalog.validate_equipment(Some(&items)).is_ok());
    assert!(catalog.validate_equipment(None).is_err());
    for item in ["theia.dev.missing", "theia.dev.training-stone"] {
        let mut bad = value.clone();
        bad["abilities"][0]["requires_main_hand"] = json!(item);
        assert!(
            parse(&bad)
                .unwrap()
                .validate_equipment(Some(&items))
                .is_err()
        );
    }
    for schema in [1, 2, 3, 7] {
        let mut bad = value.clone();
        bad["schema"] = json!(schema);
        assert!(parse(&bad).is_err());
    }
    for bad in [json!("../sword"), json!(""), json!(true), Value::Null] {
        let mut bad_value = value.clone();
        bad_value["abilities"][0]["requires_main_hand"] = bad;
        assert!(parse(&bad_value).is_err());
    }
    value["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("requires_main_hand");
    assert!(parse(&value).is_err());
    value["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("animation");
    assert!(parse(&value).is_ok());
    for bytes in [
        DATA,
        include_bytes!("../../../content/gameplay/development-resource-abilities.json"),
        include_bytes!("../../../content/gameplay/development-status-abilities.json"),
    ] {
        assert!(
            Catalog::from_json(bytes)
                .unwrap()
                .abilities()
                .iter()
                .all(|a| a.requires_main_hand().is_none() && a.animation().is_none())
        );
    }
}
#[test]
fn status_references_require_explicit_schema_three_and_checked_ids() {
    let bytes = include_bytes!("../../../content/gameplay/development-status-abilities.json");
    let catalog = Catalog::from_json(bytes).unwrap();
    assert_eq!(catalog.schema(), 3);
    assert_eq!(
        catalog.abilities()[0].effect(),
        Effect::ApplyStatus {
            status: theia_core::content::ContentId::new("theia.dev.scorch").unwrap()
        }
    );
    let original: Value = serde_json::from_slice(bytes).unwrap();
    for schema in [1, 2] {
        let mut value = original.clone();
        value["schema"] = json!(schema);
        assert!(parse(&value).is_err());
    }
    for effect in [
        json!({"kind":"apply_status","status":"../scorch"}),
        json!({"kind":"apply_status","status":"theia.dev.scorch","amount":5}),
        json!({"kind":"apply_status"}),
    ] {
        let mut value = original.clone();
        value["abilities"][0]["effect"] = effect;
        assert!(parse(&value).is_err());
    }
    let mut missing = original;
    missing["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("mana_cost");
    assert!(parse(&missing).is_err());
}
fn data() -> Value {
    serde_json::from_slice(DATA).unwrap()
}
fn parse(value: &Value) -> Result<Catalog, String> {
    Catalog::from_json(&serde_json::to_vec(value).unwrap())
}
#[test]
fn real_development_ability_defaults_to_mobile_casting_and_simulation_time() {
    let catalog = Catalog::from_json(DATA).unwrap();
    assert_eq!(catalog.abilities().len(), 1);
    let ability = catalog.ability(ID).unwrap();
    assert_eq!(ability.id().as_str(), ID);
    assert_eq!(ability.target(), Target::HostileAlive);
    assert_eq!(ability.range_metres(), 30.);
    assert_eq!(ability.effect(), Effect::Damage { amount: 20 });
    assert!(ability.cast_while_moving() && ability.uses_global_cooldown());
    assert!(catalog.ability("theia.dev.missing").is_none());
    assert!(
        catalog
            .timing("theia.dev.missing", TickRate::Hz30)
            .is_none()
    );
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let hz = u64::from(rate.hz());
        assert_eq!(
            catalog.timing(ID, rate),
            Some(Timing {
                cast_ticks: hz,
                cooldown_ticks: 2 * hz,
                global_cooldown_ticks: hz * 3 / 2,
                queue_window_ticks: hz * 3 / 10,
            })
        );
    }
}
#[test]
fn instant_off_gcd_and_explicit_stationary_cast_rules_remain_distinct() {
    let mut value = data();
    value["abilities"][0]["cast_ms"] = json!(0);
    value["abilities"][0]["cooldown_ms"] = json!(0);
    value["abilities"][0]["cast_while_moving"] = json!(false);
    value["abilities"][0]["uses_global_cooldown"] = json!(false);
    value["queue_window_ms"] = json!(0);
    let catalog = parse(&value).unwrap();
    assert!(!catalog.ability(ID).unwrap().cast_while_moving());
    assert!(!catalog.ability(ID).unwrap().uses_global_cooldown());
    assert_eq!(
        catalog.timing(ID, TickRate::Hz30),
        Some(Timing {
            cast_ticks: 0,
            cooldown_ticks: 0,
            global_cooldown_ticks: 0,
            queue_window_ticks: 0,
        })
    );
    value["abilities"][0]["uses_global_cooldown"] = json!(true);
    value["global_cooldown_ms"] = json!(0);
    assert_eq!(
        parse(&value)
            .unwrap()
            .timing(ID, TickRate::Hz60)
            .unwrap()
            .global_cooldown_ticks,
        0
    );
}
#[test]
fn fractional_tick_durations_round_up_without_expiring_early() {
    let mut value = data();
    value["abilities"][0]["cast_ms"] = json!(1);
    value["abilities"][0]["cooldown_ms"] = json!(17);
    value["global_cooldown_ms"] = json!(1501);
    value["queue_window_ms"] = json!(301);
    let catalog = parse(&value).unwrap();
    for (rate, cooldown_ticks, global_cooldown_ticks, queue_window_ticks) in [
        (TickRate::Hz20, 1, 31, 7),
        (TickRate::Hz30, 1, 46, 10),
        (TickRate::Hz60, 2, 91, 19),
    ] {
        assert_eq!(
            catalog.timing(ID, rate),
            Some(Timing {
                cast_ticks: 1,
                cooldown_ticks,
                global_cooldown_ticks,
                queue_window_ticks,
            })
        );
    }
}
#[test]
fn unknown_content_invalid_names_duplicate_ids_and_count_or_byte_overflow_fail_closed() {
    for (path, bad) in [
        ("/schema", json!(2)),
        ("/abilities/0/id", json!("../spell")),
        ("/abilities/0/target", json!("anything")),
        ("/abilities/0/effect/kind", json!("heal")),
    ] {
        let mut value = data();
        *value.pointer_mut(path).unwrap() = bad;
        assert!(parse(&value).is_err(), "Accepted {path}: {value}");
    }
    for path in ["", "/abilities/0", "/abilities/0/effect"] {
        let mut value = data();
        value.pointer_mut(path).unwrap()["unexpected"] = json!(true);
        assert!(parse(&value).is_err(), "Unknown field accepted at {path}");
    }
    let mut value = data();
    let ability = value["abilities"][0].clone();
    value["abilities"] = json!([ability.clone(), ability.clone()]);
    assert!(parse(&value).is_err());
    value["abilities"] = json!([]);
    assert!(parse(&value).is_err());
    let abilities: Vec<_> = (0..=MAX_ABILITIES)
        .map(|i| {
            let mut item = ability.clone();
            item["id"] = json!(format!("theia.dev.spell-{i}"));
            item
        })
        .collect();
    value["abilities"] = json!(abilities);
    assert!(parse(&value).is_err());
    assert!(Catalog::from_json(&vec![b' '; MAX_CATALOG_BYTES + 1]).is_err());
    for malformed in [&DATA[..DATA.len() / 2], &[0xff], b"{} {}"] {
        assert!(Catalog::from_json(malformed).is_err());
    }
}
#[test]
fn numeric_type_finite_range_damage_and_timing_budgets_are_enforced() {
    for (path, bad) in [
        ("/abilities/0/range_metres", json!(0)),
        ("/abilities/0/range_metres", json!(-1)),
        ("/abilities/0/range_metres", json!(128.01)),
        ("/abilities/0/range_metres", Value::Null),
        ("/abilities/0/cast_ms", json!(60_001)),
        ("/abilities/0/cooldown_ms", json!(60_001)),
        ("/abilities/0/cast_ms", json!(-1)),
        ("/abilities/0/cast_ms", json!(0.5)),
        ("/abilities/0/cast_ms", json!(u64::MAX)),
        ("/global_cooldown_ms", json!(10_001)),
        ("/queue_window_ms", json!(1_001)),
        ("/abilities/0/effect/amount", json!(0)),
        ("/abilities/0/effect/amount", json!(-1)),
        ("/abilities/0/effect/amount", json!(1_000_001)),
    ] {
        let mut value = data();
        *value.pointer_mut(path).unwrap() = bad;
        assert!(parse(&value).is_err(), "Accepted invalid {path}: {value}");
    }
    let overflow = String::from_utf8(DATA.to_vec())
        .unwrap()
        .replace("30.0", "1e999");
    assert!(Catalog::from_json(overflow.as_bytes()).is_err());
    let mut value = data();
    value["abilities"][0]["range_metres"] = json!(128.0);
    value["abilities"][0]["cast_ms"] = json!(60_000);
    value["abilities"][0]["cooldown_ms"] = json!(60_000);
    value["abilities"][0]["effect"]["amount"] = json!(1_000_000);
    value["global_cooldown_ms"] = json!(10_000);
    value["queue_window_ms"] = json!(1_000);
    assert_eq!(
        parse(&value).unwrap().timing(ID, TickRate::Hz60),
        Some(Timing {
            cast_ticks: 3600,
            cooldown_ticks: 3600,
            global_cooldown_ticks: 600,
            queue_window_ticks: 60,
        })
    );
}

#[test]
fn explicit_schema_two_costs_preserve_schema_one_and_reject_ambiguous_or_invalid_costs() {
    let legacy = Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-abilities.json"
    ))
    .unwrap();
    assert_eq!(legacy.schema(), 1);
    assert_eq!(legacy.abilities()[0].mana_cost(), 0);
    let data = include_bytes!("../../../content/gameplay/development-resource-abilities.json");
    let resource = Catalog::from_json(data).unwrap();
    assert_eq!(resource.schema(), 2);
    assert_eq!(resource.abilities()[0].mana_cost(), 25);
    let baseline: serde_json::Value = serde_json::from_slice(data).unwrap();
    for bad in [
        serde_json::json!(-1),
        serde_json::json!(1.5),
        serde_json::json!(true),
        serde_json::json!(1_000_001),
        serde_json::Value::Null,
    ] {
        let mut value = baseline.clone();
        value["abilities"][0]["mana_cost"] = bad;
        assert!(Catalog::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut missing = baseline.clone();
    missing["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("mana_cost");
    assert!(Catalog::from_json(&serde_json::to_vec(&missing).unwrap()).is_err());
    let mut ambiguous = baseline;
    ambiguous["schema"] = serde_json::json!(1);
    assert!(Catalog::from_json(&serde_json::to_vec(&ambiguous).unwrap()).is_err());
}

#[test]
fn schema_five_checked_hit_tables_apply_only_to_direct_damage_and_preserve_older_catalogues() {
    let bytes = include_bytes!("../../../content/gameplay/development-melee-abilities.json");
    let old = Catalog::from_json(bytes).unwrap();
    assert_eq!(old.schema(), 4);
    assert!(old.abilities().iter().all(|a| a.hit_table().is_none()));
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    value["schema"] = json!(5);
    value["abilities"][1]["hit_table"] = json!({"hit_bp":8500,"miss_bp":1000,"deflect_bp":500});
    let catalog = parse(&value).unwrap();
    let table = catalog.abilities()[1].hit_table().unwrap();
    assert_eq!(
        (table.hit_bp(), table.miss_bp(), table.deflect_bp()),
        (8500, 1000, 500)
    );
    assert!(catalog.abilities()[0].hit_table().is_none());
    let mut legacy = value.clone();
    legacy["schema"] = json!(4);
    assert!(parse(&legacy).is_err());
    let mut status = value.clone();
    status["abilities"][0]["hit_table"] = value["abilities"][1]["hit_table"].clone();
    assert!(parse(&status).is_err());
    for bad in [
        Value::Null,
        json!([]),
        json!({"hit_bp":8500,"miss_bp":1000,"deflect_bp":501}),
    ] {
        let mut invalid = value.clone();
        invalid["abilities"][1]["hit_table"] = bad;
        assert!(parse(&invalid).is_err());
    }
}

#[test]
fn schema_six_travel_requires_direct_damage_and_can_cover_declared_launch_range() {
    let bytes = include_bytes!("../../../content/gameplay/development-hit-abilities.json");
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    value["schema"] = json!(6);
    let travel = json!({"speed_metres_per_second":20.,"maximum_distance_metres":45.,"lifetime_ms":3000,"target_radius_metres":0.2});
    value["abilities"][1]["projectile"] = travel.clone();
    let parsed = parse(&value).unwrap();
    assert_eq!(parsed.schema(), 6);
    assert_eq!(
        parsed.abilities()[1]
            .projectile()
            .unwrap()
            .maximum_distance_metres(),
        45.
    );
    assert!(parsed.abilities()[0].projectile().is_none());
    for schema in 1..=5 {
        let mut old = value.clone();
        old["schema"] = json!(schema);
        assert!(parse(&old).is_err());
    }
    let mut status = value.clone();
    status["abilities"][0]["projectile"] = travel;
    assert!(parse(&status).is_err());
    for invalid in [Value::Null, json!({}), json!([])] {
        let mut bad = value.clone();
        bad["abilities"][1]["projectile"] = invalid;
        assert!(parse(&bad).is_err());
    }
    for field in ["maximum_distance_metres", "speed_metres_per_second"] {
        let mut bad = value.clone();
        bad["abilities"][1]["projectile"][field] = json!(0.5);
        assert!(parse(&bad).is_err());
    }
    for bytes in [
        DATA,
        include_bytes!("../../../content/gameplay/development-resource-abilities.json"),
        include_bytes!("../../../content/gameplay/development-status-abilities.json"),
        include_bytes!("../../../content/gameplay/development-melee-abilities.json"),
        include_bytes!("../../../content/gameplay/development-hit-abilities.json"),
    ] {
        assert!(
            Catalog::from_json(bytes)
                .unwrap()
                .abilities()
                .iter()
                .all(|a| a.projectile().is_none())
        );
    }
}
