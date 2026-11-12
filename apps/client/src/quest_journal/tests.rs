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
