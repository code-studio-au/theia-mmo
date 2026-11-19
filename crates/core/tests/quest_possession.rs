//! Pure current-possession proposals; no live deletion, loot or SQL authority.
use serde_json::{Value, json};
use std::collections::BTreeSet;
use theia_core::{
    content::ContentId,
    inventory::{Bag, Catalog as Items, MAX_REVISION, Stack},
    quests::{
        catalog::{Catalog, EventKey},
        journal::{Error, Journal, Status},
    },
};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn item() -> ContentId {
    id("theia.dev.training-stone")
}
fn items() -> Items {
    Items::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap()
}
fn bag(items: &Items, n: u32) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    if n > 0 {
        slots[0] = Stack::new(items, item(), n);
    }
    Bag::from_slots(items, 0, slots).unwrap()
}
fn raw() -> Value {
    json!({"schema":3,"id":"catalog","revision":1,"quests":[{
        "id":"delivery","version":1,"title":"Delivery","description":"Test","prerequisites":[],
        "giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"bring","stages":[{"id":"bring","completion":{"kind":"all"},"next":null,
        "objectives":[{"kind":"possess_item","id":"pelts","label":"Bring three","item":"theia.dev.training-stone",
            "required":3,"consume_on_turn_in":true}]}]}]})
}
fn parse(v: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(v).unwrap())
}
fn accepted(c: &Catalog) -> Journal {
    Journal::default()
        .propose_accept(c, &id("delivery"), &id("guide"))
        .unwrap()
        .journal
}
#[test]
fn schema_consume_flag_final_stage_and_all_completion_are_explicit() {
    let v = raw();
    let c = parse(&v).unwrap();
    assert_eq!(c.schema(), 3);
    assert!(
        c.quest(&id("delivery"))
            .unwrap()
            .has_possession_objectives()
    );
    assert!(
        c.quest(&id("delivery"))
            .unwrap()
            .stage(&id("bring"))
            .unwrap()
            .objectives()[0]
            .consume_on_turn_in()
    );
    for schema in [1, 2, 4] {
        let mut bad = v.clone();
        bad["schema"] = json!(schema);
        assert!(parse(&bad).is_err());
    }
    for rule in [json!({"kind":"any"}), json!({"kind":"at_least","count":1})] {
        let mut bad = v.clone();
        bad["quests"][0]["stages"][0]["completion"] = rule;
        assert!(parse(&bad).is_err());
    }
    for flag in [Value::Null, json!("true"), json!(1)] {
        let mut bad = v.clone();
        bad["quests"][0]["stages"][0]["objectives"][0]["consume_on_turn_in"] = flag;
        assert!(parse(&bad).is_err());
    }
    let mut bad = v.clone();
    bad["quests"][0]["stages"][0]["objectives"][0]
        .as_object_mut()
        .unwrap()
        .remove("consume_on_turn_in");
    assert!(parse(&bad).is_err());
    let mut bad = v;
    let mut next = bad["quests"][0]["stages"][0].clone();
    next["id"] = json!("after");
    next["objectives"] =
        json!([{"kind":"kill","id":"kill","label":"Kill","creature":"wolf","required":1}]);
    bad["quests"][0]["stages"][0]["next"] = json!("after");
    bad["quests"][0]["stages"]
        .as_array_mut()
        .unwrap()
        .push(next);
    assert!(parse(&bad).is_err());
}
#[test]
fn holdings_recalculate_ready_active_ready_and_surplus_does_not_add_fake_progress() {
    let c = parse(&raw()).unwrap();
    let i = items();
    let j = accepted(&c);
    let ready = j.propose_inventory(&c, &i, &bag(&i, 3)).unwrap().journal;
    assert_eq!(
        ready.progress(&id("delivery")).unwrap().status(),
        Status::Ready
    );
    assert_eq!(ready.progress(&id("delivery")).unwrap().revision(), 2);
    let same = ready.propose_inventory(&c, &i, &bag(&i, 9)).unwrap();
    assert!(same.changed.is_empty());
    assert_eq!(same.journal, ready);
    let reduced = ready
        .propose_inventory(&c, &i, &bag(&i, 2))
        .unwrap()
        .journal;
    let p = reduced.progress(&id("delivery")).unwrap();
    assert_eq!(
        (p.status(), p.count(&id("pelts")), p.revision()),
        (Status::Active, Some(2), 3)
    );
    let again = reduced
        .propose_inventory(&c, &i, &bag(&i, 3))
        .unwrap()
        .journal;
    assert_eq!(again.progress(&id("delivery")).unwrap().revision(), 4);
    assert_eq!(
        again.progress(&id("delivery")).unwrap().status(),
        Status::Ready
    );
    assert_eq!(
        Journal::restore(&c, &again.snapshot(&c).unwrap()).unwrap(),
        again
    );
}
#[test]
fn old_obtained_items_and_cached_ready_never_authorize_missing_current_items() {
    let c = parse(&raw()).unwrap();
    let i = items();
    assert!(
        Journal::default()
            .propose_inventory(&c, &i, &bag(&i, 3))
            .unwrap()
            .changed
            .is_empty()
    );
    let ready = accepted(&c)
        .propose_inventory(&c, &i, &bag(&i, 3))
        .unwrap()
        .journal;
    assert_eq!(
        ready.propose_trusted_event(&c, &EventKey::PossessItem(item())),
        Err(Error::InvalidProgress)
    );
    assert_eq!(
        ready.propose_turn_in(&c, &id("delivery"), &id("guide"), 2),
        Err(Error::InvalidProgress)
    );
    assert_eq!(
        ready.propose_possession_turn_in(&c, &i, &bag(&i, 2), &id("delivery"), &id("guide"), 2),
        Err(Error::NotReady)
    );
    assert_eq!(
        ready.propose_possession_turn_in(&c, &i, &bag(&i, 3), &id("delivery"), &id("guide"), 1),
        Err(Error::StaleRevision)
    );
    assert_eq!(
        ready.propose_possession_turn_in(&c, &i, &bag(&i, 3), &id("delivery"), &id("other"), 2),
        Err(Error::WrongNpc)
    );
}
#[test]
fn earlier_kill_conversation_and_area_facts_survive_item_loss_without_rewinding_stages() {
    let mut v = raw();
    v["quests"][0]["start_stage"] = json!("history");
    v["quests"][0]["stages"].as_array_mut().unwrap().insert(
        0,
        json!({
        "id":"history","completion":{"kind":"all"},"next":"bring","objectives":[
            {"kind":"kill","id":"k","label":"Kill","creature":"wolf","required":1},
            {"kind":"npc_interaction","id":"n","label":"Talk","npc":"guide","interaction":"road"},
            {"kind":"reach_area","id":"a","label":"Reach","area":"overlook"}]}),
    );
    let c = parse(&v).unwrap();
    let i = items();
    let mut j = accepted(&c);
    assert!(
        j.propose_inventory(&c, &i, &bag(&i, 3))
            .unwrap()
            .changed
            .is_empty()
    );
    j = j
        .propose_trusted_event(&c, &EventKey::Kill(id("wolf")))
        .unwrap()
        .journal;
    j = j
        .propose_trusted_event(
            &c,
            &EventKey::NpcInteraction {
                npc: id("guide"),
                interaction: id("road"),
            },
        )
        .unwrap()
        .journal;
    let areas = j
        .prepare_reached_areas(&c, &BTreeSet::from([id("overlook")]))
        .unwrap();
    j = j.propose_reached_areas(&c, &areas).unwrap().journal;
    j = j.propose_inventory(&c, &i, &bag(&i, 3)).unwrap().journal;
    assert_eq!(j.progress(&id("delivery")).unwrap().status(), Status::Ready);
    j = j.propose_inventory(&c, &i, &bag(&i, 0)).unwrap().journal;
    let p = j.progress(&id("delivery")).unwrap();
    assert_eq!(p.stage(), &id("bring"));
    assert_eq!(p.status(), Status::Active);
    for fact in ["k", "n", "a"] {
        assert_eq!(p.count(&id(fact)), Some(1));
    }
    assert_eq!(p.count(&id("pelts")), Some(0));
    assert_eq!(Journal::restore(&c, &j.snapshot(&c).unwrap()).unwrap(), j);
}
#[test]
fn turnin_consumes_across_stacks_and_updates_other_quests_but_completed_delivery_is_sticky() {
    let mut v = raw();
    let mut second = v["quests"][0].clone();
    second["id"] = json!("delivery2");
    second["start_stage"] = json!("bring2");
    second["stages"][0]["id"] = json!("bring2");
    second["stages"][0]["objectives"][0]["id"] = json!("pelts2");
    v["quests"].as_array_mut().unwrap().push(second);
    let c = parse(&v).unwrap();
    let i = items();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(&i, item(), 2);
    slots[7] = Stack::new(&i, item(), 1);
    let b = Bag::from_slots(&i, 4, slots).unwrap();
    let j = accepted(&c)
        .propose_accept(&c, &id("delivery2"), &id("guide"))
        .unwrap()
        .journal
        .propose_inventory(&c, &i, &b)
        .unwrap()
        .journal;
    let proposal = j
        .propose_possession_turn_in(&c, &i, &b, &id("delivery"), &id("guide"), 2)
        .unwrap();
    assert_eq!(proposal.bag.revision(), 5);
    assert!(proposal.bag.slots().iter().all(Option::is_none));
    assert_eq!(
        proposal.quest.changed,
        vec![id("delivery"), id("delivery2")]
    );
    assert_eq!(proposal.quest.reward.as_ref().unwrap().experience, 25);
    let p = proposal.quest.journal.progress(&id("delivery")).unwrap();
    assert_eq!(p.status(), Status::Completed);
    assert_eq!(p.count(&id("pelts")), Some(3));
    let other = proposal.quest.journal.progress(&id("delivery2")).unwrap();
    assert_eq!(other.status(), Status::Active);
    assert_eq!(other.count(&id("pelts2")), Some(0));
    let refreshed = proposal
        .quest
        .journal
        .propose_inventory(&c, &i, &proposal.bag)
        .unwrap();
    assert!(refreshed.changed.is_empty());
    assert_eq!(refreshed.journal.progress(&id("delivery")).unwrap(), p);
    assert_eq!(
        Journal::restore_quests(&c, &refreshed.journal.quest_snapshots(&c).unwrap()).unwrap(),
        refreshed.journal
    );
}
#[test]
fn nonconsuming_requirements_keep_items_and_exhausted_consumption_fails_without_completion() {
    let i = items();
    let mut v = raw();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(&i, item(), 3);
    let b = Bag::from_slots(&i, MAX_REVISION, slots).unwrap();
    let c = parse(&v).unwrap();
    let j = accepted(&c).propose_inventory(&c, &i, &b).unwrap().journal;
    assert_eq!(
        j.propose_possession_turn_in(&c, &i, &b, &id("delivery"), &id("guide"), 2),
        Err(Error::RevisionExhausted)
    );
    v["quests"][0]["stages"][0]["objectives"][0]["consume_on_turn_in"] = json!(false);
    let c = parse(&v).unwrap();
    let j = accepted(&c).propose_inventory(&c, &i, &b).unwrap().journal;
    let p = j
        .propose_possession_turn_in(&c, &i, &b, &id("delivery"), &id("guide"), 2)
        .unwrap();
    assert_eq!(p.bag, b);
    assert_eq!(
        p.quest.journal.progress(&id("delivery")).unwrap().status(),
        Status::Completed
    );
}
#[test]
fn item_binding_refuses_unknown_and_unreachable_counts_before_journal_mutation() {
    let i = items();
    let mut v = raw();
    v["quests"][0]["stages"][0]["objectives"][0]["item"] = json!("unknown");
    let c = parse(&v).unwrap();
    assert!(c.validate_possession_items(&i).is_err());
    assert_eq!(
        accepted(&c).propose_inventory(&c, &i, &bag(&i, 3)),
        Err(Error::DefinitionMismatch)
    );
    v["quests"][0]["stages"][0]["objectives"][0]["item"] = json!("theia.dev.training-sword");
    v["quests"][0]["stages"][0]["objectives"][0]["required"] = json!(17);
    let c = parse(&v).unwrap();
    assert!(c.validate_possession_items(&i).is_err());
}
