use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    commands::Timeline,
    dodge_input::{EDGE_LIFETIME, Input},
    dodging::{self, Controller, Profile},
    kinematic::MotionState,
    prediction::{Correction, Pose},
    procedural,
};
fn profile() -> Arc<Profile> {
    Arc::new(
        Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-dodge.json"
        ))
        .unwrap(),
    )
}
fn correction() -> Correction {
    Correction {
        world: dodging::profile_identity(procedural::IDENTITY, &profile()),
        actor: EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(1).unwrap(),
        },
        rate: TickRate::Hz30,
        input: Timeline::new(1, TickRate::Hz30, 0).unwrap().state(),
        ready_sequence: 1,
        pose: Pose {
            facing: None,
            feet: GlobalPosition::new([8000., 0., 8000.]).unwrap(),
            motion: MotionState {
                vertical_speed: 0.,
                grounded: true,
                jump_held: false,
            },
            dodge: Some(Controller::new(&profile(), TickRate::Hz30, 0).snapshot()),
        },
    }
}
#[test]
fn edge_repeats_only_until_ack_or_fixed_expiry_and_a_held_key_cannot_repeat() {
    let now = Instant::now();
    let mut c = correction();
    let mut input = Input::new(procedural::IDENTITY, profile());
    assert_eq!(input.update(Some(c), true, None, now).unwrap(), None);
    let edge = input
        .update(Some(c), true, Some([1000, 0]), now)
        .unwrap()
        .unwrap();
    assert_eq!(edge.nonce, 1);
    assert_eq!(
        input
            .update(Some(c), true, None, now + Duration::from_millis(199))
            .unwrap(),
        Some(edge)
    );
    assert_eq!(
        input
            .update(Some(c), true, None, now + EDGE_LIFETIME)
            .unwrap(),
        None
    );
    let next = input
        .update(
            Some(c),
            true,
            Some([0, 1000]),
            now + Duration::from_millis(201),
        )
        .unwrap()
        .unwrap();
    assert_eq!(next.nonce, 2);
    c.pose.dodge.as_mut().unwrap().last_nonce = 2;
    assert_eq!(
        input
            .update(Some(c), true, None, now + Duration::from_millis(202))
            .unwrap(),
        None
    );
    for n in 203..600 {
        assert_eq!(
            input
                .update(Some(c), true, None, now + Duration::from_millis(n))
                .unwrap(),
            None
        );
    }
}
#[test]
fn focus_panel_readiness_loss_stream_actor_and_disconnect_changes_discard_pending_edges() {
    for change in 0..5 {
        let now = Instant::now();
        let mut c = correction();
        let mut input = Input::new(procedural::IDENTITY, profile());
        input.update(Some(c), true, None, now).unwrap();
        assert!(
            input
                .update(Some(c), true, Some([1000, 0]), now)
                .unwrap()
                .is_some()
        );
        match change {
            0 => {
                input.update(Some(c), false, None, now).unwrap();
            }
            1 => c.input.stream_epoch = 2,
            2 => c.actor.entity = EntityId::new(2).unwrap(),
            3 => {
                input.update(None, false, None, now).unwrap();
            }
            _ => {
                input.update(Some(c), false, Some([1000, 0]), now).unwrap();
            }
        }
        assert_eq!(input.update(Some(c), true, None, now).unwrap(), None);
        assert_eq!(
            input
                .update(Some(c), true, Some([1000, 0]), now)
                .unwrap()
                .unwrap()
                .nonce,
            2
        );
    }
}
#[test]
fn airborne_press_does_not_queue_a_dodge_for_landing() {
    let now = Instant::now();
    let mut c = correction();
    let mut input = Input::new(procedural::IDENTITY, profile());
    input.update(Some(c), true, None, now).unwrap();
    c.pose.motion.grounded = false;
    assert_eq!(
        input.update(Some(c), true, Some([1000, 0]), now).unwrap(),
        None
    );
    c.pose.motion.grounded = true;
    assert_eq!(
        input
            .update(Some(c), true, None, now + Duration::from_millis(10))
            .unwrap(),
        None
    );
}
#[test]
fn invalid_bindings_directions_and_nonce_exhaustion_fail_without_wraparound() {
    let now = Instant::now();
    let mut c = correction();
    let mut input = Input::new(procedural::IDENTITY, profile());
    input.update(Some(c), true, None, now).unwrap();
    assert!(input.update(Some(c), true, Some([0, 0]), now).is_err());
    assert_eq!(
        input
            .update(Some(c), true, Some([1000, 0]), now)
            .unwrap()
            .unwrap()
            .nonce,
        1
    );
    c.world = [0; 32];
    assert!(input.update(Some(c), true, None, now).is_err());
    let mut c = correction();
    c.pose.dodge.as_mut().unwrap().last_nonce = u64::MAX;
    input.update(Some(c), true, None, now).unwrap();
    assert!(input.update(Some(c), true, Some([1000, 0]), now).is_err());
    assert_eq!(input.update(Some(c), true, None, now).unwrap(), None);
}
