//! Owned process/database restart and original-intent replay, not live NPC proof.
#[path = "../tests/support/item_rewards.rs"]
mod fixtures;
#[path = "../tests/support/mod.rs"]
mod support;
use fixtures::*;
use theia_core::{
    inventory::{Command, ItemCommand, Operation},
    quests::journal::Status,
};
use theia_persistence::*;
#[tokio::main]
async fn main() -> Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("6974656d-7265-4761-8000-000000000001").unwrap();
    let p = profiles(true, false);
    let mut store = Store::connect_loopback(&url, realm).await?;
    let who = match mode.as_str() {
        "write" => {
            let who = initialize(&mut store, "Itemrewardprobe", &p, &bag(&p, false, 3)).await;
            let ready = ready(&mut store, who, &p).await;
            if ready.journal.progress(&id("quest")).unwrap().revision() != 2 {
                return Err(Error::Invalid(
                    "Unexpected fixed reward fixture ready revision",
                ));
            }
            let done = store
                .command_quest_journal(
                    who,
                    WORLD,
                    &p.journal,
                    &p.xp,
                    &command(20, QuestJournalAction::TurnIn, 2),
                )
                .await?;
            if done.experience_granted != 25 {
                return Err(Error::Invalid("Fixed reward fixture XP mismatch"));
            }
            store
                .mutate_items_with_quest_journal(
                    who,
                    WORLD,
                    &p.items,
                    ItemCommand::Backpack(Command {
                        expected_revision: 1,
                        operation: Operation::Move { from: 0, to: 15 },
                    }),
                    &p.journal,
                )
                .await?;
            who
        }
        "read" => {
            let owner = store
                .resolve_owner("https://fixed-rewards.theia.test", "Itemrewardprobe")
                .await?;
            let list = store.list_characters(owner, WORLD).await?;
            let character = list
                .iter()
                .find(|c| c.name.as_str() == "Itemrewardprobe")
                .ok_or(Error::Invalid("Missing fixed reward restart character"))?
                .id;
            Binding { owner, character }
        }
        _ => return Err(Error::Invalid("Expected write or read")),
    };
    let before = store.load_quest_journal(who, WORLD, &p.journal).await?;
    let replay = store
        .command_quest_journal(
            who,
            WORLD,
            &p.journal,
            &p.xp,
            &command(20, QuestJournalAction::TurnIn, 2),
        )
        .await?;
    let bag = replay.inventory.as_ref().ok_or(Error::InventoryMissing)?;
    if !replay.replayed
        || replay.experience_granted != 0
        || replay.journal != before.journal
        || replay.revision != 3
        || replay.journal.progress(&id("quest")).unwrap().status() != Status::Completed
        || bag.revision() != 2
        || bag.slots()[0].is_some()
        || count(bag, "theia.dev.training-token") != 3
        || count(bag, "theia.dev.training-stone") != 0
        || bag.slots()[15]
            .as_ref()
            .is_none_or(|s| s.item() != &id("theia.dev.training-token") || s.count() != 3)
        || replay.experience.as_ref().is_none_or(|xp| xp.total() != 25)
    {
        return Err(Error::Invalid(
            "Fixed reward restart receipt/bag/XP mismatch",
        ));
    }
    println!(
        "{}",
        serde_json::json!({"passed":true,"mode":mode,"journal_revision":replay.revision,"bag_revision":bag.revision(),"xp":25,"tokens":3,"original_turn_in_replayed":true,"later_item_move_retained":true})
    );
    store.close().await?;
    drop(admin);
    driver
        .await
        .map_err(|_| Error::Invalid("Owned admin driver failed"))??;
    Ok(())
}
