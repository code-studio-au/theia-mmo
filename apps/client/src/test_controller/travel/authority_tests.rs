//! Closed-loop follower through real authoritative collision, with no display.
use super::*;
use std::{path::PathBuf, sync::Arc};
use theia_core::time::TickRate;
use theia_network::{
    commands::Command,
    prediction_wire::Commands,
    simulation::World,
    wire::{ChunkReady, Intent, VERSION},
};

#[test]
#[ignore = "requires THEIA_ROUTE_WORLD and THEIA_ROUTE_FILE with actual authored navigation"]
fn actual_route_runs_out_and_back_through_authority_at_all_rates() {
    let path = PathBuf::from(std::env::var_os("THEIA_ROUTE_FILE").unwrap());
    let source = procedural::Source::Authored(Arc::new(
        theia_network::authored_world::Store::open(&PathBuf::from(
            std::env::var_os("THEIA_ROUTE_WORLD").unwrap(),
        ))
        .unwrap(),
    ));
    let bytes = std::fs::read(&path).unwrap();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (_, mut travel, _) = Travel::load(&path, &bytes, &source).unwrap();
        travel.return_trip = true;
        let start = travel.points[0].metres();
        let procedural::Source::Authored(authored) = &source else {
            unreachable!()
        };
        // A test route may begin at a quest giver rather than the saved spawn.
        // Construct the world at its bound spawn, then restore the test actor
        // at the checked route start through the normal admission path.
        let spawn = authored.spawn(0).unwrap().metres();
        let mut world =
            World::with_terrain_source([spawn[0], spawn[2]], 1., source.clone()).unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        let mut body = start;
        body[1] += f64::from(theia_network::world::PAWN_Y);
        let actor = world
            .join_restored(GlobalPosition::new(body).unwrap())
            .unwrap();
        let mut controller = crate::classic_controls::Controller::parse(include_bytes!(
            "../../../../../content/input/classic-controls.json"
        ))
        .unwrap();
        let mut sequence = 0;
        let mut ready_sequence = 0;
        let mut chunks = Vec::new();
        for leg in ["outward", "return"] {
            if leg == "return" {
                travel.restart();
                assert!(travel.returning);
            }
            let mut heading = controller.facing();
            let mut route_turn = 0.;
            for pair in travel.points.windows(2) {
                if let Some(next) = Facing::towards(pair[0], pair[1]) {
                    route_turn += heading.difference(next).abs();
                    heading = next;
                }
            }
            for _ in 0..600 * rate.hz() {
                // The owner prepares collision outside its fixed simulation step.
                world.poll_terrain();
                let state = world.correction(actor).unwrap().state;
                let patch = world.procedural().unwrap().patch(state.pose.feet).unwrap();
                let current: Vec<_> = patch.ids.iter().map(|c| [c.x, c.z]).collect();
                if current != chunks {
                    ready_sequence += 1;
                    assert!(world.client_ready(
                        actor,
                        &ChunkReady {
                            version: VERSION,
                            epoch: world.epoch(),
                            sequence: ready_sequence,
                            chunks: current.clone(),
                        }
                    ));
                    chunks = current;
                }
                controller.sync(Some(state));
                let action = travel.sample(
                    1. / f64::from(rate.hz()),
                    state.pose.feet,
                    controller.facing(),
                    controller.keyboard_turn_rate(),
                );
                let intent = if let Some((step, seconds)) = action {
                    assert!(!step.walk, "Both checked travel legs must use run mode");
                    let input = controller.sample_script(
                        [step.strafe, -step.forward],
                        step.turn,
                        step.walk,
                        step.jump,
                        seconds,
                    );
                    Intent {
                        axes: input.axes,
                        facing: Some(controller.facing()),
                        sprint: input.sprint,
                        jump: input.jump,
                        ..Default::default()
                    }
                } else {
                    Intent {
                        facing: Some(controller.facing()),
                        ..Default::default()
                    }
                };
                sequence += 1;
                let state = world.correction(actor).unwrap().state;
                assert_eq!(
                    world.command(
                        actor,
                        &Commands {
                            epoch: world.epoch(),
                            stream_epoch: state.input.stream_epoch,
                            ready_sequence,
                            commands: vec![Command {
                                sequence,
                                tick: state.input.tick + 1,
                                intent,
                            }],
                        }
                    ),
                    1
                );
                world.step();
                if action.is_none() {
                    break;
                }
            }
            assert!(
                travel.finished(),
                "{rate:?} {leg}: {}",
                travel.description()
            );
            // Allow corridor corrections, but not repeated spins at waypoints.
            assert!(travel.turn_radians <= route_turn + 0.12 * travel.points.len() as f64);
            let feet = world.correction(actor).unwrap().state.pose.feet;
            assert!(distance(feet, *travel.points.last().unwrap()) <= ARRIVAL);
            assert_eq!(world.movement_recoveries(), 0);
            let stopped = feet;
            for _ in 0..rate.hz() {
                world.step();
            }
            assert!(distance(stopped, world.correction(actor).unwrap().state.pose.feet) < 0.001);
            println!("{rate:?} {leg}: {}", travel.description());
        }
    }
}
