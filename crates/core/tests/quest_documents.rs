//! Pure historical read proposals only; no client, item-open or SQL authority.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    documents::Catalog as Documents,
    inventory::{Bag, Catalog as Items, Command, Operation, Stack},
    quests::{
        catalog::{Catalog, EventKey, registry::Registry},
        journal::{Error, Journal, Status},
    },
};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn raw() -> Value {
    json!({"schema":4,"id":"quests","revision":1,"quests":[{
        "id":"quest","version":1,"title":"A note","description":"Read it","prerequisites":[],"giver":"guide","turn_in":"guide",
        "repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"first",
        "stages":[{"id":"first","completion":{"kind":"all"},"next":null,"objectives":[{"kind":"read_document","id":"read","label":"Read the note","document":"note"}]}]
    }]})
}
fn parse(v: &Value) -> Catalog {
    Catalog::from_json(&serde_json::to_vec(v).unwrap()).unwrap()
}
fn items() -> Items {
    Items::from_json(br#"{"schema":1,"id":"items","items":[{"id":"note-item","name":"Note","maximum_stack":20}]}"#).unwrap()
}
fn doc_raw() -> Value {
    json!({"schema":1,"id":"documents","revision":1,"item_catalog":"items","documents":[{"id":"note","item":"note-item","title":"The note","pages":["Take the road."]}]})
}
fn docs(v: &Value) -> Documents {
    Documents::from_json(&serde_json::to_vec(v).unwrap(), &items()).unwrap()
}
fn accepted(c: &Catalog) -> Journal {
    Journal::default()
        .propose_accept(c, &id("quest"), &id("guide"))
        .unwrap()
        .journal
}
fn bag(n: u32, revision: u64) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    if n > 0 {
        slots[0] = Stack::new(&items(), id("note-item"), n);
    }
    Bag::from_slots(&items(), revision, slots).unwrap()
}
#[test]
fn explicit_read_kind_requires_schema_four_known_document_and_count_one() {
    let value = raw();
    let catalog = parse(&value);
    let documents = docs(&doc_raw());
    assert!(
        catalog
            .quest(&id("quest"))
            .unwrap()
            .has_document_objectives()
    );
    assert_eq!(
        catalog.matching(&EventKey::ReadDocument(id("note"))).len(),
        1
    );
    documents.validate_quests(&catalog).unwrap();
    for schema in [1, 2, 3, 5] {
        let mut bad = value.clone();
        bad["schema"] = json!(schema);
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (field, val) in [
        ("required", json!(2)),
        ("consume_on_turn_in", json!(true)),
        ("item", json!("note-item")),
        ("document", json!("quest")),
    ] {
        let mut bad = value.clone();
        bad["quests"][0]["stages"][0]["objectives"][0][field] = val;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut unknown = value.clone();
    unknown["quests"][0]["stages"][0]["objectives"][0]["document"] = json!("missing");
    assert!(documents.validate_quests(&parse(&unknown)).is_err());
    let mut aliases = doc_raw();
    aliases["id"] = json!("guide");
    assert!(docs(&aliases).validate_quests(&catalog).is_err());
    aliases = doc_raw();
    aliases["documents"][0]["id"] = json!("guide");
    let mut aliased_quest = value.clone();
    aliased_quest["quests"][0]["stages"][0]["objectives"][0]["document"] = json!("guide");
    assert!(
        docs(&aliases)
            .validate_quests(&parse(&aliased_quest))
            .is_err()
    );
}
#[test]
fn preaccept_read_is_empty_original_stage_replay_cannot_cascade_and_new_read_can_progress() {
    let mut value = raw();
    value["quests"][0]["stages"][0]["next"] = json!("second");
    let mut second = value["quests"][0]["stages"][0].clone();
    second["id"] = json!("second");
    second["next"] = Value::Null;
    second["objectives"][0]["id"] = json!("read-again");
    value["quests"][0]["stages"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let c = parse(&value);
    let d = docs(&doc_raw());
    let empty = Journal::default();
    let before = empty.prepare_document_read(&c, &d, &id("note")).unwrap();
    assert!(before.is_empty());
    let journal = accepted(&c);
    assert_eq!(
        journal
            .propose_document_read(&c, &d, &before)
            .unwrap()
            .journal,
        journal
    );
    assert_eq!(
        journal.propose_trusted_event(&c, &EventKey::ReadDocument(id("note"))),
        Err(Error::InvalidProgress)
    );
    let first = journal.prepare_document_read(&c, &d, &id("note")).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first.goals()[0].stage(), &id("first"));
    let proposal = journal.propose_document_read(&c, &d, &first).unwrap();
    assert!(proposal.reward.is_none());
    let progressed = proposal.journal;
    let q = progressed.progress(&id("quest")).unwrap();
    assert_eq!(q.stage(), &id("second"));
    assert_eq!(q.status(), Status::Active);
    assert_eq!(q.count(&id("read-again")), Some(0));
    assert_eq!(
        progressed
            .propose_document_read(&c, &d, &first)
            .unwrap()
            .journal,
        progressed
    );
    let again = progressed
        .prepare_document_read(&c, &d, &id("note"))
        .unwrap();
    let ready = progressed
        .propose_document_read(&c, &d, &again)
        .unwrap()
        .journal;
    assert_eq!(
        ready.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    assert!(
        ready
            .prepare_document_read(&c, &d, &id("note"))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        ready.propose_document_read(&c, &d, &again).unwrap().journal,
        ready
    );
    assert_eq!(
        Journal::restore(&c, &ready.snapshot(&c).unwrap()).unwrap(),
        ready
    );
    let done = ready
        .propose_turn_in(
            &c,
            &id("quest"),
            &id("guide"),
            ready.progress(&id("quest")).unwrap().revision(),
        )
        .unwrap();
    assert_eq!(done.reward.unwrap().experience, 25);
    assert!(
        done.journal
            .prepare_document_read(&c, &d, &id("note"))
            .unwrap()
            .is_empty()
    );
}
#[test]
fn read_history_survives_item_loss_while_separate_possession_blocks_turn_in() {
    let mut value = raw();
    value["quests"][0]["stages"][0]["objectives"].as_array_mut().unwrap().push(json!({"kind":"possess_item","id":"bring","label":"Bring two","item":"note-item","required":2,"consume_on_turn_in":true}));
    let c = parse(&value);
    let d = docs(&doc_raw());
    d.validate_quests(&c).unwrap();
    let inventory = bag(2, 0);
    let journal = accepted(&c)
        .propose_inventory(&c, &items(), &inventory)
        .unwrap()
        .journal;
    let observation = journal.prepare_document_read(&c, &d, &id("note")).unwrap();
    let ready = journal
        .propose_document_read(&c, &d, &observation)
        .unwrap()
        .journal;
    assert_eq!(
        ready.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    let removed = inventory
        .propose(
            &items(),
            Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 2 },
            },
        )
        .unwrap();
    assert!(d.resolve_owned(&removed, 1, 0, &id("note")).is_err());
    let active = ready
        .propose_inventory(&c, &items(), &removed)
        .unwrap()
        .journal;
    assert_eq!(
        active.progress(&id("quest")).unwrap().count(&id("read")),
        Some(1)
    );
    assert_eq!(
        active.progress(&id("quest")).unwrap().count(&id("bring")),
        Some(0)
    );
    assert_eq!(
        active.progress(&id("quest")).unwrap().status(),
        Status::Active
    );
    assert!(
        active
            .prepare_document_read(&c, &d, &id("note"))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        active
            .propose_document_read(&c, &d, &observation)
            .unwrap()
            .journal,
        active
    );
    assert!(
        active
            .propose_possession_turn_in(
                &c,
                &items(),
                &removed,
                &id("quest"),
                &id("guide"),
                active.progress(&id("quest")).unwrap().revision()
            )
            .is_err()
    );
    let replacement = bag(2, 2);
    let ready = active
        .propose_inventory(&c, &items(), &replacement)
        .unwrap()
        .journal;
    let done = ready
        .propose_possession_turn_in(
            &c,
            &items(),
            &replacement,
            &id("quest"),
            &id("guide"),
            ready.progress(&id("quest")).unwrap().revision(),
        )
        .unwrap();
    assert!(done.bag.slots().iter().all(Option::is_none));
    assert_eq!(
        done.quest
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read")),
        Some(1)
    );
}
#[test]
fn read_observation_rejects_changed_catalogue_or_document_revision() {
    let c = parse(&raw());
    let d = docs(&doc_raw());
    let journal = accepted(&c);
    let observation = journal.prepare_document_read(&c, &d, &id("note")).unwrap();
    let mut changed = doc_raw();
    changed["revision"] = json!(2);
    assert!(
        journal
            .propose_document_read(&c, &docs(&changed), &observation)
            .is_err()
    );
    let mut changed = raw();
    changed["revision"] = json!(2);
    assert!(
        journal
            .propose_document_read(&parse(&changed), &d, &observation)
            .is_err()
    );
    assert!(
        journal
            .prepare_document_read(&c, &d, &id("unknown"))
            .is_err()
    );
}
#[test]
fn registry_preserves_read_selector_and_uses_minimum_schema_for_each_selection() {
    let mut value = raw();
    let mut old = value["quests"][0].clone();
    old["id"] = json!("old-quest");
    old["start_stage"] = json!("old-stage");
    old["stages"][0]["id"] = json!("old-stage");
    old["stages"][0]["objectives"] =
        json!([{"id":"old-goal","kind":"kill","label":"Kill","creature":"wolf","required":1}]);
    value["quests"].as_array_mut().unwrap().push(old);
    let bytes = serde_json::to_vec(&value).unwrap();
    let registry = Registry::from_catalogues(&[bytes.as_slice()]).unwrap();
    let compiled = registry
        .compile_catalogue(&id("compiled"), 1, &[id("quest")])
        .unwrap();
    assert_eq!(compiled.catalogue().schema(), 4);
    assert_eq!(
        compiled.catalogue().quest(&id("quest")),
        parse(&value).quest(&id("quest"))
    );
    assert_eq!(
        registry
            .compile_catalogue(&id("compiled"), 1, &[id("old-quest")])
            .unwrap()
            .catalogue()
            .schema(),
        1
    );
}

#[test]
fn one_read_credits_each_original_quest_once_and_respects_alternative_quorum() {
    let mut value = raw();
    let mut other = value["quests"][0].clone();
    other["id"] = json!("alternative");
    other["start_stage"] = json!("alternative-stage");
    other["stages"][0]["id"] = json!("alternative-stage");
    other["stages"][0]["completion"] = json!({"kind":"any"});
    other["stages"][0]["objectives"][0]["id"] = json!("alternative-read");
    other["stages"][0]["objectives"].as_array_mut().unwrap().push(json!({"kind":"kill","id":"alternative-kill","label":"Kill","creature":"wolf","required":1}));
    value["quests"].as_array_mut().unwrap().push(other);
    let c = parse(&value);
    let d = docs(&doc_raw());
    let journal = accepted(&c)
        .propose_accept(&c, &id("alternative"), &id("guide"))
        .unwrap()
        .journal;
    let observation = journal.prepare_document_read(&c, &d, &id("note")).unwrap();
    assert_eq!(observation.len(), 2);
    let proposal = journal.propose_document_read(&c, &d, &observation).unwrap();
    assert_eq!(proposal.changed.len(), 2);
    assert!(proposal.reward.is_none());
    let ready = proposal.journal;
    assert!(ready.quests().all(|(_, q)| q.status() == Status::Ready));
    assert_eq!(
        ready
            .progress(&id("alternative"))
            .unwrap()
            .count(&id("alternative-kill")),
        Some(0)
    );
    assert_eq!(
        ready
            .propose_trusted_event(&c, &EventKey::Kill(id("wolf")))
            .unwrap()
            .journal,
        ready
    );
    assert_eq!(
        ready
            .propose_document_read(&c, &d, &observation)
            .unwrap()
            .journal,
        ready
    );
}
