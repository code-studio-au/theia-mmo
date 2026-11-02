//! Pure policy fixtures; no claim of actual networked pursuit.
use std::num::NonZeroU64;
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    pursuit::{self, Directive, Error, Mode, Observation, Settings, State, Target},
    time::TickRate,
};
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn p(x: f64) -> GlobalPosition {
    GlobalPosition::new([15000. + x, 10., 15000.]).unwrap()
}
fn target(id: u64, x: f64) -> Target {
    Target {
        key: LifeKey {
            actor: key(id),
            generation: NonZeroU64::new(1).unwrap(),
        },
        position: p(x),
        alive: true,
        visible: true,
        in_melee: false,
    }
}
fn settings() -> Settings {
    Settings {
        perception_metres: 12.,
        leash_metres: 35.,
        chase_speed: 5.,
        return_speed: 5.,
        los_grace_ms: 2000,
        engagement_ms: 30000,
        reacquire_ms: 2000,
    }
}
fn observe(
    state: &mut State,
    tick: u64,
    x: f64,
    candidates: &[Target],
    current: Option<Target>,
) -> Directive {
    state
        .advance(
            tick,
            Observation {
                position: p(x),
                alive: true,
                candidates,
                target: current,
                provoked: None,
                blocked: false,
            },
        )
        .unwrap()
}
#[test]
fn perception_is_bounded_nearest_visible_live_same_world_and_stably_tied() {
    let mut dead = target(2, 1.);
    dead.alive = false;
    let mut hidden = target(3, 2.);
    hidden.visible = false;
    let mut foreign = target(4, 3.);
    foreign.key.actor.epoch = WorldEpoch::new(8).unwrap();
    let candidates = [
        dead,
        hidden,
        foreign,
        target(5, 12.0001),
        target(7, 8.),
        target(6, -8.),
        target(1, 0.),
    ];
    assert_eq!(
        pursuit::select(key(1), p(0.), settings(), &candidates).unwrap(),
        Some(target(6, -8.))
    );
    assert!(
        pursuit::select(key(1), p(0.), settings(), &[target(2, 12.)])
            .unwrap()
            .is_some()
    );
    assert!(
        pursuit::select(key(1), p(0.), settings(), &[target(2, 12.0001)])
            .unwrap()
            .is_none()
    );
    assert_eq!(
        pursuit::select(
            key(1),
            p(0.),
            settings(),
            &vec![target(2, 1.); pursuit::MAX_CANDIDATES + 1]
        ),
        Err(Error::CandidateBudget)
    );
}
#[test]
fn chase_attack_leash_uses_fixed_patrol_anchor_and_returns_before_reacquisition() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut state = State::new(settings(), key(1), rate).unwrap();
        let acquired = observe(&mut state, 0, 0., &[target(2, 8.)], None);
        assert_eq!(acquired.mode, Mode::Chase);
        assert_eq!(acquired.anchor, Some(p(0.)));
        assert_eq!(acquired.goal, Some(p(8.)));
        let mut near = target(2, 10.);
        near.in_melee = true;
        let attack = observe(&mut state, 1, 8., &[], Some(near));
        assert_eq!(attack.mode, Mode::Attack);
        assert!(attack.can_attack);
        assert_eq!(attack.anchor, Some(p(0.)));
        let edge = observe(&mut state, 2, 30., &[], Some(target(2, 35.)));
        assert_eq!(edge.mode, Mode::Chase);
        assert_eq!(edge.anchor, Some(p(0.)));
        let evade = observe(&mut state, 3, 31., &[], Some(target(2, 35.001)));
        assert_eq!(evade.mode, Mode::Return);
        assert_eq!(evade.goal, Some(p(0.)));
        assert_eq!(evade.target, None);
        assert!(!evade.can_attack);
        assert_eq!(
            observe(&mut state, 4, 20., &[target(3, 21.)], Some(near)).mode,
            Mode::Return
        );
        assert_eq!(
            observe(&mut state, 5, 0., &[target(3, 1.)], None).mode,
            Mode::Patrol
        );
        assert_eq!(
            observe(&mut state, 6, 0., &[target(3, 1.)], None).mode,
            Mode::Patrol
        );
        let next = 5 + u64::from(rate.hz()) * 2;
        assert_eq!(
            observe(&mut state, next, 0., &[target(3, 1.)], None).target,
            Some(target(3, 1.).key)
        );
    }
}
#[test]
fn lost_los_pursues_last_seen_then_evades_on_exact_grace_boundary() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut state = State::new(settings(), key(1), rate).unwrap();
        observe(&mut state, 0, 0., &[target(2, 8.)], None);
        let mut hidden = target(2, 20.);
        hidden.visible = false;
        hidden.in_melee = true;
        let lost = observe(&mut state, 1, 0., &[], Some(hidden));
        assert_eq!(lost.goal, Some(p(8.)));
        assert!(!lost.can_attack);
        let deadline = 1 + u64::from(rate.hz()) * 2;
        assert_eq!(
            observe(&mut state, deadline - 1, 0., &[], Some(hidden)).mode,
            Mode::Chase
        );
        assert_eq!(
            observe(&mut state, deadline, 0., &[], Some(hidden)).mode,
            Mode::Return
        );
    }
}
#[test]
fn changed_target_life_disconnect_death_and_blocked_routes_terminate_engagement() {
    for variant in 0..5 {
        let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
        observe(&mut state, 0, 0., &[target(2, 8.)], None);
        let mut t = target(2, 8.);
        match variant {
            0 => t.key.generation = NonZeroU64::new(2).unwrap(),
            1 => t.alive = false,
            2 => t.key.actor = key(3),
            _ => {}
        }
        let observation = Observation {
            position: p(0.),
            alive: true,
            candidates: &[],
            target: if variant == 3 { None } else { Some(t) },
            provoked: None,
            blocked: variant == 4,
        };
        let result = state.advance(1, observation).unwrap();
        assert_eq!(result.mode, Mode::Return);
        assert_eq!(result.target, None);
        assert!(!result.can_attack);
    }
    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    observe(&mut state, 0, 0., &[target(2, 8.)], None);
    assert_eq!(
        state
            .advance(
                1,
                Observation {
                    position: p(0.),
                    alive: false,
                    candidates: &[],
                    target: None,
                    provoked: None,
                    blocked: false
                }
            )
            .unwrap()
            .mode,
        Mode::Dead
    );
    assert_eq!(
        observe(&mut state, 2, 0., &[target(3, 1.)], None).mode,
        Mode::Dead
    );
}
#[test]
fn engagement_and_return_timeouts_are_bounded_without_teleporting() {
    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    observe(&mut state, 0, 0., &[target(2, 8.)], None);
    assert_eq!(
        observe(&mut state, 900, 5., &[], Some(target(2, 8.))).mode,
        Mode::Return
    );
    let blocked = observe(&mut state, 1800, 5., &[], None);
    assert_eq!(blocked.mode, Mode::Blocked);
    assert_eq!(blocked.anchor, Some(p(0.)));
    assert_eq!(blocked.goal, None);
}
#[test]
fn invalid_settings_stale_ticks_duplicate_ticks_and_clock_exhaustion_fail_atomically() {
    let mut bad = settings();
    bad.leash_metres = 2.;
    assert!(!bad.valid());
    let mut bad = settings();
    bad.chase_speed = f32::NAN;
    assert!(!bad.valid());
    let mut bad = settings();
    bad.perception_metres = f64::INFINITY;
    assert!(!bad.valid());
    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    let acquired = observe(&mut state, 10, 0., &[target(2, 8.)], None);
    let before = state.clone();
    assert_eq!(
        observe(&mut state, 10, 0., &[target(3, 1.)], None),
        acquired
    );
    assert_eq!(state, before);
    assert_eq!(
        state.advance(
            9,
            Observation {
                position: p(0.),
                alive: true,
                candidates: &[],
                target: None,
                provoked: None,
                blocked: false
            }
        ),
        Err(Error::StaleTick)
    );
    assert_eq!(state, before);
    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    let before = state.clone();
    assert_eq!(
        state.advance(
            u64::MAX,
            Observation {
                position: p(0.),
                alive: true,
                candidates: &[target(2, 8.)],
                target: None,
                provoked: None,
                blocked: false
            }
        ),
        Err(Error::TimeExhausted)
    );
    assert_eq!(state, before);
}

