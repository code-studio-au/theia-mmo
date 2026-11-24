use super::*;
use serde_json::{Value, json};
fn context() -> (Quests, inventory::Catalog) {
    (
        Quests::from_json(include_bytes!(
            "../../../../../content/quests/development-journal-documents.json"
        ))
        .unwrap(),
        inventory::Catalog::from_json(include_bytes!(
            "../../../../../content/gameplay/development-document-items.json"
        ))
        .unwrap(),
    )
}
fn raw(quests: &Quests, items: &inventory::Catalog) -> Value {
    json!({"schema":1,"id":"theia.test.quest-item-rewards","revision":1,
    "quest_catalog":quests.id().as_str(),"item_catalog":items.id().as_str(),
    "rewards":[{"quest":quests.quests().next().unwrap().id().as_str(),"grants":[{"item":items.definitions().next().unwrap().id().as_str(),"count":3}]}]})
}
fn parse(
    raw: &Value,
    quests: &Quests,
    items: &inventory::Catalog,
) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(raw).unwrap(), quests, items)
}
#[test]
fn fixed_grants_retain_checked_context_without_mutating_or_admitting_it() {
    let (quests, items) = context();
    let raw = raw(&quests, &items);
    let before = (quests.clone(), items.clone());
    let parsed = parse(&raw, &quests, &items).unwrap();
    let quest = quests.quests().next().unwrap().id();
    assert_eq!(parsed.revision(), 1);
    assert_eq!(parsed.rewards().len(), 1);
    assert_eq!(parsed.grants(quest).unwrap()[0].count(), 3);
    assert_eq!(parsed.quest_catalog(), &quests);
    assert_eq!(parsed.item_catalog(), &items);
    assert_eq!((quests, items), before);
}
#[test]
fn strict_schema_context_and_unknown_choices_currency_or_fields_refuse() {
    let (quests, items) = context();
    let good = raw(&quests, &items);
    for (field, value) in [
        ("schema", json!(2)),
        ("revision", json!(0)),
        ("quest_catalog", json!("theia.wrong.quests")),
        ("item_catalog", json!("theia.wrong.items")),
        ("currency", json!(10)),
        ("choices", json!([])),
    ] {
        let mut raw = good.clone();
        raw[field] = value;
        assert!(parse(&raw, &quests, &items).is_err(), "{field}");
    }
    let mut raw = good.clone();
    raw["rewards"][0]["name"] = json!("Not supported");
    assert!(parse(&raw, &quests, &items).is_err());
    let mut raw = good;
    raw["rewards"][0]["grants"][0]["chance_basis_points"] = json!(5000);
    assert!(parse(&raw, &quests, &items).is_err());
}
#[test]
fn duplicate_unknown_and_invalid_grants_refuse_instead_of_silently_combining() {
    let (quests, items) = context();
    let good = raw(&quests, &items);
    for count in [
        0,
        crate::rewards::MAX_GRANT_UNITS + 1,
        u32::MAX,
        items.definitions().next().unwrap().maximum_stack() * inventory::SLOT_COUNT as u32 + 1,
    ] {
        let mut raw = good.clone();
        raw["rewards"][0]["grants"][0]["count"] = json!(count);
        assert!(parse(&raw, &quests, &items).is_err());
    }
    let mut raw = good.clone();
    raw["rewards"][0]["quest"] = json!("theia.missing.quest");
    assert!(parse(&raw, &quests, &items).is_err());
    let mut raw = good.clone();
    raw["rewards"][0]["grants"][0]["item"] = json!("theia.missing.item");
    assert!(parse(&raw, &quests, &items).is_err());
    let mut raw = good.clone();
    raw["rewards"] = json!([good["rewards"][0], good["rewards"][0]]);
    assert!(parse(&raw, &quests, &items).is_err());
    let mut raw = good.clone();
    raw["rewards"][0]["grants"] = json!([
        good["rewards"][0]["grants"][0],
        good["rewards"][0]["grants"][0]
    ]);
    assert!(parse(&raw, &quests, &items).is_err());
}
#[test]
fn all_count_byte_and_namespace_budgets_are_enforced() {
    let (quests, items) = context();
    let good = raw(&quests, &items);
    for count in [0, MAX_QUESTS + 1] {
        let mut raw = good.clone();
        raw["rewards"] = json!(vec![good["rewards"][0].clone(); count]);
        assert!(parse(&raw, &quests, &items).is_err());
    }
    for count in [0, MAX_GRANTS + 1] {
        let mut raw = good.clone();
        raw["rewards"][0]["grants"] = json!(vec![good["rewards"][0]["grants"][0].clone(); count]);
        assert!(parse(&raw, &quests, &items).is_err());
    }
    for id in [
        quests.id(),
        items.id(),
        quests.quests().next().unwrap().giver(),
        items.definitions().next().unwrap().id(),
    ] {
        let mut raw = good.clone();
        raw["id"] = json!(id.as_str());
        assert!(parse(&raw, &quests, &items).is_err());
    }
    let mut bytes = serde_json::to_vec(&good).unwrap();
    bytes.resize(MAX_CATALOG_BYTES + 1, b' ');
    assert!(Catalog::from_json(&bytes, &quests, &items).is_err());
}
#[test]
fn a_known_item_cannot_double_as_an_npc_or_quest_definition_reward() {
    let (quests, items) = context();
    let mut item_raw: Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-document-items.json"
    ))
    .unwrap();
    let mut alias = item_raw["items"][0].clone();
    let id = quests.quests().next().unwrap().giver().as_str();
    alias["id"] = json!(id);
    item_raw["items"].as_array_mut().unwrap().push(alias);
    let aliased = inventory::Catalog::from_json(&serde_json::to_vec(&item_raw).unwrap()).unwrap();
    let mut raw = raw(&quests, &items);
    raw["rewards"][0]["grants"][0]["item"] = json!(id);
    assert!(parse(&raw, &quests, &aliased).is_err());
}
#[test]
fn reward_capacity_is_checked_for_the_whole_distinct_item_bundle() {
    let (quests, items) = context();
    let mut raw = raw(&quests, &items);
    let definitions: Vec<_> = items.definitions().take(2).collect();
    raw["rewards"][0]["grants"] = json!([
        {"item":definitions[0].id().as_str(),"count":definitions[0].maximum_stack()*9},
        {"item":definitions[1].id().as_str(),"count":definitions[1].maximum_stack()*8}
    ]);
    assert_eq!(
        parse(&raw, &quests, &items).unwrap_err(),
        "Quest reward bundle cannot fit an empty backpack"
    );
    raw["rewards"][0]["grants"][1]["count"] = json!(definitions[1].maximum_stack() * 7);
    assert!(
        parse(&raw, &quests, &items).is_ok(),
        "Exactly sixteen required slots are valid metadata"
    );
}
