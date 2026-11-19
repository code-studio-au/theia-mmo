use super::*;
use crate::{renet_runtime as net, renet_transport::*, simulation::creatures};
use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};
use theia_core::coordinates::GlobalPosition;

fn fixture() -> (World, Arc<Profile>) {
    let creature =
        include_bytes!("../../../../../content/gameplay/development-creature-patrol.json");
    let mut raw: serde_json::Value = serde_json::from_slice(creature).unwrap();
    raw["aggro"] = serde_json::json!({"perception_metres":12.,"leash_metres":20.,"chase_speed":5.,"return_speed":5.,"los_grace_ms":2000,"engagement_ms":30000,"reacquire_ms":2000});
    let creature = serde_json::to_vec(&raw).unwrap();
    let creature = creature.as_slice();
    let (mut world, _) =
        creatures::tests::fixture_profile_at(TickRate::Hz30, [8000., 8000.], creature);
    world.enable_prediction().unwrap();
    world.enable_classic().unwrap();
    let stats = include_bytes!("../../../../../content/gameplay/development-character-stats.json");
    world
        .enable_resource_combat(
            include_bytes!("../../../../../content/gameplay/development-abilities.json"),
            100,
            300,
            stats,
        )
        .unwrap();
    world
        .enable_player_life(Arc::new(
            crate::player_life_wire::Profile::from_json(
                stats,
                creature,
                include_bytes!(
                    "../../../../../content/gameplay/development-creature-retaliation.json"
                ),
                include_bytes!("../../../../../content/gameplay/development-player-respawn.json"),
            )
            .unwrap(),
        ))
        .unwrap();
    world.enable_public_creature_attacks().unwrap();
    let profile = world.public_creature_attack_profile().unwrap().clone();
    (world, profile)
}
fn welcome(world: &World) -> Welcome {
    Welcome {
        id: 2,
        epoch: world.epoch(),
        world: world.identity(),
        terrain_stream: true,
        origin: world.origin(),
    }
}
#[test]
fn critical_channel_is_bounded_reliable_and_first_only_in_opt_in_config() {
    let legacy = connection_config();
    assert_eq!(
        legacy
            .server_channels_config
            .iter()
            .map(|c| c.channel_id)
            .collect::<Vec<_>>(),
        vec![EVENT_CHANNEL, MOVEMENT_CHANNEL]
    );
    let active = connection_config_with_public_attacks();
    for channels in [
        &active.server_channels_config,
        &active.client_channels_config,
    ] {
        assert_eq!(
            channels.iter().map(|c| c.channel_id).collect::<Vec<_>>(),
            vec![CREATURE_ATTACK_CHANNEL, EVENT_CHANNEL, MOVEMENT_CHANNEL]
        );
        assert_eq!(channels[0].max_memory_usage_bytes, CHANNEL_MEMORY_BYTES);
        assert!(matches!(
            channels[0].send_type,
            renet::SendType::ReliableOrdered { .. }
        ));
    }
    assert_eq!(
        active.available_bytes_per_tick,
        legacy.available_bytes_per_tick
    );
}
#[test]
fn readiness_gates_initial_publication_and_full_critical_queue_retires_peer() {
    let (mut world, _) = fixture();
    world.require_client_chunks().unwrap();
    let mut server = RenetServer::new(connection_config_with_public_attacks());
    server.add_connection(1);
    let peer = net::admit_peer(
        &mut world,
        &mut server,
        1,
        None,
        None,
        tokio::time::Instant::now(),
    )
    .unwrap()
    .unwrap();
    let actor = peer.actor;
    let mut peers = BTreeMap::from([(1, peer)]);
    let mut stats = ServerStats::default();
    publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert_eq!(stats.public_attack_states_sent, 0);
    assert!(peers[&1].public_attack_sent.is_none());
    peers.get_mut(&1).unwrap().public_presentation_ready = true;
    publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert_eq!(stats.public_attack_states_sent, 1);
    world.step();
    publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert_eq!(
        stats.public_attack_states_sent, 1,
        "Tick-only changes are suppressed"
    );
    peers.get_mut(&1).unwrap().public_attack_sent = None;
    let available = server.channel_available_memory(1, CREATURE_ATTACK_CHANNEL);
    server.send_message(1, CREATURE_ATTACK_CHANNEL, vec![0; available]);
    publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert!(peers.is_empty());
    assert!(world.player_life_snapshot(actor).is_none());
    assert_eq!(
        (
            stats.public_attack_states_sent,
            stats.public_attack_backpressure_disconnects
        ),
        (1, 1)
    );
}
#[test]
fn receive_queue_failure_and_wrong_context_refuse_without_advancing_state() {
    let (world, profile) = fixture();
    let welcome = welcome(&world);
    let mut view = world.public_creature_attack_snapshot().unwrap().unwrap();
    let (events, mut rx) = mpsc::channel(PRESENTATION_CAPACITY);
    let mode = Client {
        profile: profile.clone(),
        events,
    };
    mode.validate().unwrap();
    let mut current = None;
    let mut stats = TransportStats::default();
    let initial = wire::encode(&profile, TickRate::Hz30, &view).unwrap();
    assert!(
        receive(
            &mode,
            &welcome,
            [Some(TickRate::Hz20), None],
            &initial,
            &mut current,
            &mut stats
        )
        .is_err()
    );
    assert!(current.is_none());
    receive(
        &mode,
        &welcome,
        [None, None],
        &initial,
        &mut current,
        &mut stats,
    )
    .unwrap();
    receive(
        &mode,
        &welcome,
        [None, None],
        &initial,
        &mut current,
        &mut stats,
    )
    .unwrap();
    assert_eq!(stats.public_attack_duplicate_states, 1);
    for tick in 1..PRESENTATION_CAPACITY as u64 {
        view.tick = tick;
        receive(
            &mode,
            &welcome,
            [None, None],
            &wire::encode(&profile, TickRate::Hz30, &view).unwrap(),
            &mut current,
            &mut stats,
        )
        .unwrap();
    }
    let old = *current.as_ref().unwrap().current().unwrap();
    view.tick += 1;
    assert!(
        receive(
            &mode,
            &welcome,
            [None, None],
            &wire::encode(&profile, TickRate::Hz30, &view).unwrap(),
            &mut current,
            &mut stats
        )
        .unwrap_err()
        .to_string()
        .contains("queue full")
    );
    assert_eq!(*current.as_ref().unwrap().current().unwrap(), old);
    assert_eq!(
        stats.public_attack_states_received,
        PRESENTATION_CAPACITY as u64
    );
    assert_eq!(rx.len(), PRESENTATION_CAPACITY);
    rx.close();
    assert!(mode.validate().is_err());
    let (events, _rx) = mpsc::channel(PRESENTATION_CAPACITY - 1);
    assert!(Client { profile, events }.validate().is_err());
}