#[test]
fn current_first_attacker_provokes_beyond_perception_without_switching_existing_target() {
    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    let first = target(2, 20.);
    let mut observation = Observation {
        position: p(0.),
        alive: true,
        candidates: &[target(3, 1.)],
        target: None,
        provoked: Some(first),
        blocked: false,
    };
    let acquired = state.advance(0, observation).unwrap();
    assert_eq!(acquired.target, Some(first.key));
    assert_eq!(acquired.mode, Mode::Chase);
    observation.provoked = Some(target(3, 1.));
    observation.target = Some(first);
    assert_eq!(
        state.advance(1, observation).unwrap().target,
        Some(first.key)
    );

    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    observation.provoked = Some(target(2, 35.01));
    observation.candidates = &[];
    assert_eq!(state.advance(0, observation).unwrap().mode, Mode::Patrol);
}

#[test]
fn restored_sight_resets_grace_and_creature_position_also_respects_original_anchor() {
    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    observe(&mut state, 0, 0., &[target(2, 8.)], None);
    let mut hidden = target(2, 20.);
    hidden.visible = false;
    observe(&mut state, 1, 0., &[], Some(hidden));
    assert_eq!(
        observe(&mut state, 60, 0., &[], Some(target(2, 10.))).goal,
        Some(p(10.))
    );
    assert_eq!(
        observe(&mut state, 61, 0., &[], Some(hidden)).mode,
        Mode::Chase
    );
    assert_eq!(
        observe(&mut state, 120, 0., &[], Some(hidden)).mode,
        Mode::Chase
    );
    assert_eq!(
        observe(&mut state, 121, 0., &[], Some(hidden)).mode,
        Mode::Return
    );

    let mut state = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    observe(&mut state, 0, 0., &[target(2, 8.)], None);
    let result = observe(&mut state, 1, 35.01, &[], Some(target(2, 10.)));
    assert_eq!(result.mode, Mode::Return);
    assert_eq!(result.goal, Some(p(0.)));
}

