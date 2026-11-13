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
    qualify(false, None, false, false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_ordinary_swings_resolve_each_life_independently_at_all_rates() {
    qualify(true, None, false, false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_special_damage_resolves_each_life_independently_at_all_rates() {
    qualify(true, Some("theia.dev.sword-slash"), false, false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_arcane_damage_resolves_each_life_independently_at_all_rates() {
    qualify(true, Some("theia.dev.arcane-bolt"), false, false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_competing_finishers_preserve_each_first_tag_at_all_rates() {
    qualify(true, Some("theia.dev.sword-slash"), true, false);
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
fn qualify(ordinary: bool, ability: Option<&str>, competing: bool, retaliating: bool) {
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
    let stats = if retaliating {
        let mut value: serde_json::Value = serde_json::from_slice(STATS).unwrap();
        // Keep long lifecycle checks independent of prototype encounter balance.
        value["maximum_health"] = 1000.into();
        serde_json::to_vec(&value).unwrap()
    } else {
        STATS.to_vec()
    };
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
                &stats,
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
        assert!(world.enable_public_avatar_life().is_err());
        assert_eq!(world.identity(), before);
        let life = crate::player_life_wire::Profile::from_json(
            &stats,
            CREATURE,
            include_bytes!(
                "../../../../../../content/gameplay/development-creature-retaliation.json"
            ),
            include_bytes!("../../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap();
        if retaliating {
            world.enable_population_player_life(Arc::new(life)).unwrap();
        } else {
            world
                .enable_passive_population_player_life(Arc::new(life))
                .unwrap();
        }
        let before_public = world.identity();
        world.enable_public_avatar_life().unwrap();
        assert_ne!(world.identity(), before_public);
        let installed_public = world.identity();
        assert!(world.enable_public_avatar_life().is_err());
        assert_eq!(world.identity(), installed_public);
        assert!(
            world
                .public_avatar_life_snapshot()
                .unwrap()
                .unwrap()
                .actors
                .is_empty()
        );
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
            assert!(
                world
                    .enable_creature_reset(include_bytes!(
                        "../../../../../../content/gameplay/development-creature-return-reset.json"
                    ))
                    .is_err()
            );
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
        if competing {
            let items = crate::renet_runtime::inventory::Profile::from_json(ITEMS).unwrap();
            let rewards =
                include_bytes!("../../../../../../content/gameplay/development-rewards.json");
            let mut wrong: serde_json::Value = serde_json::from_slice(rewards).unwrap();
            wrong["rewards"][0]["creature"] = "theia.dev.other-creature".into();
            let wrong = Arc::new(
                crate::reward_wire::Profile::from_json(
                    &serde_json::to_vec(&wrong).unwrap(),
                    &items,
                )
                .unwrap(),
            );
            let before = world.identity();
            assert!(world.enable_rewards(wrong).is_err());
            assert_eq!(world.identity(), before);
            let profile =
                Arc::new(crate::reward_wire::Profile::from_json(rewards, &items).unwrap());
            world.enable_rewards(profile.clone()).unwrap();
            let installed = world.identity();
            assert_ne!(installed, before);
            assert!(world.enable_rewards(profile).is_err());
            assert_eq!(world.identity(), installed);
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
        if retaliating {
            check_retaliators(&mut world, &players, &keys, rate);
            continue;
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
        let initial_public = world.public_avatar_life_snapshot().unwrap().unwrap();
        assert_eq!(
            initial_public.actors.len(),
            if competing {
                crate::wire::MAX_PLAYERS
            } else {
                players.len()
            }
        );
        assert!(initial_public.actors.iter().all(|actor| !actor.dead));
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
            let mut fatal_events = Vec::new();
            for _ in 0..u64::from(rate.hz()) * 60 {
                world.step();
                for event in world.combat_events().to_vec() {
                    let recipient = world.kill_recipient(&event).unwrap();
                    if competing {
                        // Repeated facts within a batch must enqueue one opportunity.
                        let offers = world
                            .reward_opportunities(&[event.clone(), event.clone()])
                            .unwrap();
                        if let Some((owner, victim)) = recipient {
                            assert_eq!(offers.len(), 1);
                            assert_eq!(offers[0].actor, owner);
                            assert_eq!(
                                offers[0].event.id,
                                theia_core::progression::KillId::from_victim(victim)
                            );
                            let index = targets.iter().position(|&key| key == victim).unwrap();
                            assert_eq!(offers[0].position, positions[index]);
                            fatal_events.push(event.clone());
                        } else {
                            assert!(offers.is_empty());
                        }
                    }
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
            if competing {
                assert_eq!(fatal_events.len(), 5);
                assert!(
                    world
                        .reward_opportunities(&fatal_events)
                        .unwrap()
                        .is_empty()
                );
            }
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
        let player = players[0];
        let old = world.player_life_snapshot(player).unwrap().key;
        let now = world.tick;
        world
            .combat
            .as_mut()
            .unwrap()
            .actors
            .get_mut(&player)
            .unwrap()
            .life
            .as_mut()
            .unwrap()
            .state
            .damage(now, old, 100)
            .unwrap();
        let dead = world.public_avatar_life_snapshot().unwrap().unwrap();
        assert_eq!(dead.actors.iter().filter(|actor| actor.dead).count(), 1);
        assert_eq!(
            dead.actors.iter().find(|actor| actor.dead).unwrap().key,
            old
        );
        let respawn = world
            .player_life_snapshot(player)
            .unwrap()
            .respawn_at
            .unwrap();
        while world.tick < respawn {
            world.step();
        }
        let fresh = world.public_avatar_life_snapshot().unwrap().unwrap();
        assert!(fresh.actors.iter().all(|actor| !actor.dead));
        for actor in &fresh.actors {
            let initial = initial_public
                .actors
                .iter()
                .find(|initial| initial.key.actor == actor.key.actor)
                .unwrap();
            if actor.key.actor == old.actor {
                assert_eq!(actor.key.generation.get(), old.generation.get() + 1);
            } else {
                assert_eq!(actor, initial);
            }
        }
        let profile = world.public_avatar_life_profile().unwrap();
        let bytes = crate::avatar_life_wire::encode(profile, rate, &fresh).unwrap();
        assert_eq!(
            crate::avatar_life_wire::decode(profile, fresh.epoch, rate, &bytes).unwrap(),
            fresh
        );
        assert_eq!(
            bytes.len(),
            crate::avatar_life_wire::HEADER_BYTES
                + fresh.actors.len() * crate::avatar_life_wire::ACTOR_BYTES
        );
        world.leave(player);
        let left = world.public_avatar_life_snapshot().unwrap().unwrap();
        assert_eq!(left.actors.len(), fresh.actors.len() - 1);
        assert!(left.actors.iter().all(|actor| actor.key.actor != old.actor));
    }
}

#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_physical_retaliation_keeps_each_engagement_independent() {
    qualify(false, None, false, true);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_ordinary_retaliation_keeps_each_engagement_independent() {
    qualify(true, None, false, true);
}
fn check_retaliators(world: &mut World, players: &[u64], keys: &[EntityKey], rate: TickRate) {
    let maximum = world
        .player_life_snapshot(players[0])
        .unwrap()
        .maximum_health;
    for (&id, &npc) in players[..2].iter().zip(keys) {
        engage(world, id, npc, 1);
    }
    let mut stopped = [false; 2];
    let mut struck = std::collections::BTreeSet::new();
    for _ in 0..u64::from(rate.hz()) * 10 {
        world.step();
        for i in 0..2 {
            let snapshot = world.combat_snapshot(players[i]).unwrap();
            let health = snapshot
                .targets
                .iter()
                .find(|t| t.key == keys[i])
                .unwrap()
                .current;
            if health < 100 && !stopped[i] {
                let reply = world.request_autoattack(
                    players[i],
                    autoattack::Request {
                        epoch: world.epoch(),
                        stream_epoch: world
                            .correction(players[i])
                            .unwrap()
                            .state
                            .input
                            .stream_epoch,
                        sequence: 2,
                        target: None,
                    },
                );
                assert_eq!(reply.outcome, autoattack::Outcome::Stopped);
                stopped[i] = true;
            }
        }
        for event in world.combat_events() {
            let pair = match event {
                Event::PhysicalDamage(f) if f.actor_generation.is_none() => {
                    Some((f.actor, f.target))
                }
                Event::WhiteMelee(f) if f.actor_generation.is_none() => Some((f.actor, f.target)),
                _ => None,
            };
            if let Some((npc, actor)) = pair {
                let index = keys.iter().position(|k| *k == npc).unwrap();
                assert!(index < 2, "Unprovoked bystander retaliated");
                assert_eq!(actor, world.entity_key(players[index]).unwrap());
                struck.insert(npc.entity.value());
            }
        }
        for &id in players {
            let life = world.player_life_snapshot(id).unwrap();
            assert_eq!(
                world.combat_snapshot(id).unwrap().vitals.unwrap().health(),
                life.health
            );
        }
        if stopped.iter().all(|v| *v)
            && struck.len() == 2
            && players[..2]
                .iter()
                .all(|id| world.player_life_snapshot(*id).unwrap().health < maximum)
        {
            break;
        }
    }
    assert!(stopped.iter().all(|v| *v));
    assert_eq!(struck.len(), 2);
    for &id in &players[..2] {
        let life = world.player_life_snapshot(id).unwrap();
        assert!(life.health > 0 && life.health < maximum);
    }
    for &id in &players[2..] {
        assert_eq!(world.player_life_snapshot(id).unwrap().health, maximum);
    }
    let departed = world.entity_key(players[0]).unwrap();
    world.leave(players[0]);
    for _ in 0..u64::from(rate.hz()) * 2 {
        world.step();
        for event in world.combat_events() {
            match event {
                Event::PhysicalDamage(f) => assert_ne!(f.target, departed),
                Event::WhiteMelee(f) => assert_ne!(f.target, departed),
                _ => {}
            }
        }
    }
    // Actual weapon damage kills one NPC; its old engagement must not survive respawn.
    engage(world, players[1], keys[1], 3);
    let mut defeated = false;
    for _ in 0..u64::from(rate.hz()) * 60 {
        world.step();
        if world
            .combat_snapshot(players[1])
            .unwrap()
            .targets
            .iter()
            .any(|t| t.key == keys[1] && t.current == 0)
        {
            defeated = true;
            break;
        }
    }
    assert!(
        defeated,
        "Retaliating NPC never died through ordinary weapon damage"
    );
    let remaining = world.player_life_snapshot(players[1]).unwrap().health;
    assert!(remaining > 0);
    let position = world
        .combat_snapshot(players[1])
        .unwrap()
        .targets
        .iter()
        .find(|t| t.key == keys[1])
        .unwrap()
        .position;
    for _ in 0..u64::from(rate.hz()) * 2 {
        world.step();
    }
    assert_eq!(
        world.player_life_snapshot(players[1]).unwrap().health,
        remaining
    );
    let fresh = world
        .combat_snapshot(players[1])
        .unwrap()
        .targets
        .iter()
        .find(|t| t.position == position && t.current == 100)
        .unwrap()
        .key;
    assert_ne!(fresh, keys[1]);
    assert!(world.creature_definition(keys[1]).is_none());
    engage(world, players[1], fresh, 4);
    let mut fresh_strike = false;
    for _ in 0..u64::from(rate.hz()) * 10 {
        world.step();
        for event in world.combat_events() {
            let npc = match event {
                Event::PhysicalDamage(f) if f.actor_generation.is_none() => Some(f.actor),
                Event::WhiteMelee(f) if f.actor_generation.is_none() => Some(f.actor),
                _ => None,
            };
            if let Some(npc) = npc {
                assert_eq!(npc, fresh);
                fresh_strike = true;
            }
        }
        if fresh_strike {
            break;
        }
    }
    assert!(
        fresh_strike,
        "Fresh creature did not establish its own retaliation"
    );
    assert!(world.player_life_error().is_none());
    assert!(world.player_life_metrics().unwrap().strikes >= 2);
}

#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_shared_pursuit_chases_attacks_and_returns_each_home_at_all_rates() {
    qualify_shared_population_pursuit(false);
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_population_return_reset_is_independent_and_preserves_fresh_lives_at_all_rates() {
    qualify_shared_population_pursuit(true);
}
fn qualify_shared_population_pursuit(reset: bool) {
    use theia_core::pursuit::Mode;
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
    let mut creature: serde_json::Value = serde_json::from_slice(CREATURE).unwrap();
    creature["aggro"] = serde_json::json!({"perception_metres":30.,"leash_metres":50.,"chase_speed":5.,"return_speed":5.,"los_grace_ms":2000,"engagement_ms":30000,"reacquire_ms":2000});
    let creature = serde_json::to_vec(&creature).unwrap();
    let definition = creatures::Definition::from_json(&creature).unwrap();
    let mut stats: serde_json::Value = serde_json::from_slice(STATS).unwrap();
    stats["maximum_health"] = 1000.into();
    let stats = serde_json::to_vec(&stats).unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let prepared = spawn_preparation::check_authored(&store, &nav, &request)
            .unwrap()
            .into_prepared()
            .unwrap();
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
            .enable_navigating_population(
                prepared,
                Binding::new(definition.id(), definition.clone(), 1).unwrap(),
                navigation::worker::Worker::load(&nav).unwrap(),
            )
            .unwrap();
        let homes: Vec<_> = keys
            .iter()
            .map(|k| world.targets[&k.entity.value()].position)
            .collect();
        world
            .enable_status_combat(
                include_bytes!(
                    "../../../../../../content/gameplay/development-classic-abilities.json"
                ),
                100,
                1000,
                &stats,
                include_bytes!("../../../../../../content/gameplay/development-statuses.json"),
            )
            .unwrap();
        world.enable_autoattack(WEAPONS, ITEMS).unwrap();
        let life = crate::player_life_wire::Profile::from_json(
            &stats,
            &creature,
            include_bytes!(
                "../../../../../../content/gameplay/development-creature-retaliation.json"
            ),
            include_bytes!("../../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap();
        world.enable_population_player_life(Arc::new(life)).unwrap();
        world
            .enable_physical_damage(DEFINITIONS, DAMAGE_STATS, ITEMS)
            .unwrap();
        world
            .enable_white_melee(include_bytes!(
                "../../../../../../content/gameplay/development-white-melee.json"
            ))
            .unwrap();
        world
            .enable_ability_damage(
                include_bytes!(
                    "../../../../../../content/gameplay/development-ability-starting.json"
                ),
                include_bytes!(
                    "../../../../../../content/gameplay/development-ability-damage.json"
                ),
                include_bytes!(
                    "../../../../../../content/gameplay/development-classic-abilities.json"
                ),
            )
            .unwrap();
        world
            .enable_combat_membership(include_bytes!(
                "../../../../../../content/gameplay/development-combat-membership.json"
            ))
            .unwrap();
        world
            .enable_creature_credit(include_bytes!(
                "../../../../../../content/gameplay/development-creature-credit-solo.json"
            ))
            .unwrap();
        if reset {
            world
                .enable_creature_reset(include_bytes!(
                    "../../../../../../content/gameplay/development-creature-return-reset.json"
                ))
                .unwrap();
        }
        let encounters: Vec<_> = keys
            .iter()
            .map(|k| world.creature_credit_encounter_for(*k).unwrap())
            .collect();
        let mut body = request.access_from;
        body[1] += f64::from(crate::world::PAWN_Y);
        let actor = world
            .join_restored(GlobalPosition::new(body).unwrap())
            .unwrap();
        let chunks = world
            .procedural()
            .unwrap()
            .patch(world.global_position(actor).unwrap())
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
        let mut moved = std::collections::BTreeSet::new();
        for _ in 0..rate.hz() * 8 {
            world.poll_terrain();
            world.step();
            assert!(
                world.player_life_error().is_none(),
                "{:?}",
                world.player_life_error()
            );
            for (key, home) in keys.iter().zip(&homes) {
                let a = world.targets[&key.entity.value()].position.metres();
                let b = home.metres();
                if (a[0] - b[0]).hypot(a[2] - b[2]) > 1. {
                    moved.insert(key.entity.value());
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            moved.len() >= 2,
            "{rate:?}: only {} creatures chased",
            moved.len()
        );
        assert!(world.player_life_snapshot(actor).unwrap().health < 1000);
        assert!(world.combat_membership(actor).unwrap().in_combat());
        assert!(world.combat_membership_error().is_none());
        assert!(world.creature_credit_error().is_none());
        if reset {
            let pair: Vec<_> = keys
                .iter()
                .copied()
                .filter(|key| moved.contains(&key.entity.value()))
                .take(2)
                .collect();
            let source = world.entity_key(actor).unwrap();
            let now = world.tick;
            let mut combat = world.combat.take().unwrap();
            // Trusted fixture damage/effects isolate per-target reset semantics;
            // the later kill still uses ordinary equipped weapon requests.
            for (index, target) in pair.iter().copied().enumerate() {
                combat.apply_damage(
                    &mut world,
                    source,
                    target,
                    20 + index as u32 * 10,
                    |amount, remaining, position| Event::Damage {
                        ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                        actor: source,
                        target,
                        sequence: index as u64 + 1,
                        amount,
                        remaining,
                        position,
                    },
                );
                let cast = super::super::Cast {
                    sequence: index as u64 + 1,
                    ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                    target,
                    started: now,
                    completes: now,
                };
                combat
                    .statuses
                    .as_mut()
                    .unwrap()
                    .apply(
                        now,
                        rate,
                        source,
                        &cast,
                        &ContentId::new("theia.dev.scorch").unwrap(),
                    )
                    .unwrap();
            }
            world.combat = Some(combat);
            world
                .population_motion
                .iter_mut()
                .find(|p| p.key == pair[0])
                .unwrap()
                .blocked = true;
            world.step();
            assert_eq!(
                world.creature_pursuit_directive_for(pair[0]).unwrap().mode,
                Mode::Return
            );
            let health = |world: &World, key| {
                world
                    .combat
                    .as_ref()
                    .unwrap()
                    .creatures
                    .iter()
                    .find(|c| c.key == key)
                    .unwrap()
                    .current
            };
            assert_eq!(health(&world, pair[0]), 100);
            assert_eq!(health(&world, pair[1]), 70);
            let statuses = world.combat_status_snapshot(actor).unwrap();
            assert_eq!(statuses.len(), 1);
            assert_eq!(statuses[0].target, pair[1]);
            let metrics = world.creature_reset_metrics().unwrap();
            assert_eq!(
                (
                    metrics.resets,
                    metrics.health_restored,
                    metrics.statuses_removed
                ),
                (1, 20, 1)
            );
            let reply = world.cast(
                actor,
                super::super::Request {
                    epoch: world.epoch(),
                    sequence: 1,
                    target: pair[0],
                    ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
                },
            );
            assert!(
                matches!(
                    reply.reply.outcome,
                    super::super::Outcome::Rejected(super::super::Reason::NotReady)
                ),
                "{:?}",
                reply.reply
            );
            world.step();
            assert_eq!(world.creature_reset_metrics().unwrap(), metrics);
            assert_eq!(health(&world, pair[1]), 70);
        }
        world.leave(actor);
        for _ in 0..rate.hz() * 10 {
            world.poll_terrain();
            world.step();
            assert!(
                world.player_life_error().is_none(),
                "{:?}",
                world.player_life_error()
            );
            if keys
                .iter()
                .all(|k| world.creature_pursuit_directive_for(*k).unwrap().mode == Mode::Patrol)
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        for (key, home) in keys.iter().zip(&homes) {
            let directive = world.creature_pursuit_directive_for(*key).unwrap();
            assert_eq!(directive.mode, Mode::Patrol, "{rate:?} {key:?}");
            let a = world.targets[&key.entity.value()].position.metres();
            let b = home.metres();
            assert!(
                (a[0] - b[0]).hypot(a[2] - b[2]) < 0.2,
                "Creature did not return to its own home"
            );
        }
        for (key, old) in keys.iter().zip(&encounters) {
            if moved.contains(&key.entity.value()) {
                assert_ne!(world.creature_credit_encounter_for(*key).unwrap(), *old);
            }
        }
        assert!(world.combat_membership_error().is_none());
        assert!(world.creature_credit_error().is_none());
        if reset {
            let metrics = world.creature_reset_metrics().unwrap();
            assert_eq!(metrics.health_restored, 50);
            // The first target was reset in-world; departure already removes
            // the remaining source-owned effect before the other return.
            assert_eq!(metrics.statuses_removed, 1);
            assert!(world.creature_reset_error().is_none());
            assert!(
                world
                    .combat
                    .as_ref()
                    .unwrap()
                    .creatures
                    .iter()
                    .all(|c| c.current == c.maximum)
            );
        }
        // Kill through the ordinary weapon path after the return, then ensure
        // the replacement life owns a fresh controller and its original home.
        let hunter = world
            .join_restored(world.targets[&keys[0].entity.value()].position)
            .unwrap();
        let chunks = world
            .procedural()
            .unwrap()
            .patch(homes[0])
            .unwrap()
            .ids
            .iter()
            .map(|c| [c.x, c.z])
            .collect();
        assert!(world.client_ready(
            hunter,
            &ChunkReady {
                version: VERSION,
                epoch: world.epoch(),
                sequence: 1,
                chunks
            }
        ));
        equip(&mut world, hunter, 1, "theia.dev.training-sword");
        engage(&mut world, hunter, keys[0], 1);
        for _ in 0..rate.hz() * 45 {
            world.poll_terrain();
            world.step();
            if !world.targets.contains_key(&keys[0].entity.value()) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            !world.targets.contains_key(&keys[0].entity.value()),
            "Ordinary weapon failed: auto={:?}, life={:?}, credit={:?}, membership={:?}",
            world.autoattack_snapshot(hunter),
            world.player_life_snapshot(hunter),
            world.creature_credit_error(),
            world.combat_membership_error()
        );
        assert_eq!(
            world.creature_kill_credit_for(keys[0]).unwrap().owner,
            world.entity_key(hunter)
        );
        world.leave(hunter);
        for _ in 0..rate.hz() * 2 {
            world.poll_terrain();
            world.step();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let fresh = world.combat.as_ref().unwrap().creatures[0].key;
        assert_ne!(fresh, keys[0]);
        assert!(world.creature_pursuit_directive_for(keys[0]).is_none());
        assert_eq!(
            world.creature_pursuit_directive_for(fresh).unwrap().mode,
            Mode::Patrol
        );
        assert!(
            world
                .combat
                .as_ref()
                .unwrap()
                .pursuit_navigation
                .as_ref()
                .unwrap()
                .pending(keys[0])
                .is_none()
        );
        let a = world.targets[&fresh.entity.value()].position.metres();
        let b = homes[0].metres();
        assert!((a[0] - b[0]).hypot(a[2] - b[2]) < 0.2);
    }
}
