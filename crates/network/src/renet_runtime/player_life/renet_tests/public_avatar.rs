//! Real encrypted death/respawn/leave/rejoin. Development tickets and trusted
//! damage/timing profiles; no normal content/native/SQL/OIDC adoption claim.
use super::*;
use crate::avatar_life_wire as avatar;
use crate::renet_runtime::avatar_life as runtime;
use crate::renet_transport::*;
type PublicFixture = (
    World,
    EntityKey,
    Arc<Catalog>,
    Arc<Profile>,
    Vec<ChunkId>,
    Arc<avatar::Profile>,
    [u8; 32],
);
fn public_fixture(rate: TickRate) -> PublicFixture {
    let (mut world, npc, catalog, life, chunks) = fixture(rate, 40);
    let base = world.identity();
    world.enable_classic().unwrap();
    world.enable_public_avatar_life().unwrap();
    let public = world.public_avatar_life_profile().unwrap().clone();
    let source = public.identity(base);
    assert_eq!(world.identity(), crate::classic::profile_identity(source));
    (world, npc, catalog, life, chunks, public, source)
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_public_avatar_death_respawn_leave_and_rejoin_preserve_private_observer_all_rates()
 {
    qualify_public_avatar_rejoin(false).await;
}

// Keep the previously failing endpoint-reuse scenario in ordinary regression.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_public_avatar_same_endpoint_rejoin_all_rates() {
    qualify_public_avatar_rejoin(true).await;
}

