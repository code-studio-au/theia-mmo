use super::*;
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::{catalog::EventKey, journal::Journal},
};
use theia_network::wire;
fn id(value: &str) -> ContentId {
    ContentId::new(value).unwrap()
}
pub(crate) fn fixture() -> (State, Arc<Profile>) {
    let xp = theia_network::progression_wire::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    let profile = Arc::new(
        Profile::from_json(
            include_bytes!("../../../../content/quests/development-journal.json"),
            &xp,
        )
        .unwrap(),
    );
    let actor = EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(3).unwrap(),
    };
    let guide = theia_network::dialogue::Guide {
        key: EntityKey {
            entity: EntityId::new(2).unwrap(),
            ..actor
        },
        position: GlobalPosition::new([2., 0.9, 0.]).unwrap(),
    };
    (
        State {
            id: Some(3),
            welcome: Some(wire::Welcome {
                terrain_stream: true,
                id: 3,
                epoch: 7,
                world: [1; 32],
                origin: [0.; 3],
            }),
            snapshot: Some(wire::Snapshot {
                version: wire::VERSION,
                epoch: 7,
                tick: 1,
                tick_hz: 30,
                origin: [0.; 3],
                players: vec![wire::Player {
                    facing: None,
                    id: 3,
                    xz: [0., 0.],
                    y: 0.9,
                    applied_sequence: 0,
                }],
            }),
            quest_journal: Some(theia_network::quest_journal_wire::State {
                actor,
                profile_hash: profile.hash(),
                revision: 0,
                journal: Journal::default(),
            }),
            guide: Some(theia_network::dialogue_wire::Presence {
                actor,
                profile_hash: [1; 32],
                contacts: vec![],
                guide: Some(guide),
            }),
            dialogue: Some(theia_network::dialogue::View {
                actor,
                profile_hash: [1; 32],
                revision: 1,
                open: Some(theia_network::dialogue::Open {
                    guide: guide.key,
                    node: id("theia.dev.quest-guide.welcome"),
                }),
            }),
            chunk_report: Some(wire::ChunkReady {
                version: wire::VERSION,
                epoch: 7,
                sequence: 1,
                chunks: vec![[0, 0]],
            }),
            ..default()
        },
        profile,
    )
}
pub(crate) fn set_journal(state: &mut State, journal: Journal) {
    let view = state.quest_journal.as_mut().unwrap();
    view.revision = journal.quests().map(|(_, q)| q.revision()).sum();
    view.journal = journal;
}
pub(crate) fn completed_introduction(profile: &Profile) -> Journal {
    let quest = id("theia.dev.quest-introduction");
    let npc = id("theia.dev.guide");
    let mut journal = Journal::default()
        .propose_accept(profile.catalog(), &quest, &npc)
        .unwrap()
        .journal;
    for interaction in ["road", "supplies", "report"] {
        journal = journal
            .propose_trusted_event(
                profile.catalog(),
                &EventKey::NpcInteraction {
                    npc: npc.clone(),
                    interaction: id(&format!("theia.dev.interaction.{interaction}")),
                },
            )
            .unwrap()
            .journal;
    }
    journal
        .propose_turn_in(profile.catalog(), &quest, &npc, 4)
        .unwrap()
        .journal
}
#[test]
fn actions_require_committed_eligibility_and_the_current_open_nearby_guide() {
    let (mut state, profile) = fixture();
    let first = profile
        .catalog()
        .quest(&id("theia.dev.quest-introduction"))
        .unwrap();
    let second = profile
        .catalog()
        .quest(&id("theia.dev.quest-follow-up"))
        .unwrap();
    let available = action(&profile, &state, first, 1).unwrap();
    assert_eq!(available.command.action, Action::Accept);
    assert!(action(&profile, &state, second, 1).is_none());
    assert_ne!(
        available.command.request_id,
        action(&profile, &state, first, 2)
            .unwrap()
            .command
            .request_id
    );
    state.quest_journal_pending = true;
    assert!(action(&profile, &state, first, 1).is_none());
    state.quest_journal_pending = false;
    state.dialogue.as_mut().unwrap().open = None;
    assert!(action(&profile, &state, first, 1).is_none());
    let (mut state, _) = fixture();
    state
        .guide
        .as_mut()
        .unwrap()
        .guide
        .as_mut()
        .unwrap()
        .position = GlobalPosition::new([3.0001, 0.9, 0.]).unwrap();
    assert!(action(&profile, &state, first, 1).is_none());
    let (mut state, _) = fixture();
    state.quest_journal.as_mut().unwrap().profile_hash = [9; 32];
    assert!(action(&profile, &state, first, 1).is_none());
    let (mut state, _) = fixture();
    state.error = Some("fixture disconnect".into());
    assert!(action(&profile, &state, first, 1).is_none());
}
#[test]
fn completed_prerequisite_unlocks_the_next_offer_and_tracker_uses_only_committed_progress() {
    let (mut state, profile) = fixture();
    let first = profile
        .catalog()
        .quest(&id("theia.dev.quest-introduction"))
        .unwrap();
    let second = profile
        .catalog()
        .quest(&id("theia.dev.quest-follow-up"))
        .unwrap();
    assert!(journal_text(&profile, &state).contains("No accepted quests"));
    let accepted = state
        .quest_journal
        .as_ref()
        .unwrap()
        .journal
        .propose_accept(profile.catalog(), first.id(), first.giver())
        .unwrap()
        .journal;
    set_journal(&mut state, accepted);
    let text = journal_text(&profile, &state);
    assert!(text.contains("In progress"));
    assert!(text.contains("0/1"));
    assert!(action(&profile, &state, first, 1).is_none());
    assert!(action(&profile, &state, second, 1).is_none());
    set_journal(&mut state, completed_introduction(&profile));
    assert!(journal_text(&profile, &state).contains("Completed"));
    assert!(action(&profile, &state, first, 2).is_none());
    assert_eq!(
        action(&profile, &state, second, 2).unwrap().command.action,
        Action::Accept
    );
}
#[test]
fn pointer_activation_retains_painted_quest_revision_and_original_identity() {
    let (state, profile) = fixture();
    let quest = id("theia.dev.quest-introduction");
    let painted = action(
        &profile,
        &state,
        profile.catalog().quest(&quest).unwrap(),
        1,
    )
    .unwrap();
    let mut app = App::new();
    app.init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate);
    app.world_mut()
        .resource_mut::<Ui>()
        .intents
        .insert(quest.clone(), painted.clone());
    let button = app.world_mut().spawn(ActionButton(quest.clone())).id();
    app.world_mut().trigger(Activate { entity: button });
    let mut updated = painted.clone();
    updated.command.expected_revision = 9;
    app.world_mut()
        .resource_mut::<Ui>()
        .intents
        .insert(quest, updated);
    app.world_mut().trigger(Activate { entity: button });
    let pending = app.world().resource::<Pending>().0.as_ref().unwrap();
    assert_eq!(pending.command.expected_revision, 0);
    assert_eq!(pending.command.request_id, painted.command.request_id);
    assert_eq!(app.world().resource::<Ui>().pointer_actions, 1);
    assert_eq!(app.world().resource::<Ui>().ignored_pending_clicks, 1);
}

