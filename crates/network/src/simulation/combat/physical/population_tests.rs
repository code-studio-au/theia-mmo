//! Real authored five-creature physical melee, with no renderer or fake equipment protocol.
use super::*;
use crate::{
    authored_world,
    navigation::{self, spawn_candidates::Request, spawn_preparation},
    procedural,
    simulation::combat::autoattack,
    simulation::population::Binding,
    wire::{ChunkReady, VERSION},
};
use theia_core::time::TickRate;
const ITEMS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-equipment-items.json");
const WEAPONS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-melee-weapons.json");
const STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-character-stats.json");
const CREATURE: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-creature-passive.json");
const DEFINITIONS: &[u8] = include_bytes!(
    "../../../../../../content/gameplay/development-physical-damage-definitions.json"
);
const DAMAGE_STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-physical-damage-stats.json");
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_physical_swings_resolve_each_life_independently_at_all_rates() {
    qualify(false, None, false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_ordinary_swings_resolve_each_life_independently_at_all_rates() {
    qualify(true, None, false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_special_damage_resolves_each_life_independently_at_all_rates() {
    qualify(true, Some("theia.dev.sword-slash"), false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_arcane_damage_resolves_each_life_independently_at_all_rates() {
    qualify(true, Some("theia.dev.arcane-bolt"), false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_competing_finishers_preserve_each_first_tag_at_all_rates() {
    qualify(true, Some("theia.dev.sword-slash"), true);
}
fn equip(world: &mut World, id: u64, revision: u64, item: &str) {
    world.observe_combat_equipment(
        world.entity_key(id).unwrap(),
        Some((
            crate::terrain::sha256(ITEMS),
            revision,
            Some(ContentId::new(item).unwrap()),
        )),
    );
}
fn engage(world: &mut World, id: u64, target: EntityKey, sequence: u64) {
    let reply = world.request_autoattack(
        id,
        autoattack::Request {
            epoch: world.epoch(),
            stream_epoch: world.correction(id).unwrap().state.input.stream_epoch,
            sequence,
            target: Some(target),
        },
    );
    assert_eq!(reply.outcome, autoattack::Outcome::Engaged);
}
fn qualify(ordinary: bool, ability: Option<&str>, competing: bool) {
    let store = Arc::new(
        authored_world::Store::open(std::path::Path::new(
            &std::env::var_os("THEIA_AUTHORED_WORLD").unwrap(),
        ))
        .unwrap(),
    );
    let nav = navigation::Store::open(
        std::path::Path::new(&std::env::var_os("THEIA_AUTHORED_NAV").unwrap()),
        store.identity(),
    )
    .unwrap();
    let request: Request = serde_json::from_slice(
        &std::fs::read(std::env::var_os("THEIA_AUTHORED_SPAWNS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(request.count, 5);
    let definition = creatures::Definition::from_json(CREATURE).unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let prepared = spawn_preparation::check_authored(&store, &nav, &request)
            .unwrap()
            .into_prepared()
            .unwrap();
        let feet = prepared.points().to_vec();
        let saved = store.spawn(0).unwrap().metres();
        let mut world = World::with_terrain_source(
            [saved[0], saved[2]],
            1.,
            procedural::Source::Authored(store.clone()),
        )
        .unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        let keys = world
            .enable_defined_population(
                prepared,
                Binding::new(definition.id(), definition.clone(), 1).unwrap(),
            )
            .unwrap();
        world
            .enable_status_combat(
                include_bytes!(
                    "../../../../../../content/gameplay/development-classic-abilities.json"
                ),
                100,
                1000,
                STATS,
                include_bytes!("../../../../../../content/gameplay/development-statuses.json"),
            )
            .unwrap();
        world.enable_autoattack(WEAPONS, ITEMS).unwrap();
        let before = world.identity();
        assert!(
            world
                .enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
                .is_err()
        );
        assert_eq!(world.identity(), before);
        let life = crate::player_life_wire::Profile::from_json(
            STATS,
            CREATURE,
            include_bytes!(
                "../../../../../../content/gameplay/development-creature-retaliation.json"
            ),
            include_bytes!("../../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap();
        world
            .enable_passive_population_player_life(Arc::new(life))
            .unwrap();
        let before = world.identity();
        let mut wrong = ITEMS.to_vec();
        wrong.push(b' ');
        assert!(
            world
                .enable_physical_damage(DEFINITIONS, DAMAGE_STATS, &wrong)
                .is_err()
        );
        assert_eq!(world.identity(), before);
        world
            .enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
            .unwrap();
        assert_ne!(world.identity(), before);
        let installed = world.identity();
        assert!(
            world
                .enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
                .is_err()
        );
        assert_eq!(world.identity(), installed);
        let profile = world.physical_damage_profile().unwrap().clone();
        if ordinary {
            world
                .enable_white_melee(include_bytes!(
                    "../../../../../../content/gameplay/development-white-melee.json"
                ))
                .unwrap();
            // No accidental admission of singleton pursuit-dependent consumers.
            let identity = world.identity();
            assert!(world.enable_public_creature_attacks().is_err());
            assert_eq!(world.identity(), identity);
        }
        if ability.is_some() {
            let starting = include_bytes!(
                "../../../../../../content/gameplay/development-ability-starting.json"
            );
            let mut policy: serde_json::Value = serde_json::from_slice(include_bytes!(
                "../../../../../../content/gameplay/development-ability-damage.json"
            ))
            .unwrap();
            if competing {
                // A deterministic positive opening special establishes the tag.
                policy["abilities"][0]["miss_bp"] = 0.into();
                policy["abilities"][0]["dodge_bp"] = 0.into();
                policy["abilities"][0]["critical_bp"] = 0.into();
            }
            let policy = serde_json::to_vec(&policy).unwrap();
            let abilities = include_bytes!(
                "../../../../../../content/gameplay/development-classic-abilities.json"
            );
            let before = world.identity();
            let mut changed = abilities.to_vec();
            changed.push(b' ');
            assert!(
                world
                    .enable_ability_damage(starting, &policy, &changed)
                    .is_err()
            );
            assert_eq!(world.identity(), before);
            world
                .enable_ability_damage(starting, &policy, abilities)
                .unwrap();
            assert_ne!(world.identity(), before);
            let installed = world.identity();
            assert!(
                world
                    .enable_ability_damage(starting, &policy, abilities)
                    .is_err()
            );
            assert_eq!(world.identity(), installed);
        }
        if ability.is_some() {
            let membership = include_bytes!(
                "../../../../../../content/gameplay/development-combat-membership.json"
            );
            let before = world.identity();
            let mut limited: serde_json::Value = serde_json::from_slice(membership).unwrap();
            limited["maximum_opponents"] = 1.into();
            assert!(
                world
                    .enable_combat_membership(&serde_json::to_vec(&limited).unwrap())
                    .is_err()
            );
            assert_eq!(world.identity(), before);
            world.enable_combat_membership(membership).unwrap();
            assert_ne!(world.identity(), before);
            let installed = world.identity();
            assert!(world.enable_combat_membership(membership).is_err());
            assert_eq!(world.identity(), installed);
        }
        if ability.is_some() {
            let credit: &[u8] = if competing {
                include_bytes!(
                    "../../../../../../content/gameplay/development-creature-credit-solo.json"
                )
            } else {
                include_bytes!(
                    "../../../../../../content/gameplay/development-creature-credit.json"
                )
            };
            let before = world.identity();
            world.enable_creature_credit(credit).unwrap();
            assert_ne!(world.identity(), before);
            let installed = world.identity();
            assert!(world.enable_creature_credit(credit).is_err());
            assert_eq!(world.identity(), installed);
            assert!(world.creature_credit_encounter().is_none());
        }
        let ability_profile = world.ability_damage_profile().cloned();
        let ordinary_profile = world.white_melee_profile().cloned();
        for &key in &keys {
            assert_eq!(
                world.creature_facing(key),
                Some(theia_core::facing::Facing::default())
            );
        }

        let mut players = Vec::new();
        for point in feet {
            let mut body = point.metres();
            body[1] += f64::from(crate::world::PAWN_Y);
            let id = world
                .join_restored(GlobalPosition::new(body).unwrap())
                .unwrap();
            let chunks = world
                .procedural()
                .unwrap()
                .patch(world.global_position(id).unwrap())
                .unwrap()
                .ids
                .iter()
                .map(|c| [c.x, c.z])
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
            world.observe_combat_equipment(
                world.entity_key(id).unwrap(),
                Some((
                    crate::terrain::sha256(ITEMS),
                    1,
                    Some(ContentId::new("theia.dev.training-sword").unwrap()),
                )),
            );
            players.push(id);
        }
        if ordinary {
            let center = world
                .combat_snapshot(players[0])
                .unwrap()
                .targets
                .iter()
                .find(|t| t.key == keys[0])
                .unwrap()
                .position
                .metres();
            for (offset, in_front) in [(-0.5, true), (0.5, false)] {
                let patch = world
                    .procedural()
                    .unwrap()
                    .patch(GlobalPosition::new(center).unwrap())
                    .unwrap();
                let mut body = patch
                    .try_spawn(center[0], center[2] + offset)
                    .unwrap()
                    .metres();
                body[1] += f64::from(crate::world::PAWN_Y);
                let probe = world
                    .join_restored(GlobalPosition::new(body).unwrap())
                    .unwrap();
                let actor = world.entity_key(probe).unwrap();
                assert!(world.creature_facing(actor).is_none());
                let state = world
                    .combat
                    .as_ref()
                    .unwrap()
                    .ordinary_creature_state(&world, keys[0], actor)
                    .unwrap();
                assert_eq!(state.in_front, in_front);
                world.leave(probe);
            }
        }
        let finishers = if competing {
            players
                .iter()
                .enumerate()
                .map(|(index, &primary)| {
                    if index >= crate::wire::MAX_PLAYERS - players.len() {
                        return primary;
                    }
                    let position = world.global_position(primary).unwrap();
                    let id = world.join_restored(position).unwrap();
                    let chunks = world
                        .procedural()
                        .unwrap()
                        .patch(position)
                        .unwrap()
                        .ids
                        .iter()
                        .map(|c| [c.x, c.z])
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
                    equip(&mut world, id, 1, "theia.dev.training-sword");
                    id
                })
                .collect::<Vec<_>>()
        } else {
            players.clone()
        };
        let player_lives: Vec<_> = players
            .iter()
            .map(|&id| world.player_life_snapshot(id).unwrap().key)
            .collect();
        let initial_targets = world.combat_snapshot(players[0]).unwrap().targets;
        let positions: Vec<_> = keys
            .iter()
            .map(|key| {
                initial_targets
                    .iter()
                    .find(|t| t.key == *key)
                    .unwrap()
                    .position
            })
            .collect();
        let mut targets = keys.clone();
        for generation in 1..=2 {
            for (&id, &target) in players.iter().zip(&targets) {
                assert!(world.select_target(id, Some(target)));
                if let Some(ability) = ability {
                    equip(
                        &mut world,
                        id,
                        generation * 2,
                        if ability.ends_with("arcane-bolt") {
                            "theia.dev.training-wand"
                        } else {
                            "theia.dev.training-sword"
                        },
                    );
                    let epoch = world.epoch();
                    let reply = world.cast(
                        id,
                        super::super::Request {
                            epoch,
                            sequence: generation,
                            target,
                            ability: ContentId::new(ability).unwrap(),
                        },
                    );
                    assert!(
                        matches!(reply.reply.outcome, super::super::Outcome::Started { .. }),
                        "{:?}",
                        reply.reply
                    );
                } else {
                    engage(&mut world, id, target, generation);
                }
            }
            let mut ability_hits = 0;
            let mut remaining = vec![100; 5];
            let mut last_hostility = BTreeMap::new();
            if ability.is_some() {
                for &id in &players {
                    assert!(!world.combat_membership(id).unwrap().in_combat());
                }
            }
            let mut deaths = 0;
            for _ in 0..u64::from(rate.hz()) * 60 {
                world.step();
                for event in world.combat_events().to_vec() {
                    let recipient = world.kill_recipient(&event).unwrap();
                    let is_ability = matches!(&event, Event::AbilityDamage(_));
                    let (actor, actor_generation, target, after, lost) = match event {
                        Event::PhysicalDamage(fact) => {
                            assert!(!ordinary);
                            let index = targets.iter().position(|&key| key == fact.target).unwrap();
                            let expected = profile
                                .weapon_result(
                                    &ContentId::new("theia.dev.training-sword").unwrap(),
                                    fact.roll,
                                    remaining[index],
                                )
                                .unwrap();
                            assert_eq!(fact.result, expected);
                            (
                                fact.actor,
                                fact.actor_generation,
                                fact.target,
                                fact.remaining,
                                expected.health_lost,
                            )
                        }
                        Event::WhiteMelee(fact) => {
                            assert!(ordinary);
                            let index = targets.iter().position(|&key| key == fact.target).unwrap();
                            let expected = ordinary_profile
                                .as_ref()
                                .unwrap()
                                .evaluate_outgoing(
                                    &ContentId::new("theia.dev.training-sword").unwrap(),
                                    fact.defender,
                                    fact.draws,
                                    remaining[index],
                                )
                                .unwrap();
                            assert_eq!(fact.result, expected);
                            (
                                fact.actor,
                                fact.actor_generation,
                                fact.target,
                                fact.remaining,
                                expected.health_lost,
                            )
                        }
                        Event::AbilityDamage(fact) => {
                            let index = targets.iter().position(|&key| key == fact.target).unwrap();
                            assert_eq!(fact.ability.as_str(), ability.unwrap());
                            let expected = ability_profile
                                .as_ref()
                                .unwrap()
                                .evaluate(
                                    &fact.ability,
                                    fact.defender,
                                    fact.draws,
                                    remaining[index],
                                )
                                .unwrap();
                            assert_eq!(fact.result, expected);
                            ability_hits += 1;
                            equip(
                                &mut world,
                                players[index],
                                generation * 2 + 1,
                                "theia.dev.training-sword",
                            );
                            if fact.remaining > 0 {
                                engage(&mut world, finishers[index], fact.target, generation);
                            }
                            (
                                fact.actor,
                                Some(fact.actor_generation),
                                fact.target,
                                fact.remaining,
                                expected.damage.health_lost,
                            )
                        }
                        _ => continue,
                    };
                    let index = targets
                        .iter()
                        .position(|&key| key == target)
                        .expect("Only current exact victims");
                    let expected_actor = if is_ability {
                        players[index]
                    } else {
                        finishers[index]
                    };
                    assert_eq!(actor, world.entity_key(expected_actor).unwrap());
                    assert_eq!(
                        actor_generation,
                        Some(
                            world
                                .player_life_snapshot(expected_actor)
                                .unwrap()
                                .key
                                .generation
                        )
                    );
                    assert_eq!(after, remaining[index] - lost);
                    remaining[index] = after;
                    last_hostility.insert(expected_actor, world.tick);
                    if ability.is_some() && lost > 0 {
                        assert_eq!(
                            world.creature_tag_owner_for(target),
                            world.entity_key(players[index])
                        );
                    }
                    if after == 0 {
                        deaths += 1;
                        if ability.is_some() {
                            let credit = world.creature_kill_credit_for(target).unwrap();
                            assert_eq!(credit.owner, world.entity_key(players[index]));
                            assert_eq!(
                                credit
                                    .contributions
                                    .iter()
                                    .map(|c| c.health_lost)
                                    .sum::<u64>(),
                                100
                            );
                            assert_eq!(
                                credit.contributions.len(),
                                if finishers[index] != players[index] {
                                    2
                                } else {
                                    1
                                }
                            );
                            assert_eq!(
                                recipient,
                                Some((world.entity_key(players[index]).unwrap(), target))
                            );
                        }
                    }
                }
                if ability.is_some() {
                    for (index, &id) in players
                        .iter()
                        .enumerate()
                        .chain(finishers.iter().enumerate())
                    {
                        let summary = world.combat_membership(id).unwrap();
                        assert_eq!(
                            summary.in_combat(),
                            last_hostility
                                .get(&id)
                                .is_some_and(|tick| world.tick < tick + u64::from(rate.hz()) * 5)
                                && remaining[index] > 0
                        );
                        assert_eq!(summary.opponents, usize::from(summary.in_combat()));
                        assert!(!summary.active_encounter && !summary.incoming_hostility);
                    }
                    assert!(world.combat_membership_error().is_none());
                }
                if deaths == 5 {
                    break;
                }
            }
            assert_eq!(deaths, 5);
            assert_eq!(ability_hits, if ability.is_some() { 5 } else { 0 });
            assert_eq!(remaining, vec![0; 5]);
            for _ in 0..=rate.hz() {
                world.step();
            }
            let snapshot = world.combat_snapshot(players[0]).unwrap();
            // Identity allocation follows death order, not spawn-point order.
            let fresh: Vec<_> = positions
                .iter()
                .map(|position| {
                    snapshot
                        .targets
                        .iter()
                        .find(|target| target.position == *position)
                        .unwrap()
                        .key
                })
                .collect();
            assert_eq!(fresh.len(), 5);
            assert!(fresh.iter().all(|key| !targets.contains(key)));
            for &old in &targets {
                assert!(world.creature_definition(old).is_none());
                assert!(world.creature_facing(old).is_none());
                assert!(world.creature_kill_credit_for(old).is_none());
                assert!(world.creature_credit_encounter_for(old).is_none());
                assert!(!world.select_target(players[0], Some(old)));
            }
            for &key in &fresh {
                assert!(world.creature_tag_owner_for(key).is_none());
                if ability.is_some() {
                    assert!(world.creature_credit_encounter_for(key).is_some());
                }
                assert_eq!(
                    world.creature_facing(key),
                    Some(theia_core::facing::Facing::default())
                );
            }
            targets = fresh;
            for (&id, &life) in players.iter().zip(&player_lives) {
                let current = world.player_life_snapshot(id).unwrap();
                assert_eq!(current.key, life);
                assert_eq!(current.health, 100);
            }
            assert!(world.physical_damage_error().is_none());
            assert!(world.white_melee_error().is_none());
            assert!(world.ability_damage_error().is_none());
            assert!(world.creature_credit_error().is_none());
            if ordinary {
                assert_eq!(world.white_melee_metrics().unwrap().incoming.attempts, 0);
            }
        }
    }
}
