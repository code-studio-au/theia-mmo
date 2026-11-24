//! Logical proposals only; these fixtures establish no live NPC entitlement.
use super::*;
use crate::{
    content::ContentId,
    inventory::{self, Catalog as Items, Stack},
    quests::{
        catalog,
        journal::{Progress, Status},
    },
};
use serde_json::{Value, json};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn objective(quest: &str, item: &str, consume: bool) -> Value {
    json!({"id":format!("{quest}-objective"),"label":"Bring items","kind":"possess_item","item":item,"required":3,"consume_on_turn_in":consume})
}
fn quest(name: &str, objective: Value) -> Value {
    json!({"id":name,"version":1,"title":"Development","description":"Test","prerequisites":[],"giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":format!("{name}-stage"),"stages":[{"id":format!("{name}-stage"),"completion":{"kind":"all"},"next":null,"objectives":[objective]}]})
}
fn context(delivery: bool, grants: Value, other: Option<&str>) -> Catalog {
    let items=Items::from_json(&serde_json::to_vec(&json!({"schema":2,"id":"items","items":[
        {"id":"pelt","name":"Pelt","maximum_stack":20}, {"id":"token","name":"Token","maximum_stack":10},
        {"id":"filler","name":"Filler","maximum_stack":1},
        {"id":"sword","name":"Sword","maximum_stack":1,"main_hand_appearance":"sword-appearance"}
    ]})).unwrap()).unwrap();
    let objective = if delivery {
        objective("delivery", "pelt", true)
    } else {
        json!({"id":"delivery-objective","label":"One kill","kind":"kill","creature":"wolf","required":1})
    };
    let mut quests = vec![quest("delivery", objective)];
    if let Some(item) = other {
        quests.push(quest("watcher", self::objective("watcher", item, false)));
    }
    let quests = catalog::Catalog::from_json(
        &serde_json::to_vec(&json!({"schema":3,"id":"quests","revision":1,"quests":quests}))
            .unwrap(),
    )
    .unwrap();
    Catalog::from_json(&serde_json::to_vec(&json!({"schema":1,"id":"quest-rewards","revision":1,"quest_catalog":"quests","item_catalog":"items","rewards":[{"quest":"delivery","grants":grants}]})).unwrap(),&quests,&items).unwrap()
}
fn tokens(n: u32) -> Value {
    json!([{"item":"token","count":n}])
}
fn bag(c: &Catalog, full: bool, pelts: u32, tokens: u32, revision: u64) -> Bag {
    let items = c.item_catalog();
    let mut slots = std::array::from_fn(|_| {
        if full {
            Stack::new(items, id("filler"), 1)
        } else {
            None
        }
    });
    if pelts > 0 {
        slots[0] = Stack::new(items, id("pelt"), pelts);
    }
    if tokens > 0 {
        slots[1] = Stack::new(items, id("token"), tokens);
    }
    Bag::from_parts(items, revision, slots, Stack::new(items, id("sword"), 1)).unwrap()
}
fn ready(c: &Catalog, bag: &Bag) -> Journal {
    let quests = c.quest_catalog();
    let definition = quests.quest(&id("delivery")).unwrap();
    let mut journal = if definition.has_possession_objectives() {
        Journal::default()
            .propose_accept(quests, &id("delivery"), &id("guide"))
            .unwrap()
            .journal
            .propose_inventory(quests, c.item_catalog(), bag)
            .unwrap()
            .journal
    } else {
        let state = Progress::from_parts(
            definition,
            2,
            Status::Ready,
            definition.start_stage().clone(),
            vec![(id("delivery-objective"), 1)],
        )
        .unwrap();
        Journal::from_progress(quests, vec![(id("delivery"), state)]).unwrap()
    };
    if quests.quest(&id("watcher")).is_some() {
        journal = journal
            .propose_accept(quests, &id("watcher"), &id("guide"))
            .unwrap()
            .journal
            .propose_inventory(quests, c.item_catalog(), bag)
            .unwrap()
            .journal;
    }
    assert_eq!(
        journal.progress(&id("delivery")).unwrap().status(),
        Status::Ready
    );
    journal
}
fn turn(c: &Catalog, j: &Journal, b: &Bag) -> Result<PossessionTurnIn, TurnInError> {
    c.propose_turn_in(
        j,
        b,
        TurnIn {
            quest: &id("delivery"),
            npc: &id("guide"),
            quest_revision: j.progress(&id("delivery")).unwrap().revision(),
            inventory_revision: b.revision(),
        },
    )
}
fn count(bag: &Bag, item: &str) -> u32 {
    bag.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &id(item))
        .map(Stack::count)
        .sum()
}
#[test]
fn consumption_grants_merge_before_empty_slots_preserve_equipment_and_return_xp_proposal_once() {
    let c = context(true, tokens(5), None);
    let b = bag(&c, false, 3, 8, 7);
    let j = ready(&c, &b);
    let before = (j.clone(), b.clone());
    let p = turn(&c, &j, &b).unwrap();
    assert_eq!(p.bag.revision(), 8);
    assert_eq!(count(&p.bag, "pelt"), 0);
    assert_eq!(count(&p.bag, "token"), 13);
    assert_eq!(p.bag.slots()[1].as_ref().unwrap().count(), 10);
    assert_eq!(p.bag.slots()[0].as_ref().unwrap().count(), 3);
    assert_eq!(p.bag.main_hand(), b.main_hand());
    assert_eq!(
        p.quest.journal.progress(&id("delivery")).unwrap().status(),
        Status::Completed
    );
    assert_eq!(p.quest.reward.as_ref().unwrap().experience, 25);
    assert_eq!((j, b), before);
    assert!(
        turn(&c, &p.quest.journal, &p.bag).is_err(),
        "Completed state cannot propose another grant"
    );
}
#[test]
fn full_backpack_refuses_ordinary_completion_and_keeps_every_input_unchanged() {
    let c = context(false, tokens(1), None);
    let b = bag(&c, true, 0, 0, 7);
    let j = ready(&c, &b);
    let before = (j.clone(), b.clone());
    assert_eq!(
        turn(&c, &j, &b),
        Err(TurnInError::Inventory(rewards::Error::NoCapacity))
    );
    assert_eq!((j, b), before);
}
#[test]
fn delivery_can_free_a_slot_for_its_reward_in_a_full_bag_with_one_revision() {
    let c = context(true, tokens(1), None);
    let b = bag(&c, true, 3, 0, 7);
    let j = ready(&c, &b);
    let p = turn(&c, &j, &b).unwrap();
    assert_eq!(p.bag.revision(), 8);
    assert_eq!(count(&p.bag, "token"), 1);
    assert_eq!(count(&p.bag, "pelt"), 0);
    assert_eq!(p.bag.slots()[0].as_ref().unwrap().item(), &id("token"));
}
#[test]
fn whole_bundle_capacity_failure_never_returns_partial_consumption_or_first_grant() {
    let c = context(
        true,
        json!([{"item":"token","count":1},{"item":"pelt","count":1}]),
        None,
    );
    let b = bag(&c, true, 3, 0, 7);
    let j = ready(&c, &b);
    let before = (j.clone(), b.clone());
    assert_eq!(
        turn(&c, &j, &b),
        Err(TurnInError::Inventory(rewards::Error::NoCapacity))
    );
    assert_eq!((j, b), before);
}
#[test]
fn other_quests_see_only_final_holdings_without_transient_revision_churn() {
    for replace in [false, true] {
        let grants = if replace {
            json!([{"item":"pelt","count":3}])
        } else {
            tokens(1)
        };
        let c = context(true, grants, Some("pelt"));
        let b = bag(&c, false, 3, 0, 7);
        let j = ready(&c, &b);
        let before = j.progress(&id("watcher")).unwrap();
        let p = turn(&c, &j, &b).unwrap();
        let after = p.quest.journal.progress(&id("watcher")).unwrap();
        if replace {
            assert_eq!(after, before);
            assert!(!p.quest.changed.contains(&id("watcher")));
        } else {
            assert_eq!(after.status(), Status::Active);
            assert_eq!(after.revision(), before.revision() + 1);
            assert!(p.quest.changed.contains(&id("watcher")));
        }
    }
}
#[test]
fn reward_holdings_can_make_another_active_item_quest_ready() {
    let c = context(true, tokens(3), Some("token"));
    let b = bag(&c, false, 3, 0, 7);
    let j = ready(&c, &b);
    assert_eq!(j.progress(&id("watcher")).unwrap().status(), Status::Active);
    let p = turn(&c, &j, &b).unwrap();
    assert_eq!(
        p.quest.journal.progress(&id("watcher")).unwrap().status(),
        Status::Ready
    );
    assert!(p.quest.changed.contains(&id("watcher")));
}
#[test]
fn stale_inventory_quest_wrong_npc_and_missing_required_items_refuse() {
    let c = context(true, tokens(1), None);
    let b = bag(&c, false, 3, 0, 7);
    let j = ready(&c, &b);
    let q = id("delivery");
    let npc = id("guide");
    let revision = j.progress(&q).unwrap().revision();
    let good = TurnIn {
        quest: &q,
        npc: &npc,
        quest_revision: revision,
        inventory_revision: 7,
    };
    assert_eq!(
        c.propose_turn_in(
            &j,
            &b,
            TurnIn {
                inventory_revision: 6,
                ..good
            }
        ),
        Err(TurnInError::Inventory(rewards::Error::StaleRevision))
    );
    assert_eq!(
        c.propose_turn_in(
            &j,
            &b,
            TurnIn {
                quest_revision: revision + 1,
                ..good
            }
        ),
        Err(TurnInError::Quest(journal::Error::StaleRevision))
    );
    assert_eq!(
        c.propose_turn_in(
            &j,
            &b,
            TurnIn {
                npc: &id("wrong-guide"),
                ..good
            }
        ),
        Err(TurnInError::Quest(journal::Error::WrongNpc))
    );
    let missing = bag(&c, false, 2, 0, 7);
    assert_eq!(
        turn(&c, &j, &missing),
        Err(TurnInError::Quest(journal::Error::NotReady))
    );
    assert_eq!(
        c.propose_turn_in(
            &j,
            &b,
            TurnIn {
                quest: &id("missing"),
                ..good
            }
        ),
        Err(TurnInError::NoReward)
    );
}
#[test]
fn combined_mutation_uses_last_available_bag_revision_and_exhaustion_is_atomic() {
    let c = context(true, tokens(1), None);
    let b = bag(&c, true, 3, 0, inventory::MAX_REVISION - 1);
    let j = ready(&c, &b);
    let p = turn(&c, &j, &b).unwrap();
    assert_eq!(p.bag.revision(), inventory::MAX_REVISION);
    let exhausted = bag(&c, true, 3, 0, inventory::MAX_REVISION);
    let before = (j.clone(), exhausted.clone());
    assert_eq!(
        turn(&c, &j, &exhausted),
        Err(TurnInError::Quest(journal::Error::RevisionExhausted))
    );
    assert_eq!((j, exhausted), before);
}
