//! Encrypted game command bridge with explicitly synthetic commit publications.
//! Real OIDC/SQL and retirement are qualified separately, not by this test.
use super::*;
use theia_core::{
    content::ContentId,
    quests::journal::{Action, Command as QuestCommand, Status},
};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn profiles() -> Profiles {
    let items = Profile::from_json(include_bytes!(
        "../../../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let xp = Arc::new(
        crate::progression_wire::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap(),
    );
    let q=serde_json::to_vec(&serde_json::json!({"schema":3,"id":"fixed-bridge-quests","revision":1,"quests":[{"id":"quest","version":1,"title":"Delivery","description":"Bring three stones","giver":"theia.dev.guide","turn_in":"theia.dev.guide","prerequisites":[],"repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"bring","stages":[{"id":"bring","next":null,"completion":{"kind":"all"},"objectives":[{"id":"stones","kind":"possess_item","label":"Stones","item":"theia.dev.training-stone","required":3,"consume_on_turn_in":true}]}]}]})).unwrap();
    let r=serde_json::to_vec(&serde_json::json!({"schema":1,"id":"fixed-bridge-rewards","revision":1,"quest_catalog":"fixed-bridge-quests","item_catalog":items.catalog().id().as_str(),"rewards":[{"quest":"quest","grants":[{"item":"theia.dev.training-token","count":2}]}]})).unwrap();
    let journal = Arc::new(
        crate::quest_journal::Profile::from_json(&q, &xp)
            .unwrap()
            .with_item_rewards(&r, &items)
            .unwrap(),
    );
    let talk = Arc::new(
        crate::dialogue::Profile::from_json(include_bytes!(
            "../../../../../content/quests/development-journal-dialogue.json"
        ))
        .unwrap(),
    );
    Profiles {
        items,
        journal,
        xp,
        talk,
    }
}
fn intent(
    npc: EntityKey,
    request: u128,
    action: Action,
    revision: u64,
) -> net::quest_journal::Intent {
    net::quest_journal::Intent {
        npc,
        dialogue_revision: 1,
        command: QuestCommand {
            request_id: std::num::NonZeroU128::new(request).unwrap(),
            quest: id("quest"),
            npc: id("theia.dev.guide"),
            action,
            expected_revision: revision,
        },
    }
}
async fn next(
    input: &mut mpsc::Receiver<net::progression::Input>,
) -> net::progression::JournalInput {
    let net::progression::Input::Journal(input) =
        tokio::time::timeout(Duration::from_secs(4), input.recv())
            .await
            .unwrap()
            .unwrap()
    else {
        panic!("wrong operation")
    };
    input
}
fn quest_command(input: &net::progression::JournalInput) -> QuestCommand {
    let net::progression::JournalOperation::Command(command) = &input.operation else {
        panic!("wrong operation")
    };
    command.clone()
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_fixed_rewards_capacity_commit_retry_and_private_state_at_all_rates() {
    use crate::{
        inventory_wire::{Outcome, Reply},
        quest_journal_wire::Rejection,
    };
    use net::progression::{JournalRejection, JournalReply};
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let profiles = profiles();
        let mut world = crate::simulation::World::with_procedural([8000., 8000.], 1.).unwrap();
        assert!(world.set_tick_rate(rate));
        let guide = world
            .spawn_development_guide(profiles.talk.clone(), [8000., 8000.])
            .unwrap();
        world
            .enable_item_reward_quest_journal(
                profiles.journal.clone(),
                profiles.xp.clone(),
                &profiles.items,
            )
            .unwrap();
        world.enable_prediction().unwrap();
        let identity = world.identity();
        let position = guide.position;
        let chunks = world
            .procedural()
            .unwrap()
            .patch(position)
            .unwrap()
            .ids
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let mut slots = std::array::from_fn(|_| {
            Stack::new(profiles.items.catalog(), id("theia.dev.training-token"), 99)
        });
        slots[0] = Stack::new(
            profiles.items.catalog(),
            theia_core::content::ContentId::new("theia.dev.training-stone").unwrap(),
            20,
        );
        let bag = Bag::from_slots(profiles.items.catalog(), 0, slots).unwrap();
        let mut publications = Vec::new();
        let mut receivers = Vec::new();
        let mut ports = Vec::new();
        let mut progress_ports = Vec::new();
        let mut keepalive = Vec::new();
        let mut progress_inputs = Vec::new();
        for _ in 0..2 {
            let (requests, receiver) = mpsc::channel(1);
            let (publish, state) = watch::channel(Observation {
                bag: Some(bag.clone()),
                ..Default::default()
            });
            publications.push(publish);
            receivers.push(receiver);
            ports.push(Port {
                documents: None,
                vendor: None,
                rewards: None,
                opportunities: None,
                catalog: Arc::new(profiles.items.catalog().clone()),
                hash: profiles.items.hash(),
                requests,
                state,
            });
            let (input, receiver) = mpsc::channel(net::progression::CAPACITY);
            let (publish, state) = watch::channel(net::progression::Observation {
                progress: Some(
                    theia_core::progression::State::new(profiles.xp.profile(), 0, 0).unwrap(),
                ),
                journal: Some(net::progression::JournalObservation {
                    catalog: profiles.journal.hash(),
                    revision: 0,
                    journal: Journal::default(),
                    reply: None,
                }),
                ..Default::default()
            });
            keepalive.push(publish);
            progress_inputs.push(receiver);
            progress_ports.push(net::progression::Port {
                journal: Some(profiles.journal.hash()),
                lifecycle: None,
                quest: None,
                profile: profiles.xp.clone(),
                input,
                state,
            });
        }
        let policy = Arc::new(Policy {
            items: ports.try_into().unwrap_or_else(|_| panic!("ports")),
            progression: progress_ports
                .try_into()
                .unwrap_or_else(|_| panic!("progress")),
            position,
            used: Mutex::new([false; 2]),
        });
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "fixed-rewards-impaired",
                    delay_ms: 20,
                    jitter_ms: 30,
                    loss_every: 11,
                    duplicate_every: 17,
                })
            })
            .collect();
        let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
        let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, addresses.clone()).unwrap();
        let tokens: Vec<_> = (1..=2)
            .map(|id| {
                let mut data = [0; 256];
                data[0] = id as u8;
                issuer.issue(id, Some(&data), 120).unwrap().bytes().to_vec()
            })
            .collect();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let workers: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_with_admissions(
            transport, shutdown, world, policy,
        ));
        let mut a = PeerClient::start(
            addresses[0],
            tokens[0].clone(),
            identity,
            &profiles,
            chunks.clone(),
        );
        let mut b = PeerClient::start(addresses[1], tokens[1].clone(), identity, &profiles, chunks);
        for client in [&mut a, &mut b] {
            client
                .until(|s| {
                    s.inventory.is_some()
                        && s.quest_journal.is_some()
                        && s.progression.is_some()
                        && s.chunk_ack.is_some()
                })
                .await;
        }

        a.until(|s| s.dialogue.is_some()).await;
        a._keepalive
            ._talk
            .try_send(net::dialogue::Intent {
                expected_revision: 0,
                action: crate::dialogue::Action::Open(guide.key),
            })
            .unwrap();
        a.until(|s| {
            s.dialogue
                .as_ref()
                .is_some_and(|v| v.revision == 1 && v.open.is_some())
                && !s.dialogue_pending
        })
        .await;
        let accept = intent(guide.key, 100, Action::Accept, 0);
        a._keepalive._actions.try_send(accept.clone()).unwrap();
        let accepted = next(&mut progress_inputs[0]).await;
        assert_eq!(quest_command(&accepted), accept.command);
        let core = profiles.journal.catalog();
        let journal = Journal::default()
            .propose_accept(core, &id("quest"), &id("theia.dev.guide"))
            .unwrap()
            .journal
            .propose_inventory(core, profiles.items.catalog(), &bag)
            .unwrap()
            .journal;
        let ready = journal.progress(&id("quest")).unwrap();
        assert_eq!(ready.status(), Status::Ready);
        let revision = journal.next_head_revision(0).unwrap();
        keepalive[0].send_modify(|s| {
            let j = s.journal.as_mut().unwrap();
            j.journal = journal.clone();
            j.revision = revision;
            j.reply = Some(JournalReply {
                sequence: accepted.sequence,
                rejection: None,
            });
        });
        a.until(|s| {
            s.quest_journal_reply
                .as_ref()
                .is_some_and(|r| r.sequence == 1 && r.rejection.is_none())
        })
        .await;
        let turn = intent(guide.key, 101, Action::TurnIn, ready.revision());
        a._keepalive._actions.try_send(turn.clone()).unwrap();
        let refused = next(&mut progress_inputs[0]).await;
        assert_eq!(quest_command(&refused), turn.command);
        let proposal = theia_core::quests::item_rewards::TurnIn {
            quest: &id("quest"),
            npc: &id("theia.dev.guide"),
            quest_revision: ready.revision(),
            inventory_revision: 0,
        };
        assert!(matches!(
            profiles
                .journal
                .item_rewards()
                .unwrap()
                .propose_turn_in(&journal, &bag, proposal),
            Err(theia_core::quests::item_rewards::TurnInError::Inventory(
                theia_core::rewards::Error::NoCapacity
            ))
        ));
        keepalive[0].send_modify(|s| {
            s.journal.as_mut().unwrap().reply = Some(JournalReply {
                sequence: refused.sequence,
                rejection: Some(JournalRejection::InventoryFull),
            })
        });
        a.until(|s| {
            s.quest_journal_reply
                .as_ref()
                .is_some_and(|r| r.sequence == 2 && r.rejection == Some(Rejection::InventoryFull))
                && !s.quest_journal_pending
        })
        .await;
        a.commands
            .try_send(
                Command {
                    expected_revision: 0,
                    operation: Operation::Destroy {
                        from: 15,
                        amount: 99,
                    },
                }
                .into(),
            )
            .unwrap();
        let free = tokio::time::timeout(Duration::from_secs(4), receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        let freed = free
            .command
            .propose(&bag, profiles.items.catalog())
            .unwrap();
        publications[0].send_modify(|s| {
            s.bag = Some(freed.clone());
            s.reply = Some(Reply {
                sequence: free.sequence,
                outcome: Outcome::Applied,
            });
        });
        a.until(|s| {
            s.inventory.as_ref().is_some_and(|v| v.bag.revision() == 1) && !s.inventory_pending
        })
        .await;
        a._keepalive._actions.try_send(turn.clone()).unwrap();
        let completing = next(&mut progress_inputs[0]).await;
        assert_eq!(quest_command(&completing), turn.command);
        a.until(|s| s.quest_journal_pending).await;
        tokio::time::sleep(Duration::from_millis(120)).await;
        assert_eq!(a.state.borrow().inventory.as_ref().unwrap().bag, freed);
        assert_eq!(
            a.state
                .borrow()
                .progression
                .as_ref()
                .unwrap()
                .progress
                .total(),
            0
        );
        assert_ne!(
            a.state
                .borrow()
                .quest_journal_reply
                .as_ref()
                .unwrap()
                .sequence,
            3
        );
        let done = profiles
            .journal
            .item_rewards()
            .unwrap()
            .propose_turn_in(
                &journal,
                &freed,
                theia_core::quests::item_rewards::TurnIn {
                    quest: &id("quest"),
                    npc: &id("theia.dev.guide"),
                    quest_revision: ready.revision(),
                    inventory_revision: 1,
                },
            )
            .unwrap();
        let finished = done.quest.journal;
        let after = done.bag;
        let head = finished.next_head_revision(revision).unwrap();
        let xp = profiles
            .xp
            .profile()
            .propose_grant(
                &theia_core::progression::State::new(profiles.xp.profile(), 0, 0).unwrap(),
                done.quest.reward.unwrap().experience,
            )
            .unwrap()
            .state;
        publications[0].send_modify(|s| s.bag = Some(after.clone()));
        keepalive[0].send_modify(|s| {
            s.progress = Some(xp.clone());
            let j = s.journal.as_mut().unwrap();
            j.journal = finished.clone();
            j.revision = head;
            j.reply = Some(JournalReply {
                sequence: completing.sequence,
                rejection: None,
            });
        });
        a.until(|s| {
            s.quest_journal_reply
                .as_ref()
                .is_some_and(|r| r.sequence == 3 && r.rejection.is_none())
                && s.inventory.as_ref().is_some_and(|v| v.bag.revision() == 2)
                && s.progression
                    .as_ref()
                    .is_some_and(|p| p.progress.total() == 25)
        })
        .await;
        assert_eq!(
            a.state.borrow().inventory.as_ref().unwrap().bag.slots()[15]
                .as_ref()
                .unwrap()
                .count(),
            2
        );
        // Ordinary deletion after completion must not be undone by an old successful retry.
        a.commands
            .try_send(
                Command {
                    expected_revision: 2,
                    operation: Operation::Destroy {
                        from: 15,
                        amount: 2,
                    },
                }
                .into(),
            )
            .unwrap();
        let remove = tokio::time::timeout(Duration::from_secs(4), receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        let removed = remove
            .command
            .propose(&after, profiles.items.catalog())
            .unwrap();
        publications[0].send_modify(|s| {
            s.bag = Some(removed.clone());
            s.reply = Some(Reply {
                sequence: remove.sequence,
                outcome: Outcome::Applied,
            });
        });
        a.until(|s| {
            s.inventory.as_ref().is_some_and(|v| v.bag.revision() == 3) && !s.inventory_pending
        })
        .await;
        a._keepalive._actions.try_send(turn.clone()).unwrap();
        let retry = next(&mut progress_inputs[0]).await;
        assert_eq!(quest_command(&retry), turn.command);
        keepalive[0].send_modify(|s| {
            s.journal.as_mut().unwrap().reply = Some(JournalReply {
                sequence: retry.sequence,
                rejection: None,
            })
        });
        a.until(|s| {
            s.quest_journal_reply
                .as_ref()
                .is_some_and(|r| r.sequence == 4 && r.rejection.is_none())
                && !s.quest_journal_pending
        })
        .await;
        assert_eq!(a.state.borrow().inventory.as_ref().unwrap().bag, removed);
        assert_eq!(
            a.state
                .borrow()
                .progression
                .as_ref()
                .unwrap()
                .progress
                .total(),
            25
        );
        assert_eq!(b.state.borrow().inventory.as_ref().unwrap().bag, bag);
        assert_eq!(
            b.state
                .borrow()
                .progression
                .as_ref()
                .unwrap()
                .progress
                .total(),
            0
        );
        assert!(
            b.state
                .borrow()
                .quest_journal
                .as_ref()
                .unwrap()
                .journal
                .quests()
                .next()
                .is_none()
        );
        assert!(b.state.borrow().quest_journal_reply.is_none());
        assert!(
            receivers[0].try_recv().is_err()
                && receivers[1].try_recv().is_err()
                && progress_inputs[0].try_recv().is_err()
                && progress_inputs[1].try_recv().is_err()
        );
        a.stop.send_replace(true);
        b.stop.send_replace(true);
        a.task.await.unwrap().unwrap();
        b.task.await.unwrap().unwrap();
        stop.send_replace(true);
        server.await.unwrap().unwrap();
        for worker in workers {
            worker.await.unwrap();
        }
        for control in controls {
            let c = control.lock().unwrap();
            assert!(c.stats.queue_high_water <= relay::QUEUE_LIMIT && c.stats.overflow == 0);
            assert!(c.stats.configured_drops > 0 && c.stats.duplicates > 0);
        }
    }
}
