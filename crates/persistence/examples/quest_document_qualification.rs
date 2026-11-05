//! Owned restart/retry probe; trusted bindings/NPC actions, not native viewing.
#[path = "../tests/support/documents.rs"]
mod documents;
#[path = "../tests/support/mod.rs"]
mod support;
use documents::*;
use theia_core::quests::journal::Status;
use theia_persistence::*;
#[tokio::main]
async fn main() -> Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("646f6375-6d65-4e74-8000-000000000001").unwrap();
    let p = profiles(false);
    let mut store = Store::connect_loopback(&url, realm).await?;
    let who = match mode.as_str() {
        "write" => {
            let who = initialize(&mut store, "Documentprobe", 2, &p).await;
            store
                .open_quest_document(who, WORLD, &p.journal, &open(&p, 100, 0, 0))
                .await?;
            accept(&mut store, who, &p).await;
            store
                .open_quest_document(who, WORLD, &p.journal, &open(&p, 101, 0, 0))
                .await?;
            let ready = store
                .open_quest_document(who, WORLD, &p.journal, &open(&p, 102, 0, 0))
                .await?;
            let revision = ready.journal.progress(&id("quest")).unwrap().revision();
            store
                .command_quest_journal(
                    who,
                    WORLD,
                    &p.journal,
                    &p.xp,
                    &command(2, QuestJournalAction::TurnIn, revision),
                )
                .await?;
            who
        }
        "read" => {
            let owner = store
                .resolve_owner("https://documents.theia.test", "Documentprobe")
                .await?;
            let people = store.list_characters(owner, WORLD).await?;
            let character = people
                .iter()
                .find(|c| c.name.as_str() == "Documentprobe")
                .ok_or(Error::Invalid("Missing document restart character"))?
                .id;
            Binding { owner, character }
        }
        _ => return Err(Error::Invalid("Expected write or read")),
    };
    let before = store.load_quest_journal(who, WORLD, &p.journal).await?;
    let old = store
        .open_quest_document(who, WORLD, &p.journal, &open(&p, 101, 0, 0))
        .await?;
    let progress = old.journal.progress(&id("quest")).unwrap();
    if !old.replayed
        || old.journal != before.journal
        || old.revision != before.revision
        || progress.status() != Status::Completed
        || progress.count(&id("read-first")) != Some(1)
        || progress.count(&id("read-final")) != Some(1)
    {
        return Err(Error::Invalid("Document restart read receipt mismatch"));
    }
    let xp = store.load_progression(who, WORLD, &p.xp).await?;
    if xp.total() != 25 || old.inventory.as_ref().unwrap().revision() != 0 {
        return Err(Error::Invalid("Document restart bag/XP mismatch"));
    }
    println!(
        "document_probe {}",
        serde_json::json!({"character":who.character.value().to_string(),"revision":old.revision,"quest":{"revision":progress.revision(),"status":format!("{:?}",progress.status()),"read_first":progress.count(&id("read-first")),"read_final":progress.count(&id("read-final"))},"bag":serde_json::from_slice::<serde_json::Value>(&old.inventory.as_ref().unwrap().to_json()).unwrap(),"xp":xp.total(),"journal_profile":p.journal.hash(),"document_profile":p.documents.hash()})
    );
    store.close().await?;
    drop(admin);
    driver
        .await
        .map_err(|_| Error::Invalid("Admin driver stopped"))??;
    Ok(())
}
