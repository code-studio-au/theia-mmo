//! Actual encrypted two-peer PL02 observations with synthetic admitted inventory.
//! Deterministic zero-avoidance special and a 20-HP NPC are explicit fixtures.
use super::*;
fn combat(s: &State) -> Option<theia_core::combat_membership::Summary> {
    s.player_life.as_ref()?.current()?.combat
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_membership_is_owner_private_and_releases_on_kill_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let f = fixture_mode(rate, true);
        let npc = f.npc;
        let session = f.session.clone();
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "membership-wifi",
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
        let (transport, issuer) = net::secure_issuer(socket, addresses.clone()).unwrap();
        let tokens: Vec<_> = (1..=2)
            .map(|id| issuer.issue(id, None, 60).unwrap().bytes().to_vec())
            .collect();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let relays: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_with_admissions(
            transport,
            shutdown.clone(),
            f.world,
            f.policy,
        ));
        let mut a = Peer::start(
            addresses[0],
            tokens[0].clone(),
            session.clone(),
            shutdown.clone(),
            false,
        );
        let mut b = Peer::start(addresses[1], tokens[1].clone(), session, shutdown, false);
        let ready = |s: &State, _: &Seen| {
            combat(s).is_some_and(|c| !c.in_combat())
                && s.autoattack.is_some()
                && s.chunk_ack.is_some()
                && s.correction.is_some()
                && s.snapshot.as_ref().is_some_and(|v| v.players.len() == 2)
        };
        let initial = a.until(ready).await;
        b.until(ready).await;
        let owner = initial.player_life.as_ref().unwrap().current().unwrap().key;
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        a.until(|s, seen| {
            seen.facts.len() == 1
                && combat(s).is_some_and(|c| {
                    c.outgoing_hostility && c.active_encounter && c.incoming_hostility
                })
        })
        .await;
        let observer = b
            .until(|s, seen| seen.facts.len() == 1 && combat(s).is_some_and(|c| !c.in_combat()))
            .await;
        assert_eq!(
            observer
                .player_life
                .as_ref()
                .unwrap()
                .current()
                .unwrap()
                .health,
            100
        );
        assert_ne!(
            observer
                .player_life
                .as_ref()
                .unwrap()
                .current()
                .unwrap()
                .key
                .actor,
            owner.actor
        );
        a.until(|s, _| {
            s.combat.as_ref().is_some_and(|c| {
                c.gcd_until <= c.tick && c.cooldowns.iter().all(|(_, until)| *until <= c.tick)
            })
        })
        .await;
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        let ended = a
            .until(|s, seen| seen.facts.len() == 2 && combat(s).is_some_and(|c| !c.in_combat()))
            .await;
        b.until(|s, seen| seen.facts.len() == 2 && combat(s).is_some_and(|c| !c.in_combat()))
            .await;
        assert_eq!(
            ended.player_life.as_ref().unwrap().current().unwrap().key,
            owner
        );
        assert!(
            ended
                .player_life
                .as_ref()
                .unwrap()
                .movement_allowed(&ended.correction.unwrap().state)
        );
        stop.send_replace(true);
        let owner_seen = a.finish(false).await;
        let observer_seen = b.finish(false).await;
        let stats = server.await.unwrap().unwrap();
        assert_eq!(owner_seen.facts, observer_seen.facts);
        assert_eq!(owner_seen.facts.last().unwrap().remaining, 0);
        assert!(owner_seen.life.iter().any(|v| {
            v.combat
                .is_some_and(|c| c.active_encounter && c.incoming_hostility)
        }));
        assert!(
            owner_seen
                .life
                .iter()
                .all(|v| v.key.actor == owner.actor && v.combat.is_some())
        );
        assert!(!owner_seen.life.last().unwrap().combat.unwrap().in_combat());
        assert!(
            observer_seen
                .life
                .iter()
                .all(|v| v.key.actor != owner.actor && v.combat.is_some_and(|c| !c.in_combat()))
        );
        assert!(
            observer_seen
                .ordinary
                .iter()
                .all(|f| f.target_generation.is_none())
        );
        assert!((120..=144).contains(&stats.life_max_frame_bytes));
        assert!(stats.life_states_sent >= 4);
        for r in relays {
            r.await.unwrap();
        }
        for c in controls {
            let c = c.lock().unwrap();
            assert!(
                c.stats.configured_drops > 0 && c.stats.duplicates > 0 && c.stats.reordered > 0
            );
        }
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mismatched_membership_receive_modes_disconnect_without_silent_downgrade() {
    for enabled in [false, true] {
        let f = fixture_mode(TickRate::Hz30, enabled);
        let mut session = f.session;
        session.membership = if enabled {
            None
        } else {
            Some(Arc::new(
                crate::combat_membership_wire::Profile::from_json(include_bytes!(
                    "../../../../../../../../content/gameplay/development-combat-membership.json"
                ))
                .unwrap(),
            ))
        };
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
        let token = issuer.issue(1, None, 60).unwrap().bytes().to_vec();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let server = tokio::spawn(net::run_server_with_admissions(
            transport,
            shutdown.clone(),
            f.world,
            f.policy,
        ));
        let mut peer = Peer::start(address, token, session, shutdown, false);
        let failed = peer.until(|s, _| s.error.is_some()).await;
        assert!(failed.error.unwrap().contains("frame header"));
        assert!(peer.state.borrow().player_life.is_none());
        stop.send_replace(true);
        peer.finish(true).await;
        server.await.unwrap().unwrap();
    }
}
