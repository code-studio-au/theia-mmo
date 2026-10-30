use theia_core::coordinates::GlobalPosition;
use theia_core::{
    abilities::Catalog,
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
};
use theia_network::{
    combat_wire::{self as wire, Batch, Health, State},
    simulation::combat::{self, Cast, Event, Outcome, Queued, Reason, Reply},
};
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn catalog(count: usize) -> Catalog {
    let abilities:Vec<_>=(0..count).map(|i| serde_json::json!({"id":format!("theia.test.ability-{i:02}"),"target":"hostile_alive","range_metres":30.,"cast_ms":1000,"cooldown_ms":60000,"effect":{"kind":"damage","amount":20}})).collect();
    Catalog::from_json(&serde_json::to_vec(&serde_json::json!({"schema":1,"global_cooldown_ms":1500,"queue_window_ms":300,"abilities":abilities})).unwrap()).unwrap()
}
fn cast(catalog: &Catalog) -> Cast {
    Cast {
        sequence: 1,
        ability: catalog.abilities()[0].id().clone(),
        target: key(1),
        started: 10,
        completes: 40,
    }
}
fn queued(catalog: &Catalog) -> Queued {
    Queued {
        request: combat::Request {
            epoch: 7,
            sequence: 2,
            ability: catalog.abilities()[0].id().clone(),
            target: key(1),
        },
        at: 40,
    }
}
fn largest(catalog: &Catalog) -> State {
    State {
        vitals: None,
        epoch: 7,
        tick: 10,
        reply: Some(Reply {
            sequence: 3,
            outcome: Outcome::Rejected(Reason::Unavailable),
        }),
        gcd_until: 55,
        cast: Some(cast(catalog)),
        queued: Some(queued(catalog)),
        cooldowns: catalog
            .abilities()
            .iter()
            .map(|a| (a.id().clone(), 3610))
            .collect(),
        targets: (1..=16)
            .map(|id| Health {
                entity: key(id).entity,
                current: 100,
                maximum: 100,
                respawn_at: None,
            })
            .collect(),
    }
}
#[test]
fn largest_legal_state_and_event_batch_fit_existing_datagram_budget_and_roundtrip() {
    let catalog = catalog(64);
    let state = largest(&catalog);
    let bytes = state.encode(&catalog).unwrap();
    assert_eq!(bytes.len(), 1066);
    assert!(bytes.len() <= wire::MAX_BYTES);
    assert_eq!(State::decode(&bytes, &catalog).unwrap(), state);
    let batch = Batch {
        epoch: 7,
        sequence: 1,
        tick: 10,
        events: vec![
            Event::Started {
                actor: key(100),
                cast: cast(&catalog),
                position: GlobalPosition::new([8064., 1., 8000.]).unwrap()
            };
            16
        ],
    };
    let bytes = batch.encode(&catalog).unwrap();
    assert_eq!(bytes.len(), 1085);
    assert!(bytes.len() <= wire::MAX_BYTES);
    assert_eq!(Batch::decode(&bytes, &catalog).unwrap(), batch);
    let request =
        wire::Request::new(7, 1, catalog.abilities()[0].id(), key(1).entity, &catalog).unwrap();
    let bytes = request.encode().unwrap();
    assert_eq!(bytes.len(), wire::REQUEST_BYTES);
    assert_eq!(wire::Request::decode(&bytes, &catalog).unwrap(), request);
    assert_eq!(
        request.command(&catalog).unwrap().ability,
        *catalog.abilities()[0].id()
    );
}
#[test]
fn every_truncation_trailing_byte_unknown_flag_and_ordinal_fails_without_panicking() {
    let catalog = catalog(64);
    let state = largest(&catalog);
    let bytes = state.encode(&catalog).unwrap();
    for length in 0..bytes.len() {
        assert!(
            State::decode(&bytes[..length], &catalog).is_err(),
            "length={length}"
        );
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(State::decode(&bad, &catalog).is_err());
    let mut bad = bytes.clone();
    bad[20] = 8;
    assert!(State::decode(&bad, &catalog).is_err());
    let mut bad = bytes.clone();
    bad[0] = b'X';
    assert!(State::decode(&bad, &catalog).is_err());
    let request = wire::Request::new(7, 1, catalog.abilities()[0].id(), key(1).entity, &catalog)
        .unwrap()
        .encode()
        .unwrap();
    for length in 0..request.len() {
        assert!(wire::Request::decode(&request[..length], &catalog).is_err());
    }
    for ordinal in [64, 255] {
        let mut bad = request.clone();
        bad[20] = ordinal;
        assert!(wire::Request::decode(&bad, &catalog).is_err());
    }
    let mut bad = request.clone();
    bad[12..20].copy_from_slice(&0u64.to_le_bytes());
    assert!(wire::Request::decode(&bad, &catalog).is_err());
    let mut bad = request.clone();
    bad[4..12].copy_from_slice(&0u64.to_le_bytes());
    assert!(wire::Request::decode(&bad, &catalog).is_err());
    let mut bad = request.clone();
    bad[21..29].copy_from_slice(&0u64.to_le_bytes());
    assert!(wire::Request::decode(&bad, &catalog).is_err());
}
#[test]
fn health_order_life_deadlines_cast_identity_and_cooldown_contracts_fail_closed() {
    let catalog = catalog(64);
    let good = largest(&catalog);
    let mut bad = good.clone();
    bad.targets.swap(0, 1);
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.targets[1] = bad.targets[0].clone();
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.targets[0].current = 101;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.targets[0].current = 0;
    assert!(bad.encode(&catalog).is_err());
    let mut dead = good.clone();
    dead.targets[0].current = 0;
    dead.targets[0].respawn_at = Some(40);
    assert_eq!(
        State::decode(&dead.encode(&catalog).unwrap(), &catalog).unwrap(),
        dead
    );
    let mut bad = dead.clone();
    bad.targets[0].respawn_at = Some(10);
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.cooldowns.swap(0, 1);
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.cooldowns[0].1 = 3611;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.cooldowns[0].1 = 10;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.cast.as_mut().unwrap().target.epoch = WorldEpoch::new(8).unwrap();
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.cast.as_mut().unwrap().completes = 10;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.cast.as_mut().unwrap().ability = ContentId::new("theia.unknown").unwrap();
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.reply = None;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.queued.as_mut().unwrap().at = 3611;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = good.clone();
    bad.targets.push(bad.targets[0].clone());
    assert!(bad.encode(&catalog).is_err());
    let mut later = good.clone();
    later.tick = 11;
    assert!(
        later.same_view(&good),
        "Tick alone must not repeat static state"
    );
}
#[test]
fn all_event_kinds_preserve_identity_and_strict_payloads() {
    let catalog = catalog(1);
    let batch = Batch {
        epoch: 7,
        sequence: 1,
        tick: 10,
        events: vec![
            Event::Started {
                actor: key(100),
                cast: cast(&catalog),
                position: GlobalPosition::new([8064., 1., 8000.]).unwrap(),
            },
            Event::Queued {
                actor: key(100),
                queued: queued(&catalog),
            },
            Event::Cancelled {
                actor: key(100),
                sequence: 1,
                reason: Reason::Moving,
            },
            Event::Damage {
                ability: catalog.abilities()[0].id().clone(),
                actor: key(100),
                sequence: 1,
                target: key(1),
                amount: 20,
                remaining: 0,
                position: GlobalPosition::new([8064., 1., 8000.]).unwrap(),
            },
            Event::Died {
                target: key(1),
                respawn_at: 40,
                position: GlobalPosition::new([8064., 1., 8000.]).unwrap(),
            },
            Event::Respawned {
                previous: key(1),
                target: key(200),
                position: GlobalPosition::new([8064., 1., 8000.]).unwrap(),
            },
        ],
    };
    let bytes = batch.encode(&catalog).unwrap();
    assert_eq!(Batch::decode(&bytes, &catalog).unwrap(), batch);
    for length in 0..bytes.len() {
        assert!(Batch::decode(&bytes[..length], &catalog).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(Batch::decode(&bad, &catalog).is_err());
    let mut bad = bytes.clone();
    bad[28] = 17;
    assert!(Batch::decode(&bad, &catalog).is_err());
    let mut bad = bytes.clone();
    bad[29] = 255;
    assert!(Batch::decode(&bad, &catalog).is_err());
    let mut bad = batch.clone();
    bad.events.clear();
    assert!(bad.encode(&catalog).is_err());
    let mut bad = batch.clone();
    bad.sequence = 0;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = batch.clone();
    bad.events[5] = Event::Respawned {
        previous: key(2),
        target: key(1),
        position: GlobalPosition::new([8064., 1., 8000.]).unwrap(),
    };
    assert!(bad.encode(&catalog).is_err());
    let mut bad = batch.clone();
    bad.events[3] = Event::Damage {
        ability: catalog.abilities()[0].id().clone(),
        actor: key(100),
        sequence: 1,
        target: key(1),
        amount: 0,
        remaining: 0,
        position: GlobalPosition::new([8064., 1., 8000.]).unwrap(),
    };
    assert!(bad.encode(&catalog).is_err());
    let mut bad = batch.clone();
    bad.events[4] = Event::Died {
        target: key(1),
        respawn_at: 10,
        position: GlobalPosition::new([8064., 1., 8000.]).unwrap(),
    };
    assert!(bad.encode(&catalog).is_err());
}

#[test]
fn spatial_cues_keep_checked_global_positions_without_a_live_target_or_render_frame() {
    let catalog = catalog(1);
    let position = GlobalPosition::new([1e9 + 8064., 23.125, -1e9 + 8000.]).unwrap();
    let batch = Batch {
        epoch: 7,
        sequence: 1,
        tick: 10,
        events: vec![
            Event::Damage {
                ability: catalog.abilities()[0].id().clone(),
                actor: key(100),
                sequence: 1,
                target: key(1),
                amount: 20,
                remaining: 0,
                position,
            },
            Event::Died {
                target: key(1),
                respawn_at: 40,
                position,
            },
        ],
    };
    let bytes = batch.encode(&catalog).unwrap();
    assert_eq!(&bytes[..4], b"CE02");
    assert_eq!(Batch::decode(&bytes, &catalog).unwrap(), batch);
    let mut old = bytes.clone();
    old[..4].copy_from_slice(b"CE01");
    assert!(Batch::decode(&old, &catalog).is_err());
    // First damage payload: 29-byte batch header, then 34-byte event fields.
    for bad_value in [f64::NAN, f64::INFINITY, 1e30] {
        let mut bad = bytes.clone();
        bad[63..71].copy_from_slice(&bad_value.to_le_bytes());
        assert!(Batch::decode(&bad, &catalog).is_err());
    }
    for origin in [
        [1e9 + 8000., 0., -1e9 + 8000.],
        [1e9 + 8192., 0., -1e9 + 7936.],
    ] {
        let frame = theia_core::coordinates::LocalFrame::new(GlobalPosition::new(origin).unwrap());
        assert_eq!(
            frame.to_global(frame.to_local(position).unwrap()),
            Some(position)
        );
    }
}

#[test]
fn resource_state_has_an_explicit_tag_and_checked_vitals_without_changing_legacy_frames() {
    use theia_core::resources::Vitals;
    let catalog = catalog(64);
    let legacy = largest(&catalog);
    let legacy_bytes = legacy.encode(&catalog).unwrap();
    assert!(legacy_bytes.starts_with(wire::STATE_TAG));
    assert_eq!(legacy_bytes.len(), 1066);
    let mut resource = legacy.clone();
    resource.vitals = Some(Vitals::new(100, 100, 25, 100).unwrap());
    resource.reply = Some(Reply {
        sequence: 3,
        outcome: Outcome::Rejected(Reason::InsufficientMana),
    });
    let bytes = resource.encode(&catalog).unwrap();
    assert!(bytes.starts_with(wire::RESOURCE_STATE_TAG));
    assert_eq!(bytes.len(), 1082);
    assert_eq!(State::decode(&bytes, &catalog).unwrap(), resource);
    assert!(!resource.same_view(&legacy));
    let at = bytes.len() - 16;
    for (offset, value) in [(0, 101u32), (4, 0), (8, 101), (12, 1_000_001)] {
        let mut invalid = bytes.clone();
        invalid[at + offset..at + offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(State::decode(&invalid, &catalog).is_err());
    }
    for length in 0..bytes.len() {
        assert!(State::decode(&bytes[..length], &catalog).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(State::decode(&trailing, &catalog).is_err());
    let mut wrong = bytes;
    wrong[..4].copy_from_slice(wire::STATE_TAG);
    assert!(State::decode(&wrong, &catalog).is_err());
    resource.vitals = None;
    assert!(resource.encode(&catalog).is_err());
}
