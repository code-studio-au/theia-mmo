use super::*;
use crate::chat_client::{Client, Compose};
use theia_core::{
    chat::{Area, Channel, Text},
    identity::{EntityId, WorldEpoch},
};
fn compose() -> Compose {
    Compose {
        channel: Channel::Say,
        text: Text::parse("Hello travellers").unwrap(),
    }
}
#[test]
fn worker_timeout_discards_queued_chat_without_disconnect_and_late_reply_preserves_new_pending() {
    let area = Area::new(1).unwrap();
    let world = crate::chat::profile_identity([1; 32], area);
    let (send, receive) = mpsc::channel(4);
    let mut driver =
        Driver::new(Client::new([1; 32], area, receive).unwrap(), world, false).unwrap();
    let welcome = Welcome {
        terrain_stream: true,
        id: 1,
        epoch: 7,
        world,
        origin: [0.; 3],
    };
    let (state, observed) = watch::channel(State {
        welcome: Some(welcome.clone()),
        ..Default::default()
    });
    send.try_send(compose()).unwrap(); // stale work before admission is discarded
    driver.admit(&welcome, &state).unwrap();
    let mut transport = RenetClient::new(connection_config());
    transport.set_connected();
    let mut stats = TransportStats::default();
    let now = Instant::now();
    driver
        .tick(&mut transport, now, &state, &mut stats)
        .unwrap();
    assert_eq!(stats.chat_requests_sent, 0);
    send.try_send(compose()).unwrap();
    driver
        .tick(&mut transport, now, &state, &mut stats)
        .unwrap();
    assert!(observed.borrow().chat.as_ref().unwrap().pending);
    assert_eq!(stats.chat_requests_sent, 1);
    send.try_send(compose()).unwrap();
    driver
        .tick(
            &mut transport,
            now + Duration::from_secs(5),
            &state,
            &mut stats,
        )
        .unwrap();
    assert!(observed.borrow().chat.as_ref().unwrap().timed_out);
    assert!(!observed.borrow().chat.as_ref().unwrap().pending);
    assert!(transport.is_connected());
    assert!(observed.borrow().error.is_none());
    assert_eq!(stats.chat_timeouts, 1);
    driver
        .tick(
            &mut transport,
            now + Duration::from_secs(6),
            &state,
            &mut stats,
        )
        .unwrap();
    assert_eq!(stats.chat_requests_sent, 1);
    send.try_send(compose()).unwrap();
    driver
        .tick(
            &mut transport,
            now + Duration::from_secs(6),
            &state,
            &mut stats,
        )
        .unwrap();
    let owner = EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(1).unwrap(),
    };
    let mut reply = crate::chat_wire::Reply {
        world,
        owner,
        sequence: 1,
        tick: 100,
        outcome: crate::chat_wire::Outcome::Accepted,
    };
    driver
        .receive(&reply.encode().unwrap(), &state, &mut stats)
        .unwrap();
    assert!(observed.borrow().chat.as_ref().unwrap().pending);
    reply.sequence = 2;
    reply.outcome = crate::chat_wire::Outcome::RateLimited;
    driver
        .receive(&reply.encode().unwrap(), &state, &mut stats)
        .unwrap();
    assert!(!observed.borrow().chat.as_ref().unwrap().pending);
    assert_eq!(
        observed
            .borrow()
            .chat
            .as_ref()
            .unwrap()
            .inbox
            .reply()
            .unwrap()
            .outcome,
        crate::chat_wire::Outcome::RateLimited
    );
    let (_, too_big) = mpsc::channel(5);
    assert!(Client::new([1; 32], area, too_big).is_err());
    let (_, wrong) = mpsc::channel(4);
    assert!(Driver::new(Client::new([1; 32], area, wrong).unwrap(), [2; 32], false).is_err());
}

#[test]
fn client_queue_pressure_preserves_unsent_text_and_sequence_exhaustion_refuses_without_disconnect()
{
    let area = Area::new(1).unwrap();
    let world = crate::chat::profile_identity([1; 32], area);
    let (submit, outgoing) = mpsc::channel(4);
    let mut driver =
        Driver::new(Client::new([1; 32], area, outgoing).unwrap(), world, false).unwrap();
    let welcome = Welcome {
        terrain_stream: true,
        id: 1,
        epoch: 7,
        world,
        origin: [0.; 3],
    };
    let (state, observed) = watch::channel(State {
        welcome: Some(welcome.clone()),
        ..Default::default()
    });
    driver.admit(&welcome, &state).unwrap();
    let mut client = RenetClient::new(connection_config());
    client.set_connected();
    let mut stats = TransportStats::default();
    let now = Instant::now();
    while client.can_send_message(EVENT_CHANNEL, GAMEPLAY_RESERVE_BYTES + 64) {
        client.send_message(EVENT_CHANNEL, vec![0; 64]);
    }
    submit.try_send(compose()).unwrap();
    driver.tick(&mut client, now, &state, &mut stats).unwrap();
    assert_eq!(stats.chat_requests_sent, 0);
    let mut recovered = RenetClient::new(connection_config());
    recovered.set_connected();
    driver
        .tick(&mut recovered, now, &state, &mut stats)
        .unwrap();
    assert_eq!(stats.chat_requests_sent, 1);
    driver.pending = None;
    let view = driver.view.as_mut().unwrap();
    view.submitted = u64::MAX;
    view.pending = false;
    submit.try_send(compose()).unwrap();
    driver
        .tick(&mut recovered, now, &state, &mut stats)
        .unwrap();
    assert!(observed.borrow().chat.as_ref().unwrap().exhausted);
    assert!(!observed.borrow().chat.as_ref().unwrap().pending);
    assert!(recovered.is_connected());
    assert!(observed.borrow().error.is_none());
    assert_eq!(stats.chat_requests_sent, 1);
    let mut foreign = welcome;
    foreign.world = [2; 32];
    assert!(driver.admit(&foreign, &state).is_err());
}
