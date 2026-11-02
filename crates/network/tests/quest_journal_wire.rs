//! Codec-only synthetic journals. No actor entitlement, SQL or encrypted runtime.
use serde_json::json;
use std::sync::Arc;
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::{
        catalog::EventKey,
        journal::{Journal, Progress, Status},
    },
};
use theia_network::{
    quest_journal::Profile,
    quest_journal_wire::{HEADER_BYTES, MAX_STATE_BYTES, State},
};
const DATA: &[u8] = include_bytes!("../../../content/quests/development-journal.json");
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn actor() -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(91).unwrap(),
        entity: EntityId::new(7).unwrap(),
    }
}
fn profile() -> Profile {
    Profile::from_json(
        DATA,
        &theia_network::progression_wire::Profile::from_json(XP).unwrap(),
    )
    .unwrap()
}
fn state(profile: &Profile, revision: u64, journal: Journal) -> State {
    State {
        actor: actor(),
        profile_hash: profile.hash(),
        revision,
        journal,
    }
}
fn event(name: &str) -> EventKey {
    EventKey::NpcInteraction {
        npc: id("theia.dev.guide"),
        interaction: id(&format!("theia.dev.interaction.{name}")),
    }
}
fn states(p: &Profile) -> Vec<State> {
    let c = p.catalog();
    let guide = id("theia.dev.guide");
    let first = id("theia.dev.quest-introduction");
    let mut out = vec![state(p, 0, Journal::default())];
    let mut j = Journal::default()
        .propose_accept(c, &first, &guide)
        .unwrap()
        .journal;
    out.push(state(p, 1, j.clone()));
    for (n, action) in [(2, "road"), (3, "supplies"), (4, "report")] {
        j = j.propose_trusted_event(c, &event(action)).unwrap().journal;
        out.push(state(p, n, j.clone()));
    }
    j = j.propose_turn_in(c, &first, &guide, 4).unwrap().journal;
    out.push(state(p, 5, j.clone()));
    j = j
        .propose_accept(c, &id("theia.dev.quest-follow-up"), &guide)
        .unwrap()
        .journal;
    out.push(state(p, 6, j));
    out
}
#[test]
fn compact_round_trip_preserves_every_stage_and_accepts_coalesced_and_noop_heads() {
    let p = profile();
    let all = states(&p);
    for (n, s) in all.iter().enumerate() {
        let bytes = s.encode(&p).unwrap();
        assert!(bytes.len() <= MAX_STATE_BYTES);
        let decoded = State::decode(&bytes, &p).unwrap();
        assert_eq!(&decoded, s);
        decoded
            .validate_successor(&p, actor(), n.checked_sub(1).map(|i| &all[i]))
            .unwrap();
    }
    all.last()
        .unwrap()
        .validate_successor(&p, actor(), Some(&all[1]))
        .unwrap();
    let mut noop = all.last().unwrap().clone();
    noop.revision += 1;
    noop.validate_successor(&p, actor(), all.last()).unwrap();
}
#[test]
fn malformed_noncanonical_truncated_and_foreign_frames_refuse() {
    let p = profile();
    let all = states(&p);
    let encoded = all[1].encode(&p).unwrap();
    for n in 0..encoded.len() {
        assert!(State::decode(&encoded[..n], &p).is_err(), "prefix {n}");
    }
    let mut extra = encoded.clone();
    extra.push(0);
    assert!(State::decode(&extra, &p).is_err());
    for (at, value) in [
        (0, b'X'),
        (60, 33),
        (61, 255),
        (70, 0),
        (71, 255),
        (72, 255),
    ] {
        let mut bad = encoded.clone();
        bad[at] = value;
        assert!(State::decode(&bad, &p).is_err(), "offset {at}");
    }
    for range in [4..12, 12..20, 20..52, 52..60, 62..70] {
        let mut bad = encoded.clone();
        bad[range].fill(0);
        assert!(State::decode(&bad, &p).is_err());
    }
    let mut future = encoded.clone();
    future[74] = 1; // A later-stage objective must remain zero.
    future[62..70].copy_from_slice(&2u64.to_le_bytes());
    future[52..60].copy_from_slice(&2u64.to_le_bytes());
    assert!(State::decode(&future, &p).is_err());
    assert!(State::decode(&vec![0; MAX_STATE_BYTES + 1], &p).is_err());
    let mut changed = DATA.to_vec();
    changed.push(b' ');
    let other = Profile::from_json(
        &changed,
        &theia_network::progression_wire::Profile::from_json(XP).unwrap(),
    )
    .unwrap();
    assert!(State::decode(&encoded, &other).is_err());
    let mut two = all.last().unwrap().encode(&p).unwrap();
    let first_index = usize::from(two[HEADER_BYTES]);
    let first = p.catalog().quests().nth(first_index).unwrap();
    let next = HEADER_BYTES + 11 + first.stages().map(|s| s.objectives().len()).sum::<usize>();
    two[next] = two[HEADER_BYTES];
    assert!(State::decode(&two, &p).is_err());
}
#[test]
fn valid_but_regressed_missing_completed_or_conflicting_states_refuse() {
    let p = profile();
    let all = states(&p);
    assert!(
        all[2]
            .validate_successor(&p, actor(), Some(&all[3]))
            .is_err()
    );
    let mut conflicting = all[2].clone();
    conflicting.revision = all[1].revision;
    assert!(
        conflicting
            .validate_successor(&p, actor(), Some(&all[1]))
            .is_err()
    );
    let mut missing = all[0].clone();
    missing.revision = 100;
    assert!(
        missing
            .validate_successor(&p, actor(), Some(&all[5]))
            .is_err()
    );
    let mut rewind = all[4].clone();
    rewind.revision = 100;
    assert!(
        rewind
            .validate_successor(&p, actor(), Some(&all[5]))
            .is_err()
    );
    let mut wrong = actor();
    wrong.entity = EntityId::new(8).unwrap();
    assert!(all[1].validate_successor(&p, wrong, None).is_err());
}
#[test]
fn maximum_catalogue_objective_state_fits_one_small_reliable_frame() {
    let mut quests = Vec::new();
    for q in 0..32 {
        let stages:Vec<_>=(0..2).map(|s| {
            let objectives:Vec<_>=(0..8).map(|o|json!({"id":format!("o{q:02}{s}{o}"),"label":"Kill","kind":"kill","creature":format!("c{o}"),"required":1})).collect();
            let mut stage=json!({"id":format!("s{q:02}{s}"),"completion":{"kind":"all"},"objectives":objectives});
            if s==0 {stage["next"]=json!(format!("s{q:02}1"));}stage
        }).collect();
        quests.push(json!({"id":format!("q{q:02}"),"version":1,"title":"Quest","description":"Test.","prerequisites":[],"giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":0,"start_stage":format!("s{q:02}0"),"stages":stages}));
    }
    let bytes = serde_json::to_vec(
        &json!({"schema":1,"id":"maximum-fixture","revision":1,"quests":quests}),
    )
    .unwrap();
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let p = Arc::new(Profile::from_json(&bytes, &xp).unwrap());
    let mut entries = Vec::new();
    for (n, definition) in p.catalog().quests().enumerate() {
        let complete = n < 16;
        let counts = definition
            .stages()
            .flat_map(|s| s.objectives())
            .map(|o| (o.id().clone(), u32::from(complete)))
            .collect();
        let stage = definition
            .stages()
            .nth(usize::from(complete))
            .unwrap()
            .id()
            .clone();
        let progress = Progress::from_parts(
            definition,
            if complete { 18 } else { 1 },
            if complete {
                Status::Completed
            } else {
                Status::Active
            },
            stage,
            counts,
        )
        .unwrap();
        entries.push((definition.id().clone(), progress));
    }
    let s = state(
        &p,
        100,
        Journal::from_progress(p.catalog(), entries).unwrap(),
    );
    let frame = s.encode(&p).unwrap();
    assert_eq!(frame.len(), MAX_STATE_BYTES);
    assert_eq!(MAX_STATE_BYTES, 925);
    assert!(frame.len() < theia_network::renet_transport::CHANNEL_MEMORY_BYTES);
    assert_eq!(State::decode(&frame, &p).unwrap(), s);
}

