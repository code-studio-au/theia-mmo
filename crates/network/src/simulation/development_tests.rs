use super::*;
use crate::{commands::Command, prediction_wire::Commands, wire::ChunkReady};
fn world() -> (World, u64) {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    world.enable_classic().unwrap();
    assert!(!world.reset_development_position());
    world.enable_development_speed().unwrap();
    assert!(!world.reset_development_position());
    let id = world.join().unwrap();
    (world, id)
}
fn poll_reset(world: &mut World, id: u64, previous: u64) {
    let until = std::time::Instant::now() + Duration::from_secs(5);
    while world.correction(id).unwrap().state.input.stream_epoch == previous {
        assert!(std::time::Instant::now() < until);
        world.poll_terrain();
        world.step();
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn trusted_reset_waits_for_collision_clears_falling_and_rejects_old_input() {
    let (mut w, id) = world();
    let home = w.global_position(id).unwrap();
    assert!(w.set_development_speed(16));
    let stream = w.correction(id).unwrap().state.input.stream_epoch;
    assert!(w.teleport_development(id, [12000., 8000.]));
    poll_reset(&mut w, id, stream);
    let previous = w.correction(id).unwrap().state;
    let ids = w
        .procedural()
        .unwrap()
        .patch(w.global_position(id).unwrap())
        .unwrap()
        .ids
        .iter()
        .map(|c| [c.x, c.z])
        .collect();
    assert!(w.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: w.epoch(),
            sequence: 1,
            chunks: ids
        }
    ));
    let actor = w.players.get_mut(&id).unwrap();
    let [x, _, z] = actor.position.metres();
    actor.position = GlobalPosition::new([x, -900., z]).unwrap();
    actor.body = crate::kinematic::Motion::restore(crate::kinematic::MotionState {
        vertical_speed: -50.,
        grounded: false,
        jump_held: true,
    })
    .unwrap();
    assert!(w.reset_development_position());
    assert!(w.reset_development_position()); // Same pending request is idempotent.
    let before = w.global_position(id).unwrap();
    w.step(); // No additional fall while the home patch is loading.
    assert_eq!(w.global_position(id).unwrap(), before);
    poll_reset(&mut w, id, previous.input.stream_epoch);
    let actual = w.global_position(id).unwrap().metres();
    for (a, b) in actual.into_iter().zip(home.metres()) {
        assert!((a - b).abs() < 0.1);
    }
    let c = w.correction(id).unwrap();
    assert_eq!(c.speed, 16);
    assert!(c.state.pose.motion.vertical_speed.abs() < 1.);
    assert!(w.players[&id].ready.as_ref().unwrap().is_empty());
    assert_eq!(
        w.command(
            id,
            &Commands {
                epoch: w.epoch(),
                stream_epoch: previous.input.stream_epoch,
                ready_sequence: 1,
                commands: vec![Command {
                    sequence: 100,
                    tick: previous.input.tick + 1,
                    intent: Intent::default()
                }]
            }
        ),
        0
    );
    assert_eq!(
        crate::development::decode(&crate::development::encode(c).unwrap()).unwrap(),
        c
    );
}
#[test]
fn local_heightfield_fall_recovers_before_codec_limit_but_airborne_actor_is_preserved() {
    let (mut w, id) = world();
    let home = w.global_position(id).unwrap();
    let mut above = home.metres();
    above[1] += 15.;
    w.players.get_mut(&id).unwrap().position = GlobalPosition::new(above).unwrap();
    w.poll_terrain();
    assert_eq!(w.global_position(id).unwrap().metres(), above);
    let old = w.correction(id).unwrap().state.input.stream_epoch;
    let mut below = home.metres();
    below[1] -= 5.;
    w.players.get_mut(&id).unwrap().position = GlobalPosition::new(below).unwrap();
    w.poll_terrain();
    assert_eq!(w.correction(id).unwrap().state.input.stream_epoch, old + 1);
    assert_eq!(w.global_position(id).unwrap(), home);
    assert_eq!(w.movement_recoveries(), 1);
    w.poll_terrain();
    assert_eq!(w.movement_recoveries(), 1);
    w.join().unwrap();
    assert!(!w.reset_development_position());
}
