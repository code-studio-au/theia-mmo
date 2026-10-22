use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use theia_core::coordinates::ChunkId;
use theia_network::{
    commands::Command,
    prediction_wire::Commands,
    procedural,
    simulation::World,
    wire::{ChunkReady, Intent, VERSION},
};
#[test]
fn queued_commands_acknowledge_completed_steps_and_teleports_invalidate_pending_streams() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let id = world.join().unwrap();
    let ready = |world: &mut World, sequence, owner| {
        assert!(
            world.client_ready(
                id,
                &ChunkReady {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence,
                    chunks: procedural::neighbours(owner, 1)
                        .iter()
                        .map(|id| [id.x, id.z])
                        .collect()
                }
            )
        );
    };
    ready(&mut world, 1, ChunkId { x: 62, z: 62 });
    let packet = Commands {
        epoch: world.epoch(),
        stream_epoch: 1,
        ready_sequence: 1,
        commands: vec![Command {
            sequence: 1,
            tick: 2,
            intent: Intent {
                axes: [1000, 0],
                jump: true,
                sprint: true,
            },
        }],
    };
    assert_eq!(world.command(id, &packet), 1);
    assert_eq!(
        world.correction(id).unwrap().state.input.applied_sequence,
        0
    );
    world.step();
    assert_eq!(
        world.correction(id).unwrap().state.input.applied_sequence,
        0
    );
    world.step();
    assert_eq!(
        world.correction(id).unwrap().state.input.applied_sequence,
        1
    );
    assert!(!world.correction(id).unwrap().state.pose.motion.grounded);
    assert_eq!(world.command(id, &packet), 0);
    assert!(world.teleport_development(id, [15800., 180.]));
    let deadline = Instant::now() + Duration::from_secs(3);
    while world.correction(id).unwrap().state.input.stream_epoch == 1 {
        assert!(Instant::now() < deadline);
        world.poll_terrain();
        sleep(Duration::from_millis(1));
    }
    let correction = world.correction(id).unwrap().state;
    assert_eq!(correction.input.stream_epoch, 2);
    assert_eq!(correction.input.applied_sequence, 0);
    assert_eq!(world.command(id, &packet), 0);
    // Waiting longer than input expiry must not manufacture a jump release.
    for _ in 0..20 {
        world.step();
        assert!(world.correction(id).unwrap().state.pose.motion.jump_held);
    }
    let correction = world.correction(id).unwrap().state;
    ready(&mut world, 2, ChunkId { x: 123, z: 1 });
    let next = Commands {
        stream_epoch: 2,
        ready_sequence: 2,
        commands: vec![Command {
            sequence: 1,
            tick: correction.input.tick + 1,
            intent: Intent {
                axes: [1000, 0],
                jump: true,
                ..Intent::default()
            },
        }],
        ..packet
    };
    assert_eq!(world.command(id, &next), 1);
    world.step();
    let next = world.correction(id).unwrap().state;
    assert_eq!(next.input.applied_sequence, 1);
    assert_eq!(next.input.stream_epoch, 2);
    assert!(next.pose.feet.metres()[0] > 15800.);
    assert!(
        next.pose.motion.grounded,
        "Held jump retriggered after teleport bootstrap"
    );
}
