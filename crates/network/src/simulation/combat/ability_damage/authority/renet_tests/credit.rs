//! Actual encrypted two-peer recipient routing. Owners are bounded synthetic
//! ports; queue acceptance here is deliberately not SQL/quest/loot durability.
use super::*;
const SOLO: &[u8] = include_bytes!(
    "../../../../../../../../content/gameplay/development-creature-credit-solo.json"
);
const REWARDS: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-rewards.json");
const XP: &[u8] =
    include_bytes!("../../../../../../../../content/gameplay/development-progression.json");
#[derive(Clone)]
struct Ports {
    inventory: net::inventory::Port,
    progression: net::progression::Port,
}
struct CreditAdmissions {
    ports: [Ports; 2],
    position: GlobalPosition,
}
struct CreditLease(Ports);
impl AdmissionLease for CreditLease {
    fn inventory(&self) -> Result<Option<net::inventory::Port>> {
        Ok(Some(self.0.inventory.clone()))
    }
    fn progression(&self) -> Result<Option<net::progression::Port>> {
        Ok(Some(self.0.progression.clone()))
    }
    fn live(&self) -> Result<bool> {
        Ok(true)
    }
    fn record(&self, _: EntityKey, _: GlobalPosition, _: u64) -> Result<()> {
        Ok(())
    }
}
impl AccountAdmissions for CreditAdmissions {
    fn consume(self: Arc<Self>, id: u64, _: &[u8; 256]) -> Result<Option<PlayerAdmission>> {
        Ok((1..=2).contains(&id).then(|| PlayerAdmission {
            position: self.position,
            lease: Box::new(CreditLease(self.ports[id as usize - 1].clone())),
        }))
    }
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; 8]> {
        Ok([None; 8])
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_competing_hitter_routes_only_tag_owner_to_xp_and_reward_queues_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut f = fixture_mode(rate, true);
        let npc = f.npc;
        f.world.enable_creature_credit(SOLO).unwrap();
        f.session.items = f.session.items.clone().with_rewards(REWARDS).unwrap();
        let rewards = Arc::new(f.session.items.rewards().unwrap().clone());
        let xp = Arc::new(crate::progression_wire::Profile::from_json(XP).unwrap());
        f.world.enable_rewards(rewards.clone()).unwrap();
        f.world.enable_progression(xp.clone()).unwrap();
        f.session.progression = Some(xp.clone());
        f.session.identity = f.world.identity();
        let combat_source = f
            .world
            .combat
            .as_ref()
            .unwrap()
            .identity(f.world.patrol_identity.unwrap());
        f.session.source = crate::progression_wire::profile_identity(
            crate::simulation::rewards::profile_identity(combat_source, rewards.hash()),
            xp.hash(),
        );
        assert_eq!(
            f.session.identity,
            crate::classic::profile_identity(f.session.source)
        );
        let mut ports = Vec::new();
        let mut xp_receivers = Vec::new();
        let mut reward_receivers = Vec::new();
        let mut inventory_publishers = Vec::new();
        let mut progression_publishers = Vec::new();
        for _ in 0..2 {
            let mut inventory = f.policy.0.clone();
            let mut observation = inventory.state.borrow().clone();
            observation.rewards = Some(net::inventory::OwnedRewards {
                generation: 1,
                available: vec![],
            });
            let (publish, state) = watch::channel(observation);
            inventory_publishers.push(publish);
            let (opportunities, reward_rx) = mpsc::channel(16);
            reward_receivers.push(reward_rx);
            inventory.state = state;
            inventory.rewards = Some(rewards.clone());
            inventory.opportunities = Some(opportunities);
            let (input, xp_rx) = mpsc::channel(net::progression::CAPACITY);
            xp_receivers.push(xp_rx);
            let (publish, state) = watch::channel(net::progression::Observation {
                progress: Some(theia_core::progression::State::new(xp.profile(), 0, 0).unwrap()),
                ..Default::default()
            });
            progression_publishers.push(publish);
            ports.push(Ports {
                inventory,
                progression: net::progression::Port {
                    profile: xp.clone(),
                    quest: None,
                    lifecycle: None,
                    input,
                    state,
                },
            });
        }
        let admissions = Arc::new(CreditAdmissions {
            ports: ports.try_into().ok().unwrap(),
            position: f.policy.1,
        });
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "credit-wifi",
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
            admissions,
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
            s.player_life
                .as_ref()
                .and_then(|life| life.current())
                .and_then(|life| life.combat)
                .is_some()
                && s.progression.is_some()
                && s.inventory.is_some()
                && s.rewards.is_some()
                && s.correction.is_some()
                && s.snapshot.as_ref().is_some_and(|v| v.players.len() == 2)
        };
        let first = a.until(ready).await;
        b.until(ready).await;
        let owner = first
            .player_life
            .as_ref()
            .unwrap()
            .current()
            .unwrap()
            .key
            .actor;
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        a.until(|_, seen| seen.facts.len() == 1).await;
        b.until(|_, seen| seen.facts.len() == 1).await;
        assert!(xp_receivers.iter_mut().all(|r| r.try_recv().is_err()));
        assert!(reward_receivers.iter_mut().all(|r| r.try_recv().is_err()));
        b._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        let final_a = a
            .until(|s, seen| {
                seen.facts.len() == 2
                    && s.player_life
                        .as_ref()
                        .and_then(|p| p.current())
                        .and_then(|p| p.combat)
                        .is_some_and(|m| !m.in_combat())
            })
            .await;
        b.until(|_, seen| seen.facts.len() == 2).await;
        let credit = tokio::time::timeout(Duration::from_secs(2), xp_receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        let net::progression::Input::Kill(credit) = credit else {
            panic!("Unexpected quest action")
        };
        assert_eq!(credit.id, theia_core::progression::KillId::from_victim(npc));
        assert_eq!(credit.creature.as_str(), "theia.dev.training-creature");
        let reward = tokio::time::timeout(Duration::from_secs(2), reward_receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            reward,
            rewards.catalog().definitions().next().unwrap().id().clone()
        );
        assert!(xp_receivers[1].try_recv().is_err());
        assert!(reward_receivers[1].try_recv().is_err());
        // Queued work has not been committed by these synthetic owners.
        assert_eq!(final_a.progression.as_ref().unwrap().progress.total(), 0);
        stop.send_replace(true);
        let seen_a = a.finish(false).await;
        let seen_b = b.finish(false).await;
        assert_eq!(seen_a.facts, seen_b.facts);
        assert_eq!(seen_a.facts[0].actor, owner);
        assert_ne!(seen_a.facts[1].actor, owner);
        assert_eq!(seen_a.facts[1].remaining, 0);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(stats.experience_opportunities_forwarded, 1);
        assert_eq!(stats.reward_opportunities_forwarded, 1);
        assert!(xp_receivers.iter_mut().all(|r| r.try_recv().is_err()));
        assert!(reward_receivers.iter_mut().all(|r| r.try_recv().is_err()));
        for r in relays {
            r.await.unwrap();
        }
        let totals: Vec<_> = controls
            .iter()
            .map(|c| c.lock().unwrap().stats.clone())
            .collect();
        assert!(totals.iter().any(|c| c.reordered > 0));
        drop((inventory_publishers, progression_publishers));
    }
}
