//! Real World install/cast boundaries; durable eligibility is tested separately.
use super::*;
use crate::{progression_wire, quest_journal, renet_runtime::inventory};
use theia_core::time::TickRate;
const ITEMS: &[u8] = include_bytes!("../../../../../content/gameplay/development-loot-items.json");
const XP: &[u8] = include_bytes!("../../../../../content/gameplay/development-progression.json");
fn profiles() -> (
    inventory::Profile,
    Arc<progression_wire::Profile>,
    Arc<quest_journal::Profile>,
    Arc<Profile>,
) {
    profiles_for(ITEMS)
}
fn profiles_for(
    bytes: &[u8],
) -> (
    inventory::Profile,
    Arc<progression_wire::Profile>,
    Arc<quest_journal::Profile>,
    Arc<Profile>,
) {
    let items = inventory::Profile::from_json(bytes).unwrap();
    let xp = Arc::new(progression_wire::Profile::from_json(XP).unwrap());
    let quest = serde_json::json!({"schema":3,"id":"catalog","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Pelts","description":"Fixture","prerequisites":[],
        "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
        "kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.wolf-pelt","required":3,"consume_on_turn_in":true}]}]}]});
    let journal = Arc::new(
        quest_journal::Profile::from_json(&serde_json::to_vec(&quest).unwrap(), &xp)
            .unwrap()
            .with_inventory(&items)
            .unwrap(),
    );
    let mut rewards: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-random-creature-loot-rewards.json"
    ))
    .unwrap();
    rewards["schema"] = 4.into();
    rewards["rewards"][1]["quest_required"] = true.into();
    let profile = Arc::new(
        Profile::from_json_with_quest_journal(
            &serde_json::to_vec(&rewards).unwrap(),
            &items,
            &journal,
        )
        .unwrap(),
    );
    (items, xp, journal, profile)
}
fn world(rate: TickRate) -> (World, EntityKey) {
    let (mut world, npc) = super::super::creatures::tests::fixture(rate);
    let mut abilities: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-abilities.json"
    ))
    .unwrap();
    abilities["abilities"][0]["cast_ms"] = 0.into();
    abilities["abilities"][0]["effect"]["amount"] = 100.into();
    world
        .enable_combat(&serde_json::to_vec(&abilities).unwrap(), 100, 1000)
        .unwrap();
    world
        .spawn_development_guide(
            Arc::new(
                crate::dialogue::Profile::from_json(include_bytes!(
                    "../../../../../content/quests/development-journal-dialogue.json"
                ))
                .unwrap(),
            ),
            [8000., 8000.],
        )
        .unwrap();
    (world, npc)
}
#[test]
fn conditional_install_requires_exact_installed_journal_and_inactive_world() {
    let (items, xp, journal, rewards) = profiles();
    let (mut world, _) = world(TickRate::Hz30);
    let before = world.identity();
    assert!(world.enable_quest_loot(rewards.clone()).is_err());
    assert_eq!(world.identity(), before);
    let mut changed = ITEMS.to_vec();
    changed.push(b' ');
    let foreign = inventory::Profile::from_json(&changed).unwrap();
    let foreign_journal = profiles_for(&changed).2;
    world
        .enable_item_quest_journal(foreign_journal, xp.clone(), &foreign)
        .unwrap();
    let installed = world.identity();
    assert!(world.enable_quest_loot(rewards.clone()).is_err());
    assert_eq!(world.identity(), installed);
    assert!(world.reward_profile().is_none());
    let (mut world, _) = self::world(TickRate::Hz30);
    world
        .enable_item_quest_journal(journal.clone(), xp.clone(), &items)
        .unwrap();
    let before = world.identity();
    assert!(world.enable_rewards(rewards.clone()).is_err());
    assert!(world.enable_creature_loot(rewards.clone()).is_err());
    assert_eq!(world.identity(), before);
    world.enable_quest_loot(rewards.clone()).unwrap();
    let installed = world.identity();
    assert_ne!(installed, before);
    assert!(world.enable_quest_loot(rewards.clone()).is_err());
    assert_eq!(world.identity(), installed);
    for joined in [false, true] {
        let (mut active, _) = self::world(TickRate::Hz30);
        active
            .enable_item_quest_journal(journal.clone(), xp.clone(), &items)
            .unwrap();
        let identity = active.identity();
        if joined {
            active.join().unwrap();
        } else {
            active.step();
        }
        assert!(active.enable_quest_loot(rewards.clone()).is_err());
        assert!(active.reward_profile().is_none());
        assert_eq!(active.identity(), identity);
    }
    world.join().unwrap();
    assert!(world.enable_quest_loot(rewards).is_err());
    assert_eq!(world.identity(), installed);
}
#[test]
fn conditional_world_forwards_original_fatal_life_without_inventory_or_need_at_all_rates() {
    let (items, xp, journal, rewards) = profiles();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = world(rate);
        world
            .enable_item_quest_journal(journal.clone(), xp.clone(), &items)
            .unwrap();
        world.enable_quest_loot(rewards.clone()).unwrap();
        let player = world.join().unwrap();
        let actor = world.entity_key(player).unwrap();
        let position = world.global_position(player).unwrap();
        let chunks = world
            .procedural
            .as_ref()
            .unwrap()
            .patch(position)
            .unwrap()
            .ids
            .iter()
            .map(|c| [c.x, c.z])
            .collect();
        assert!(world.client_ready(
            player,
            &crate::wire::ChunkReady {
                version: crate::wire::VERSION,
                epoch: world.epoch(),
                sequence: 1,
                chunks
            }
        ));
        world.cast(
            player,
            super::super::combat::Request {
                epoch: world.epoch(),
                sequence: 1,
                target: npc,
                ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            },
        );
        world.step();
        let events = world.combat_events().to_vec();
        let opportunities = world.reward_opportunities(&events).unwrap();
        assert_eq!(opportunities.len(), 3);
        for fact in &opportunities {
            assert_eq!(fact.actor, actor);
            assert_eq!(fact.event.id, KillId::from_victim(npc));
            assert_eq!(
                &fact.event.creature,
                world.patrol.as_ref().unwrap().definition()
            );
            assert_eq!(fact.position, world.patrol.as_ref().unwrap().position());
        }
        assert!(opportunities.iter().any(|f| {
            rewards
                .catalog()
                .definition(&f.reward)
                .unwrap()
                .quest_item()
                .is_some()
        }));
        let fatal = events
            .iter()
            .find(|e| matches!(e, Event::Damage { remaining: 0, .. }))
            .unwrap()
            .clone();
        assert_eq!(
            world
                .reward_opportunities(&[fatal.clone(), fatal.clone()])
                .unwrap(),
            opportunities
        );
        let mut nonfatal = fatal.clone();
        if let Event::Damage { remaining, .. } = &mut nonfatal {
            *remaining = 1;
        }
        assert!(world.reward_opportunities(&[nonfatal]).unwrap().is_empty());
        let source = theia_core::rewards::CorpseSource {
            life: KillId::from_victim(npc),
            position,
        };
        assert!(world.corpse_pickup_eligible(player, rewards.hash(), source));
        let mut outside = position.metres();
        outside[0] += 3.01;
        assert!(!world.corpse_pickup_eligible(
            player,
            rewards.hash(),
            theia_core::rewards::CorpseSource {
                position: theia_core::coordinates::GlobalPosition::new(outside).unwrap(),
                ..source
            }
        ));
        for _ in 0..(rate.hz() + 1) {
            world.step();
        }
        assert_ne!(world.patrol.as_ref().unwrap().key, npc);
        assert!(world.reward_opportunities(&[fatal]).unwrap().is_empty());
    }
}
