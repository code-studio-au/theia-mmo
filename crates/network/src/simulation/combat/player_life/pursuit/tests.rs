//! Actual fixed-step controller/worker/retaliation integration on a checked
//! gentle-terrain triangulation fixture; not native or authenticated peers.
use super::*;
use crate::{
    simulation::{World, creatures},
    wire::Intent,
};
use std::{sync::Arc, thread, time::Duration};
use theia_core::pursuit::Mode;
const STATS: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-character-stats.json");
const CREATURE: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-creature-patrol.json");
const POLICY: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-creature-retaliation.json");
const RESPAWN: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-player-respawn.json");
fn fixture(rate: TickRate) -> (World, EntityKey) {
    let mut creature: serde_json::Value = serde_json::from_slice(CREATURE).unwrap();
    creature["aggro"] = serde_json::json!({"perception_metres":12.,"leash_metres":35.,"chase_speed":5.,"return_speed":5.,"los_grace_ms":2000,"engagement_ms":30000,"reacquire_ms":2000});
    let creature = serde_json::to_vec(&creature).unwrap();
    let (mut world, npc) = creatures::tests::fixture_profile_at(rate, [8000., 8000.], &creature);
    world.enable_prediction().unwrap();
    let mut abilities: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../../../content/gameplay/development-abilities.json"
    ))
    .unwrap();
    abilities["abilities"][0]["cast_ms"] = 0.into();
    abilities["abilities"][0]["effect"]["amount"] = 100.into();
    world
        .enable_resource_combat(&serde_json::to_vec(&abilities).unwrap(), 100, 1000, STATS)
        .unwrap();
    world
        .enable_player_life(Arc::new(
            super::super::Profile::from_json(STATS, &creature, POLICY, RESPAWN).unwrap(),
        ))
        .unwrap();
    (world, npc)
}
fn join(world: &mut World, x: f64) -> u64 {
    super::super::tests::join(world, [x, 8000.])
}
fn step(world: &mut World, actor: Option<(u64, [i16; 2])>) {
    world.poll_terrain();
    if let Some((id, axes)) = actor
        && world.player_alive(id)
    {
        let player = &world.players[&id];
        let state = player.commands.as_ref().unwrap().state();
        let commands = crate::prediction_wire::Commands {
            epoch: world.epoch(),
            stream_epoch: state.stream_epoch,
            ready_sequence: player.ready_sequence,
            commands: vec![crate::commands::Command {
                sequence: state.applied_sequence + 1,
                tick: world.tick + 1,
                intent: Intent {
                    axes,
                    ..Default::default()
                },
            }],
        };
        assert_eq!(world.command(id, &commands), 1);
    }
    world.step();
    assert!(
        world.player_life_error().is_none(),
        "{:?}",
        world.player_life_error()
    );
    thread::sleep(Duration::from_millis(2));
}
#[test]
fn passive_perception_real_chase_melee_leash_return_and_saved_patrol_at_every_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, _) = fixture(rate);
        let actor = join(&mut world, 8018.);
        let original = world.patrol_snapshot().unwrap();
        let mut attacked = false;
        for _ in 0..rate.hz() * 5 {
            step(&mut world, Some((actor, [0, 0])));
            if world.player_life_snapshot(actor).unwrap().health < 100 {
                attacked = true;
                break;
            }
        }
        assert!(attacked, "{:?}", world.creature_pursuit_directive());
        let metrics = world.creature_pursuit_metrics().unwrap();
        assert_eq!(metrics.acquisitions, 1);
        assert!(metrics.chase_ticks > 0 && metrics.attack_ticks > 0 && metrics.published > 0);
        let anchor = metrics.last_anchor.unwrap();
        assert!((anchor[0] - original.position[0]).abs() < 0.001);
        assert!(world.patrol_snapshot().unwrap().position[0] > original.position[0] + 5.);
        let mut returned = false;
        for _ in 0..rate.hz() * 9 {
            step(&mut world, Some((actor, [1000, 0])));
            returned |= world.creature_pursuit_directive().unwrap().mode == Mode::Return;
        }
        assert!(returned);
        let strikes = world.player_life_metrics().unwrap().strikes;
        for _ in 0..rate.hz() * 12 {
            step(&mut world, Some((actor, [0, 0])));
            if world.creature_pursuit_metrics().unwrap().arrivals > 0 {
                break;
            }
        }
        let metrics = world.creature_pursuit_metrics().unwrap();
        assert_eq!(metrics.returns, 1);
        assert_eq!(metrics.arrivals, 1);
        assert_eq!(world.player_life_metrics().unwrap().strikes, strikes);
        let resumed = world.patrol_snapshot().unwrap();
        assert_eq!(resumed.waypoint, original.waypoint);
        assert!((resumed.position[0] - anchor[0]).hypot(resumed.position[2] - anchor[2]) < 0.1);
        for _ in 0..rate.hz() * 2 {
            step(&mut world, Some((actor, [0, 0])));
        }
        assert!(world.patrol_snapshot().unwrap().position[0] > anchor[0] + 1.);
    }
}
#[test]
fn departure_during_chase_clears_attack_and_returns_without_following_old_jobs() {
    let (mut world, _) = fixture(TickRate::Hz30);
    let actor = join(&mut world, 8018.);
    step(&mut world, Some((actor, [0, 0])));
    assert_eq!(
        world.creature_pursuit_directive().unwrap().mode,
        Mode::Chase
    );
    let old = world.player_life_snapshot(actor).unwrap().key;
    world.leave(actor);
    assert!(world.global_position(actor).is_none());
    for _ in 0..150 {
        step(&mut world, None);
    }
    let directive = world.creature_pursuit_directive().unwrap();
    assert_ne!(directive.target, Some(old));
    assert_eq!(directive.mode, Mode::Patrol);
    assert_eq!(world.creature_pursuit_metrics().unwrap().arrivals, 1);
    assert_eq!(world.player_life_metrics().unwrap().strikes, 0);
}

