//! Actual Avian queries against original static geometry, not mocked hit results.
use avian3d::prelude::Collider;
use bevy::prelude::Vec3;
use theia_core::{coordinates::LocalPosition, time::TickRate};
use theia_network::{
    kinematic::{self, Motion},
    physics::{GROUND, PlacedCollider, SOLID, StaticScene},
    wire::Intent,
};
fn block(center: [f32; 3], size: [f32; 3]) -> PlacedCollider {
    PlacedCollider {
        shape: Collider::cuboid(size[0], size[1], size[2]),
        center: Vec3::from_array(center),
        layer: GROUND | SOLID,
    }
}
fn floor() -> PlacedCollider {
    block([0., -0.25, 0.], [100., 0.5, 100.])
}
fn walk(axes: [i16; 2]) -> Intent {
    Intent {
        jump: false,
        axes,
        sprint: false,
    }
}
fn tick(
    scene: &StaticScene,
    body: &mut Motion,
    p: [f32; 3],
    mut intent: Intent,
    jump: bool,
    rate: TickRate,
) -> [f32; 3] {
    intent.jump = jump;
    let feet = kinematic::advance(scene, body, LocalPosition::new(p).unwrap(), intent, rate)
        .unwrap_or_else(|e| panic!("{e}; pose={p:?} state={body:?}"));
    kinematic::validate_pose(scene, feet, body.grounded())
        .unwrap_or_else(|e| panic!("{e}; pose={feet:?} state={body:?}"));
    feet.metres()
}
#[test]
fn fixed_rates_speed_gravity_and_jump_press_release_are_server_controllable() {
    let scene = StaticScene::new(vec![floor()]);
    for hz in [20, 30, 60] {
        let rate = TickRate::from_hz(hz).unwrap();
        let mut body = Motion::default();
        let mut p = [0., 0., 0.];
        for _ in 0..hz {
            p = tick(&scene, &mut body, p, walk([1000, 0]), false, rate);
        }
        assert!((p[0] - 5.).abs() < 0.0001);
        assert!(body.grounded());
        assert!((p[1] - 0.008).abs() < 0.001);
        let mut maximum = 0f32;
        for _ in 0..hz * 2 {
            p = tick(&scene, &mut body, p, Intent::default(), true, rate);
            maximum = maximum.max(p[1]);
        }
        assert!((1.20..1.24).contains(&maximum), "{hz}: {maximum}");
        assert!(
            body.grounded() && p[1] < 0.01,
            "Held jump cannot automatically retrigger"
        );
        p = tick(&scene, &mut body, p, Intent::default(), false, rate);
        let next = tick(&scene, &mut body, p, Intent::default(), true, rate);
        assert!(next[1] > 0.1 && !body.grounded());
        let mut fall = Motion::default();
        let mut position = [0., 5., 0.];
        for _ in 0..hz * 2 {
            position = tick(&scene, &mut fall, position, Intent::default(), false, rate);
        }
        assert!(fall.grounded() && (position[1] - 0.008).abs() < 0.002);
    }
}
#[test]
fn thin_wall_blocks_capsule_allows_sliding_and_retreat() {
    let scene = StaticScene::new(vec![floor(), block([2., 2., 0.], [0.02, 4., 16.])]);
    let mut body = Motion::default();
    let mut p = [0., 0., 0.];
    for _ in 0..30 {
        p = tick(
            &scene,
            &mut body,
            p,
            walk([1000, 1000]),
            false,
            TickRate::Hz30,
        );
    }
    assert!(p[0] > 1.63 && p[0] < 1.641 && p[2] > 3.4, "{p:?}");
    let before = p;
    p = tick(
        &scene,
        &mut body,
        p,
        walk([-1000, 0]),
        false,
        TickRate::Hz30,
    );
    assert!(p[0] < before[0] - 0.16);
}
#[test]
fn steps_require_height_and_full_capsule_headroom() {
    for (height, ceiling, can_step) in [
        (0.25, false, true),
        (0.6, false, false),
        (0.25, true, false),
    ] {
        let mut geometry = vec![floor(), block([2., height / 2., 0.], [2., height, 8.])];
        if ceiling {
            geometry.push(block([0., 1.95, 0.], [10., 0.2, 10.]));
        }
        let scene = StaticScene::new(geometry);
        let mut body = Motion::default();
        let mut p = [0., 0., 0.];
        let mut maximum = 0f32;
        for _ in 0..15 {
            p = tick(&scene, &mut body, p, walk([1000, 0]), false, TickRate::Hz30);
            maximum = maximum.max(p[1]);
        }
        if can_step {
            assert!(p[0] > 2.4 && maximum > 0.24, "{p:?}");
        } else {
            assert!(p[0] < 0.66 && maximum < 0.01, "{height}/{ceiling}: {p:?}");
        }
    }
}
#[test]
fn stacked_floors_do_not_teleport_lower_actor_to_highest_surface() {
    let scene = StaticScene::new(vec![floor(), block([0., 3., 0.], [12., 0.2, 12.])]);
    let mut lower = Motion::default();
    let mut upper = Motion::default();
    let mut a = [0., 0., 0.];
    let mut b = [0., 3.1, 0.];
    for _ in 0..60 {
        a = tick(
            &scene,
            &mut lower,
            a,
            Intent::default(),
            false,
            TickRate::Hz30,
        );
        b = tick(
            &scene,
            &mut upper,
            b,
            Intent::default(),
            false,
            TickRate::Hz30,
        );
        assert!(a[1] < 0.01 && (b[1] - 3.108).abs() < 0.001);
    }
    let ceiling = StaticScene::new(vec![floor(), block([0., 1.95, 0.], [8., 0.2, 8.])]);
    let mut body = Motion::default();
    let mut p = [0., 0., 0.];
    let mut high = 0f32;
    for _ in 0..60 {
        p = tick(
            &ceiling,
            &mut body,
            p,
            Intent::default(),
            true,
            TickRate::Hz30,
        );
        high = high.max(p[1]);
    }
    assert!(high < 0.051 && body.grounded(), "{high}");
}
#[test]
fn large_convex_floor_support_never_snaps_a_valid_capsule_into_the_floor() {
    let scene = StaticScene::new(vec![block([120., -25.5, 136.], [512., 1., 512.])]);
    for hz in [20, 30, 60] {
        let mut body = Motion::default();
        let mut p = LocalPosition::new([120.355354, -24.992046, 135.52518]).unwrap();
        for _ in 0..hz {
            p = kinematic::advance(
                &scene,
                &mut body,
                p,
                walk([-1000, -1000]),
                TickRate::from_hz(hz).unwrap(),
            )
            .unwrap();
            assert!(
                p.metres()[1] >= -25.0001,
                "Support must not snap into the flat floor: {:?}",
                p.metres()
            );
        }
        assert!(body.grounded());
    }
}

