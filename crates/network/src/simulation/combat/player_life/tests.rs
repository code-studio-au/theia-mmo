use super::*;
use crate::{
    simulation::{
        combat::{Outcome, Request},
        creatures,
    },
    wire::{ChunkReady, VERSION},
};
use serde_json::{Value, json};
use theia_core::{content::ContentId, time::TickRate};
const STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-character-stats.json");
const CREATURE: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-creature-patrol.json");
const POLICY: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-creature-retaliation.json");
const RESPAWN: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-player-respawn.json");
fn profile(fast: bool) -> Arc<Profile> {
    let mut policy: Value = serde_json::from_slice(POLICY).unwrap();
    let mut respawn: Value = serde_json::from_slice(RESPAWN).unwrap();
    if fast {
        policy["damage"] = json!(40);
        policy["windup_ms"] = json!(100);
        policy["recovery_ms"] = json!(100);
        respawn["delay_ms"] = json!(300);
    }
    Arc::new(
        Profile::from_json(
            STATS,
            CREATURE,
            &serde_json::to_vec(&policy).unwrap(),
            &serde_json::to_vec(&respawn).unwrap(),
        )
        .unwrap(),
    )
}
fn fixture(rate: TickRate, enable: bool) -> (World, EntityKey) {
    let (mut world, npc) = creatures::tests::fixture(rate);
    world.enable_prediction().unwrap();
    let mut abilities: Value = serde_json::from_slice(include_bytes!(
        "../../../../../../content/gameplay/development-abilities.json"
    ))
    .unwrap();
    // Explicit authority fixture: one ordinary spell hit arms retaliation without
    // killing the creature, and no cooldown slows contract setup.
    for ability in abilities["abilities"].as_array_mut().unwrap() {
        ability["cast_ms"] = json!(0);
        ability["cooldown_ms"] = json!(0);
        ability["uses_global_cooldown"] = json!(false);
        if ability["effect"]["kind"] == "damage" {
            ability["effect"]["amount"] = json!(1);
        }
    }
    abilities["queue_window_ms"] = json!(1000);
    let mut long = abilities["abilities"][0].clone();
    long["id"] = json!("theia.dev.long-bolt");
    long["cast_ms"] = json!(1000);
    long["cast_while_moving"] = json!(false);
    long["cooldown_ms"] = json!(5000);
    abilities["abilities"].as_array_mut().unwrap().push(long);
    world
        .enable_resource_combat(&serde_json::to_vec(&abilities).unwrap(), 100, 300, STATS)
        .unwrap();
    if enable {
        world.enable_player_life(profile(true)).unwrap();
    }
    (world, npc)
}
pub(super) fn join(world: &mut World, xz: [f64; 2]) -> u64 {
    let position = GlobalPosition::new([xz[0], 0., xz[1]]).unwrap();
    let patch = world.procedural.as_ref().unwrap().patch(position).unwrap();
    let mut p = patch.try_spawn(xz[0], xz[1]).unwrap().metres();
    p[1] += f64::from(crate::world::PAWN_Y);
    let id = world
        .join_restored(GlobalPosition::new(p).unwrap())
        .unwrap();
    ready(world, id, 1);
    id
}
fn ready(world: &mut World, id: u64, sequence: u64) {
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
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence,
            chunks
        }
    ));
}
fn hit(world: &mut World, actor: u64, npc: EntityKey, sequence: u64) {
    let reply = world.cast(
        actor,
        Request {
            epoch: world.epoch(),
            sequence,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target: npc,
        },
    );
    assert!(
        matches!(reply.reply.outcome, Outcome::Started { .. }),
        "{:?}",
        reply.reply
    );
}
fn dead(world: &mut World, id: u64) {
    for _ in 0..60 {
        world.step();
        assert!(
            world.player_life_error().is_none(),
            "{:?}",
            world.player_life_error()
        );
        if !world.player_alive(id) {
            return;
        }
    }
    panic!("Creature did not defeat its attacker");
}
#[test]
fn profile_is_exact_and_activation_checks_dependencies_without_mutating_identity() {
    let normal = Profile::from_json(STATS, CREATURE, POLICY, RESPAWN).unwrap();
    let mut changed = STATS.to_vec();
    changed.push(b' ');
    let changed = Profile::from_json(&changed, CREATURE, POLICY, RESPAWN).unwrap();
    assert_ne!(normal.hash(), changed.hash());
    let mut invalid: Value = serde_json::from_slice(POLICY).unwrap();
    invalid["creature"] = json!("theia.other.creature");
    assert!(
        Profile::from_json(
            STATS,
            CREATURE,
            &serde_json::to_vec(&invalid).unwrap(),
            RESPAWN
        )
        .is_err()
    );
    let (mut no_prediction, _) = creatures::tests::fixture(TickRate::Hz30);
    no_prediction
        .enable_resource_combat(
            include_bytes!("../../../../../../content/gameplay/development-abilities.json"),
            100,
            300,
            STATS,
        )
        .unwrap();
    let identity = no_prediction.identity();
    assert!(no_prediction.enable_player_life(profile(true)).is_err());
    assert_eq!(no_prediction.identity(), identity);
    no_prediction.enable_prediction().unwrap();
    no_prediction.enable_player_life(profile(true)).unwrap();
    let (mut multiple, _) = creatures::tests::fixture(TickRate::Hz30);
    multiple.spawn_development_target([8010., 8000.]).unwrap();
    multiple.enable_prediction().unwrap();
    multiple
        .enable_resource_combat(
            include_bytes!("../../../../../../content/gameplay/development-abilities.json"),
            100,
            300,
            STATS,
        )
        .unwrap();
    let identity = multiple.identity();
    assert!(multiple.enable_player_life(profile(true)).is_err());
    assert_eq!(multiple.identity(), identity);
    let (mut world, _) = fixture(TickRate::Hz30, false);
    let before = world.identity();
    assert!(world.enable_player_life(Arc::new(changed)).is_err());
    assert_eq!(world.identity(), before);
    let mut creature = CREATURE.to_vec();
    creature.push(b' ');
    let changed = Profile::from_json(STATS, &creature, POLICY, RESPAWN).unwrap();
    assert!(world.enable_player_life(Arc::new(changed)).is_err());
    assert_eq!(world.identity(), before);
    world.enable_player_life(profile(true)).unwrap();
    assert_ne!(world.identity(), before);
    let selected = world.identity();
    assert!(world.enable_player_life(profile(true)).is_err());
    assert_eq!(world.identity(), selected);
}
#[test]
fn actual_authority_retaliates_and_dead_input_casts_and_interactions_cannot_act() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture(rate, true);
        let actor = join(&mut world, [8006., 8000.]);
        hit(&mut world, actor, npc, 1);
        dead(&mut world, actor);
        let snapshot = world.player_life_snapshot(actor).unwrap();
        assert_eq!(snapshot.health, 0);
        assert_eq!(snapshot.movement_stream, 2);
        assert_eq!(snapshot.key.generation.get(), 1);
        assert_eq!(
            world
                .combat_snapshot(actor)
                .unwrap()
                .vitals
                .unwrap()
                .health(),
            0
        );
        let p = world.global_position(actor).unwrap();
        let before = world.correction(actor).unwrap();
        let packet = crate::prediction_wire::Commands {
            epoch: world.epoch(),
            stream_epoch: 2,
            ready_sequence: 1,
            commands: vec![crate::commands::Command {
                sequence: 1,
                tick: world.tick + 1,
                intent: Intent {
                    axes: [1000, 0],
                    jump: true,
                    ..Default::default()
                },
            }],
        };
        assert_eq!(world.command(actor, &packet), 0);
        let reply = world.cast(
            actor,
            Request {
                epoch: world.epoch(),
                sequence: 2,
                ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
                target: npc,
            },
        );
        assert_eq!(reply.reply.outcome, Outcome::Rejected(Reason::Interrupted));
        assert!(!world.guide_interaction_eligible(actor, 1, npc));
        world.step();
        let after = world.global_position(actor).unwrap();
        assert!((p.metres()[0] - after.metres()[0]).abs() < 0.001);
        assert_eq!(
            world
                .correction(actor)
                .unwrap()
                .state
                .input
                .applied_sequence,
            before.state.input.applied_sequence
        );
        let metrics = world.player_life_metrics().unwrap();
        assert_eq!(
            (metrics.strikes, metrics.damage, metrics.deaths),
            (3, 100, 1)
        );
    }
}
#[test]
fn same_patch_respawn_changes_life_and_input_stream_and_preserves_validated_readiness() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let actor = join(&mut world, [8006., 8000.]);
    let home = world.global_position(actor).unwrap();
    hit(&mut world, actor, npc, 1);
    dead(&mut world, actor);
    let old = world.player_life_snapshot(actor).unwrap();
    while !world.player_alive(actor) {
        world.step();
    }
    let new = world.player_life_snapshot(actor).unwrap();
    assert_eq!(
        (new.health, new.key.generation.get(), new.movement_stream),
        (100, 2, 3)
    );
    assert_ne!(new.key, old.key);
    assert!(
        world
            .procedural
            .as_ref()
            .unwrap()
            .patch(world.global_position(actor).unwrap())
            .unwrap()
            .covers(
                world.global_position(actor).unwrap(),
                world.players[&actor].ready.as_ref()
            )
    );
    let position = world.global_position(actor).unwrap();
    assert!((position.metres()[0] - home.metres()[0]).abs() < 0.001);
    assert!((position.metres()[2] - home.metres()[2]).abs() < 0.001);
    let mut commands = crate::prediction_wire::Commands {
        epoch: world.epoch(),
        stream_epoch: 2,
        ready_sequence: 1,
        commands: vec![crate::commands::Command {
            sequence: 1,
            tick: world.tick + 1,
            intent: Intent {
                axes: [1000, 0],
                ..Default::default()
            },
        }],
    };
    assert_eq!(world.command(actor, &commands), 0);
    commands.stream_epoch = 3;
    assert_eq!(world.command(actor, &commands), 1);
    world.step();
    assert!(world.global_position(actor).unwrap().metres()[0] > position.metres()[0]);
    assert_eq!(world.player_life_metrics().unwrap().respawns, 1);
    assert_eq!(world.player_life_snapshot(actor).unwrap().health, 100);
}
#[test]
fn another_player_does_not_steal_engagement_or_receive_the_attackers_windup_projection() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let a = join(&mut world, [8006., 8000.]);
    let b = join(&mut world, [8006., 8001.]);
    hit(&mut world, a, npc, 1);
    hit(&mut world, b, npc, 1);
    world.step();
    assert!(world.player_life_snapshot(a).unwrap().windup.is_some());
    assert!(world.player_life_snapshot(b).unwrap().windup.is_none());
    dead(&mut world, a);
    assert_eq!(world.player_life_snapshot(b).unwrap().health, 100);
}
#[test]
fn leaving_reach_cancels_the_real_swing_before_damage_and_legacy_patrol_never_retaliates() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let actor = join(&mut world, [8006., 8000.]);
    hit(&mut world, actor, npc, 1);
    world.step();
    assert!(world.player_life_snapshot(actor).unwrap().windup.is_some());
    let mut p = world.global_position(actor).unwrap().metres();
    p[0] -= 10.;
    world.players.get_mut(&actor).unwrap().position = GlobalPosition::new(p).unwrap();
    for _ in 0..20 {
        world.step();
    }
    assert_eq!(world.player_life_snapshot(actor).unwrap().health, 100);
    assert!(world.player_life_metrics().unwrap().cancelled > 0);
    let (mut legacy, npc) = fixture(TickRate::Hz30, false);
    let actor = join(&mut legacy, [8006., 8000.]);
    hit(&mut legacy, actor, npc, 1);
    for _ in 0..60 {
        legacy.step();
    }
    assert!(legacy.player_life_snapshot(actor).is_none());
    assert_eq!(
        legacy
            .combat_snapshot(actor)
            .unwrap()
            .vitals
            .unwrap()
            .health(),
        100
    );
}