#[test]
fn static_occlusion_prevents_passive_aggro_until_sight_is_restored() {
    use crate::physics::{PlacedCollider, SOLID, StaticScene};
    use avian3d::prelude::Collider;
    use bevy::prelude::Vec3;
    let (mut world, _) = fixture(TickRate::Hz30);
    let actor = join(&mut world, 8018.);
    let original = world.patrol.as_ref().unwrap().patch.clone();
    let ground = original.try_spawn(8013., 8000.).unwrap();
    let p = original.frame.to_local(ground).unwrap().metres();
    let mut colliders = original.scene.colliders.clone();
    colliders.push(PlacedCollider {
        shape: Collider::cuboid(0.5, 8., 8.),
        center: Vec3::new(p[0], p[1] + 4., p[2]),
        layer: SOLID,
    });
    let blocked = Arc::new(crate::procedural::Patch {
        owner: original.owner,
        frame: original.frame,
        ids: original.ids.clone(),
        scene: StaticScene::new(colliders),
    });
    world
        .procedural
        .as_mut()
        .unwrap()
        .patches
        .insert(original.owner, blocked.clone());
    world.patrol.as_mut().unwrap().patch = blocked;
    step(&mut world, Some((actor, [0, 0])));
    assert_eq!(world.creature_pursuit_metrics().unwrap().acquisitions, 0);
    world
        .procedural
        .as_mut()
        .unwrap()
        .patches
        .insert(original.owner, original.clone());
    world.patrol.as_mut().unwrap().patch = original;
    for _ in 0..5 {
        step(&mut world, Some((actor, [0, 0])));
    }
    assert_eq!(world.creature_pursuit_metrics().unwrap().acquisitions, 1);
}

#[test]
fn unavailable_navigation_cancels_chase_without_a_direct_line_fallback() {
    let (mut world, _) = fixture(TickRate::Hz30);
    let patch = world.patrol.as_ref().unwrap().patch.clone();
    let query = creatures::tests::query_fixture(&world, &patch, 80.);
    let runtime = world
        .combat
        .as_mut()
        .unwrap()
        .pve
        .as_mut()
        .unwrap()
        .pursuit
        .as_mut()
        .unwrap();
    runtime.worker = Worker::spawn(query, runtime.worker.navigation()).unwrap();
    let actor = join(&mut world, 8018.);
    let anchor = world.patrol_snapshot().unwrap().position;
    for _ in 0..60 {
        step(&mut world, Some((actor, [0, 0])));
        if world.creature_pursuit_metrics().unwrap().arrivals > 0 {
            break;
        }
    }
    let metrics = world.creature_pursuit_metrics().unwrap();
    assert_eq!(metrics.failed, 1);
    assert_eq!(metrics.published, 0);
    assert_eq!(metrics.returns, 1);
    assert_eq!(metrics.arrivals, 1);
    let p = world.patrol_snapshot().unwrap().position;
    assert!((p[0] - anchor[0]).hypot(p[2] - anchor[2]) < 0.001);
    assert_eq!(world.player_life_metrics().unwrap().strikes, 0);
}

