use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    quests::catalog::{
        Catalog, MAX_CATALOG_BYTES, MAX_QUESTS,
        registry::{
            Error as RegistryError, MAX_CATALOGUES, MAX_REGISTRY_BYTES, MAX_REGISTRY_QUESTS,
            Registry,
        },
    },
};
fn id(value: &str) -> ContentId {
    ContentId::new(value).unwrap()
}
fn quest(n: usize) -> Value {
    json!({"id":format!("theia.test.quest.{n}"),"version":1,"title":"A test quest","description":"A bounded executable definition.",
        "prerequisites":[],"giver":"theia.test.guide","turn_in":"theia.test.guide","repeat_policy":"once","credit_scope":"character","experience":10,
        "start_stage":format!("theia.test.stage.{n}"),"stages":[{"id":format!("theia.test.stage.{n}"),"completion":{"kind":"all"},"next":null,
            "objectives":[{"id":format!("theia.test.objective.{n}"),"kind":"kill","label":"Defeat a creature","creature":"theia.test.wolf","required":1}]}]})
}
fn catalogue(n: usize, quests: Vec<Value>) -> Value {
    json!({"schema":1,"id":format!("theia.test.pack.{n}"),"revision":1,"quests":quests})
}
fn parse(values: &[Value]) -> Result<Registry, RegistryError> {
    let bytes: Vec<_> = values
        .iter()
        .map(|v| serde_json::to_vec(v).unwrap())
        .collect();
    Registry::from_catalogues(&bytes.iter().map(Vec::as_slice).collect::<Vec<_>>())
}
fn standalone(value: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(value).unwrap())
}
#[test]
fn registry_resolves_canonical_cross_pack_prerequisites_without_weakening_live_catalogues() {
    let original = include_bytes!("../../../content/quests/development-journal.json");
    let raw: Value = serde_json::from_slice(original).unwrap();
    let closed = Catalog::from_json(original).unwrap();
    let a = catalogue(0, vec![raw["quests"][0].clone()]);
    let b = catalogue(1, vec![raw["quests"][1].clone()]);
    assert!(standalone(&a).is_ok());
    assert!(standalone(&b).is_err());
    let registry = parse(&[b.clone(), a.clone()]).unwrap();
    let ordered = parse(&[a, b]).unwrap();
    assert_eq!(registry.quest_count(), 2);
    for quest in closed.quests() {
        assert_eq!(registry.quest(quest.id()), Some(quest));
        assert_eq!(ordered.quest(quest.id()), Some(quest));
        assert_eq!(
            registry.catalogue_for(quest.id()),
            ordered.catalogue_for(quest.id())
        );
    }
    let info: Vec<_> = registry.catalogues().collect();
    assert_eq!(info.len(), 2);
    assert_eq!(info[0].id, &id("theia.test.pack.1"));
    assert_eq!((info[0].revision, info[0].quests), (1, 1));
    assert_eq!(registry.quest(&id("theia.test.unknown")), None);
    assert_eq!(registry.catalogue_for(&id("theia.test.unknown")), None);
}
#[test]
fn global_prerequisites_reject_missing_self_and_cross_pack_cycles_but_allow_shared_diamonds() {
    let a = catalogue(0, vec![quest(0)]);
    let mut b = catalogue(1, vec![quest(1)]);
    b["quests"][0]["prerequisites"] = json!(["theia.test.quest.0"]);
    assert!(parse(&[b.clone()]).is_err());
    let mut self_cycle = a.clone();
    self_cycle["quests"][0]["prerequisites"] = json!(["theia.test.quest.0"]);
    assert!(parse(&[self_cycle]).is_err());
    let mut a_cycle = a.clone();
    a_cycle["quests"][0]["prerequisites"] = json!(["theia.test.quest.1"]);
    assert!(parse(&[a_cycle, b.clone()]).is_err());
    let mut c = catalogue(2, vec![quest(2)]);
    c["quests"][0]["prerequisites"] = json!(["theia.test.quest.0"]);
    let mut d = catalogue(3, vec![quest(3)]);
    d["quests"][0]["prerequisites"] = json!(["theia.test.quest.1", "theia.test.quest.2"]);
    assert_eq!(parse(&[d, c, b, a]).unwrap().quest_count(), 4);
}
#[test]
fn every_definition_kind_shares_one_global_namespace() {
    for kind in ["catalogue", "quest", "stage", "objective", "cross_kind"] {
        let a = catalogue(0, vec![quest(0)]);
        let mut b = catalogue(1, vec![quest(1)]);
        match kind {
            "catalogue" => b["id"] = a["id"].clone(),
            "quest" => b["quests"][0]["id"] = a["quests"][0]["id"].clone(),
            "stage" | "cross_kind" => {
                let alias = if kind == "stage" {
                    a["quests"][0]["start_stage"].clone()
                } else {
                    a["id"].clone()
                };
                b["quests"][0]["start_stage"] = alias.clone();
                b["quests"][0]["stages"][0]["id"] = alias;
            }
            "objective" => {
                b["quests"][0]["stages"][0]["objectives"][0]["id"] =
                    a["quests"][0]["stages"][0]["objectives"][0]["id"].clone()
            }
            _ => unreachable!(),
        }
        assert!(standalone(&a).is_ok());
        assert!(standalone(&b).is_ok(), "{kind}");
        assert!(parse(&[a, b]).is_err(), "{kind}");
    }
}
#[test]
fn world_targets_cannot_alias_a_definition_in_another_pack() {
    for role in ["giver", "turn_in", "creature", "npc", "interaction"] {
        let a = catalogue(0, vec![quest(0)]);
        let mut b = catalogue(1, vec![quest(1)]);
        let alias = "theia.test.quest.0";
        match role {
            "giver" | "turn_in" => b["quests"][0][role] = json!(alias),
            "creature" => b["quests"][0]["stages"][0]["objectives"][0][role] = json!(alias),
            "npc" | "interaction" => {
                b["quests"][0]["stages"][0]["objectives"][0] = json!({"id":"theia.test.objective.1","kind":"npc_interaction","label":"Speak","npc":"theia.test.guide","interaction":"theia.test.speak"});
                b["quests"][0]["stages"][0]["objectives"][0][role] = json!(alias);
            }
            _ => unreachable!(),
        }
        assert!(standalone(&a).is_ok());
        assert!(standalone(&b).is_ok(), "{role}");
        assert!(parse(&[a, b]).is_err(), "{role}");
    }
}
#[test]
fn registry_retains_strict_definition_and_per_pack_budgets() {
    assert!(Registry::from_catalogues(&[]).is_err());
    assert!(Registry::from_catalogues(&vec![b"{}".as_slice(); MAX_CATALOGUES + 1]).is_err());
    assert!(Registry::from_catalogues(&[&vec![b' '; MAX_CATALOG_BYTES + 1]]).is_err());
    let mut invalid = catalogue(0, vec![quest(0)]);
    invalid["quests"][0]["script"] = json!("unapproved.luau");
    assert!(parse(&[invalid]).is_err());
    let mut invalid = catalogue(0, vec![quest(0)]);
    invalid["quests"][0]["credit_scope"] = json!("party");
    assert!(parse(&[invalid]).is_err());
    let invalid = catalogue(0, (0..=MAX_QUESTS).map(quest).collect());
    assert!(parse(&[invalid]).is_err());
}
#[test]
fn full_bounded_registry_accepts_1024_quest_chain_at_byte_budget_and_refuses_its_cycle() {
    let mut values = Vec::new();
    for pack in 0..MAX_CATALOGUES {
        let quests = (0..MAX_QUESTS)
            .map(|index| {
                let n = pack * MAX_QUESTS + index;
                let mut q = quest(n);
                if n > 0 {
                    q["prerequisites"] = json!([format!("theia.test.quest.{}", n - 1)]);
                }
                q
            })
            .collect();
        values.push(catalogue(pack, quests));
    }
    let sources: Vec<_> = values
        .iter()
        .map(|v| {
            let mut b = serde_json::to_vec(v).unwrap();
            assert!(b.len() <= MAX_CATALOG_BYTES);
            b.resize(MAX_CATALOG_BYTES, b' ');
            b
        })
        .collect();
    assert_eq!(
        sources.iter().map(Vec::len).sum::<usize>(),
        MAX_REGISTRY_BYTES
    );
    let result =
        Registry::from_catalogues(&sources.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap();
    assert_eq!(result.quest_count(), MAX_REGISTRY_QUESTS);
    assert_eq!(
        result.definition_count(),
        MAX_CATALOGUES + MAX_REGISTRY_QUESTS * 3
    );
    values[0]["quests"][0]["prerequisites"] =
        json!([format!("theia.test.quest.{}", MAX_REGISTRY_QUESTS - 1)]);
    assert!(parse(&values).is_err());
}

#[test]
fn authoring_errors_identify_source_indexes_and_stable_ids_without_dumping_prose() {
    let a = catalogue(0, vec![quest(0)]);
    let mut b = catalogue(1, vec![quest(1)]);
    b["quests"][0]["prerequisites"] = json!(["theia.test.absent"]);
    let error = parse(&[a.clone(), b.clone()]).unwrap_err();
    assert_eq!(
        error,
        RegistryError::UnknownPrerequisite {
            quest: id("theia.test.quest.1"),
            prerequisite: id("theia.test.absent"),
            catalogue: 1
        }
    );
    assert!(error.to_string().contains("theia.test.absent"));
    b["quests"][0]["prerequisites"] = json!([]);
    b["quests"][0]["id"] = json!("theia.test.quest.0");
    assert_eq!(
        parse(&[a.clone(), b]).unwrap_err(),
        RegistryError::DuplicateDefinition {
            id: id("theia.test.quest.0"),
            first: 0,
            second: 1
        }
    );
    let mut invalid = catalogue(1, vec![quest(1)]);
    invalid["quests"][0]["script"] = json!("private prose not echoed");
    let error = parse(&[a, invalid]).unwrap_err();
    assert!(matches!(error, RegistryError::Catalogue { index: 1, .. }));
    assert!(!error.to_string().contains("private prose"));
}