#[test]
fn death_cancels_both_cast_and_queue_preserves_cooldown_and_stops_dead_mana_regeneration() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let actor = join(&mut world, [8006., 8000.]);
    hit(&mut world, actor, npc, 1);
    world.step();
    let long = ContentId::new("theia.dev.long-bolt").unwrap();
    let result = world.cast(
        actor,
        Request {
            epoch: world.epoch(),
            sequence: 2,
            ability: long.clone(),
            target: npc,
        },
    );
    assert!(matches!(result.reply.outcome, Outcome::Started { .. }));
    let result = world.cast(
        actor,
        Request {
            epoch: world.epoch(),
            sequence: 3,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target: npc,
        },
    );
    assert!(matches!(result.reply.outcome, Outcome::Queued { .. }));
    let cooldown = world.combat.as_ref().unwrap().actors[&actor].cooldowns[&long];
    world
        .combat
        .as_mut()
        .unwrap()
        .actors
        .get_mut(&actor)
        .unwrap()
        .mana = Pool::new(50, 100);
    dead(&mut world, actor);
    let cancelled: Vec<_> = world
        .combat_events()
        .iter()
        .filter_map(|event| match event {
            super::super::Event::Cancelled {
                actor: key,
                sequence,
                reason: Reason::Interrupted,
            } if key.entity.value() == actor => Some(*sequence),
            _ => None,
        })
        .collect();
    assert_eq!(cancelled, vec![2, 3]);
    let mana = world.combat_snapshot(actor).unwrap().vitals.unwrap().mana();
    for _ in 0..3 {
        world.step();
        assert_eq!(
            world.combat_snapshot(actor).unwrap().vitals.unwrap().mana(),
            mana
        );
    }
    while !world.player_alive(actor) {
        world.step();
    }
    assert_eq!(
        world.combat_snapshot(actor).unwrap().vitals.unwrap().mana(),
        100
    );
    assert_eq!(
        world.combat.as_ref().unwrap().actors[&actor].cooldowns[&long],
        cooldown
    );
    let result = world.cast(
        actor,
        Request {
            epoch: world.epoch(),
            sequence: 4,
            ability: long,
            target: npc,
        },
    );
    assert_eq!(result.reply.outcome, Outcome::Rejected(Reason::Unavailable));
}
#[test]
fn next_player_can_arm_a_fresh_engagement_immediately_after_the_previous_life_dies() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let a = join(&mut world, [8006., 8000.]);
    let b = join(&mut world, [8006., 8001.]);
    hit(&mut world, a, npc, 1);
    dead(&mut world, a);
    hit(&mut world, b, npc, 1);
    world.step();
    assert!(world.player_life_snapshot(b).unwrap().windup.is_some());
    assert_eq!(world.player_life_snapshot(b).unwrap().health, 100);
}

