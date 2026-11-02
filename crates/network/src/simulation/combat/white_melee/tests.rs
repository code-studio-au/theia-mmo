use super::*;
use serde_json::{Value, json};
const DATA: &[u8] =
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
fn physical() -> Arc<physical::Profile> {
    physical_with(DEFINITIONS, STATS)
}
fn profile() -> Profile {
    Profile::new(DATA, physical()).unwrap()
}
fn raw() -> Value {
    serde_json::from_slice(DATA).unwrap()
}
fn bytes(raw: &Value) -> Vec<u8> {
    serde_json::to_vec(raw).unwrap()
}
fn item() -> ContentId {
    ContentId::new("theia.dev.training-sword").unwrap()
}
fn state() -> DefenderState {
    DefenderState {
        standing: true,
        can_react: true,
        in_front: true,
        usable_melee_weapon: true,
        usable_shield: true,
        ..Default::default()
    }
}
fn draws(outcome: u16) -> Draws {
    Draws {
        outcome: Roll::new(outcome).unwrap(),
        base: 11,
        glancing: None,
    }
}
#[test]
fn candidate_binds_level_one_rates_without_granting_glancing_parry_block_or_crushing() {
    let p = profile();
    assert_eq!(
        p.outgoing_distribution(&item(), state()).unwrap().weights(),
        [50, 500, 0, 0, 0, 500, 0, 8950]
    );
    assert_eq!(
        p.incoming_distribution(state()).unwrap().weights(),
        [500, 500, 0, 0, 0, 500, 0, 8500]
    );
    let behind = DefenderState {
        in_front: false,
        ..state()
    };
    assert_eq!(
        p.incoming_distribution(behind).unwrap().weights(),
        [500, 0, 0, 0, 0, 500, 0, 9000]
    );
    assert_eq!(
        p.outgoing_distribution(&item(), behind).unwrap().weights(),
        [50, 500, 0, 0, 0, 500, 0, 8950]
    );
    assert_eq!(p.outgoing_rates(&item()).unwrap().nominal()[4], 1000);
}
#[test]
fn checked_draws_resolve_both_directions_with_separate_avoidance_and_overkill() {
    let p = profile();
    let hit = p
        .evaluate_outgoing(&item(), state(), draws(9999), 3)
        .unwrap();
    assert_eq!(
        (
            hit.outcome,
            hit.resolved_damage,
            hit.health_lost,
            hit.overkill
        ),
        (Outcome::Hit, 12, 3, 9)
    );
    let crit = p
        .evaluate_outgoing(&item(), state(), draws(550), 100)
        .unwrap();
    assert_eq!(crit.resolved_damage, 24);
    for (roll, outcome) in [(0, Outcome::Miss), (50, Outcome::Dodge)] {
        let r = p
            .evaluate_outgoing(&item(), state(), draws(roll), 100)
            .unwrap();
        assert_eq!(r.outcome, outcome);
        assert_eq!(r.health_lost, 0);
        assert!(r.armour.is_none());
    }
    let incoming = p
        .evaluate_incoming(
            state(),
            Draws {
                base: 5,
                ..draws(1000)
            },
            100,
        )
        .unwrap();
    assert_eq!(
        (incoming.outcome, incoming.resolved_damage),
        (Outcome::Critical, 8)
    );
}
#[test]
fn capability_and_gear_are_checked_separately_from_nominal_defensive_rates() {
    let mut r = raw();
    for role in ["player", "creature"] {
        r[role]["defences"]["parry_bp"] = json!(500);
        r[role]["defences"]["block_bp"] = json!(500);
        r[role]["defences"]["block_value"] = json!(3);
    }
    let p = Profile::new(&bytes(&r), physical()).unwrap();
    let no_gear = DefenderState {
        usable_melee_weapon: false,
        usable_shield: false,
        ..state()
    };
    assert_eq!(
        p.incoming_distribution(no_gear).unwrap().weights(),
        [500, 500, 0, 0, 0, 500, 0, 8500]
    );
    let front = p.outgoing_distribution(&item(), state()).unwrap();
    assert_eq!(front.weights()[2..4], [500, 500]);
    let behind = DefenderState {
        in_front: false,
        ..state()
    };
    assert_eq!(
        p.outgoing_distribution(&item(), behind).unwrap().weights()[2..4],
        [0, 0]
    );
    let block = p
        .evaluate_outgoing(&item(), state(), draws(1050), 100)
        .unwrap();
    assert_eq!(
        (block.outcome, block.blocked, block.resolved_damage),
        (Outcome::Block, 3, 9)
    );
    let casting = DefenderState {
        casting_non_melee: true,
        ..state()
    };
    assert_eq!(
        p.incoming_distribution(casting).unwrap().weights()[1..4],
        [0, 0, 0]
    );
    assert!(p.incoming_distribution(DefenderState::default()).is_err());
}
#[test]
fn profile_identity_binds_exact_bytes_and_the_complete_physical_context() {
    let physical = physical();
    let first = Profile::new(DATA, physical.clone()).unwrap();
    let mut spaced = DATA.to_vec();
    spaced.push(b' ');
    let changed = Profile::new(&spaced, physical).unwrap();
    assert_ne!(first.context_hash(), changed.context_hash());
    assert_ne!(first.identity([7; 32]), changed.identity([7; 32]));
    let mut stats: Value = serde_json::from_slice(STATS).unwrap();
    stats["player"]["physical"]["armour"] = json!(101);
    let other = Profile::new(DATA, physical_with(DEFINITIONS, &bytes(&stats))).unwrap();
    assert_ne!(first.context_hash(), other.context_hash());
    assert_eq!(first.id().as_str(), "theia.dev.white-melee-v1");
}
#[test]
fn profile_rejects_unknown_missing_null_and_incompatible_references_without_echoing_input() {
    for (key, value) in [
        ("schema", json!(2)),
        ("id", json!("bad id")),
        ("definitions", json!("theia.dev.other")),
        ("stats", json!("theia.dev.other")),
        ("ruleset", json!("other")),
        ("private", json!("DO-NOT-ECHO")),
    ] {
        let mut r = raw();
        r[key] = value;
        let error = Profile::new(&bytes(&r), physical())
            .unwrap_err()
            .to_string();
        assert!(!error.contains("DO-NOT-ECHO"));
    }
    for key in ["player", "creature", "definitions", "stats"] {
        let mut r = raw();
        r.as_object_mut().unwrap().remove(key);
        assert!(Profile::new(&bytes(&r), physical()).is_err());
        let mut r = raw();
        r[key] = Value::Null;
        assert!(Profile::new(&bytes(&r), physical()).is_err());
    }
    let mut r = raw();
    r["creature"]["id"] = json!("theia.dev.other");
    assert!(Profile::new(&bytes(&r), physical()).is_err());
    let mut r = raw();
    r["player"]["glancing_policy"] = json!("caster");
    assert!(Profile::new(&bytes(&r), physical()).is_err());
}
#[test]
fn complete_weapon_bindings_and_all_numeric_budgets_are_enforced() {
    for key in ["skill", "hit_bonus_bp"] {
        let mut r = raw();
        r["player"]["weapons"][0][key] = json!(10001);
        assert!(Profile::new(&bytes(&r), physical()).is_err());
    }
    for role in ["player", "creature"] {
        let mut r = raw();
        r[role]["critical_bp"] = json!(10001);
        assert!(Profile::new(&bytes(&r), physical()).is_err());
        for key in ["dodge_bp", "parry_bp", "block_bp"] {
            let mut r = raw();
            r[role]["defences"][key] = json!(10001);
            assert!(Profile::new(&bytes(&r), physical()).is_err());
        }
        let mut r = raw();
        r[role]["defences"]["block_value"] = json!(MAX_VALUE + 1);
        assert!(Profile::new(&bytes(&r), physical()).is_err());
    }
    let mut r = raw();
    r["player"]["defense_skill"] = json!(1001);
    assert!(Profile::new(&bytes(&r), physical()).is_err());
    for count in [0, 2, 33] {
        let mut r = raw();
        let weapon = r["player"]["weapons"][0].clone();
        r["player"]["weapons"] = json!(vec![weapon; count]);
        assert!(Profile::new(&bytes(&r), physical()).is_err());
    }
    let mut r = raw();
    r["player"]["weapons"][0]["item"] = json!("theia.dev.training-wand");
    assert!(Profile::new(&bytes(&r), physical()).is_err());
    assert!(Profile::new(&vec![b' '; MAX_PROFILE_BYTES + 1], physical()).is_err());
}
#[test]
fn invalid_health_base_draws_or_unused_glancing_draws_cannot_produce_a_result() {
    let p = profile();
    for health in [0, 101, MAX_VALUE] {
        assert!(
            p.evaluate_outgoing(&item(), state(), draws(9999), health)
                .is_err()
        );
    }
    for base in [9, 13] {
        assert!(
            p.evaluate_outgoing(
                &item(),
                state(),
                Draws {
                    base,
                    ..draws(9999)
                },
                100
            )
            .is_err()
        );
    }
    assert!(
        p.evaluate_outgoing(
            &item(),
            state(),
            Draws {
                glancing: Some(65),
                ..draws(9999)
            },
            100
        )
        .is_err()
    );
    assert!(
        p.evaluate_incoming(
            state(),
            Draws {
                base: 5,
                glancing: Some(65),
                ..draws(9999)
            },
            100
        )
        .is_err()
    );
}
#[test]
fn glancing_evaluation_requires_an_eligible_bound_percent_and_lower_crit_policy_is_explicit() {
    let mut stats: Value = serde_json::from_slice(STATS).unwrap();
    stats["player"]["physical"]["level"] = json!(60);
    stats["creatures"][0]["physical"]["level"] = json!(63);
    let mut r = raw();
    r["player"]["weapons"][0]["skill"] = json!(300);
    r["player"]["defense_skill"] = json!(300);
    let p = Profile::new(&bytes(&r), physical_with(DEFINITIONS, &bytes(&stats))).unwrap();
    let draw = Draws {
        glancing: Some(65),
        ..draws(1450)
    };
    let glance = p.evaluate_outgoing(&item(), state(), draw, 100).unwrap();
    assert_eq!(glance.outcome, Outcome::Glancing);
    assert!(glance.resolved_damage < glance.armour.unwrap().remaining);
    for percent in [None, Some(54), Some(76)] {
        assert!(
            p.evaluate_outgoing(
                &item(),
                state(),
                Draws {
                    glancing: percent,
                    ..draw
                },
                100
            )
            .is_err()
        );
    }
    stats["creatures"][0]["physical"]["level"] = json!(59);
    assert!(Profile::new(&bytes(&r), physical_with(DEFINITIONS, &bytes(&stats))).is_err());
}
#[test]
fn startup_rejects_outcome_overflow_before_an_attack_can_be_admitted() {
    let mut definitions: Value = serde_json::from_slice(DEFINITIONS).unwrap();
    definitions["attacks"][0]["minimum"] = json!(600000);
    definitions["attacks"][0]["maximum"] = json!(600001);
    let mut stats: Value = serde_json::from_slice(STATS).unwrap();
    stats["creatures"][0]["physical"]["armour"] = json!(0);
    assert!(Profile::new(DATA, physical_with(&bytes(&definitions), &bytes(&stats))).is_err());
}

#[test]
fn supported_ability_check_rejects_cast_time_melee_with_or_without_animation_cue() {
    let p = profile();
    let mut abilities: Value = serde_json::from_slice(include_bytes!(
        "../../../../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    p.check_supported_abilities(
        &theia_core::abilities::Catalog::from_json(&bytes(&abilities)).unwrap(),
    )
    .unwrap();
    abilities["abilities"][1]["cast_ms"] = json!(500);
    assert!(
        p.check_supported_abilities(
            &theia_core::abilities::Catalog::from_json(&bytes(&abilities)).unwrap()
        )
        .is_err()
    );
    abilities["abilities"][1]
        .as_object_mut()
        .unwrap()
        .remove("animation");
    assert!(
        p.check_supported_abilities(
            &theia_core::abilities::Catalog::from_json(&bytes(&abilities)).unwrap()
        )
        .is_err()
    );
}
