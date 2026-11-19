//! Headless authority fixtures, not durable credit or authenticated clients.
use super::*;
use serde_json::{Value, json};
use theia_core::{content::ContentId, coordinates::GlobalPosition, quests::journal::Journal};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn data() -> (Vec<u8>, Vec<u8>) {
    let mut q: Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/quests/development-journal.json"
    ))
    .unwrap();
    q["schema"] = json!(2);
    q["quests"].as_array_mut().unwrap().truncate(1);
    q["quests"][0]["stages"].as_array_mut().unwrap().truncate(1);
    let stage = &mut q["quests"][0]["stages"][0];
    stage["next"] = Value::Null;
    stage["objectives"] = json!([{"kind":"reach_area","id":"theia.dev.goal.overlook","label":"Reach",
        "area":"theia.dev.area.overlook"}]);
    let areas = json!({"schema":1,"id":"theia.dev.areas","revision":1,"areas":[{
        "id":"theia.dev.area.overlook","name":"Overlook","min":[7990.,-64.,7990.],"max":[8010.,64.,8010.]}]});
    (
        serde_json::to_vec(&q).unwrap(),
        serde_json::to_vec(&areas).unwrap(),
    )
}
fn xp() -> Arc<crate::progression_wire::Profile> {
    Arc::new(
        crate::progression_wire::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap(),
    )
}
fn profile(q: &[u8], a: &[u8]) -> Arc<crate::quest_journal::Profile> {
    Arc::new(crate::quest_journal::Profile::from_json_with_areas(q, a, &xp()).unwrap())
}
fn journal(p: &crate::quest_journal::Profile) -> Journal {
    Journal::default()
        .propose_accept(
            p.catalog(),
            &id("theia.dev.quest-introduction"),
            &id("theia.dev.guide"),
        )
        .unwrap()
        .journal
}
fn fixture(hz: u32) -> World {
    let mut w = super::tests::setup_at(hz);
    w.enable_prediction().unwrap();
    w
}
#[test]
fn exact_area_bytes_bind_profile_and_world_without_changing_legacy_hash() {
    let (q, a) = data();
    let p = profile(&q, &a);
    let mut changed = a.clone();
    changed.push(b' ');
    let other = profile(&q, &changed);
    assert_ne!(p.hash(), other.hash());
    let old = crate::quest_journal::Profile::from_json(&q, &xp()).unwrap();
    let mut bound = theia_core::quests::journal::PROFILE_DOMAIN.to_vec();
    bound.extend(&q);
    bound.extend(xp().hash());
    assert_eq!(old.hash(), crate::terrain::sha256(&bound));
    let mut w = fixture(30);
    let base = w.identity();
    assert!(w.enable_quest_journal(Arc::new(old), xp()).is_err());
    assert_eq!(base, w.identity());
    w.enable_quest_journal(p, xp()).unwrap();
    let identity = w.identity();
    assert_ne!(base, identity);
    let mut second = fixture(30);
    second.enable_quest_journal(other, xp()).unwrap();
    assert_ne!(identity, second.identity());
}
#[test]
fn unresolved_aliases_and_outside_bounds_refuse_atomically() {
    let (q, a) = data();
    let p = profile(&q, &a);
    let mut missing: Value = serde_json::from_slice(&a).unwrap();
    missing["areas"][0]["id"] = json!("theia.dev.other");
    assert!(
        crate::quest_journal::Profile::from_json_with_areas(
            &q,
            &serde_json::to_vec(&missing).unwrap(),
            &xp()
        )
        .is_err()
    );
    for alias in [
        "theia.dev.guide",
        "theia.dev.quest-introduction",
        "theia.dev.goal.overlook",
    ] {
        let mut areas: Value = serde_json::from_slice(&a).unwrap();
        areas["id"] = json!(alias);
        assert!(
            crate::quest_journal::Profile::from_json_with_areas(
                &q,
                &serde_json::to_vec(&areas).unwrap(),
                &xp()
            )
            .is_err()
        );
    }
    let mut outside: Value = serde_json::from_slice(&a).unwrap();
    outside["areas"][0]["min"][0] = json!(-1.);
    outside["areas"][0]["max"][0] = json!(1.);
    let mut w = fixture(30);
    let before = w.identity();
    assert!(
        w.enable_quest_journal(profile(&q, &serde_json::to_vec(&outside).unwrap()), xp())
            .is_err()
    );
    assert_eq!(before, w.identity());
    assert!(w.quest_journal_profile().is_none());
    w.enable_quest_journal(p.clone(), xp()).unwrap();
    let before = w.identity();
    assert!(w.enable_quest_journal(p, xp()).is_err());
    assert_eq!(before, w.identity());
}
#[test]
fn server_feet_samples_at_all_rates_reject_height_stale_actor_world_and_missing_collision() {
    for hz in [20, 30, 60] {
        let (q, a) = data();
        let p = profile(&q, &a);
        let j = journal(&p);
        let mut w = fixture(hz);
        let prepared = w.prepare_quest_journal_profile(&p).unwrap();
        assert!(w.prepare_quest_journal(p.shared_catalog()).is_err());
        let actor = w.join().unwrap();
        let key = w.entity_key(actor).unwrap();
        // A newly admitted peer has not reported its terrain footprint yet.
        assert!(w.observe_quest_areas(&prepared, key, &j).is_err());
        let ready = w
            .procedural
            .as_ref()
            .unwrap()
            .patches
            .values()
            .flat_map(|p| p.ids.iter().copied())
            .collect();
        w.players.get_mut(&actor).unwrap().ready = Some(ready);
        assert_eq!(w.observe_quest_areas(&prepared, key, &j).unwrap().len(), 1);
        let centre = w.players[&actor].position;
        // Exact upper face admits feet, even though capsule centre is above it.
        for (feet_y, expected) in [(64., 1), (64.001, 0), (-64., 1), (-64.001, 0)] {
            let mut pos = centre.metres();
            pos[1] = feet_y + f64::from(crate::world::PAWN_Y);
            w.players.get_mut(&actor).unwrap().position = GlobalPosition::new(pos).unwrap();
            assert_eq!(
                w.observe_quest_areas(&prepared, key, &j).unwrap().len(),
                expected
            );
        }
        w.players.get_mut(&actor).unwrap().position = centre;
        // Movement and rebasing never trust a render-space position.
        let mut pos = centre.metres();
        pos[0] = 8010.001;
        w.players.get_mut(&actor).unwrap().position = GlobalPosition::new(pos).unwrap();
        assert!(
            w.observe_quest_areas(&prepared, key, &j)
                .unwrap()
                .is_empty()
        );
        w.players.get_mut(&actor).unwrap().position = centre;
        let patches = std::mem::take(&mut w.procedural.as_mut().unwrap().patches);
        assert!(w.observe_quest_areas(&prepared, key, &j).is_err());
        w.procedural.as_mut().unwrap().patches = patches;
        w.players.get_mut(&actor).unwrap().ready = Some(Default::default());
        assert!(w.observe_quest_areas(&prepared, key, &j).is_err());
        w.players.get_mut(&actor).unwrap().ready = None;
        let other = fixture(hz);
        assert!(other.observe_quest_areas(&prepared, key, &j).is_err());
        w.leave(actor);
        assert!(w.observe_quest_areas(&prepared, key, &j).is_err());
        let new = w.join().unwrap();
        assert_ne!(w.entity_key(new), Some(key));
        assert!(w.observe_quest_areas(&prepared, key, &j).is_err());
    }
}

