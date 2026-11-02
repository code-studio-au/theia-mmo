use super::*;
use crate::physics::PlacedCollider;
use theia_core::{
    coordinates::{ChunkId, GlobalPosition, LocalFrame},
    dodge::{Direction, Gate, Profile, State},
};
fn block(center: [f32; 3], size: [f32; 3]) -> PlacedCollider {
    PlacedCollider {
        shape: Collider::cuboid(size[0], size[1], size[2]),
        center: Vec3::from_array(center),
        layer: WORLD,
    }
}
fn floor() -> PlacedCollider {
    block([0., -0.25, 0.], [100., 0.5, 100.])
}
fn profile() -> Profile {
    Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-dodge.json"
    ))
    .unwrap()
}
fn charged(
    scene: &StaticScene,
    rate: TickRate,
    intent: Intent,
    boundary: Option<&dyn Boundary>,
    start: [f32; 3],
) -> (Motion, LocalPosition) {
    let p = profile();
    let mut policy = State::default();
    let mut body = Motion::default();
    let mut feet = advance_bounded(
        scene,
        &mut body,
        LocalPosition::new(start).unwrap(),
        Intent::default(),
        rate,
        boundary,
    )
    .unwrap();
    assert!(body.grounded());
    policy
        .start(
            &p,
            rate,
            0,
            Direction::from_axes([1000, 0]).unwrap(),
            Gate {
                alive: true,
                grounded: body.grounded(),
                terrain_ready: true,
            },
        )
        .unwrap();
    for tick in 1..=rate.ticks_for(p.duration()).unwrap() {
        feet = advance_dodge(
            scene,
            &mut body,
            feet,
            intent,
            rate,
            boundary,
            policy.advance(tick).unwrap(),
        )
        .unwrap();
        validate_pose(scene, feet, body.grounded()).unwrap();
    }
    assert!(!policy.active());
    (body, feet)
}
#[test]
fn dodge_uses_one_swept_budget_instead_of_adding_walk_sprint_or_diagonal_speed() {
    let scene = StaticScene::new(vec![floor()]);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (body, feet) = charged(
            &scene,
            rate,
            Intent {
                facing: None,
                axes: [1000, 1000],
                sprint: true,
                jump: false,
                dodge: None,
            },
            None,
            [0., 0., 0.],
        );
        assert!((feet.metres()[0] - 3.).abs() < 0.0001);
        assert!(feet.metres()[2].abs() < 0.0001);
        assert!(body.grounded());
    }
}
#[test]
fn thin_walls_block_high_speed_capsules_at_every_rate() {
    let scene = StaticScene::new(vec![floor(), block([1.25, 2., 0.], [0.01, 4., 10.])]);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (_, feet) = charged(&scene, rate, Intent::default(), None, [0., 0., 0.]);
        assert!(
            feet.metres()[0] > 0.8 && feet.metres()[0] < 0.9,
            "{:?}",
            feet.metres()
        );
    }
}
#[test]
fn normal_step_and_headroom_limits_apply_to_dodge() {
    for (height, ceiling, allowed) in [
        (0.25, false, true),
        (0.6, false, false),
        (0.25, true, false),
    ] {
        let mut geometry = vec![floor(), block([2., height / 2., 0.], [2., height, 8.])];
        if ceiling {
            geometry.push(block([0., 1.95, 0.], [10., 0.2, 10.]));
        }
        let scene = StaticScene::new(geometry);
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (_, feet) = charged(&scene, rate, Intent::default(), None, [0., 0., 0.]);
            if allowed {
                assert!(
                    feet.metres()[0] > 2.9 && feet.metres()[1] > 0.24,
                    "{height}/{ceiling}: {:?}",
                    feet.metres()
                );
            } else {
                assert!(
                    feet.metres()[0] < 0.67 && feet.metres()[1] < 0.01,
                    "{height}/{ceiling}: {:?}",
                    feet.metres()
                );
            }
        }
    }
}
#[test]
fn actual_chunk_permission_stops_dodge_before_an_unloaded_neighbor() {
    let frame = LocalFrame::new(GlobalPosition::new([14976., 0., 14976.]).unwrap());
    let coverage =
        crate::chunks::Coverage::procedural_cells(frame, [ChunkId { x: 117, z: 117 }]).unwrap();
    let scene = StaticScene::new(vec![block([64., -0.25, 64.], [128., 0.5, 128.])]);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (_, feet) = charged(
            &scene,
            rate,
            Intent::default(),
            Some(&coverage),
            [127., 0., 64.],
        );
        assert!(feet.metres()[0] > 127.6 && feet.metres()[0] < 127.65);
        let global = frame.to_global(feet).unwrap();
        assert_eq!(global.chunk(128), Some(ChunkId { x: 117, z: 117 }));
    }
}
#[test]
fn dodge_keeps_held_jump_latched_until_a_real_release_and_press() {
    let scene = StaticScene::new(vec![floor()]);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let jump = Intent {
            jump: true,
            ..Default::default()
        };
        let (mut body, feet) = charged(&scene, rate, jump, None, [0., 0., 0.]);
        assert!(body.grounded() && body.snapshot().jump_held);
        let held = advance(&scene, &mut body, feet, jump, rate).unwrap();
        assert!(body.grounded());
        let released = advance(&scene, &mut body, held, Intent::default(), rate).unwrap();
        let jumped = advance(&scene, &mut body, released, jump, rate).unwrap();
        assert!(!body.grounded() && jumped.metres()[1] > released.metres()[1]);
    }
}
#[test]
fn dodge_retains_exactly_one_gravity_step_and_rejects_invalid_proposals_atomically() {
    let scene = StaticScene::new(vec![floor()]);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut normal = Motion::default();
        let mut dodge = Motion::default();
        let start = LocalPosition::new([0., 5., 0.]).unwrap();
        let a = advance(&scene, &mut normal, start, Intent::default(), rate).unwrap();
        let b = advance_dodge(
            &scene,
            &mut dodge,
            start,
            Intent::default(),
            rate,
            None,
            None,
        )
        .unwrap();
        assert_eq!(a, b);
        assert_eq!(normal.snapshot(), dodge.snapshot());
        let before = dodge.snapshot();
        for horizontal_metres in [[f64::NAN, 0.], [0., f64::INFINITY], [2., 0.]] {
            let step = theia_core::dodge::Step {
                horizontal_metres,
                finished: false,
            };
            assert!(
                advance_dodge(
                    &scene,
                    &mut dodge,
                    b,
                    Intent::default(),
                    rate,
                    None,
                    Some(step)
                )
                .is_err()
            );
            assert_eq!(dodge.snapshot(), before);
        }
    }
}

