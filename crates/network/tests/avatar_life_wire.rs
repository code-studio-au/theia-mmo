use std::{num::NonZeroU64, sync::Arc};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    time::TickRate,
};
use theia_network::{
    avatar_life_wire::{self as wire, Actor, Applied, ClientState, Observation, Profile},
    player_life_wire,
};
fn profile(padded: bool) -> Arc<Profile> {
    let mut stats =
        include_bytes!("../../../content/gameplay/development-character-stats.json").to_vec();
    if padded {
        stats.push(b' ');
    }
    Arc::new(Profile::new(Arc::new(
        player_life_wire::Profile::from_json(
            &stats,
            include_bytes!("../../../content/gameplay/development-creature-deflect-pursuit.json"),
            include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
            include_bytes!("../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    )))
}
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(3).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn actor(id: u64, generation: u64, dead: bool) -> Actor {
    Actor {
        key: LifeKey {
            actor: key(id),
            generation: NonZeroU64::new(generation).unwrap(),
        },
        dead,
    }
}
fn observation(p: &Profile) -> Observation {
    Observation {
        profile: p.hash(),
        epoch: key(1).epoch,
        tick: 50,
        actors: vec![actor(1, 1, false), actor(2, 1, false)],
    }
}
#[test]
fn public_roster_round_trips_empty_through_maximum_with_fixed_budget_all_rates() {
    let p = profile(false);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for count in 0..=wire::MAX_ACTORS {
            let mut view = observation(&p);
            view.actors = (1..=count as u64)
                .map(|id| actor(id, 1, id % 2 == 0))
                .collect();
            let bytes = wire::encode(&p, rate, &view).unwrap();
            assert_eq!(bytes.len(), wire::HEADER_BYTES + wire::ACTOR_BYTES * count);
            assert!(bytes.len() <= 190);
            assert_eq!(wire::decode(&p, key(1).epoch, rate, &bytes).unwrap(), view);
        }
    }
    assert_eq!(wire::MAX_FRAME_BYTES, 190);
}
#[test]
fn roster_rejects_malformed_headers_flags_counts_order_keys_and_context() {
    let p = profile(false);
    let rate = TickRate::Hz30;
    let view = observation(&p);
    let bytes = wire::encode(&p, rate, &view).unwrap();
    for len in 0..bytes.len() {
        assert!(wire::decode(&p, key(1).epoch, rate, &bytes[..len]).is_err());
    }
    for (offset, value) in [
        (0, b'X'),
        (4, 25),
        (5, bytes[5] ^ 1),
        (37, 4),
        (53, 9),
        (54 + 16, 2),
    ] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        assert!(wire::decode(&p, key(1).epoch, rate, &bad).is_err());
    }
    for offset in [54, 62] {
        let mut bad = bytes.clone();
        bad[offset..offset + 8].fill(0);
        assert!(wire::decode(&p, key(1).epoch, rate, &bad).is_err());
    }
    let mut bad = bytes.clone();
    bad.extend([0]);
    assert!(wire::decode(&p, key(1).epoch, rate, &bad).is_err());
    assert!(wire::decode(&p, key(1).epoch, TickRate::Hz60, &bytes).is_err());
    assert!(wire::decode(&profile(true), key(1).epoch, rate, &bytes).is_err());
    for actors in [
        vec![actor(2, 1, false), actor(1, 1, false)],
        vec![actor(1, 1, false), actor(1, 2, true)],
        (1..=9).map(|id| actor(id, 1, false)).collect(),
    ] {
        let mut bad = view.clone();
        bad.actors = actors;
        assert!(wire::encode(&p, rate, &bad).is_err());
    }
    let mut bad = view;
    bad.actors[0].key.actor.epoch = WorldEpoch::new(4).unwrap();
    assert!(wire::encode(&p, rate, &bad).is_err());
}
#[test]
fn receive_death_respawn_join_leave_and_coalesced_generations_preserve_current_lifetimes() {
    let p = profile(false);
    let rate = TickRate::Hz30;
    let mut state = ClientState::new(p.clone(), key(1), rate, [7; 32]).unwrap();
    let mut view = observation(&p);
    let apply = |state: &mut ClientState, view: &Observation| {
        state.apply(&wire::encode(&p, rate, view).unwrap()).unwrap()
    };
    assert_eq!(apply(&mut state, &view), Applied::Changed);
    assert_eq!(apply(&mut state, &view), Applied::Duplicate);
    view.tick += 1;
    view.actors[1].dead = true;
    assert_eq!(apply(&mut state, &view), Applied::Changed);
    assert!(state.actor(key(2)).unwrap().dead);
    view.tick += 1;
    view.actors[1] = actor(2, 2, false);
    assert_eq!(apply(&mut state, &view), Applied::Changed);
    assert_eq!(state.actor(key(2)).unwrap().key.generation.get(), 2);
    // A stalled renderer may sample only this fresh life, but its generation
    // still invalidates the previous animation even if it missed the dead pose.
    view.tick += 1;
    view.actors[1] = actor(2, 4, false);
    assert_eq!(apply(&mut state, &view), Applied::Changed);
    view.tick += 1;
    view.actors.push(actor(3, 1, false));
    assert_eq!(apply(&mut state, &view), Applied::Changed);
    view.tick += 1;
    view.actors.remove(1);
    assert_eq!(apply(&mut state, &view), Applied::Changed);
    assert!(state.actor(key(2)).is_none());
    assert!(state.actor(key(3)).is_some());
    assert!(
        state
            .actor(EntityKey {
                epoch: WorldEpoch::new(4).unwrap(),
                ..key(1)
            })
            .is_none()
    );
}
#[test]
fn receive_refusals_are_atomic_and_old_frames_cannot_restore_a_dead_or_former_life() {
    let p = profile(false);
    let rate = TickRate::Hz30;
    let mut state = ClientState::new(p.clone(), key(1), rate, [7; 32]).unwrap();
    let mut view = observation(&p);
    let initial = wire::encode(&p, rate, &view).unwrap();
    state.apply(&initial).unwrap();
    view.tick += 1;
    view.actors[1] = actor(2, 2, true);
    state
        .apply(&wire::encode(&p, rate, &view).unwrap())
        .unwrap();
    let current = state.current().unwrap().clone();
    assert_eq!(state.apply(&initial).unwrap(), Applied::Stale);
    assert_eq!(state.current(), Some(&current));
    for (tick, actors) in [
        (view.tick, vec![actor(1, 1, false), actor(2, 2, false)]),
        (view.tick + 1, vec![actor(1, 1, false), actor(2, 1, true)]),
        (view.tick + 1, vec![actor(1, 1, false), actor(2, 2, false)]),
        (view.tick + 1, vec![actor(2, 3, false)]),
    ] {
        let bad = Observation {
            tick,
            actors,
            ..view.clone()
        };
        assert!(state.apply(&wire::encode(&p, rate, &bad).unwrap()).is_err());
        assert_eq!(state.current(), Some(&current));
    }
}
#[test]
fn profile_world_binding_is_exact_and_terminal_clock_is_valid_without_wrapping() {
    let p = profile(false);
    assert_ne!(p.hash(), profile(true).hash());
    assert_ne!(p.identity([7; 32]), profile(true).identity([7; 32]));
    assert!(ClientState::new(p.clone(), key(1), TickRate::Hz30, [0; 32]).is_err());
    let mut state = ClientState::new(p.clone(), key(1), TickRate::Hz30, [7; 32]).unwrap();
    assert_eq!(state.world(), [7; 32]);
    assert_eq!(state.profile().hash(), p.hash());
    assert_eq!(state.rate(), TickRate::Hz30);
    let mut view = observation(&p);
    view.tick = u64::MAX;
    view.actors[1] = actor(2, u64::MAX, true);
    let bytes = wire::encode(&p, TickRate::Hz30, &view).unwrap();
    assert_eq!(state.apply(&bytes).unwrap(), Applied::Changed);
    assert_eq!(state.apply(&bytes).unwrap(), Applied::Duplicate);
}