#[test]
fn real_static_obstruction_cancels_at_completion_and_blocks_another_windup_until_clear() {
    use bevy::prelude::Vec3;
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let actor = join(&mut world, [8006., 8000.]);
    hit(&mut world, actor, npc, 1);
    world.step();
    let p = world.global_position(actor).unwrap();
    let original = world.procedural.as_ref().unwrap().patch(p).unwrap().clone();
    let from = Vec3::from_array(original.frame.to_local(p).unwrap().metres());
    let to = Vec3::from_array(
        original
            .frame
            .to_local(world.targets[&npc.entity.value()].position)
            .unwrap()
            .metres(),
    );
    let mut geometry = original.scene.colliders.clone();
    geometry.push(crate::physics::PlacedCollider {
        shape: avian3d::prelude::Collider::cuboid(0.1, 8., 16.),
        center: (from + to) / 2.,
        layer: crate::physics::SOLID,
    });
    let wall = Arc::new(crate::procedural::Patch {
        owner: original.owner,
        frame: original.frame,
        ids: original.ids.clone(),
        scene: crate::physics::StaticScene::new(geometry),
    });
    world
        .procedural
        .as_mut()
        .unwrap()
        .patches
        .insert(original.owner, wall);
    for _ in 0..10 {
        world.step();
    }
    assert_eq!(world.player_life_snapshot(actor).unwrap().health, 100);
    assert!(world.player_life_snapshot(actor).unwrap().windup.is_none());
    assert_eq!(world.player_life_metrics().unwrap().strikes, 0);
    world
        .procedural
        .as_mut()
        .unwrap()
        .patches
        .insert(original.owner, original);
    world.step();
    assert!(world.player_life_snapshot(actor).unwrap().windup.is_some());
    for _ in 0..3 {
        world.step();
    }
    assert_eq!(world.player_life_snapshot(actor).unwrap().health, 60);
}
#[test]
fn death_closes_a_real_guide_session_and_old_revision_cannot_reopen_after_respawn() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let guide = world
        .spawn_development_guide(
            Arc::new(
                crate::dialogue::Profile::from_json(include_bytes!(
                    "../../../../../../content/gameplay/development-dialogue.json"
                ))
                .unwrap(),
            ),
            [8006., 8001.],
        )
        .unwrap();
    let actor = join(&mut world, [8006., 8000.]);
    world
        .interact(
            actor,
            &crate::dialogue::Request {
                sequence: 1,
                expected_revision: 0,
                action: crate::dialogue::Action::Open(guide.key),
            },
        )
        .unwrap();
    let opened = world.dialogue_view(actor).unwrap().revision;
    assert!(world.guide_interaction_eligible(actor, opened, guide.key));
    hit(&mut world, actor, npc, 1);
    dead(&mut world, actor);
    let closed = world.dialogue_view(actor).unwrap();
    assert!(closed.open.is_none());
    assert!(closed.revision > opened);
    assert!(world.nearby_guide(actor).is_none());
    assert!(!world.guide_interaction_eligible(actor, opened, guide.key));
    while !world.player_alive(actor) {
        world.step();
    }
    assert!(
        world
            .interact(
                actor,
                &crate::dialogue::Request {
                    sequence: 2,
                    expected_revision: opened,
                    action: crate::dialogue::Action::Open(guide.key)
                }
            )
            .is_err()
    );
    world
        .interact(
            actor,
            &crate::dialogue::Request {
                sequence: 3,
                expected_revision: closed.revision,
                action: crate::dialogue::Action::Open(guide.key),
            },
        )
        .unwrap();
}
#[test]
fn creature_death_cancels_its_attack_and_fresh_home_respawn_has_a_new_policy_incarnation() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let actor = join(&mut world, [8006., 8000.]);
    hit(&mut world, actor, npc, 1);
    world.step();
    assert!(world.player_life_snapshot(actor).unwrap().windup.is_some());
    world
        .combat
        .as_mut()
        .unwrap()
        .creatures
        .iter_mut()
        .find(|c| c.key == npc)
        .unwrap()
        .current = 1;
    hit(&mut world, actor, npc, 2);
    world.step();
    assert!(world.player_life_snapshot(actor).unwrap().windup.is_none());
    assert_eq!(world.player_life_snapshot(actor).unwrap().health, 100);
    for _ in 0..10 {
        world.step();
    }
    let next = world.combat.as_ref().unwrap().creatures[0].key;
    assert_ne!(npc, next);
    assert_eq!(world.player_life_metrics().unwrap().strikes, 0);
    hit(&mut world, actor, next, 3);
    world.step();
    assert_eq!(
        world
            .player_life_snapshot(actor)
            .unwrap()
            .windup
            .unwrap()
            .creature,
        next
    );
}
#[test]
fn departure_discards_the_old_pending_attack_and_fresh_admission_starts_full_with_new_actor_identity()
 {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let old = join(&mut world, [8006., 8000.]);
    hit(&mut world, old, npc, 1);
    world.step();
    let old_key = world.player_life_snapshot(old).unwrap().key;
    world.leave(old);
    let fresh = join(&mut world, [8006., 8000.]);
    let new = world.player_life_snapshot(fresh).unwrap();
    assert_ne!(new.key.actor, old_key.actor);
    assert_eq!((new.key.generation.get(), new.health), (1, 100));
    assert!(new.windup.is_none());
    assert!(world.player_life_snapshot(old).is_none());
    hit(&mut world, fresh, npc, 1);
    world.step();
    assert_eq!(
        world
            .player_life_snapshot(fresh)
            .unwrap()
            .windup
            .unwrap()
            .target,
        new.key
    );
}

