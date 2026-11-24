use theia_core::{facing::Facing, time::TickRate};
use theia_network::{
    classic, classic_wire,
    commands::Command,
    development,
    prediction_wire::Commands,
    simulation::World,
    wire::{ChunkReady, Intent, VERSION},
};
#[test]
fn owner_speed_switches_match_prediction_and_keep_normal_wire_strict() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        assert!(!world.set_development_speed(8));
        assert!(world.enable_development_speed().is_err());
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        world.enable_development_speed().unwrap();
        let id = world.join().unwrap();
        assert!(world.enable_development_speed().is_err());
        let patch = world
            .procedural()
            .unwrap()
            .patch(world.global_position(id).unwrap())
            .unwrap()
            .clone();
        assert!(world.client_ready(
            id,
            &ChunkReady {
                version: VERSION,
                epoch: world.epoch(),
                sequence: 1,
                chunks: patch.ids.iter().map(|c| [c.x, c.z]).collect()
            }
        ));
        world.step();
        let mut pose = world.correction(id).unwrap().state.pose;
        let mut sequence = 0;
        for speed in [1, 2, 4, 8, 16, 1] {
            assert!(world.set_development_speed(speed));
            for running in [true, false] {
                for n in 0..3 {
                    sequence += 1;
                    let facing = Facing::from_units(16384);
                    let intent = Intent {
                        facing: Some(facing),
                        axes: facing.world_axes([0, -1000]).unwrap(),
                        sprint: running,
                        jump: n == 0,
                        ..Intent::default()
                    };
                    let c = world.correction(id).unwrap().state;
                    assert_eq!(
                        world.command(
                            id,
                            &Commands {
                                epoch: world.epoch(),
                                stream_epoch: c.input.stream_epoch,
                                ready_sequence: c.ready_sequence,
                                commands: vec![Command {
                                    sequence,
                                    tick: c.input.tick + 1,
                                    intent
                                }]
                            }
                        ),
                        1
                    );
                    world.step();
                    pose = classic::terrain_step_development(
                        &patch,
                        &patch.ids,
                        pose,
                        intent,
                        rate,
                        f32::from(speed),
                    )
                    .unwrap()
                    .unwrap();
                    let corrected = world.correction(id).unwrap();
                    assert_eq!(corrected.speed, speed);
                    assert_eq!(pose.facing, corrected.state.pose.facing);
                    for (a, b) in pose
                        .feet
                        .metres()
                        .into_iter()
                        .zip(corrected.state.pose.feet.metres())
                    {
                        assert!((a - b).abs() < 0.0002);
                    }
                    let bytes = development::encode(corrected).unwrap();
                    assert_eq!(development::decode(&bytes).unwrap(), corrected);
                    assert!(classic_wire::decode_correction(&bytes).is_err());
                    if speed != 1 {
                        assert!(classic_wire::encode_correction(corrected).is_err());
                    }
                    let mut bad = bytes;
                    *bad.last_mut().unwrap() = 3;
                    assert!(development::decode(&bad).is_err());
                }
            }
        }
        for invalid in [0, 3, 32, 255] {
            assert!(!world.set_development_speed(invalid));
            assert_eq!(world.correction(id).unwrap().speed, 1);
        }
    }
}
