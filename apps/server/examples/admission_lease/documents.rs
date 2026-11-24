//! Real retained owner, with direct lease requests after an actual encrypted
//! ticket handshake. This is not document-wire or native UI proof.
use super::*;
use net::progression::{Input, JournalInput, JournalOperation};
use theia_core::{
    content::ContentId,
    documents::OpenCommand,
    quests::journal::{Action, Command, Status},
};
use theia_persistence::{InventoryCatalog, Publisher, SavedCharacter};
pub(super) struct Fixture {
    pub gate: Arc<admissions::GameAdmissions>,
    pub publisher: Publisher,
    pub worker: Worker,
    pub transport: renet_netcode::NetcodeServerTransport,
    pub context: theia_auth::SessionAuthorization<theia_persistence::OwnerId>,
    pub saved: SavedCharacter,
    pub profile: InventoryCatalog,
    pub admin: tokio_postgres::Client,
    pub driver: tokio::task::JoinHandle<std::result::Result<(), tokio_postgres::Error>>,
    pub url: String,
    pub realm: RealmId,
    pub world: [u8; 32],
}
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn open(request: u128, revision: u64, slot: u8) -> theia_core::inventory::ItemCommand {
    OpenCommand {
        request_id: std::num::NonZeroU128::new(request).unwrap(),
        expected_revision: revision,
        slot,
        document: 0,
    }
    .into()
}
async fn read(
    port: &mut net::inventory::Port,
    sequence: u64,
    command: theia_core::inventory::ItemCommand,
) -> net::inventory::Observation {
    port.requests
        .try_send(net::inventory::Request { sequence, command })
        .unwrap();
    inventory_until(port, |s| s.reply.is_some_and(|r| r.sequence == sequence)).await
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
    let notes = serde_json::to_vec(
        &serde_json::json!({"schema":1,"id":"documents","revision":1,"item_catalog":profile.catalog().id().as_str(),"documents":[{"id":"note","item":"theia.dev.training-stone","title":"Orders","pages":["Follow the eastern road.","Return to the guide."]}]}),
    )?;
    let wire_items = net::inventory::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-equipment-items.json"
    ))?
    .with_documents(&notes)?;
    let wire = Arc::new(wire_items.documents().unwrap().clone());
    let stored = theia_persistence::DocumentCatalog::from_json(&notes, &profile)?;
    let data = serde_json::to_vec(
        &serde_json::json!({"schema":4,"id":"catalog","revision":1,"quests":[{"id":"quest","version":1,"title":"Orders","description":"Read the owned orders twice in sequence.","prerequisites":[],"giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"first","stages":[{"id":"first","completion":{"kind":"all"},"next":"second","objectives":[{"kind":"read_document","id":"read-first","label":"Read orders","document":"note"}]},{"id":"second","completion":{"kind":"all"},"next":null,"objectives":[{"kind":"read_document","id":"read-final","label":"Read again","document":"note"}]}]}]}),
    )?;
    let journal = theia_persistence::QuestJournalCatalog::from_json_with_documents(
        &data, None, &xp, &stored,
    )?;
    admissions::GameAdmissions::with_progression(
        &mut gate,
        xp.clone(),
        Arc::new(theia_network::progression_wire::Profile::from_json(
            xp_bytes,
        )?),
        publisher.clone(),
    )?;
    assert!(admissions::GameAdmissions::with_quest_journal(&mut gate, journal.clone()).is_err());
    let mut changed = notes.clone();
    changed.push(b' ');
    let foreign = Arc::new(theia_network::documents::Profile::from_json(
        &changed,
        &wire_items,
    )?);
    assert!(
        admissions::GameAdmissions::with_document_quest_journal(
            &mut gate,
            journal.clone(),
            foreign
        )
        .is_err()
    );
    admissions::GameAdmissions::with_document_quest_journal(
        &mut gate,
        journal.clone(),
        wire.clone(),
    )?;
    let actor = EntityKey {
        epoch: WorldEpoch::new(174).unwrap(),
        entity: EntityId::new(1).unwrap(),
    };
    let ticket = gate.reserve(context.clone(), binding)?.issue(saved)?;
    let lease = gate
        .clone()
        .consume(1, &connected_data(&mut transport, ticket.bytes()).await)?
        .unwrap()
        .lease;
    lease.record(actor, saved.position, 1)?;
    let mut items = lease.inventory()?.unwrap();
    let mut progress = lease.progression()?.unwrap();
    assert_eq!(items.documents.as_ref().unwrap().hash(), wire.hash());
    inventory_until(&mut items, |s| s.bag.is_some()).await;
    progression_until(&mut progress, |s| {
        s.journal.is_some() && s.progress.is_some()
    })
    .await;
    assert!(matches!(
        read(&mut items, 1, open(100, 0, 0))
            .await
            .reply
            .unwrap()
            .outcome,
        theia_network::inventory_wire::Outcome::Applied
    ));
    progress
        .input
        .try_send(action(actor, journal.hash(), 1, 200, Action::Accept, 0))?;
    progression_until(&mut progress, |s| {
        s.journal
            .as_ref()
            .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 1))
    })
    .await;
    assert!(matches!(
        read(&mut items, 2, open(100, 0, 0))
            .await
            .reply
            .unwrap()
            .outcome,
        theia_network::inventory_wire::Outcome::Replayed
    ));
    let snapshot = progress.state.borrow().journal.clone().unwrap();
    assert_eq!(
        snapshot
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read-first")),
        Some(0)
    );
    read(&mut items, 3, open(101, 0, 0)).await;
    progression_until(&mut progress, |s| {
        s.journal.as_ref().is_some_and(|j| {
            j.journal
                .progress(&id("quest"))
                .is_some_and(|p| p.stage() == &id("second"))
        })
    })
    .await;
    let guard = admin.transaction().await?;
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &binding.character.value()],
        )
        .await?;
    items.requests.try_send(net::inventory::Request {
        sequence: 4,
        command: open(102, 0, 0),
    })?;
    tokio::time::timeout(Duration::from_millis(300),async {loop {
        guard.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
        if guard.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE '%theia.backpacks%'",&[]).await.unwrap().is_some(){break;}
        tokio::time::sleep(Duration::from_millis(2)).await;
    }}).await.expect("Document open did not wait on owned bag lock");
    // This second accepted request must drain behind the original open during retirement.
    items.requests.try_send(net::inventory::Request {
        sequence: 5,
        command: ItemCommand {
            expected_revision: 0,
            operation: ItemOperation::Destroy {
                from: 0,
                amount: 13,
            },
        }
        .into(),
    })?;
    drop(lease);
    gate.flush(&publisher, &worker.status()).unwrap();
    assert!(gate.reserve(context.clone(), binding).is_err());
    guard.commit().await?;
    let final_items = inventory_until(&mut items, |s| s.closed).await;
    assert!(!final_items.failed);
    assert_eq!(final_items.bag.as_ref().unwrap().revision(), 1);
    assert!(final_items.bag.as_ref().unwrap().slots()[0].is_none());
    // Retirement commits accepted work without delivering a fresh answer to the departed session.
    assert_eq!(final_items.reply.unwrap().sequence, 3);
    let ready = progression_until(&mut progress, |s| s.closed).await;
    assert!(!ready.failed);
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
    let mut items = lease.inventory()?.unwrap();
    let mut progress = lease.progression()?.unwrap();
    inventory_until(&mut items, |s| s.bag.is_some()).await;
    progression_until(&mut progress, |s| {
        s.journal.is_some() && s.progress.is_some()
    })
    .await;
    assert!(matches!(
        read(&mut items, 1, open(103, 1, 0))
            .await
            .reply
            .unwrap()
            .outcome,
        theia_network::inventory_wire::Outcome::Rejected(
            theia_network::inventory_wire::Reason::EmptySource
        )
    ));
    assert!(matches!(
        read(&mut items, 2, open(102, 0, 0))
            .await
            .reply
            .unwrap()
            .outcome,
        theia_network::inventory_wire::Outcome::Replayed
    ));
    assert!(matches!(
        read(&mut items, 3, open(104, 1, 1))
            .await
            .reply
            .unwrap()
            .outcome,
        theia_network::inventory_wire::Outcome::Applied
    ));
    let revision = progress
        .state
        .borrow()
        .journal
        .as_ref()
        .unwrap()
        .journal
        .progress(&id("quest"))
        .unwrap()
        .revision();
    progress.input.try_send(action(
        actor,
        journal.hash(),
        1,
        201,
        Action::TurnIn,
        revision,
    ))?;
    let completed = progression_until(&mut progress, |s| {
        s.journal
            .as_ref()
            .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 1))
    })
    .await;
    assert_eq!(completed.progress.as_ref().unwrap().total(), 25);
    assert_eq!(
        completed
            .journal
            .as_ref()
            .unwrap()
            .journal
            .progress(&id("quest"))
            .unwrap()
            .status(),
        Status::Completed
    );
    progress.input.try_send(action(
        actor,
        journal.hash(),
        2,
        201,
        Action::TurnIn,
        revision,
    ))?;
    let replayed = progression_until(&mut progress, |s| {
        s.journal
            .as_ref()
            .is_some_and(|j| j.reply.as_ref().is_some_and(|r| r.sequence == 2))
    })
    .await;
    assert!(
        replayed
            .journal
            .as_ref()
            .unwrap()
            .reply
            .as_ref()
            .unwrap()
            .rejection
            .is_none()
    );
    assert_eq!(replayed.progress.as_ref().unwrap().total(), 25);
    drop(lease);
    gate.flush(&publisher, &worker.status()).unwrap();
    inventory_until(&mut items, |s| s.closed).await;
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
    assert_eq!(status.journal_events, 6);
    assert_eq!(status.journal_replayed, 3);
    assert_eq!(status.journal_commands, 3);
    let mut reopened = Store::connect_loopback(&url, realm).await?;
    let result = reopened
        .open_quest_document(
            binding,
            world,
            &journal,
            &theia_persistence::DocumentOpenCommand {
                request_id: std::num::NonZeroU128::new(102).unwrap(),
                profile_hash: wire.hash(),
                document: id("note"),
                expected_revision: 0,
                slot: 0,
            },
        )
        .await?;
    assert!(result.replayed);
    assert_eq!(
        result.journal.progress(&id("quest")).unwrap().status(),
        Status::Completed
    );
    assert_eq!(
        result
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read-first")),
        Some(1)
    );
    assert_eq!(
        result
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read-final")),
        Some(1)
    );
    let experience = reopened.load_progression(binding, world, &xp).await?;
    assert_eq!(experience.total(), 25);
    let receipts: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.quest_journal_events WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &binding.character.value()],
        )
        .await?
        .get(0);
    assert_eq!(receipts, 4);
    reopened.close().await?;
    drop(admin);
    driver.await??;
    emit(
        serde_json::json!({"passed":true,"document_owner_enabled":true,"actual_oidc_and_postgres":true,"encrypted_ticket_binding":true,"original_read_stage_and_preaccept_replay":true,"read_survives_retirement_and_item_loss":true,"fresh_missing_rejected_nonfatal":true,"rejoin_original_authorization":true,"atomic_turn_in_once":true,"direct_lease_contract_not_native_game":true,"document_wire_not_exercised_by_this_fixture":true}),
    );
    Ok(())
}
