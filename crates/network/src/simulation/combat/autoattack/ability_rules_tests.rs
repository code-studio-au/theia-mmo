use super::*;
use crate::simulation::combat::{self, Reason};
use serde_json::{Value, json};
const STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-character-stats.json");
fn profile(rule: Value, cast_ms: u32, minimum: f64) -> Vec<u8> {
    serde_json::to_vec(&json!({"schema":7,"global_cooldown_ms":0,"queue_window_ms":300,"abilities":[{
        "id":"theia.test.rules","target":"hostile_alive","range_metres":30.0,"cast_ms":cast_ms,"cooldown_ms":0,
        "cast_while_moving":true,"uses_global_cooldown":false,"mana_cost":1,"requires_main_hand":"theia.dev.training-sword",
        "effect":{"kind":"damage","amount":1},"combat":{"minimum_range_metres":minimum,"facing":{"kind":"frontal","arc_degrees":180.0},"autoattack":rule}
    }]})).unwrap()
}
fn setup(rate: TickRate, rule: Value, cast_ms: u32, minimum: f64) -> (World, u64, EntityKey) {
    setup_mobile(rate, rule, cast_ms, minimum, true)
}
fn setup_mobile(
    rate: TickRate,
    rule: Value,
    cast_ms: u32,
    minimum: f64,
    mobile: bool,
) -> (World, u64, EntityKey) {
    let mut data: Value = serde_json::from_slice(&profile(rule, cast_ms, minimum)).unwrap();
    data["abilities"][0]["cast_while_moving"] = json!(mobile);
    let data = serde_json::to_vec(&data).unwrap();
    let mut w = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(w.set_tick_rate(rate));
    w.enable_prediction().unwrap();
    w.enable_classic().unwrap();
    let target = w.spawn_development_target([8000., 7998.]).unwrap();
    w.enable_status_combat(
        &data,
        10000,
        5000,
        STATS,
        include_bytes!("../../../../../../content/gameplay/development-statuses.json"),
    )
    .unwrap();
    w.enable_autoattack(WEAPONS, ITEMS).unwrap();
    let id = w.join().unwrap();
    ready(&mut w, id, 1);
    gear(&mut w, id, 1, Some("theia.dev.training-sword"));
    (w, id, target)
}
fn cast(w: &mut World, id: u64, target: EntityKey, sequence: u64) -> combat::CastResult {
    w.cast(
        id,
        combat::Request {
            epoch: w.epoch(),
            sequence,
            ability: ContentId::new("theia.test.rules").unwrap(),
            target,
        },
    )
}
fn advance(w: &mut World, tick: u64) {
    while w.tick < tick {
        w.step();
    }
}
fn mana(w: &World, id: u64) -> u32 {
    w.combat_snapshot(id).unwrap().vitals.unwrap().mana()
}
#[test]
fn all_four_rules_use_real_weapon_clocks_and_cast_completion_at_every_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for kind in ["continue", "suspend", "delay", "reset"] {
            let rule = if kind == "delay" {
                json!({"kind":kind,"delay_ms":500})
            } else {
                json!({"kind":kind})
            };
            let (mut w, id, target) = setup(rate, rule, 3000, 0.0);
            request(&mut w, id, 1, Some(target));
            w.step();
            let before = w.autoattack_snapshot(id).unwrap();
            let original = before.ready_at.unwrap();
            let start = w.tick;
            let result = cast(&mut w, id, target, 1);
            let combat::Outcome::Started { completes } = result.reply.outcome else {
                panic!("{:?}", result.reply);
            };
            let delayed = rate
                .ticks_for(std::time::Duration::from_millis(500))
                .unwrap();
            assert_eq!(
                w.autoattack_snapshot(id).unwrap().ready_at,
                Some(match kind {
                    "delay" => original + delayed,
                    "reset" =>
                        start
                            + rate
                                .ticks_for(std::time::Duration::from_millis(2400))
                                .unwrap(),
                    _ => original,
                })
            );
            if kind == "continue" {
                advance(&mut w, original);
                assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 2);
                assert!(w.combat_snapshot(id).unwrap().cast.is_some());
            } else {
                advance(&mut w, completes - 1);
                assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 1);
                w.step();
                if kind == "suspend" {
                    let resumed = completes + original - start;
                    assert_eq!(w.autoattack_snapshot(id).unwrap().ready_at, Some(resumed));
                    assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 1);
                    advance(&mut w, resumed);
                }
                assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 2);
            }
            assert_eq!(w.targeting_snapshot(id).unwrap().selected, Some(target));
        }
    }
}
#[test]
fn facing_is_checked_on_request_and_completion_without_cost_or_clock_on_rejection() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"suspend"}), 300, 0.0);
        request(&mut w, id, 1, Some(target));
        w.step();
        turn(&mut w, id, 32768);
        let before = w.autoattack_snapshot(id).unwrap();
        let balance = mana(&w, id);
        assert_eq!(
            cast(&mut w, id, target, 1).reply.outcome,
            combat::Outcome::Rejected(Reason::NotFacing)
        );
        assert_eq!(mana(&w, id), balance);
        assert_eq!(w.autoattack_snapshot(id).unwrap(), before);
        turn(&mut w, id, 0);
        let result = cast(&mut w, id, target, 2);
        let combat::Outcome::Started { completes } = result.reply.outcome else {
            panic!()
        };
        let health_before = health(&w, target);
        turn(&mut w, id, 32768);
        advance(&mut w, completes);
        assert_eq!(health(&w, target), health_before);
        assert!(w.combat_events().iter().any(|e| matches!(
            e,
            Event::Cancelled {
                reason: Reason::NotFacing,
                ..
            }
        )));
        assert_eq!(w.targeting_snapshot(id).unwrap().selected, Some(target));
    }
}
#[test]
fn minimum_range_rejects_before_spending_or_building_a_clock() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"reset"}), 0, 10.0);
        let before = mana(&w, id);
        let pose = w.global_position(id);
        assert_eq!(
            cast(&mut w, id, target, 1).reply.outcome,
            combat::Outcome::Rejected(Reason::TooClose)
        );
        assert_eq!(mana(&w, id), before);
        assert_eq!(w.global_position(id), pose);
        assert_eq!(w.autoattack_snapshot(id).unwrap().ready_at, None);
    }
}
#[test]
fn queued_intent_changes_the_timer_only_at_actual_start_and_replay_never_delays_again() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"delay","delay_ms":500}), 1000, 0.0);
        request(&mut w, id, 1, Some(target));
        w.step();
        let first = cast(&mut w, id, target, 1);
        let combat::Outcome::Started { completes } = first.reply.outcome else {
            panic!()
        };
        let deadline = w.autoattack_snapshot(id).unwrap().ready_at.unwrap();
        let window = rate
            .ticks_for(std::time::Duration::from_millis(200))
            .unwrap();
        advance(&mut w, completes - window);
        assert!(matches!(
            cast(&mut w, id, target, 2).reply.outcome,
            combat::Outcome::Queued { .. }
        ));
        assert_eq!(w.autoattack_snapshot(id).unwrap().ready_at, Some(deadline));
        advance(&mut w, completes);
        let delayed = rate
            .ticks_for(std::time::Duration::from_millis(500))
            .unwrap();
        assert_eq!(
            w.autoattack_snapshot(id).unwrap().ready_at,
            Some(deadline + delayed)
        );
        let before = w.autoattack_snapshot(id).unwrap();
        assert_eq!(
            cast(&mut w, id, target, 2).reply.outcome,
            combat::Outcome::Ignored
        );
        assert_eq!(w.autoattack_snapshot(id).unwrap(), before);
    }
}
#[test]
fn clearing_during_suspend_still_resumes_the_inactive_clock_when_cast_ends() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"suspend"}), 1000, 0.0);
        request(&mut w, id, 1, Some(target));
        w.step();
        let start = w.tick;
        let ready_at = w.autoattack_snapshot(id).unwrap().ready_at.unwrap();
        let result = cast(&mut w, id, target, 1);
        let combat::Outcome::Started { completes } = result.reply.outcome else {
            panic!()
        };
        w.step();
        request(&mut w, id, 2, None);
        assert!(w.select_target(id, None));
        advance(&mut w, completes);
        assert_eq!(
            w.autoattack_snapshot(id).unwrap().ready_at,
            Some(completes + ready_at - start)
        );
        assert_eq!(w.autoattack_snapshot(id).unwrap().target, None);
        advance(&mut w, completes + ready_at - start);
        request(&mut w, id, 3, Some(target));
        w.step();
        assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 2);
    }
}
#[test]
fn repeated_delay_is_bounded_by_total_future_readiness_before_mana_is_spent() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"delay","delay_ms":10000}), 0, 0.0);
        request(&mut w, id, 1, Some(target));
        w.step();
        let mut rejected = false;
        for sequence in 1..=32 {
            let before = w.autoattack_snapshot(id).unwrap();
            let balance = mana(&w, id);
            let result = cast(&mut w, id, target, sequence);
            if result.reply.outcome == combat::Outcome::Rejected(Reason::TimeExhausted) {
                assert_eq!(mana(&w, id), balance);
                assert_eq!(w.autoattack_snapshot(id).unwrap(), before);
                rejected = true;
                break;
            }
            assert!(
                matches!(result.reply.outcome, combat::Outcome::Started { .. }),
                "{:?}",
                result.reply
            );
            w.step();
            let view = w.autoattack_snapshot(id).unwrap();
            assert!(view.ready_at.unwrap().saturating_sub(w.tick) <= MAX_READY_AHEAD_TICKS);
        }
        assert!(rejected);
    }
}

