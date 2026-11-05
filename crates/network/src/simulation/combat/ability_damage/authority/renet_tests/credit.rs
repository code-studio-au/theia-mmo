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
    exercise_credit(false, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_per_life_loot_requires_private_pickup_and_preserves_full_bag_availability_at_all_rates()
 {
    exercise_credit(true, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_conditional_loot_routes_original_tag_life_and_private_pickup_at_all_rates() {
    exercise_credit(true, true).await;
}
async fn exercise_credit(per_life: bool, conditional: bool) {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut f = fixture_mode(rate, true);
        let npc = f.npc;
        f.world.enable_creature_credit(SOLO).unwrap();
        let xp = Arc::new(crate::progression_wire::Profile::from_json(XP).unwrap());
        let placement = [8008., 8002.];
        if conditional {
            let talk = Arc::new(
                crate::dialogue::Profile::from_json(include_bytes!(
                    "../../../../../../../../content/quests/development-journal-dialogue.json"
                ))
                .unwrap(),
            );
            f.world
                .spawn_development_guide(talk.clone(), placement)
                .unwrap();
            let quest = serde_json::json!({"schema":3,"id":"catalog","revision":1,"quests":[{
                "id":"quest","version":1,"title":"Tokens","description":"Encrypted fixture","prerequisites":[],
                "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
                "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
                "kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.training-token","required":3,"consume_on_turn_in":true}]}]}]});
            let journal = Arc::new(
                crate::quest_journal::Profile::from_json(&serde_json::to_vec(&quest).unwrap(), &xp)
                    .unwrap()
                    .with_inventory(&f.session.items)
                    .unwrap(),
            );
            f.world
                .enable_item_quest_journal(journal.clone(), xp.clone(), &f.session.items)
                .unwrap();
            let mut value: serde_json::Value = serde_json::from_slice(REWARDS).unwrap();
            value["schema"] = 4.into();
            value["rewards"][0]["scope"] = "per_creature_life".into();
            value["rewards"][0]["chance_basis_points"] = 10_000.into();
            value["rewards"][0]["quest_required"] = true.into();
            f.session.items = f
                .session
                .items
                .clone()
                .with_quest_loot(&serde_json::to_vec(&value).unwrap(), &journal)
                .unwrap();
            f.session.journal = Some(journal);
            f.session.dialogue = Some(talk);
        } else {
            f.session.items = if per_life {
                let mut value: serde_json::Value = serde_json::from_slice(REWARDS).unwrap();
                value["schema"] = 2.into();
                value["rewards"][0]["scope"] = "per_creature_life".into();
                f.session
                    .items
                    .clone()
                    .with_creature_loot(&serde_json::to_vec(&value).unwrap())
                    .unwrap()
            } else {
                f.session.items.clone().with_rewards(REWARDS).unwrap()
            };
        }
        let rewards = Arc::new(f.session.items.rewards().unwrap().clone());
        if conditional {
            f.world.enable_quest_loot(rewards.clone()).unwrap();
        } else if per_life {
            f.world.enable_creature_loot(rewards.clone()).unwrap();
        } else {
            f.world.enable_rewards(rewards.clone()).unwrap();
        }
        if !conditional {
            f.world.enable_progression(xp.clone()).unwrap();
        }
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
        if let Some(journal) = &f.session.journal {
            f.session.source =
                crate::quest_journal::profile_identity(f.session.source, journal.hash());
            f.session.source = crate::dialogue::profile_identity(
                f.session.source,
                f.session.dialogue.as_ref().unwrap().hash(),
                placement,
            );
        }
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
                journal: f
                    .session
                    .journal
                    .as_ref()
                    .map(|j| net::progression::JournalObservation {
                        catalog: j.hash(),
                        revision: 0,
                        journal: Default::default(),
                        reply: None,
                    }),
                progress: Some(theia_core::progression::State::new(xp.profile(), 0, 0).unwrap()),
                ..Default::default()
            });
            progression_publishers.push(publish);
            ports.push(Ports {
                inventory,
                progression: net::progression::Port {
                    journal: f.session.journal.as_ref().map(|j| j.hash()),
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
                && (!conditional || s.quest_journal.is_some())
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
        let credit = match credit {
            net::progression::Input::Kill(credit) if !conditional => credit,
            net::progression::Input::Journal(net::progression::JournalInput {
                operation: net::progression::JournalOperation::Kill(credit),
                ..
            }) if conditional => credit,
            _ => panic!("Unexpected quest action"),
        };
        assert_eq!(credit.id, theia_core::progression::KillId::from_victim(npc));
        assert_eq!(credit.creature.as_str(), "theia.dev.training-creature");
        let reward = tokio::time::timeout(Duration::from_secs(2), reward_receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            reward.reward,
            rewards.catalog().definitions().next().unwrap().id().clone()
        );
        assert_eq!(reward.actor, owner);
        assert_eq!(reward.event, credit);
        assert!(reward.position.metres().into_iter().all(f64::is_finite));
        assert!(xp_receivers[1].try_recv().is_err());
        assert!(reward_receivers[1].try_recv().is_err());
        // Queued work has not been committed by these synthetic owners.
        assert_eq!(final_a.progression.as_ref().unwrap().progress.total(), 0);
        if per_life {
            use crate::{
                inventory_wire::{Outcome as ItemOutcome, Reason, Reply},
                renet_runtime::inventory::Pickup,
                reward_wire::Available,
            };
            use theia_core::rewards::{ClaimCommand, ClaimId, CorpseSource};
            let claim = ClaimId::new([77; 16]).unwrap();
            let baseline = final_a.inventory.as_ref().unwrap().bag.clone();
            assert!(final_a.rewards.as_ref().unwrap().available.is_empty());
            // Synthetic committed owner boundary: delivery is encrypted, but
            // SQL durability is separately qualified by persistence contracts.
            inventory_publishers[0].send_modify(|s| {
                s.rewards = Some(net::inventory::OwnedRewards {
                    generation: 2,
                    available: vec![Available {
                        reward: reward.reward.clone(),
                        claim,
                        corpse: Some(CorpseSource {
                            life: reward.event.id,
                            position: reward.position,
                        }),
                    }],
                })
            });
            let offered = a
                .until(|s, _| s.rewards.as_ref().is_some_and(|r| r.available.len() == 1))
                .await;
            assert_eq!(offered.inventory.as_ref().unwrap().bag, baseline);
            let command = ClaimCommand {
                claim,
                expected_revision: baseline.revision(),
            };
            // A competing final hitter cannot collect another owner's claim.
            b._pickups
                .try_send(Pickup {
                    command,
                    life: reward.event.id,
                })
                .unwrap();
            let denied = b
                .until(|s, _| {
                    s.inventory
                        .as_ref()
                        .and_then(|i| i.reply)
                        .is_some_and(|r| r.sequence == 1)
                })
                .await;
            assert_eq!(
                denied.inventory.unwrap().reply.unwrap().outcome,
                ItemOutcome::Rejected(Reason::Conflict)
            );
            assert!(f._requests.try_recv().is_err());
            a._pickups
                .try_send(Pickup {
                    command,
                    life: reward.event.id,
                })
                .unwrap();
            let first_pickup = tokio::time::timeout(Duration::from_secs(3), f._requests.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                first_pickup.command,
                theia_core::inventory::ItemCommand::RewardClaim(command)
            );
            assert_eq!(a.state.borrow().inventory.as_ref().unwrap().bag, baseline);
            inventory_publishers[0].send_modify(|s| {
                s.reply = Some(Reply {
                    sequence: first_pickup.sequence,
                    outcome: ItemOutcome::Rejected(Reason::Full),
                })
            });
            let full = a
                .until(|s, _| {
                    !s.inventory_pending
                        && s.inventory
                            .as_ref()
                            .and_then(|i| i.reply)
                            .is_some_and(|r| r.sequence == first_pickup.sequence)
                })
                .await;
            assert_eq!(full.inventory.as_ref().unwrap().bag, baseline);
            assert_eq!(full.rewards.as_ref().unwrap().available.len(), 1);
            a._pickups
                .try_send(Pickup {
                    command,
                    life: reward.event.id,
                })
                .unwrap();
            let retry = tokio::time::timeout(Duration::from_secs(3), f._requests.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(retry.sequence, first_pickup.sequence + 1);
            let granted = rewards
                .catalog()
                .definition(&reward.reward)
                .unwrap()
                .propose(&baseline, f.policy.0.catalog.as_ref(), baseline.revision())
                .unwrap();
            inventory_publishers[0].send_modify(|s| {
                s.bag = Some(granted.clone());
                s.reply = Some(Reply {
                    sequence: retry.sequence,
                    outcome: ItemOutcome::Applied,
                });
                s.rewards = Some(net::inventory::OwnedRewards {
                    generation: 3,
                    available: vec![],
                });
            });
            let collected = a
                .until(|s, _| {
                    !s.inventory_pending
                        && s.inventory
                            .as_ref()
                            .is_some_and(|i| i.bag.revision() == granted.revision())
                        && s.rewards.as_ref().is_some_and(|r| r.available.is_empty())
                })
                .await;
            assert_eq!(collected.inventory.unwrap().bag, granted);
            // A new sequence with the old claim is refused before owner enqueue.
            a._pickups
                .try_send(Pickup {
                    command: ClaimCommand {
                        expected_revision: granted.revision(),
                        ..command
                    },
                    life: reward.event.id,
                })
                .unwrap();
            let stale = a
                .until(|s, _| {
                    s.inventory
                        .as_ref()
                        .and_then(|i| i.reply)
                        .is_some_and(|r| r.sequence == retry.sequence + 1)
                })
                .await;
            assert_eq!(
                stale.inventory.unwrap().reply.unwrap().outcome,
                ItemOutcome::Rejected(Reason::Conflict)
            );
            assert!(f._requests.try_recv().is_err());
        }
        stop.send_replace(true);
        let seen_a = a.finish(false).await;
        let seen_b = b.finish(false).await;
        assert_eq!(seen_a.facts, seen_b.facts);
        assert_eq!(seen_a.facts[0].actor, owner);
        assert_ne!(seen_a.facts[1].actor, owner);
        assert_eq!(seen_a.facts[1].remaining, 0);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(
            stats.experience_opportunities_forwarded,
            u64::from(!conditional)
        );
        assert_eq!(stats.journal_kills_forwarded, u64::from(conditional));
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
