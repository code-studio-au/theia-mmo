//! Actual provider/session, encrypted ticket and PostgreSQL retirement contract.
//! Production gate source is included verbatim so its private constructor remains
//! internal. This qualifier drives leases directly; it is not native game proof.
#[path = "../src/admissions.rs"]
mod admissions;
#[path = "admission_lease/documents.rs"]
mod document_owner;
#[path = "admission_lease/item_rewards.rs"]
mod item_reward_owner;
#[path = "../../../crates/persistence/tests/support/mod.rs"]
mod support;
use std::{
    io::{BufRead, Read, Write},
    net::UdpSocket,
    sync::{Arc, Mutex},
    time::Duration,
};
use theia_auth::{Provider, Sessions};
use theia_core::inventory::{Command as ItemCommand, Operation as ItemOperation};
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
};
use theia_network::{
    renet_runtime::{self as net, AccountAdmissions},
    renet_transport,
};
use theia_persistence::{Binding, CharacterName, RealmId, Store, Worker};
fn emit(value: serde_json::Value) {
    println!("{value}");
    std::io::stdout().flush().unwrap();
}
fn input() -> String {
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .take(16385)
        .read_line(&mut line)
        .unwrap();
    assert!(line.len() <= 16384);
    serde_json::from_str(&line).unwrap()
}
async fn inventory_until(
    port: &mut net::inventory::Port,
    predicate: impl Fn(&net::inventory::Observation) -> bool,
) -> net::inventory::Observation {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let state = port.state.borrow().clone();
            if predicate(&state) {
                return state;
            }
            assert!(
                !state.failed,
                "Inventory owner failed before expected observation"
            );
            port.state.changed().await.unwrap();
        }
    })
    .await
    .expect("Inventory owner observation timed out")
}
async fn wait_inventory_lock(transaction: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_millis(300),async {
        loop {
            transaction.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
            if let Some(row)=transaction.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND (query LIKE 'SELECT catalog_hash,%' OR query LIKE 'INSERT INTO theia.backpacks%')",&[]).await.unwrap() {return row.get(0)}
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }).await.expect("Inventory SQL did not wait on owned fixture row")
}
async fn progression_until(
    port: &mut net::progression::Port,
    predicate: impl Fn(&net::progression::Observation) -> bool,
) -> net::progression::Observation {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let state = port.state.borrow().clone();
            assert!(!state.failed, "Progression owner failed");
            if predicate(&state) {
                return state;
            }
            port.state.changed().await.unwrap();
        }
    })
    .await
    .expect("Progression owner observation timed out")
}
async fn wait_lifecycle_lock(tx: &tokio_postgres::Transaction<'_>) {
    tokio::time::timeout(Duration::from_millis(300),async { loop {
        tx.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
        if tx.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE '%theia.quest_lifecycles%'",&[]).await.unwrap().is_some(){break;}
        tokio::time::sleep(Duration::from_millis(2)).await;
    }}).await.expect("Quest SQL never reached its owned lock");
}
async fn connected_data(
    transport: &mut renet_netcode::NetcodeServerTransport,
    ticket: &[u8],
) -> [u8; 256] {
    let token = renet_netcode::ConnectToken::read(&mut std::io::Cursor::new(ticket)).unwrap();
    let mut client = renet::RenetClient::new(renet_transport::connection_config());
    let mut client_transport = renet_transport::secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        net::now().unwrap(),
        token,
    )
    .unwrap();
    let mut server = renet::RenetServer::new(renet_transport::connection_config());
    for _ in 0..300 {
        let delta = Duration::from_millis(5);
        client.update(delta);
        client_transport.update(delta, &mut client).unwrap();
        client_transport.send_packets(&mut client).unwrap();
        server.update(delta);
        transport.update(delta, &mut server).unwrap();
        transport.send_packets(&mut server);
        if let Some(data) = transport.user_data(1) {
            transport.disconnect_all(&mut server);
            return data;
        }
        tokio::time::sleep(delta).await;
    }
    panic!("Actual encrypted ticket did not connect");
}
#[tokio::main]
async fn main() -> theia_network::Result<()> {
    let provider = Provider::discover_loopback(
        &std::env::var("THEIA_TEST_OIDC_ISSUER")?,
        &std::env::var("THEIA_TEST_OIDC_CLIENT")?,
        &std::env::var("THEIA_TEST_OIDC_REDIRECT")?,
    )
    .await?;
    let attempt = provider.begin();
    emit(serde_json::json!({"authorization_url":attempt.authorization_url().as_str()}));
    let identity = attempt.finish(&input()).await?;
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(identity.issuer(), identity.subject())
        .await?;
    let world = [103; 32];
    let position = GlobalPosition::new([15400., 3., 12860.]).unwrap();
    let character = store
        .create_character(owner, world, CharacterName::parse("Arden")?, position)
        .await?;
    let binding = character.saved.binding;
    let sessions = Arc::new(Mutex::new(Sessions::default()));
    let secret = sessions.lock().unwrap().issue(&identity, owner)?;
    let context = sessions.lock().unwrap().authorize(secret.expose_secret())?;
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let address = socket.local_addr()?;
    let (mut transport, issuer) = net::secure_issuer(socket, vec![address])?;
    let item_rewards = std::env::var("THEIA_TEST_QUEST_ITEM_REWARDS").is_ok_and(|v| v == "1");
    let document_journal = std::env::var("THEIA_TEST_QUEST_DOCUMENTS").is_ok_and(|v| v == "1");
    let quest_loot = std::env::var("THEIA_TEST_QUEST_LOOT").is_ok_and(|v| v == "1");
    let item_journal = std::env::var("THEIA_TEST_QUEST_ITEMS").is_ok_and(|v| v == "1");
    let vendors = std::env::var("THEIA_TEST_VENDORS").is_ok_and(|v| v == "1");
    let inventory = item_rewards
        || document_journal
        || quest_loot
        || item_journal
        || vendors
        || std::env::var("THEIA_TEST_INVENTORY").is_ok_and(|v| v == "1");
    let creature_loot = std::env::var("THEIA_TEST_CREATURE_LOOT").is_ok_and(|v| v == "1");
    let rewards_enabled =
        creature_loot || std::env::var("THEIA_TEST_REWARDS").is_ok_and(|v| v == "1");
    let item_bytes: &[u8] = if quest_loot {
        include_bytes!("../../../content/gameplay/development-loot-items.json")
    } else if item_rewards || document_journal || item_journal || rewards_enabled || vendors {
        include_bytes!("../../../content/gameplay/development-equipment-items.json")
    } else {
        include_bytes!("../../../content/gameplay/development-items.json")
    };
    let profile = theia_persistence::InventoryCatalog::from_json(item_bytes)?;
    let mut recipe: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-rewards.json"
    ))?;
    let mut second = recipe["rewards"][0].clone();
    second["id"] = "theia.dev.second-training-kill".into();
    recipe["rewards"].as_array_mut().unwrap().push(second);
    if creature_loot {
        recipe["schema"] = 3.into();
        recipe["rewards"].as_array_mut().unwrap().truncate(1);
        recipe["rewards"][0]["scope"] = "per_creature_life".into();
        recipe["rewards"][0]["chance_basis_points"] = 10_000.into();
        let mut miss = recipe["rewards"][0].clone();
        miss["id"] = "theia.dev.zero-drop".into();
        miss["chance_basis_points"] = 0.into();
        recipe["rewards"].as_array_mut().unwrap().push(miss);
    }
    let reward_bytes = serde_json::to_vec(&recipe)?;
    let rewards = if rewards_enabled {
        Some(theia_persistence::RewardCatalog::from_json(
            &reward_bytes,
            &profile,
        )?)
    } else {
        None
    };
    let (publisher, worker) = Worker::start(store);
    let mut gate = if let Some(rewards) = &rewards {
        let items = net::inventory::Profile::from_json(item_bytes)?;
        let wire = Arc::new(if creature_loot {
            theia_network::reward_wire::Profile::from_json_with_creature_loot(
                &reward_bytes,
                &items,
            )?
        } else {
            theia_network::reward_wire::Profile::from_json(&reward_bytes, &items)?
        });
        admissions::GameAdmissions::new_rewards(
            sessions.clone(),
            issuer,
            world,
            rewards.clone(),
            wire,
            publisher.clone(),
        )?
    } else if inventory {
        admissions::GameAdmissions::new_inventory(
            sessions.clone(),
            issuer,
            world,
            profile.clone(),
            publisher.clone(),
        )?
    } else {
        admissions::GameAdmissions::new(sessions.clone(), issuer, world)
    };
    if document_journal || item_rewards {
        let fixture = document_owner::Fixture {
            gate,
            publisher,
            worker,
            transport,
            context,
            saved: character.saved,
            profile,
            admin,
            driver,
            url,
            realm,
            world,
        };
        return if item_rewards {
            item_reward_owner::run(fixture).await
        } else {
            document_owner::run(fixture).await
        };
    }
    if quest_loot {
        use net::progression::{Input, JournalInput, JournalOperation};
        use theia_core::{
            content::ContentId,
            progression::{KillEvent, KillId},
            quests::journal::{Action, Command, Status},
            rewards::ClaimCommand,
        };
        use theia_network::simulation::rewards::Opportunity;
        let id = |s: &str| ContentId::new(s).unwrap();
        let xp_bytes = include_bytes!("../../../content/gameplay/development-progression.json");
        let xp = theia_persistence::ProgressionCatalog::from_json(xp_bytes)?;
        let data = serde_json::to_vec(
            &serde_json::json!({"schema":3,"id":"catalog","revision":1,"quests":[{
            "id":"quest","version":1,"title":"Pelts","description":"Owned conditional loot fixture","prerequisites":[],
            "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
            "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
            "kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.wolf-pelt","required":3,"consume_on_turn_in":true}]}]}]}),
        )?;
        let q = theia_persistence::QuestJournalCatalog::from_json(&data, &xp)?
            .with_inventory(&profile)?;
        let wi = net::inventory::Profile::from_json(item_bytes)?;
        let wj = theia_network::quest_journal::Profile::from_json(
            &data,
            &theia_network::progression_wire::Profile::from_json(xp_bytes)?,
        )?
        .with_inventory(&wi)?;
        let mut source: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/gameplay/development-random-creature-loot-rewards.json"
        ))?;
        source["schema"] = 4.into();
        source["rewards"].as_array_mut().unwrap().remove(0);
        source["rewards"][0]["quest_required"] = true.into();
        for entry in source["rewards"].as_array_mut().unwrap() {
            entry["chance_basis_points"] = 10_000.into();
        }
        let source = serde_json::to_vec(&source)?;
        let rewards =
            theia_persistence::RewardCatalog::from_json_with_quest_journal(&source, &profile, &q)?;
        let wire = Arc::new(
            theia_network::reward_wire::Profile::from_json_with_quest_journal(&source, &wi, &wj)?,
        );
        admissions::GameAdmissions::with_progression(
            &mut gate,
            xp,
            Arc::new(theia_network::progression_wire::Profile::from_json(
                xp_bytes,
            )?),
            publisher.clone(),
        )?;
        // Refuse before installing the bound journal, without partially adopting rewards.
        assert!(
            admissions::GameAdmissions::with_quest_loot(&mut gate, rewards.clone(), wire.clone())
                .is_err()
        );
        admissions::GameAdmissions::with_quest_journal(&mut gate, q.clone())?;
        let mut changed_source = source.clone();
        changed_source.push(b' ');
        let foreign_wire = Arc::new(
            theia_network::reward_wire::Profile::from_json_with_quest_journal(
                &changed_source,
                &wi,
                &wj,
            )?,
        );
        assert!(
            admissions::GameAdmissions::with_quest_loot(&mut gate, rewards.clone(), foreign_wire,)
                .is_err()
        );
        admissions::GameAdmissions::with_quest_loot(&mut gate, rewards.clone(), wire.clone())?;
        assert!(
            admissions::GameAdmissions::with_quest_loot(&mut gate, rewards.clone(), wire,).is_err()
        );
        let actor = EntityKey {
            epoch: WorldEpoch::new(195).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        let command = |request, action, revision| Command {
            request_id: std::num::NonZeroU128::new(request).unwrap(),
            quest: id("quest"),
            npc: id("theia.dev.guide"),
            action,
            expected_revision: revision,
        };
        let input = |sequence, command| {
            Input::Journal(JournalInput {
                actor,
                catalog: q.hash(),
                sequence,
                operation: JournalOperation::Command(command),
            })
        };
        let ticket = gate
            .reserve(context.clone(), binding)?
            .issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        lease.record(actor, position, 1)?;
        let mut items = lease.inventory()?.unwrap();
        let mut progress = lease.progression()?.unwrap();
        inventory_until(&mut items, |s| s.bag.is_some() && s.rewards.is_some()).await;
        progression_until(&mut progress, |s| {
            s.journal.is_some() && s.progress.is_some()
        })
        .await;
        let fact = |n, reward: &str| Opportunity {
            actor,
            reward: id(reward),
            position,
            event: KillEvent {
                id: KillId::from_victim(EntityKey {
                    entity: EntityId::new(n).unwrap(),
                    ..actor
                }),
                creature: id("theia.dev.training-creature"),
            },
        };
        let pelt = "theia.dev.wolf-pelt-loot";
        let token = "theia.dev.wolf-token-loot";
        items
            .opportunities
            .as_ref()
            .unwrap()
            .send(fact(100, pelt))
            .await?;
        items
            .opportunities
            .as_ref()
            .unwrap()
            .send(fact(100, token))
            .await?;
        let ordinary = inventory_until(&mut items, |s| {
            s.rewards.as_ref().is_some_and(|r| r.available.len() == 1)
        })
        .await;
        assert_eq!(ordinary.rewards.unwrap().available[0].reward, id(token));
        progress
            .input
            .send(input(1, command(1, Action::Accept, 0)))
            .await?;
        progression_until(&mut progress, |s| {
            s.journal
                .as_ref()
                .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 1))
        })
        .await;
        // Replay after acceptance cannot create rights on the old denied corpse.
        let before = worker.status().reward_offers;
        items
            .opportunities
            .as_ref()
            .unwrap()
            .send(fact(100, pelt))
            .await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().reward_offers == before {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        assert!(
            publisher
                .list_quest_reward_claims_nearby(binding, world, &rewards, position)
                .await?
                .iter()
                .all(|c| c.reward == id(token))
        );
        for n in 101..106 {
            items
                .opportunities
                .as_ref()
                .unwrap()
                .send(fact(n, pelt))
                .await?;
        }
        let offered = inventory_until(&mut items, |s| {
            s.rewards.as_ref().is_some_and(|r| r.available.len() == 6)
        })
        .await;
        let pelts: Vec<_> = offered
            .rewards
            .unwrap()
            .available
            .into_iter()
            .filter(|r| r.reward == id(pelt))
            .collect();
        assert_eq!(pelts.len(), 5);
        for (index, claim) in pelts.iter().take(3).enumerate() {
            let revision = items.state.borrow().bag.as_ref().unwrap().revision();
            items
                .requests
                .send(net::inventory::Request {
                    sequence: index as u64 + 1,
                    command: ClaimCommand {
                        claim: claim.claim,
                        expected_revision: revision,
                    }
                    .into(),
                })
                .await?;
            inventory_until(&mut items, |s| {
                s.reply.is_some_and(|r| r.sequence == index as u64 + 1)
            })
            .await;
        }
        progression_until(&mut progress, |s| {
            s.journal.as_ref().is_some_and(|j| {
                j.journal
                    .progress(&id("quest"))
                    .is_some_and(|q| q.status() == Status::Ready)
            })
        })
        .await;
        inventory_until(&mut items, |s| {
            s.rewards
                .as_ref()
                .is_some_and(|r| r.available.len() == 1 && r.available[0].reward == id(token))
        })
        .await;
        let ready_bag = items.state.borrow().bag.as_ref().unwrap().clone();
        // Fresh stale-pane pickup is a normal rejection, not owner failure.
        items
            .requests
            .send(net::inventory::Request {
                sequence: 4,
                command: ClaimCommand {
                    claim: pelts[3].claim,
                    expected_revision: ready_bag.revision(),
                }
                .into(),
            })
            .await?;
        let rejected =
            inventory_until(&mut items, |s| s.reply.is_some_and(|r| r.sequence == 4)).await;
        assert_eq!(
            rejected.reply.unwrap().outcome,
            theia_network::inventory_wire::Outcome::Rejected(
                theia_network::inventory_wire::Reason::Conflict
            )
        );
        let slot = ready_bag
            .slots()
            .iter()
            .position(|s| {
                s.as_ref()
                    .is_some_and(|s| s.item() == &id("theia.dev.wolf-pelt"))
            })
            .unwrap() as u8;
        items
            .requests
            .send(net::inventory::Request {
                sequence: 5,
                command: ItemCommand {
                    expected_revision: ready_bag.revision(),
                    operation: ItemOperation::Destroy {
                        from: slot,
                        amount: 1,
                    },
                }
                .into(),
            })
            .await?;
        inventory_until(&mut items, |s| s.reply.is_some_and(|r| r.sequence == 5)).await;
        progression_until(&mut progress, |s| {
            s.journal.as_ref().is_some_and(|j| {
                j.journal
                    .progress(&id("quest"))
                    .is_some_and(|q| q.status() == Status::Active)
            })
        })
        .await;
        // No lease.record after the first pose: this must refresh while stationary.
        inventory_until(&mut items, |s| {
            s.rewards
                .as_ref()
                .is_some_and(|r| r.available.iter().any(|r| r.claim == pelts[3].claim))
        })
        .await;
        let revision = items.state.borrow().bag.as_ref().unwrap().revision();
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        items
            .requests
            .send(net::inventory::Request {
                sequence: 6,
                command: ClaimCommand {
                    claim: pelts[3].claim,
                    expected_revision: revision,
                }
                .into(),
            })
            .await?;
        wait_inventory_lock(&guard).await;
        drop(lease);
        assert!(gate.reserve(context.clone(), binding).is_err());
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let saved = publisher.select_character(binding, world).await?.saved;
        let reservation = gate.reserve(context.clone(), binding)?;
        assert_eq!(
            reservation.check_gameplay_profiles(&publisher).await?,
            theia_persistence::ProfileCompatibility::Compatible
        );
        let ticket = reservation.issue(saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let actor2 = EntityKey {
            entity: EntityId::new(2).unwrap(),
            ..actor
        };
        lease.record(actor2, position, 2)?;
        let mut items = lease.inventory()?.unwrap();
        let mut progress = lease.progression()?.unwrap();
        let loaded = inventory_until(&mut items, |s| s.bag.is_some() && s.rewards.is_some()).await;
        assert_eq!(loaded.bag.as_ref().unwrap().revision(), revision + 1);
        assert_eq!(loaded.rewards.unwrap().available.len(), 1);
        let restored = progression_until(&mut progress, |s| {
            s.journal.as_ref().is_some_and(|j| {
                j.journal
                    .progress(&id("quest"))
                    .is_some_and(|q| q.status() == Status::Ready)
            })
        })
        .await;
        // Latest committed retry succeeds despite reaching the required quantity.
        items
            .requests
            .send(net::inventory::Request {
                sequence: 1,
                command: ClaimCommand {
                    claim: pelts[3].claim,
                    expected_revision: revision,
                }
                .into(),
            })
            .await?;
        let replay =
            inventory_until(&mut items, |s| s.reply.is_some_and(|r| r.sequence == 1)).await;
        assert_eq!(
            replay.reply.unwrap().outcome,
            theia_network::inventory_wire::Outcome::Replayed
        );
        let turn = command(
            2,
            Action::TurnIn,
            restored
                .journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&id("quest"))
                .unwrap()
                .revision(),
        );
        progress
            .input
            .send(Input::Journal(JournalInput {
                actor: actor2,
                catalog: q.hash(),
                sequence: 1,
                operation: JournalOperation::Command(turn),
            }))
            .await?;
        let completed = progression_until(&mut progress, |s| {
            s.journal.as_ref().is_some_and(|j| {
                j.journal
                    .progress(&id("quest"))
                    .is_some_and(|q| q.status() == Status::Completed)
            })
        })
        .await;
        assert_eq!(completed.progress.unwrap().total(), 25);
        let bag = inventory_until(&mut items, |s| {
            s.bag.as_ref().is_some_and(|b| b.revision() == revision + 2)
        })
        .await
        .bag
        .unwrap();
        assert!(
            bag.slots()
                .iter()
                .flatten()
                .all(|s| s.item() != &id("theia.dev.wolf-pelt"))
        );
        inventory_until(&mut items, |s| {
            s.rewards
                .as_ref()
                .is_some_and(|r| r.available.len() == 1 && r.available[0].reward == id(token))
        })
        .await;
        // Leave one entitled pelt uncollected: completion must keep it hidden.
        assert_eq!(
            publisher
                .list_quest_reward_claims_nearby(binding, world, &rewards, position)
                .await?
                .len(),
            1
        );
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        items
            .opportunities
            .as_ref()
            .unwrap()
            .send(fact(200, pelt))
            .await?;
        wait_inventory_lock(&guard).await;
        items
            .opportunities
            .as_ref()
            .unwrap()
            .send(fact(201, pelt))
            .await?;
        drop(lease);
        assert!(gate.reserve(context.clone(), binding).is_err());
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let row=admin.query_one("SELECT count(*),count(*) FILTER (WHERE NOT generation_eligible) FROM theia.loot_rolls WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&binding.character.value()]).await?;
        assert_eq!(row.get::<_, i64>(0), 9);
        assert_eq!(row.get::<_, i64>(1), 3);
        assert_eq!(
            publisher
                .list_quest_reward_claims_nearby(binding, world, &rewards, position)
                .await?
                .len(),
            1
        );
        let status = worker.shutdown().await?;
        assert!(!status.failed);
        drop(admin);
        driver.await??;
        emit(
            serde_json::json!({"passed":true,"quest_loot_enabled":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"denied_generation_stays_denied":true,"stationary_need_refresh":true,"stale_need_nonfatal":true,"pickup_survives_retirement":true,"latest_pickup_replay":true,"atomic_turn_in":true,"completed_leftovers_hidden":true,"queued_generation_survives_retirement":true,"direct_lease_contract_not_native_game":true}),
        );
        return Ok(());
    }
    if item_journal {
        use net::progression::{Input, JournalInput, JournalOperation};
        use theia_core::{
            content::ContentId,
            quests::journal::{Action, Command, Status},
        };
        let id = |s: &str| ContentId::new(s).unwrap();
        let xp_bytes = include_bytes!("../../../content/gameplay/development-progression.json");
        let xp = theia_persistence::ProgressionCatalog::from_json(xp_bytes)?;
        let data = serde_json::to_vec(
            &serde_json::json!({"schema":3,"id":"catalog","revision":1,"quests":[{
            "id":"quest","version":1,"title":"Bring","description":"Owned owner fixture","prerequisites":[],
            "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
            "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
                "kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.training-stone","required":28,"consume_on_turn_in":true}]}]}]}),
        )?;
        let q = theia_persistence::QuestJournalCatalog::from_json(&data, &xp)?
            .with_inventory(&profile)?;
        admissions::GameAdmissions::with_progression(
            &mut gate,
            xp,
            Arc::new(theia_network::progression_wire::Profile::from_json(
                xp_bytes,
            )?),
            publisher.clone(),
        )?;
        admissions::GameAdmissions::with_quest_journal(&mut gate, q.clone())?;
        let vendor_bytes = include_bytes!("../../../content/gameplay/development-vendor.json");
        admissions::GameAdmissions::with_vendor(
            &mut gate,
            theia_persistence::VendorCatalog::from_json(vendor_bytes, &profile)?,
            Arc::new(theia_network::vendor_wire::Profile::from_json(
                vendor_bytes,
                &net::inventory::Profile::from_json(item_bytes)?,
            )?),
        )?;
        let actor = EntityKey {
            epoch: WorldEpoch::new(174).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        let command = |request_id, action, expected_revision| Command {
            request_id: std::num::NonZeroU128::new(request_id).unwrap(),
            quest: id("quest"),
            npc: id("theia.dev.guide"),
            action,
            expected_revision,
        };
        let input = |sequence, cmd| {
            Input::Journal(JournalInput {
                actor,
                catalog: q.hash(),
                sequence,
                operation: JournalOperation::Command(cmd),
            })
        };
        let reservation = gate.reserve(context.clone(), binding)?;
        let ticket = reservation.issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        lease.record(actor, position, 1)?;
        let mut inventory = lease.inventory()?.unwrap();
        let mut progression = lease.progression()?.unwrap();
        inventory_until(&mut inventory, |s| s.bag.is_some()).await;
        progression_until(&mut progression, |s| {
            s.journal.is_some() && s.progress.is_some()
        })
        .await;
        progression
            .input
            .try_send(input(1, command(1, Action::Accept, 0)))?;
        let ready = progression_until(&mut progression, |s| {
            s.journal
                .as_ref()
                .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 1))
        })
        .await;
        assert_eq!(
            ready
                .journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&id("quest"))
                .unwrap()
                .status(),
            Status::Ready
        );
        inventory.requests.try_send(net::inventory::Request {
            sequence: 1,
            command: ItemCommand {
                expected_revision: 0,
                operation: ItemOperation::Split {
                    from: 0,
                    to: 15,
                    amount: 1,
                },
            }
            .into(),
        })?;
        inventory_until(&mut inventory, |s| s.reply.is_some_and(|r| r.sequence == 1)).await;
        assert_eq!(
            progression.state.borrow().journal.as_ref().unwrap().journal,
            ready.journal.as_ref().unwrap().journal
        );
        inventory.requests.try_send(net::inventory::Request {
            sequence: 2,
            command: ItemCommand {
                expected_revision: 1,
                operation: ItemOperation::Destroy {
                    from: 15,
                    amount: 1,
                },
            }
            .into(),
        })?;
        inventory_until(&mut inventory, |s| s.reply.is_some_and(|r| r.sequence == 2)).await;
        let active = progression_until(&mut progression, |s| {
            s.journal.as_ref().is_some_and(|j| {
                j.journal
                    .progress(&id("quest"))
                    .is_some_and(|q| q.status() == Status::Active)
            })
        })
        .await;
        assert_eq!(
            active
                .journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&id("quest"))
                .unwrap()
                .count(&id("goal")),
            Some(27)
        );
        progression.input.try_send(input(
            2,
            command(
                2,
                Action::TurnIn,
                ready
                    .journal
                    .as_ref()
                    .unwrap()
                    .journal
                    .progress(&id("quest"))
                    .unwrap()
                    .revision(),
            ),
        ))?;
        let rejected = progression_until(&mut progression, |s| {
            s.journal
                .as_ref()
                .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 2))
        })
        .await;
        assert!(
            rejected
                .journal
                .as_ref()
                .unwrap()
                .reply
                .as_ref()
                .unwrap()
                .rejection
                .is_some()
        );
        assert_eq!(rejected.progress.as_ref().unwrap().total(), 0);
        let guide = EntityKey {
            epoch: actor.epoch,
            entity: EntityId::new(90).unwrap(),
        };
        inventory.requests.try_send(net::inventory::Request {
            sequence: 3,
            command: theia_core::inventory::ItemCommand::Vendor(theia_core::vendors::Command {
                expected_revision: 2,
                dialogue_revision: 1,
                npc: guide,
                offer: 0,
            }),
        })?;
        inventory_until(&mut inventory, |s| s.reply.is_some_and(|r| r.sequence == 3)).await;
        let restored = progression_until(&mut progression, |s| {
            s.journal.as_ref().is_some_and(|j| {
                j.journal
                    .progress(&id("quest"))
                    .is_some_and(|q| q.status() == Status::Ready)
            })
        })
        .await;
        let turn = command(
            3,
            Action::TurnIn,
            restored
                .journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&id("quest"))
                .unwrap()
                .revision(),
        );
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.quest_journal_heads WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        progression.input.try_send(input(3, turn.clone()))?;
        tokio::time::timeout(Duration::from_millis(300),async {
            loop {
                guard.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
                let waiting:bool=guard.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE wait_event_type='Lock' AND query LIKE '%quest_journal_heads%' AND pid<>pg_backend_pid())",&[]).await.unwrap().get(0);
                if waiting {break;}
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        }).await?;
        drop(lease);
        assert_eq!(gate.flush(&publisher, &worker.status())?, 1);
        assert!(gate.reserve(context.clone(), binding).is_err());
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let done = progression.state.borrow().clone();
        assert!(!done.failed && done.closed);
        assert_eq!(
            done.journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&id("quest"))
                .unwrap()
                .status(),
            Status::Completed
        );
        assert_eq!(done.progress.as_ref().unwrap().total(), 25);
        assert_eq!(inventory.state.borrow().bag.as_ref().unwrap().revision(), 4);
        let selected = publisher.select_character(binding, world).await?.saved;
        let reservation = gate.reserve(context.clone(), binding)?;
        assert_eq!(
            reservation.check_gameplay_profiles(&publisher).await?,
            theia_persistence::ProfileCompatibility::Compatible
        );
        let ticket = reservation.issue(selected)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let actor2 = EntityKey {
            epoch: actor.epoch,
            entity: EntityId::new(2).unwrap(),
        };
        lease.record(actor2, position, 2)?;
        let mut reopened = lease.progression()?.unwrap();
        let mut reopened_items = lease.inventory()?.unwrap();
        let loaded = progression_until(&mut reopened, |s| {
            s.journal.is_some() && s.progress.is_some()
        })
        .await;
        assert_eq!(
            loaded.journal.as_ref().unwrap().journal,
            done.journal.as_ref().unwrap().journal
        );
        reopened.input.try_send(Input::Journal(JournalInput {
            actor: actor2,
            catalog: q.hash(),
            sequence: 1,
            operation: JournalOperation::Command(turn),
        }))?;
        let replay = progression_until(&mut reopened, |s| {
            s.journal
                .as_ref()
                .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 1))
        })
        .await;
        assert_eq!(replay.progress.as_ref().unwrap().total(), 25);
        let bag = inventory_until(&mut reopened_items, |s| {
            s.bag.as_ref().is_some_and(|b| b.revision() == 4)
        })
        .await;
        assert_eq!(bag.bag.as_ref(), inventory.state.borrow().bag.as_ref());
        drop(lease);
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let status = worker.shutdown().await?;
        assert!(!status.failed);
        drop(admin);
        driver.await??;
        emit(
            serde_json::json!({"passed":true,"item_journal_enabled":true,"current_possession_reversal":true,"atomic_turn_in_after_retirement":true,"rejoin_replay_does_not_consume_again":true,"xp":25,"npc_entitlement":"trusted owner fixture; not World or native range proof"}),
        );
        return Ok(());
    }
    if std::env::var("THEIA_TEST_QUEST_JOURNAL").is_ok_and(|v| v == "1") {
        use net::progression::{Input, JournalInput, JournalOperation};
        use std::num::NonZeroU128;
        use theia_core::{
            content::ContentId,
            quests::{
                catalog::EventKey,
                journal::{Action, Command, Status},
            },
        };
        let xp_bytes = include_bytes!("../../../content/gameplay/development-progression.json");
        let xp = theia_persistence::ProgressionCatalog::from_json(xp_bytes)?;
        let q = theia_persistence::QuestJournalCatalog::from_json(
            include_bytes!("../../../content/quests/development-journal.json"),
            &xp,
        )?;
        admissions::GameAdmissions::with_progression(
            &mut gate,
            xp.clone(),
            Arc::new(theia_network::progression_wire::Profile::from_json(
                xp_bytes,
            )?),
            publisher.clone(),
        )?;
        let network_xp = theia_network::progression_wire::Profile::from_json(xp_bytes)?;
        assert_eq!(
            q.hash(),
            theia_network::quest_journal::Profile::from_json(
                include_bytes!("../../../content/quests/development-journal.json"),
                &network_xp
            )?
            .hash()
        );
        admissions::GameAdmissions::with_quest_journal(&mut gate, q.clone())?;
        let id = |s: &str| ContentId::new(s).unwrap();
        let first = id("theia.dev.quest-introduction");
        let second = id("theia.dev.quest-follow-up");
        let cmd = |n, quest, action, rev| Command {
            request_id: NonZeroU128::new(n).unwrap(),
            quest,
            npc: id("theia.dev.guide"),
            action,
            expected_revision: rev,
        };
        let actor = EntityKey {
            epoch: WorldEpoch::new(174).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        let input = |sequence, operation| {
            Input::Journal(JournalInput {
                actor,
                catalog: q.hash(),
                sequence,
                operation,
            })
        };
        let event = |n, name| JournalOperation::Event {
            id: NonZeroU128::new(n).unwrap(),
            event: EventKey::NpcInteraction {
                npc: id("theia.dev.guide"),
                interaction: id(&format!("theia.dev.interaction.{name}")),
            },
        };
        let reservation = gate.reserve(context.clone(), binding)?;
        assert_eq!(
            reservation.check_gameplay_profiles(&publisher).await?,
            theia_persistence::ProfileCompatibility::Compatible
        );
        let ticket = reservation.issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        lease.record(actor, position, 1)?;
        let mut port = lease.progression()?.unwrap();
        progression_until(&mut port, |s| s.journal.is_some()).await;
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.quest_journal_heads WHERE realm_id=$1 AND character_id=$2 FOR UPDATE", &[&realm.value(), &binding.character.value()]).await?;
        port.input.try_send(input(
            1,
            JournalOperation::Command(cmd(1, first.clone(), Action::Accept, 0)),
        ))?;
        tokio::time::timeout(Duration::from_secs(2), async { loop {
            guard.query_one("SELECT pg_stat_clear_snapshot()", &[]).await.unwrap();
            let waiting: bool = guard.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE wait_event_type='Lock' AND query LIKE '%quest_journal_heads%' AND pid<>pg_backend_pid())", &[]).await.unwrap().get(0);
            if waiting {break;} tokio::time::sleep(Duration::from_millis(2)).await;
        }}).await?;
        for (sequence, n, name) in [
            (2, 1, "road"),
            (3, 2, "supplies"),
            (4, 3, "report"),
            (5, 1, "road"),
        ] {
            port.input.try_send(input(sequence, event(n, name)))?;
        }
        port.input.try_send(input(
            6,
            JournalOperation::Command(cmd(2, first.clone(), Action::TurnIn, 4)),
        ))?;
        port.input.try_send(input(
            7,
            JournalOperation::Command(cmd(3, second.clone(), Action::Accept, 0)),
        ))?;
        // A later event must not overwrite the pending command's acknowledgement.
        port.input.try_send(input(8, event(4, "training")))?;
        let kill = |n, creature: &str| {
            JournalOperation::Kill(theia_core::progression::KillEvent {
                id: theia_core::progression::KillId::from_victim(EntityKey {
                    epoch: actor.epoch,
                    entity: EntityId::new(n).unwrap(),
                }),
                creature: id(creature),
            })
        };
        for (sequence, victim) in [(9, 100), (10, 101), (11, 102), (12, 100)] {
            port.input
                .try_send(input(sequence, kill(victim, "theia.dev.training-creature")))?;
        }
        // Quest-only facts must also survive retirement without an XP recipe.
        port.input
            .try_send(input(13, kill(103, "theia.dev.no-xp-creature")))?;
        sessions.lock().unwrap().revoke(secret.expose_secret());
        drop(lease);
        assert_eq!(gate.flush(&publisher, &worker.status())?, 1);
        assert_eq!(port.state.borrow().journal.as_ref().unwrap().revision, 0);
        assert_eq!(port.state.borrow().progress.as_ref().unwrap().total(), 0);
        let secret2 = sessions.lock().unwrap().issue(&identity, owner)?;
        let context2 = sessions
            .lock()
            .unwrap()
            .authorize(secret2.expose_secret())?;
        assert!(
            gate.reserve(context2.clone(), binding).is_err(),
            "Reconnection overtook accepted journal work"
        );
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if gate.flush(&publisher, &worker.status()).unwrap() == 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let saved = port.state.borrow().clone();
        assert!(saved.closed && !saved.failed);
        let journal = saved.journal.as_ref().unwrap();
        assert_eq!(journal.revision, 11);
        assert_eq!(
            journal.journal.progress(&first).unwrap().status(),
            Status::Completed
        );
        assert_eq!(
            journal.journal.progress(&second).unwrap().status(),
            Status::Ready
        );
        assert_eq!(saved.progress.as_ref().unwrap().total(), 100);
        assert_eq!(journal.reply.as_ref().unwrap().sequence, 7);
        assert!(journal.reply.as_ref().unwrap().rejection.is_none());
        assert!(port.input.try_send(input(9, event(5, "road"))).is_err());
        let selected = publisher.select_character(binding, world).await?.saved;
        let reservation = gate.reserve(context2.clone(), binding)?;
        assert_eq!(
            reservation.check_gameplay_profiles(&publisher).await?,
            theia_persistence::ProfileCompatibility::Compatible
        );
        let ticket = reservation.issue(selected)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let actor2 = EntityKey {
            epoch: actor.epoch,
            entity: EntityId::new(2).unwrap(),
        };
        lease.record(actor2, position, 2)?;
        let mut reloaded = lease.progression()?.unwrap();
        let loaded = progression_until(&mut reloaded, |s| s.journal.is_some()).await;
        assert_eq!(loaded.journal.as_ref().unwrap().journal, journal.journal);
        assert_eq!(loaded.progress.as_ref().unwrap().total(), 100);
        reloaded.input.try_send(Input::Journal(JournalInput {
            actor: actor2,
            catalog: q.hash(),
            sequence: 1,
            operation: JournalOperation::Command(cmd(2, first.clone(), Action::TurnIn, 4)),
        }))?;
        progression_until(&mut reloaded, |s| {
            s.journal
                .as_ref()
                .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 1))
        })
        .await;
        assert_eq!(
            reloaded.state.borrow().progress.as_ref().unwrap().total(),
            100
        );
        reloaded.input.try_send(Input::Journal(JournalInput {
            actor: actor2,
            catalog: q.hash(),
            sequence: 2,
            operation: kill(100, "theia.dev.training-creature"),
        }))?;
        sessions.lock().unwrap().revoke(secret2.expose_secret());
        drop(lease);
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if gate.flush(&publisher, &worker.status()).unwrap() == 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let status = worker.shutdown().await?;
        assert_eq!(
            reloaded.state.borrow().progress.as_ref().unwrap().total(),
            100
        );
        assert_eq!(
            reloaded.state.borrow().journal.as_ref().unwrap().journal,
            journal.journal
        );
        assert_eq!(status.journal_experience_units, 100);
        assert_eq!(status.journal_replayed, 4);
        assert!(!status.failed);
        emit(
            serde_json::json!({"passed":true, "expanded_journal_enabled":true,
            "accepted_executing_and_queued_journal_work_survives_logout":true,
            "reconnect_blocked_until_retirement":true, "completion_xp_awarded":25, "kill_xp_awarded":75, "optional_xp_kills_retained":true,
            "replayed_event_and_completion_award_no_extra":true,
            "scope":"Actual OIDC/encrypted ticket/SQL retained owner; NPC and semantic events are trusted synthetic inputs, no live World or new client UI"}),
        );
        drop(transport);
        drop(gate);
        drop(publisher);
        drop(admin);
        driver.await??;
        return Ok(());
    }
    let vendor_bytes = include_bytes!("../../../content/gameplay/development-vendor.json");
    let vendor_profile = if vendors {
        Some(theia_persistence::VendorCatalog::from_json(
            vendor_bytes,
            &profile,
        )?)
    } else {
        None
    };
    if vendors {
        let items = net::inventory::Profile::from_json(item_bytes)?;
        admissions::GameAdmissions::with_vendor(
            &mut gate,
            vendor_profile.clone().unwrap(),
            Arc::new(theia_network::vendor_wire::Profile::from_json(
                vendor_bytes,
                &items,
            )?),
        )?;
    }
    if std::env::var("THEIA_TEST_QUEST_LIFECYCLE").is_ok_and(|v| v == "1") {
        use theia_core::{
            progression::{KillEvent, KillId},
            quests::lifecycle::{Action, Command, Phase},
        };
        use theia_network::{
            quest_lifecycle_wire::{Rejection, Request},
            renet_runtime::progression::Input,
        };
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/gameplay/development-progression.json"
        ))?;
        let completion_xp = std::env::var("THEIA_TEST_QUEST_TURN_IN_XP").is_ok_and(|v| v == "1");
        if !completion_xp {
            raw["level_thresholds"] = serde_json::json!([0, 25]);
        }
        let bytes = serde_json::to_vec(&raw)?;
        let xp = theia_persistence::ProgressionCatalog::from_json(&bytes)?;
        let xp_wire = Arc::new(theia_network::progression_wire::Profile::from_json(&bytes)?);
        admissions::GameAdmissions::with_progression(
            &mut gate,
            xp.clone(),
            xp_wire.clone(),
            publisher.clone(),
        )?;
        let bytes = include_bytes!("../../../content/gameplay/development-quest.json");
        let (quest, wire) = if completion_xp {
            let recipe =
                include_bytes!("../../../content/gameplay/development-quest-turn-in-xp.json");
            (
                theia_persistence::QuestLifecycleCatalog::from_json_with_turn_in_xp(
                    bytes, recipe, &xp,
                )?,
                Arc::new(
                    theia_network::quest_lifecycle_wire::Profile::from_json_with_turn_in_xp(
                        bytes, recipe, &xp_wire,
                    )?,
                ),
            )
        } else {
            (
                theia_persistence::QuestLifecycleCatalog::from_json(bytes)?,
                Arc::new(theia_network::quest_lifecycle_wire::Profile::from_json(
                    bytes,
                )?),
            )
        };
        admissions::GameAdmissions::with_quest_lifecycle(&mut gate, quest.clone(), wire)?;
        let event = |id| KillEvent {
            id: KillId::from_victim(EntityKey {
                epoch: WorldEpoch::new(123).unwrap(),
                entity: EntityId::new(id).unwrap(),
            }),
            creature: theia_core::content::ContentId::new("theia.dev.training-creature").unwrap(),
        };
        let action = |actor, sequence, expected_revision, action| Request {
            actor,
            sequence,
            command: Command {
                expected_revision,
                action,
            },
            dialogue_revision: 1,
            npc: EntityKey {
                epoch: WorldEpoch::new(123).unwrap(),
                entity: EntityId::new(2).unwrap(),
            },
        };
        let reservation = gate.reserve(context.clone(), binding)?;
        assert_eq!(
            reservation.check_gameplay_profiles(&publisher).await?,
            theia_persistence::ProfileCompatibility::Compatible
        );
        let ticket = reservation.issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut port = lease.progression()?.unwrap();
        progression_until(&mut port, |s| s.lifecycle.is_some()).await;
        let actor = EntityKey {
            epoch: WorldEpoch::new(123).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        lease.record(actor, position, 1)?;
        port.input
            .try_send(Input::QuestAction(action(actor, 1, 0, Action::TurnIn)))?;
        let denied = progression_until(&mut port, |s| {
            s.lifecycle_reply.is_some_and(|r| r.sequence == 1)
        })
        .await;
        assert_eq!(
            denied.lifecycle_reply.unwrap().rejection,
            Some(Rejection::NotAvailable)
        );
        assert_eq!(denied.lifecycle.as_ref().unwrap().revision(), 0);
        assert!(!worker.status().failed);
        port.input.try_send(Input::Kill(event(100)))?;
        port.input.try_send(Input::Kill(event(101)))?;
        tokio::time::timeout(Duration::from_secs(3),async{loop{
            let n:i64=admin.query_one("SELECT count(*) FROM theia.quest_lifecycle_kills WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&binding.character.value()]).await.unwrap().get(0);
            if n==2{break;}tokio::time::sleep(Duration::from_millis(2)).await;
        }}).await?;
        let expected_preaccept = if completion_xp { 50 } else { 25 };
        progression_until(&mut port, |s| {
            s.progress
                .as_ref()
                .is_some_and(|p| p.total() == expected_preaccept)
        })
        .await;
        assert_eq!(
            port.state.borrow().progress.as_ref().unwrap().total(),
            expected_preaccept
        );
        assert_eq!(
            port.state.borrow().lifecycle.as_ref().unwrap().revision(),
            0
        );
        gate.flush(&publisher, &worker.status())?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().completed_generation < publisher.submissions() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let before = worker.status();
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.quest_lifecycles WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        port.input
            .try_send(Input::QuestAction(action(actor, 2, 0, Action::Accept)))?;
        wait_lifecycle_lock(&guard).await;
        for id in 102..=104 {
            port.input.try_send(Input::Kill(event(id)))?;
        }
        sessions.lock().unwrap().revoke(secret.expose_secret());
        drop(lease);
        assert_eq!(gate.flush(&publisher, &before)?, 1);
        assert_eq!(
            port.state.borrow().lifecycle.as_ref().unwrap().revision(),
            0,
            "Accepted but uncommitted acceptance cannot be advertised"
        );
        let secret2 = sessions.lock().unwrap().issue(&identity, owner)?;
        let context2 = sessions
            .lock()
            .unwrap()
            .authorize(secret2.expose_secret())?;
        assert!(
            gate.reserve(context2.clone(), binding).is_err(),
            "Fresh authorization must not bypass retiring accepted quest work"
        );
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let ready = port.state.borrow().lifecycle.clone().unwrap();
        assert_eq!(ready.progress(quest.profile()).unwrap().phase, Phase::Ready);
        assert_eq!(ready.revision(), 4);
        assert_eq!(
            port.state.borrow().progress.as_ref().unwrap().total(),
            if completion_xp { 125 } else { 25 }
        );
        assert!(port.input.try_send(Input::Kill(event(100))).is_err());
        let saved = publisher.select_character(binding, world).await?.saved;
        let ticket = gate.reserve(context2.clone(), binding)?.issue(saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut loaded = lease.progression()?.unwrap();
        progression_until(&mut loaded, |s| s.lifecycle.is_some()).await;
        assert_eq!(loaded.state.borrow().lifecycle.as_ref(), Some(&ready));
        let actor2 = EntityKey {
            entity: EntityId::new(3).unwrap(),
            ..actor
        };
        lease.record(actor2, position, 2)?;
        gate.flush(&publisher, &worker.status())?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().completed_generation < publisher.submissions() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let before = worker.status();
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.quest_lifecycles WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        loaded
            .input
            .try_send(Input::QuestAction(action(actor2, 1, 4, Action::TurnIn)))?;
        wait_lifecycle_lock(&guard).await;
        loaded.input.try_send(Input::Kill(event(100)))?;
        loaded.input.try_send(Input::Kill(event(105)))?;
        sessions.lock().unwrap().revoke(secret2.expose_secret());
        drop(lease);
        assert_eq!(gate.flush(&publisher, &before)?, 1);
        assert_eq!(
            loaded.state.borrow().lifecycle.as_ref(),
            Some(&ready),
            "Uncommitted turn-in cannot be advertised"
        );
        assert_eq!(
            loaded.state.borrow().progress.as_ref().unwrap().total(),
            if completion_xp { 125 } else { 25 },
            "Uncommitted completion XP cannot be advertised"
        );
        let secret3 = sessions.lock().unwrap().issue(&identity, owner)?;
        let context3 = sessions
            .lock()
            .unwrap()
            .authorize(secret3.expose_secret())?;
        assert!(gate.reserve(context3.clone(), binding).is_err());
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let completed = loaded.state.borrow().lifecycle.clone().unwrap();
        let completed_xp = if completion_xp { 175 } else { 25 };
        assert_eq!(
            loaded.state.borrow().progress.as_ref().unwrap().total(),
            completed_xp
        );
        assert_eq!(completed.revision(), 5);
        assert_eq!(
            completed.progress(quest.profile()).unwrap().phase,
            Phase::TurnedIn
        );
        let saved = publisher.select_character(binding, world).await?.saved;
        let ticket = gate.reserve(context3, binding)?.issue(saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut restored = lease.progression()?.unwrap();
        progression_until(&mut restored, |s| s.lifecycle.is_some()).await;
        assert_eq!(restored.state.borrow().lifecycle.as_ref(), Some(&completed));
        let actor3 = EntityKey {
            entity: EntityId::new(5).unwrap(),
            ..actor
        };
        lease.record(actor3, position, 3)?;
        for (sequence, revision, kind) in [(1, 4, Action::TurnIn), (2, 0, Action::Accept)] {
            restored
                .input
                .try_send(Input::QuestAction(action(actor3, sequence, revision, kind)))?;
            let result = progression_until(&mut restored, |s| {
                s.lifecycle_reply.is_some_and(|r| r.sequence == sequence)
            })
            .await;
            assert_eq!(result.lifecycle_reply.unwrap().rejection, None);
            assert_eq!(result.lifecycle.as_ref(), Some(&completed));
            assert_eq!(result.progress.as_ref().unwrap().total(), completed_xp);
        }
        drop(lease);
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let status = worker.shutdown().await?;
        assert_eq!(status.lifecycle_actions, 2);
        assert_eq!(status.lifecycle_turn_ins, 1);
        assert_eq!(status.lifecycle_advances, 3);
        assert_eq!(
            status.experience_committed,
            if completion_xp { 7 } else { 1 }
        );
        if completion_xp {
            assert_eq!(status.lifecycle_xp_awards, 1);
            assert_eq!(status.lifecycle_xp_units, 25);
            let r=admin.query_one("SELECT xp_granted,xp_revision,xp_total FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2 AND action=2",&[&realm.value(),&binding.character.value()]).await?;
            assert_eq!(
                (r.get::<_, i32>(0), r.get::<_, i64>(1), r.get::<_, i32>(2)),
                (25, 6, 150),
                "Completion must commit before the later queued kill"
            );
        }
        let row=admin.query_one("SELECT (SELECT count(*) FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2),(SELECT count(*) FROM theia.quest_lifecycle_kills WHERE realm_id=$1 AND character_id=$2),(SELECT count(*) FROM theia.quest_lifecycle_kills WHERE realm_id=$1 AND character_id=$2 AND counted)",&[&realm.value(),&binding.character.value()]).await?;
        assert_eq!(
            (
                row.get::<_, i64>(0),
                row.get::<_, i64>(1),
                row.get::<_, i64>(2)
            ),
            (2, 6, 3)
        );
        drop(admin);
        driver.await??;
        emit(
            serde_json::json!({"passed":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"explicit_lifecycle_enabled":true,"accepted_executing_and_queued_lifecycle_work_survives_logout":true,"fresh_authorization_cannot_bypass_retiring_owner":true,"no_acceptance_or_turn_in_before_commit":true,"single_fifo_acceptance_then_kills":true,"preaccept_xp_cap_and_old_receipts_preserved":!completion_xp,"preaccept_kills_remain_uncounted":true,"positive_completion_xp_enabled":completion_xp,"completion_xp_awarded":if completion_xp {25}else{0},"normal_quest_refusal_keeps_owner_live":true,"fresh_lease_restores_ready_and_turned_in":true,"old_action_receipts_do_not_retake_or_repeat_turn_in":true,"direct_lease_contract_not_native_game":true,"live_npc_entitlement_qualified":false,"turn_in_item_or_xp_grants_enabled":completion_xp}),
        );
        return Ok(());
    }
    if std::env::var("THEIA_TEST_PROGRESSION").is_ok_and(|v| v == "1") {
        use theia_core::progression::{KillEvent, KillId};
        let bytes = include_bytes!("../../../content/gameplay/development-progression.json");
        let profile = theia_persistence::ProgressionCatalog::from_json(bytes)?;
        admissions::GameAdmissions::with_progression(
            &mut gate,
            profile.clone(),
            Arc::new(theia_network::progression_wire::Profile::from_json(bytes)?),
            publisher.clone(),
        )?;
        let quest_enabled = std::env::var("THEIA_TEST_QUESTS").is_ok_and(|v| v == "1");
        if quest_enabled {
            let bytes = include_bytes!("../../../content/gameplay/development-quest.json");
            admissions::GameAdmissions::with_quest(
                &mut gate,
                theia_persistence::QuestCatalog::from_json(bytes)?,
                Arc::new(theia_network::quest_wire::Profile::from_json(bytes)?),
            )?;
        }
        let ticket = gate
            .reserve(context.clone(), binding)?
            .issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut port = lease.progression()?.unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while port.state.borrow().progress.is_none() {
                port.state.changed().await.unwrap();
            }
        })
        .await?;
        let actor = EntityKey {
            epoch: WorldEpoch::new(123).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        lease.record(actor, position, 1)?;
        gate.flush(&publisher, &worker.status())?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().completed_generation < publisher.submissions() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let before = worker.status();
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        if quest_enabled {
            assert_eq!(port.state.borrow().quest.as_ref().unwrap().count(), 0);
        }
        let first = KillEvent {
            id: KillId::from_victim(EntityKey {
                entity: EntityId::new(100).unwrap(),
                ..actor
            }),
            creature: theia_core::content::ContentId::new("theia.dev.training-creature").unwrap(),
        };
        let second = KillEvent {
            id: KillId::from_victim(EntityKey {
                entity: EntityId::new(101).unwrap(),
                ..actor
            }),
            ..first.clone()
        };
        port.input
            .try_send(net::progression::Input::Kill(first.clone()))?;
        tokio::time::timeout(Duration::from_secs(3),async {loop {
            guard.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
            if guard.query_opt("SELECT pid FROM pg_stat_activity WHERE application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE '%theia.progression%'",&[]).await.unwrap().is_some() {break;}
            tokio::time::sleep(Duration::from_millis(2)).await;
        }}).await?;
        port.input.try_send(net::progression::Input::Kill(second))?;
        port.input
            .try_send(net::progression::Input::Kill(first.clone()))?;
        sessions.lock().unwrap().revoke(secret.expose_secret());
        drop(lease);
        assert_eq!(gate.flush(&publisher, &before)?, 1);
        assert_eq!(
            port.state.borrow().progress.as_ref().unwrap().total(),
            0,
            "Uncommitted XP cannot be advertised"
        );
        if quest_enabled {
            assert_eq!(
                port.state.borrow().quest.as_ref().unwrap().count(),
                0,
                "Uncommitted objective cannot be advertised"
            );
        }
        assert!(gate.reserve(context.clone(), binding).is_err());
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        assert_eq!(port.state.borrow().progress.as_ref().unwrap().total(), 50);
        if quest_enabled {
            assert_eq!(port.state.borrow().quest.as_ref().unwrap().count(), 2);
            assert_eq!(worker.status().quest_advances, 2);
            assert_eq!(worker.status().quest_replayed, 1);
        }
        assert_eq!(worker.status().experience_committed, 2);
        assert_eq!(worker.status().experience_replayed, 1);
        assert!(
            port.input
                .try_send(net::progression::Input::Kill(first.clone()))
                .is_err()
        );
        let secret = sessions.lock().unwrap().issue(&identity, owner)?;
        let context = sessions.lock().unwrap().authorize(secret.expose_secret())?;
        let saved = publisher.select_character(binding, world).await?.saved;
        let ticket = gate.reserve(context, binding)?.issue(saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut loaded = lease.progression()?.unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while loaded.state.borrow().progress.is_none() {
                loaded.state.changed().await.unwrap();
            }
        })
        .await?;
        assert_eq!(loaded.state.borrow().progress.as_ref().unwrap().total(), 50);
        if quest_enabled {
            assert_eq!(loaded.state.borrow().quest.as_ref().unwrap().count(), 2);
        }
        loaded
            .input
            .try_send(net::progression::Input::Kill(first))?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().experience_replayed < 2 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        assert_eq!(loaded.state.borrow().progress.as_ref().unwrap().total(), 50);
        if quest_enabled {
            assert_eq!(loaded.state.borrow().quest.as_ref().unwrap().count(), 2);
        }
        drop(lease);
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        worker.shutdown().await?;
        drop(admin);
        driver.await??;
        emit(
            serde_json::json!({"passed":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"accepted_executing_and_distinct_queued_xp_survive_logout":true,"pose_commit_does_not_release_pending_xp":true,"no_xp_before_commit":true,"duplicate_old_event_no_extra_xp":true,"fresh_lease_restores_progression":true,"closed_generation_rejects_facts":true,"direct_lease_contract_not_native_game":true,"objective_enabled":quest_enabled,"accepted_objective_facts_survive_retirement":quest_enabled}),
        );
        return Ok(());
    }
    if creature_loot {
        use theia_core::{
            progression::{KillEvent, KillId},
            rewards::ClaimCommand,
        };
        use theia_network::simulation::rewards::Opportunity;
        let rewards = rewards.as_ref().unwrap();
        let recipe = rewards.catalog().definitions().next().unwrap();
        let ticket = gate
            .reserve(context.clone(), binding)?
            .issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let actor = EntityKey {
            epoch: WorldEpoch::new(123).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        lease.record(actor, position, 1)?;
        let mut port = lease.inventory()?.unwrap();
        let initial = inventory_until(&mut port, |s| s.bag.is_some() && s.rewards.is_some()).await;
        let baseline = initial.bag.unwrap();
        assert!(initial.rewards.unwrap().available.is_empty());
        let mut xyz = position.metres();
        xyz[0] += 1000.;
        let distant = GlobalPosition::new(xyz).unwrap();
        let fact = |n, position| Opportunity {
            actor,
            reward: recipe.id().clone(),
            position,
            event: KillEvent {
                id: KillId::from_victim(EntityKey {
                    entity: EntityId::new(n).unwrap(),
                    ..actor
                }),
                creature: recipe.creature().clone(),
            },
        };
        let mut miss = fact(99, position);
        miss.reward = theia_core::content::ContentId::new("theia.dev.zero-drop").unwrap();
        for _ in 0..2 {
            port.opportunities
                .as_ref()
                .unwrap()
                .send(miss.clone())
                .await?;
        }
        for n in 100..117 {
            port.opportunities
                .as_ref()
                .unwrap()
                .send(fact(n, distant))
                .await?;
        }
        port.opportunities
            .as_ref()
            .unwrap()
            .send(fact(117, position))
            .await?;
        let offered = inventory_until(&mut port, |s| {
            s.rewards.as_ref().is_some_and(|r| r.available.len() == 1)
        })
        .await;
        assert_eq!(offered.bag.as_ref().unwrap(), &baseline);
        let claim = offered.rewards.unwrap().available[0].clone();
        assert_eq!(claim.corpse.unwrap().position, position);
        lease.record(actor, distant, 2)?;
        let far = inventory_until(&mut port, |s| {
            s.rewards.as_ref().is_some_and(|r| r.available.len() == 16)
        })
        .await;
        assert!(
            far.rewards
                .unwrap()
                .available
                .iter()
                .all(|c| c.corpse.unwrap().position == distant)
        );
        assert_eq!(far.bag.as_ref().unwrap(), &baseline);
        lease.record(actor, position, 3)?;
        let back = inventory_until(&mut port, |s| {
            s.rewards.as_ref().is_some_and(|r| r.available.len() == 1)
        })
        .await;
        assert_eq!(back.rewards.unwrap().available[0].claim, claim.claim);
        // Direct accepted owner command: World range/life checks are separately
        // exercised by encrypted runtime tests, not bypassed in the product.
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        port.requests
            .send(net::inventory::Request {
                sequence: 1,
                command: ClaimCommand {
                    claim: claim.claim,
                    expected_revision: baseline.revision(),
                }
                .into(),
            })
            .await?;
        wait_inventory_lock(&guard).await;
        assert_eq!(port.state.borrow().bag.as_ref().unwrap(), &baseline);
        drop(lease);
        assert!(matches!(
            gate.reserve(context.clone(), binding),
            Err(admissions::Error::Conflict)
        ));
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let saved = publisher.select_character(binding, world).await?.saved;
        assert_eq!(saved.position, position);
        let ticket = gate.reserve(context.clone(), binding)?.issue(saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut fresh = lease.inventory()?.unwrap();
        let loaded = inventory_until(&mut fresh, |s| s.bag.is_some() && s.rewards.is_some()).await;
        assert_eq!(loaded.bag.unwrap().revision(), baseline.revision() + 1);
        assert!(loaded.rewards.unwrap().available.is_empty());
        let next_actor = EntityKey {
            entity: EntityId::new(2).unwrap(),
            ..actor
        };
        lease.record(next_actor, distant, 1)?;
        let recovered = inventory_until(&mut fresh, |s| {
            s.rewards.as_ref().is_some_and(|r| r.available.len() == 16)
        })
        .await;
        assert!(
            recovered
                .rewards
                .unwrap()
                .available
                .iter()
                .all(|c| c.corpse.unwrap().position == distant)
        );
        let all_far = publisher
            .list_reward_claims_nearby(binding, world, rewards, distant)
            .await?;
        assert_eq!(all_far.len(), 16);
        // Replaying an already collected fatal life does not restore loot.
        lease.record(next_actor, position, 2)?;
        inventory_until(&mut fresh, |s| {
            s.rewards.as_ref().is_some_and(|r| r.available.is_empty())
        })
        .await;
        let offered_before = worker.status().reward_offers;
        fresh
            .opportunities
            .as_ref()
            .unwrap()
            .send(fact(117, position))
            .await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().reward_offers == offered_before {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        drop(lease);
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        assert!(
            publisher
                .list_reward_claims_nearby(binding, world, rewards, position)
                .await?
                .is_empty()
        );
        assert_eq!(
            publisher
                .load_inventory(binding, world, &profile)
                .await?
                .revision(),
            baseline.revision() + 1
        );
        let row=admin.query_one("SELECT count(*),count(*) FILTER(WHERE consumed_revision IS NULL) FROM theia.reward_claims WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&binding.character.value()]).await?;
        assert_eq!(row.get::<_, i64>(0), 18);
        assert_eq!(row.get::<_, i64>(1), 17);
        let rolls = admin
            .query_one(
                "SELECT count(*) FROM theia.loot_rolls WHERE realm_id=$1 AND character_id=$2",
                &[&realm.value(), &binding.character.value()],
            )
            .await?;
        assert_eq!(
            rolls.get::<_, i64>(0),
            19,
            "Repeated miss must retain one roll without advertising loot"
        );
        worker.shutdown().await?;
        drop(admin);
        driver.await??;
        emit(
            serde_json::json!({"passed":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"creature_loot_enabled":true,"persisted_rolls_including_miss":true,"nearby_interest_follows_retained_pose":true,"pickup_survives_retirement":true,"distant_uncollected_loot_restores":true,"death_offer_does_not_grant_items":true,"consumed_life_replay_does_not_restore_loot":true,"direct_lease_contract_not_native_game":true}),
        );
        return Ok(());
    }
    if let Some(rewards) = &rewards {
        use theia_core::rewards::ClaimCommand;
        let reward = rewards.catalog().definitions().next().unwrap().id().clone();
        let ticket = gate
            .reserve(context.clone(), binding)?
            .issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut port = lease.inventory()?.unwrap();
        inventory_until(&mut port, |s| s.bag.is_some() && s.rewards.is_some()).await;
        let actor = EntityKey {
            epoch: WorldEpoch::new(123).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        lease.record(actor, position, 1)?;
        gate.flush(&publisher, &worker.status())?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().completed_generation < publisher.submissions() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let before = worker.status();
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        let earned = port.opportunities.as_ref().unwrap().clone();
        let fact = |reward: theia_core::content::ContentId| {
            theia_network::simulation::rewards::Opportunity {
                actor,
                event: theia_core::progression::KillEvent {
                    id: theia_core::progression::KillId::from_victim(EntityKey {
                        entity: EntityId::new(99).unwrap(),
                        ..actor
                    }),
                    creature: rewards
                        .catalog()
                        .definition(&reward)
                        .unwrap()
                        .creature()
                        .clone(),
                },
                reward,
                position,
            }
        };
        earned.try_send(fact(reward.clone()))?;
        wait_inventory_lock(&guard).await;
        // A distinct queued recipe must commit too; a duplicate alone could
        // conceal a broken drain behind the first executing transaction.
        let second = theia_core::content::ContentId::new("theia.dev.second-training-kill").unwrap();
        earned.try_send(fact(second.clone()))?;
        earned.try_send(fact(reward.clone()))?;
        sessions.lock().unwrap().revoke(secret.expose_secret());
        drop(lease);
        assert_eq!(gate.flush(&publisher, &before)?, 1);
        assert_eq!(
            port.state
                .borrow()
                .rewards
                .as_ref()
                .unwrap()
                .available
                .len(),
            0,
            "No availability before COMMIT"
        );
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        assert!(
            earned.try_send(fact(reward.clone())).is_err(),
            "Retired generation cannot accept new facts"
        );
        assert_eq!(
            worker.status().reward_offers,
            2,
            "Repeated accepted kill cannot mint a second claim"
        );
        let secret = sessions.lock().unwrap().issue(&identity, owner)?;
        let context = sessions.lock().unwrap().authorize(secret.expose_secret())?;
        let saved = publisher.select_character(binding, world).await?.saved;
        let ticket = gate.reserve(context.clone(), binding)?.issue(saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let admission = gate.clone().consume(1, &data)?.unwrap();
        let mut port = admission.lease.inventory()?.unwrap();
        let loaded = inventory_until(&mut port, |s| {
            s.bag.is_some() && s.rewards.as_ref().is_some_and(|r| r.available.len() == 2)
        })
        .await;
        assert_eq!(loaded.bag.as_ref().unwrap().revision(), 0);
        let claim = loaded.rewards.unwrap().available[0].claim;
        admission.lease.record(
            EntityKey {
                entity: EntityId::new(2).unwrap(),
                ..actor
            },
            position,
            1,
        )?;
        gate.flush(&publisher, &worker.status())?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().completed_generation < publisher.submissions() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let before = worker.status();
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        port.requests
            .send(net::inventory::Request {
                sequence: 1,
                command: ClaimCommand {
                    claim,
                    expected_revision: 0,
                }
                .into(),
            })
            .await?;
        wait_inventory_lock(&guard).await;
        drop(admission);
        assert_eq!(gate.flush(&publisher, &before)?, 1);
        assert!(matches!(
            gate.reserve(context.clone(), binding),
            Err(admissions::Error::Conflict)
        ));
        assert!(
            port.state.borrow().reply.is_none(),
            "Blocked claim has no success reply"
        );
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let saved = publisher.select_character(binding, world).await?.saved;
        let ticket = gate.reserve(context.clone(), binding)?.issue(saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let admission = gate.clone().consume(1, &data)?.unwrap();
        let mut port = admission.lease.inventory()?.unwrap();
        let loaded = inventory_until(&mut port, |s| s.bag.is_some() && s.rewards.is_some()).await;
        let bag = loaded.bag.unwrap();
        assert_eq!(bag.revision(), 1);
        let remaining = loaded.rewards.unwrap().available;
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].reward, second);
        assert_eq!(
            bag.slots()
                .iter()
                .flatten()
                .filter(|s| s.item().as_str() == "theia.dev.training-token")
                .map(|s| s.count())
                .sum::<u32>(),
            100
        );
        port.requests
            .send(net::inventory::Request {
                sequence: 1,
                command: ClaimCommand {
                    claim,
                    expected_revision: 0,
                }
                .into(),
            })
            .await?;
        let replay = inventory_until(&mut port, |s| s.reply.is_some()).await;
        assert_eq!(
            replay.reply.unwrap().outcome,
            theia_network::inventory_wire::Outcome::Replayed
        );
        assert_eq!(replay.bag.unwrap(), bag);
        drop(admission);
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        worker.shutdown().await?;
        drop(admin);
        driver.await??;
        emit(
            serde_json::json!({"passed":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"accepted_offers_survive_logout_retirement":true,"distinct_queued_reward_drained":true,"duplicate_offer_stays_once":true,"unclaimed_second_recipe_recovers":true,"no_availability_before_commit":true,"consumed_reward_survives_fresh_lease":true,"claim_commit_before_character_reuse":true,"lost_reply_replays_once":true,"closed_generation_rejects_facts":true,"direct_lease_contract_not_native_game":true}),
        );
        return Ok(());
    }
    if inventory {
        let ticket = gate
            .reserve(context.clone(), binding)?
            .issue(character.saved)?;
        let data = connected_data(&mut transport, ticket.bytes()).await;
        let lease = gate.clone().consume(1, &data)?.unwrap().lease;
        let mut port = lease.inventory()?.unwrap();
        inventory_until(&mut port, |s| s.bag.is_some()).await;
        let actor = EntityKey {
            epoch: WorldEpoch::new(123).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        let final_pose = GlobalPosition::new([15401.25, 3.5, 12861.125]).unwrap();
        lease.record(actor, final_pose, 2)?;
        gate.flush(&publisher, &worker.status())?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().completed_generation < 1 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let committed_pose = worker.status();
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        let purchase = |revision| theia_core::vendors::Command {
            expected_revision: revision,
            dialogue_revision: 1,
            npc: EntityKey {
                entity: EntityId::new(99).unwrap(),
                ..actor
            },
            offer: 0,
        };
        let intent: theia_core::inventory::ItemCommand = if vendors {
            purchase(0).into()
        } else {
            ItemCommand {
                expected_revision: 0,
                operation: ItemOperation::Split {
                    from: 0,
                    to: 3,
                    amount: 5,
                },
            }
            .into()
        };
        port.requests
            .send(net::inventory::Request {
                sequence: 1,
                command: intent,
            })
            .await?;
        wait_inventory_lock(&guard).await;
        let queued: theia_core::inventory::ItemCommand = if vendors {
            purchase(1).into()
        } else {
            ItemCommand {
                expected_revision: 1,
                operation: ItemOperation::Move { from: 3, to: 4 },
            }
            .into()
        };
        port.requests.try_send(net::inventory::Request {
            sequence: 2,
            command: queued,
        })?;
        assert!(
            port.requests
                .try_send(net::inventory::Request {
                    sequence: 3,
                    command: queued,
                })
                .is_err(),
            "Accepted item queue must remain bounded to one"
        );
        assert_eq!(port.state.borrow().bag.as_ref().unwrap().revision(), 0);
        drop(lease);
        assert_eq!(
            gate.flush(&publisher, &committed_pose)?,
            1,
            "Committed pose alone must not release an executing item mutation"
        );
        assert!(matches!(
            gate.reserve(context.clone(), binding),
            Err(admissions::Error::Conflict)
        ));
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        assert_eq!(worker.status().inventory_committed, 2);
        assert!(
            port.requests
                .try_send(net::inventory::Request {
                    sequence: 3,
                    command: queued,
                })
                .is_err(),
            "Retired owner must reject new commands"
        );
        let saved = publisher.select_character(binding, world).await?.saved;
        assert_eq!(saved.position, final_pose);
        // A never-joined lease must also retain ownership during initialization.
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        let ticket = gate.reserve(context.clone(), binding)?.issue(saved)?;
        let fresh_data = connected_data(&mut transport, ticket.bytes()).await;
        let admission = gate.clone().consume(1, &fresh_data)?.unwrap();
        wait_inventory_lock(&guard).await;
        let submissions = publisher.submissions();
        drop(admission);
        assert_eq!(gate.flush(&publisher, &worker.status())?, 1);
        assert_eq!(
            publisher.submissions(),
            submissions,
            "Never-joined retirement must not invent a pose checkpoint"
        );
        assert!(matches!(
            gate.reserve(context.clone(), binding),
            Err(admissions::Error::Conflict)
        ));
        guard.commit().await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while gate.flush(&publisher, &worker.status()).unwrap() != 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let ticket = gate.reserve(context.clone(), binding)?.issue(saved)?;
        let fresh_data = connected_data(&mut transport, ticket.bytes()).await;
        let admission = gate.clone().consume(1, &fresh_data)?.unwrap();
        let mut port = admission.lease.inventory()?.unwrap();
        let loaded = inventory_until(&mut port, |s| s.bag.is_some())
            .await
            .bag
            .unwrap();
        assert_eq!(loaded.revision(), 2);
        if vendors {
            assert_eq!(loaded.slots()[0].as_ref().unwrap().count(), 19);
            assert_eq!(loaded.slots()[1].as_ref().unwrap().count(), 15);
            assert_eq!(loaded.slots()[2].as_ref().unwrap().count(), 97);
            assert_eq!(
                loaded.slots()[3].as_ref().unwrap().item().as_str(),
                "theia.dev.training-sword"
            );
            assert_eq!(worker.status().vendor_committed, 2);
        } else {
            assert_eq!(loaded.slots()[0].as_ref().unwrap().count(), 8);
            assert!(loaded.slots()[3].is_none());
            assert_eq!(loaded.slots()[4].as_ref().unwrap().count(), 5);
        }
        assert!(gate.clone().consume(1, &data)?.is_none());
        port.requests
            .send(net::inventory::Request {
                sequence: 1,
                command: queued,
            })
            .await?;
        let replay = inventory_until(&mut port, |s| s.reply.is_some_and(|r| r.sequence == 1)).await;
        assert_eq!(
            replay.reply.unwrap().outcome,
            theia_network::inventory_wire::Outcome::Replayed
        );
        assert_eq!(replay.bag.unwrap(), loaded);
        let actor = EntityKey {
            entity: EntityId::new(2).unwrap(),
            ..actor
        };
        admission.lease.record(actor, final_pose, 1)?;
        gate.flush(&publisher, &worker.status())?;
        let generation = publisher.submissions();
        tokio::time::timeout(Duration::from_secs(5), async {
            while worker.status().completed_generation < generation {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await?;
        let guard = admin.transaction().await?;
        guard.query_one("SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await?;
        port.requests
            .send(net::inventory::Request {
                sequence: 2,
                command: if vendors {
                    purchase(2).into()
                } else {
                    ItemCommand {
                        expected_revision: 2,
                        operation: ItemOperation::Move { from: 4, to: 5 },
                    }
                    .into()
                },
            })
            .await?;
        let backend = wait_inventory_lock(&guard).await;
        assert!(
            guard
                .query_one("SELECT pg_terminate_backend($1)", &[&backend])
                .await?
                .get::<_, bool>(0)
        );
        inventory_until(&mut port, |s| s.failed).await;
        drop(admission);
        assert!(matches!(
            gate.reserve(context, binding),
            Err(admissions::Error::Unavailable)
        ));
        assert!(gate.retained_positions().is_err());
        guard.rollback().await?;
        assert!(worker.shutdown().await.is_err());
        let mut store = Store::connect_loopback(&url, realm).await?;
        assert_eq!(
            store.load_inventory(binding, world, &profile).await?,
            loaded
        );
        let latest = if let Some(vendor_profile) = &vendor_profile {
            store
                .purchase_vendor(
                    binding,
                    world,
                    vendor_profile,
                    &theia_core::vendors::Purchase {
                        expected_revision: 1,
                        offer: vendor_profile
                            .catalog()
                            .offers()
                            .next()
                            .unwrap()
                            .id()
                            .clone(),
                    },
                )
                .await?
        } else {
            let theia_core::inventory::ItemCommand::Backpack(queued) = queued else {
                unreachable!()
            };
            store
                .mutate_inventory(binding, world, &profile, queued)
                .await?
        };
        assert!(latest.replayed);
        store.close().await?;
        drop(admin);
        driver.await??;
        emit(
            serde_json::json!({"passed":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"pose_commit_does_not_release_pending_item":true,"never_joined_initialization_retained":true,"item_commit_before_character_reuse":true,"accepted_executing_and_queued_items_survive_logout":true,"bounded_queue_and_retired_rejection":true,"no_item_state_before_commit":true,"fresh_lease_load_and_latest_retry":true,"backend_loss_denies_all_admission":true,"direct_lease_contract_not_native_game":true,"vendor_enabled":vendors,"accepted_executing_and_queued_purchases_survive_logout":vendors}),
        );
        return Ok(());
    }
    let cancelled = gate.reserve(context.clone(), binding)?;
    drop(cancelled);
    assert!(matches!(
        gate.reserve(
            context.clone(),
            Binding {
                owner: theia_persistence::OwnerId::random(),
                ..binding
            }
        ),
        Err(admissions::Error::Unauthorized)
    ));
    let reservation = gate.reserve(context.clone(), binding)?;
    assert!(matches!(
        gate.reserve(context.clone(), binding),
        Err(admissions::Error::Conflict)
    ));
    let invalid = theia_persistence::SavedCharacter {
        world: [104; 32],
        ..character.saved
    };
    assert!(reservation.issue(invalid).is_err());
    let ticket = gate
        .reserve(context.clone(), binding)?
        .issue(character.saved)?;
    let data = connected_data(&mut transport, ticket.bytes()).await;
    let mut tampered = data;
    tampered[16] ^= 1;
    assert!(gate.clone().consume(1, &tampered)?.is_none());
    assert!(gate.clone().consume(2, &data)?.is_none());
    let lease = gate.clone().consume(1, &data)?.unwrap().lease;
    assert!(
        gate.clone().consume(1, &data)?.is_none(),
        "A valid, unexpired nonce must be one-use"
    );
    assert!(lease.live()?);
    let actor = EntityKey {
        epoch: WorldEpoch::new(123).unwrap(),
        entity: EntityId::new(1).unwrap(),
    };
    lease.record(actor, position, 1)?;
    assert_eq!(gate.flush(&publisher, &worker.status())?, 1);
    tokio::time::timeout(Duration::from_secs(5), async {
        while worker.status().completed_generation < 1 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    let before = worker.status();
    let final_pose = GlobalPosition::new([15401.25, 3.5, 12861.125]).unwrap();
    lease.record(actor, final_pose, 2)?;
    drop(lease);
    assert_eq!(gate.flush(&publisher, &before)?, 1);
    assert!(
        matches!(
            gate.reserve(context.clone(), binding),
            Err(admissions::Error::Conflict)
        ),
        "Queued final state must not free a reservation"
    );
    // An older acknowledgement must still not release the changed final pose.
    assert_eq!(gate.flush(&publisher, &before)?, 1);
    assert!(gate.reserve(context.clone(), binding).is_err());
    tokio::time::timeout(Duration::from_secs(5), async {
        while worker.status().completed_generation <= before.completed_generation {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    assert_eq!(gate.flush(&publisher, &worker.status())?, 0);
    let restored = publisher.select_character(binding, world).await?;
    assert_eq!(restored.saved.position, final_pose);
    let fresh = gate
        .reserve(context.clone(), binding)?
        .issue(restored.saved)?;
    assert!(
        gate.clone().consume(1, &data)?.is_none(),
        "Old generation cannot consume replacement reservation"
    );
    let fresh_data = connected_data(&mut transport, fresh.bytes()).await;
    let second = gate.clone().consume(1, &fresh_data)?.unwrap();
    assert_eq!(second.position, final_pose);
    sessions.lock().unwrap().revoke(secret.expose_secret());
    assert!(!second.lease.live()?);
    drop(second);
    assert!(matches!(
        gate.reserve(context, binding),
        Err(admissions::Error::Unauthorized)
    ));
    let next_secret = sessions.lock().unwrap().issue(&identity, owner)?;
    let next = sessions
        .lock()
        .unwrap()
        .authorize(next_secret.expose_secret())?;
    let expired = gate.reserve(next.clone(), binding)?.issue(restored.saved)?;
    let expired_data = connected_data(&mut transport, expired.bytes()).await;
    tokio::time::sleep(Duration::from_secs(21)).await;
    assert!(gate.clone().consume(1, &expired_data)?.is_none());
    let reservation = gate.reserve(next, binding)?;
    drop(reservation);
    let status = worker.shutdown().await?;
    assert!(!status.failed && status.stale_records == 0);
    gate.fail_closed();
    assert!(gate.retained_positions().is_err());
    drop(admin);
    driver.await??;
    emit(
        serde_json::json!({"passed":true,"actual_oidc_and_postgres":true,"valid_unexpired_nonce_one_use":true,"tampered_nonce_and_wrong_slot_denied":true,"cancelled_selection_released":true,"queued_final_pose_not_saved":true,"old_commit_cannot_release_new_pose":true,"actual_final_commit_before_reuse":true,"stale_generation_denied":true,"revoked_lease_denied":true,"expired_reservation_reclaimed":true,"direct_lease_contract_not_native_game":true}),
    );
    Ok(())
}
