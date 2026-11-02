use serde_json::{Value, json};
use theia_core::{
    abilities::{Catalog, classic::Swing},
    time::TickRate,
};
fn profile() -> Value {
    let mut data: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    data["schema"] = json!(7);
    for ability in data["abilities"].as_array_mut().unwrap() {
        ability["combat"] = json!({"minimum_range_metres":0.0,"facing":{"kind":"frontal","arc_degrees":180.0},"autoattack":{"kind":"continue"}});
    }
    data
}
fn parse(data: &Value) -> Result<Catalog, String> {
    Catalog::from_json(&serde_json::to_vec(data).unwrap())
}
#[test]
fn explicit_rules_preserve_distinct_timer_policies_and_metric_ranges() {
    for (raw, expected) in [
        (json!({"kind":"continue"}), Swing::Continue),
        (json!({"kind":"suspend"}), Swing::Suspend),
        (
            json!({"kind":"delay","delay_ms":125}),
            Swing::Delay { delay_ms: 125 },
        ),
        (json!({"kind":"reset"}), Swing::Reset),
    ] {
        let mut data = profile();
        data["abilities"][0]["combat"]["autoattack"] = raw;
        let catalog = parse(&data).unwrap();
        let rules = catalog.abilities()[0].combat_rules().unwrap();
        assert_eq!(rules.autoattack(), expected);
        assert_eq!(rules.minimum_range_metres(), 0.0);
        assert!(rules.arc().is_some());
        for (rate, delay) in [
            (TickRate::Hz20, 3),
            (TickRate::Hz30, 4),
            (TickRate::Hz60, 8),
        ] {
            assert_eq!(
                rules.autoattack().delay_ticks(rate),
                if matches!(expected, Swing::Delay { .. }) {
                    Some(delay)
                } else {
                    None
                }
            );
        }
    }
    let mut data = profile();
    data["abilities"][0]["combat"]["minimum_range_metres"] = json!(5.0);
    data["abilities"][0]["combat"]["facing"] = json!({"kind":"any"});
    let catalog = parse(&data).unwrap();
    let rules = catalog.abilities()[0].combat_rules().unwrap();
    assert_eq!(rules.minimum_range_metres(), 5.0);
    assert!(rules.arc().is_none());
}
#[test]
fn missing_null_unknown_and_legacy_combat_rules_fail_closed() {
    for version in 1..=6 {
        let mut data = profile();
        data["schema"] = json!(version);
        assert!(parse(&data).is_err());
    }
    let mut data = profile();
    data["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("combat");
    assert!(parse(&data).is_err());
    let mut data = profile();
    data["abilities"][0]["combat"] = Value::Null;
    assert!(parse(&data).is_err());
    for field in ["minimum_range_metres", "facing", "autoattack"] {
        let mut data = profile();
        data["abilities"][0]["combat"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(parse(&data).is_err());
        let mut data = profile();
        data["abilities"][0]["combat"][field] = Value::Null;
        assert!(parse(&data).is_err());
    }
    let mut data = profile();
    data["abilities"][0]["combat"]["autoattack"] = json!({"kind":"continue","delay_ms":1});
    assert!(parse(&data).is_err());
    assert!(
        Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-projectile-abilities.json"
        ))
        .is_ok()
    );
}
#[test]
fn minimum_arc_and_delay_boundaries_are_checked_before_authority() {
    for minimum in [-0.1, 30.0, 31.0] {
        let mut data = profile();
        data["abilities"][0]["combat"]["minimum_range_metres"] = json!(minimum);
        assert!(parse(&data).is_err());
    }
    for degrees in [0.0, -1.0, 360.1] {
        let mut data = profile();
        data["abilities"][0]["combat"]["facing"]["arc_degrees"] = json!(degrees);
        assert!(parse(&data).is_err());
    }
    for delay_ms in [0, 10_001, u32::MAX] {
        let mut data = profile();
        data["abilities"][0]["combat"]["autoattack"] = json!({"kind":"delay","delay_ms":delay_ms});
        assert!(parse(&data).is_err());
    }
    for delay_ms in [1, 10_000] {
        let mut data = profile();
        data["abilities"][0]["combat"]["autoattack"] = json!({"kind":"delay","delay_ms":delay_ms});
        assert!(parse(&data).is_ok());
    }
}
