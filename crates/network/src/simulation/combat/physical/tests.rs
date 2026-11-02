use super::*;
use serde_json::{Value, json};
const DEFINITIONS: &[u8] = include_bytes!(
    "../../../../../../content/gameplay/development-physical-damage-definitions.json"
);
const STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-physical-damage-stats.json");
struct Known {
    weapons: melee::Catalog,
    items: inventory::Catalog,
    abilities: abilities::Catalog,
    creature: creatures::Profile,
}
impl Known {
    fn new() -> Self {
        Self {
            weapons: melee::Catalog::from_json(include_bytes!(
                "../../../../../../content/gameplay/development-melee-weapons.json"
            ))
            .unwrap(),
            items: inventory::Catalog::from_json(include_bytes!(
                "../../../../../../content/gameplay/development-equipment-items.json"
            ))
            .unwrap(),
            abilities: abilities::Catalog::from_json(include_bytes!(
                "../../../../../../content/gameplay/development-classic-abilities.json"
            ))
            .unwrap(),
            creature: creatures::Profile::from_json(include_bytes!(
                "../../../../../../content/gameplay/development-creature-deflect-pursuit.json"
            ))
            .unwrap(),
        }
    }
    fn bind(&self, player: u32, creature: u32) -> Bindings<'_> {
        Bindings {
            weapons: &self.weapons,
            items: &self.items,
            abilities: &self.abilities,
            creature: &self.creature,
            player_maximum_health: player,
            creature_maximum_health: creature,
        }
    }
    fn profile(&self) -> Profile {
        Profile::new(DEFINITIONS, STATS, self.bind(100, 1000)).unwrap()
    }
}
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn raw() -> Value {
    serde_json::from_slice(DEFINITIONS).unwrap()
}
fn bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
#[test]
fn candidate_profile_resolves_both_directions_with_checked_declared_stats() {
    let known = Known::new();
    let profile = known.profile();
    assert_eq!(profile.player_maximum_health(), 100);
    assert_eq!(profile.creature_maximum_health(), 1000);
    assert_eq!(profile.creature_definition(), known.creature.id());
    assert_eq!(profile.stats().player().physical().level(), 1);
    assert_eq!(
        profile
            .weapon(&id("theia.dev.training-sword"))
            .unwrap()
            .0
            .as_str(),
        "theia.dev.training-sword-swing"
    );
    assert_eq!(profile.creature_attack().0.as_str(), "theia.dev.wolf-bite");
    let result = profile
        .weapon_result(&id("theia.dev.training-sword"), 11, 10)
        .unwrap();
    assert_eq!(
        (
            result.starting_damage,
            result.armour_prevented,
            result.resolved_damage,
            result.health_lost,
            result.overkill
        ),
        (15, 3, 12, 10, 2)
    );
    assert_eq!(profile.creature_result(5, 100).unwrap().resolved_damage, 4);
}
#[test]
fn identity_binds_exact_definition_stat_and_health_bytes_after_the_previous_profile() {
    let known = Known::new();
    let profile = known.profile();
    let base = [7; 32];
    let identity = profile.identity(base);
    assert_eq!(identity, profile.clone().identity(base));
    assert_ne!(identity, base);
    assert_ne!(identity, profile.identity([8; 32]));
    let mut data = DEFINITIONS.to_vec();
    data.push(b' ');
    assert_ne!(
        identity,
        Profile::new(&data, STATS, known.bind(100, 1000))
            .unwrap()
            .identity(base)
    );
    let mut stats = STATS.to_vec();
    stats.push(b' ');
    assert_ne!(
        identity,
        Profile::new(DEFINITIONS, &stats, known.bind(100, 1000))
            .unwrap()
            .identity(base)
    );
    assert_ne!(
        identity,
        Profile::new(DEFINITIONS, STATS, known.bind(101, 1000))
            .unwrap()
            .identity(base)
    );
    assert_ne!(
        identity,
        Profile::new(DEFINITIONS, STATS, known.bind(100, 1001))
            .unwrap()
            .identity(base)
    );
}
#[test]
fn positional_ability_rules_and_valid_health_bounds_are_required_before_binding() {
    let mut known = Known::new();
    for (player, creature) in [
        (0, 100),
        (100, 0),
        (MAX_VALUE + 1, 100),
        (100, MAX_VALUE + 1),
    ] {
        assert!(Profile::new(DEFINITIONS, STATS, known.bind(player, creature)).is_err());
    }
    let mut legacy: Value = serde_json::from_slice(include_bytes!(
        "../../../../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    legacy["schema"] = json!(6);
    for ability in legacy["abilities"].as_array_mut().unwrap() {
        ability.as_object_mut().unwrap().remove("combat");
    }
    known.abilities = abilities::Catalog::from_json(&bytes(&legacy)).unwrap();
    assert!(Profile::new(DEFINITIONS, STATS, known.bind(100, 1000)).is_err());
}
#[test]
fn every_existing_weapon_requires_a_matching_definition_and_base_scaling_speed() {
    let known = Known::new();
    let mut data = raw();
    data["attacks"].as_array_mut().unwrap().remove(0);
    assert!(Profile::new(&bytes(&data), STATS, known.bind(100, 1000)).is_err());
    let mut data = raw();
    data["attacks"][0]["base_speed_ms"] = json!(2800);
    assert!(Profile::new(&bytes(&data), STATS, known.bind(100, 1000)).is_err());
    let mut data = raw();
    let mut extra = data["attacks"][0].clone();
    extra["id"] = json!("theia.test.wand-swing");
    extra["item"] = json!("theia.dev.training-wand");
    data["attacks"].as_array_mut().unwrap().push(extra);
    assert!(Profile::new(&bytes(&data), STATS, known.bind(100, 1000)).is_err());
}
#[test]
fn unknown_creature_item_and_unused_stats_do_not_fall_back_to_legacy_damage() {
    let known = Known::new();
    for (index, key, value) in [
        (0, "item", "theia.dev.training-stone"),
        (1, "creature", "theia.test.unknown"),
    ] {
        let mut data = raw();
        data["attacks"][index][key] = json!(value);
        assert!(Profile::new(&bytes(&data), STATS, known.bind(100, 1000)).is_err());
    }
    let mut stats: Value = serde_json::from_slice(STATS).unwrap();
    stats["creatures"][0]["creature"] = json!("theia.test.unused");
    assert!(Profile::new(DEFINITIONS, &bytes(&stats), known.bind(100, 1000)).is_err());
    let mut data = raw();
    data["attacks"].as_array_mut().unwrap().pop();
    stats["creatures"] = json!([]);
    assert!(Profile::new(&bytes(&data), &bytes(&stats), known.bind(100, 1000)).is_err());
}
#[test]
fn first_live_profile_rejects_spell_definitions_and_unused_spell_bonus() {
    let known = Known::new();
    let candidate =
        include_bytes!("../../../../../../content/gameplay/development-damage-definitions.json");
    assert!(
        Profile::new(candidate, STATS, known.bind(100, 1000))
            .unwrap_err()
            .to_string()
            .contains("spell")
    );
    let mut stats: Value = serde_json::from_slice(STATS).unwrap();
    stats["player"]["spell_bonus_damage"] = json!(1);
    assert!(
        Profile::new(DEFINITIONS, &bytes(&stats), known.bind(100, 1000))
            .unwrap_err()
            .to_string()
            .contains("spell")
    );
}
#[test]
fn result_calculation_rejects_invalid_rolls_health_and_unknown_equipment() {
    let known = Known::new();
    let profile = known.profile();
    let sword = id("theia.dev.training-sword");
    for roll in [9, 13, u32::MAX] {
        assert!(profile.weapon_result(&sword, roll, 100).is_err());
    }
    for roll in [3, 7, u32::MAX] {
        assert!(profile.creature_result(roll, 100).is_err());
    }
    assert!(
        profile
            .weapon_result(&id("theia.dev.training-wand"), 11, 100)
            .is_err()
    );
    assert!(profile.weapon_result(&sword, 11, 1001).is_err());
    assert!(profile.creature_result(5, 101).is_err());
    let dead = profile.creature_result(5, 0).unwrap();
    assert_eq!((dead.health_lost, dead.overkill), (0, 4));
}
#[test]
fn combined_magnitude_failure_prevents_configuration_not_just_health_application() {
    let known = Known::new();
    let mut data = raw();
    data["attacks"][0]["maximum"] = json!(MAX_VALUE);
    assert!(Profile::new(&bytes(&data), STATS, known.bind(100, 1000)).is_err());
}
#[test]
fn definition_order_changes_exact_identity_but_not_logical_result_lookup() {
    let known = Known::new();
    let a = known.profile();
    let mut data = raw();
    data["attacks"].as_array_mut().unwrap().reverse();
    let b = Profile::new(&bytes(&data), STATS, known.bind(100, 1000)).unwrap();
    assert_eq!(
        a.catalog().attacks().collect::<Vec<_>>(),
        b.catalog().attacks().collect::<Vec<_>>()
    );
    assert_eq!(
        a.weapon_result(&id("theia.dev.training-sword"), 11, 100)
            .unwrap(),
        b.weapon_result(&id("theia.dev.training-sword"), 11, 100)
            .unwrap()
    );
    assert_ne!(a.identity([7; 32]), b.identity([7; 32]));
}
