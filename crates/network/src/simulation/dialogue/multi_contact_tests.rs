use super::*;
use theia_core::time::TickRate;
const CONTACTS: &[u8] =
    include_bytes!("../../../../../content/quests/development-valley-contacts.json");
fn id(value: &str) -> ContentId {
    ContentId::new(value).unwrap()
}
fn profile() -> Arc<Profile> {
    Arc::new(Profile::from_catalog_json(CONTACTS).unwrap())
}
fn placements() -> Vec<(ContentId, Placement)> {
    vec![
        (id("theia.dev.guide"), Placement::Horizontal([8062., 8000.])),
        (
            id("theia.dev.steward"),
            Placement::Horizontal([8063., 8000.]),
        ),
    ]
}
fn request(sequence: u64, revision: u64, action: Action) -> Request {
    Request {
        sequence,
        expected_revision: revision,
        action,
    }
}
#[test]
fn contacts_are_atomic_sorted_supported_and_bound_to_world_identity() {
    let mut a = World::with_procedural([8060., 8000.], 1.).unwrap();
    let mut b = World::with_procedural([8060., 8000.], 1.).unwrap();
    let base = a.identity();
    let next = a.next_id;
    let mut missing = placements();
    missing.pop();
    assert!(a.spawn_placed_contacts(profile(), missing).is_err());
    let mut duplicate = placements();
    duplicate[1].0 = duplicate[0].0.clone();
    assert!(a.spawn_placed_contacts(profile(), duplicate).is_err());
    let mut unsafe_position = placements();
    unsafe_position[1].1 = Placement::Feet(GlobalPosition::new([8063., 1000., 8000.]).unwrap());
    assert!(a.spawn_placed_contacts(profile(), unsafe_position).is_err());
    assert_eq!(a.identity(), base);
    assert_eq!(a.next_id, next);
    assert!(a.dialogue.is_none());
    assert!(
        a.spawn_placed_guide(profile(), Placement::Horizontal([8062., 8000.]))
            .is_err()
    );
    let contacts = a.spawn_placed_contacts(profile(), placements()).unwrap();
    let mut reversed = placements();
    reversed.reverse();
    let other = b.spawn_placed_contacts(profile(), reversed).unwrap();
    assert_eq!(
        contacts
            .iter()
            .map(|c| (&c.npc, c.guide.key.entity, c.guide.position))
            .collect::<Vec<_>>(),
        other
            .iter()
            .map(|c| (&c.npc, c.guide.key.entity, c.guide.position))
            .collect::<Vec<_>>()
    );
    assert_eq!(a.identity(), b.identity());
    assert_ne!(base, a.identity());
    assert_ne!(contacts[0].guide.key, contacts[1].guide.key);
    let identity = a.identity();
    a.retire_development_guide(contacts[0].guide.key);
    assert_eq!(a.identity(), identity);
}
#[test]
fn exact_contact_controls_choices_private_sessions_retirement_range_and_vendor_all_rates() {
    for hz in [20, 30, 60] {
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        world.rate = TickRate::from_hz(hz).unwrap();
        let contacts = world
            .spawn_placed_contacts(profile(), placements())
            .unwrap();
        let [a, b] = [contacts[0].guide, contacts[1].guide];
        let items = crate::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let vendor = Arc::new(
            crate::vendor_wire::Profile::from_json(
                include_bytes!("../../../../../content/gameplay/development-vendor.json"),
                &items,
            )
            .unwrap(),
        );
        world.enable_vendor(vendor).unwrap();
        let first = world.join().unwrap();
        let second = world.join().unwrap();
        world.players.get_mut(&first).unwrap().position = a.position;
        world.players.get_mut(&second).unwrap().position = b.position;
        world
            .interact(first, &request(1, 0, Action::Open(a.key)))
            .unwrap();
        world
            .interact(second, &request(1, 0, Action::Open(b.key)))
            .unwrap();
        assert_eq!(
            world.dialogue_view(first).unwrap().open.unwrap().node,
            id("theia.dev.quest-guide.welcome")
        );
        assert_eq!(
            world.dialogue_view(second).unwrap().open.unwrap().node,
            id("theia.dev.steward.welcome")
        );
        assert!(!world.guide_interaction_eligible(first, 1, b.key));
        let buy = |npc| theia_core::vendors::Command {
            expected_revision: 0,
            dialogue_revision: 1,
            npc,
            offer: 0,
        };
        assert!(world.vendor_eligible(first, buy(a.key)));
        assert!(!world.vendor_eligible(second, buy(b.key)));
        assert_eq!(
            world.interact(
                first,
                &request(2, 1, Action::Choose(id("theia.dev.steward.report")))
            ),
            Err(Rejection::InvalidAction)
        );
        assert_eq!(
            world.interact(
                second,
                &request(2, 0, Action::Choose(id("theia.dev.steward.report")))
            ),
            Err(Rejection::StaleRevision)
        );
        world
            .interact(
                second,
                &request(3, 1, Action::Choose(id("theia.dev.steward.report"))),
            )
            .unwrap();
        assert_eq!(
            world.dialogue_view(second).unwrap().open.unwrap().guide,
            b.key
        );
        assert!(world.retire_development_guide(a.key));
        assert!(world.dialogue_view(first).unwrap().open.is_none());
        assert!(world.dialogue_view(second).unwrap().open.is_some());
        world.players.get_mut(&second).unwrap().position =
            GlobalPosition::new([8063., 100., 8000.]).unwrap();
        world.advance_dialogue();
        assert!(world.dialogue_view(second).unwrap().open.is_none());
        assert_eq!(world.visible_contacts(first).len(), 1);
    }
}
#[test]
fn semantic_occurrences_name_the_session_contact_and_reject_cross_contact_choices() {
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    let contacts = world
        .spawn_placed_contacts(profile(), placements())
        .unwrap();
    let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/quests/development-valley-contact-quests.json"
    ))
    .unwrap();
    data["schema"] = serde_json::json!(1);
    data["quests"] =
        serde_json::json!([data["quests"].as_array().unwrap().last().unwrap().clone()]);
    let catalog = Arc::new(
        theia_core::quests::catalog::Catalog::from_json(&serde_json::to_vec(&data).unwrap())
            .unwrap(),
    );
    let prepared = world.prepare_quest_journal(catalog).unwrap();
    let actor = world.join().unwrap();
    let b = contacts[1].guide;
    world.players.get_mut(&actor).unwrap().position = b.position;
    world
        .interact_quest_journal(&prepared, actor, &request(1, 0, Action::Open(b.key)))
        .unwrap();
    let fact = world
        .interact_quest_journal(
            &prepared,
            actor,
            &request(2, 1, Action::Choose(id("theia.dev.steward.report"))),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        fact.event,
        theia_core::quests::catalog::EventKey::NpcInteraction {
            npc: id("theia.dev.steward"),
            interaction: id("theia.dev.steward.report")
        }
    );
    assert!(
        world
            .interact_quest_journal(
                &prepared,
                actor,
                &request(2, 1, Action::Choose(id("theia.dev.steward.report")))
            )
            .is_err()
    );
}

