use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    quests::catalog::{
        Catalog, EventKey, MAX_CATALOG_BYTES, MAX_QUESTS,
        registry::{CompileError, Registry},
    },
};
fn id(value: &str) -> ContentId {
    ContentId::new(value).unwrap()
}
fn registry(values: &[Value]) -> Registry {
    let sources: Vec<_> = values
        .iter()
        .map(|v| serde_json::to_vec(v).unwrap())
        .collect();
    Registry::from_catalogues(&sources.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap()
}
fn canonical() -> (Value, Vec<Value>) {
    let raw: Value = serde_json::from_slice(include_bytes!(
        "../../../content/quests/development-journal.json"
    ))
    .unwrap();
    let packs=(0..2).map(|n|json!({"schema":1,"id":format!("theia.test.pack.{n}"),"revision":1,"quests":[raw["quests"][n].clone()]})).collect();
    (raw, packs)
}
fn simple(n: usize) -> Value {
    json!({"id":format!("q{n}"),"version":1,"title":"Q","description":"D","prerequisites":[],"giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":10,
        "start_stage":format!("s{n}-0"),"stages":[{"id":format!("s{n}-0"),"completion":{"kind":"all"},"next":null,"objectives":[{"kind":"kill","id":format!("o{n}-0-0"),"label":"x","creature":"wolf0","required":1}]}]})
}
fn pack(n: usize, quests: Vec<Value>) -> Value {
    json!({"schema":1,"id":format!("pack{n}"),"revision":1,"quests":quests})
}
#[test]
fn selected_follow_up_compiles_every_prerequisite_and_preserves_definitions_versions_and_indexes() {
    let (raw, packs) = canonical();
    let registry = registry(&packs);
    let compiled = registry
        .compile_catalogue(
            &id("theia.test.compiled"),
            7,
            &[id("theia.dev.quest-follow-up")],
        )
        .unwrap();
    let closed = Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let parsed = Catalog::from_json(compiled.as_bytes()).unwrap();
    assert_eq!(&parsed, compiled.catalogue());
    assert_eq!(parsed.id(), &id("theia.test.compiled"));
    assert_eq!(parsed.revision(), 7);
    assert_eq!(parsed.quests().count(), 2);
    for quest in closed.quests() {
        assert_eq!(parsed.quest(quest.id()), Some(quest));
    }
    for event in [
        EventKey::Kill(id("theia.dev.training-creature")),
        EventKey::NpcInteraction {
            npc: id("theia.dev.guide"),
            interaction: id("theia.dev.interaction.road"),
        },
    ] {
        assert_eq!(parsed.matching(&event), closed.matching(&event));
    }

    let one = registry
        .compile_catalogue(
            &id("theia.test.one"),
            1,
            &[id("theia.dev.quest-introduction")],
        )
        .unwrap();
    assert_eq!(one.catalogue().quests().count(), 1);
    assert!(
        one.catalogue()
            .quest(&id("theia.dev.quest-follow-up"))
            .is_none()
    );
    assert_eq!(one.as_bytes(), one.clone().into_bytes());
}
#[test]
fn source_and_root_order_do_not_change_compiled_bytes() {
    let (_, mut packs) = canonical();
    let first = registry(&packs);
    packs.reverse();
    let second = registry(&packs);
    let output = id("theia.test.compiled");
    let a = id("theia.dev.quest-introduction");
    let b = id("theia.dev.quest-follow-up");
    let one = first
        .compile_catalogue(&output, 1, &[a.clone(), b.clone()])
        .unwrap();
    let two = second.compile_catalogue(&output, 1, &[b, a]).unwrap();
    assert_eq!(one.as_bytes(), two.as_bytes());
}
#[test]
fn invalid_selection_and_metadata_refuse_without_renaming_or_weakening_the_closed_validator() {
    let (_, packs) = canonical();
    let r = registry(&packs);
    let output = id("theia.test.compiled");
    let q = id("theia.dev.quest-introduction");
    assert_eq!(
        r.compile_catalogue(&output, 0, std::slice::from_ref(&q))
            .unwrap_err(),
        CompileError::Revision
    );
    assert_eq!(
        r.compile_catalogue(&output, 1, &[]).unwrap_err(),
        CompileError::SelectionCount
    );
    assert_eq!(
        r.compile_catalogue(&output, 1, &vec![q.clone(); MAX_QUESTS + 1])
            .unwrap_err(),
        CompileError::SelectionCount
    );
    assert_eq!(
        r.compile_catalogue(&output, 1, &[q.clone(), q.clone()])
            .unwrap_err(),
        CompileError::DuplicateSelection { quest: q.clone() }
    );
    assert_eq!(
        r.compile_catalogue(&output, 1, &[id("theia.test.missing")])
            .unwrap_err(),
        CompileError::UnknownSelection {
            quest: id("theia.test.missing")
        }
    );
    for alias in [q.clone(), id("theia.dev.guide")] {
        assert!(matches!(
            r.compile_catalogue(&alias, 1, std::slice::from_ref(&q))
                .unwrap_err(),
            CompileError::ClosedCatalogue { .. }
        ));
    }
}
#[test]
fn transitive_closure_refuses_thirty_three_quests_instead_of_omitting_an_ancestor() {
    let quests: Vec<_> = (0..=MAX_QUESTS)
        .map(|n| {
            let mut q = simple(n);
            if n > 0 {
                q["prerequisites"] = json!([format!("q{}", n - 1)]);
            }
            q
        })
        .collect();
    let r = registry(&[
        pack(0, quests[..MAX_QUESTS].to_vec()),
        pack(1, quests[MAX_QUESTS..].to_vec()),
    ]);
    let output = id("compiled");
    assert_eq!(
        r.compile_catalogue(&output, 1, &[id(&format!("q{MAX_QUESTS}"))])
            .unwrap_err(),
        CompileError::QuestBudget
    );
    let boundary = r
        .compile_catalogue(&output, 1, &[id(&format!("q{}", MAX_QUESTS - 1))])
        .unwrap();
    assert_eq!(boundary.catalogue().quests().count(), MAX_QUESTS);
}
#[test]
fn complete_objective_budget_is_preserved_and_excess_refuses_before_an_output() {
    let packs:Vec<_>=(0..9).map(|n|{
        let mut q=simple(n);
        q["stages"]=json!((0..8).map(|stage|json!({"id":format!("s{n}-{stage}"),"completion":{"kind":"all"},"next":if stage<7 {Some(format!("s{n}-{}",stage+1))} else {None},
            "objectives":(0..8).map(|o|json!({"kind":"kill","id":format!("o{n}-{stage}-{o}"),"label":"x","creature":format!("wolf{o}"),"required":1})).collect::<Vec<_>>()})).collect::<Vec<_>>());
        pack(n,vec![q])
    }).collect();
    let r = registry(&packs);
    let roots: Vec<_> = (0..9).map(|n| id(&format!("q{n}"))).collect();
    let compiled = r
        .compile_catalogue(&id("compiled"), 1, &roots[..8])
        .unwrap();
    assert_eq!(
        compiled
            .catalogue()
            .quests()
            .flat_map(|q| q.stages())
            .map(|s| s.objectives().len())
            .sum::<usize>(),
        512
    );
    assert_eq!(
        r.compile_catalogue(&id("compiled"), 1, &roots).unwrap_err(),
        CompileError::ObjectiveBudget
    );
}
#[test]
fn a_valid_large_source_selection_still_refuses_the_runtime_byte_limit() {
    let packs: Vec<_> = (0..MAX_QUESTS)
        .map(|n| {
            let mut q = simple(n);
            q["description"] = json!("x".repeat(2048));
            pack(n, vec![q])
        })
        .collect();
    let r = registry(&packs);
    let roots: Vec<_> = (0..MAX_QUESTS).map(|n| id(&format!("q{n}"))).collect();
    assert_eq!(
        r.compile_catalogue(&id("compiled"), 1, &roots).unwrap_err(),
        CompileError::ByteBudget
    );
    let smaller = r
        .compile_catalogue(&id("compiled"), 1, &roots[..2])
        .unwrap();
    assert!(smaller.as_bytes().len() <= MAX_CATALOG_BYTES);
}
#[test]
fn all_any_and_at_least_rules_round_trip_through_the_closed_catalogue() {
    for rule in [
        json!({"kind":"all"}),
        json!({"kind":"any"}),
        json!({"kind":"at_least","count":1}),
    ] {
        let mut q = simple(0);
        q["version"] = json!(9);
        q["stages"][0]["completion"] = rule;
        let source = pack(0, vec![q]);
        let expected = Catalog::from_json(&serde_json::to_vec(&source).unwrap()).unwrap();
        let r = registry(&[source]);
        let compiled = r
            .compile_catalogue(&id("compiled"), 1, &[id("q0")])
            .unwrap();
        assert_eq!(
            compiled.catalogue().quest(&id("q0")),
            expected.quest(&id("q0"))
        );
    }
}
#[test]
fn reach_area_compilation_selects_minimum_schema_and_preserves_selector() {
    let mut q = simple(0);
    q["stages"][0]["objectives"][0] =
        json!({"kind":"reach_area","id":"o0-0-0","label":"Reach","area":"overlook"});
    let mut source = pack(0, vec![q, simple(1)]);
    source["schema"] = json!(2);
    let r = registry(&[source.clone()]);
    let c = r
        .compile_catalogue(&id("compiled"), 1, &[id("q0")])
        .unwrap();
    assert_eq!(c.catalogue().schema(), 2);
    assert_eq!(
        c.catalogue()
            .matching(&EventKey::ReachArea(id("overlook")))
            .len(),
        1
    );
    assert_eq!(
        r.compile_catalogue(&id("compiled"), 1, &[id("q1")])
            .unwrap()
            .catalogue()
            .schema(),
        1
    );
    assert_eq!(
        r.compile_catalogue(&id("compiled"), 1, &[id("q1"), id("q0")])
            .unwrap()
            .as_bytes(),
        r.compile_catalogue(&id("compiled"), 1, &[id("q0"), id("q1")])
            .unwrap()
            .as_bytes()
    );
    // An area selector may not alias a definition in a different source pack.
    let mut collision = simple(2);
    collision["id"] = json!("overlook");
    let a = serde_json::to_vec(&source).unwrap();
    let b = serde_json::to_vec(&pack(2, vec![collision])).unwrap();
    assert!(Registry::from_catalogues(&[&a, &b]).is_err());
}
#[test]
fn possession_compilation_retains_explicit_consumption_and_selects_minimum_schema() {
    let mut q = simple(0);
    q["stages"][0]["objectives"][0] = json!({"kind":"possess_item","id":"o0-0-0",
        "label":"Bring","item":"pelts","required":3,"consume_on_turn_in":false});
    let mut source = pack(0, vec![q, simple(1)]);
    source["schema"] = json!(3);
    let r = registry(&[source.clone()]);
    let c = r
        .compile_catalogue(&id("compiled"), 1, &[id("q0")])
        .unwrap();
    assert_eq!(c.catalogue().schema(), 3);
    let objective = &c
        .catalogue()
        .quest(&id("q0"))
        .unwrap()
        .stages()
        .next()
        .unwrap()
        .objectives()[0];
    assert_eq!(objective.event(), &EventKey::PossessItem(id("pelts")));
    assert_eq!(objective.required(), 3);
    assert!(!objective.consume_on_turn_in());
    let raw: Value = serde_json::from_slice(c.as_bytes()).unwrap();
    assert_eq!(
        raw["quests"][0]["stages"][0]["objectives"][0]["consume_on_turn_in"],
        json!(false)
    );
    assert_eq!(
        r.compile_catalogue(&id("compiled"), 1, &[id("q1")])
            .unwrap()
            .catalogue()
            .schema(),
        1
    );
    let mut collision = simple(2);
    collision["id"] = json!("pelts");
    let a = serde_json::to_vec(&source).unwrap();
    let b = serde_json::to_vec(&pack(1, vec![collision])).unwrap();
    assert!(Registry::from_catalogues(&[&a, &b]).is_err());
}