#[test]
fn walkable_slope_climbs_steep_slope_blocks_uphill_and_falls_downhill() {
    for (slope, walkable) in [(0.5, true), (1.5, false)] {
        // Flat approach to a ramp rising from x=1; no artificial highest-surface ray.
        let vertices = vec![
            Vec3::new(1., 0., -10.),
            Vec3::new(1., 0., 10.),
            Vec3::new(11., 10. * slope, -10.),
            Vec3::new(11., 10. * slope, 10.),
        ];
        let ramp = PlacedCollider {
            shape: Collider::trimesh(vertices, vec![[0, 1, 2], [2, 1, 3]]),
            center: Vec3::ZERO,
            layer: GROUND | SOLID,
        };
        let scene = StaticScene::new(vec![floor(), ramp]);
        let mut body = Motion::default();
        let mut p = [0., 0., 0.];
        for _ in 0..30 {
            p = tick(&scene, &mut body, p, walk([1000, 0]), false, TickRate::Hz30);
        }
        if walkable {
            assert!(p[0] > 3. && p[1] > 1., "{p:?}");
        } else {
            assert!(p[0] < 1.2, "Steep ramp climbed: {p:?}");
        }
    }
}

#[test]
fn sprinting_uphill_jump_keeps_its_upward_launch() {
    let slope = 0.75;
    let scene = StaticScene::new(vec![PlacedCollider {
        shape: Collider::trimesh(
            vec![
                Vec3::new(-10., -10. * slope, -10.),
                Vec3::new(-10., -10. * slope, 10.),
                Vec3::new(10., 10. * slope, -10.),
                Vec3::new(10., 10. * slope, 10.),
            ],
            vec![[0, 1, 2], [2, 1, 3]],
        ),
        center: Vec3::ZERO,
        layer: GROUND | SOLID,
    }]);
    for hz in [20, 30, 60] {
        let mut motion = Motion::default();
        let start = kinematic::spawn(&scene, LocalPosition::new([-4., -3., 0.]).unwrap()).unwrap();
        let intent = Intent {
            axes: [1000, 0],
            sprint: true,
            jump: true,
        };
        let feet = kinematic::advance(
            &scene,
            &mut motion,
            start,
            intent,
            TickRate::from_hz(hz).unwrap(),
        )
        .unwrap();
        assert!(
            !motion.grounded(),
            "Upward jump was grounded at {hz} Hz: {start:?} -> {feet:?}"
        );
        assert!(motion.vertical_speed() > 0.);
        assert!(feet.metres()[1] > start.metres()[1]);
        kinematic::validate_pose(&scene, feet, false).unwrap();
    }
}
#[test]
fn walking_off_ledge_falls_and_other_actors_never_enter_collision_set() {
    let scene = StaticScene::new(vec![block([0., 2., 0.], [2., 0.2, 8.]), floor()]);
    let mut a = Motion::default();
    let mut b = Motion::default();
    let mut p = [0., 2.1, 0.];
    let mut q = p;
    let mut fell = false;
    for _ in 0..60 {
        p = tick(&scene, &mut a, p, walk([1000, 0]), false, TickRate::Hz30);
        q = tick(&scene, &mut b, q, walk([1000, 0]), false, TickRate::Hz30);
        assert_eq!(p, q);
        fell |= !a.grounded() && a.vertical_speed() < 0.;
    }
    assert!(fell && a.grounded() && p[1] < 0.01);
    assert_eq!(scene.colliders.len(), 2);
}
#[test]
fn invalid_input_and_penetrating_starts_fail_without_mutating_state() {
    let scene = StaticScene::new(vec![floor(), block([2., 1., 0.], [1., 2., 1.])]);
    let mut body = Motion::default();
    assert!(
        kinematic::advance(
            &scene,
            &mut body,
            LocalPosition::new([0., 0., 0.]).unwrap(),
            walk([1001, 0]),
            TickRate::Hz30
        )
        .is_err()
    );
    assert!(
        kinematic::advance(
            &scene,
            &mut body,
            LocalPosition::new([2., 0., 0.]).unwrap(),
            Intent::default(),
            TickRate::Hz30
        )
        .is_err()
    );
    assert_eq!(body.vertical_speed(), 0.);
    assert!(!body.grounded());
}