#[test]
fn ready_and_completed_quests_cannot_gain_partial_counts_after_they_freeze() {
    let mut data: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    data["quests"][1]["stages"][0]["completion"] = json!({"kind":"any"});
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let p = Profile::from_json(&serde_json::to_vec(&data).unwrap(), &xp).unwrap();
    let base = states(&p).pop().unwrap();
    let ready = state(
        &p,
        7,
        base.journal
            .propose_trusted_event(p.catalog(), &event("training"))
            .unwrap()
            .journal,
    );
    let quest = id("theia.dev.quest-follow-up");
    let completed = state(
        &p,
        8,
        ready
            .journal
            .propose_turn_in(p.catalog(), &quest, &id("theia.dev.guide"), 2)
            .unwrap()
            .journal,
    );
    for before in [ready, completed] {
        let definition = p.catalog().quest(&quest).unwrap();
        let prior = before.journal.progress(&quest).unwrap();
        let counts = definition
            .stages()
            .flat_map(|s| s.objectives())
            .map(|o| {
                (
                    o.id().clone(),
                    prior.count(o.id()).unwrap()
                        + u32::from(matches!(o.event(), EventKey::Kill(_))),
                )
            })
            .collect();
        let forged = Progress::from_parts(
            definition,
            prior.revision() + 1,
            prior.status(),
            prior.stage().clone(),
            counts,
        )
        .unwrap();
        let entries = before
            .journal
            .quests()
            .map(|(id, q)| {
                (
                    id.clone(),
                    if id == &quest {
                        forged.clone()
                    } else {
                        q.clone()
                    },
                )
            })
            .collect();
        let after = state(
            &p,
            before.revision + 1,
            Journal::from_progress(p.catalog(), entries).unwrap(),
        );
        State::decode(&after.encode(&p).unwrap(), &p).unwrap();
        assert!(
            after
                .validate_successor(&p, actor(), Some(&before))
                .is_err()
        );
    }
}
