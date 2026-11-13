//! Real passive-World facts and owned SQL receipts, with one synthetic player.
//! This deliberately retains the development last-hit fixture. It does not
//! adopt population first-tag eligibility, normal launch flags, OIDC or native UI.
#[path = "../../../crates/persistence/tests/support/mod.rs"]
mod support;
use std::{collections::BTreeSet, num::NonZeroU128, path::PathBuf, sync::Arc, time::Duration};
use theia_core::{content::ContentId, coordinates::GlobalPosition, progression::KillEvent};
use theia_network::{
    procedural::Source,
    simulation::{World, combat, population, quest_journal},
    wire::{ChunkReady, VERSION},
};
use theia_persistence::{
    CharacterName, ProgressionCatalog, QuestJournalAction, QuestJournalCatalog,
    QuestJournalCommand, RealmId, Store, Worker,
};
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
fn id(value: &str) -> ContentId {
    ContentId::new(value).unwrap()
}
fn quest_bytes() -> Vec<u8> {
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/quests/development-journal.json"
    ))
    .unwrap();
    // Explicit isolated receipt fixture: no introductory conversation, ten
    // creature lives, no turn-in grant. Normal quest content is untouched.
    let mut quest = raw["quests"][1].clone();
    quest["prerequisites"] = serde_json::json!([]);
    quest["stages"][0]["objectives"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    quest["stages"][0]["objectives"][0]["required"] = serde_json::json!(10);
    raw["quests"] = serde_json::json!([quest]);
    serde_json::to_vec(&raw).unwrap()
}
fn prepare(bytes: &[u8]) -> theia_network::Result<(World, u64, quest_journal::Prepared)> {
    let package = PathBuf::from(std::env::var("THEIA_POPULATION_PACKAGE")?);
    let store = Arc::new(theia_network::authored_world::Store::open(
        &package.join("world"),
    )?);
    let spawn = store
        .spawn(0)
        .ok_or("Population package has no player spawn")?
        .metres();
    let source = Source::Authored(store);
    let loaded =
        population::configuration::Loaded::open(&package.join("population.json"), &source)?;
    assert_eq!(loaded.count(), 5);
    let respawn_ms = loaded.respawn_ms();
    let (prepared, binding) = loaded.into_parts();
    let feet = prepared.points()[0];
    let mut world = World::with_terrain_source([spawn[0], spawn[2]], 1., source)?;
    world.enable_prediction()?;
    world.enable_classic()?;
    world.enable_defined_population(prepared, binding)?;
    world.enable_combat(
        include_bytes!("../../../content/gameplay/development-abilities.json"),
        100,
        respawn_ms,
    )?;
    world.enable_progression(Arc::new(
        theia_network::progression_wire::Profile::from_json(XP)?,
    ))?;
    world.spawn_placed_guide(
        Arc::new(theia_network::dialogue::Profile::from_json(
            include_bytes!("../../../content/quests/development-journal-dialogue.json"),
        )?),
        theia_network::dialogue::Placement::Feet(feet),
    )?;
    let journal = world.prepare_quest_journal(Arc::new(
        theia_core::quests::catalog::Catalog::from_json(bytes)?,
    ))?;
    let mut body = feet.metres();
    body[1] += f64::from(theia_network::world::PAWN_Y);
    let actor = world
        .join_restored(GlobalPosition::new(body).unwrap())
        .ok_or("Population fixture admission refused")?;
    let chunks = world
        .procedural()
        .unwrap()
        .patch(feet)
        .unwrap()
        .ids
        .iter()
        .map(|c| [c.x, c.z])
        .collect();
    assert!(world.client_ready(
        actor,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks
        }
    ));
    Ok((world, actor, journal))
}
fn kills(
    world: &mut World,
    actor: u64,
    journal: &quest_journal::Prepared,
    sequence: &mut u64,
) -> theia_network::Result<Vec<KillEvent>> {
    let keys: Vec<_> = world
        .combat_snapshot(actor)
        .unwrap()
        .targets
        .iter()
        .map(|t| {
            assert_eq!(t.current, 100);
            t.key
        })
        .collect();
    assert_eq!(keys.len(), 5);
    let mut facts = Vec::new();
    for target in keys {
        for _ in 0..5 {
            *sequence += 1;
            let result = world.cast(
                actor,
                combat::Request {
                    epoch: world.epoch(),
                    sequence: *sequence,
                    ability: id("theia.dev.arcane-bolt"),
                    target,
                },
            );
            assert!(
                matches!(result.reply.outcome, combat::Outcome::Started { .. }),
                "{:?}",
                result.reply
            );
            for _ in 0..2 * world.tick_rate().hz() {
                world.step();
                let xp = world.experience_opportunities(world.combat_events())?;
                let quest = world.quest_kill_occurrences(journal, world.combat_events())?;
                assert_eq!(xp.len(), quest.len());
                for (xp, quest) in xp.into_iter().zip(quest) {
                    assert_eq!(xp.actor, world.entity_key(actor).unwrap());
                    assert_eq!(quest, xp);
                    facts.push(xp.event);
                }
            }
        }
    }
    assert_eq!(facts.len(), 5);
    Ok(facts)
}
#[tokio::main]
async fn main() -> theia_network::Result<()> {
    let (url, mut admin, driver) = support::owned().await;
    let bytes = quest_bytes();
    let (mut world, actor, journal) = prepare(&bytes)?;
    let identity = world.identity();
    let xp = ProgressionCatalog::from_json(XP)?;
    let quest = QuestJournalCatalog::from_json(&bytes, &xp)?;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await?;
    let account = store
        .resolve_owner(
            "http://127.0.0.1:40123/population-receipt-fixture",
            "one-player",
        )
        .await?;
    let binding = store
        .create_character(
            account,
            identity,
            CharacterName::parse("Populationowner")?,
            world.global_position(actor).unwrap(),
        )
        .await?
        .saved
        .binding;
    store.ensure_progression(binding, identity, &xp).await?;
    store
        .ensure_quest_journal(binding, identity, &quest)
        .await?;
    store
        .command_quest_journal(
            binding,
            identity,
            &quest,
            &xp,
            &QuestJournalCommand {
                request_id: NonZeroU128::new(1).unwrap(),
                quest: id("theia.dev.quest-follow-up"),
                npc: id("theia.dev.guide"),
                expected_revision: 0,
                action: QuestJournalAction::Accept,
            },
        )
        .await?;
    let (publisher, worker) = Worker::start(store);
    let mut sequence = 0;
    let first = kills(&mut world, actor, &journal, &mut sequence)?;
    let tx = admin.transaction().await?;
    tx.query_one(
        "SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
        &[&realm.value(), &binding.character.value()],
    )
    .await?;
    let award = {
        let publisher = publisher.clone();
        let xp = xp.clone();
        let quest = quest.clone();
        let event = first[0].clone();
        tokio::spawn(async move {
            publisher
                .award_progression_and_quest_journal(binding, identity, &xp, &quest, &event)
                .await
        })
    };
    tokio::time::timeout(Duration::from_secs(2), async { loop {
        tx.query_one("SELECT pg_stat_clear_snapshot()", &[]).await?;
        if tx.query_opt("SELECT pid FROM pg_stat_activity WHERE application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE '%theia.progression%'", &[]).await?.is_some() {
            return Ok::<(),tokio_postgres::Error>(());
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
    }}).await??;
    let tick = world.completed_tick();
    for _ in 0..10 {
        world.step();
    }
    assert_eq!(world.completed_tick(), tick + 10);
    assert!(!award.is_finished());
    tx.rollback().await?;
    assert_eq!(award.await??.experience_granted, 25);
    for event in &first[1..] {
        assert_eq!(
            publisher
                .award_progression_and_quest_journal(binding, identity, &xp, &quest, event)
                .await?
                .experience_granted,
            25
        );
    }
    // Drive only fixed simulation time until all five slots have fresh lives.
    let deadline = world
        .combat_snapshot(actor)
        .unwrap()
        .targets
        .iter()
        .filter_map(|t| t.respawn_at)
        .max();
    if let Some(deadline) = deadline {
        while world.completed_tick() <= deadline {
            world.step();
        }
    }
    let second = kills(&mut world, actor, &journal, &mut sequence)?;
    let all: Vec<_> = first.into_iter().chain(second).collect();
    assert_eq!(
        all.iter()
            .map(|f| f.id.bytes())
            .collect::<BTreeSet<_>>()
            .len(),
        10
    );
    for event in &all[5..] {
        assert_eq!(
            publisher
                .award_progression_and_quest_journal(binding, identity, &xp, &quest, event)
                .await?
                .experience_granted,
            25
        );
    }
    for event in all.iter().rev().chain(all.iter()) {
        let result = publisher
            .award_progression_and_quest_journal(binding, identity, &xp, &quest, event)
            .await?;
        assert!(result.replayed);
        assert_eq!(result.experience_granted, 0);
    }
    worker.shutdown().await?;
    let mut reopened = Store::connect_loopback(&url, realm).await?;
    assert_eq!(
        reopened
            .load_progression(binding, identity, &xp)
            .await?
            .total(),
        250
    );
    let saved = reopened
        .load_quest_journal(binding, identity, &quest)
        .await?;
    let progress = saved
        .journal
        .progress(&id("theia.dev.quest-follow-up"))
        .unwrap();
    assert_eq!(
        progress.count(&id("theia.dev.quest-follow-up.objective.wolves")),
        Some(10)
    );
    assert_eq!(
        progress.status(),
        theia_core::quests::journal::Status::Ready
    );
    assert_eq!(world.movement_recoveries(), 0);
    println!(
        "{}",
        serde_json::json!({"passed":true,"population_receipts":true,"actors":5,
        "distinct_lives":10,"experience":250,"quest_count":10,"duplicate_awards":0,
        "sql_lock_prevented_ack":true,"simulation_advanced_while_sql_blocked":true,
        "connection_restart_verified":true,"recipient_policy":"single-player development last-hit fixture",
        "population_first_tag":false,"normal_launch":false,"oidc":false,"native_ui":false})
    );
    drop(reopened);
    drop(admin);
    driver.await??;
    Ok(())
}