#[test]
fn minimum_range_boundary_uses_authoritative_metric_centres() {
    for distance in [2.999, 3.0, 3.001] {
        let (mut w, id, target) = setup(TickRate::Hz30, json!({"kind":"continue"}), 0, 3.0);
        // Trusted geometry fixture only; clients have no position setter.
        let mut p = w.global_position(id).unwrap().metres();
        p[2] -= distance;
        let point = GlobalPosition::new(p).unwrap();
        w.targets.get_mut(&target.entity.value()).unwrap().position = point;
        w.combat.as_mut().unwrap().creatures[0].position = point;
        let outcome = cast(&mut w, id, target, 1).reply.outcome;
        if distance < 3.0 {
            assert_eq!(outcome, combat::Outcome::Rejected(Reason::TooClose));
        } else {
            assert!(
                matches!(outcome, combat::Outcome::Started { .. }),
                "{outcome:?}"
            );
        }
    }
}
#[test]
fn queued_start_rechecks_facing_without_a_second_cost() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"suspend"}), 500, 0.0);
        let result = cast(&mut w, id, target, 1);
        let combat::Outcome::Started { completes } = result.reply.outcome else {
            panic!()
        };
        let window = rate
            .ticks_for(std::time::Duration::from_millis(200))
            .unwrap();
        advance(&mut w, completes - window);
        assert!(matches!(
            cast(&mut w, id, target, 2).reply.outcome,
            combat::Outcome::Queued { .. }
        ));
        turn(&mut w, id, 32768);
        advance(&mut w, completes - 1);
        let before = mana(&w, id);
        let health_before = health(&w, target);
        w.step();
        assert!(mana(&w, id) >= before);
        assert_eq!(health(&w, target), health_before);
        assert_eq!(
            w.combat_events()
                .iter()
                .filter(|e| matches!(
                    e,
                    Event::Cancelled {
                        reason: Reason::NotFacing,
                        ..
                    }
                ))
                .count(),
            2
        );
        assert!(w.combat_snapshot(id).unwrap().cast.is_none());
        assert!(w.combat_snapshot(id).unwrap().queued.is_none());
    }
}
#[test]
fn committed_gear_withdrawal_cancels_a_suspending_cast_and_resumes_remaining_time() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"suspend"}), 1000, 0.0);
        request(&mut w, id, 1, Some(target));
        w.step();
        let start = w.tick;
        let ready_at = w.autoattack_snapshot(id).unwrap().ready_at.unwrap();
        assert!(matches!(
            cast(&mut w, id, target, 1).reply.outcome,
            combat::Outcome::Started { .. }
        ));
        w.observe_combat_equipment(w.entity_key(id).unwrap(), None);
        w.step();
        let cancelled = w.tick;
        assert!(w.combat_events().iter().any(|e| matches!(
            e,
            Event::Cancelled {
                reason: Reason::EquipmentRequired,
                ..
            }
        )));
        assert_eq!(
            w.autoattack_snapshot(id).unwrap().ready_at,
            Some(cancelled + ready_at - start)
        );
        gear(&mut w, id, 2, Some("theia.dev.training-sword"));
        advance(&mut w, cancelled + ready_at - start);
        assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 2);
    }
}

