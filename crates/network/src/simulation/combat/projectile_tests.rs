use super::*;
use serde_json::{Value, json};
use std::sync::Arc;
use theia_core::{hit_outcomes::Table, time::TickRate};
const ID: &str = "theia.dev.arcane-bolt";
const STATS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-character-stats.json");
const CREATURE: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-creature-patrol.json");
fn fixture(rate: TickRate, miss: bool, life: bool, fast: bool) -> (World, u64, EntityKey) {
    let mut creature: Value = serde_json::from_slice(CREATURE).unwrap();
    creature["wait_ms"] = json!(60000);
    let creature = serde_json::to_vec(&creature).unwrap();
    let (mut world, target) =
        crate::simulation::creatures::tests::fixture_profile_at(rate, [8006.5, 8000.], &creature);
    world.enable_prediction().unwrap();
    let mut stats: Value = serde_json::from_slice(STATS).unwrap();
    stats["mana_regeneration_per_second"] = json!(0);
    let stats = serde_json::to_vec(&stats).unwrap();
    let mut data: Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-resource-abilities.json"
    ))
    .unwrap();
    data["schema"] = json!(6);
    let a = &mut data["abilities"][0];
    a["cast_ms"] = json!(if fast { 0 } else { 100 });
    a["mana_cost"] = json!(if fast { 1 } else { 25 });
    a["cooldown_ms"] = json!(if fast { 0 } else { 2000 });
    a["uses_global_cooldown"] = json!(!fast);
    a["requires_main_hand"] = json!("theia.dev.training-wand");
    a["projectile"] = json!({"speed_metres_per_second":5.,"maximum_distance_metres":45.,"lifetime_ms":10000,"target_radius_metres":0.2});
    if fast {
        a["range_metres"] = json!(3.);
        a["projectile"]["speed_metres_per_second"] = json!(0.5);
    }
    let table = if miss {
        Table::new(0, 10000, 0).unwrap()
    } else {
        Table::certain_hit()
    };
    a["hit_table"] =
        json!({"hit_bp":table.hit_bp(),"miss_bp":table.miss_bp(),"deflect_bp":table.deflect_bp()});
    world
        .enable_status_combat(
            &serde_json::to_vec(&data).unwrap(),
            100,
            5000,
            &stats,
            include_bytes!("../../../../../content/gameplay/development-statuses.json"),
        )
        .unwrap();
    if life {
        world
            .enable_player_life(Arc::new(
                crate::player_life_wire::Profile::from_json(
                    &stats,
                    &creature,
                    include_bytes!(
                        "../../../../../content/gameplay/development-creature-retaliation.json"
                    ),
                    include_bytes!(
                        "../../../../../content/gameplay/development-player-respawn.json"
                    ),
                )
                .unwrap(),
            ))
            .unwrap();
    }
    let actor = join(&mut world);
    (world, actor, target)
}
fn join(world: &mut World) -> u64 {
    let id = world.join().unwrap();
    let p = world.global_position(id).unwrap();
    let chunks = world
        .procedural
        .as_ref()
        .unwrap()
        .patch(p)
        .unwrap()
        .ids
        .iter()
        .map(|id| [id.x, id.z])
        .collect();
    assert!(world.client_ready(
        id,
        &crate::wire::ChunkReady {
            version: crate::wire::VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks
        }
    ));
    world.observe_combat_equipment(
        world.entity_key(id).unwrap(),
        Some((
            [8; 32],
            1,
            Some(ContentId::new("theia.dev.training-wand").unwrap()),
        )),
    );
    id
}
fn request(world: &World, target: EntityKey, sequence: u64) -> Request {
    Request {
        epoch: world.epoch(),
        sequence,
        ability: ContentId::new(ID).unwrap(),
        target,
    }
}
fn launch(world: &mut World, actor: u64, target: EntityKey, rate: TickRate) {
    let result = world.cast(actor, request(world, target, 1));
    assert!(matches!(result.reply.outcome, Outcome::Started { .. }));
    for _ in 0..rate.ticks_for(Duration::from_millis(100)).unwrap() {
        world.step();
    }
    assert!(matches!(
        world.combat_events(),
        [Event::ProjectileLaunched { .. }]
    ));
    assert_eq!(world.projectile_metrics().unwrap().launched, 1);
    assert_eq!(world.combat_outcome_metrics().unwrap().random_draws, 0);
    assert_eq!(
        world.combat_snapshot(actor).unwrap().targets[0].current,
        100
    );
}
fn no_resolution(world: &World) {
    let metrics = world.combat_outcome_metrics().unwrap();
    assert_eq!(metrics.random_draws, 0);
    assert_eq!(metrics.resolved, 0);
    assert!(world.combat.as_ref().unwrap().projectiles.is_empty());
}
#[test]
fn travel_delays_damage_or_miss_until_one_impact_and_survives_post_launch_unequip_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for miss in [false, true] {
            let (mut world, actor, target) = fixture(rate, miss, false, false);
            launch(&mut world, actor, target, rate);
            assert_eq!(
                world.combat_snapshot(actor).unwrap().vitals.unwrap().mana(),
                75
            );
            assert_eq!(
                world.cast(actor, request(&world, target, 1)).reply.outcome,
                Outcome::Ignored
            );
            world.observe_combat_equipment(world.entity_key(actor).unwrap(), None);
            let mut results = 0;
            for _ in 0..rate.hz() * 2 {
                world.step();
                for event in world.combat_events() {
                    if miss {
                        results += usize::from(matches!(
                            event,
                            Event::Avoided {
                                kind: Avoidance::Miss,
                                ..
                            }
                        ));
                    } else {
                        results += usize::from(matches!(event, Event::Damage { amount: 20, .. }));
                    }
                    assert!(!matches!(
                        event,
                        Event::Cancelled { .. } | Event::ProjectileEnded { .. }
                    ));
                }
            }
            assert_eq!(results, 1);
            assert_eq!(world.projectile_metrics().unwrap().impacts, 1);
            assert_eq!(world.combat_outcome_metrics().unwrap().random_draws, 1);
            assert_eq!(
                world.combat_snapshot(actor).unwrap().targets[0].current,
                if miss { 100 } else { 80 }
            );
            assert!(world.combat.as_ref().unwrap().projectiles.is_empty());
        }
    }
}
#[test]
fn source_departure_death_and_replaced_target_life_cancel_without_damage_or_randomness() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for action in [0, 1, 2] {
            let (mut world, actor, target) = fixture(rate, false, action == 1, false);
            launch(&mut world, actor, target, rate);
            if action == 0 {
                world.leave(actor);
            } else if action == 1 {
                let player = world
                    .combat
                    .as_mut()
                    .unwrap()
                    .actors
                    .get_mut(&actor)
                    .unwrap()
                    .life
                    .as_mut()
                    .unwrap();
                let key = player.state.key();
                player.state.damage(world.tick, key, 100).unwrap();
            } else {
                let health = &mut world.combat.as_mut().unwrap().creatures[0];
                health.current = 0;
                health.respawn_at = Some(world.tick + 1);
            }
            world.step();
            let reason = if action == 2 {
                ProjectileEnd::TargetGone
            } else {
                ProjectileEnd::SourceGone
            };
            assert!(
                world
                    .combat_events()
                    .iter()
                    .any(|e| matches!(e,Event::ProjectileEnded{reason:r,..} if *r==reason))
            );
            assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, 100);
            no_resolution(&world);
            if action == 2 {
                assert_ne!(world.combat.as_ref().unwrap().creatures[0].key, target);
            }
        }
    }
}
#[test]
fn homing_target_movement_exhausts_a_fixed_distance_without_drawing_or_damaging() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, actor, target) = fixture(rate, false, false, false);
        launch(&mut world, actor, target, rate);
        world.patrol = None;
        let old = world.combat.as_ref().unwrap().creatures[0]
            .position
            .metres();
        world.combat.as_mut().unwrap().creatures[0].position =
            GlobalPosition::new([old[0] + 100., old[1] + 100., old[2]]).unwrap();
        let mut ended = 0;
        for _ in 0..rate.hz() * 11 {
            world.step();
            ended += world
                .combat_events()
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        Event::ProjectileEnded {
                            reason: ProjectileEnd::Expired,
                            ..
                        }
                    )
                })
                .count();
        }
        assert_eq!(ended, 1);
        assert_eq!(world.projectile_metrics().unwrap().expired, 1);
        assert_eq!(
            world.combat_snapshot(actor).unwrap().targets[0].current,
            100
        );
        no_resolution(&world);
    }
}
#[test]
fn thin_static_obstruction_and_unavailable_collision_fail_closed_at_last_clear_position() {
    use crate::{
        physics::{PlacedCollider, SOLID, StaticScene},
        procedural::Patch,
    };
    use avian3d::prelude::Collider;
    use bevy::prelude::Vec3;
    for unavailable in [false, true] {
        let (mut world, actor, target) = fixture(TickRate::Hz30, false, false, false);
        launch(&mut world, actor, target, TickRate::Hz30);
        let first = world.combat.as_ref().unwrap().projectiles[0].clear_position;
        let mut goal = world.combat.as_ref().unwrap().creatures[0]
            .position
            .metres();
        goal[1] += 0.6;
        let delta = Vec3::from_array(std::array::from_fn(|i| {
            (goal[i] - first.metres()[i]) as f32
        }));
        let obstacle_offset = delta.normalize() * ((delta.length() - 0.2) * 0.5);
        let runtime = world.procedural.as_mut().unwrap();
        if unavailable {
            runtime.patches.clear();
        } else {
            for patch in runtime.patches.values_mut() {
                let center = patch.frame.to_local(first).unwrap().metres();
                let mut colliders = patch.scene.colliders.clone();
                colliders.push(PlacedCollider {
                    shape: Collider::sphere(0.01),
                    center: Vec3::from_array(center) + obstacle_offset,
                    layer: SOLID,
                });
                *patch = Arc::new(Patch {
                    owner: patch.owner,
                    frame: patch.frame,
                    ids: patch.ids.clone(),
                    scene: StaticScene::new(colliders),
                });
            }
        }
        let mut ended = None;
        for _ in 0..30 {
            world.step();
            for event in world.combat_events() {
                if let Event::ProjectileEnded {
                    reason, position, ..
                } = event
                {
                    ended = Some((*reason, *position));
                }
            }
        }
        let (reason, position) = ended.unwrap();
        assert_eq!(
            reason,
            if unavailable {
                ProjectileEnd::Unavailable
            } else {
                ProjectileEnd::Obstructed
            }
        );
        if !unavailable {
            let travelled = Vec3::from_array(std::array::from_fn(|i| {
                (position.metres()[i] - first.metres()[i]) as f32
            }))
            .length();
            assert!(travelled < obstacle_offset.length());
        }
        no_resolution(&world);
        assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, 100);
    }
}
#[test]
fn per_actor_and_global_capacity_are_reserved_before_any_additional_cost() {
    let (mut world, first, target) = fixture(TickRate::Hz20, false, false, true);
    let position = world.global_position(first).unwrap();
    let mut players = vec![first];
    for _ in 0..3 {
        let id = join(&mut world);
        world.players.get_mut(&id).unwrap().position = position;
        players.push(id);
    }
    for sequence in 1..=4 {
        for &actor in &players {
            let result = world.cast(actor, request(&world, target, sequence));
            assert!(matches!(result.reply.outcome, Outcome::Started { .. }));
        }
        world.step();
    }
    assert_eq!(world.combat.as_ref().unwrap().projectiles.len(), 16);
    for actor in players {
        assert_eq!(
            world.combat_snapshot(actor).unwrap().vitals.unwrap().mana(),
            96
        );
        let result = world.cast(actor, request(&world, target, 5));
        assert_eq!(result.reply.outcome, Outcome::Rejected(Reason::Unavailable));
        assert_eq!(
            world.combat_snapshot(actor).unwrap().vitals.unwrap().mana(),
            96
        );
    }
    assert_eq!(world.projectile_metrics().unwrap().capacity_refusals, 4);
    assert_eq!(world.combat_outcome_metrics().unwrap().random_draws, 0);
}
