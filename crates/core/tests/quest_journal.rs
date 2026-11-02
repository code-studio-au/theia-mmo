//! Pure proposals only; SQL receipts, World eligibility and actual rewards are
//! separate acceptance boundaries.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    quests::{
        catalog::{Catalog, EventKey},
        journal::{Error, Journal, MAX_JOURNAL_BYTES, Status},
    },
};
const DATA: &[u8] = include_bytes!("../../../content/quests/development-journal.json");
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn first() -> ContentId {
    id("theia.dev.quest-introduction")
}
fn second() -> ContentId {
    id("theia.dev.quest-follow-up")
}
fn npc() -> ContentId {
    id("theia.dev.guide")
}
fn interaction(name: &str) -> EventKey {
    EventKey::NpcInteraction {
        npc: npc(),
        interaction: id(&format!("theia.dev.interaction.{name}")),
    }
}
fn catalog(v: &Value) -> Catalog {
    Catalog::from_json(&serde_json::to_vec(v).unwrap()).unwrap()
}
fn ready(c: &Catalog) -> Journal {
    let mut j = Journal::default()
        .propose_accept(c, &first(), &npc())
        .unwrap()
        .journal;
    for name in ["road", "supplies", "report"] {
        j = j
            .propose_trusted_event(c, &interaction(name))
            .unwrap()
            .journal;
    }
    assert_eq!(j.progress(&first()).unwrap().status(), Status::Ready);
    j
}
#[test]
fn two_quest_flow_requires_completed_prerequisite_and_proposes_one_xp_completion() {
    let c = Catalog::from_json(DATA).unwrap();
    let initial = Journal::default();
    assert_eq!(
        initial.propose_accept(&c, &second(), &npc()),
        Err(Error::Prerequisite)
    );
    let j = ready(&c);
    assert_eq!(j.progress(&first()).unwrap().revision(), 4);
    assert_eq!(
        j.propose_accept(&c, &second(), &npc()),
        Err(Error::Prerequisite)
    );
    let p = j.propose_turn_in(&c, &first(), &npc(), 4).unwrap();
    assert_eq!(p.reward.as_ref().unwrap().experience, 25);
    assert_eq!(p.reward.as_ref().unwrap().completed_revision, 5);
    assert_eq!(j.progress(&first()).unwrap().status(), Status::Ready);
    let mut j = p
        .journal
        .propose_accept(&c, &second(), &npc())
        .unwrap()
        .journal;
    let kill = EventKey::Kill(id("theia.dev.training-creature"));
    for _ in 0..3 {
        j = j.propose_trusted_event(&c, &kill).unwrap().journal;
    }
    assert_eq!(j.progress(&second()).unwrap().status(), Status::Active);
    j = j
        .propose_trusted_event(&c, &interaction("training"))
        .unwrap()
        .journal;
    assert_eq!(j.progress(&second()).unwrap().status(), Status::Ready);
    let revision = j.progress(&second()).unwrap().revision();
    let p = j.propose_turn_in(&c, &second(), &npc(), revision).unwrap();
    assert_eq!(p.reward.unwrap().experience, 25);
    assert_eq!(p.journal.quests().count(), 2);
    p.journal.validate(&c).unwrap();
}
#[test]
fn preaccept_wrong_semantic_action_and_completed_events_never_advance() {
    let c = Catalog::from_json(DATA).unwrap();
    let j = Journal::default();
    let kill = EventKey::Kill(id("theia.dev.training-creature"));
    assert_eq!(j.propose_trusted_event(&c, &kill).unwrap().journal, j);
    let accepted = j.propose_accept(&c, &first(), &npc()).unwrap().journal;
    for event in [
        kill,
        interaction("unknown"),
        EventKey::NpcInteraction {
            npc: id("theia.dev.other"),
            interaction: id("theia.dev.interaction.road"),
        },
        interaction("report"),
    ] {
        let p = accepted.propose_trusted_event(&c, &event).unwrap();
        assert!(p.changed.is_empty());
        assert_eq!(p.journal, accepted);
    }
    let j = ready(&c);
    assert_eq!(
        j.propose_trusted_event(&c, &interaction("road"))
            .unwrap()
            .journal,
        j
    );
    let j = j.propose_turn_in(&c, &first(), &npc(), 4).unwrap().journal;
    assert_eq!(
        j.propose_trusted_event(&c, &interaction("report"))
            .unwrap()
            .journal,
        j
    );
    assert_eq!(
        j.propose_accept(&c, &first(), &npc()),
        Err(Error::AlreadyAccepted)
    );
    assert_eq!(
        j.propose_turn_in(&c, &first(), &npc(), 5),
        Err(Error::NotReady)
    );
}
#[test]
fn one_event_cannot_cascade_across_stages_even_with_the_same_selector() {
    for rule in [json!({"kind":"any"}), json!({"kind":"at_least","count":1})] {
        let mut v: Value = serde_json::from_slice(DATA).unwrap();
        v["quests"][0]["stages"][0]["completion"] = rule;
        v["quests"][0]["stages"][1]["objectives"][0]["interaction"] =
            json!("theia.dev.interaction.road");
        let c = catalog(&v);
        let j = Journal::default()
            .propose_accept(&c, &first(), &npc())
            .unwrap()
            .journal;
        let p = j.propose_trusted_event(&c, &interaction("road")).unwrap();
        assert_eq!(p.changed, vec![first()]);
        let state = p.journal.progress(&first()).unwrap();
        assert_eq!(state.status(), Status::Active);
        assert_eq!(
            state.count(&id("theia.dev.quest-introduction.objective.report")),
            Some(0)
        );
        let p = p
            .journal
            .propose_trusted_event(&c, &interaction("road"))
            .unwrap();
        assert_eq!(
            p.journal.progress(&first()).unwrap().status(),
            Status::Ready
        );
    }
}
#[test]
fn shared_event_can_advance_two_active_quests_without_sharing_character_state() {
    let mut v: Value = serde_json::from_slice(DATA).unwrap();
    v["quests"][1]["prerequisites"] = json!([]);
    v["quests"][0]["stages"][0]["objectives"][0] = json!({"kind":"kill","id":"theia.dev.quest-introduction.objective.road","label":"Practice","creature":"theia.dev.training-creature","required":2});
    let c = catalog(&v);
    let mut a = Journal::default();
    for q in [first(), second()] {
        a = a.propose_accept(&c, &q, &npc()).unwrap().journal;
    }
    let b = a.clone();
    let p = a
        .propose_trusted_event(&c, &EventKey::Kill(id("theia.dev.training-creature")))
        .unwrap();
    assert_eq!(p.changed.len(), 2);
    assert_eq!(p.journal.progress(&first()).unwrap().revision(), 2);
    assert_eq!(p.journal.progress(&second()).unwrap().revision(), 2);
    assert_eq!(b.progress(&first()).unwrap().revision(), 1);
    assert_eq!(
        Journal::restore(&c, &p.journal.snapshot(&c).unwrap()).unwrap(),
        p.journal
    );
}
#[test]
fn snapshot_checks_versions_counts_stage_order_revisions_and_duplicate_keys() {
    let c = Catalog::from_json(DATA).unwrap();
    let j = Journal::default()
        .propose_accept(&c, &first(), &npc())
        .unwrap()
        .journal;
    let raw: Value = serde_json::from_slice(&j.snapshot(&c).unwrap()).unwrap();
    for (field, value) in [
        ("definition_version", json!(2)),
        ("revision", json!(999)),
        ("status", json!("ready")),
        ("stage", json!("theia.dev.missing")),
        ("unknown", json!(true)),
    ] {
        let mut v = raw.clone();
        v["quests"][0][field] = value;
        assert!(Journal::restore(&c, &serde_json::to_vec(&v).unwrap()).is_err());
    }
    let mut v = raw.clone();
    v["quests"][0]["objectives"][0]["count"] = json!(101);
    assert!(Journal::restore(&c, &serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = raw.clone();
    let duplicate = v["quests"][0].clone();
    v["quests"].as_array_mut().unwrap().push(duplicate);
    assert!(Journal::restore(&c, &serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = raw.clone();
    let duplicate = v["quests"][0]["objectives"][0].clone();
    v["quests"][0]["objectives"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(Journal::restore(&c, &serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = raw.clone();
    v["catalog_revision"] = json!(2);
    assert_eq!(
        Journal::restore(&c, &serde_json::to_vec(&v).unwrap()),
        Err(Error::DefinitionMismatch)
    );
    assert_eq!(
        Journal::restore(&c, &vec![b' '; MAX_JOURNAL_BYTES + 1]),
        Err(Error::InvalidSnapshot)
    );
}
#[test]
fn actions_refuse_wrong_npc_early_turn_in_and_stale_revisions_without_mutating_input() {
    let c = Catalog::from_json(DATA).unwrap();
    let j = Journal::default();
    assert_eq!(
        j.propose_accept(&c, &first(), &id("theia.dev.other")),
        Err(Error::WrongNpc)
    );
    let j = j.propose_accept(&c, &first(), &npc()).unwrap().journal;
    assert_eq!(
        j.propose_turn_in(&c, &first(), &npc(), 1),
        Err(Error::NotReady)
    );
    let j = ready(&c);
    let before = j.clone();
    assert_eq!(
        j.propose_turn_in(&c, &first(), &npc(), 3),
        Err(Error::StaleRevision)
    );
    assert_eq!(
        j.propose_turn_in(&c, &first(), &id("theia.dev.other"), 4),
        Err(Error::WrongNpc)
    );
    assert_eq!(j, before);
}
#[test]
fn active_journal_capacity_is_bounded_before_acceptance() {
    let raw: Value = serde_json::from_slice(DATA).unwrap();
    let template = raw["quests"][1].clone();
    let mut v = raw.clone();
    v["quests"] = json!([]);
    for n in 0..17 {
        let source = serde_json::to_string(&template)
            .unwrap()
            .replace("theia.dev.quest-follow-up", &format!("theia.dev.q{n}"));
        let mut q: Value = serde_json::from_str(&source).unwrap();
        q["prerequisites"] = json!([]);
        v["quests"].as_array_mut().unwrap().push(q);
    }
    let c = catalog(&v);
    let mut j = Journal::default();
    for n in 0..16 {
        j = j
            .propose_accept(&c, &id(&format!("theia.dev.q{n}")), &npc())
            .unwrap()
            .journal;
    }
    assert_eq!(
        j.propose_accept(&c, &id("theia.dev.q16"), &npc()),
        Err(Error::Capacity)
    );
    assert_eq!(j.quests().count(), 16);
}

#[test]
fn per_quest_snapshots_bind_row_metadata_and_restore_the_entire_journal() {
    let c = Catalog::from_json(DATA).unwrap();
    let j = ready(&c);
    let rows = j.quest_snapshots(&c).unwrap();
    assert_eq!(Journal::restore_quests(&c, &rows).unwrap(), j);
    let mut bad = rows.clone();
    bad[0].revision += 1;
    assert!(Journal::restore_quests(&c, &bad).is_err());
    let mut bad = rows.clone();
    bad[0].status = Status::Completed;
    assert!(Journal::restore_quests(&c, &bad).is_err());
    let mut bad = rows.clone();
    bad.push(rows[0].clone());
    assert!(Journal::restore_quests(&c, &bad).is_err());
}
