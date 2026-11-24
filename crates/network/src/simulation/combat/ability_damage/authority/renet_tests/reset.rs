//! Actual encrypted movement and return-reset observations, synthetic admitted
//! inventory only. No normal balance, OIDC, SQL, native or capacity claim.
use super::*;
const CREDIT: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-credit-solo.json"
);
const RESET: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-return-reset.json"
);
fn combat(state: &State) -> Option<theia_core::combat_membership::Summary> {
    state.player_life.as_ref()?.current()?.combat
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_movement_return_restores_same_npc_without_false_death_or_player_life_change_at_all_rates()
 {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut f = fixture_mode(rate, true);
        let npc = f.npc;
        f.world.enable_creature_credit(CREDIT).unwrap();
        f.world.enable_creature_reset(RESET).unwrap();
        f.session.source = f
            .world
            .combat
            .as_ref()
            .unwrap()
            .identity(f.world.patrol_identity.unwrap());
        f.session.identity = f.world.identity();
        assert_eq!(
            f.session.identity,
            crate::classic::profile_identity(f.session.source)
        );
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "reset-wifi",
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
            f.session.clone(),
            shutdown.clone(),
            false,
        );
        let mut b = Peer::start(
            addresses[1],
            tokens[1].clone(),
            f.session,
            shutdown.clone(),
            false,
        );
        let ready = |s: &State, _: &Seen| {
            combat(s).is_some_and(|m| !m.in_combat())
                && s.correction.is_some()
                && s.chunk_ack.is_some()
                && s.snapshot.as_ref().is_some_and(|v| v.players.len() == 2)
        };
        let initial = a.until(ready).await;
        b.until(ready).await;
        let initial_life = *initial.player_life.as_ref().unwrap().current().unwrap();
        let initial_position = initial.correction.unwrap().state.pose.feet;
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        a.until(|s, seen| seen.facts.len() == 1 && combat(s).is_some_and(|m| m.in_combat()))
            .await;
        b.until(|_, seen| seen.facts.len() == 1).await;
        a._input.send_replace(Intent {
            axes: [1000, 0],
            sprint: true, // Escape at the bound run speed, rather than walking.
            facing: Some(Facing::from_units(0)),
            ..Default::default()
        });
        let reset = |s: &State, seen: &Seen| {
            seen.facts.len() == 1
                && combat(s).is_some_and(|m| !m.in_combat())
                && s.combat.as_ref().is_some_and(|c| {
                    c.targets
                        .iter()
                        .any(|t| t.entity == npc.entity && t.current == t.maximum && t.current > 20)
                })
        };
        let final_a = a.until(reset).await;
        a._input.send_replace(Intent::default());
        let final_b = b.until(reset).await;
        let life = final_a.player_life.as_ref().unwrap().current().unwrap();
        assert_eq!(life.key, initial_life.key);
        assert_eq!(life.movement_stream, initial_life.movement_stream);
        assert!(life.health > 0 && life.windup.is_none());
        let position = final_a.correction.unwrap().state.pose.feet.metres();
        let initial_position = initial_position.metres();
        assert!((position[0] - initial_position[0]).hypot(position[2] - initial_position[2]) > 0.5);
        assert_eq!(
            final_b
                .player_life
                .as_ref()
                .unwrap()
                .current()
                .unwrap()
                .health,
            100
        );
        stop.send_replace(true);
        let seen_a = a.finish(false).await;
        let seen_b = b.finish(false).await;
        assert_eq!(seen_a.facts, seen_b.facts);
        assert_eq!(seen_a.facts.len(), 1);
        assert!(seen_a.facts[0].remaining > 0);
        assert_eq!(seen_a.deaths, 0);
        assert_eq!(seen_b.deaths, 0);
        assert_eq!(seen_a.respawns, 0);
        assert_eq!(seen_b.respawns, 0);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(stats.experience_opportunities_forwarded, 0);
        assert_eq!(stats.reward_opportunities_forwarded, 0);
        assert_eq!(stats.life_backpressure_disconnects, 0);
        for relay in relays {
            relay.await.unwrap();
        }
        assert!(
            controls
                .iter()
                .any(|control| control.lock().unwrap().stats.reordered > 0)
        );
    }
}
