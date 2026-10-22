//! The future prediction command contract, independently qualified before wire adoption.
use theia_core::{coordinates::LocalPosition, time::TickRate};
use theia_network::{
    commands::{Command, FUTURE_TICKS, Receipt, Timeline},
    kinematic::{self, Motion},
    wire::Intent,
    world,
};
fn command(sequence: u64, tick: u64) -> Command {
    Command {
        sequence,
        tick,
        intent: Intent {
            axes: [100, 0],
            sprint: true,
            jump: (6..=30).contains(&tick),
        },
    }
}
#[test]
fn receipt_is_not_application_and_packets_cannot_advance_time() {
    let mut input = Timeline::new(7, TickRate::Hz30, 100).unwrap();
    let next = command(1, 103);
    assert_eq!(input.offer(7, next), Ok(Receipt::Queued));
    for _ in 0..1000 {
        assert_eq!(input.offer(7, next), Ok(Receipt::Duplicate));
    }
    assert_eq!(input.state().tick, 100);
    assert_eq!(input.state().applied_sequence, 0);
    assert_eq!(input.pending(), 1);
    assert_eq!(input.advance().unwrap().applied_sequence, 0);
    assert_eq!(input.advance().unwrap().applied_sequence, 0);
    let state = input.advance().unwrap();
    assert_eq!(state.tick, 103);
    assert_eq!(state.applied_sequence, 1);
    assert_eq!(input.pending(), 0);
    assert!(input.offer(7, next).is_err());
}
#[test]
fn reordered_future_commands_are_bounded_and_invalid_input_cannot_poison_progress() {
    let mut input = Timeline::new(7, TickRate::Hz30, 0).unwrap();
    assert!(input.offer(8, command(u64::MAX, 1)).is_err());
    assert!(input.offer(7, command(u64::MAX, FUTURE_TICKS + 1)).is_err());
    for n in (1..=FUTURE_TICKS).rev() {
        assert_eq!(input.offer(7, command(n, n)), Ok(Receipt::Queued));
    }
    assert_eq!(input.pending(), FUTURE_TICKS as usize);
    assert!(input.offer(7, command(u64::MAX, 2)).is_err());
    for n in 1..=FUTURE_TICKS {
        let state = input.advance().unwrap();
        assert_eq!(state.applied_sequence, n);
    }
    assert_eq!(input.pending(), 0);
    assert_eq!(input.offer(7, command(9, 9)), Ok(Receipt::Queued));
}
#[test]
fn held_input_expiry_uses_simulation_ticks_at_every_authority_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut input = Timeline::new(9, rate, 0).unwrap();
        input.offer(9, command(1, 1)).unwrap();
        let active = input.advance().unwrap();
        let timeout = rate
            .ticks_for(std::time::Duration::from_secs_f64(1. / 3.))
            .unwrap();
        for _ in 1..timeout {
            assert_eq!(input.advance().unwrap().intent, active.intent);
        }
        assert_eq!(input.advance().unwrap().intent, Intent::default());
    }
}
#[test]
fn completed_tick_and_motion_state_replay_exactly_with_held_input_and_missing_commands() {
    let scene = world::physics_scene();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut input = Timeline::new(17, rate, 0).unwrap();
        let mut motion = Motion::default();
        let mut feet = kinematic::spawn(scene, LocalPosition::new([0., 0., 0.]).unwrap()).unwrap();
        let mut history = Vec::new();
        for tick in 1..=80 {
            if tick % 3 != 0 {
                input.offer(17, command(tick, tick)).unwrap();
            }
            let applied = input.advance().unwrap();
            feet = kinematic::advance(scene, &mut motion, feet, applied.intent, rate).unwrap();
            history.push((applied, feet, motion.snapshot()));
        }
        let (state, mut feet, body) = history[9];
        let mut input = Timeline::restore(rate, state).unwrap();
        let mut motion = Motion::restore(body).unwrap();
        for tick in 11..=80 {
            if tick % 3 != 0 {
                input.offer(17, command(tick, tick)).unwrap();
            }
            let applied = input.advance().unwrap();
            feet = kinematic::advance(scene, &mut motion, feet, applied.intent, rate).unwrap();
            assert_eq!(
                (applied, feet, motion.snapshot()),
                history[tick as usize - 1]
            );
        }
    }
}

#[test]
fn prequeued_commands_and_retransmission_do_not_extend_receipt_based_input_expiry() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut input = Timeline::new(7, rate, 0).unwrap();
        let timeout = rate
            .ticks_for(std::time::Duration::from_secs_f64(1. / 3.))
            .unwrap();
        let window = FUTURE_TICKS.min(timeout);
        for tick in 1..=window {
            input.offer(7, command(tick, tick)).unwrap();
        }
        for _ in 0..timeout {
            let state = input.advance().unwrap();
            assert_ne!(state.intent, Intent::default());
            assert_eq!(state.fresh_tick, 0);
        }
        let expired = input.advance().unwrap();
        assert_eq!(expired.intent, Intent::default());
        assert_eq!(expired.applied_sequence, window);
        let mut invalid = expired;
        invalid.intent = command(1, 1).intent;
        assert!(Timeline::restore(rate, invalid).is_err());
    }
}

#[test]
fn final_representable_ticks_advance_without_wrapping_or_rejecting_valid_commands() {
    let mut input = Timeline::new(7, TickRate::Hz30, u64::MAX - 2).unwrap();
    input.offer(7, command(1, u64::MAX - 1)).unwrap();
    assert_eq!(input.advance().unwrap().tick, u64::MAX - 1);
    input.offer(7, command(2, u64::MAX)).unwrap();
    let last = input.advance().unwrap();
    assert_eq!(last.tick, u64::MAX);
    assert_eq!(last.applied_sequence, 2);
    assert!(input.advance().is_err());
    assert_eq!(input.state(), last);
    assert!(input.offer(7, command(3, 0)).is_err());
    assert_eq!(input.state(), last);
}