#[test]
fn melee_contact_refreshes_soft_allowance_but_never_moves_the_hard_anchor() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut state = State::new(settings(), key(1), rate).unwrap();
        let mut near = target(2, 2.);
        near.in_melee = true;
        observe(&mut state, 0, 0., &[near], None);
        let second = u64::from(rate.hz());
        // Three acquisition lifetimes, without requiring either side to land
        // damage: avoidance and slow attacks must not cause a melee reset.
        for tick in 1..=second * 90 {
            let d = observe(&mut state, tick, 0., &[], Some(near));
            assert_eq!(d.mode, Mode::Attack);
            assert_eq!(d.anchor, Some(p(0.)));
            assert_eq!(d.target, Some(near.key));
        }
        near.position = p(35.001);
        assert_eq!(
            observe(&mut state, second * 90 + 1, 34., &[], Some(near)).mode,
            Mode::Return
        );
    }
}

#[test]
fn leaving_melee_expires_from_last_contact_and_hidden_or_blocked_contact_cannot_refresh() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut state = State::new(settings(), key(1), rate).unwrap();
        let mut near = target(2, 2.);
        near.in_melee = true;
        observe(&mut state, 0, 0., &[near], None);
        let contact = u64::from(rate.hz()) * 20;
        assert_eq!(
            observe(&mut state, contact, 0., &[], Some(near)).mode,
            Mode::Attack
        );
        let until = contact + u64::from(rate.hz()) * 30;
        assert_eq!(
            observe(&mut state, until - 1, 0., &[], Some(target(2, 8.))).mode,
            Mode::Chase
        );
        assert_eq!(
            observe(&mut state, until, 0., &[], Some(target(2, 8.))).mode,
            Mode::Return
        );

        let mut state = State::new(settings(), key(1), rate).unwrap();
        observe(&mut state, 0, 0., &[near], None);
        near.visible = false;
        observe(&mut state, 1, 0., &[], Some(near));
        assert_eq!(
            observe(
                &mut state,
                1 + u64::from(rate.hz()) * 2,
                0.,
                &[],
                Some(near)
            )
            .mode,
            Mode::Return
        );

        let mut state = State::new(settings(), key(1), rate).unwrap();
        near.visible = true;
        observe(&mut state, 0, 0., &[near], None);
        assert_eq!(
            state
                .advance(
                    1,
                    Observation {
                        position: p(0.),
                        alive: true,
                        candidates: &[],
                        target: Some(near),
                        provoked: None,
                        blocked: true
                    }
                )
                .unwrap()
                .mode,
            Mode::Return
        );
    }
}

