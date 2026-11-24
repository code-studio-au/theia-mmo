//! Inventory is locked before journal rows. All returned views share one commit.
use super::*;
use theia_core::{inventory::Bag, quests::journal::Proposal};

pub(super) async fn lock(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    catalog: &QuestJournalCatalog,
) -> Result<Option<Bag>> {
    let Some(items) = catalog.inventory() else {
        return Ok(None);
    };
    let row = bounded(tx.query_opt(
        "SELECT catalog_hash,revision,state FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
        &[&realm.value(), &binding.character.value()],
    )).await?.ok_or(Error::InventoryMissing)?;
    Ok(Some(super::super::inventory::checked(&row, items)?))
}
pub(super) fn reconcile(
    catalog: &QuestJournalCatalog,
    bag: Option<&Bag>,
    mut proposal: Proposal,
) -> Result<Proposal> {
    if let Some(bag) = bag {
        let items = catalog.inventory().ok_or(Error::QuestJournalData)?;
        let after = proposal
            .journal
            .propose_inventory(catalog.catalog(), items.catalog(), bag)
            .map_err(Error::QuestJournalCommand)?;
        proposal.changed = proposal
            .changed
            .into_iter()
            .chain(after.changed)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        proposal.journal = after.journal;
    }
    Ok(proposal)
}
#[allow(clippy::too_many_arguments)]
pub(super) async fn refresh(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    catalog: &QuestJournalCatalog,
    revision: u64,
    journal: Journal,
    bag: Option<Bag>,
    replayed: bool,
) -> Result<QuestJournalApplied> {
    let proposal = reconcile(
        catalog,
        bag.as_ref(),
        Proposal {
            journal,
            changed: Vec::new(),
            reward: None,
        },
    )?;
    let revision = if proposal.changed.is_empty() {
        revision
    } else {
        save(
            tx,
            realm,
            binding,
            catalog,
            revision,
            &proposal.journal,
            &proposal.changed,
        )
        .await?
    };
    let mut result = applied(proposal.journal, revision, proposal.changed, replayed);
    result.inventory = bag;
    Ok(result)
}
pub(super) async fn consume(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    before: &Bag,
    after: &Bag,
) -> Result<()> {
    if before == after {
        return Ok(());
    }
    if before.revision().checked_add(1) != Some(after.revision()) {
        return Err(Error::InventoryData);
    }
    // Domain-separated internal mutation marker supersedes the latest player
    // intent receipt without weakening the existing positive-revision constraint.
    let mut marker = b"theia-quest-inventory-consumption-v1\0".to_vec();
    marker.extend(after.to_json());
    let marker = ring::digest::digest(&ring::digest::SHA256, &marker);
    let changed = bounded(tx.execute(
        "UPDATE theia.backpacks SET revision=$1,state=$2,last_command=$6,updated_at=clock_timestamp() WHERE realm_id=$3 AND character_id=$4 AND revision=$5",
        &[&(after.revision() as i64), &after.to_json(), &realm.value(), &binding.character.value(), &(before.revision() as i64), &marker.as_ref()],
    )).await?;
    if changed != 1 {
        return Err(Error::InventoryData);
    }
    Ok(())
}
