use super::*;
use serde_json::{Value, json};

fn contact(prefix: &str) -> Value {
    json!({"schema":1,"id":format!("{prefix}.dialogue"),"npc":format!("{prefix}.npc"),
        "name":prefix,"start":format!("{prefix}.hello"),"nodes":[
            {"id":format!("{prefix}.hello"),"text":"Welcome","choices":[
                {"id":format!("{prefix}.report"),"label":"Report","next":format!("{prefix}.bye")}]},
            {"id":format!("{prefix}.bye"),"text":"Safe travels","choices":[]}]})
}
fn fixture() -> Value {
    json!({"schema":2,"id":"contacts","revision":1,"contacts":[contact("elder"),contact("ranger")]})
}
fn parse(value: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(value).unwrap())
}
fn quests() -> Value {
    json!({"schema":1,"id":"quests","revision":1,"quests":[{
        "id":"delivery","version":1,"title":"Meet the ranger","description":"Report to the ranger",
        "giver":"elder.npc","turn_in":"ranger.npc","prerequisites":[],"repeat_policy":"once",
        "credit_scope":"character","experience":25,"start_stage":"delivery.stage",
        "stages":[{"id":"delivery.stage","next":null,"completion":{"kind":"all"},"objectives":[{
            "kind":"npc_interaction","id":"delivery.report","label":"Report to the ranger",
            "npc":"ranger.npc","interaction":"ranger.report"}]}]}]})
}

#[test]
fn distinct_giver_and_turn_in_resolve_without_conflating_semantic_actions() {
    let catalog = parse(&fixture()).unwrap();
    assert_eq!(catalog.id().as_str(), "contacts");
    assert_eq!(catalog.revision(), 1);
    assert_eq!(catalog.contacts().count(), 2);
    let validate = |value: &Value| {
        let quests = Quests::from_json(&serde_json::to_vec(value).unwrap()).unwrap();
        catalog.validate_quest_contacts(&quests)
    };
    validate(&quests()).unwrap();
    for field in ["giver", "turn_in"] {
        let mut value = quests();
        value["quests"][0][field] = json!("missing.npc");
        assert!(validate(&value).is_err());
    }
    let mut value = quests();
    value["quests"][0]["stages"][0]["objectives"][0]["interaction"] = json!("elder.report");
    assert!(
        validate(&value).is_err(),
        "An existing choice on another contact is not equivalent"
    );
    value["quests"][0]["stages"][0]["objectives"][0]["npc"] = json!("elder.npc");
    validate(&value).unwrap();
}

#[test]
fn global_aliases_unknown_fields_and_invalid_graphs_are_refused() {
    let original = fixture();
    for mutation in [
        ("/contacts/1/npc", json!("elder.npc")),
        ("/contacts/1/id", json!("elder.dialogue")),
        ("/contacts/1/nodes/0/choices/0/id", json!("elder.report")),
        ("/contacts/1/nodes/0/id", json!("elder.hello")),
        ("/id", json!("elder.report")),
        ("/contacts/1/nodes/0/choices/0/next", json!("elder.bye")),
    ] {
        let mut value = original.clone();
        *value.pointer_mut(mutation.0).unwrap() = mutation.1;
        assert!(parse(&value).is_err(), "{}", mutation.0);
    }
    let mut value = original.clone();
    value["contacts"][0]["vendor"] = json!("unvalidated");
    assert!(parse(&value).is_err());
    let mut value = original;
    value["unknown"] = json!(true);
    assert!(parse(&value).is_err());
    let duplicate = serde_json::to_string(&fixture()).unwrap().replace(
        "\"npc\":\"elder.npc\"",
        "\"npc\":\"other.npc\",\"npc\":\"elder.npc\"",
    );
    assert!(Catalog::from_json(duplicate.as_bytes()).is_err());
}

#[test]
fn budgets_and_legacy_runtime_boundary_are_explicit() {
    let value = fixture();
    assert!(Profile::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(Catalog::from_json(&serde_json::to_vec(&contact("elder")).unwrap()).is_err());
    assert!(Catalog::from_json(&vec![b' '; MAX_CATALOG_BYTES + 1]).is_err());
    for (field, invalid) in [
        ("schema", json!(1)),
        ("revision", json!(0)),
        ("contacts", json!([])),
    ] {
        let mut value = value.clone();
        value[field] = invalid;
        assert!(parse(&value).is_err());
    }
    let mut value = fixture();
    value["contacts"] = json!(
        (0..MAX_CONTACTS)
            .map(|i| contact(&format!("npc{i}")))
            .collect::<Vec<_>>()
    );
    assert_eq!(parse(&value).unwrap().contacts().count(), MAX_CONTACTS);
    value["contacts"]
        .as_array_mut()
        .unwrap()
        .push(contact("extra"));
    assert!(parse(&value).is_err());
}

#[test]
fn quest_definition_aliases_are_not_contact_references() {
    let catalog = parse(&fixture()).unwrap();
    let mut value = quests();
    value["id"] = json!("elder.report");
    let quests = Quests::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(catalog.validate_quest_contacts(&quests).is_err());
}