#[test]
fn a_real_swap_through_a_non_melee_item_resets_but_loading_and_same_item_do_not() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup(rate, json!({"kind":"continue"}), 0, 0.0);
        request(&mut w, id, 1, Some(target));
        w.step();
        let original = w.autoattack_snapshot(id).unwrap().ready_at.unwrap();
        advance(&mut w, original - 2);
        w.observe_combat_equipment(w.entity_key(id).unwrap(), None);
        gear(&mut w, id, 2, Some("theia.dev.training-sword"));
        assert_eq!(w.autoattack_snapshot(id).unwrap().ready_at, Some(original));
        gear(&mut w, id, 3, Some("theia.dev.training-wand"));
        w.step();
        gear(&mut w, id, 4, Some("theia.dev.training-sword"));
        let period = rate
            .ticks_for(std::time::Duration::from_millis(2400))
            .unwrap();
        assert_eq!(
            w.autoattack_snapshot(id).unwrap().ready_at,
            Some(w.tick + period)
        );
        w.step();
        assert_eq!(w.autoattack_snapshot(id).unwrap().swing_sequence, 1);
    }
}

#[test]
fn stationary_ability_rejects_motion_and_interrupts_then_resumes_a_suspended_clock() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, target) = setup_mobile(rate, json!({"kind":"suspend"}), 500, 0.0, false);
        request(&mut w, id, 1, Some(target));
        w.step();
        let command = |w: &mut World, axes: [i16; 2]| {
            let c = w.correction(id).unwrap().state;
            assert_eq!(
                w.command(
                    id,
                    &Commands {
                        epoch: w.epoch(),
                        stream_epoch: c.input.stream_epoch,
                        ready_sequence: c.ready_sequence,
                        commands: vec![Command {
                            sequence: c.input.applied_sequence + 1,
                            tick: c.input.tick + 1,
                            intent: Intent {
                                facing: Some(Facing::from_units(0)),
                                axes,
                                ..Default::default()
                            }
                        }]
                    }
                ),
                1
            );
            w.step();
        };
        command(&mut w, [1000, 0]);
        let balance = mana(&w, id);
        let before = w.autoattack_snapshot(id).unwrap();
        assert_eq!(
            cast(&mut w, id, target, 1).reply.outcome,
            combat::Outcome::Rejected(Reason::Moving)
        );
        assert_eq!(mana(&w, id), balance);
        assert_eq!(w.autoattack_snapshot(id).unwrap(), before);
        command(&mut w, [0, 0]);
        assert!(matches!(
            cast(&mut w, id, target, 2).reply.outcome,
            combat::Outcome::Started { .. }
        ));
        command(&mut w, [1000, 0]);
        assert!(w.combat_events().iter().any(|e| matches!(
            e,
            Event::Cancelled {
                reason: Reason::Moving,
                ..
            }
        )));
        assert!(w.combat_snapshot(id).unwrap().cast.is_none());
        let actor = &w.combat.as_ref().unwrap().actors[&id];
        assert!(!actor.autoattack.clock.unwrap().suspended());
        assert_eq!(w.targeting_snapshot(id).unwrap().selected, Some(target));
    }
}