#[test]
fn steep_surface_cannot_hold_an_idle_actor_and_combined_mesh_wall_still_blocks() {
    let ramp = PlacedCollider {
        shape: Collider::trimesh(
            vec![
                Vec3::new(1., 0., -10.),
                Vec3::new(1., 0., 10.),
                Vec3::new(11., 15., -10.),
                Vec3::new(11., 15., 10.),
            ],
            vec![[0, 1, 2], [2, 1, 3]],
        ),
        center: Vec3::ZERO,
        layer: GROUND | SOLID,
    };
    let scene = StaticScene::new(vec![floor(), ramp]);
    let mut body = Motion::default();
    let mut p = [5., 10., 0.];
    let mut saw_slide = false;
    for _ in 0..180 {
        p = tick(
            &scene,
            &mut body,
            p,
            Intent::default(),
            false,
            TickRate::Hz30,
        );
        saw_slide |= p[0] < 4.5 && !body.grounded();
    }
    assert!(
        saw_slide && body.grounded() && p[1] < 0.01,
        "{p:?} {body:?}"
    );
    let mesh = PlacedCollider {
        shape: Collider::trimesh(
            vec![
                Vec3::new(-20., 0., -20.),
                Vec3::new(-20., 0., 20.),
                Vec3::new(20., 0., -20.),
                Vec3::new(20., 0., 20.),
                Vec3::new(2., 0., -8.),
                Vec3::new(2., 4., -8.),
                Vec3::new(2., 4., 8.),
                Vec3::new(2., 0., 8.),
            ],
            vec![[0, 1, 2], [2, 1, 3], [4, 6, 5], [4, 7, 6]],
        ),
        center: Vec3::ZERO,
        layer: GROUND | SOLID,
    };
    let scene = StaticScene::new(vec![mesh]);
    let mut body = Motion::default();
    let mut p = [0., 0., 0.];
    for _ in 0..30 {
        p = tick(&scene, &mut body, p, walk([1000, 0]), false, TickRate::Hz30);
    }
    assert!(
        p[0] > 1.63 && p[0] < 1.65,
        "Combined floor/wall mesh failed: {p:?}"
    );
}

#[test]
fn signed_contact_depth_distinguishes_near_floor_clearance_from_penetration() {
    let scene = StaticScene::new(vec![floor()]);
    let capsule = Collider::capsule(0.35, 1.1);
    for x in [0., 0.25, 1., 5.] {
        let feet = 0.008;
        assert!(!scene.penetrates(
            &capsule,
            Vec3::new(x, 0.9 + feet, 0.),
            GROUND | SOLID,
            0.0001
        ));
        assert!(scene.penetrates(&capsule, Vec3::new(x, 0.8, 0.), GROUND | SOLID, 0.0001));
    }
}

