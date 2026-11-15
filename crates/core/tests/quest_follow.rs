use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    quests::{
        catalog::{Catalog, EventKey},
        journal::{Error, Journal, Status},
    },
};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn fixture() -> Value {
    json!({"schema":5,"id":"quests","revision":1,"quests":[{
    "id":"follow","version":1,"title":"Follow the NPC","description":"Stay within ten metres.","prerequisites":[],"giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"travel","stages":[{"id":"travel","completion":{"kind":"all"},"next":null,"objectives":[{"kind":"follow_npc","id":"arrival","label":"Follow the guide","npc":"guide","travel":{"destination":1202,"max_distance_metres":10,"speed_millimetres_per_second":1500}}]}]}]})
}
fn parse(v: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(v).unwrap())
}
#[test]
fn follow_requires_acceptance_then_arrival_then_exact_npc_turn_in() {
    let c = parse(&fixture()).unwrap();
    let quest = id("follow");
    let npc = id("guide");
    let event = EventKey::FollowNpc(npc.clone());
    let empty = Journal::default();
    assert!(
        empty
            .propose_trusted_event(&c, &event)
            .unwrap()
            .changed
            .is_empty()
    );
    let active = empty.propose_accept(&c, &quest, &npc).unwrap().journal;
    assert_eq!(
        active.propose_turn_in(&c, &quest, &npc, 1),
        Err(Error::NotReady)
    );
    let ready = active.propose_trusted_event(&c, &event).unwrap();
    assert!(ready.reward.is_none());
    assert_eq!(
        ready.journal.progress(&quest).unwrap().status(),
        Status::Ready
    );
    assert!(
        ready
            .journal
            .propose_trusted_event(&c, &event)
            .unwrap()
            .changed
            .is_empty()
    );
    assert_eq!(
        ready.journal.propose_turn_in(&c, &quest, &id("other"), 2),
        Err(Error::WrongNpc)
    );
    let done = ready.journal.propose_turn_in(&c, &quest, &npc, 2).unwrap();
    assert_eq!(done.reward.unwrap().experience, 25);
    assert_eq!(
        done.journal.progress(&quest).unwrap().status(),
        Status::Completed
    );
}
#[test]
fn follow_rejects_unknown_fields_versions_ambiguous_contacts_and_bounds() {
    for (field, value) in [
        ("destination", json!(0)),
        ("max_distance_metres", json!(0)),
        ("max_distance_metres", json!(21)),
        ("speed_millimetres_per_second", json!(8001)),
        ("teleport", json!(true)),
    ] {
        let mut v = fixture();
        v["quests"][0]["stages"][0]["objectives"][0]["travel"][field] = value;
        assert!(parse(&v).is_err());
    }
    for field in ["giver", "turn_in"] {
        let mut v = fixture();
        v["quests"][0][field] = json!("other");
        assert!(parse(&v).is_err());
    }
    for schema in [1, 2, 3, 4, 6] {
        let mut v = fixture();
        v["schema"] = json!(schema);
        assert!(parse(&v).is_err());
    }
    let mut v = fixture();
    v["quests"][0]["stages"][0]["completion"] = json!({"kind":"any"});
    assert!(parse(&v).is_err());
}
