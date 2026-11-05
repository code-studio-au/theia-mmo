//! Actual encrypted peers and server-owned threat under deterministic UDP
//! impairment. Admission/inventory are committed synthetic ports, not SQL/OIDC.
use super::*;
const SOLO: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-credit-solo.json"
);
const RESET: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-return-reset.json"
);
const THREAT: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-creature-threat.json");
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_competing_damage_transfers_private_windup_and_keeps_public_facts_all_rates() {
    competition(false, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_public_attack_timing_reaches_victim_and_observer_all_rates() {
    competition(true, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_avatar_and_npc_contracts_coexist_on_the_priority_channel_all_rates() {
    competition(true, true).await;
}
async fn competition(public_attacks: bool, public_avatar: bool) {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut f = fixture_mode(rate, true);
        // Durable reward consumers are outside this fixture. Keep the NPC alive
        // so that competing public ability/white swings can select a new victim.
        f.world.combat.as_mut().unwrap().creatures[0].current = 1000;
        f.world.enable_creature_credit(SOLO).unwrap();
        f.world.enable_creature_reset(RESET).unwrap();
        f.world.enable_creature_threat(THREAT).unwrap();
        if public_attacks {
            f.world.enable_public_creature_attacks().unwrap();
            f.session.public_attacks = f.world.public_creature_attack_profile().cloned();
        }
        if public_avatar {
            f.world.enable_public_avatar_life().unwrap();
            f.session.public_avatar = f.world.public_avatar_life_profile().cloned();
        }
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
        let npc = f.npc;
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "threat-wifi",
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
        let mut b = Peer::start(addresses[1], tokens[1].clone(), f.session, shutdown, false);
        let ready = |s: &State, _: &Seen| {
            (!public_avatar || s.public_avatar_life.is_some())
                && (!public_attacks || s.public_creature_attack.is_some())
                && s.autoattack.is_some()
                && s.chunk_ack.is_some()
                && s.inventory.is_some()
                && s.correction.is_some()
                && s.player_life
                    .as_ref()
                    .and_then(|p| p.current())
                    .and_then(|p| p.combat)
                    .is_some()
                && s.snapshot.as_ref().is_some_and(|v| v.players.len() == 2)
        };
        let initial_a = a.until(ready).await;
        let initial_b = b.until(ready).await;
        eprintln!("threat_stage ready rate={}", rate.hz());
        let a_life = initial_a.player_life.unwrap().current().unwrap().key;
        let b_life = initial_b.player_life.unwrap().current().unwrap().key;
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        a.until(|s, seen| {
            seen.facts.len() == 1
                && s.player_life
                    .as_ref()
                    .and_then(|p| p.current())
                    .is_some_and(|p| p.windup.is_some())
        })
        .await;
        eprintln!("threat_stage initial_windup");
        b.until(|_, seen| seen.facts.len() == 1).await;
        if public_attacks {
            for peer in [&mut a, &mut b] {
                let observed = peer
                    .until(|s, _| {
                        s.public_creature_attack
                            .as_ref()
                            .and_then(|p| p.current())
                            .is_some_and(|p| p.windup.is_some_and(|w| w.target == a_life.actor))
                    })
                    .await;
                assert_eq!(
                    observed.public_creature_attack.as_ref().unwrap().rate(),
                    rate
                );
            }
            let observer = b.state.borrow().clone();
            let private = observer.player_life.as_ref().unwrap().current().unwrap();
            assert!(private.windup.is_none());
            assert_eq!(private.health, 100);
        }
        b._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        b.attack(Some(npc));
        b.until(|s, seen| {
            seen.facts.len() == 2
                && s.player_life
                    .as_ref()
                    .and_then(|p| p.current())
                    .is_some_and(|p| p.windup.is_some_and(|w| w.target == b_life))
        })
        .await;
        eprintln!("threat_stage switched");
        b.attack(None);
        // Unchanged private state is intentionally not resent each tick. The
        // cancellation frame may precede the new victim's later windup frame.
        a.until(|s, seen| {
            seen.facts.len() == 2
                && s.player_life
                    .as_ref()
                    .and_then(|p| p.current())
                    .is_some_and(|p| p.windup.is_none() && p.combat.is_some_and(|m| m.in_combat()))
        })
        .await;
        eprintln!("threat_stage old_cancelled");
        // Await one actual strike against the new victim, rather than infer
        // transfer solely from client selection or a server-side counter.
        b.until(|_, seen| {
            seen.ordinary
                .iter()
                .any(|f| f.actor == npc && f.target == b_life.actor)
        })
        .await;
        eprintln!("threat_stage new_struck");
        if public_attacks {
            for peer in [&mut a, &mut b] {
                peer.until(|s, _| {
                    s.public_creature_attack
                        .as_ref()
                        .and_then(|p| p.current())
                        .is_some_and(|p| p.windup.is_some_and(|w| w.target == b_life.actor))
                })
                .await;
            }
        }
        let public_count = b
            .seen
            .borrow()
            .ordinary
            .iter()
            .filter(|f| f.target == npc)
            .count();
        a.until(|_, seen| seen.ordinary.iter().filter(|f| f.target == npc).count() >= public_count)
            .await;
        stop.send_replace(true);
        let seen_a = a.finish(false).await;
        let seen_b = b.finish(false).await;
        let stats = server.await.unwrap().unwrap();
        assert_eq!(seen_a.facts, seen_b.facts);
        if public_avatar {
            for seen in [&seen_a, &seen_b] {
                assert!(seen.avatars.iter().any(|v| v.actors.len() == 2));
                assert!(
                    seen.avatars
                        .iter()
                        .all(|v| v.actors.iter().all(|a| !a.dead))
                );
            }
            assert!(stats.avatar_life_states_sent >= 2);
            assert_eq!(stats.avatar_life_backpressure_disconnects, 0);
        } else {
            assert!(seen_a.avatars.is_empty() && seen_b.avatars.is_empty());
            assert_eq!(stats.avatar_life_states_sent, 0);
        }
        if public_attacks {
            // Both reliable streams preserve the same transitions. The final
            // shutdown can leave one last in-flight update, so compare their
            // common prefix and require the full initial/transfer sequence.
            let transitions = |seen: &Seen| {
                seen.public_attacks
                    .iter()
                    .skip_while(|p| p.windup.is_none())
                    .copied()
                    .collect::<Vec<_>>()
            };
            let public_a = transitions(&seen_a);
            let public_b = transitions(&seen_b);
            let common = public_a.len().min(public_b.len());
            assert!(common >= 3);
            assert_eq!(&public_a[..common], &public_b[..common]);
            for seen in [&seen_a, &seen_b] {
                assert!(
                    seen.public_attacks
                        .iter()
                        .any(|p| p.windup.is_some_and(|w| w.target == a_life.actor))
                );
                assert!(
                    seen.public_attacks
                        .iter()
                        .any(|p| p.windup.is_some_and(|w| w.target == b_life.actor))
                );
                assert!(
                    seen.public_attacks
                        .windows(2)
                        .any(|p| p[0].windup.is_some_and(|w| w.target == a_life.actor)
                            && p[1].windup.is_none())
                );
            }
            assert!(stats.public_attack_states_sent >= (common * 2) as u64);
            assert_eq!(
                stats.public_attack_max_frame_bytes,
                crate::creature_attack_wire::MAX_FRAME_BYTES
            );
            assert!(
                stats.public_attack_max_queue_bytes <= crate::renet_transport::CHANNEL_MEMORY_BYTES
            );
            assert_eq!(stats.public_attack_backpressure_disconnects, 0);
            assert_eq!(stats.public_attack_uploads_rejected, 0);
        } else {
            assert!(seen_a.public_attacks.is_empty() && seen_b.public_attacks.is_empty());
            assert_eq!(stats.public_attack_states_sent, 0);
        }
        // NPC retaliation is private to its victim. Both peers must agree on
        // public outgoing results, while neither receives the other's strike.
        let public = |seen: &Seen| {
            seen.ordinary
                .iter()
                .filter(|f| f.target == npc)
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(public(&seen_a), public(&seen_b));
        assert!(
            seen_a
                .ordinary
                .iter()
                .filter(|f| f.actor == npc)
                .all(|f| f.target == a_life.actor)
        );
        assert!(
            seen_b
                .ordinary
                .iter()
                .filter(|f| f.actor == npc)
                .all(|f| f.target == b_life.actor)
        );
        assert_eq!(seen_a.facts[0].actor, a_life.actor);
        assert_eq!(seen_a.facts[1].actor, b_life.actor);
        let threat = stats.creature_threat.unwrap();
        assert!(threat.transfers >= 1);
        let health_lost: u64 = seen_a
            .facts
            .iter()
            .filter(|f| f.target == npc)
            .map(|f| u64::from(f.result.damage.health_lost))
            .sum::<u64>()
            + seen_a
                .ordinary
                .iter()
                .filter(|f| f.target == npc)
                .map(|f| u64::from(f.result.health_lost))
                .sum::<u64>();
        assert_eq!(threat.health_lost, health_lost);
        for relay in relays {
            relay.await.unwrap();
        }
        let totals: Vec<_> = controls
            .iter()
            .map(|c| c.lock().unwrap().stats.clone())
            .collect();
        assert!(totals.iter().any(|s| s.reordered > 0));
        assert!(totals.iter().any(|s| s.configured_drops > 0));
        assert!(totals.iter().any(|s| s.duplicates > 0));
        assert!(totals.iter().all(|s| s.overflow == 0));
        eprintln!(
            "{}",
            serde_json::json!({"creature_threat_impairment":{"tick_hz":rate.hz(),"relays":totals}})
        );
    }
}
