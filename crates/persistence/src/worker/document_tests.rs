use super::*;
use crate::documents::fixtures::*;
use theia_core::{
    inventory::{Command, Operation},
    quests::catalog::EventKey,
};
#[tokio::test]
async fn bound_document_jobs_refuse_before_queueing_or_worker_failure() {
    let p = profiles();
    let who = binding();
    let world = [3; 32];
    let (latest, _latest) = watch::channel(None);
    let (accounts, _accounts) = mpsc::channel(16);
    let (inventory, mut requests) = mpsc::channel(16);
    let publisher = Publisher {
        latest,
        submitted: Arc::new(AtomicU64::new(0)),
        accounts,
        inventory,
    };
    let receipt = std::num::NonZeroU128::new(1).unwrap();
    let claim = crate::ClaimId::new([5; 16]).unwrap();
    let position = theia_core::coordinates::GlobalPosition::new([0.; 3]).unwrap();
    let reward = id("theia.dev.first-training-kill");
    let item = Command {
        expected_revision: 0,
        operation: Operation::Destroy { from: 0, amount: 1 },
    }
    .into();
    let purchase = theia_core::vendors::Purchase {
        expected_revision: 0,
        offer: id("theia.dev.training-vendor.stones"),
    };
    refused(publisher.ensure_quest_journal(who, world, &p.journal).await);
    refused(publisher.load_quest_journal(who, world, &p.journal).await);
    refused(publisher.enqueue_quest_journal_command(who, world, &p.journal, &p.xp, command()));
    refused(
        publisher
            .command_quest_journal(who, world, &p.journal, &p.xp, command())
            .await,
    );
    refused(publisher.enqueue_quest_event(
        who,
        world,
        &p.journal,
        receipt,
        EventKey::ReadDocument(id("note")),
    ));
    refused(
        publisher
            .record_quest_event(
                who,
                world,
                &p.journal,
                receipt,
                EventKey::ReadDocument(id("note")),
            )
            .await,
    );
    refused(publisher.enqueue_reached_areas(who, world, &p.journal, receipt, area(&p.journal)));
    refused(
        publisher
            .record_reached_areas(who, world, &p.journal, receipt, area(&p.journal))
            .await,
    );
    refused(publisher.enqueue_progression_and_quest_journal(
        who,
        world,
        &p.xp,
        &p.journal,
        event(),
    ));
    refused(
        publisher
            .award_progression_and_quest_journal(who, world, &p.xp, &p.journal, &event())
            .await,
    );
    refused(publisher.enqueue_items_with_quest_journal(who, world, &p.items, item, &p.journal));
    refused(
        publisher
            .mutate_items_with_quest_journal(who, world, &p.items, item, &p.journal)
            .await,
    );
    refused(
        publisher
            .claim_reward_with_quest_journal(who, world, &p.ordinary, claim, 0, &p.journal)
            .await,
    );
    refused(
        publisher
            .purchase_vendor_with_quest_journal(who, world, &p.vendor, &purchase, &p.journal)
            .await,
    );
    refused(publisher.enqueue_quest_loot_generation(
        who,
        world,
        &p.conditional,
        &reward,
        &event(),
        position,
    ));
    refused(
        publisher
            .generate_quest_loot_for_kill(who, world, &p.conditional, &reward, &event(), position)
            .await,
    );
    refused(
        publisher
            .list_quest_reward_claims_nearby(who, world, &p.conditional, position)
            .await,
    );
    refused(publisher.enqueue_quest_loot_pickup(who, world, &p.conditional, claim, 0));
    refused(
        publisher
            .pickup_quest_reward(who, world, &p.conditional, claim, 0)
            .await,
    );
    assert!(matches!(
        requests.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    ));
    assert_eq!(publisher.inventory.capacity(), 16);
}
