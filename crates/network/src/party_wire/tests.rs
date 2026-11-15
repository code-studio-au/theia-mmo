use super::*;
use theia_core::time::TickRate;

fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(21).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn authority() -> party::Authority {
    let mut authority = party::Authority::new(key(1).epoch, TickRate::Hz30);
    for id in 1..=8 {
        authority.admit(key(id)).unwrap();
    }
    authority.advance(1).unwrap();
    authority
}
fn state(authority: &party::Authority, id: u64, refusal: Option<Refusal>) -> State {
    State {
        world: [17; 32],
        tick: 1,
        view: authority.view(key(id)).unwrap(),
        refusal,
        roster: (1..=8)
            .map(|id| NamedActor {
                key: key(id),
                name: CharacterName::parse(
                    [
                        "Arden", "Birch", "Cedar", "Dawn", "Elowen", "Fern", "Glen", "Hazel",
                    ][(id - 1) as usize],
                )
                .unwrap(),
            })
            .collect(),
    }
}
fn request(authority: &mut party::Authority, id: u64, action: Action) -> Option<Refusal> {
    let view = authority.view(key(id)).unwrap();
    let frame = Request {
        epoch: key(id).epoch,
        intent: party::Request {
            sequence: view.sequence + 1,
            revision: view.revision,
            action,
        },
    };
    let decoded = Request::decode(&frame.encode().unwrap()).unwrap();
    assert_eq!(decoded, frame);
    authority.request(key(id), decoded.intent).err()
}
fn round_trip(state: State) {
    let bytes = state.encode().unwrap();
    assert!(bytes.len() <= MAX_STATE_BYTES);
    assert_eq!(State::decode(&bytes).unwrap(), state);
    for length in 0..bytes.len() {
        assert!(State::decode(&bytes[..length]).is_err(), "length {length}");
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(State::decode(&trailing).is_err());
}

#[test]
fn real_authority_states_round_trip_for_solo_invitation_and_full_party() {
    let mut a = authority();
    round_trip(state(&a, 1, None));
    for id in 2..=5 {
        assert_eq!(request(&mut a, 1, Action::Invite(key(id))), None);
    }
    for id in 1..=8 {
        round_trip(state(&a, id, None));
    }
    for id in 2..=5 {
        let invite = a.view(key(id)).unwrap().membership.incoming.unwrap().id;
        assert_eq!(request(&mut a, id, Action::Accept(invite)), None);
        round_trip(state(&a, 1, None));
        round_trip(state(&a, id, None));
    }
    let refused = request(&mut a, 2, Action::Invite(key(6)));
    assert_eq!(refused, Some(Refusal::NotLeader));
    round_trip(state(&a, 2, refused));
    let mut longest = state(&a, 1, None);
    for name in &mut longest.roster {
        name.name = CharacterName::parse("Abcdefghijklmnopqrst").unwrap();
    }
    round_trip(longest);
}

#[test]
fn all_actions_are_canonical_and_invalid_discriminants_lengths_and_ids_refuse() {
    for action in [
        Action::Invite(key(2)),
        Action::Accept(3),
        Action::Decline(3),
        Action::Cancel(3),
        Action::Leave,
        Action::Kick(key(2)),
        Action::Promote(key(2)),
    ] {
        let frame = Request {
            epoch: key(1).epoch,
            intent: party::Request {
                sequence: 9,
                revision: 7,
                action,
            },
        };
        let bytes = frame.encode().unwrap();
        assert_eq!(bytes.len(), REQUEST_BYTES);
        assert_eq!(Request::decode(&bytes).unwrap(), frame);
        for length in 0..bytes.len() {
            assert!(Request::decode(&bytes[..length]).is_err());
        }
        for at in [4, 12, 20] {
            let mut invalid = bytes.clone();
            invalid[at..at + 8].fill(0);
            assert!(Request::decode(&invalid).is_err());
        }
        for kind in [0, 8, 255] {
            let mut invalid = bytes.clone();
            invalid[28] = kind;
            assert!(Request::decode(&invalid).is_err());
        }
        let mut invalid = bytes.clone();
        invalid.push(0);
        assert!(Request::decode(&invalid).is_err());
        let mut invalid = bytes;
        if action == Action::Leave {
            invalid[29] = 1;
        } else {
            invalid[29..].fill(0);
        }
        assert!(Request::decode(&invalid).is_err());
    }
    let foreign = EntityKey {
        epoch: WorldEpoch::new(22).unwrap(),
        ..key(2)
    };
    assert!(
        Request {
            epoch: key(1).epoch,
            intent: party::Request {
                sequence: 1,
                revision: 1,
                action: Action::Invite(foreign)
            }
        }
        .encode()
        .is_err()
    );
}

#[test]
fn conflicting_membership_roster_and_invitation_shapes_fail_before_publication() {
    let mut a = authority();
    request(&mut a, 1, Action::Invite(key(2)));
    let pending = state(&a, 1, None);
    let invite = a.view(key(2)).unwrap().membership.incoming.unwrap().id;
    request(&mut a, 2, Action::Accept(invite));
    request(&mut a, 1, Action::Invite(key(3)));
    let grouped = state(&a, 1, None);
    for kind in 0..16 {
        let mut invalid = grouped.clone();
        match kind {
            0 => invalid.world = [0; 32],
            1 => invalid.tick = 0,
            2 => invalid.view.revision = 0,
            3 => invalid.roster.clear(),
            4 => invalid.roster.push(invalid.roster[0].clone()),
            5 => {
                invalid.roster.remove(0);
            }
            6 => invalid.roster[1].key.epoch = WorldEpoch::new(22).unwrap(),
            7 => invalid.view.membership.party.as_mut().unwrap().leader = key(8),
            8 => invalid.view.membership.party.as_mut().unwrap().members[1] = key(1),
            9 => {
                invalid
                    .view
                    .membership
                    .party
                    .as_mut()
                    .unwrap()
                    .members
                    .remove(0);
            }
            10 => {
                invalid.view.membership.party.as_mut().unwrap().id = 0;
            }
            11 => {
                invalid.view.membership.outgoing[0].expires_at = invalid.tick;
            }
            12 => {
                invalid.view.membership.outgoing[0].inviter = key(2);
            }
            13 => {
                invalid.view.membership.outgoing[0].recipient = key(2);
            }
            14 => {
                invalid.view.membership.outgoing[0].party = None;
            }
            _ => {
                invalid
                    .view
                    .membership
                    .outgoing
                    .push(invalid.view.membership.outgoing[0].clone());
            }
        };
        assert!(invalid.encode().is_err(), "kind {kind}");
    }
    let mut zero_sequence = pending.clone();
    zero_sequence.view.sequence = 0;
    zero_sequence.refusal = Some(Refusal::Stale);
    assert!(zero_sequence.encode().is_err());
    let mut oversized = pending;
    oversized.view.membership.outgoing = vec![oversized.view.membership.outgoing[0].clone(); 1000];
    assert!(oversized.encode().is_err());
}

#[test]
fn owner_world_revision_ack_and_clock_are_validated_together() {
    let mut a = authority();
    let first = state(&a, 1, None);
    request(&mut a, 1, Action::Invite(key(2)));
    let latest = state(&a, 1, None);
    latest
        .validate_successor(key(1), first.world, Some(&first))
        .unwrap();
    latest
        .validate_successor(key(1), first.world, Some(&latest))
        .unwrap();
    assert!(
        latest
            .validate_successor(key(2), first.world, None)
            .is_err()
    );
    assert!(latest.validate_successor(key(1), [18; 32], None).is_err());
    assert!(
        first
            .validate_successor(key(1), first.world, Some(&latest))
            .is_err()
    );
    let mut conflict = latest.clone();
    conflict.view.membership.outgoing.clear();
    assert!(
        conflict
            .validate_successor(key(1), first.world, Some(&latest))
            .is_err()
    );
    let mut conflict = latest.clone();
    conflict.refusal = Some(Refusal::Stale);
    assert!(
        conflict
            .validate_successor(key(1), first.world, Some(&latest))
            .is_err()
    );
    let mut advanced = latest.clone();
    advanced.tick += 1;
    advanced
        .validate_successor(key(1), first.world, Some(&latest))
        .unwrap();
    assert!(
        latest
            .validate_successor(key(1), first.world, Some(&advanced))
            .is_err()
    );
    // Unrelated trusted arrivals may update labels without changing membership.
    let mut roster = latest.clone();
    roster.roster.pop();
    roster
        .validate_successor(key(1), first.world, Some(&latest))
        .unwrap();
}

#[test]
fn bounded_arbitrary_input_never_panics_and_valid_decodes_are_canonical() {
    let mut random = 0xabcd_u64;
    for length in 0..=MAX_STATE_BYTES + 1 {
        let mut bytes = vec![0; length];
        for byte in &mut bytes {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (random >> 32) as u8;
        }
        if length >= 4 {
            bytes[..4].copy_from_slice(STATE_TAG);
        }
        if let Ok(state) = State::decode(&bytes) {
            assert_eq!(state.encode().unwrap(), bytes);
        }
    }
    let baseline = state(&authority(), 1, None).encode().unwrap();
    for at in 0..baseline.len() {
        for value in [0, 1, 8, 13, 255] {
            let mut bytes = baseline.clone();
            bytes[at] = value;
            if let Ok(state) = State::decode(&bytes) {
                assert_eq!(state.encode().unwrap(), bytes);
            }
        }
    }
}
