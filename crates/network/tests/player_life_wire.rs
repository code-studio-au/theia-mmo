//! Private codec/receive-barrier contracts only; no new-mode transport claim.
use std::{num::NonZeroU64, sync::Arc};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    time::TickRate,
};
use theia_network::{
    player_life_wire::{self as wire, Applied, ClientState, Profile},
    prediction::{Correction, Pose},
    simulation::combat::player_life::{Observation, Windup},
};
const STATS: &[u8] = include_bytes!("../../../content/gameplay/development-character-stats.json");
const CREATURE: &[u8] =
    include_bytes!("../../../content/gameplay/development-creature-patrol.json");
const POLICY: &[u8] =
    include_bytes!("../../../content/gameplay/development-creature-retaliation.json");
const RESPAWN: &[u8] = include_bytes!("../../../content/gameplay/development-player-respawn.json");
fn profile() -> Arc<Profile> {
    Arc::new(Profile::from_json(STATS, CREATURE, POLICY, RESPAWN).unwrap())
}
fn actor(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(1).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn value(p: &Profile) -> Observation {
    Observation {
        combat: None,
        profile: p.hash(),
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
    }
}
fn correction(stream: u64, tick: u64) -> Correction {
    Correction {
        world: [9; 32],
        actor: actor(2),
        rate: TickRate::Hz30,
        input: theia_network::commands::Timeline::new(stream, TickRate::Hz30, tick)
            .unwrap()
            .state(),
        pose: Pose {
            facing: None,
            feet: theia_core::coordinates::GlobalPosition::new([8000., 0., 8000.]).unwrap(),
            motion: theia_network::kinematic::Motion::default().snapshot(),
            dodge: None,
        },
        ready_sequence: 1,
    }
}
#[test]
fn fixed_bounded_frames_round_trip_exact_own_life_health_deadline_and_windup() {
    let p = profile();
    let base = value(&p);
    let bytes = wire::encode(&p, TickRate::Hz30, &base).unwrap();
    assert_eq!(bytes.len(), 86);
    assert_eq!(bytes[4], 30);
    assert_eq!(&bytes[..4], b"PL01");
    assert_eq!(&bytes[77..86], &[90, 0, 0, 0, 100, 0, 0, 0, 0]);
    assert_eq!(
        wire::decode(&p, actor(2), TickRate::Hz30, &bytes).unwrap(),
        base
    );
    let winding = Observation {
        windup: Some(Windup {
            creature: actor(1),
            target: base.key,
            completes: 22,
        }),
        ..base
    };
    let bytes = wire::encode(&p, TickRate::Hz30, &winding).unwrap();
    assert_eq!(bytes.len(), wire::MAX_FRAME_BYTES);
    assert_eq!(
        wire::decode(&p, actor(2), TickRate::Hz30, &bytes).unwrap(),
        winding
    );
    let dead = Observation {
        tick: 8,
        movement_stream: 2,
        health: 0,
        respawn_at: Some(98),
        ..base
    };
    let bytes = wire::encode(&p, TickRate::Hz30, &dead).unwrap();
    assert_eq!(bytes.len(), 94);
    assert_eq!(
        wire::decode(&p, actor(2), TickRate::Hz30, &bytes).unwrap(),
        dead
    );
    let waiting = Observation {
        tick: 100,
        respawn_at: Some(98),
        ..dead
    };
    assert!(
        wire::encode(&p, TickRate::Hz30, &waiting).is_ok(),
        "expired deadlines can be awaiting collision"
    );
}
#[test]
fn foreign_owner_profile_tags_flags_zero_identities_truncation_and_trailing_bytes_are_refused() {
    let p = profile();
    let v = value(&p);
    let base = wire::encode(&p, TickRate::Hz30, &v).unwrap();
    assert!(wire::decode(&p, actor(3), TickRate::Hz30, &base).is_err());
    let mut changed = POLICY.to_vec();
    changed.push(b' ');
    let other = Profile::from_json(STATS, CREATURE, &changed, RESPAWN).unwrap();
    assert!(wire::decode(&other, actor(2), TickRate::Hz30, &base).is_err());
    for index in [0, 4, 5, 85] {
        let mut bytes = base.clone();
        bytes[index] ^= 128;
        assert!(wire::decode(&p, actor(2), TickRate::Hz30, &bytes).is_err());
    }
    for range in [37..45, 45..53, 53..61, 69..77] {
        let mut bytes = base.clone();
        bytes[range].fill(0);
        assert!(wire::decode(&p, actor(2), TickRate::Hz30, &bytes).is_err());
    }
    for length in 0..base.len() {
        assert!(wire::decode(&p, actor(2), TickRate::Hz30, &base[..length]).is_err());
    }
    let mut extra = base.clone();
    extra.push(0);
    assert!(wire::decode(&p, actor(2), TickRate::Hz30, &extra).is_err());
    assert!(
        wire::decode(
            &p,
            actor(2),
            TickRate::Hz30,
            &[0; wire::MAX_FRAME_BYTES + 1]
        )
        .is_err()
    );
}
#[test]
fn unsupported_health_phase_identity_and_deadline_combinations_never_encode() {
    let p = profile();
    let v = value(&p);
    let dead = Observation {
        movement_stream: 2,
        health: 0,
        respawn_at: Some(97),
        ..v
    };
    let winding = Windup {
        creature: actor(1),
        target: v.key,
        completes: 22,
    };
    for bad in [
        Observation { health: 101, ..v },
        Observation {
            maximum_health: 99,
            ..v
        },
        Observation {
            movement_stream: 0,
            ..v
        },
        Observation { health: 0, ..v },
        Observation {
            respawn_at: Some(97),
            ..v
        },
        Observation {
            respawn_at: Some(0),
            ..dead
        },
        Observation {
            movement_stream: 1,
            ..dead
        },
        Observation {
            respawn_at: Some(98),
            ..dead
        },
        Observation {
            windup: Some(winding),
            ..dead
        },
        Observation {
            windup: Some(Windup {
                creature: actor(2),
                ..winding
            }),
            ..v
        },
        Observation {
            windup: Some(Windup {
                target: LifeKey {
                    actor: actor(3),
                    ..v.key
                },
                ..winding
            }),
            ..v
        },
        Observation {
            windup: Some(Windup {
                completes: 7,
                ..winding
            }),
            ..v
        },
        Observation {
            windup: Some(Windup {
                completes: 23,
                ..winding
            }),
            ..v
        },
    ] {
        assert!(wire::encode(&p, TickRate::Hz30, &bad).is_err(), "{bad:?}");
    }
}
#[test]
fn configured_tick_rate_bounds_deadline_lengths_without_trusting_unbounded_future_ticks() {
    let p = profile();
    let v = value(&p);
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let until = v.tick + 3 * u64::from(rate.hz());
        let dead = Observation {
            movement_stream: 2,
            health: 0,
            respawn_at: Some(until),
            ..v
        };
        let bytes = wire::encode(&p, rate, &dead).unwrap();
        assert_eq!(wire::frame_rate(&bytes), Ok(rate));
        if rate != TickRate::Hz30 {
            assert!(wire::decode(&p, actor(2), TickRate::Hz30, &bytes).is_err());
        }
        assert!(wire::encode(&p, rate, &dead).is_ok());
        assert!(
            wire::encode(
                &p,
                rate,
                &Observation {
                    respawn_at: Some(until + 1),
                    ..dead
                }
            )
            .is_err()
        );
        let at = v.tick + u64::from(rate.hz()) / 2;
        let windup = Windup {
            creature: actor(1),
            target: v.key,
            completes: at,
        };
        assert!(
            wire::encode(
                &p,
                rate,
                &Observation {
                    windup: Some(windup),
                    ..v
                }
            )
            .is_ok()
        );
        assert!(
            wire::encode(
                &p,
                rate,
                &Observation {
                    windup: Some(Windup {
                        completes: at + 1,
                        ..windup
                    }),
                    ..v
                }
            )
            .is_err()
        );
    }
}
#[test]
fn death_and_respawn_gate_prediction_until_the_matching_completed_movement_stream() {
    let p = profile();
    let mut client = ClientState::new(p.clone(), actor(2), TickRate::Hz30, [9; 32]).unwrap();
    let alive = value(&p);
    assert!(!client.movement_allowed(&correction(1, 7)));
    client
        .apply(&wire::encode(&p, TickRate::Hz30, &alive).unwrap())
        .unwrap();
    assert!(client.movement_allowed(&correction(1, 7)));
    let dead = Observation {
        health: 0,
        movement_stream: 2,
        tick: 8,
        respawn_at: Some(98),
        ..alive
    };
    client
        .apply(&wire::encode(&p, TickRate::Hz30, &dead).unwrap())
        .unwrap();
    assert!(!client.movement_allowed(&correction(2, 98)));
    let fresh = Observation {
        key: LifeKey {
            generation: NonZeroU64::new(2).unwrap(),
            ..alive.key
        },
        tick: 98,
        movement_stream: 3,
        health: 100,
        ..alive
    };
    client
        .apply(&wire::encode(&p, TickRate::Hz30, &fresh).unwrap())
        .unwrap();
    assert!(!client.movement_allowed(&correction(2, 100)));
    assert!(!client.movement_allowed(&correction(3, 97)));
    assert!(client.movement_allowed(&correction(3, 98)));
    let health = Observation {
        health: 95,
        tick: 99,
        ..fresh
    };
    client
        .apply(&wire::encode(&p, TickRate::Hz30, &health).unwrap())
        .unwrap();
    assert!(
        client.movement_allowed(&correction(3, 98)),
        "ordinary health updates must not reestablish the respawn barrier"
    );
    let reset = Observation {
        tick: 120,
        movement_stream: 4,
        ..health
    };
    client
        .apply(&wire::encode(&p, TickRate::Hz30, &reset).unwrap())
        .unwrap();
    assert!(!client.movement_allowed(&correction(3, 121)));
    assert!(!client.movement_allowed(&correction(4, 119)));
    assert!(client.movement_allowed(&correction(4, 120)));
}
#[test]
fn stale_duplicate_conflicting_and_impossible_transitions_leave_the_receive_state_coherent() {
    let p = profile();
    let mut client = ClientState::new(p.clone(), actor(2), TickRate::Hz30, [9; 32]).unwrap();
    let v = Observation {
        movement_stream: 5,
        ..value(&p)
    };
    let bytes = wire::encode(&p, TickRate::Hz30, &v).unwrap();
    assert_eq!(client.apply(&bytes), Ok(Applied::Changed));
    assert_eq!(client.apply(&bytes), Ok(Applied::Duplicate));
    let stale = Observation { tick: 6, ..v };
    assert_eq!(
        client.apply(&wire::encode(&p, TickRate::Hz30, &stale).unwrap()),
        Ok(Applied::Stale)
    );
    for bad in [
        Observation { health: 80, ..v },
        Observation {
            tick: 8,
            health: 95,
            ..v
        },
        Observation {
            tick: 8,
            health: 0,
            respawn_at: Some(98),
            ..v
        },
        Observation {
            tick: 8,
            key: LifeKey {
                generation: NonZeroU64::new(2).unwrap(),
                ..v.key
            },
            ..v
        },
    ] {
        assert!(
            client
                .apply(&wire::encode(&p, TickRate::Hz30, &bad).unwrap())
                .is_err()
        );
        assert_eq!(client.current(), Some(&v));
        assert!(client.movement_allowed(&correction(5, 7)));
    }
}
#[test]
fn foreign_world_actor_and_rate_corrections_cannot_unlock_the_movement_barrier() {
    let p = profile();
    assert!(ClientState::new(p.clone(), actor(2), TickRate::Hz30, [0; 32]).is_err());
    let mut client = ClientState::new(p.clone(), actor(2), TickRate::Hz30, [9; 32]).unwrap();
    client
        .apply(&wire::encode(&p, TickRate::Hz30, &value(&p)).unwrap())
        .unwrap();
    let base = correction(1, 7);
    for bad in [
        Correction {
            world: [8; 32],
            ..base
        },
        Correction {
            actor: actor(3),
            ..base
        },
        Correction {
            rate: TickRate::Hz20,
            ..base
        },
    ] {
        assert!(!client.movement_allowed(&bad));
    }
}

#[test]
fn exact_default_profile_binding_matches_an_independently_calculated_reference() {
    let p = profile();
    let hex = p.hash().map(|v| format!("{v:02x}")).concat();
    assert_eq!(
        hex,
        "099a307ba8f90d8e75677ba21d2fe6a6c6c5597beb2f04fcd87e11df72acace5"
    );
}
