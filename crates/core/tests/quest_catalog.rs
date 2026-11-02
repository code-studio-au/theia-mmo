use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    quests::catalog::{Catalog, Completion, EventKey, MAX_CATALOG_BYTES},
};
const DATA: &[u8] = include_bytes!("../../../content/quests/development-journal.json");
fn raw() -> Value {
    serde_json::from_slice(DATA).unwrap()
}
fn parse(v: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(v).unwrap())
}
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
#[test]
fn catalogue_keeps_versions_prerequisites_and_exact_event_indexes() {
    let c = Catalog::from_json(DATA).unwrap();
    assert_eq!(c.quests().count(), 2);
    let q = c.quest(&id("theia.dev.quest-follow-up")).unwrap();
    assert_eq!(q.version(), 1);
    assert_eq!(q.prerequisites(), &[id("theia.dev.quest-introduction")]);
    let hits = c.matching(&EventKey::Kill(id("theia.dev.training-creature")));
    assert_eq!(hits.len(), 1);
    assert_eq!(&hits[0].quest, q.id());
    let npc = EventKey::NpcInteraction {
        npc: id("theia.dev.guide"),
        interaction: id("theia.dev.interaction.road"),
    };
    assert_eq!(c.matching(&npc).len(), 1);
    assert!(
        c.matching(&EventKey::NpcInteraction {
            npc: id("theia.dev.other"),
            interaction: id("theia.dev.interaction.road")
        })
        .is_empty()
    );
    assert!(
        c.matching(&EventKey::Kill(id("theia.dev.other")))
            .is_empty()
    );
}
#[test]
fn strict_versions_fields_and_bounds_refuse_unsupported_capabilities() {
    for (field, value) in [
        ("schema", json!(2)),
        ("revision", json!(0)),
        ("unknown", json!(true)),
        ("quests", json!([])),
    ] {
        let mut v = raw();
        v[field] = value;
        assert!(parse(&v).is_err());
    }
    for (field, value) in [
        ("version", json!(0)),
        ("repeat_policy", json!("daily")),
        ("credit_scope", json!("party")),
        ("experience", json!(1001)),
        ("title", json!(" bad ")),
        ("script", json!("quest.luau")),
        ("id", json!("H1-01")),
    ] {
        let mut v = raw();
        v["quests"][0][field] = value;
        assert!(parse(&v).is_err());
    }
    assert!(Catalog::from_json(&vec![b' '; MAX_CATALOG_BYTES + 1]).is_err());
    assert!(Catalog::from_json(b"null").is_err());
}
#[test]
fn prerequisites_require_known_unique_acyclic_quests() {
    for prerequisites in [
        json!(["theia.dev.missing"]),
        json!(["theia.dev.quest-introduction"]),
        json!(["theia.dev.quest-follow-up", "theia.dev.quest-follow-up"]),
    ] {
        let mut v = raw();
        v["quests"][0]["prerequisites"] = prerequisites;
        assert!(parse(&v).is_err());
    }
    let mut v = raw();
    v["quests"][0]["prerequisites"] = json!(["theia.dev.quest-follow-up"]);
    assert!(parse(&v).is_err());
}
#[test]
fn stage_links_refuse_missing_unreachable_cyclic_and_cross_quest_targets() {
    for next in [
        json!("theia.dev.missing"),
        json!("theia.dev.quest-introduction.meet"),
        Value::Null,
        json!("theia.dev.quest-follow-up.practice"),
    ] {
        let mut v = raw();
        v["quests"][0]["stages"][0]["next"] = next;
        assert!(parse(&v).is_err());
    }
    let mut v = raw();
    v["quests"][0]["start_stage"] = json!("theia.dev.missing");
    assert!(parse(&v).is_err());
}
#[test]
fn global_definition_ids_and_per_stage_event_selectors_must_not_alias() {
    let mut v = raw();
    v["quests"][1]["id"] = v["quests"][0]["id"].clone();
    assert!(parse(&v).is_err());
    let mut v = raw();
    v["quests"][0]["stages"][0]["objectives"][1]["id"] =
        v["quests"][0]["stages"][0]["objectives"][0]["id"].clone();
    assert!(parse(&v).is_err());
    let mut v = raw();
    v["quests"][0]["giver"] = v["quests"][1]["id"].clone();
    assert!(parse(&v).is_err());
    let mut v = raw();
    v["quests"][0]["stages"][0]["objectives"][1]["interaction"] =
        v["quests"][0]["stages"][0]["objectives"][0]["interaction"].clone();
    assert!(parse(&v).is_err());
}
#[test]
fn completion_rules_and_objective_counts_cannot_be_impossible() {
    for completion in [
        json!({"kind":"at_least","count":0}),
        json!({"kind":"at_least","count":3}),
        json!({"kind":"all","count":1}),
        json!({"kind":"branch"}),
    ] {
        let mut v = raw();
        v["quests"][0]["stages"][0]["completion"] = completion;
        assert!(parse(&v).is_err());
    }
    for count in [0, 101] {
        let mut v = raw();
        v["quests"][1]["stages"][0]["objectives"][0]["required"] = json!(count);
        assert!(parse(&v).is_err());
    }
    let mut v = raw();
    v["quests"][0]["stages"][0]["objectives"][0]["required"] = json!(5);
    assert!(parse(&v).is_err());
    assert!(!Completion::All.satisfied(1, 2));
    assert!(Completion::All.satisfied(2, 2));
    assert!(Completion::Any.satisfied(1, 2));
    assert!(!Completion::Any.satisfied(0, 2));
    assert!(Completion::AtLeast { count: 2 }.satisfied(2, 3));
}
#[test]
fn valid_alternative_completion_rules_and_ordering_are_preserved() {
    for rule in [json!({"kind":"any"}), json!({"kind":"at_least","count":1})] {
        let mut v = raw();
        v["quests"][0]["stages"][0]["completion"] = rule;
        assert!(parse(&v).is_ok());
    }
    let mut v = raw();
    v["quests"].as_array_mut().unwrap().reverse();
    assert_eq!(parse(&v).unwrap(), Catalog::from_json(DATA).unwrap());
}
