use super::*;
use crate::{combat_wire::Batch, simulation::combat::Event};
use theia_core::{abilities, coordinates::GlobalPosition};
fn weapons() -> Catalog {
    Catalog::from_json(include_bytes!(
        "../../../../../content/gameplay/development-melee-weapons.json"
    ))
    .unwrap()
}
fn abilities() -> abilities::Catalog {
    abilities::Catalog::from_json(br#"{"schema":1,"global_cooldown_ms":1500,"queue_window_ms":300,"abilities":[{"id":"theia.test.hit","target":"hostile_alive","range_metres":30,"cast_ms":1000,"cooldown_ms":1000,"effect":{"kind":"damage","amount":20}}]}"#).unwrap()
}
fn state() -> Snapshot {
    Snapshot {
        actor: key(7, 1).unwrap(),
        stream_epoch: 9,
        tick: 100,
        target: Some(key(7, 2).unwrap()),
        phase: Phase::Paused(Pause::NotFacing),
        weapon: Some(ContentId::new("theia.dev.training-sword").unwrap()),
        ready_at: Some(172),
        swing_sequence: 3,
        reply: Some(Reply {
            sequence: 8,
            outcome: Outcome::Engaged,
        }),
    }
}
#[test]
fn intent_canonical_lengths_and_context() {
    for target in [None, Some(key(7, 2).unwrap())] {
        let request = autoattack::Request {
            epoch: 7,
            stream_epoch: 9,
            sequence: 1,
            target,
        };
        let bytes = encode_request(request).unwrap();
        assert_eq!(bytes.len(), REQUEST_BYTES);
        let decoded = decode_request(&bytes).unwrap();
        assert_eq!(encode_request(decoded).unwrap(), bytes);
        assert_eq!(decoded.target, target);
        for n in 0..bytes.len() {
            assert!(decode_request(&bytes[..n]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(decode_request(&trailing).is_err());
        for offset in [4, 12, 20] {
            let mut bad = bytes.clone();
            bad[offset..offset + 8].fill(0);
            assert!(decode_request(&bad).is_err());
        }
    }
    let foreign = autoattack::Request {
        epoch: 7,
        stream_epoch: 9,
        sequence: 1,
        target: Some(key(8, 2).unwrap()),
    };
    assert!(encode_request(foreign).is_err());
}
#[test]
fn state_historical_ack_is_independent_of_current_engagement() {
    let catalog = weapons();
    let mut view = state();
    for phase in [
        Phase::Engaged,
        Phase::Attacking,
        Phase::Paused(Pause::NotFacing),
        Phase::Paused(Pause::TimeExhausted),
    ] {
        view.phase = phase;
        let bytes = encode_state(&view, &catalog).unwrap();
        assert_eq!(bytes.len(), MAX_STATE_BYTES);
        assert_eq!(decode_state(&bytes, &catalog).unwrap(), view);
    }
    // Death/clear can stop current engagement while retaining its historical ack.
    view.target = None;
    view.phase = Phase::Idle;
    assert_eq!(
        decode_state(&encode_state(&view, &catalog).unwrap(), &catalog).unwrap(),
        view
    );
    view.weapon = None;
    view.ready_at = None;
    view.reply = None;
    assert_eq!(
        decode_state(&encode_state(&view, &catalog).unwrap(), &catalog).unwrap(),
        view
    );
}
#[test]
fn state_rejects_bad_flags_ordinals_deadlines_and_trailing_data() {
    let catalog = weapons();
    let view = state();
    let bytes = encode_state(&view, &catalog).unwrap();
    for n in 0..bytes.len() {
        assert!(decode_state(&bytes[..n], &catalog).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(decode_state(&bad, &catalog).is_err());
    for (offset, value) in [(52, 255), (53, 4), (54, 32), (71, 255)] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        assert!(decode_state(&bad, &catalog).is_err(), "offset {offset}");
    }
    let mut bad = view.clone();
    bad.ready_at = Some(view.tick + 3601);
    assert!(encode_state(&bad, &catalog).is_err());
    bad = view.clone();
    bad.target = Some(view.actor);
    assert!(encode_state(&bad, &catalog).is_err());
    bad = view.clone();
    bad.reply.as_mut().unwrap().outcome = Outcome::Ignored;
    assert!(encode_state(&bad, &catalog).is_err());
    bad = view.clone();
    bad.target = None;
    bad.phase = Phase::Attacking;
    assert!(encode_state(&bad, &catalog).is_err());
}
fn hit() -> Event {
    Event::MeleeDamage {
        weapon: state().weapon.unwrap(),
        actor: key(7, 1).unwrap(),
        sequence: 1,
        target: key(7, 2).unwrap(),
        position: GlobalPosition::new([0., 0., -2.]).unwrap(),
        amount: 15,
        remaining: 85,
    }
}
#[test]
fn explicit_mixed_events_preserve_order_and_reject_legacy_decoders() {
    let catalog = abilities();
    let weapons = weapons();
    let mut batch = Batch {
        epoch: 7,
        sequence: 1,
        tick: 100,
        events: vec![
            hit(),
            Event::Damage {
                ability: ContentId::new("theia.test.hit").unwrap(),
                actor: key(7, 1).unwrap(),
                sequence: 8,
                target: key(7, 2).unwrap(),
                position: GlobalPosition::new([0., 0., -2.]).unwrap(),
                amount: 20,
                remaining: 65,
            },
            Event::Died {
                target: key(7, 2).unwrap(),
                respawn_at: 190,
                position: GlobalPosition::new([0., 0., -2.]).unwrap(),
            },
        ],
    };
    let bytes = batch.encode_melee(&catalog, &weapons).unwrap();
    assert_eq!(
        Batch::decode_melee(&bytes, &catalog, &weapons).unwrap(),
        batch
    );
    assert!(Batch::decode(&bytes, &catalog).is_err());
    assert!(batch.encode(&catalog).is_err());
    for n in 0..bytes.len() {
        assert!(Batch::decode_melee(&bytes[..n], &catalog, &weapons).is_err());
    }
    batch.events = vec![hit(); super::super::MAX_BATCH_EVENTS];
    assert!(batch.encode_melee(&catalog, &weapons).unwrap().len() <= super::super::MAX_BYTES);
    batch.events.push(hit());
    assert!(batch.encode_melee(&catalog, &weapons).is_err());
}
#[test]
fn events_validate_weapon_damage_and_incarnation() {
    let catalog = abilities();
    let weapons = weapons();
    for damage in [0, 16, u32::MAX] {
        let mut event = hit();
        if let Event::MeleeDamage { amount, .. } = &mut event {
            *amount = damage;
        }
        assert!(
            Batch {
                epoch: 7,
                sequence: 1,
                tick: 100,
                events: vec![event]
            }
            .encode_melee(&catalog, &weapons)
            .is_err()
        );
    }
    for target in [key(8, 2).unwrap(), key(7, 1).unwrap()] {
        let mut event = hit();
        if let Event::MeleeDamage { target: t, .. } = &mut event {
            *t = target;
        }
        assert!(
            Batch {
                epoch: 7,
                sequence: 1,
                tick: 100,
                events: vec![event]
            }
            .encode_melee(&catalog, &weapons)
            .is_err()
        );
    }
}

#[test]
fn positional_reasons_require_explicit_ability_rules_on_state_and_events() {
    use crate::simulation::combat::{self, Reason};
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-classic-abilities.json"
    ))
    .unwrap();
    let mut legacy_raw = raw.clone();
    legacy_raw["schema"] = serde_json::json!(6);
    for ability in legacy_raw["abilities"].as_array_mut().unwrap() {
        ability.as_object_mut().unwrap().remove("combat");
    }
    let legacy = abilities::Catalog::from_json(&serde_json::to_vec(&legacy_raw).unwrap()).unwrap();
    raw["schema"] = serde_json::json!(7);
    for ability in raw["abilities"].as_array_mut().unwrap() {
        ability["combat"] = serde_json::json!({"minimum_range_metres":0.0,"facing":{"kind":"frontal","arc_degrees":180.0},"autoattack":{"kind":"continue"}});
    }
    let explicit = abilities::Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    for reason in [Reason::NotFacing, Reason::TooClose] {
        let view = super::super::State {
            vitals: None,
            epoch: 7,
            tick: 100,
            reply: Some(combat::Reply {
                sequence: 1,
                outcome: combat::Outcome::Rejected(reason),
            }),
            gcd_until: 0,
            cast: None,
            queued: None,
            cooldowns: vec![],
            targets: vec![],
        };
        assert!(view.encode(&legacy).is_err());
        let bytes = view.encode(&explicit).unwrap();
        assert_eq!(
            super::super::State::decode(&bytes, &explicit).unwrap(),
            view
        );
        assert!(super::super::State::decode(&bytes, &legacy).is_err());
        let batch = Batch {
            epoch: 7,
            sequence: 1,
            tick: 100,
            events: vec![Event::Cancelled {
                actor: key(7, 1).unwrap(),
                sequence: 1,
                reason,
            }],
        };
        assert!(batch.encode_melee(&legacy, &weapons()).is_err());
        let bytes = batch.encode_melee(&explicit, &weapons()).unwrap();
        assert!(Batch::decode_melee(&bytes, &legacy, &weapons()).is_err());
        assert_eq!(
            Batch::decode_melee(&bytes, &explicit, &weapons()).unwrap(),
            batch
        );
    }
}
