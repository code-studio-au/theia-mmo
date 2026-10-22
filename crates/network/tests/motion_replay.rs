//! P2-04 prerequisite: controller state correction/replay, not client prediction.
use theia_core::{coordinates::LocalPosition, time::TickRate};
use theia_network::{
    kinematic::{self, Motion, MotionState},
    wire::Intent,
    world,
};

#[test]
fn corrections_replay_identically_midair_and_while_jump_is_held_after_landing() {
    let scene = world::physics_scene();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut body = Motion::default();
        let mut position =
            kinematic::spawn(scene, LocalPosition::new([0., 0., 0.]).unwrap()).unwrap();
        let mut samples = Vec::new();
        let intent = |tick| Intent {
            axes: [100, 0],
            sprint: true,
            jump: (6..=80).contains(&tick) || (91..=105).contains(&tick),
        };
        for tick in 1..=180 {
            position = kinematic::advance(scene, &mut body, position, intent(tick), rate).unwrap();
            samples.push((position, body.snapshot()));
        }
        for checkpoint in [8, 50, 85] {
            let (mut position, state) = samples[checkpoint - 1];
            let wire = serde_json::to_vec(&state).unwrap();
            let mut replay = Motion::restore(serde_json::from_slice(&wire).unwrap()).unwrap();
            for tick in checkpoint + 1..=180 {
                position =
                    kinematic::advance(scene, &mut replay, position, intent(tick), rate).unwrap();
                assert_eq!(
                    position,
                    samples[tick - 1].0,
                    "Position diverged at {rate:?}, checkpoint {checkpoint}, tick {tick}"
                );
                assert_eq!(
                    replay.snapshot(),
                    samples[tick - 1].1,
                    "Jump/ground state diverged at tick {tick}"
                );
            }
        }
    }
}
#[test]
fn invalid_motion_corrections_are_rejected_and_diagnostics_are_not_imported() {
    for speed in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1024.01, 7.01] {
        assert!(
            Motion::restore(MotionState {
                vertical_speed: speed,
                grounded: false,
                jump_held: false
            })
            .is_err()
        );
    }
    assert!(
        Motion::restore(MotionState {
            vertical_speed: -0.1,
            grounded: true,
            jump_held: false
        })
        .is_err()
    );
    let state = MotionState {
        vertical_speed: -0.,
        grounded: true,
        jump_held: true,
    };
    let body = Motion::restore(state).unwrap();
    assert!(body.snapshot().jump_held);
    assert_eq!(body.snapshot().vertical_speed.to_bits(), 0f32.to_bits());
    assert_eq!(body.solver_rollbacks(), 0);
    let injected = serde_json::json!({"vertical_speed":0.,"grounded":true,"jump_held":false,"solver_rollbacks":100});
    assert!(serde_json::from_value::<MotionState>(injected).is_err());
}