#[test]
fn far_home_respawn_waits_for_actual_collision_loading_and_then_for_client_footprint_readiness() {
    use std::time::{Duration, Instant};
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let far = GlobalPosition::new([12000., 0., 12000.]).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !world.terrain_prepared_for(far) {
        world.poll_terrain_keeping(&[far]).unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    let actor = join(&mut world, [12000., 12000.]);
    let home = world.global_position(actor).unwrap();
    let patch = world
        .procedural
        .as_ref()
        .unwrap()
        .patch(GlobalPosition::new([8006., 0., 8000.]).unwrap())
        .unwrap();
    let mut near = patch.try_spawn(8006., 8000.).unwrap().metres();
    near[1] += f64::from(crate::world::PAWN_Y);
    // Trusted test setup relocation: the admitted home remains immutable.
    world.players.get_mut(&actor).unwrap().position = GlobalPosition::new(near).unwrap();
    world.players.get_mut(&actor).unwrap().body.reset(false);
    ready(&mut world, actor, 2);
    world.poll_terrain();
    assert!(!world.terrain_prepared_for(home));
    hit(&mut world, actor, npc, 1);
    dead(&mut world, actor);
    for _ in 0..20 {
        world.step();
    }
    assert!(!world.player_alive(actor));
    assert_eq!(
        world.player_life_snapshot(actor).unwrap().movement_stream,
        2
    );
    assert!(world.player_life_metrics().unwrap().held_respawns > 0);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !world.terrain_prepared_for(home) {
        world.poll_terrain();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    world.step();
    let life = world.player_life_snapshot(actor).unwrap();
    assert_eq!(
        (life.health, life.key.generation.get(), life.movement_stream),
        (100, 2, 3)
    );
    let position = world.global_position(actor).unwrap();
    assert!((position.metres()[0] - home.metres()[0]).abs() < 0.001);
    let patch = world.procedural.as_ref().unwrap().patch(position).unwrap();
    assert!(!patch.covers(position, world.players[&actor].ready.as_ref()));
    let mut packet = crate::prediction_wire::Commands {
        epoch: world.epoch(),
        stream_epoch: 3,
        ready_sequence: 2,
        commands: vec![crate::commands::Command {
            sequence: 1,
            tick: world.tick + 1,
            intent: Intent {
                axes: [1000, 0],
                ..Default::default()
            },
        }],
    };
    assert_eq!(world.command(actor, &packet), 1);
    world.step();
    assert!(
        (world.global_position(actor).unwrap().metres()[0] - position.metres()[0]).abs() < 0.001
    );
    ready(&mut world, actor, 3);
    packet.ready_sequence = 3;
    packet.commands[0].sequence = 2;
    packet.commands[0].tick = world.tick + 1;
    assert_eq!(world.command(actor, &packet), 1);
    world.step();
    assert!(world.global_position(actor).unwrap().metres()[0] > position.metres()[0]);
    assert!(world.player_life_error().is_none());
}
#[test]
fn committed_periodic_effects_can_finish_while_dead_but_cannot_acquire_the_respawned_life() {
    let (mut world, npc) = creatures::tests::fixture(TickRate::Hz30);
    world.enable_prediction().unwrap();
    let mut abilities: Value = serde_json::from_slice(include_bytes!(
        "../../../../../../content/gameplay/development-status-abilities.json"
    ))
    .unwrap();
    let mut direct = abilities["abilities"][0].clone();
    direct["id"] = json!("theia.dev.arcane-bolt");
    direct["effect"] = json!({"kind":"damage","amount":1});
    abilities["abilities"].as_array_mut().unwrap().push(direct);
    for a in abilities["abilities"].as_array_mut().unwrap() {
        a["cast_ms"] = json!(0);
        a["cooldown_ms"] = json!(0);
        a["uses_global_cooldown"] = json!(false);
        a["mana_cost"] = json!(0);
        if a["effect"]["kind"] == "damage" {
            a["effect"]["amount"] = json!(1);
        }
    }
    let mut statuses: Value = serde_json::from_slice(include_bytes!(
        "../../../../../../content/gameplay/development-statuses.json"
    ))
    .unwrap();
    statuses["statuses"][0]["interval_ms"] = json!(100);
    statuses["statuses"][0]["effect"]["amount"] = json!(1);
    world
        .enable_status_combat(
            &serde_json::to_vec(&abilities).unwrap(),
            100,
            300,
            STATS,
            &serde_json::to_vec(&statuses).unwrap(),
        )
        .unwrap();
    world.enable_player_life(profile(true)).unwrap();
    let actor = join(&mut world, [8006., 8000.]);
    hit(&mut world, actor, npc, 1);
    world.step();
    let reply = world.cast(
        actor,
        Request {
            epoch: world.epoch(),
            sequence: 2,
            ability: ContentId::new("theia.dev.scorch").unwrap(),
            target: npc,
        },
    );
    assert!(matches!(reply.reply.outcome, Outcome::Started { .. }));
    dead(&mut world, actor);
    assert!(!world.combat_status_snapshot(actor).unwrap().is_empty());
    let health = world.combat.as_ref().unwrap().creatures[0].current;
    for _ in 0..3 {
        world.step();
    }
    assert!(world.combat.as_ref().unwrap().creatures[0].current < health);
    assert!(!world.player_alive(actor));
    while !world.player_alive(actor) {
        world.step();
    }
    assert!(world.combat_status_snapshot(actor).unwrap().is_empty());
    let health = world.combat.as_ref().unwrap().creatures[0].current;
    for _ in 0..10 {
        world.step();
    }
    assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, health);
    assert_eq!(world.player_life_snapshot(actor).unwrap().health, 100);
}

#[test]
fn real_completed_world_observations_round_trip_and_gate_each_death_and_respawn_correction() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    let actor = join(&mut world, [8006., 8000.]);
    let profile = world.player_life_profile().unwrap().clone();
    let mut client = crate::player_life_wire::ClientState::new(
        profile.clone(),
        world.entity_key(actor).unwrap(),
        world.rate,
        world.identity(),
    )
    .unwrap();
    let initial = world.player_life_snapshot(actor).unwrap();
    client
        .apply(&crate::player_life_wire::encode(&profile, world.rate, &initial).unwrap())
        .unwrap();
    hit(&mut world, actor, npc, 1);
    let (mut saw_dead, mut saw_respawn) = (false, false);
    for _ in 0..40 {
        world.step();
        let value = world.player_life_snapshot(actor).unwrap();
        let bytes = crate::player_life_wire::encode(&profile, world.rate, &value).unwrap();
        assert!(bytes.len() <= crate::player_life_wire::MAX_FRAME_BYTES);
        client.apply(&bytes).unwrap();
        assert_eq!(client.current(), Some(&value));
        assert_eq!(
            client.movement_allowed(&world.correction(actor).unwrap().state),
            world.player_alive(actor)
        );
        saw_dead |= value.health == 0;
        saw_respawn |= value.key.generation.get() == 2;
    }
    assert!(saw_dead && saw_respawn);
}