async fn qualify_public_avatar_rejoin(reuse_backend_endpoint: bool) {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (world, npc, catalog, life, chunks, public, source) = public_fixture(rate);
        let identity = world.identity();
        let relays: Vec<_> = (0..3)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "avatar-life",
                    delay_ms: 20,
                    jitter_ms: 30,
                    loss_every: 11,
                    duplicate_every: 17,
                })
            })
            .collect();
        let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
        let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let advertised = if reuse_backend_endpoint {
            addresses[..2].to_vec()
        } else {
            addresses.clone()
        };
        let (transport, issuer) = net::secure_issuer(socket, advertised).unwrap();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let workers: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
        let mut a = Peer::start_mode(
            addresses[0],
            issuer.issue(1, None, 60).unwrap().bytes().to_vec(),
            identity,
            life.clone(),
            catalog.clone(),
            chunks.clone(),
            Some((source, public.clone())),
        );
        let mut b = Peer::start_mode(
            addresses[1],
            issuer.issue(2, None, 60).unwrap().bytes().to_vec(),
            identity,
            life.clone(),
            catalog.clone(),
            chunks.clone(),
            Some((source, public.clone())),
        );
        let ready = |s: &State, _: &[Observation]| {
            s.chunk_ack.is_some()
                && s.player_life.is_some()
                && s.public_avatar_life
                    .as_ref()
                    .and_then(|p| p.current())
                    .is_some_and(|p| p.actors.len() == 2)
        };
        let initial = a.until(ready).await;
        b.until(ready).await;
        let old = initial.player_life.unwrap().current().unwrap().key;
        a.cast(npc).await;
        a.until(|_, seen| seen.iter().any(|v| v.key == old && v.health == 0))
            .await;
        b.until_avatar(|_, seen| {
            seen.iter()
                .any(|v| v.actors.iter().any(|a| a.key == old && a.dead))
        })
        .await;
        let fresh = b
            .until_avatar(|s, _| {
                s.public_avatar_life
                    .as_ref()
                    .and_then(|p| p.actor(old.actor))
                    .is_some_and(|a| !a.dead && a.key.generation.get() == old.generation.get() + 1)
            })
            .await;
        let observer = fresh.player_life.as_ref().unwrap().current().unwrap();
        assert_eq!(observer.health, 100);
        assert!(observer.windup.is_none());
        assert!(b.seen.iter().all(|v| v.health == 100 && v.windup.is_none()));
        let old_packets: Vec<_> = controls[0]
            .lock()
            .unwrap()
            .captures
            .iter()
            .cloned()
            .collect();
        a.finish().await.unwrap();
        b.until_avatar(|s, _| {
            s.public_avatar_life.as_ref().is_some_and(|p| {
                p.actor(old.actor).is_none() && p.current().unwrap().actors.len() == 1
            })
        })
        .await;
        let replacement_relay = if reuse_backend_endpoint { 0 } else { 2 };
        let mut rejoined = Peer::start_mode(
            addresses[replacement_relay],
            issuer.issue(3, None, 60).unwrap().bytes().to_vec(),
            identity,
            life,
            catalog,
            chunks,
            Some((source, public)),
        );
        let rejoin = rejoined.until(ready).await;
        let new = rejoin.player_life.as_ref().unwrap().current().unwrap().key;
        assert_ne!(new.actor, old.actor);
        assert_eq!(new.generation.get(), 1);
        b.until_avatar(|s, _| {
            s.public_avatar_life.as_ref().is_some_and(|p| {
                p.actor(old.actor).is_none() && p.actor(new.actor).is_some_and(|a| !a.dead)
            })
        })
        .await;
        // The ordinary acceptance uses a fresh relay endpoint, as a new native
        // client socket does. The manual diagnostic also exercises reuse of the
        // same backend endpoint. Neither logs nor persists encrypted packets.
        {
            let mut control = controls[replacement_relay].lock().unwrap();
            control.stats.injected_old_packets += old_packets.len() as u64;
            control.inject.extend(old_packets);
        }
        let barrier = rejoin.snapshot.as_ref().unwrap().tick + u64::from(rate.hz() / 2);
        let intact = rejoined
            .until(|s, _| s.snapshot.as_ref().is_some_and(|v| v.tick >= barrier))
            .await;
        assert_eq!(
            intact.player_life.as_ref().unwrap().current().unwrap().key,
            new
        );
        assert_eq!(
            intact
                .public_avatar_life
                .as_ref()
                .unwrap()
                .actor(new.actor)
                .unwrap()
                .key,
            new
        );
        rejoined.finish().await.unwrap();
        b.finish().await.unwrap();
        stop.send_replace(true);
        let stats = server.await.unwrap().unwrap();
        assert!(stats.avatar_life_states_sent >= 8);
        assert!(
            stats.avatar_life_max_frame_bytes <= avatar::HEADER_BYTES + 2 * avatar::ACTOR_BYTES
        );
        assert!(stats.avatar_life_max_queue_bytes <= CHANNEL_MEMORY_BYTES);
        assert_eq!(stats.avatar_life_backpressure_disconnects, 0);
        assert_eq!(stats.avatar_life_uploads_rejected, 0);
        assert_eq!(
            stats.public_attack_states_sent, 0,
            "Independent avatar mode need not publish NPC timing"
        );
        for worker in workers {
            worker.await.unwrap();
        }
        let totals: Vec<_> = controls
            .iter()
            .map(|c| c.lock().unwrap().stats.clone())
            .collect();
        assert!(totals.iter().any(|s| s.configured_drops > 0));
        assert!(totals.iter().any(|s| s.duplicates > 0));
        assert!(totals.iter().any(|s| s.reordered > 0));
        assert!(totals.iter().all(|s| s.overflow == 0));
        eprintln!(
            "{}",
            serde_json::json!({"public_avatar_impairment":{"reuse_backend_endpoint":reuse_backend_endpoint,"tick_hz":rate.hz(),"frames":stats.avatar_life_states_sent,"max_bytes":stats.avatar_life_max_frame_bytes,"relays":totals}})
        );
    }
}
#[test]
fn readiness_and_full_server_channel_never_mark_unsent_lifecycle_delivered() {
    let (mut world, _, _, _, _, _, _) = public_fixture(TickRate::Hz30);
    world.require_client_chunks().unwrap();
    let mut server = RenetServer::new(connection_config_with_public_presentation());
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
    runtime::publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert_eq!(stats.avatar_life_states_sent, 0);
    peers.get_mut(&1).unwrap().public_presentation_ready = true;
    runtime::publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert_eq!(stats.avatar_life_states_sent, 1);
    world.step();
    runtime::publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert_eq!(stats.avatar_life_states_sent, 1);
    peers.get_mut(&1).unwrap().avatar_life_sent = None;
    let available = server.channel_available_memory(1, CRITICAL_PRESENTATION_CHANNEL);
    server.send_message(1, CRITICAL_PRESENTATION_CHANNEL, vec![0; available]);
    runtime::publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert!(peers.is_empty());
    assert!(world.player_life_snapshot(actor).is_none());
    assert_eq!(
        (
            stats.avatar_life_states_sent,
            stats.avatar_life_backpressure_disconnects
        ),
        (1, 1)
    );
}
#[test]
fn bounded_public_presentation_refuses_full_queue_without_advancing_roster() {
    let (mut world, _, _, _, _, profile, _) = public_fixture(TickRate::Hz30);
    let id = world.join().unwrap();
    let welcome = Welcome {
        id,
        epoch: world.epoch(),
        world: world.identity(),
        origin: world.origin(),
        terrain_stream: true,
    };
    let mut view = world.public_avatar_life_snapshot().unwrap().unwrap();
    let (events, mut rx) = mpsc::channel(runtime::PRESENTATION_CAPACITY);
    let mode = runtime::Client {
        profile: profile.clone(),
        events,
    };
    mode.validate().unwrap();
    let mut current = None;
    let mut stats = TransportStats::default();
    for tick in 0..runtime::PRESENTATION_CAPACITY as u64 {
        view.tick = tick;
        runtime::receive(
            &mode,
            &welcome,
            [None, None],
            &avatar::encode(&profile, TickRate::Hz30, &view).unwrap(),
            &mut current,
            &mut stats,
        )
        .unwrap();
    }
    let duplicate = avatar::encode(&profile, TickRate::Hz30, &view).unwrap();
    runtime::receive(
        &mode,
        &welcome,
        [None, None],
        &duplicate,
        &mut current,
        &mut stats,
    )
    .unwrap();
    assert_eq!(stats.avatar_life_duplicate_states, 1);
    let before = current.as_ref().unwrap().current().unwrap().clone();
    view.tick += 1;
    assert!(
        runtime::receive(
            &mode,
            &welcome,
            [None, None],
            &avatar::encode(&profile, TickRate::Hz30, &view).unwrap(),
            &mut current,
            &mut stats
        )
        .is_err()
    );
    assert_eq!(current.as_ref().unwrap().current(), Some(&before));
    assert_eq!(
        stats.avatar_life_states_received,
        runtime::PRESENTATION_CAPACITY as u64
    );
    assert_eq!(rx.len(), runtime::PRESENTATION_CAPACITY);
    rx.close();
    assert!(mode.validate().is_err());
}

