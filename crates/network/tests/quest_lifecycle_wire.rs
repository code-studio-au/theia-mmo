//! Pure wire contracts only; no live authority, SQL or NPC entitlement claim.
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::lifecycle::{Action, Command, State as Progress, Status},
};
use theia_network::quest_lifecycle_wire::{
    self as wire, Profile, Rejection, Reply, Request, State,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-quest.json");
fn profile() -> Profile {
    Profile::from_json(DATA).unwrap()
}
fn actor(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn state(p: &Profile, status: Status, count: u32) -> State {
    let revision = match status {
        Status::Available => 0,
        Status::Accepted => u64::from(count) + 1,
        Status::TurnedIn => u64::from(count) + 2,
    };
    State {
        actor: actor(9),
        profile_hash: p.hash(),
        progress: Progress::restore(p.profile(), revision, status, count).unwrap(),
    }
}
fn request() -> Request {
    Request {
        actor: actor(9),
        sequence: 1,
        command: Command {
            expected_revision: 0,
            action: Action::Accept,
        },
        dialogue_revision: 1,
        npc: actor(1),
    }
}
fn reply(p: &Profile) -> Reply {
    Reply {
        actor: actor(9),
        profile_hash: p.hash(),
        sequence: 1,
        quest_revision: 1,
        rejection: None,
    }
}
#[test]
fn lifecycle_frames_preserve_checked_phases_and_reject_malformed_or_aliased_observations() {
    let p = profile();
    for s in [
        state(&p, Status::Available, 0),
        state(&p, Status::Accepted, 0),
        state(&p, Status::Accepted, 3),
        state(&p, Status::TurnedIn, 3),
    ] {
        let bytes = s.encode(&p).unwrap();
        assert_eq!(bytes.len(), wire::STATE_BYTES);
        assert_eq!(State::decode(&bytes, &p).unwrap(), s);
        for len in 0..bytes.len() {
            assert!(State::decode(&bytes[..len], &p).is_err());
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(State::decode(&extra, &p).is_err());
        for range in [0..4, 4..12, 12..20, 20..52] {
            let mut bad = bytes.clone();
            bad[range].fill(0);
            assert!(State::decode(&bad, &p).is_err());
        }
        assert!(
            theia_network::quest_wire::State::decode(
                &bytes,
                &theia_network::quest_wire::Profile::from_json(DATA).unwrap()
            )
            .is_err()
        );
    }
    let bytes = state(&p, Status::Accepted, 3).encode(&p).unwrap();
    for code in 0..=u8::MAX {
        let mut bad = bytes.clone();
        bad[64] = code;
        assert_eq!(State::decode(&bad, &p).is_ok(), code == 1);
    }
    let mut bad = bytes.clone();
    bad[52..60].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(State::decode(&bad, &p).is_err());
    let mut bad = bytes.clone();
    bad[60..64].copy_from_slice(&101_u32.to_le_bytes());
    assert!(State::decode(&bad, &p).is_err());
}
#[test]
fn private_successors_allow_acceptance_and_turn_in_without_count_increments_but_never_rollback() {
    let p = profile();
    let available = state(&p, Status::Available, 0);
    let active = state(&p, Status::Accepted, 0);
    let ready = state(&p, Status::Accepted, 3);
    let done = state(&p, Status::TurnedIn, 3);
    for (before, after) in [
        (&available, &active),
        (&active, &ready),
        (&ready, &done),
        (&available, &done),
        (&done, &done),
    ] {
        after
            .validate_successor(&p, actor(9), Some(before))
            .unwrap();
    }
    for (before, after) in [(&active, &available), (&ready, &active), (&done, &ready)] {
        assert!(
            after
                .validate_successor(&p, actor(9), Some(before))
                .is_err()
        );
    }
    assert!(done.validate_successor(&p, actor(10), None).is_err());
    let mut foreign = done.clone();
    foreign.actor = actor(10);
    assert!(
        foreign
            .validate_successor(&p, actor(9), Some(&ready))
            .is_err()
    );
}
#[test]
fn intents_bind_displayed_revisions_npc_incarnation_and_action_without_client_credit_fields() {
    let r = request();
    let bytes = r.encode().unwrap();
    assert_eq!(bytes.len(), wire::REQUEST_BYTES);
    assert_eq!(Request::decode(&bytes).unwrap(), r);
    for len in 0..bytes.len() {
        assert!(Request::decode(&bytes[..len]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(Request::decode(&extra).is_err());
    for range in [0..4, 4..12, 12..20, 20..28, 36..44, 44..52, 52..60] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(Request::decode(&bad).is_err());
    }
    for code in 0..=u8::MAX {
        let mut bad = bytes.clone();
        bad[60] = code;
        assert_eq!(Request::decode(&bad).is_ok(), matches!(code, 1 | 2));
    }
    let mut bad = r;
    bad.command.expected_revision = 102;
    assert!(bad.encode().is_err());
    let mut bad = r;
    bad.npc = r.actor;
    assert!(bad.encode().is_err());
    let mut bad = r;
    bad.npc.epoch = WorldEpoch::new(8).unwrap();
    assert!(bad.encode().is_err());
    assert!(theia_network::vendor_wire::Request::decode(&bytes).is_err());
    let mut turn_in = r;
    turn_in.command = Command {
        expected_revision: 4,
        action: Action::TurnIn,
    };
    assert_ne!(turn_in.encode().unwrap(), bytes);
    assert_eq!(
        Request::decode(&turn_in.encode().unwrap()).unwrap(),
        turn_in
    );
}
#[test]
fn action_replies_roundtrip_bounded_success_and_normal_refusal_without_aliases() {
    let p = profile();
    let r = reply(&p);
    let bytes = r.encode(&p).unwrap();
    assert_eq!(bytes.len(), wire::REPLY_BYTES);
    assert_eq!(Reply::decode(&bytes, &p).unwrap(), r);
    for len in 0..bytes.len() {
        assert!(Reply::decode(&bytes[..len], &p).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(Reply::decode(&extra, &p).is_err());
    for range in [0..4, 4..12, 12..20, 20..52, 52..60] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(Reply::decode(&bad, &p).is_err());
    }
    for code in 0..=u8::MAX {
        let mut bad = bytes.clone();
        bad[68] = code;
        assert_eq!(Reply::decode(&bad, &p).is_ok(), code <= 6);
    }
    for reason in [
        Rejection::NotEntitled,
        Rejection::StaleRevision,
        Rejection::NotAvailable,
        Rejection::NotReady,
        Rejection::AlreadyTurnedIn,
        Rejection::Busy,
    ] {
        let rejected = Reply {
            rejection: Some(reason),
            ..r
        };
        assert_eq!(
            Reply::decode(&rejected.encode(&p).unwrap(), &p).unwrap(),
            rejected
        );
    }
    let mut bad = r;
    bad.quest_revision = 6;
    assert!(bad.encode(&p).is_err());
}
#[test]
fn reply_sequence_checks_reject_foreign_unsent_and_regressed_acknowledgements() {
    let p = profile();
    let old = reply(&p);
    old.validate_successor(&p, actor(9), 1, None).unwrap();
    old.validate_successor(&p, actor(9), 1, Some(old)).unwrap();
    let next = Reply {
        sequence: 2,
        quest_revision: 4,
        ..old
    };
    next.validate_successor(&p, actor(9), 2, Some(old)).unwrap();
    for bad in [
        Reply {
            actor: actor(10),
            ..next
        },
        Reply {
            sequence: 3,
            ..next
        },
        Reply {
            quest_revision: 0,
            ..next
        },
        Reply {
            rejection: Some(Rejection::Busy),
            ..old
        },
    ] {
        assert!(bad.validate_successor(&p, actor(9), 2, Some(old)).is_err());
    }
    assert!(old.validate_successor(&p, actor(9), 2, Some(next)).is_err());
}
#[test]
fn exact_byte_domains_distinguish_lifecycle_from_automatic_profiles_and_worlds() {
    let p = profile();
    let legacy = theia_network::quest_wire::Profile::from_json(DATA).unwrap();
    assert_ne!(p.hash(), legacy.hash());
    let padded = Profile::from_json(&[DATA, b"\n"].concat()).unwrap();
    assert_ne!(p.hash(), padded.hash());
    let s = state(&p, Status::Accepted, 0);
    assert!(State::decode(&s.encode(&p).unwrap(), &padded).is_err());
    let r = reply(&p);
    assert!(Reply::decode(&r.encode(&p).unwrap(), &padded).is_err());
    assert_ne!(
        wire::profile_identity([1; 32], p.hash()),
        theia_network::quest_wire::profile_identity([1; 32], legacy.hash())
    );
    assert_ne!(
        wire::profile_identity([1; 32], p.hash()),
        wire::profile_identity([2; 32], p.hash())
    );
    assert_ne!(
        wire::profile_identity([1; 32], p.hash()),
        wire::profile_identity([1; 32], padded.hash())
    );
}

#[test]
fn completion_xp_mode_pins_exact_recipe_progression_and_a_distinct_state_identity() {
    let xp_bytes = include_bytes!("../../../content/gameplay/development-progression.json");
    let recipe = include_bytes!("../../../content/gameplay/development-quest-turn-in-xp.json");
    let xp = theia_network::progression_wire::Profile::from_json(xp_bytes).unwrap();
    let p = Profile::from_json_with_turn_in_xp(DATA, recipe, &xp).unwrap();
    assert_eq!(
        p.hash(),
        [
            75, 208, 22, 15, 231, 89, 117, 241, 99, 77, 123, 190, 136, 210, 147, 85, 222, 171, 182,
            91, 126, 21, 242, 172, 83, 178, 103, 139, 205, 173, 27, 74
        ]
    );
    assert_eq!(p.turn_in_xp().unwrap().experience(), 25);
    assert!(p.check_progression(&xp).is_ok());
    let old = profile();
    assert_ne!(p.hash(), old.hash());
    let bytes = state(&p, Status::TurnedIn, 3).encode(&p).unwrap();
    assert_eq!(bytes.len(), wire::STATE_BYTES);
    assert!(State::decode(&bytes, &old).is_err());
    assert!(State::decode(&state(&old, Status::Accepted, 3).encode(&old).unwrap(), &p).is_err());
    let changed =
        theia_network::progression_wire::Profile::from_json(&[xp_bytes.as_slice(), b" "].concat())
            .unwrap();
    assert!(p.check_progression(&changed).is_err());
    assert!(old.check_progression(&changed).is_ok());
    assert_ne!(
        p.hash(),
        Profile::from_json_with_turn_in_xp(DATA, &[recipe.as_slice(), b" "].concat(), &xp)
            .unwrap()
            .hash()
    );
    assert_ne!(
        p.hash(),
        Profile::from_json_with_turn_in_xp(DATA, recipe, &changed)
            .unwrap()
            .hash()
    );
    assert_ne!(
        wire::profile_identity([1; 32], p.hash()),
        wire::profile_identity([1; 32], old.hash())
    );
}