async fn upload(address: SocketAddr, ticket: Vec<u8>, identity: [u8; 32], channel: u8) {
    let token = net::read_ticket(&ticket, address).unwrap();
    let mut transport = secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        net::now().unwrap(),
        token,
    )
    .unwrap();
    let mut client = renet::RenetClient::new(connection_config_with_public_attacks());
    let mut connection_confirmed = false;
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut welcomed = false;
        let mut sent = false;
        let mut last = tokio::time::Instant::now();
        loop {
            let now = tokio::time::Instant::now();
            let delta = now.duration_since(last);
            last = now;
            client.update(delta);
            if transport.update(delta, &mut client).is_err() || client.is_disconnected() {
                assert!(sent);
                break;
            }
            while let Some(bytes) = client.receive_message(EVENT_CHANNEL) {
                if !welcomed {
                    Welcome::decode(&bytes, identity).unwrap();
                    welcomed = true;
                }
            }
            if welcomed && !sent {
                client.send_message(channel, wire::TAG.to_vec());
                sent = true;
            }
            crate::renet_transport::confirm_connection(&mut client, &mut connection_confirmed)
                .unwrap();
            if transport.send_packets(&mut client).is_err() {
                assert!(sent);
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("Server did not reject public attack upload");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_public_attack_uploads_on_both_channels_disconnect_without_gameplay_effects() {
    for channel in [CREATURE_ATTACK_CHANNEL, EVENT_CHANNEL] {
        let (world, _) = fixture();
        let identity = world.identity();
        // World owns terrain; no external database, provider or desktop process.
        assert!(
            world
                .procedural()
                .unwrap()
                .patch(GlobalPosition::new([8000., 0., 8000.]).unwrap())
                .is_some()
        );
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
        let ticket = issuer.issue(1, None, 60).unwrap().bytes().to_vec();
        let (stop, shutdown) = tokio::sync::watch::channel(false);
        let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
        upload(address, ticket, identity, channel).await;
        stop.send_replace(true);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(stats.public_attack_uploads_rejected, 1);
        assert_eq!(
            stats.public_attack_states_sent, 0,
            "No readiness: no public publication"
        );
        assert_eq!(stats.player_life.unwrap().strikes, 0);
    }
}
