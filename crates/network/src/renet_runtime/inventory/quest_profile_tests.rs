use super::*;
const ITEMS: &[u8] = include_bytes!("../../../../../content/gameplay/development-loot-items.json");
const XP: &[u8] = include_bytes!("../../../../../content/gameplay/development-progression.json");
const QUESTS: &[u8] =
    include_bytes!("../../../../../content/quests/development-journal-items.json");
const AREAS: &[u8] = include_bytes!("../../../../../content/quests/development-quest-areas.json");
const REWARDS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-random-creature-loot-rewards.json");
fn journal(items: &Profile, quests: &[u8]) -> crate::quest_journal::Profile {
    crate::quest_journal::Profile::from_json_with_areas(
        quests,
        AREAS,
        &crate::progression_wire::Profile::from_json(XP).unwrap(),
    )
    .unwrap()
    .with_inventory(items)
    .unwrap()
}
fn rewards() -> Vec<u8> {
    let mut source: serde_json::Value = serde_json::from_slice(REWARDS).unwrap();
    source["schema"] = 4.into();
    source["rewards"][1]["quest_required"] = true.into();
    serde_json::to_vec(&source).unwrap()
}
#[test]
fn conditional_inventory_attachment_preserves_item_vendor_identity_and_checks_exact_journal() {
    let items = Profile::from_json(ITEMS).unwrap();
    let journal = journal(&items, QUESTS);
    let vendor = include_bytes!("../../../../../content/gameplay/development-vendor.json");
    let items = items.with_vendor(vendor).unwrap();
    let vendor_hash = items.vendor().unwrap().hash();
    let attached = items.clone().with_quest_loot(&rewards(), &journal).unwrap();
    assert_eq!(items.hash(), attached.hash());
    assert_eq!(items.catalog(), attached.catalog());
    assert_eq!(attached.vendor().unwrap().hash(), vendor_hash);
    assert_eq!(
        attached.rewards().unwrap().quest_journal_hash(),
        Some(journal.hash())
    );
    attached.check_quest_journal(Some(&journal)).unwrap();
    assert!(attached.check_quest_journal(None).is_err());
    let mut changed = QUESTS.to_vec();
    changed.push(b' ');
    let other = self::journal(&items, &changed);
    assert!(attached.check_quest_journal(Some(&other)).is_err());
    let mut changed = ITEMS.to_vec();
    changed.push(b' ');
    let other_items = Profile::from_json(&changed).unwrap();
    let other = self::journal(&other_items, QUESTS);
    assert!(attached.check_quest_journal(Some(&other)).is_err());
    assert!(other_items.with_quest_loot(&rewards(), &journal).is_err());
    assert!(attached.with_quest_loot(&rewards(), &journal).is_err());
    assert!(items.clone().with_rewards(&rewards()).is_err());
    assert!(items.clone().with_creature_loot(&rewards()).is_err());
    let ordinary = items.clone().with_creature_loot(REWARDS).unwrap();
    ordinary.check_quest_journal(None).unwrap();
    ordinary.check_quest_journal(Some(&journal)).unwrap();
    assert!(ordinary.with_quest_loot(&rewards(), &journal).is_err());
}

async fn startup_rejection(
    items: Option<Profile>,
    journal: Option<crate::quest_journal::Profile>,
) -> String {
    let (_, input) = watch::channel(crate::wire::Intent::default());
    let (state, _) = watch::channel(crate::client::State::default());
    let (_, shutdown) = watch::channel(false);
    let (_, ready) = watch::channel(None);
    let inventory = items.map(|profile| {
        let (_, commands) = mpsc::channel(1);
        Client {
            profile,
            commands,
            pickups: None,
        }
    });
    let journal = journal.map(|profile| {
        let (_, actions) = mpsc::channel(4);
        super::super::quest_journal::Client {
            profile: Arc::new(profile),
            actions,
        }
    });
    super::super::run_client_gameplay(
        "127.0.0.1:1".parse().unwrap(),
        &[],
        input,
        state,
        shutdown,
        [1; 32],
        super::super::GameplayClient {
            chat: None,
            avatar_life: None,
            public_attacks: None,
            classic_source: None,
            dodge: None,
            player_life: None,
            lifecycle: None,
            dialogue: None,
            commands: None,
            choices: None,
            combat: None,
            quest: None,
            progression: None,
            statuses: None,
            ready,
            journal,
            inventory,
        },
    )
    .await
    .unwrap_err()
    .to_string()
}
#[tokio::test]
async fn gameplay_client_refuses_missing_or_foreign_item_journal_before_reading_ticket() {
    let items = Profile::from_json(ITEMS).unwrap();
    let bound = journal(&items, QUESTS);
    let attached = items.with_quest_loot(&rewards(), &bound).unwrap();
    assert_eq!(
        startup_rejection(None, Some(bound.clone())).await,
        "Item-bound journal requires client inventory"
    );
    assert_eq!(
        startup_rejection(Some(attached.clone()), None).await,
        "Quest loot requires the exact client item journal"
    );
    let mut changed = QUESTS.to_vec();
    changed.push(b' ');
    assert_eq!(
        startup_rejection(Some(attached.clone()), Some(journal(&attached, &changed))).await,
        "Quest loot requires the exact client item journal"
    );
    let mut changed = ITEMS.to_vec();
    changed.push(b' ');
    assert_eq!(
        startup_rejection(Some(Profile::from_json(&changed).unwrap()), Some(bound)).await,
        "Journal/inventory client profile mismatch"
    );
}
