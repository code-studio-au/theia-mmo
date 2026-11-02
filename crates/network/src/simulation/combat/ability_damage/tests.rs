use super::*;
use serde_json::{Value, json};
const STARTING: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-ability-starting.json");
const POLICY: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-ability-damage.json");
const ABILITIES: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-classic-abilities.json");
const ORDINARY: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-white-melee.json");
const STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-physical-damage-stats.json");
const DEFINITIONS: &[u8] = include_bytes!(
    "../../../../../../content/gameplay/development-physical-damage-definitions.json"
);
fn physical_with(definitions: &[u8], stats: &[u8]) -> Arc<physical::Profile> {
    let weapons = theia_core::melee::Catalog::from_json(include_bytes!(
        "../../../../../../content/gameplay/development-melee-weapons.json"
    ))
    .unwrap();
    let items = theia_core::inventory::Catalog::from_json(include_bytes!(
        "../../../../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let abilities = theia_core::abilities::Catalog::from_json(include_bytes!(
        "../../../../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    let creature = crate::simulation::creatures::Profile::from_json(include_bytes!(
        "../../../../../../content/gameplay/development-creature-deflect-pursuit.json"
    ))
    .unwrap();
    Arc::new(
        physical::Profile::new(
            definitions,
            stats,
            physical::Bindings {
                weapons: &weapons,
                items: &items,
                abilities: &abilities,
                creature: &creature,
                player_maximum_health: 100,
                creature_maximum_health: 100,
            },
        )
        .unwrap(),
    )
}