#[test]
fn active_dodge_has_no_invulnerability_and_death_respawn_keep_nonce_and_cooldown() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, _) = fixture(rate, true);
        let p = Arc::new(
            crate::dodging::Profile::from_json(include_bytes!(
                "../../../../../../content/gameplay/development-dodge.json"
            ))
            .unwrap(),
        );
        world.enable_dodge(p.clone()).unwrap();
        let actor = join(&mut world, [8006., 8000.]);
        world.step();
        let packet = crate::prediction_wire::Commands {
            epoch: world.epoch(),
            stream_epoch: 1,
            ready_sequence: 1,
            commands: vec![crate::commands::Command {
                sequence: 1,
                tick: world.tick + 1,
                intent: Intent {
                    facing: None,
                    dodge: Some(crate::dodging::Request {
                        nonce: 1,
                        axes: [1000, 0],
                    }),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(world.command(actor, &packet), 1);
        world.step();
        let before = world.correction(actor).unwrap().state.pose.dodge.unwrap();
        assert!(before.policy.active.is_some());
        let key = world.player_life_snapshot(actor).unwrap().key;
        let mut combat = world.combat.take().unwrap();
        let damage = combat.damage_player(&mut world, key, 100).unwrap().unwrap();
        world.combat = Some(combat);
        assert!(damage.died);
        assert!(!world.player_alive(actor));
        let after = world.correction(actor).unwrap().state.pose.dodge.unwrap();
        assert!(after.policy.active.is_none());
        assert_eq!(
            (after.last_nonce, after.policy.ready_at),
            (before.last_nonce, before.policy.ready_at)
        );
        assert_eq!(world.command(actor, &packet), 0);
        while !world.player_alive(actor) {
            world.step();
        }
        let c = world.correction(actor).unwrap().state;
        let d = c.pose.dodge.unwrap();
        assert!(d.valid_at(&p, rate, c.input.tick));
        assert!(d.policy.active.is_none());
        assert_eq!(
            (d.last_nonce, d.policy.ready_at),
            (before.last_nonce, before.policy.ready_at)
        );
        assert_eq!(c.input.stream_epoch, 3);
    }
}

#[test]
fn stationary_cast_cancels_on_dodge_start_tick_before_any_displacement() {
    let (mut world, npc) = fixture(TickRate::Hz30, true);
    world
        .enable_dodge(Arc::new(
            crate::dodging::Profile::from_json(include_bytes!(
                "../../../../../../content/gameplay/development-dodge.json"
            ))
            .unwrap(),
        ))
        .unwrap();
    let actor = join(&mut world, [8006., 8000.]);
    world.step();
    let reply = world.cast(
        actor,
        Request {
            epoch: world.epoch(),
            sequence: 1,
            ability: ContentId::new("theia.dev.long-bolt").unwrap(),
            target: npc,
        },
    );
    assert!(matches!(reply.reply.outcome, Outcome::Started { .. }));
    let before = world.global_position(actor).unwrap();
    assert_eq!(
        world.command(
            actor,
            &crate::prediction_wire::Commands {
                epoch: world.epoch(),
                stream_epoch: 1,
                ready_sequence: 1,
                commands: vec![crate::commands::Command {
                    sequence: 1,
                    tick: world.tick + 1,
                    intent: Intent {
                        facing: None,
                        dodge: Some(crate::dodging::Request {
                            nonce: 1,
                            axes: [1000, 0]
                        }),
                        ..Default::default()
                    }
                }]
            }
        ),
        1
    );
    world.step();
    assert_eq!(
        world.global_position(actor).unwrap().metres()[0],
        before.metres()[0]
    );
    assert!(world.combat_events().iter().any(|e| matches!(
        e,
        super::super::Event::Cancelled {
            sequence: 1,
            reason: Reason::Moving,
            ..
        }
    )));
}

#[test]
fn basic_weapon_hit_arms_real_retaliation_and_owner_death_respawn_never_resume_engagement() {
    use crate::simulation::combat::autoattack;
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, npc) = fixture(rate, true);
        world.enable_classic().unwrap();
        let items =
            include_bytes!("../../../../../../content/gameplay/development-equipment-items.json");
        world
            .enable_autoattack(
                include_bytes!("../../../../../../content/gameplay/development-melee-weapons.json"),
                items,
            )
            .unwrap();
        let actor = join(&mut world, [8007., 8000.]);
        let key = world.entity_key(actor).unwrap();
        world.observe_combat_equipment(
            key,
            Some((
                crate::terrain::sha256(items),
                1,
                Some(ContentId::new("theia.dev.training-sword").unwrap()),
            )),
        );
        let c = world.correction(actor).unwrap().state;
        assert_eq!(
            world.command(
                actor,
                &crate::prediction_wire::Commands {
                    epoch: world.epoch(),
                    stream_epoch: c.input.stream_epoch,
                    ready_sequence: c.ready_sequence,
                    commands: vec![crate::commands::Command {
                        sequence: 1,
                        tick: c.input.tick + 1,
                        intent: Intent {
                            facing: Some(theia_core::facing::Facing::from_units(49152)),
                            ..Intent::default()
                        }
                    }]
                }
            ),
            1
        );
        world.step();
        assert_eq!(
            world
                .request_autoattack(
                    actor,
                    autoattack::Request {
                        stream_epoch: c.input.stream_epoch,
                        epoch: world.epoch(),
                        sequence: 1,
                        target: Some(npc)
                    }
                )
                .outcome,
            autoattack::Outcome::Engaged
        );
        world.step();
        assert!(
            world
                .combat_events()
                .iter()
                .any(|e| matches!(e, super::super::Event::MeleeDamage { amount: 15, .. }))
        );
        dead(&mut world, actor);
        assert_eq!(world.autoattack_snapshot(actor).unwrap().target, None);
        let health = world.combat.as_ref().unwrap().creatures[0].current;
        ready(&mut world, actor, 2);
        for _ in 0..rate.hz() * 2 {
            world.step();
        }
        assert!(world.player_alive(actor));
        assert_eq!(world.autoattack_snapshot(actor).unwrap().target, None);
        assert_eq!(world.autoattack_snapshot(actor).unwrap().swing_sequence, 1);
        assert_eq!(world.combat.as_ref().unwrap().creatures[0].current, health);
        let stale = autoattack::Request {
            epoch: world.epoch(),
            stream_epoch: c.input.stream_epoch,
            sequence: 2,
            target: Some(npc),
        };
        assert_eq!(
            world.request_autoattack(actor, stale).outcome,
            autoattack::Outcome::Ignored
        );
        let stream = world.correction(actor).unwrap().state.input.stream_epoch;
        assert_ne!(stream, c.input.stream_epoch);
        assert_eq!(
            world
                .request_autoattack(
                    actor,
                    autoattack::Request {
                        stream_epoch: stream,
                        sequence: 3,
                        ..stale
                    }
                )
                .outcome,
            autoattack::Outcome::Engaged
        );
    }
}
