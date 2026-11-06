//! Cross-layer metadata agreement and closed-admission proof, not saved rewards.
use serde_json::json;
use std::sync::Arc;
use theia_network::{
    Result, quest_journal::Profile as Wire, renet_runtime::inventory::Profile as WireItems,
};
use theia_persistence::{InventoryCatalog, ProgressionCatalog, QuestJournalCatalog as Stored};
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-document-items.json");
const QUESTS: &[u8] = include_bytes!("../../../content/quests/development-journal-documents.json");
const DOCS: &[u8] = include_bytes!("../../../content/quests/development-documents.json");
const AREAS: &[u8] = include_bytes!("../../../content/quests/development-quest-areas.json");
fn source() -> Vec<u8> {
    serde_json::to_vec(&json!({"schema":1,"id":"quests","revision":1,"quests":[{"id":"quest","version":1,"title":"Test","description":"Test","prerequisites":[],"giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{"kind":"kill","id":"goal","label":"One wolf","creature":"wolf","required":1}]}]}]})).unwrap()
}
fn rewards(quest: &[u8]) -> Vec<u8> {
    let q: serde_json::Value = serde_json::from_slice(quest).unwrap();
    serde_json::to_vec(&json!({"schema":1,"id":"theia.test.fixed-quest-rewards","revision":1,"quest_catalog":q["id"],"item_catalog":"theia.dev.equipment-items","rewards":[{"quest":q["quests"][0]["id"],"grants":[{"item":"theia.dev.training-token","count":3}]}]})).unwrap()
}
fn pair(
    q: &[u8],
    items: &[u8],
    xp: &[u8],
    reward: &[u8],
    areas: Option<&[u8]>,
    docs: Option<&[u8]>,
) -> Result<(Wire, Stored)> {
    let wi = WireItems::from_json(items)?;
    let si = InventoryCatalog::from_json(items)?;
    let wx = theia_network::progression_wire::Profile::from_json(xp)?;
    let sx = ProgressionCatalog::from_json(xp)?;
    let (wire, stored) = if let Some(docs) = docs {
        let wd = theia_network::documents::Profile::from_json(docs, &wi)?;
        let sd = theia_persistence::DocumentCatalog::from_json(docs, &si)?;
        (
            Wire::from_json_with_documents(q, areas, &wx, &wd)?,
            Stored::from_json_with_documents(q, areas, &sx, &sd)?,
        )
    } else if let Some(areas) = areas {
        (
            Wire::from_json_with_areas(q, areas, &wx)?,
            Stored::from_json_with_areas(q, areas, &sx)?,
        )
    } else {
        (Wire::from_json(q, &wx)?, Stored::from_json(q, &sx)?)
    };
    let wire = wire.with_item_rewards(reward, &wi)?;
    let stored = stored.with_item_rewards(reward, &si)?;
    assert_eq!(wire.hash(), stored.hash());
    assert_eq!(wire.inventory_hash(), Some(si.hash()));
    Ok((wire, stored))
}
#[test]
fn reward_only_quests_bind_inventory_and_preserve_the_legacy_hash_domain() {
    let q = source();
    let r = rewards(&q);
    let (wire, stored) = pair(&q, ITEMS, XP, &r, None, None).unwrap();
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let old = Wire::from_json(&q, &xp).unwrap();
    assert!(old.inventory_hash().is_none());
    assert!(old.item_rewards().is_none());
    assert!(wire.item_rewards().is_some());
    assert!(stored.item_rewards().is_some());
    let mut legacy = theia_core::quests::journal::PROFILE_DOMAIN.to_vec();
    legacy.extend(&q);
    legacy.extend(xp.hash());
    assert_eq!(old.hash(), theia_network::terrain::sha256(&legacy));
    let mut items = theia_core::quests::journal::ITEM_PROFILE_DOMAIN.to_vec();
    items.extend(old.hash());
    items.extend(WireItems::from_json(ITEMS).unwrap().hash());
    let mut bound = theia_core::quests::item_rewards::PROFILE_DOMAIN.to_vec();
    bound.extend(theia_network::terrain::sha256(&items));
    bound.extend(&r);
    assert_eq!(wire.hash(), theia_network::terrain::sha256(&bound));
    assert_ne!(wire.hash(), old.hash());
}
#[test]
fn every_raw_context_change_distinguishes_matching_wire_and_storage_reward_policies() {
    let r = rewards(QUESTS);
    let (base, _) = pair(QUESTS, ITEMS, XP, &r, Some(AREAS), Some(DOCS)).unwrap();
    for index in 0..6 {
        let mut parts = [
            QUESTS.to_vec(),
            ITEMS.to_vec(),
            XP.to_vec(),
            r.clone(),
            AREAS.to_vec(),
            DOCS.to_vec(),
        ];
        parts[index].push(b' ');
        let (wire, _) = pair(
            &parts[0],
            &parts[1],
            &parts[2],
            &parts[3],
            Some(&parts[4]),
            Some(&parts[5]),
        )
        .unwrap();
        assert_ne!(
            wire.hash(),
            base.hash(),
            "Context {index} lost exact-byte binding"
        );
    }
}
#[test]
fn duplicate_binding_and_same_id_but_different_inventory_bytes_refuse() {
    let q = source();
    let r = rewards(&q);
    let (wire, stored) = pair(&q, ITEMS, XP, &r, None, None).unwrap();
    let wi = WireItems::from_json(ITEMS).unwrap();
    let si = InventoryCatalog::from_json(ITEMS).unwrap();
    assert!(wire.clone().with_item_rewards(&r, &wi).is_err());
    assert!(stored.clone().with_item_rewards(&r, &si).is_err());
    let mut changed = ITEMS.to_vec();
    changed.push(b' ');
    let wx = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let sx = ProgressionCatalog::from_json(XP).unwrap();
    let wd = theia_network::documents::Profile::from_json(DOCS, &wi).unwrap();
    let sd = theia_persistence::DocumentCatalog::from_json(DOCS, &si).unwrap();
    assert!(
        Wire::from_json_with_documents(QUESTS, Some(AREAS), &wx, &wd)
            .unwrap()
            .with_item_rewards(&rewards(QUESTS), &WireItems::from_json(&changed).unwrap())
            .is_err()
    );
    assert!(
        Stored::from_json_with_documents(QUESTS, Some(AREAS), &sx, &sd)
            .unwrap()
            .with_item_rewards(
                &rewards(QUESTS),
                &InventoryCatalog::from_json(&changed).unwrap()
            )
            .is_err()
    );
}
#[test]
fn installed_area_and_document_definition_namespaces_cannot_alias_reward_catalogue() {
    let good = rewards(QUESTS);
    let a: serde_json::Value = serde_json::from_slice(AREAS).unwrap();
    let d: serde_json::Value = serde_json::from_slice(DOCS).unwrap();
    for id in [
        &a["id"],
        &a["areas"][0]["id"],
        &d["id"],
        &d["documents"][0]["id"],
    ] {
        let mut r: serde_json::Value = serde_json::from_slice(&good).unwrap();
        r["id"] = id.clone();
        let r = serde_json::to_vec(&r).unwrap();
        assert!(pair(QUESTS, ITEMS, XP, &r, Some(AREAS), Some(DOCS)).is_err());
    }
}
#[tokio::test]
async fn exact_reward_metadata_cannot_enter_existing_world_or_owner_admission() {
    let q = source();
    let r = rewards(&q);
    let (wire, stored) = pair(&q, ITEMS, XP, &r, None, None).unwrap();
    let xp = Arc::new(theia_network::progression_wire::Profile::from_json(XP).unwrap());
    let mut world = theia_network::simulation::World::default();
    let identity = world.identity();
    assert!(
        world
            .enable_item_quest_journal(Arc::new(wire), xp, &WireItems::from_json(ITEMS).unwrap())
            .unwrap_err()
            .to_string()
            .contains("Quest item reward World")
    );
    assert_eq!(world.identity(), identity);
    assert!(world.quest_journal_profile().is_none());
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (_, issuer) = theia_network::renet_runtime::secure_issuer(socket, vec![address]).unwrap();
    let mut gate = crate::admissions::GameAdmissions::new(
        Arc::new(std::sync::Mutex::new(theia_auth::Sessions::default())),
        issuer,
        [1; 32],
    );
    assert!(
        crate::admissions::GameAdmissions::with_quest_journal(&mut gate, stored.clone())
            .unwrap_err()
            .to_string()
            .contains("Quest item reward owner")
    );
    assert!(
        stored
            .check_progression(&ProgressionCatalog::from_json(XP).unwrap())
            .unwrap_err()
            .to_string()
            .contains("Quest item reward storage")
    );
}