#[test]
fn accepted_start_tick_holds_horizontal_motion_and_latches_jump() {
    let scene = StaticScene::new(vec![floor()]);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut body = Motion::default();
        let intent = Intent {
            facing: None,
            axes: [1000, 1000],
            sprint: true,
            jump: true,
            dodge: None,
        };
        let feet = advance_dodge(
            &scene,
            &mut body,
            LocalPosition::new([0., 0., 0.]).unwrap(),
            intent,
            rate,
            None,
            None,
        )
        .unwrap();
        assert!(feet.metres()[0].abs() < 0.0001 && feet.metres()[2].abs() < 0.0001);
        assert!(body.grounded() && body.snapshot().jump_held);
    }
}
#[test]
fn dodge_keeps_walkable_and_steep_slope_policy() {
    for (slope, walkable) in [(0.5, true), (1.5, false)] {
        let ramp = PlacedCollider {
            shape: Collider::trimesh(
                vec![
                    Vec3::new(1., 0., -10.),
                    Vec3::new(1., 0., 10.),
                    Vec3::new(11., 10. * slope, -10.),
                    Vec3::new(11., 10. * slope, 10.),
                ],
                vec![[0, 1, 2], [2, 1, 3]],
            ),
            center: Vec3::ZERO,
            layer: WORLD,
        };
        let scene = StaticScene::new(vec![floor(), ramp]);
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (_, feet) = charged(&scene, rate, Intent::default(), None, [0., 0., 0.]);
            if walkable {
                assert!(
                    feet.metres()[0] > 2. && feet.metres()[1] > 0.5,
                    "{rate:?}: {:?}",
                    feet.metres()
                );
            } else {
                assert!(feet.metres()[0] < 1.2, "{rate:?}: {:?}", feet.metres());
            }
        }
    }
}