async fn upload(address: SocketAddr, ticket: Vec<u8>, identity: [u8; 32], channel: u8) {
    let token = net::read_ticket(&ticket, address).unwrap();
    let mut transport = secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        net::now().unwrap(),
        token,
    )
    .unwrap();
    let mut client = renet::RenetClient::new(connection_config_with_public_presentation());
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
                client.send_message(channel, avatar::TAG.to_vec());
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
    .expect("Forged public lifecycle upload was not rejected");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_forged_avatar_state_on_both_channels_is_rejected_before_gameplay() {
    for channel in [CRITICAL_PRESENTATION_CHANNEL, EVENT_CHANNEL] {
        let (world, _, _, _, _, _, _) = public_fixture(TickRate::Hz30);
        let identity = world.identity();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
        let ticket = issuer.issue(1, None, 60).unwrap().bytes().to_vec();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
        upload(address, ticket, identity, channel).await;
        stop.send_replace(true);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(stats.avatar_life_uploads_rejected, 1);
        assert_eq!(stats.avatar_life_states_sent, 0);
        assert_eq!(stats.player_life.unwrap().strikes, 0);
    }
}

#[tokio::test]
async fn foreign_public_profile_is_refused_before_ticket_or_socket_io() {
    let (world, _, catalog, life, chunks, _, source) = public_fixture(TickRate::Hz30);
    let foreign = Arc::new(avatar::Profile::new(Arc::new(
        Profile::from_json(STATS, CREATURE, POLICY, RESPAWN).unwrap(),
    )));
    let peer = Peer::start_mode(
        "127.0.0.1:9".parse().unwrap(),
        vec![],
        world.identity(),
        life,
        catalog,
        chunks,
        Some((source, foreign)),
    );
    let error = peer.finish().await.unwrap_err().to_string();
    assert!(
        error.contains("Public avatar life requires matching Classic player life"),
        "{error}"
    );
}
