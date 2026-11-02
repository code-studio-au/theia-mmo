//! Owned real PostgreSQL + public authoritative World recipient qualification.
//! Accounts, unequipped/no-cost Arcane and quest acceptance are explicit fixtures;
//! this is not OIDC,
//! normal player interaction, encrypted clients, loot consumption or native UI.
#[path = "../../../crates/persistence/tests/support/mod.rs"]
mod support;
use std::{path::PathBuf, sync::Arc, time::Duration};
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    quests::lifecycle::{Action, Command},
};
use theia_network::{
    procedural::Source,
    saved_terrain,
    simulation::{World, combat},
    wire::{ChunkReady, VERSION},
};
use theia_persistence::{
    CharacterName, InventoryCatalog, ProgressionCatalog, QuestLifecycleCatalog, RealmId,
    RewardCatalog, Store, Worker,
};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const STATS: &[u8] = include_bytes!("../../../content/gameplay/development-character-stats.json");
const ABILITIES: &[u8] =
    include_bytes!("../../../content/gameplay/development-classic-abilities.json");
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const QUEST: &[u8] = include_bytes!("../../../content/gameplay/development-quest.json");
const REWARDS: &[u8] = include_bytes!("../../../content/gameplay/development-rewards.json");
async fn world() -> theia_network::Result<World> {
    let terrain = PathBuf::from(std::env::var("THEIA_TEST_SAVED_TERRAIN")?);
    let navigation = PathBuf::from(std::env::var("THEIA_TEST_NAVIGATION")?);
    let source = Source::Saved(Arc::new(saved_terrain::Store::open(&terrain)?));
    let mut world = World::with_terrain_source([8008., 8002.], 1., source.clone())?;
    world.enable_prediction()?;
    world.enable_classic()?;
    theia_server::development::prepare_guide(
        &mut world,
        Arc::new(theia_network::dialogue::Profile::from_json(
            include_bytes!("../../../content/gameplay/development-dialogue.json"),
        )?),
        [8003., 8002.],
    )
    .await?;
    let mut creature: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-creature-deflect-pursuit.json"
    ))?;
    creature["speed_metres_per_second"] = serde_json::json!(0.1);
    creature["aggro"]["perception_metres"] = serde_json::json!(1.0);
    let creature = serde_json::to_vec(&creature)?;
    let nav = theia_network::navigation::Store::open(&navigation, source.identity())?;
    world.enable_patrol(theia_network::simulation::creatures::Prepared::load(
        theia_network::simulation::creatures::Profile::from_json(&creature)?,
        &nav,
        &source,
    )?)?;
    // Explicit unequipped/no-cost Arcane fixture keeps the private committed-
    // equipment boundary intact; this checks recipients and COMMIT, not gear.
    let mut abilities: serde_json::Value = serde_json::from_slice(ABILITIES)?;
    abilities["abilities"][0]
        .as_object_mut()
        .unwrap()
        .remove("requires_main_hand");
    abilities["abilities"][0]["mana_cost"] = serde_json::json!(0);
    let abilities = serde_json::to_vec(&abilities)?;
    world.enable_status_combat(
        &abilities,
        100,
        5000,
        STATS,
        include_bytes!("../../../content/gameplay/development-statuses.json"),
    )?;
    world.enable_player_life(Arc::new(
        theia_network::player_life_wire::Profile::from_json(
            STATS,
            &creature,
            include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
            include_bytes!("../../../content/gameplay/development-player-respawn.json"),
        )?,
    ))?;
    world.enable_autoattack(
        include_bytes!("../../../content/gameplay/development-melee-weapons.json"),
        ITEMS,
    )?;
    world.enable_physical_damage(
        include_bytes!("../../../content/gameplay/development-physical-damage-definitions.json"),
        include_bytes!("../../../content/gameplay/development-physical-damage-stats.json"),
        ITEMS,
    )?;
    world.enable_white_melee(include_bytes!(
        "../../../content/gameplay/development-white-melee.json"
    ))?;
    world.enable_ability_damage(
        include_bytes!("../../../content/gameplay/development-ability-starting.json"),
        include_bytes!("../../../content/gameplay/development-ability-damage.json"),
        &abilities,
    )?;
    world.enable_combat_membership(include_bytes!(
        "../../../content/gameplay/development-combat-membership.json"
    ))?;
    world.enable_creature_credit(include_bytes!(
        "../../../content/gameplay/development-creature-credit-solo.json"
    ))?;
    let items = theia_network::renet_runtime::inventory::Profile::from_json(ITEMS)?;
    world.enable_rewards(Arc::new(theia_network::reward_wire::Profile::from_json(
        REWARDS, &items,
    )?))?;
    world.enable_progression(Arc::new(
        theia_network::progression_wire::Profile::from_json(XP)?,
    ))?;
    world.enable_quest_lifecycle(Arc::new(
        theia_network::quest_lifecycle_wire::Profile::from_json(QUEST)?,
    ))?;
    Ok(world)
}
fn join(world: &mut World) -> theia_network::Result<u64> {
    let point = GlobalPosition::new([8008., 0., 8002.]).unwrap();
    let patch = world
        .procedural()
        .unwrap()
        .patch(point)
        .ok_or("Fixture collision missing")?;
    let mut centre = patch
        .try_spawn(8008., 8002.)
        .ok_or("Fixture spawn missing")?
        .metres();
    centre[1] += f64::from(theia_network::world::PAWN_Y);
    let id = world
        .join_restored(GlobalPosition::new(centre).unwrap())
        .ok_or("Fixture admission refused")?;
    let chunks = world
        .procedural()
        .unwrap()
        .patch(point)
        .unwrap()
        .ids
        .iter()
        .map(|id| [id.x, id.z])
        .collect();
    assert!(world.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks
        }
    ));
    Ok(id)
}
fn attack(
    world: &mut World,
    id: u64,
    npc: theia_core::identity::EntityKey,
    sequence: u64,
) -> theia_network::Result<()> {
    let reply = world.cast(
        id,
        combat::Request {
            epoch: world.epoch(),
            sequence,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target: npc,
        },
    );
    if !matches!(reply.reply.outcome, combat::Outcome::Started { .. }) {
        return Err(format!("Fixture cast refused: {:?}", reply.reply.outcome).into());
    }
    Ok(())
}
#[tokio::main]
async fn main() -> theia_network::Result<()> {
    let (url, mut admin, driver) = support::owned().await;
    let mut world = world().await?;
    let identity = world.identity();
    let a = join(&mut world)?;
    let b = join(&mut world)?;
    let npc = world.combat_snapshot(a).unwrap().targets[0].key;
    let owner = world.entity_key(a).unwrap();
    let killer = world.entity_key(b).unwrap();
    let mut first_sequence = 1;
    loop {
        attack(&mut world, a, npc, first_sequence)?;
        for _ in 0..65 {
            world.step();
        }
        if world.creature_tag_owner() == Some(owner) {
            break;
        }
        first_sequence += 1;
        if first_sequence > 10 {
            return Err("Fixture never established effective tag".into());
        }
    }
    let mut events = None;
    for sequence in 1..=30 {
        attack(&mut world, b, npc, sequence)?;
        for _ in 0..65 {
            world.step();
            if world.creature_kill_credit().is_some() {
                events = Some(world.combat_events().to_vec());
                break;
            }
        }
        if events.is_some() {
            break;
        }
    }
    let events = events.ok_or("Fixture NPC did not die before bounded rounds")?;
    let xp_facts = world.experience_opportunities(&events)?;
    let reward_facts = world.reward_opportunities(&events)?;
    assert_eq!(xp_facts.len(), 1);
    assert_eq!(reward_facts.len(), 1);
    assert_eq!(xp_facts[0].actor, owner);
    assert_eq!(reward_facts[0].actor, owner);
    assert!(
        events.iter().any(
            |e| matches!(e,combat::Event::AbilityDamage(f) if f.actor==killer&&f.remaining==0)
        )
    );
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await?;
    let items = InventoryCatalog::from_json(ITEMS)?;
    let xp = ProgressionCatalog::from_json(XP)?;
    let quest = QuestLifecycleCatalog::from_json(QUEST)?;
    let rewards = RewardCatalog::from_json(REWARDS, &items)?;
    let mut bindings = Vec::new();
    for (subject, name) in [
        ("synthetic-tag-owner", "Creditowner"),
        ("synthetic-final-hitter", "Creditkiller"),
    ] {
        let account = store
            .resolve_owner("http://127.0.0.1:40123/creature-credit-fixture", subject)
            .await?;
        let saved = store
            .create_character(
                account,
                identity,
                CharacterName::parse(name)?,
                world.global_position(a).unwrap(),
            )
            .await?
            .saved;
        let binding = saved.binding;
        store
            .ensure_inventory(
                binding,
                identity,
                &items,
                &theia_core::inventory::Bag::empty(items.catalog()),
            )
            .await?;
        store.ensure_progression(binding, identity, &xp).await?;
        store
            .ensure_quest_lifecycle(binding, identity, &quest)
            .await?;
        store
            .command_quest_lifecycle(
                binding,
                identity,
                &quest,
                Command {
                    expected_revision: 0,
                    action: Action::Accept,
                },
            )
            .await?;
        bindings.push(binding);
    }
    let (publisher, worker) = Worker::start(store);
    let tx = admin.transaction().await?;
    tx.query_one(
        "SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
        &[&realm.value(), &bindings[0].character.value()],
    )
    .await?;
    let award = {
        let publish = publisher.clone();
        let xp = xp.clone();
        let quest = quest.clone();
        let event = xp_facts[0].event.clone();
        let binding = bindings[0];
        tokio::spawn(async move {
            publish
                .award_progression_and_quest_lifecycle(binding, identity, &xp, &quest, &event)
                .await
        })
    };
    tokio::time::timeout(Duration::from_secs(1),async {loop {
        tx.query_one("SELECT pg_stat_clear_snapshot()",&[]).await?;
        if tx.query_opt("SELECT pid FROM pg_stat_activity WHERE application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE '%theia.progression%'",&[]).await?.is_some(){return Ok::<(),tokio_postgres::Error>(());}
        tokio::time::sleep(Duration::from_millis(2)).await;
    }}).await??;
    assert!(!award.is_finished());
    let tick = world.completed_tick();
    for _ in 0..10 {
        world.step();
    }
    assert_eq!(world.completed_tick(), tick + 10);
    assert!(!award.is_finished());
    tx.rollback().await?;
    let applied = award.await??;
    assert_eq!(applied.experience.granted, 25);
    assert_eq!(applied.quest.state.count(), 1);
    let claim = publisher
        .offer_reward(bindings[0], identity, &rewards, &reward_facts[0].reward)
        .await?;
    let replay = publisher
        .award_progression_and_quest_lifecycle(
            bindings[0],
            identity,
            &xp,
            &quest,
            &xp_facts[0].event,
        )
        .await?;
    assert!(replay.experience.replayed);
    assert_eq!(replay.experience.granted, 0);
    assert_eq!(replay.quest.state.count(), 1);
    assert_eq!(
        publisher
            .offer_reward(bindings[0], identity, &rewards, &reward_facts[0].reward)
            .await?
            .id,
        claim.id
    );
    worker.shutdown().await?;
    let mut restarted = Store::connect_loopback(&url, realm).await?;
    assert!(matches!(
        restarted
            .award_progression_and_quest_lifecycle(
                bindings[1],
                identity,
                &xp,
                &quest,
                &xp_facts[0].event
            )
            .await,
        Err(theia_persistence::Error::ProgressionConflict)
    ));
    for (index, binding) in bindings.iter().enumerate() {
        let progress = restarted.load_progression(*binding, identity, &xp).await?;
        let objective = restarted
            .load_quest_lifecycle(*binding, identity, &quest)
            .await?;
        let claims = restarted
            .list_reward_claims(*binding, identity, &rewards)
            .await?;
        assert_eq!(progress.total(), if index == 0 { 25 } else { 0 });
        assert_eq!(objective.count(), if index == 0 { 1 } else { 0 });
        assert_eq!(claims.len(), if index == 0 { 1 } else { 0 });
        if index == 0 {
            assert_eq!(claims[0].id, claim.id);
            assert_eq!(claims[0].consumed_revision, None);
        }
    }
    println!(
        "{}",
        serde_json::json!({"passed":true,"actual_world_competing_hitter":true,"actual_postgres":true,"sql_lock_prevented_ack":true,"simulation_advanced_while_sql_blocked":true,"owner_xp":25,"owner_quest_count":1,"owner_training_claims":1,"killer_xp":0,"killer_quest_count":0,"killer_training_claims":0,"replay_no_duplicate":true,"wrong_recipient_replay_refused":true,"connection_restart_verified":true,"oidc":false,"encrypted_clients":false,"native_ui":false,"loot_claim_consumed":false})
    );
    drop(restarted);
    drop(admin);
    driver.await??;
    Ok(())
}