#[test]
fn fixed_items_are_named_on_offers_and_unfinished_journal_but_not_promised_again() {
    let (mut state, legacy) = fixture();
    let inventory = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-document-items.json"
    ))
    .unwrap();
    let rewards = serde_json::to_vec(&serde_json::json!({
        "schema":1,"id":"theia.test.ui-rewards","revision":1,
        "quest_catalog":legacy.catalog().id().as_str(),
        "item_catalog":inventory.catalog().id().as_str(),
        "rewards":[{"quest":"theia.dev.quest-introduction","grants":[
            {"item":"theia.dev.training-token","count":3},
            {"item":"theia.dev.training-sword","count":1}
        ]}]
    }))
    .unwrap();
    let profile = legacy
        .as_ref()
        .clone()
        .with_item_rewards(&rewards, &inventory)
        .unwrap();
    state.quest_journal.as_mut().unwrap().profile_hash = profile.hash();
    let quest = profile
        .catalog()
        .quest(&id("theia.dev.quest-introduction"))
        .unwrap();
    for verb in [Action::Accept, Action::TurnIn] {
        let label = action_text(&profile, quest, verb);
        assert!(label.contains("(25 XP)"));
        assert!(label.contains("3 x Training Token"));
        assert!(label.contains("1 x Training Sword"));
        assert!(!label.contains("theia.dev.training"));
    }
    // Actual offer construction uses only the bound fixed-reward metadata.
    // The unskinned grid and quests without grants acquire no extra artwork.
    for skinned in [false, true] {
        let mut app = App::new();
        app.insert_resource(Definitions(Arc::new(profile.clone())))
            .add_systems(Startup, setup);
        if skinned {
            app.insert_resource(crate::fantasy_ui::Skin::fixture());
        }
        app.world_mut().spawn(crate::dialogue::Pane);
        app.update();
        let world = app.world_mut();
        let mut icons = world.query::<(&crate::fantasy_ui::RewardIcon, &Pickable)>();
        let rendered: BTreeMap<_, _> = icons
            .iter(world)
            .map(|(icon, pickable)| {
                assert_eq!(*pickable, Pickable::IGNORE);
                (icon.item.as_str().to_owned(), icon.count)
            })
            .collect();
        let expected = if skinned {
            BTreeMap::from([
                ("theia.dev.training-token".to_owned(), 3),
                ("theia.dev.training-sword".to_owned(), 1),
            ])
        } else {
            BTreeMap::new()
        };
        assert_eq!(rendered, expected);
        assert!(
            !world.contains_resource::<Pending>(),
            "Decoration installs no quest command producer"
        );
    }
    let journal = Journal::default()
        .propose_accept(profile.catalog(), quest.id(), quest.giver())
        .unwrap()
        .journal;
    set_journal(&mut state, journal);
    let tracker = journal_text(&profile, &state);
    assert!(tracker.contains("Completion reward: 25 XP"));
    assert!(tracker.contains("3 x Training Token") && tracker.contains("1 x Training Sword"));
    set_journal(&mut state, completed_introduction(&profile));
    let tracker = journal_text(&profile, &state);
    assert!(tracker.contains("Completed"));
    assert!(!tracker.contains("Completion reward") && !tracker.contains("Items:"));
    let other = profile
        .catalog()
        .quest(&id("theia.dev.quest-follow-up"))
        .unwrap();
    assert_eq!(
        action_text(&profile, other, Action::Accept),
        action_text(&legacy, other, Action::Accept)
    );
    let label = action_text(&legacy, quest, Action::Accept);
    assert!(label.starts_with("Accept: Meet the guide (25 XP)"));
    assert!(label.contains(quest.description()));
}