#[test]
fn explicit_transfer_preserves_original_anchor_and_soft_deadline_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut s = State::new(settings(), key(1), rate).unwrap();
        observe(&mut s, 0, 0., &[target(2, 8.)], None);
        let next = target(3, 25.);
        let switched = s.transfer(u64::from(rate.hz()) * 20, p(20.), next).unwrap();
        assert_eq!(switched.target, Some(next.key));
        assert_eq!(switched.anchor, Some(p(0.)));
        assert_eq!(switched.goal, Some(p(25.)));
        let expired = observe(&mut s, u64::from(rate.hz()) * 30, 20., &[], Some(next));
        assert_eq!(expired.mode, Mode::Return);
        assert_eq!(expired.goal, Some(p(0.)));
    }
}
#[test]
fn transfer_refuses_outside_original_boundary_invalid_lives_and_evaluated_ticks_atomically() {
    let mut s = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    observe(&mut s, 0, 0., &[target(2, 8.)], None);
    let before = s.clone();
    let mut dead = target(3, 2.);
    dead.alive = false;
    let mut hidden = target(3, 2.);
    hidden.visible = false;
    let mut foreign = target(3, 2.);
    foreign.key.actor.epoch = WorldEpoch::new(8).unwrap();
    for bad in [dead, hidden, foreign, target(1, 2.), target(3, 35.001)] {
        assert_eq!(s.transfer(1, p(20.), bad), Err(Error::InvalidTransfer));
        assert_eq!(s, before);
    }
    assert_eq!(
        s.transfer(1, p(35.001), target(3, 2.)),
        Err(Error::InvalidTransfer)
    );
    assert_eq!(s, before);
    assert_eq!(s.transfer(0, p(0.), target(3, 2.)), Err(Error::StaleTick));
    assert_eq!(s, before);
    let mut near = target(3, 2.);
    near.in_melee = true;
    assert_eq!(s.transfer(1, p(0.), near).unwrap().mode, Mode::Attack);
    assert_eq!(
        observe(&mut s, 1, 0., &[], Some(near)).target,
        Some(near.key)
    );
    let before = s.clone();
    assert_eq!(s.transfer(1, p(0.), target(4, 3.)), Err(Error::StaleTick));
    assert_eq!(s, before);
}
#[test]
fn transfer_cannot_start_patrol_or_revive_a_returning_blocked_or_dead_encounter() {
    let mut patrol = State::new(settings(), key(1), TickRate::Hz30).unwrap();
    let before = patrol.clone();
    assert_eq!(
        patrol.transfer(1, p(0.), target(3, 2.)),
        Err(Error::InvalidTransfer)
    );
    assert_eq!(patrol, before);
    let mut returning = patrol.clone();
    observe(&mut returning, 0, 0., &[target(2, 8.)], None);
    observe(&mut returning, 1, 10., &[], None);
    assert_eq!(returning.directive().mode, Mode::Return);
    let mut blocked = returning.clone();
    observe(&mut blocked, 902, 10., &[], None);
    assert_eq!(blocked.directive().mode, Mode::Blocked);
    let mut dead = patrol;
    dead.advance(
        0,
        Observation {
            position: p(0.),
            alive: false,
            candidates: &[],
            target: None,
            provoked: None,
            blocked: false,
        },
    )
    .unwrap();
    for mut state in [returning, blocked, dead] {
        let before = state.clone();
        assert_eq!(
            state.transfer(903, p(10.), target(3, 12.)),
            Err(Error::InvalidTransfer)
        );
        assert_eq!(state, before);
    }
}
