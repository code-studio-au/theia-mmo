use serde_json::{Value, json};
use theia_core::{
    abilities,
    content::ContentId,
    damage::catalog::{
        Attack, Catalog, MAX_ATTACKS, MAX_CREATURES, MAX_PROFILE_BYTES, MAX_SPELLS, MAX_WEAPONS,
    },
    inventory,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-damage-definitions.json");
fn profile() -> Value {
    serde_json::from_slice(DATA).unwrap()
}
fn parse(value: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(value).unwrap())
}
fn known() -> (inventory::Catalog, abilities::Catalog, Vec<ContentId>) {
    (
        inventory::Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap(),
        abilities::Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap(),
        vec![ContentId::new("theia.dev.training-creature").unwrap()],
    )
}
#[test]
fn candidate_definitions_preserve_weapon_spell_and_creature_arithmetic() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let (items, abilities, creatures) = known();
    assert_eq!(
        catalog.id().as_str(),
        "theia.dev.damage-numerical-fixture-v1"
    );
    catalog
        .validate_references(&items, &abilities, &creatures)
        .unwrap();
    let mut seen = 0;
    for (_, attack) in catalog.attacks() {
        match attack {
            Attack::Weapon { damage, .. } => {
                assert_eq!(damage.starting_damage(90, 140).unwrap().floor(), 118)
            }
            Attack::Spell { damage, .. } => {
                assert_eq!(damage.starting_damage(200, 100).unwrap().floor(), 260)
            }
            Attack::Creature { damage, .. } => {
                for roll in 4..=6 {
                    assert_eq!(damage.starting_damage(roll).unwrap().floor(), roll);
                }
                assert!(damage.starting_damage(3).is_err());
                assert!(damage.starting_damage(7).is_err());
            }
        }
        seen += 1;
    }
    assert_eq!(seen, 3);
}
#[test]
fn identity_and_reference_duplicates_are_rejected_independently() {
    let mut data = profile();
    data["attacks"][1]["id"] = data["attacks"][0]["id"].clone();
    assert!(parse(&data).is_err());
    for index in 0..3 {
        let mut data = profile();
        let mut duplicate = data["attacks"][index].clone();
        duplicate["id"] = json!("theia.test.unique-duplicate");
        data["attacks"].as_array_mut().unwrap().push(duplicate);
        assert!(parse(&data).is_err());
    }
    for value in ["../escape", "InvalidName", "", "theia..invalid"] {
        let mut data = profile();
        data["attacks"][0]["id"] = json!(value);
        assert!(parse(&data).is_err());
        let mut data = profile();
        data["attacks"][0]["item"] = json!(value);
        assert!(parse(&data).is_err());
    }
}
#[test]
fn schemas_nulls_missing_and_unknown_fields_never_silently_change_a_definition() {
    for version in [0, 2, u32::MAX] {
        let mut data = profile();
        data["schema"] = json!(version);
        assert!(parse(&data).is_err());
    }
    for key in ["schema", "id", "attacks"] {
        let mut data = profile();
        data.as_object_mut().unwrap().remove(key);
        assert!(parse(&data).is_err());
        let mut data = profile();
        data[key] = Value::Null;
        assert!(parse(&data).is_err());
    }
    for index in 0..3 {
        let original = profile()["attacks"][index]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for key in original {
            let mut data = profile();
            data["attacks"][index].as_object_mut().unwrap().remove(&key);
            assert!(parse(&data).is_err());
            let mut data = profile();
            data["attacks"][index][&key] = Value::Null;
            assert!(parse(&data).is_err());
        }
        let mut data = profile();
        data["attacks"][index]["unexpected"] = json!(true);
        assert!(parse(&data).is_err());
    }
    let mut data = profile();
    data["unexpected"] = json!(true);
    assert!(parse(&data).is_err());
}
#[test]
fn per_category_total_and_byte_budgets_fail_before_adoption() {
    assert!(Catalog::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1]).is_err());
    let mut data = profile();
    data["attacks"] = json!([]);
    assert!(parse(&data).is_err());
    for (index, limit, key) in [
        (0, MAX_WEAPONS, "item"),
        (1, MAX_SPELLS, "ability"),
        (2, MAX_CREATURES, "creature"),
    ] {
        let original = profile()["attacks"][index].clone();
        let mut data = profile();
        data["attacks"] = Value::Array(
            (0..=limit)
                .map(|n| {
                    let mut a = original.clone();
                    a["id"] = json!(format!("theia.test.attack-{n}"));
                    a[key] = json!(format!("theia.test.reference-{n}"));
                    a
                })
                .collect(),
        );
        assert!(parse(&data).is_err());
        data["attacks"].as_array_mut().unwrap().pop();
        assert!(parse(&data).is_ok());
    }
    let mut data = profile();
    let template = data["attacks"].clone();
    data["attacks"] = Value::Array(
        (0..=MAX_ATTACKS)
            .map(|n| {
                let i = if n < 32 {
                    0
                } else if n < 64 {
                    1
                } else {
                    2
                };
                let mut a = template[i].clone();
                a["id"] = json!(format!("theia.test.attack-{n}"));
                a[["item", "ability", "creature"][i]] = json!(format!("theia.test.reference-{n}"));
                a
            })
            .collect(),
    );
    assert!(parse(&data).is_err());
    data["attacks"].as_array_mut().unwrap().pop();
    assert!(parse(&data).is_ok());
}
#[test]
fn numerical_bounds_are_rechecked_when_loading_untrusted_definition_bytes() {
    for (index, key, value) in [
        (0, "minimum", json!(101)),
        (0, "maximum", json!(1_000_001)),
        (0, "base_speed_ms", json!(99)),
        (0, "base_speed_ms", json!(10_001)),
        (1, "coefficient_bp", json!(100_001)),
        (1, "minimum", json!(-1)),
        (2, "maximum", json!(3)),
    ] {
        let mut data = profile();
        data["attacks"][index][key] = value;
        assert!(parse(&data).is_err());
    }
}
#[test]
fn missing_and_incompatible_references_fail_without_falling_back_to_fixed_damage() {
    let (items, abilities, creatures) = known();
    for (index, key) in [(0, "item"), (1, "ability"), (2, "creature")] {
        let mut data = profile();
        data["attacks"][index][key] = json!("theia.test.missing");
        let catalog = parse(&data).unwrap();
        assert!(
            catalog
                .validate_references(&items, &abilities, &creatures)
                .is_err()
        );
    }
    let mut data = profile();
    data["attacks"][0]["item"] = json!("theia.dev.training-stone");
    assert!(
        parse(&data)
            .unwrap()
            .validate_references(&items, &abilities, &creatures)
            .is_err()
    );
    let mut raw: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    raw["abilities"][0]["effect"] = json!({"kind":"apply_status","status":"theia.test.status"});
    raw["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("hit_table");
    raw["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("projectile");
    let incompatible = abilities::Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert!(
        Catalog::from_json(DATA)
            .unwrap()
            .validate_references(&items, &incompatible, &creatures)
            .is_err()
    );
    let mut raw: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    raw["abilities"][0]["requires_main_hand"] = json!("theia.test.unknown-wand");
    let invalid_equipment =
        abilities::Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert!(
        Catalog::from_json(DATA)
            .unwrap()
            .validate_references(&items, &invalid_equipment, &creatures)
            .is_err()
    );
    assert!(
        Catalog::from_json(DATA)
            .unwrap()
            .validate_references(&items, &abilities, &[])
            .is_err()
    );
}
#[test]
fn definition_order_cannot_change_logical_lookup() {
    let a = Catalog::from_json(DATA).unwrap();
    let mut data = profile();
    data["attacks"].as_array_mut().unwrap().reverse();
    let b = parse(&data).unwrap();
    assert_eq!(
        a.attacks().collect::<Vec<_>>(),
        b.attacks().collect::<Vec<_>>()
    );
    for (id, attack) in a.attacks() {
        assert_eq!(b.attack(id), Some(attack));
    }
}