#[test]
fn backpack_refusal_explains_retry_without_confusing_journal_capacity_or_pending() {
    use theia_network::quest_journal_wire::{Rejection, Reply};
    let (mut state, profile) = fixture();
    state.quest_journal_reply = Some(Reply {
        actor: state.quest_journal.as_ref().unwrap().actor,
        profile_hash: profile.hash(),
        sequence: 1,
        request_id: NonZeroU128::new(1).unwrap(),
        revision: 0,
        quest: id("theia.dev.quest-introduction"),
        quest_revision: 0,
        rejection: Some(Rejection::InventoryFull),
    });
    assert!(status_text(&state, false).contains("Free some space"));
    assert!(status_text(&state, false).contains("try completing the quest again"));
    assert_eq!(status_text(&state, true), "Waiting for confirmation...");
    state.error = Some("disconnected".into());
    assert_eq!(
        status_text(&state, true),
        "Connection lost. Rejoin to refresh your journal."
    );
    state.error = None;
    state.quest_journal_reply.as_mut().unwrap().rejection = Some(Rejection::Capacity);
    assert!(!status_text(&state, false).contains("backpack"));
    state.quest_journal_reply.as_mut().unwrap().rejection = None;
    assert_eq!(status_text(&state, false), "");
}

#[test]
fn compact_tracker_hides_history_and_reward_details_but_journal_retains_them() {
    let (mut state, profile) = fixture();
    set_journal(&mut state, completed_introduction(&profile));
    assert!(!journal_content(&profile, &state, false).contains("Completed"));
    assert!(journal_text(&profile, &state).contains("Completed"));
    let quest = profile
        .catalog()
        .quest(&id("theia.dev.quest-introduction"))
        .unwrap();
    set_journal(
        &mut state,
        Journal::default()
            .propose_accept(profile.catalog(), quest.id(), quest.giver())
            .unwrap()
            .journal,
    );
    let compact = journal_content(&profile, &state, false);
    assert!(compact.contains("0/1"));
    assert!(!compact.contains("Completion reward"));
    assert!(journal_text(&profile, &state).contains(quest.description()));
}

#[test]
fn overlapping_contacts_offer_only_their_own_acceptance_and_turn_in() {
    let (mut state, old) = fixture();
    let xp = theia_network::progression_wire::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../content/quests/development-journal.json"
    ))
    .unwrap();
    data["quests"][0]["turn_in"] = serde_json::json!("theia.dev.steward");
    let profile = Profile::from_json(&serde_json::to_vec(&data).unwrap(), &xp).unwrap();
    state.quest_journal.as_mut().unwrap().profile_hash = profile.hash();
    let first = id("theia.dev.quest-introduction");
    let quest = profile.catalog().quest(&first).unwrap();
    let presence = state.guide.as_mut().unwrap();
    let a = presence.guide.take().unwrap();
    let mut b = a;
    b.key.entity = EntityId::new(4).unwrap();
    presence.contacts = vec![
        theia_network::dialogue::ContactPresence {
            npc: id("theia.dev.guide"),
            guide: a,
        },
        theia_network::dialogue::ContactPresence {
            npc: id("theia.dev.steward"),
            guide: b,
        },
    ];
    assert!(action(&profile, &state, quest, 1).is_some());
    state
        .dialogue
        .as_mut()
        .unwrap()
        .open
        .as_mut()
        .unwrap()
        .guide = b.key;
    assert!(action(&profile, &state, quest, 1).is_none());
    let mut journal = Journal::default()
        .propose_accept(profile.catalog(), &first, &id("theia.dev.guide"))
        .unwrap()
        .journal;
    for name in ["road", "supplies", "report"] {
        journal = journal
            .propose_trusted_event(
                profile.catalog(),
                &EventKey::NpcInteraction {
                    npc: id("theia.dev.guide"),
                    interaction: id(&format!("theia.dev.interaction.{name}")),
                },
            )
            .unwrap()
            .journal;
    }
    set_journal(&mut state, journal);
    assert_eq!(
        action(&profile, &state, quest, 2).unwrap().command.action,
        Action::TurnIn
    );
    state
        .dialogue
        .as_mut()
        .unwrap()
        .open
        .as_mut()
        .unwrap()
        .guide = a.key;
    assert!(action(&profile, &state, quest, 2).is_none());
    assert_ne!(profile.hash(), old.hash());
}
