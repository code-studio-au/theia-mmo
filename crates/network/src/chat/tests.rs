use super::*;
use theia_core::{
    character_name::CharacterName,
    chat::{Area, Channel, Request as Intent, Text},
    identity::EntityId,
};
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn speaker(id: u64, x: f64, area: u32) -> Speaker {
    Speaker {
        key: key(id),
        name: CharacterName::parse(if id == 1 { "Arden" } else { "Briar" }).unwrap(),
        position: GlobalPosition::new([x, 0., 0.]).unwrap(),
        area: Area::new(area),
    }
}
fn request(sequence: u64, channel: Channel) -> Request {
    Request {
        epoch: key(1).epoch,
        intent: Intent {
            sequence,
            channel,
            text: Text::parse("Hello travellers").unwrap(),
        },
    }
}
fn router() -> Router {
    Router::new([1; 32], key(1).epoch, TickRate::Hz30).unwrap()
}
fn drain(router: &mut Router) -> Vec<(EntityKey, Delivery)> {
    let mut messages = Vec::new();
    router
        .flush(|key, bytes| {
            messages.push((key, Delivery::decode(bytes).unwrap()));
            true
        })
        .unwrap();
    messages
}
#[test]
fn routes_only_to_admitted_nearby_or_matching_area_peers_with_server_selected_name() {
    let mut router = router();
    router.admit(speaker(1, 0., 1)).unwrap();
    router.admit(speaker(2, 25., 2)).unwrap();
    router.admit(speaker(3, 100., 1)).unwrap();
    router.submit(key(1), request(1, Channel::Say), 1).unwrap();
    let delivered = drain(&mut router);
    assert_eq!(
        delivered
            .iter()
            .map(|(key, _)| key.entity.value())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    for (_, message) in &delivered {
        assert_eq!(message.sender, key(1));
        assert_eq!(message.name.as_str(), "Arden");
        assert_eq!(message.text.as_str(), "Hello travellers");
    }
    router.submit(key(1), request(2, Channel::Area), 2).unwrap();
    assert_eq!(
        drain(&mut router)
            .iter()
            .map(|(key, _)| key.entity.value())
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    assert!(router.update(
        key(2),
        GlobalPosition::new([26., 0., 0.]).unwrap(),
        Area::new(1)
    ));
    router.submit(key(1), request(3, Channel::Say), 3).unwrap();
    assert_eq!(drain(&mut router).len(), 1);
    assert_eq!(router.stats().accepted, 3);
}
#[test]
fn moderation_replay_context_and_retirement_do_not_reset_live_flood_gate() {
    let mut router = router();
    router.admit(speaker(1, 0., 1)).unwrap();
    router.admit(speaker(2, 1., 1)).unwrap();
    assert!(router.admit(speaker(1, 0., 1)).is_err());
    assert!(router.mute(key(1), true));
    assert_eq!(
        router.submit(key(1), request(1, Channel::Say), 1),
        Err(Rejected::Policy(Refusal::Muted))
    );
    assert!(router.mute(key(1), false));
    assert_eq!(
        router.submit(key(1), request(1, Channel::Say), 2),
        Err(Rejected::Policy(Refusal::Sequence))
    );
    router.submit(key(1), request(2, Channel::Say), 2).unwrap();
    assert!(router.block(key(2), key(1), true));
    assert_eq!(drain(&mut router).len(), 1);
    assert!(router.block(key(2), key(1), false));
    let mut stale = request(3, Channel::Say);
    stale.epoch = WorldEpoch::new(8).unwrap();
    assert_eq!(router.submit(key(1), stale, 3), Err(Rejected::Admission));
    assert_eq!(
        router.submit(key(3), request(3, Channel::Say), 3),
        Err(Rejected::Admission)
    );
    router.submit(key(1), request(3, Channel::Say), 3).unwrap();
    assert!(router.retire(key(2)));
    router.admit(speaker(2, 1., 1)).unwrap();
    // The replacement recipient cannot inherit the old queued message.
    assert_eq!(drain(&mut router).len(), 1);
    router.submit(key(1), request(4, Channel::Say), 4).unwrap();
    assert_eq!(
        router.submit(key(1), request(5, Channel::Area), 5),
        Err(Rejected::Policy(Refusal::RateLimited))
    );
}
#[test]
fn slow_outboxes_drop_new_text_at_fixed_capacity_and_flush_one_per_peer_without_blocking() {
    let mut router = router();
    router.admit(speaker(1, 0., 1)).unwrap();
    router.admit(speaker(2, 1., 1)).unwrap();
    for sequence in 1..=20 {
        router
            .submit(key(1), request(sequence, Channel::Say), sequence * 150)
            .unwrap();
    }
    assert_eq!(router.stats().queued, 32);
    assert_eq!(router.stats().dropped_full, 8);
    router.flush(|_, _| false).unwrap();
    assert_eq!(router.stats().deferred, 2);
    for expected in 1..=OUTBOX_MESSAGES {
        let messages = drain(&mut router);
        assert_eq!(messages.len(), 2);
        assert!(
            messages
                .iter()
                .all(|(_, message)| message.sequence == expected as u64)
        );
    }
    assert!(drain(&mut router).is_empty());
    assert_eq!(router.stats().sent, 32);
    assert!(Router::new([0; 32], key(1).epoch, TickRate::Hz30).is_err());
    for id in 3..=MAX_PEERS as u64 {
        router.admit(speaker(id, 0., 1)).unwrap();
    }
    assert!(router.admit(speaker(MAX_PEERS as u64 + 1, 0., 1)).is_err());
    let mut wrong = speaker(99, 0., 1);
    wrong.key.epoch = WorldEpoch::new(8).unwrap();
    assert!(router.admit(wrong).is_err());
}

#[test]
fn delivery_serial_is_world_owned_monotonic_across_senders_and_refusals_and_never_wraps() {
    let mut router = router();
    router.admit(speaker(1, 0., 1)).unwrap();
    router.admit(speaker(2, 1., 1)).unwrap();
    router.submit(key(1), request(1, Channel::Say), 1).unwrap();
    let first = drain(&mut router);
    assert!(first.iter().all(|(_, message)| message.serial == 1));
    router.submit(key(2), request(1, Channel::Say), 2).unwrap();
    assert!(
        drain(&mut router)
            .iter()
            .all(|(_, message)| message.serial == 2)
    );
    assert_eq!(
        router.submit(key(1), request(2, Channel::Say), 1),
        Err(Rejected::Policy(Refusal::Time))
    );
    router.submit(key(1), request(2, Channel::Say), 2).unwrap();
    assert!(
        drain(&mut router)
            .iter()
            .all(|(_, message)| message.serial == 3)
    );
    router.serial = u64::MAX;
    assert_eq!(
        router.submit(key(1), request(3, Channel::Say), 3),
        Err(Rejected::Exhausted)
    );
    assert!(drain(&mut router).is_empty());
}