#[test]
fn creature_death_respawn_and_explicit_despawn_retire_old_path_identity() {
    use crate::simulation::combat::{Outcome, Request as Cast};
    use theia_core::content::ContentId;
    let (mut world, npc) = fixture(TickRate::Hz30);
    let actor = join(&mut world, 8018.);
    step(&mut world, Some((actor, [0, 0])));
    let cast = world.cast(
        actor,
        Cast {
            epoch: world.epoch(),
            sequence: 1,
            target: npc,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
        },
    );
    assert!(matches!(cast.reply.outcome, Outcome::Started { .. }));
    // Complete death while the original path may still be pending/unpolled.
    world.step();
    step(&mut world, Some((actor, [0, 0])));
    assert_eq!(world.creature_pursuit_directive().unwrap().mode, Mode::Dead);
    for _ in 0..90 {
        step(&mut world, Some((actor, [0, 0])));
        if world.creature_pursuit_metrics().unwrap().acquisitions >= 2 {
            break;
        }
    }
    let fresh = world.patrol.as_ref().unwrap().key;
    assert_ne!(fresh, npc);
    assert_eq!(world.creature_pursuit_metrics().unwrap().acquisitions, 2);
    let runtime = world
        .combat
        .as_ref()
        .unwrap()
        .pve
        .as_ref()
        .unwrap()
        .pursuit
        .as_ref()
        .unwrap();
    if let Some(request) = runtime.worker.pending() {
        assert_eq!(request.tag.creature, fresh);
    }
    assert!(world.remove_development_target(fresh));
    for _ in 0..5 {
        step(&mut world, Some((actor, [0, 0])));
    }
    assert_eq!(world.creature_pursuit_directive().unwrap().mode, Mode::Dead);
    assert_eq!(world.player_life_snapshot(actor).unwrap().health, 100);
}

#[test]
fn target_death_and_new_life_never_inherit_old_pursuit_or_windup() {
    let (mut world, _) = fixture(TickRate::Hz30);
    let actor = join(&mut world, 8018.);
    let old = world.player_life_snapshot(actor).unwrap().key;
    let mut died = false;
    for _ in 0..30 * 75 {
        step(&mut world, Some((actor, [0, 0])));
        if !world.player_alive(actor) {
            died = true;
            break;
        }
    }
    assert!(died);
    step(&mut world, None);
    assert_ne!(
        world.creature_pursuit_directive().unwrap().target,
        Some(old)
    );
    assert!(world.player_life_snapshot(actor).unwrap().windup.is_none());
    for _ in 0..100 {
        step(&mut world, None);
        if world.player_alive(actor) {
            break;
        }
    }
    let fresh = world.player_life_snapshot(actor).unwrap();
    assert_ne!(fresh.key, old);
    assert_eq!(fresh.health, 100);
    assert_ne!(
        world.creature_pursuit_directive().unwrap().target,
        Some(old)
    );
    assert!(
        fresh
            .windup
            .is_none_or(|w| w.target == fresh.key && w.completes > world.tick)
    );
}

#[test]
fn baked_height_bias_does_not_stall_a_reached_waypoint_or_skip_another_floor() {
    let p = GlobalPosition::new([15040., 25., 15040.]).unwrap();
    assert!(waypoint_reached(
        GlobalPosition::new([15040.04, 25.2, 15040.03]).unwrap(),
        p
    ));
    assert!(!waypoint_reached(
        GlobalPosition::new([15040., 26.8, 15040.]).unwrap(),
        p
    ));
    assert!(!waypoint_reached(
        GlobalPosition::new([15040.11, 25., 15040.]).unwrap(),
        p
    ));
}

#[test]
fn stationary_attack_facing_tracks_the_current_target_without_moving_the_creature() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture(rate);
        let home_x = world.patrol_snapshot().unwrap().position[0];
        let player = join(&mut world, home_x + 1.);
        for _ in 0..10 {
            step(&mut world, Some((player, [0, 0])));
            if world.creature_pursuit_directive().unwrap().mode == Mode::Attack {
                break;
            }
        }
        assert_eq!(
            world.creature_pursuit_directive().unwrap().mode,
            Mode::Attack
        );
        let start = world.patrol_snapshot().unwrap().position;
        let first = world.patrol_facing(npc).unwrap();
        for _ in 0..rate.hz() / 4 {
            step(&mut world, Some((player, [0, 1000])));
            assert_eq!(
                world.creature_pursuit_directive().unwrap().mode,
                Mode::Attack
            );
            let position = GlobalPosition::new(world.patrol_snapshot().unwrap().position).unwrap();
            let expected = theia_core::facing::Facing::towards(
                position,
                world.global_position(player).unwrap(),
            )
            .unwrap();
            assert!(world.patrol_facing(npc).unwrap().difference(expected).abs() < 0.0001);
            let after = position.metres();
            assert!((after[0] - start[0]).hypot(after[2] - start[2]) < 0.00001);
        }
        assert!(first.difference(world.patrol_facing(npc).unwrap()).abs() > 0.3);
    }
}

#[test]
fn ongoing_melee_does_not_reset_at_the_original_thirty_second_deadline() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, _) = fixture(rate);
        let home_x = world.patrol_snapshot().unwrap().position[0];
        let actor = join(&mut world, home_x + 1.);
        for _ in 0..u64::from(rate.hz()) * 35 {
            step(&mut world, Some((actor, [0, 0])));
            assert!(world.player_alive(actor));
            assert_eq!(
                world.creature_pursuit_directive().unwrap().mode,
                Mode::Attack
            );
        }
        assert_eq!(world.creature_pursuit_metrics().unwrap().acquisitions, 1);
        assert!(world.player_life_metrics().unwrap().strikes > 0);
    }
}
