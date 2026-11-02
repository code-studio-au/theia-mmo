//! Codec/owner receive contracts; actual new-mode UDP is qualified separately.
use std::{num::NonZeroU64, sync::Arc};
use theia_core::{
    combat_membership::Summary,
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    time::TickRate,
};
use theia_network::{
    combat_membership_wire as wire,
    player_life_wire::{self as life, Applied, ClientState},
    simulation::combat::player_life::{Observation, Windup},
};
const POLICY: &[u8] =
    include_bytes!("../../../content/gameplay/development-combat-membership.json");
fn profiles() -> (Arc<life::Profile>, Arc<wire::Profile>) {
    (
        Arc::new(
            life::Profile::from_json(
                include_bytes!("../../../content/gameplay/development-character-stats.json"),
                include_bytes!("../../../content/gameplay/development-creature-patrol.json"),
                include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
                include_bytes!("../../../content/gameplay/development-player-respawn.json"),
            )
            .unwrap(),
        ),
        Arc::new(wire::Profile::from_json(POLICY).unwrap()),
    )
}
fn actor(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(1).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn value(profile: &life::Profile) -> Observation {
    Observation {
        profile: profile.hash(),
        key: LifeKey {
            actor: actor(2),
            generation: NonZeroU64::MIN,
        },
        tick: 7,
        movement_stream: 1,
        health: 90,
        maximum_health: 100,
        respawn_at: None,
        windup: None,
        combat: Some(Summary::default()),
    }
}
fn bytes(p: &life::Profile, m: &wire::Profile, v: Observation) -> Vec<u8> {
    wire::encode(p, m, TickRate::Hz30, &v).unwrap()
}
#[test]
fn all_reason_combinations_and_bounded_live_dead_windup_frames_round_trip_at_every_rate() {
    let (p, m) = profiles();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for flags in 0..8 {
            let mut v = value(&p);
            v.combat = Some(Summary {
                opponents: if flags == 0 { 0 } else { 32 },
                outgoing_hostility: flags & 1 != 0,
                incoming_hostility: flags & 2 != 0,
                active_encounter: flags & 4 != 0,
            });
            let frame = wire::encode(&p, &m, rate, &v).unwrap();
            assert_eq!(frame.len(), 120);
            assert_eq!(wire::frame_rate(&frame).unwrap(), rate);
            assert_eq!(wire::decode(&p, &m, actor(2), rate, &frame).unwrap(), v);
        }
        let mut v = value(&p);
        v.windup = Some(Windup {
            creature: actor(3),
            target: v.key,
            completes: v.tick + u64::from(rate.hz()) / 2,
        });
        let frame = wire::encode(&p, &m, rate, &v).unwrap();
        assert_eq!(frame.len(), wire::MAX_FRAME_BYTES);
        assert_eq!(wire::decode(&p, &m, actor(2), rate, &frame).unwrap(), v);
        v.windup = None;
        v.health = 0;
        v.movement_stream = 2;
        v.respawn_at = Some(v.tick + u64::from(rate.hz()) * 3);
        let frame = wire::encode(&p, &m, rate, &v).unwrap();
        assert_eq!(frame.len(), 128);
        assert_eq!(wire::decode(&p, &m, actor(2), rate, &frame).unwrap(), v);
    }
}
#[test]
fn malformed_truncated_trailing_foreign_and_wrong_profile_frames_are_refused() {
    let (p, m) = profiles();
    let frame = bytes(&p, &m, value(&p));
    for len in 0..frame.len() {
        assert!(wire::decode(&p, &m, actor(2), TickRate::Hz30, &frame[..len]).is_err());
    }
    let mut trailing = frame.clone();
    trailing.push(0);
    assert!(wire::decode(&p, &m, actor(2), TickRate::Hz30, &trailing).is_err());
    for (index, byte) in [
        (0, b'X'),
        (4, 31),
        (5, 255),
        (86, 255),
        (118, 33),
        (119, 128),
        (119, 1),
        (118, 1),
    ] {
        let mut bad = frame.clone();
        bad[index] = byte;
        assert!(
            wire::decode(&p, &m, actor(2), TickRate::Hz30, &bad).is_err(),
            "byte {index}"
        );
    }
    assert!(wire::decode(&p, &m, actor(3), TickRate::Hz30, &frame).is_err());
    assert!(wire::decode(&p, &m, actor(2), TickRate::Hz60, &frame).is_err());
    let mut changed = POLICY.to_vec();
    changed.push(b' ');
    let other = wire::Profile::from_json(&changed).unwrap();
    assert_ne!(m.hash(), other.hash());
    assert!(wire::decode(&p, &other, actor(2), TickRate::Hz30, &frame).is_err());
}
#[test]
fn invalid_summary_dead_combat_and_implicit_downgrades_are_refused() {
    let (p, m) = profiles();
    let v = value(&p);
    for summary in [
        Summary {
            opponents: 1,
            ..Default::default()
        },
        Summary {
            outgoing_hostility: true,
            ..Default::default()
        },
        Summary {
            opponents: 33,
            incoming_hostility: true,
            ..Default::default()
        },
    ] {
        assert!(
            wire::encode(
                &p,
                &m,
                TickRate::Hz30,
                &Observation {
                    combat: Some(summary),
                    ..v
                }
            )
            .is_err()
        );
    }
    let dead = Observation {
        health: 0,
        movement_stream: 2,
        respawn_at: Some(97),
        combat: Some(Summary {
            opponents: 1,
            active_encounter: true,
            ..Default::default()
        }),
        ..v
    };
    assert!(wire::encode(&p, &m, TickRate::Hz30, &dead).is_err());
    assert!(life::encode(&p, TickRate::Hz30, &v).is_err());
    assert!(wire::encode(&p, &m, TickRate::Hz30, &Observation { combat: None, ..v }).is_err());
    let legacy = life::encode(&p, TickRate::Hz30, &Observation { combat: None, ..v }).unwrap();
    let frame = bytes(&p, &m, v);
    assert!(life::is_any_state(&frame));
    assert!(!life::is_state(&frame));
    assert!(life::decode(&p, actor(2), TickRate::Hz30, &frame).is_err());
    assert!(wire::decode(&p, &m, actor(2), TickRate::Hz30, &legacy).is_err());
    let mut old = ClientState::new(p.clone(), actor(2), TickRate::Hz30, [9; 32]).unwrap();
    assert!(old.apply(&frame).is_err());
    assert!(old.current().is_none());
    let mut new =
        ClientState::with_membership(p.clone(), m.clone(), actor(2), TickRate::Hz30, [9; 32])
            .unwrap();
    assert!(new.apply(&legacy).is_err());
    assert!(new.current().is_none());
}
#[test]
fn receive_duplicate_conflict_stale_and_respawn_cannot_restore_old_combat() {
    let (p, m) = profiles();
    let mut v = value(&p);
    v.combat = Some(Summary {
        opponents: 1,
        outgoing_hostility: true,
        ..Default::default()
    });
    let mut state =
        ClientState::with_membership(p.clone(), m.clone(), actor(2), TickRate::Hz30, [9; 32])
            .unwrap();
    let original = bytes(&p, &m, v);
    assert_eq!(state.apply(&original).unwrap(), Applied::Changed);
    assert_eq!(state.apply(&original).unwrap(), Applied::Duplicate);
    let conflict = bytes(
        &p,
        &m,
        Observation {
            combat: Some(Summary::default()),
            ..v
        },
    );
    assert!(state.apply(&conflict).is_err());
    assert_eq!(state.current(), Some(&v));
    let dead = Observation {
        tick: 8,
        health: 0,
        movement_stream: 2,
        respawn_at: Some(98),
        combat: Some(Summary::default()),
        ..v
    };
    state.apply(&bytes(&p, &m, dead)).unwrap();
    let fresh = Observation {
        tick: 98,
        key: LifeKey {
            generation: NonZeroU64::new(2).unwrap(),
            ..v.key
        },
        health: 100,
        movement_stream: 3,
        combat: Some(Summary::default()),
        ..v
    };
    state.apply(&bytes(&p, &m, fresh)).unwrap();
    assert_eq!(state.apply(&original).unwrap(), Applied::Stale);
    assert_eq!(state.current(), Some(&fresh));
    let same_life_heal = Observation {
        tick: 9,
        health: 95,
        ..v
    };
    let mut state =
        ClientState::with_membership(p.clone(), m.clone(), actor(2), TickRate::Hz30, [9; 32])
            .unwrap();
    state.apply(&original).unwrap();
    assert!(state.apply(&bytes(&p, &m, same_life_heal)).is_err());
    assert_eq!(state.current(), Some(&v));
}
#[test]
fn membership_only_update_does_not_move_the_prediction_correction_barrier() {
    let (p, m) = profiles();
    let v = value(&p);
    let mut state =
        ClientState::with_membership(p.clone(), m.clone(), actor(2), TickRate::Hz30, [9; 32])
            .unwrap();
    state.apply(&bytes(&p, &m, v)).unwrap();
    let correction = theia_network::prediction::Correction {
        world: [9; 32],
        actor: actor(2),
        rate: TickRate::Hz30,
        input: theia_network::commands::Timeline::new(1, TickRate::Hz30, 7)
            .unwrap()
            .state(),
        pose: theia_network::prediction::Pose {
            facing: None,
            feet: theia_core::coordinates::GlobalPosition::new([8000., 0., 8000.]).unwrap(),
            motion: theia_network::kinematic::Motion::default().snapshot(),
            dodge: None,
        },
        ready_sequence: 1,
    };
    assert!(state.movement_allowed(&correction));
    state
        .apply(&bytes(
            &p,
            &m,
            Observation {
                tick: 8,
                combat: Some(Summary {
                    opponents: 1,
                    active_encounter: true,
                    ..Default::default()
                }),
                ..v
            },
        ))
        .unwrap();
    assert!(state.movement_allowed(&correction));
}
#[test]
fn profile_bounds_are_checked_before_hashing_and_preserve_exact_policy_identity() {
    assert!(wire::Profile::from_json(&vec![b' '; 4097]).is_err());
    assert!(wire::Profile::from_json(b"{}").is_err());
    let (_, m) = profiles();
    assert_eq!(m.policy().maximum_opponents(), 32);
    let mut raw: serde_json::Value = serde_json::from_slice(POLICY).unwrap();
    raw["maximum_opponents"] = 1.into();
    let smaller = wire::Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let (p, _) = profiles();
    let v = Observation {
        combat: Some(Summary {
            opponents: 2,
            outgoing_hostility: true,
            ..Default::default()
        }),
        ..value(&p)
    };
    assert!(wire::encode(&p, &smaller, TickRate::Hz30, &v).is_err());
}
