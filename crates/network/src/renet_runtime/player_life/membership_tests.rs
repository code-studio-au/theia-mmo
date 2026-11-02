//! Composite PL02 still uses the existing bounded critical presentation queue.
use super::*;
use std::num::NonZeroU64;
use theia_core::{combat_membership::Summary, player_life::LifeKey};
#[test]
fn full_or_closed_membership_queue_refuses_before_publishing_receive_state() {
    let profile = Arc::new(
        Profile::from_json(
            include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
            include_bytes!("../../../../../content/gameplay/development-creature-patrol.json"),
            include_bytes!("../../../../../content/gameplay/development-creature-retaliation.json"),
            include_bytes!("../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    );
    let membership = Arc::new(
        crate::combat_membership_wire::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-combat-membership.json"
        ))
        .unwrap(),
    );
    let (events, mut receiver) = mpsc::channel(PRESENTATION_CAPACITY);
    let mode = Client {
        profile: profile.clone(),
        membership: Some(membership.clone()),
        events,
    };
    mode.validate().unwrap();
    let welcome = Welcome {
        id: 2,
        epoch: 1,
        world: [9; 32],
        origin: [8000., 0., 8000.],
        terrain_stream: true,
    };
    let actor = EntityKey {
        epoch: WorldEpoch::new(1).unwrap(),
        entity: EntityId::new(2).unwrap(),
    };
    let frame = |tick: u64| {
        let combat = if tick.is_multiple_of(2) {
            Summary {
                opponents: 1,
                outgoing_hostility: true,
                ..Default::default()
            }
        } else {
            Summary::default()
        };
        crate::combat_membership_wire::encode(
            &profile,
            &membership,
            TickRate::Hz30,
            &Observation {
                profile: profile.hash(),
                key: LifeKey {
                    actor,
                    generation: NonZeroU64::MIN,
                },
                tick,
                movement_stream: 1,
                health: 100,
                maximum_health: 100,
                respawn_at: None,
                windup: None,
                combat: Some(combat),
            },
        )
        .unwrap()
    };
    let mut current = None;
    let mut stats = TransportStats::default();
    for tick in 1..=PRESENTATION_CAPACITY as u64 {
        receive(
            &mode,
            &welcome,
            [Some(TickRate::Hz30), None],
            &frame(tick),
            &mut current,
            &mut stats,
        )
        .unwrap();
    }
    let last = current.as_ref().unwrap().current().copied().unwrap();
    assert_eq!(receiver.len(), PRESENTATION_CAPACITY);
    receive(
        &mode,
        &welcome,
        [None, None],
        &frame(PRESENTATION_CAPACITY as u64),
        &mut current,
        &mut stats,
    )
    .unwrap();
    receive(
        &mode,
        &welcome,
        [None, None],
        &frame(1),
        &mut current,
        &mut stats,
    )
    .unwrap();
    assert_eq!(stats.life_duplicate_states, 1);
    assert_eq!(stats.life_stale_states, 1);
    let failed = receive(
        &mode,
        &welcome,
        [None, None],
        &frame(17),
        &mut current,
        &mut stats,
    )
    .unwrap_err();
    assert!(failed.to_string().contains("queue full or closed"));
    assert_eq!(current.as_ref().unwrap().current(), Some(&last));
    assert_eq!(stats.life_states_received, PRESENTATION_CAPACITY as u64);
    assert_eq!(receiver.try_recv().unwrap().tick, 1);
    drop(receiver);
    assert!(
        receive(
            &mode,
            &welcome,
            [None, None],
            &frame(17),
            &mut current,
            &mut stats
        )
        .is_err()
    );
    assert_eq!(current.as_ref().unwrap().current(), Some(&last));
}