#[test]
fn safe_spawn_resolves_low_static_road_and_contact_does_not_disable_grounding() {
    let scene = StaticScene::new(vec![
        floor(),
        block([0., 0.015, 0.], [3., 0.03, 10.]),
        block([2., 2., 0.], [0.02, 4., 16.]),
    ]);
    let feet = kinematic::spawn(&scene, LocalPosition::new([0., 0., 0.]).unwrap())
        .unwrap()
        .metres();
    assert!((feet[1] - 0.038).abs() < 0.002);
    let mut body = Motion::default();
    let mut p = feet;
    for _ in 0..60 {
        p = tick(&scene, &mut body, p, walk([1000, 0]), false, TickRate::Hz30);
        assert!(body.grounded(), "Ground contact was lost next to the wall");
    }
    assert!(p[0] < 1.65);
}

#[test]
fn exact_initial_wall_contact_blocks_input_and_permits_retreat() {
    let scene = theia_network::world::physics_scene();
    let feet = kinematic::spawn(scene, LocalPosition::new([-2.65, 0., -7.]).unwrap())
        .unwrap()
        .metres();
    let mut body = Motion::default();
    let p = tick(
        scene,
        &mut body,
        feet,
        Intent {
            axes: [-1000, 0],
            sprint: true,
            jump: false,
        },
        false,
        TickRate::Hz60,
    );
    assert!((p[0] - feet[0]).abs() < 0.001, "{feet:?} -> {p:?}");
    let retreated = tick(scene, &mut body, p, walk([1000, 0]), false, TickRate::Hz60);
    assert!(retreated[0] > p[0] + 0.08);
}

#[test]
fn sloped_ground_wall_contacts_remain_fixed_under_persistent_head_on_input() {
    let a = [-24., -2.4, -24.];
    let b = [24., 2.4, -24.];
    let c = [24., 2.4, 24.];
    let d = [-24., -2.4, 24.];
    let terrain = theia_network::terrain::Terrain::decode(
        &serde_json::to_vec(&serde_json::json!({
            "schema":1,"source_origin":[0,0,0],"visual_sha256":vec![0;32],
            "triangles":[[a,c,b],[a,d,c]],
            "solids":[{"center":[-8,3,-8],"size":[8.6,10,8.6]}]
        }))
        .unwrap(),
    )
    .unwrap();
    for hz in [20, 30, 60] {
        let rate = TickRate::from_hz(hz).unwrap();
        // Include both normal fixed-step approach positions and the exact
        // lateral pose from the failed real-network regression.
        for x in [-7.6, -7.75, -7.83333, -7.916663, -8., -8.166663] {
            let mut body = Motion::default();
            let mut p = kinematic::spawn(
                &terrain.physics,
                LocalPosition::new([x, x * 0.1, 0.]).unwrap(),
            )
            .unwrap()
            .metres();
            for _ in 0..hz * 2 {
                p = tick(
                    &terrain.physics,
                    &mut body,
                    p,
                    walk([0, -1000]),
                    false,
                    rate,
                );
                if p[2] < -3.34 {
                    break;
                }
            }
            assert!(p[2] < -3.34, "Approach did not reach wall: {p:?}");
            let contact = p;
            for _ in 0..hz * 2 {
                p = tick(
                    &terrain.physics,
                    &mut body,
                    p,
                    walk([0, -1000]),
                    false,
                    rate,
                );
                assert_eq!(
                    [p[0], p[2]],
                    [contact[0], contact[2]],
                    "{hz} Hz, x={x}: {contact:?} -> {p:?}"
                );
                assert!(body.grounded());
            }
            let tangent = tick(
                &terrain.physics,
                &mut body,
                p,
                walk([1, -1000]),
                false,
                rate,
            );
            assert!(
                tangent[0] > p[0] + 0.00007,
                "Small intentional tangential input was suppressed: {p:?} -> {tangent:?}"
            );
            let retreat = tick(
                &terrain.physics,
                &mut body,
                tangent,
                walk([0, 1000]),
                false,
                rate,
            );
            assert!(retreat[2] > p[2] + 0.07);
        }
    }
}

#[test]
fn exact_touch_keeps_tangential_motion_and_diagonal_slide_on_both_sides() {
    let scene = theia_network::world::physics_scene();
    for z in [-9., -7., -5.] {
        for axes in [[0, -1000], [0, 1000], [-1000, -1000], [-1000, 1000]] {
            let mut p = kinematic::spawn(scene, LocalPosition::new([-2.65, 0., z]).unwrap())
                .unwrap()
                .metres();
            let initial = p;
            let mut body = Motion::default();
            for _ in 0..6 {
                p = tick(scene, &mut body, p, walk(axes), false, TickRate::Hz60);
            }
            assert!(
                (p[0] - initial[0]).abs() < 0.001,
                "{z} {axes:?}: {initial:?} -> {p:?}"
            );
            assert!(
                (p[2] - initial[2]).abs() > 0.34,
                "Tangential motion blocked: {axes:?} {p:?}"
            );
        }
    }
}