#[test]
fn area_definitions_cannot_alias_unreferenced_installed_world_content() {
    for alias in ["theia.dev.training-creature", "theia.dev.interaction.road"] {
        let (q, a) = data();
        let mut q: Value = serde_json::from_slice(&q).unwrap();
        let mut a: Value = serde_json::from_slice(&a).unwrap();
        q["quests"][0]["stages"][0]["objectives"][0]["area"] = json!(alias);
        a["areas"][0]["id"] = json!(alias);
        let p = profile(
            &serde_json::to_vec(&q).unwrap(),
            &serde_json::to_vec(&a).unwrap(),
        );
        let mut w = fixture(30);
        let old = w.identity();
        assert!(w.enable_quest_journal(p, xp()).is_err());
        assert_eq!(old, w.identity());
    }
}
#[test]
fn router_reserves_capacity_retries_and_does_not_publish_or_repeat_uncommitted_observations() {
    use crate::renet_runtime::progression::{
        CAPACITY, Input, JournalInput, JournalObservation, JournalOperation, Observation, Port,
    };
    let (q, a) = data();
    let p = profile(&q, &a);
    let j = journal(&p);
    let mut w = fixture(30);
    let prepared = p.prepare(&w).unwrap();
    let actor = w.join().unwrap();
    let key = w.entity_key(actor).unwrap();
    let ready = w
        .procedural
        .as_ref()
        .unwrap()
        .patches
        .values()
        .flat_map(|p| p.ids.iter().copied())
        .collect();
    w.players.get_mut(&actor).unwrap().ready = Some(ready);
    let mut router = prepared.bind(key);
    let (input, mut inputs) = tokio::sync::mpsc::channel(CAPACITY);
    let (publish, state) = tokio::sync::watch::channel(Observation {
        journal: Some(JournalObservation {
            catalog: p.hash(),
            revision: 1,
            journal: j.clone(),
            reply: None,
        }),
        ..Default::default()
    });
    let port = Port {
        journal: Some(p.hash()),
        lifecycle: None,
        quest: None,
        profile: xp(),
        input,
        state,
    };
    // Fill without consuming a producer sequence or performing any owner work.
    let filler = Input::Journal(JournalInput {
        actor: key,
        catalog: p.hash(),
        sequence: 999,
        operation: JournalOperation::Event {
            id: NonZeroU128::new(1).unwrap(),
            event: EventKey::Kill(id("unused")),
        },
    });
    for _ in 0..CAPACITY {
        port.input.try_send(filler.clone()).unwrap();
    }
    assert!(!router.observe_areas(&w, &port).unwrap());
    for _ in 0..CAPACITY {
        inputs.try_recv().unwrap();
    }
    assert!(router.observe_areas(&w, &port).unwrap());
    let Input::Journal(input) = inputs.try_recv().unwrap() else {
        panic!("Wrong input")
    };
    assert_eq!(input.sequence, 1);
    let JournalOperation::Areas { observation, .. } = input.operation else {
        panic!("Wrong operation")
    };
    assert_eq!(observation.len(), 1);
    for _ in 0..100 {
        assert!(!router.observe_areas(&w, &port).unwrap());
    }
    assert!(inputs.try_recv().is_err());
    assert_eq!(port.state.borrow().journal.as_ref().unwrap().journal, j);
    let committed = j
        .propose_reached_areas(p.catalog(), &observation)
        .unwrap()
        .journal;
    publish.send_modify(|s| {
        s.journal.as_mut().unwrap().journal = committed;
    });
    assert!(!router.observe_areas(&w, &port).unwrap());
    publish.send_modify(|s| {
        s.failed = true;
    });
    assert!(router.observe_areas(&w, &port).is_err());
}