#[test]
fn forged_quest_npc_cannot_borrow_another_open_contacts_entity_or_consume_the_sequence() {
    use crate::{
        quest_journal_wire::{Rejection as Refusal, Request as QuestRequest},
        renet_runtime::progression::{self, Input, JournalObservation, Observation, Port},
    };
    use theia_core::quests::journal::{Action as QuestAction, Command, Journal};
    for hz in [20, 30, 60] {
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        world.rate = TickRate::from_hz(hz).unwrap();
        let contacts = world
            .spawn_placed_contacts(profile(), placements())
            .unwrap();
        let [a, b] = [contacts[0].guide, contacts[1].guide];
        let xp = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/quests/development-valley-contact-quests.json"
        ))
        .unwrap();
        raw["schema"] = serde_json::json!(1);
        raw["quests"] =
            serde_json::json!([raw["quests"].as_array().unwrap().last().unwrap().clone()]);
        let journal = Arc::new(
            crate::quest_journal::Profile::from_json(&serde_json::to_vec(&raw).unwrap(), &xp)
                .unwrap(),
        );
        let prepared = journal.prepare(&world).unwrap();
        let actor = world.join().unwrap();
        world.players.get_mut(&actor).unwrap().position = b.position;
        let actor = world.entity_key(actor).unwrap();
        let mut router = prepared.bind(actor);
        let (input, mut receiver) = tokio::sync::mpsc::channel(progression::CAPACITY);
        let (_publish, state) = tokio::sync::watch::channel(Observation {
            journal: Some(JournalObservation {
                catalog: journal.hash(),
                revision: 0,
                journal: Journal::default(),
                reply: None,
            }),
            ..Observation::default()
        });
        let port = Port {
            journal: Some(journal.hash()),
            profile: xp,
            input,
            state,
            quest: None,
            lifecycle: None,
        };
        world
            .interact(actor.entity.value(), &request(1, 0, Action::Open(b.key)))
            .unwrap();
        let mut command = QuestRequest {
            actor,
            profile_hash: journal.hash(),
            sequence: 1,
            npc: b.key,
            dialogue_revision: 1,
            command: Command {
                request_id: std::num::NonZeroU128::new(1).unwrap(),
                quest: id("theia.dev.quest-steward-delivery"),
                npc: id("theia.dev.guide"),
                expected_revision: 0,
                action: QuestAction::Accept,
            },
        };
        assert!(
            command.encode(&journal).is_ok(),
            "Well-formed wire claim still needs live NPC authorization"
        );
        assert_eq!(
            router.command(&world, &command, &port),
            Err(Refusal::NotEntitled)
        );
        assert!(receiver.try_recv().is_err());
        command.npc = a.key;
        assert_eq!(
            router.command(&world, &command, &port),
            Err(Refusal::NotEntitled)
        );
        assert!(receiver.try_recv().is_err());
        world
            .interact(actor.entity.value(), &request(2, 1, Action::Close))
            .unwrap();
        world
            .interact(actor.entity.value(), &request(3, 2, Action::Open(a.key)))
            .unwrap();
        command.dialogue_revision = 3;
        router.command(&world, &command, &port).unwrap();
        let Input::Journal(queued) = receiver.try_recv().unwrap() else {
            panic!("Expected durable owner command")
        };
        assert_eq!(queued.sequence, 1);
        assert!(receiver.try_recv().is_err());
    }
}
