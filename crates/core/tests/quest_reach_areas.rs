use serde_json::{Value, json};
use std::collections::BTreeSet;
use theia_core::{
    content::ContentId,
    quests::{
        catalog::{Catalog, EventKey},
        journal::{Error, Journal, MAX_REACHED_AREA_GOALS, Status},
    },
};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn area(n: usize) -> Value {
    json!({"kind":"reach_area","id":format!("o{n}"),"label":"Reach","area":format!("area{n}")})
}
fn fixture() -> Value {
    json!({"schema":2,"id":"catalog","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Explore","description":"Test","prerequisites":[],
        "giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"first","stages":[
            {"id":"first","completion":{"kind":"all"},"next":"second","objectives":[area(0)]},
            {"id":"second","completion":{"kind":"all"},"next":null,"objectives":[area(1)]}
        ]
    }]})
}
fn parse(v: &Value) -> Catalog {
    Catalog::from_json(&serde_json::to_vec(v).unwrap()).unwrap()
}
fn accepted(c: &Catalog) -> Journal {
    Journal::default()
        .propose_accept(c, &id("quest"), &id("guide"))
        .unwrap()
        .journal
}
fn inside(names: &[&str]) -> BTreeSet<ContentId> {
    names.iter().map(|s| id(s)).collect()
}
#[test]
fn schema_and_selector_validation_remain_strict() {
    let v = fixture();
    let c = parse(&v);
    assert_eq!(c.schema(), 2);
    assert_eq!(c.matching(&EventKey::ReachArea(id("area0"))).len(), 1);
    for schema in [1, 4] {
        let mut bad = v.clone();
        bad["schema"] = json!(schema);
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (key, value) in [("required", json!(2)), ("area", json!("first"))] {
        let mut bad = v.clone();
        bad["quests"][0]["stages"][0]["objectives"][0][key] = value;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = v;
    let mut duplicate = area(0);
    duplicate["id"] = json!("other");
    bad["quests"][0]["stages"][0]["objectives"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
}
#[test]
fn observation_never_retroactively_accepts_or_cascades_and_replay_is_inert() {
    let c = parse(&fixture());
    let empty = Journal::default();
    let old = empty
        .prepare_reached_areas(&c, &inside(&["area0", "area1"]))
        .unwrap();
    assert!(old.is_empty());
    let j = accepted(&c);
    assert_eq!(j.propose_reached_areas(&c, &old).unwrap().journal, j);
    let batch = j
        .prepare_reached_areas(&c, &inside(&["area0", "area1"]))
        .unwrap();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch.goals()[0].stage(), &id("first"));
    let proposal = j.propose_reached_areas(&c, &batch).unwrap();
    assert_eq!(proposal.changed, vec![id("quest")]);
    assert!(proposal.reward.is_none());
    let next = proposal.journal;
    assert_eq!(next.progress(&id("quest")).unwrap().stage(), &id("second"));
    assert_eq!(
        next.progress(&id("quest")).unwrap().count(&id("o1")),
        Some(0)
    );
    assert_eq!(
        next.propose_reached_areas(&c, &batch).unwrap().journal,
        next
    );
    let fresh = next.prepare_reached_areas(&c, &inside(&["area1"])).unwrap();
    let ready = next.propose_reached_areas(&c, &fresh).unwrap().journal;
    assert_eq!(
        ready.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    assert!(
        ready
            .prepare_reached_areas(&c, &inside(&["area0", "area1"]))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        j.propose_trusted_event(&c, &EventKey::ReachArea(id("area0"))),
        Err(Error::InvalidProgress)
    );
}
#[test]
fn same_selector_in_later_stage_cannot_consume_old_observation() {
    let mut v = fixture();
    v["quests"][0]["stages"][1]["objectives"][0]["area"] = json!("area0");
    let c = parse(&v);
    let j = accepted(&c);
    let batch = j.prepare_reached_areas(&c, &inside(&["area0"])).unwrap();
    let next = j.propose_reached_areas(&c, &batch).unwrap().journal;
    assert_eq!(
        next.propose_reached_areas(&c, &batch).unwrap().journal,
        next
    );
    let fresh = next.prepare_reached_areas(&c, &inside(&["area0"])).unwrap();
    assert_eq!(
        next.propose_reached_areas(&c, &fresh)
            .unwrap()
            .journal
            .progress(&id("quest"))
            .unwrap()
            .status(),
        Status::Ready
    );
}
#[test]
fn other_progress_in_original_stage_does_not_discard_an_observation() {
    let mut v = fixture();
    v["quests"][0]["stages"][0]["objectives"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"kill","id":"kill","label":"Kill","creature":"wolf","required":2}));
    let c = parse(&v);
    let j = accepted(&c);
    let batch = j.prepare_reached_areas(&c, &inside(&["area0"])).unwrap();
    let j = j
        .propose_trusted_event(&c, &EventKey::Kill(id("wolf")))
        .unwrap()
        .journal;
    let j = j.propose_reached_areas(&c, &batch).unwrap().journal;
    assert_eq!(j.progress(&id("quest")).unwrap().stage(), &id("first"));
    assert_eq!(j.progress(&id("quest")).unwrap().revision(), 3);
    assert_eq!(j.progress(&id("quest")).unwrap().count(&id("o0")), Some(1));
    assert_eq!(j.propose_reached_areas(&c, &batch).unwrap().journal, j);
}
#[test]
fn overlapping_goals_respect_all_any_and_at_least_quorums() {
    for (completion, expected) in [
        (json!({"kind":"all"}), 3),
        (json!({"kind":"any"}), 1),
        (json!({"kind":"at_least","count":2}), 2),
    ] {
        let mut v = fixture();
        v["quests"][0]["stages"][0]["completion"] = completion;
        v["quests"][0]["stages"][0]["objectives"] = json!([area(0), area(2), area(3)]);
        let c = parse(&v);
        let j = accepted(&c);
        let batch = j
            .prepare_reached_areas(&c, &inside(&["area0", "area1", "area2", "area3"]))
            .unwrap();
        assert_eq!(batch.len(), 3);
        let p = j.propose_reached_areas(&c, &batch).unwrap();
        assert_eq!(p.changed, vec![id("quest")]);
        let state = p.journal.progress(&id("quest")).unwrap();
        assert_eq!(state.stage(), &id("second"));
        assert_eq!(state.revision(), 1 + expected);
        assert_eq!(state.count(&id("o1")), Some(0));
    }
}
#[test]
fn metadata_mismatch_refuses_and_new_quests_cannot_consume_old_batch() {
    let mut v = fixture();
    let mut second = v["quests"][0].clone();
    second["id"] = json!("quest2");
    second["start_stage"] = json!("third");
    second["stages"] = json!([{"id":"third","completion":{"kind":"all"},"next":null,
        "objectives":[{"kind":"reach_area","id":"o2","label":"Reach","area":"area0"}]}]);
    v["quests"].as_array_mut().unwrap().push(second);
    let c = parse(&v);
    let j = accepted(&c);
    let batch = j.prepare_reached_areas(&c, &inside(&["area0"])).unwrap();
    let j = j
        .propose_accept(&c, &id("quest2"), &id("guide"))
        .unwrap()
        .journal;
    let next = j.propose_reached_areas(&c, &batch).unwrap().journal;
    assert_eq!(
        next.progress(&id("quest2")).unwrap().count(&id("o2")),
        Some(0)
    );
    for (field, value) in [("id", json!("other-catalog")), ("revision", json!(2))] {
        let mut other = v.clone();
        other[field] = value;
        let other = parse(&other);
        assert_eq!(
            j.propose_reached_areas(&other, &batch),
            Err(Error::DefinitionMismatch)
        );
    }
    let mut other = v;
    other["quests"][0]["version"] = json!(2);
    let other = parse(&other);
    let other_j = accepted(&other);
    assert_eq!(
        other_j.propose_reached_areas(&other, &batch),
        Err(Error::DefinitionMismatch)
    );
}
#[test]
fn batch_and_input_are_bounded_at_sixteen_active_times_eight_goals() {
    let mut v = fixture();
    let template = v["quests"][0].clone();
    v["quests"] = json!([]);
    for q in 0..16 {
        let mut quest = template.clone();
        quest["id"] = json!(format!("quest{q}"));
        quest["start_stage"] = json!(format!("stage{q}"));
        let goals:Vec<_>=(0..8).map(|a|json!({"kind":"reach_area","id":format!("goal{q}-{a}"),"label":"Reach","area":format!("area{a}")})).collect();
        quest["stages"] = json!([{"id":format!("stage{q}"),"completion":{"kind":"all"},"next":null,"objectives":goals}]);
        v["quests"].as_array_mut().unwrap().push(quest);
    }
    let c = parse(&v);
    let mut j = Journal::default();
    for q in 0..16 {
        j = j
            .propose_accept(&c, &id(&format!("quest{q}")), &id("guide"))
            .unwrap()
            .journal;
    }
    let set = (0..8).map(|a| id(&format!("area{a}"))).collect();
    let batch = j.prepare_reached_areas(&c, &set).unwrap();
    assert_eq!(batch.len(), MAX_REACHED_AREA_GOALS);
    let p = j.propose_reached_areas(&c, &batch).unwrap();
    assert_eq!(p.changed.len(), 16);
    for q in 0..16 {
        assert_eq!(
            p.journal
                .progress(&id(&format!("quest{q}")))
                .unwrap()
                .status(),
            Status::Ready
        );
    }
    let too_many = (0..33).map(|a| id(&format!("area{a}"))).collect();
    assert_eq!(j.prepare_reached_areas(&c, &too_many), Err(Error::Capacity));
}
#[test]
fn reached_counts_roundtrip_existing_saved_shape_without_migration() {
    let c = parse(&fixture());
    let j = accepted(&c);
    let batch = j.prepare_reached_areas(&c, &inside(&["area0"])).unwrap();
    let j = j.propose_reached_areas(&c, &batch).unwrap().journal;
    let bytes = j.snapshot(&c).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap()["schema"],
        json!(1)
    );
    assert_eq!(Journal::restore(&c, &bytes).unwrap(), j);
    let rows = j.quest_snapshots(&c).unwrap();
    assert_eq!(Journal::restore_quests(&c, &rows).unwrap(), j);
    let mut other = fixture();
    other["quests"][0]["version"] = json!(2);
    assert!(Journal::restore(&parse(&other), &bytes).is_err());
}
#[test]
fn batch_head_is_monotonic_covers_multi_objective_revisions_and_refuses_exhaustion() {
    let mut v = fixture();
    v["quests"][0]["stages"][0]["objectives"] = json!([area(0), area(2), area(3)]);
    let c = parse(&v);
    let j = accepted(&c);
    let batch = j
        .prepare_reached_areas(&c, &inside(&["area0", "area2", "area3"]))
        .unwrap();
    let p = j.propose_reached_areas(&c, &batch).unwrap().journal;
    assert_eq!(p.progress(&id("quest")).unwrap().revision(), 4);
    assert_eq!(p.next_head_revision(1), Ok(4));
    assert_eq!(p.next_head_revision(10), Ok(11));
    assert_eq!(
        p.next_head_revision(theia_core::quests::MAX_REVISION),
        Err(Error::RevisionExhausted)
    );
    assert_eq!(Journal::default().next_head_revision(0), Ok(1));
}
