use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    damage::{
        MAX_VALUE,
        catalog::{Attack, Catalog},
        physical::normal_reference_v1,
        stats::{MAX_CREATURES, MAX_PROFILE_BYTES, Profile, Ruleset},
    },
};
const STATS: &[u8] = include_bytes!("../../../content/gameplay/development-damage-stats.json");
const ATTACKS: &[u8] =
    include_bytes!("../../../content/gameplay/development-damage-definitions.json");
fn value() -> Value {
    serde_json::from_slice(STATS).unwrap()
}
fn parse(data: &Value) -> Result<Profile, &'static str> {
    Profile::from_json(&serde_json::to_vec(data).unwrap())
}
#[test]
fn explicit_templates_share_the_reference_calculation_in_both_directions() {
    let stats = Profile::from_json(STATS).unwrap();
    let catalog = Catalog::from_json(ATTACKS).unwrap();
    stats.validate_definitions(&catalog).unwrap();
    assert_eq!(stats.id().as_str(), "theia.dev.damage-numerical-stats-v1");
    assert_eq!(stats.ruleset(), Ruleset::PhysicalReferenceV1);
    let player = stats.player();
    let creature = stats
        .creature(&ContentId::new("theia.dev.training-creature").unwrap())
        .unwrap();
    assert_eq!(player.physical().level(), 60);
    assert_eq!(player.attack_power(), 140);
    assert_eq!(player.spell_bonus_damage(), 100);
    assert_eq!(player.physical().armour(), 5500);
    assert_eq!(player.physical().armour_penetration(), 0);
    for (_, attack) in catalog.attacks() {
        match attack {
            Attack::Weapon { damage, .. } => {
                let amount = damage.starting_damage(90, player.attack_power()).unwrap();
                assert_eq!(
                    normal_reference_v1(
                        amount,
                        player.physical().level(),
                        creature.armour(),
                        player.physical().armour_penetration(),
                        100
                    )
                    .unwrap()
                    .health_lost,
                    59
                );
            }
            Attack::Creature { damage, .. } => {
                let amount = damage.starting_damage(5).unwrap();
                assert_eq!(
                    normal_reference_v1(
                        amount,
                        creature.level(),
                        player.physical().armour(),
                        creature.armour_penetration(),
                        100
                    )
                    .unwrap()
                    .health_lost,
                    2
                );
            }
            Attack::Spell { damage, .. } => assert_eq!(
                damage
                    .starting_damage(200, player.spell_bonus_damage())
                    .unwrap()
                    .floor(),
                260
            ),
        }
    }
}
#[test]
fn combined_stat_and_definition_magnitudes_fail_before_live_adoption() {
    let stats = Profile::from_json(STATS).unwrap();
    for index in [0, 1] {
        let mut raw: Value = serde_json::from_slice(ATTACKS).unwrap();
        raw["attacks"][index]["maximum"] = json!(MAX_VALUE);
        let catalog = Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(stats.validate_definitions(&catalog).is_err());
    }
    let mut raw = value();
    raw["player"]["spell_bonus_damage"] = json!(MAX_VALUE);
    let stats = parse(&raw).unwrap();
    let mut defs: Value = serde_json::from_slice(ATTACKS).unwrap();
    defs["attacks"][1]["coefficient_bp"] = json!(100000);
    let catalog = Catalog::from_json(&serde_json::to_vec(&defs).unwrap()).unwrap();
    assert!(stats.validate_definitions(&catalog).is_err());
}
#[test]
fn creature_coverage_is_exact_and_not_an_implicit_default() {
    let catalog = Catalog::from_json(ATTACKS).unwrap();
    let mut raw = value();
    raw["creatures"] = json!([]);
    assert!(parse(&raw).unwrap().validate_definitions(&catalog).is_err());
    let mut raw = value();
    raw["creatures"][0]["creature"] = json!("theia.test.unknown");
    assert!(parse(&raw).unwrap().validate_definitions(&catalog).is_err());
    let mut raw = value();
    let mut extra = raw["creatures"][0].clone();
    extra["creature"] = json!("theia.test.unused");
    raw["creatures"].as_array_mut().unwrap().push(extra);
    assert!(parse(&raw).unwrap().validate_definitions(&catalog).is_err());
    assert!(
        Profile::from_json(STATS)
            .unwrap()
            .creature(&ContentId::new("theia.test.missing").unwrap())
            .is_none()
    );
}
#[test]
fn schema_ruleset_null_unknown_and_required_fields_are_checked() {
    for key in ["schema", "id", "ruleset", "player", "creatures"] {
        let mut raw = value();
        raw.as_object_mut().unwrap().remove(key);
        assert!(parse(&raw).is_err());
        let mut raw = value();
        raw[key] = Value::Null;
        assert!(parse(&raw).is_err());
    }
    for version in [0, 2, u32::MAX] {
        let mut raw = value();
        raw["schema"] = json!(version);
        assert!(parse(&raw).is_err());
    }
    let mut raw = value();
    raw["ruleset"] = json!("retail_armour");
    assert!(parse(&raw).is_err());
    for key in ["physical", "attack_power", "spell_bonus_damage"] {
        let mut raw = value();
        raw["player"].as_object_mut().unwrap().remove(key);
        assert!(parse(&raw).is_err());
        let mut raw = value();
        raw["player"][key] = Value::Null;
        assert!(parse(&raw).is_err());
    }
    for owner in ["player", "creature"] {
        for key in ["level", "armour", "armour_penetration"] {
            for missing in [true, false] {
                let mut raw = value();
                let physical = if owner == "player" {
                    &mut raw["player"]["physical"]
                } else {
                    &mut raw["creatures"][0]["physical"]
                };
                if missing {
                    physical.as_object_mut().unwrap().remove(key);
                } else {
                    physical[key] = Value::Null;
                }
                assert!(parse(&raw).is_err());
            }
        }
    }
    for path in [vec![], vec!["player"], vec!["player", "physical"]] {
        let mut raw = value();
        let mut location = &mut raw;
        for key in path {
            location = &mut location[key];
        }
        location["unexpected"] = json!(0);
        assert!(parse(&raw).is_err());
    }
    let mut raw = value();
    raw["creatures"][0]["unexpected"] = json!(0);
    assert!(parse(&raw).is_err());
    let mut raw = value();
    raw["creatures"][0]["physical"]["unexpected"] = json!(0);
    assert!(parse(&raw).is_err());
}
#[test]
fn reference_level_and_nonnegative_stat_bounds_are_explicit() {
    for level in [0, 64, 80, 255, 256] {
        for owner in ["player", "creature"] {
            let mut raw = value();
            let physical = if owner == "player" {
                &mut raw["player"]["physical"]
            } else {
                &mut raw["creatures"][0]["physical"]
            };
            physical["level"] = json!(level);
            assert!(parse(&raw).is_err());
        }
    }
    for key in ["attack_power", "spell_bonus_damage"] {
        for amount in [-1, MAX_VALUE as i64 + 1] {
            let mut raw = value();
            raw["player"][key] = json!(amount);
            assert!(parse(&raw).is_err());
        }
    }
    for key in ["armour", "armour_penetration"] {
        for owner in ["player", "creature"] {
            let mut raw = value();
            let physical = if owner == "player" {
                &mut raw["player"]["physical"]
            } else {
                &mut raw["creatures"][0]["physical"]
            };
            physical[key] = json!(MAX_VALUE + 1);
            assert!(parse(&raw).is_err());
        }
    }
    let mut raw = value();
    raw["player"]["attack_power"] = json!(0);
    raw["player"]["spell_bonus_damage"] = json!(MAX_VALUE);
    assert!(parse(&raw).is_ok());
}
#[test]
fn byte_count_duplicate_and_identity_bounds_hold_before_reference_validation() {
    assert!(Profile::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1]).is_err());
    let mut raw = value();
    let first = raw["creatures"][0].clone();
    raw["creatures"].as_array_mut().unwrap().push(first.clone());
    assert!(parse(&raw).is_err());
    let mut raw = value();
    raw["creatures"] = Value::Array(
        (0..=MAX_CREATURES)
            .map(|n| {
                let mut c = first.clone();
                c["creature"] = json!(format!("theia.test.creature-{n}"));
                c
            })
            .collect(),
    );
    assert!(parse(&raw).is_err());
    raw["creatures"].as_array_mut().unwrap().pop();
    assert!(parse(&raw).is_ok());
    for invalid in ["../bad", "theia..bad", "Invalid"] {
        let mut raw = value();
        raw["id"] = json!(invalid);
        assert!(parse(&raw).is_err());
        let mut raw = value();
        raw["creatures"][0]["creature"] = json!(invalid);
        assert!(parse(&raw).is_err());
    }
}
#[test]
fn logical_stat_lookup_is_independent_of_source_array_order() {
    let mut raw = value();
    let mut other = raw["creatures"][0].clone();
    other["creature"] = json!("theia.test.second-creature");
    other["physical"]["armour"] = json!(100);
    raw["creatures"].as_array_mut().unwrap().push(other);
    let a = parse(&raw).unwrap();
    raw["creatures"].as_array_mut().unwrap().reverse();
    let b = parse(&raw).unwrap();
    assert_eq!(
        a.creatures().collect::<Vec<_>>(),
        b.creatures().collect::<Vec<_>>()
    );
    for (id, physical) in a.creatures() {
        assert_eq!(b.creature(id), Some(*physical));
    }
}
