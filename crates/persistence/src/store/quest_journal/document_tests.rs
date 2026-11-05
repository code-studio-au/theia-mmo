use super::*;
use crate::documents::fixtures::*;
use theia_core::inventory::{Command, Operation};
#[tokio::test]
async fn bound_document_journals_cannot_reach_legacy_or_compound_store_io() {
    let p = profiles();
    // Reaching any database transaction on this fixture would panic.
    let mut store = Store {
        client: None,
        driver: None,
        realm: RealmId::random(),
        session: uuid::Uuid::new_v4(),
    };
    let who = binding();
    let world = [3; 32];
    let receipt = NonZeroU128::new(1).unwrap();
    let claim = crate::ClaimId::new([5; 16]).unwrap();
    let position = theia_core::coordinates::GlobalPosition::new([0.; 3]).unwrap();
    let item = Command {
        expected_revision: 0,
        operation: Operation::Destroy { from: 0, amount: 1 },
    }
    .into();
    let purchase = theia_core::vendors::Purchase {
        expected_revision: 0,
        offer: id("theia.dev.training-vendor.stones"),
    };
    refused(store.ensure_quest_journal(who, world, &p.journal).await);
    refused(store.load_quest_journal(who, world, &p.journal).await);
    refused(
        store
            .record_quest_event(
                who,
                world,
                &p.journal,
                receipt,
                &EventKey::ReadDocument(id("note")),
            )
            .await,
    );
    refused(
        store
            .record_reached_areas(who, world, &p.journal, receipt, &area(&p.journal))
            .await,
    );
    refused(
        store
            .award_progression_and_quest_journal(who, world, &p.xp, &p.journal, &event())
            .await,
    );
    refused(
        store
            .command_quest_journal(who, world, &p.journal, &p.xp, &command())
            .await,
    );
    refused(
        store
            .mutate_items_with_quest_journal(who, world, &p.items, item, &p.journal)
            .await,
    );
    refused(
        store
            .purchase_vendor_with_quest_journal(who, world, &p.vendor, &purchase, &p.journal)
            .await,
    );
    refused(
        store
            .claim_reward_with_quest_journal(who, world, &p.ordinary, claim, 0, &p.journal)
            .await,
    );
    refused(
        store
            .generate_quest_loot_for_kill(
                who,
                world,
                &p.conditional,
                &id("theia.dev.first-training-kill"),
                &event(),
                position,
            )
            .await,
    );
    refused(
        store
            .list_quest_reward_claims_nearby(who, world, &p.conditional, position)
            .await,
    );
    refused(
        store
            .pickup_quest_reward(who, world, &p.conditional, claim, 0)
            .await,
    );
}
