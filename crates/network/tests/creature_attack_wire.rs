//! Public timing contract; no transport/native adoption is inferred here.
use std::sync::Arc;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    creature_attack_wire::{self as wire, Applied, ClientState, Observation, Profile, Windup},
    player_life_wire,
};
fn profile(padded: bool) -> Arc<Profile> {
    let mut retaliation =
        include_bytes!("../../../content/gameplay/development-creature-retaliation.json").to_vec();
    if padded {
        retaliation.push(b' ');
    }
    Arc::new(Profile::new(Arc::new(
        player_life_wire::Profile::from_json(
            include_bytes!("../../../content/gameplay/development-character-stats.json"),
            include_bytes!("../../../content/gameplay/development-creature-deflect-pursuit.json"),
            &retaliation,
            include_bytes!("../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    )))
}
fn key(n: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(1).unwrap(),
        entity: EntityId::new(n).unwrap(),
    }
}
fn pending(p: &Profile, rate: TickRate) -> Observation {
    Observation {
        profile: p.hash(),
        creature: key(1),
        tick: 100,
        windup: Some(Windup {
            target: key(2),
            starts: 100,
            completes: 100 + p.windup_ticks(rate).unwrap(),
        }),
    }
}
#[test]
fn public_idle_and_windup_round_trip_with_fixed_budgets_all_rates() {
    let p = profile(false);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let value = pending(&p, rate);
        let bytes = wire::encode(&p, rate, &value).unwrap();
        assert_eq!(bytes.len(), wire::MAX_FRAME_BYTES);
        assert_eq!(bytes.len(), 86);
        assert_eq!(wire::frame_rate(&bytes).unwrap(), rate);
        assert_eq!(wire::decode(&p, key(1).epoch, rate, &bytes).unwrap(), value);
        let idle = Observation {
            windup: None,
            ..value
        };
        let bytes = wire::encode(&p, rate, &idle).unwrap();
        assert_eq!(bytes.len(), 62);
        assert_eq!(bytes.len(), wire::MIN_FRAME_BYTES);
        assert_eq!(wire::decode(&p, key(1).epoch, rate, &bytes).unwrap(), idle);
    }
}
#[test]
fn strict_frame_headers_keys_lengths_and_windows_refuse_before_state_changes() {
    let p = profile(false);
    let rate = TickRate::Hz30;
    let value = pending(&p, rate);
    let bytes = wire::encode(&p, rate, &value).unwrap();
    for length in 0..bytes.len() {
        assert!(
            wire::decode(&p, key(1).epoch, rate, &bytes[..length]).is_err(),
            "{length}"
        );
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(wire::decode(&p, key(1).epoch, rate, &bad).is_err());
    for (offset, contents) in [
        (0, vec![b'X']),
        (4, vec![25]),
        (5, vec![bytes[5] ^ 1]),
        (37, 0u64.to_le_bytes().to_vec()),
        (45, 0u64.to_le_bytes().to_vec()),
        (61, vec![2]),
        (62, 0u64.to_le_bytes().to_vec()),
        (62, 1u64.to_le_bytes().to_vec()),
        (70, 101u64.to_le_bytes().to_vec()),
        (78, 100u64.to_le_bytes().to_vec()),
        (78, 114u64.to_le_bytes().to_vec()),
    ] {
        let mut bad = bytes.clone();
        bad[offset..offset + contents.len()].copy_from_slice(&contents);
        assert!(
            wire::decode(&p, key(1).epoch, rate, &bad).is_err(),
            "offset {offset}"
        );
    }
    assert!(wire::decode(&p, WorldEpoch::new(2).unwrap(), rate, &bytes).is_err());
    assert!(wire::decode(&p, key(1).epoch, TickRate::Hz60, &bytes).is_err());
    let mut foreign = value;
    foreign.windup.as_mut().unwrap().target.epoch = WorldEpoch::new(2).unwrap();
    assert!(wire::encode(&p, rate, &foreign).is_err());
}
#[test]
fn receive_state_is_atomic_on_conflicts_and_stale_windups_cannot_restore_a_cancelled_or_new_npc() {
    let p = profile(false);
    let rate = TickRate::Hz30;
    let value = pending(&p, rate);
    let bytes = wire::encode(&p, rate, &value).unwrap();
    let mut state = ClientState::new(p.clone(), key(1).epoch, rate, [7; 32]).unwrap();
    assert_eq!(state.world(), [7; 32]);
    assert_eq!(state.apply(&bytes).unwrap(), Applied::Changed);
    assert_eq!(state.apply(&bytes).unwrap(), Applied::Duplicate);
    let conflict = Observation {
        windup: None,
        ..value
    };
    assert!(
        state
            .apply(&wire::encode(&p, rate, &conflict).unwrap())
            .is_err()
    );
    assert_eq!(state.current(), Some(&value));
    let cancel = Observation {
        tick: 101,
        ..conflict
    };
    assert_eq!(
        state
            .apply(&wire::encode(&p, rate, &cancel).unwrap())
            .unwrap(),
        Applied::Changed
    );
    assert_eq!(state.apply(&bytes).unwrap(), Applied::Stale);
    assert!(state.active_at(102).is_none());
    let fresh = Observation {
        creature: key(8),
        tick: 102,
        ..cancel
    };
    assert_eq!(
        state
            .apply(&wire::encode(&p, rate, &fresh).unwrap())
            .unwrap(),
        Applied::Changed
    );
    assert_eq!(state.apply(&bytes).unwrap(), Applied::Stale);
    assert_eq!(state.current(), Some(&fresh));
}
#[test]
fn expired_late_arriving_windups_are_not_active_after_the_contact_tick_all_rates() {
    let p = profile(false);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let value = pending(&p, rate);
        let w = value.windup.unwrap();
        let mut state = ClientState::new(p.clone(), key(1).epoch, rate, [7; 32]).unwrap();
        state
            .apply(&wire::encode(&p, rate, &value).unwrap())
            .unwrap();
        assert!(state.active_at(99).is_none());
        assert_eq!(state.active_at(100), Some(&w));
        assert_eq!(state.active_at(w.completes - 1), Some(&w));
        assert!(state.active_at(w.completes).is_none());
        assert!(state.active_at(u64::MAX).is_none());
    }
}
#[test]
fn exact_life_profile_and_explicit_public_identity_bind_the_contract() {
    let p = profile(false);
    let other = profile(true);
    assert_ne!(p.hash(), other.hash());
    assert_ne!(p.identity([7; 32]), [7; 32]);
    assert_ne!(p.identity([7; 32]), other.identity([7; 32]));
    assert_eq!(p.hash(), Profile::new(Arc::new(p.life().clone())).hash());
    let bytes = wire::encode(&p, TickRate::Hz30, &pending(&p, TickRate::Hz30)).unwrap();
    assert!(wire::decode(&other, key(1).epoch, TickRate::Hz30, &bytes).is_err());
    assert!(ClientState::new(p, key(1).epoch, TickRate::Hz30, [0; 32]).is_err());
}
#[test]
fn terminal_tick_window_is_checked_without_wrapping() {
    let p = profile(false);
    let rate = TickRate::Hz60;
    let duration = p.windup_ticks(rate).unwrap();
    let value = Observation {
        tick: u64::MAX - 1,
        windup: Some(Windup {
            target: key(2),
            starts: u64::MAX - duration,
            completes: u64::MAX,
        }),
        ..pending(&p, rate)
    };
    assert_eq!(
        wire::decode(
            &p,
            key(1).epoch,
            rate,
            &wire::encode(&p, rate, &value).unwrap()
        )
        .unwrap(),
        value
    );
    let mut bad = value;
    bad.windup.as_mut().unwrap().completes = 0;
    assert!(wire::encode(&p, rate, &bad).is_err());
}
