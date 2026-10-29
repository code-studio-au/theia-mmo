use serde_json::{Value, json};
use theia_core::{
    abilities::{Catalog, Effect, MAX_ABILITIES, MAX_CATALOG_BYTES, Target, Timing},
    time::TickRate,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-abilities.json");
const ID: &str = "theia.dev.arcane-bolt";
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