fn white() -> Arc<white_melee::Profile> {
    Arc::new(white_melee::Profile::new(ORDINARY, physical_with(DEFINITIONS, STATS)).unwrap())
}
fn profile() -> Profile {
    Profile::new(STARTING, POLICY, ABILITIES, white()).unwrap()
}
fn raw() -> Value {
    serde_json::from_slice(POLICY).unwrap()
}
fn bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
fn key(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn sword() -> ContentId {
    key("theia.dev.sword-slash")
}
fn bolt() -> ContentId {
    key("theia.dev.arcane-bolt")
}
fn state() -> white_melee::DefenderState {
    white_melee::DefenderState {
        standing: true,
        can_react: true,
        in_front: true,
        ..Default::default()
    }
}
fn draws(base: u32, landing: u16, critical: Option<u16>) -> Draws {
    Draws {
        base,
        landing: Roll::new(landing).unwrap(),
        critical: critical.map(|r| Roll::new(r).unwrap()),
    }
}
fn parse(v: &Value) -> Result<Profile> {
    Profile::new(STARTING, &bytes(v), ABILITIES, white())
}
#[test]
fn candidate_keeps_physical_specials_and_arcane_outcomes_and_mitigation_distinct() {
    let p = profile();
    assert_eq!(
        p.id().as_str(),
        "theia.dev.ability-damage-reference-fixture-v1"
    );
    assert_eq!(p.recipes().recipes().len(), 2);
    let normal = p
        .evaluate(&sword(), state(), draws(11, 1000, Some(500)), 100)
        .unwrap();
    assert_eq!(
        normal.outcome,
        Outcome::MeleeSpecial(melee_special::Outcome::Hit)
    );
    assert_eq!(normal.damage.starting_exact.exact_ratio(), (104, 5));
    assert_eq!(normal.damage.resolved_damage, 16);
    let crit = p
        .evaluate(&sword(), state(), draws(11, 1000, Some(499)), 3)
        .unwrap();
    assert_eq!(
        crit.outcome,
        Outcome::MeleeSpecial(melee_special::Outcome::Critical)
    );
    assert_eq!(
        (
            crit.damage.resolved_damage,
            crit.damage.health_lost,
            crit.damage.overkill
        ),
        (33, 3, 30)
    );
    assert_eq!(
        p.evaluate(&sword(), state(), draws(11, 499, None), 100)
            .unwrap()
            .outcome,
        Outcome::MeleeSpecial(melee_special::Outcome::Miss)
    );
    assert_eq!(
        p.evaluate(&sword(), state(), draws(11, 500, None), 100)
            .unwrap()
            .outcome,
        Outcome::MeleeSpecial(melee_special::Outcome::Dodge)
    );
    let normal = p
        .evaluate(&bolt(), state(), draws(20, 400, Some(500)), 100)
        .unwrap();
    assert_eq!(normal.outcome, Outcome::Arcane(spell::Outcome::Hit));
    assert_eq!(
        (
            normal.damage.resolved_damage,
            normal.damage.armour_prevented
        ),
        (20, 0)
    );
    let crit = p
        .evaluate(&bolt(), state(), draws(21, 400, Some(499)), 100)
        .unwrap();
    assert_eq!(crit.outcome, Outcome::Arcane(spell::Outcome::Critical));
    assert_eq!(crit.damage.resolved_damage, 31);
    assert_eq!(
        p.evaluate(&bolt(), state(), draws(20, 399, None), 100)
            .unwrap()
            .outcome,
        Outcome::Arcane(spell::Outcome::Miss)
    );
}
#[test]
fn spell_bonus_is_explicit_and_does_not_change_shared_physical_or_ordinary_profiles() {
    let mut v = raw();
    v["spell_bonus_damage"] = json!(100);
    let p = parse(&v).unwrap();
    assert_eq!(p.physical().stats().player().spell_bonus_damage(), 0);
    assert_eq!(
        p.evaluate(&bolt(), state(), draws(20, 400, Some(500)), 100)
            .unwrap()
            .damage
            .resolved_damage,
        80
    );
    assert_eq!(
        p.evaluate(&sword(), state(), draws(11, 1000, Some(500)), 100)
            .unwrap()
            .damage
            .resolved_damage,
        16
    );
    assert_ne!(p.context_hash(), profile().context_hash());
}
#[test]
fn exact_starting_policy_ability_and_ordinary_bytes_are_identity_bound() {
    let p = profile();
    assert_ne!(p.identity([1; 32]), p.identity([2; 32]));
    for which in 0..3 {
        let mut b = [STARTING.to_vec(), POLICY.to_vec(), ABILITIES.to_vec()];
        b[which].push(b' ');
        let other = Profile::new(&b[0], &b[1], &b[2], white()).unwrap();
        assert_ne!(p.context_hash(), other.context_hash());
        assert_ne!(p.identity([1; 32]), other.identity([1; 32]));
    }
    let mut data = ORDINARY.to_vec();
    data.push(b' ');
    let w = Arc::new(white_melee::Profile::new(&data, physical_with(DEFINITIONS, STATS)).unwrap());
    assert_ne!(
        p.context_hash(),
        Profile::new(STARTING, POLICY, ABILITIES, w)
            .unwrap()
            .context_hash()
    );
}
#[test]
fn required_fields_schemas_rulesets_and_nested_policies_reject_silent_fallbacks() {
    for k in raw().as_object().unwrap().keys() {
        let mut v = raw();
        v.as_object_mut().unwrap().remove(k);
        assert!(parse(&v).is_err(), "missing {k}");
        let mut v = raw();
        v[k] = Value::Null;
        assert!(parse(&v).is_err(), "null {k}");
    }
    for i in 0..2 {
        for k in raw()["abilities"][i].as_object().unwrap().keys() {
            let mut v = raw();
            v["abilities"][i].as_object_mut().unwrap().remove(k);
            assert!(parse(&v).is_err(), "missing {k}");
            let mut v = raw();
            v["abilities"][i][k] = Value::Null;
            assert!(parse(&v).is_err(), "null {k}");
        }
        let mut v = raw();
        v["abilities"][i]["extra"] = json!(true);
        assert!(parse(&v).is_err());
    }
    for (k, value) in [
        ("schema", json!(0)),
        ("schema", json!(2)),
        ("ruleset", json!("classic_exact")),
        ("starting", json!("theia.test.other")),
        ("ordinary", json!("theia.test.other")),
        ("id", json!("../escape")),
        ("extra", json!(true)),
    ] {
        let mut v = raw();
        v[k] = value;
        assert!(parse(&v).is_err(), "{k}");
    }
}
#[test]
fn recipes_and_policies_must_have_exact_coverage_categories_and_known_creature_capabilities() {
    let mut v = raw();
    v["abilities"].as_array_mut().unwrap().pop();
    assert!(parse(&v).is_err());
    let mut v = raw();
    v["abilities"][1] = v["abilities"][0].clone();
    assert!(parse(&v).is_err());
    for i in 0..2 {
        let mut v = raw();
        v["abilities"][i]["ability"] = json!("theia.test.unknown");
        assert!(parse(&v).is_err());
    }
    let mut v = raw();
    v["abilities"][0]["ability"] = json!("theia.dev.arcane-bolt");
    assert!(parse(&v).is_err());
    let mut v = raw();
    v["abilities"][1]["ability"] = json!("theia.dev.sword-slash");
    assert!(parse(&v).is_err());
    let mut v = raw();
    v["abilities"][0]["parry_bp"] = json!(500);
    assert!(parse(&v).is_err());
    let mut w: Value = serde_json::from_slice(ORDINARY).unwrap();
    w["creature"]["defences"]["block_bp"] = json!(500);
    let w =
        Arc::new(white_melee::Profile::new(&bytes(&w), physical_with(DEFINITIONS, STATS)).unwrap());
    assert!(Profile::new(STARTING, POLICY, ABILITIES, w).is_err());
    let mut s: Value = serde_json::from_slice(STATS).unwrap();
    s["creatures"][0]["physical"]["level"] = json!(2);
    let w = Arc::new(
        white_melee::Profile::new(ORDINARY, physical_with(DEFINITIONS, &bytes(&s))).unwrap(),
    );
    assert!(Profile::new(STARTING, POLICY, ABILITIES, w).is_err());
}
#[test]
fn current_defender_state_and_conditional_draws_are_checked_without_mutating_health() {
    let p = profile();
    let disabled = white_melee::DefenderState {
        can_react: false,
        ..state()
    };
    assert!(
        p.needs_critical_roll(&sword(), disabled, Roll::new(500).unwrap())
            .unwrap()
    );
    assert_eq!(
        p.evaluate(&sword(), disabled, draws(11, 500, Some(500)), 100)
            .unwrap()
            .outcome,
        Outcome::MeleeSpecial(melee_special::Outcome::Hit)
    );
    assert!(
        p.evaluate(&sword(), state(), draws(11, 500, Some(0)), 100)
            .is_err()
    );
    assert!(
        p.evaluate(&sword(), state(), draws(11, 1000, None), 100)
            .is_err()
    );
    assert!(
        p.evaluate(&bolt(), state(), draws(20, 399, Some(0)), 100)
            .is_err()
    );
    assert!(
        p.evaluate(&bolt(), state(), draws(20, 400, None), 100)
            .is_err()
    );
    assert!(
        p.evaluate(
            &sword(),
            white_melee::DefenderState {
                standing: false,
                ..state()
            },
            draws(11, 1000, Some(500)),
            100
        )
        .is_err()
    );
    for a in [sword(), bolt()] {
        for hp in [0, 101, u32::MAX] {
            assert!(
                p.evaluate(
                    &a,
                    state(),
                    draws(p.range(&a).unwrap().maximum(), 9999, Some(9999)),
                    hp
                )
                .is_err()
            );
        }
        assert!(
            p.evaluate(
                &a,
                state(),
                draws(p.range(&a).unwrap().maximum() + 1, 9999, Some(9999)),
                100
            )
            .is_err()
        );
    }
    assert!(p.range(&key("theia.test.unknown")).is_err());
}
#[test]
fn byte_numeric_combined_damage_and_ability_scope_bounds_fail_before_activation() {
    assert!(
        Profile::new(
            STARTING,
            &vec![b' '; MAX_PROFILE_BYTES + 1],
            ABILITIES,
            white()
        )
        .is_err()
    );
    assert!(
        Profile::new(
            &vec![b' '; theia_core::damage::ability_starting::catalog::MAX_PROFILE_BYTES + 1],
            POLICY,
            ABILITIES,
            white()
        )
        .is_err()
    );
    assert!(
        Profile::new(
            STARTING,
            POLICY,
            &vec![b' '; abilities::MAX_CATALOG_BYTES + 1],
            white()
        )
        .is_err()
    );
    for (i, k, n) in [
        (0, "miss_bp", 10001),
        (0, "dodge_bp", 10001),
        (0, "parry_bp", 10001),
        (0, "critical_bp", 10001),
        (1, "hit_bonus_bp", 10001),
        (1, "critical_bp", 10001),
    ] {
        let mut v = raw();
        v["abilities"][i][k] = json!(n);
        assert!(parse(&v).is_err());
    }
    let mut v = raw();
    v["spell_bonus_damage"] = json!(MAX_VALUE + 1);
    assert!(parse(&v).is_err());
    let mut start: Value = serde_json::from_slice(STARTING).unwrap();
    start["recipes"][0]["flat_bonus"] = json!(MAX_VALUE);
    assert!(Profile::new(&bytes(&start), POLICY, ABILITIES, white()).is_err());
    let mut a: Value = serde_json::from_slice(ABILITIES).unwrap();
    a["abilities"][1]["range_metres"] = json!(30);
    assert!(Profile::new(STARTING, POLICY, &bytes(&a), white()).is_err());
    let mut a: Value = serde_json::from_slice(ABILITIES).unwrap();
    a["abilities"][1]["combat"]["facing"] = json!({"kind":"any"});
    assert!(Profile::new(STARTING, POLICY, &bytes(&a), white()).is_err());
}
