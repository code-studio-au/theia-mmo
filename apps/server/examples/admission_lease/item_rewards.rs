//! Actual OIDC/SQL owner proof with direct lease requests after ticket handshake.
//! This deliberately does not drive the game command wire or desktop.
use super::*;
use document_owner::Fixture;
use net::progression::{Input, JournalInput, JournalOperation};
use theia_core::{
    content::ContentId,
    quests::journal::{Action, Command, Status},
};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn action(
    actor: EntityKey,
    catalog: [u8; 32],
    sequence: u64,
    request: u128,
    action: Action,
    revision: u64,
) -> Input {
    Input::Journal(JournalInput {
        actor,
        catalog,
        sequence,
        operation: JournalOperation::Command(Command {
            request_id: std::num::NonZeroU128::new(request).unwrap(),
            quest: id("quest"),
            npc: id("theia.dev.guide"),
            action,
            expected_revision: revision,
        }),
    })
}
async fn reply(port: &mut net::progression::Port, sequence: u64) -> net::progression::Observation {
    progression_until(port, |s| {
        s.journal
            .as_ref()
            .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == sequence))
    })
    .await
}
fn tokens(bag: &theia_core::inventory::Bag) -> u32 {
    bag.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &id("theia.dev.training-token"))
        .map(|s| s.count())
        .sum()
}
pub(super) async fn run(f: Fixture) -> theia_network::Result<()> {
    let Fixture {
        mut gate,
        publisher,
        worker,
        mut transport,
        context,
        saved,
        profile,
        mut admin,
        driver,
        url,
        realm,
        world,
    } = f;
    let binding = saved.binding;
    let xp_bytes = include_bytes!("../../../../content/gameplay/development-progression.json");
    let xp = theia_persistence::ProgressionCatalog::from_json(xp_bytes)?;
    let wx = Arc::new(theia_network::progression_wire::Profile::from_json(
        xp_bytes,
    )?);
    let items = net::inventory::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-equipment-items.json"
    ))?;
    let q = serde_json::to_vec(
        &serde_json::json!({"schema":3,"id":"fixed-owner-quests","revision":1,"quests":[{"id":"quest","version":1,"title":"Delivery","description":"Bring thirteen stones","giver":"theia.dev.guide","turn_in":"theia.dev.guide","prerequisites":[],"repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"stage","stages":[{"id":"stage","next":null,"completion":{"kind":"all"},"objectives":[{"kind":"possess_item","id":"bring","label":"Stones","item":"theia.dev.training-stone","required":13,"consume_on_turn_in":true}]}]}]}),
    )?;
    let r = serde_json::to_vec(
        &serde_json::json!({"schema":1,"id":"fixed-owner-rewards","revision":1,"quest_catalog":"fixed-owner-quests","item_catalog":profile.catalog().id().as_str(),"rewards":[{"quest":"quest","grants":[{"item":"theia.dev.training-token","count":3}]}]}),
    )?;
    let journal = theia_persistence::QuestJournalCatalog::from_json(&q, &xp)?
        .with_item_rewards(&r, &profile)?;
    let wire = Arc::new(
        theia_network::quest_journal::Profile::from_json(&q, &wx)?.with_item_rewards(&r, &items)?,
    );
    assert_eq!(journal.hash(), wire.hash());
    admissions::GameAdmissions::with_progression(&mut gate, xp.clone(), wx, publisher.clone())?;
    assert!(admissions::GameAdmissions::with_quest_journal(&mut gate, journal.clone()).is_err());
    let mut changed = r.clone();
    changed.push(b' ');
    let wrong = Arc::new(
        theia_network::quest_journal::Profile::from_json(
            &q,
            &theia_network::progression_wire::Profile::from_json(xp_bytes)?,
        )?
        .with_item_rewards(&changed, &items)?,
    );
    assert!(
        admissions::GameAdmissions::with_item_reward_quest_journal(
            &mut gate,
            journal.clone(),
            wrong
        )
        .is_err()
    );
    admissions::GameAdmissions::with_item_reward_quest_journal(&mut gate, journal.clone(), wire)?;
    let actor = EntityKey {
        epoch: WorldEpoch::new(175).unwrap(),
        entity: EntityId::new(1).unwrap(),
    };
    // Explicit owned fixture: a full bag whose delivery does not free a slot.
    let mut slots = std::array::from_fn(|_| {
        theia_core::inventory::Stack::new(profile.catalog(), id("theia.dev.training-sword"), 1)
    });
    for (slot, item, count) in [
        (0, "theia.dev.training-stone", 20),
        (1, "theia.dev.training-stone", 8),
        (2, "theia.dev.training-token", 99),
    ] {
        slots[slot] = theia_core::inventory::Stack::new(profile.catalog(), id(item), count);
    }
    assert!(slots.iter().all(Option::is_some));
    let seed = theia_core::inventory::Bag::from_slots(profile.catalog(), 0, slots).unwrap();
    publisher
        .ensure_inventory(binding, world, &profile, &seed)
        .await?;
    let ticket = gate.reserve(context.clone(), binding)?.issue(saved)?;
    let lease = gate
        .clone()
        .consume(1, &connected_data(&mut transport, ticket.bytes()).await)?
        .unwrap()
        .lease;
    lease.record(actor, saved.position, 1)?;
    let mut inventory = lease.inventory()?.unwrap();
    let mut progress = lease.progression()?.unwrap();
    let initial = inventory_until(&mut inventory, |s| s.bag.is_some()).await;
    assert_eq!(tokens(initial.bag.as_ref().unwrap()), 99);
    progression_until(&mut progress, |s| {
        s.journal.is_some() && s.progress.is_some()
    })
    .await;
    progress
        .input
        .try_send(action(actor, journal.hash(), 1, 100, Action::Accept, 0))?;
    let accepted = reply(&mut progress, 1).await;
    let ready = accepted
        .journal
        .as_ref()
        .unwrap()
        .journal
        .progress(&id("quest"))
        .unwrap();
    assert_eq!(ready.status(), Status::Ready);
    let revision = ready.revision();
    progress.input.try_send(action(
        actor,
        journal.hash(),
        2,
        101,
        Action::TurnIn,
        revision,
    ))?;
    let refused = reply(&mut progress, 2).await;
    assert_eq!(
        refused
            .journal
            .as_ref()
            .unwrap()
            .reply
            .as_ref()
            .unwrap()
            .rejection,
        Some(net::progression::JournalRejection::InventoryFull)
    );
    assert_eq!(refused.progress.as_ref().unwrap().total(), 0);
    assert_eq!(inventory.state.borrow().bag.as_ref().unwrap().revision(), 0);
    // Free two filler slots using ordinary owner commands. The same previously
    // failed quest request must remain retryable because no receipt was saved.
    for (sequence, slot) in [(1, 14), (2, 15)] {
        inventory.requests.try_send(net::inventory::Request {
            sequence,
            command: ItemCommand {
                expected_revision: sequence - 1,
                operation: ItemOperation::Destroy {
                    from: slot,
                    amount: 1,
                },
            }
            .into(),
        })?;
        inventory_until(&mut inventory, |s| {
            s.reply.is_some_and(|r| r.sequence == sequence)
        })
        .await;
    }
    // Hold the bag so the turn-in has been accepted but cannot commit or publish.
    let guard = admin.transaction().await?;
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &binding.character.value()],
        )
        .await?;
    progress.input.try_send(action(
        actor,
        journal.hash(),
        3,
        101,
        Action::TurnIn,
        revision,
    ))?;
    wait_inventory_lock(&guard).await;
    assert_eq!(tokens(inventory.state.borrow().bag.as_ref().unwrap()), 99);
    assert_eq!(
        progress.state.borrow().progress.as_ref().unwrap().total(),
        0
    );
    // Item work queued behind the shared owner mutation must retain its ordering
    // even when this session retires and stops receiving fresh replies.
    inventory.requests.try_send(net::inventory::Request {
        sequence: 3,
        command: ItemCommand {
            expected_revision: 3,
            operation: ItemOperation::Move { from: 14, to: 15 },
        }
        .into(),
    })?;
    drop(lease);
    gate.flush(&publisher, &worker.status()).unwrap();
    assert!(gate.reserve(context.clone(), binding).is_err());
    guard.commit().await?;
    let final_items = inventory_until(&mut inventory, |s| s.closed).await;
    let final_progress = progression_until(&mut progress, |s| s.closed).await;
    assert!(!final_items.failed && !final_progress.failed);
    let bag = final_items.bag.as_ref().unwrap();
    assert_eq!(bag.revision(), 4);
    assert_eq!(tokens(bag), 102);
    assert!(bag.slots()[14].is_none());
    assert_eq!(bag.slots()[15].as_ref().unwrap().count(), 3);
    assert_eq!(final_progress.progress.as_ref().unwrap().total(), 25);
    assert_eq!(
        final_progress
            .journal
            .as_ref()
            .unwrap()
            .journal
            .progress(&id("quest"))
            .unwrap()
            .status(),
        Status::Completed
    );
    let ticket = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            gate.flush(&publisher, &worker.status()).unwrap();
            if let Ok(reservation) = gate.reserve(context.clone(), binding) {
                break reservation.issue(saved).unwrap();
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    let lease = gate
        .clone()
        .consume(1, &connected_data(&mut transport, ticket.bytes()).await)?
        .unwrap()
        .lease;
    lease.record(actor, saved.position, 2)?;
    let mut inventory = lease.inventory()?.unwrap();
    let mut progress = lease.progression()?.unwrap();
    inventory_until(&mut inventory, |s| s.bag.is_some()).await;
    progression_until(&mut progress, |s| {
        s.journal.is_some() && s.progress.is_some()
    })
    .await;
    progress.input.try_send(action(
        actor,
        journal.hash(),
        1,
        101,
        Action::TurnIn,
        revision,
    ))?;
    let replay = reply(&mut progress, 1).await;
    assert!(
        replay
            .journal
            .as_ref()
            .unwrap()
            .reply
            .as_ref()
            .unwrap()
            .rejection
            .is_none()
    );
    assert_eq!(replay.progress.as_ref().unwrap().total(), 25);
    let replay_items = inventory.state.borrow().clone();
    assert_eq!(tokens(replay_items.bag.as_ref().unwrap()), 102);
    assert_eq!(replay_items.bag.as_ref().unwrap().revision(), 4);
    drop(lease);
    gate.flush(&publisher, &worker.status()).unwrap();
    inventory_until(&mut inventory, |s| s.closed).await;
    progression_until(&mut progress, |s| s.closed).await;
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            gate.flush(&publisher, &worker.status()).unwrap();
            if let Ok(reservation) = gate.reserve(context.clone(), binding) {
                drop(reservation);
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    let status = worker.shutdown().await?;
    assert!(!status.failed);
    let mut reopened = Store::connect_loopback(&url, realm).await?;
    let cmd = theia_persistence::QuestJournalCommand {
        request_id: std::num::NonZeroU128::new(101).unwrap(),
        quest: id("quest"),
        npc: id("theia.dev.guide"),
        action: theia_persistence::QuestJournalAction::TurnIn,
        expected_revision: revision,
    };
    let retry = reopened
        .command_quest_journal(binding, world, &journal, &xp, &cmd)
        .await?;
    assert!(retry.replayed);
    assert_eq!(retry.experience_granted, 0);
    assert_eq!(tokens(retry.inventory.as_ref().unwrap()), 102);
    reopened.close().await?;
    drop(admin);
    driver.await??;
    emit(
        serde_json::json!({"passed":true,"fixed_quest_reward_owner_enabled":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"no_reward_before_commit":true,"capacity_refusal_nonfatal_and_same_intent_retried":true,"accepted_turn_in_and_item_move_survive_retirement":true,"original_retry_after_rejoin_no_extra_grant":true,"exact_reward_policy_required":true,"direct_lease_contract_not_native_game":true,"game_command_wire_exercised":false}),
    );
    Ok(())
}
