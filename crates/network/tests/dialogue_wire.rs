//! Framing/validation only; no encrypted endpoint or native UI is implied.
use theia_core::{
    content::ContentId,
    coordinates::{GlobalPosition, MAX_GLOBAL_METRES},
    identity::{EntityId, EntityKey, WorldEpoch},
};
use theia_network::{
    dialogue::{Action, Guide, Open, Profile, Request, View},
    dialogue_wire::{self as wire, Presence},
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-dialogue.json");
fn key(epoch: u64, entity: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(epoch).unwrap(),
        entity: EntityId::new(entity).unwrap(),
    }
}
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn profile() -> Profile {
    Profile::from_json(DATA).unwrap()
}

#[test]
fn bounded_requests_roundtrip_and_reject_obsolete_epochs_and_malformed_actions() {
    let cases = [
        Request {
            sequence: 1,
            expected_revision: 0,
            action: Action::Open(key(7, 1)),
        },
        Request {
            sequence: 2,
            expected_revision: 1,
            action: Action::Choose(id("theia.dev.guide.ask-movement")),
        },
        Request {
            sequence: 3,
            expected_revision: 2,
            action: Action::Close,
        },
        Request {
            sequence: u64::MAX,
            expected_revision: u64::MAX,
            action: Action::Choose(id(&"a".repeat(96))),
        },
    ];
    for request in &cases {
        let bytes = wire::encode_request(7, request).unwrap();
        assert!(bytes.len() <= wire::MAX_REQUEST_BYTES);
        assert_eq!(&wire::decode_request(&bytes, 7).unwrap(), request);
        assert!(wire::decode_request(&bytes, 8).is_err());
        assert!(wire::decode_request(&bytes, 0).is_err());
        for end in 0..bytes.len() {
            assert!(
                wire::decode_request(&bytes[..end], 7).is_err(),
                "truncation {end}"
            );
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(wire::decode_request(&trailing, 7).is_err());
        for at in [0, 28] {
            let mut bad = bytes.clone();
            bad[at] = 255;
            assert!(wire::decode_request(&bad, 7).is_err());
        }
        let mut zero = bytes.clone();
        zero[12..20].fill(0);
        assert!(wire::decode_request(&zero, 7).is_err());
    }
    assert_eq!(
        wire::encode_request(7, &cases[3]).unwrap().len(),
        wire::MAX_REQUEST_BYTES
    );
    let mut zero = cases[0].clone();
    zero.sequence = 0;
    assert!(wire::encode_request(7, &zero).is_err());
    assert!(wire::encode_request(0, &cases[2]).is_err());
    assert!(wire::encode_request(8, &cases[0]).is_err());
    let mut bad = wire::encode_request(7, &cases[0]).unwrap();
    bad[29..37].fill(0);
    assert!(wire::decode_request(&bad, 7).is_err());
    let mut bad = wire::encode_request(7, &cases[0]).unwrap();
    bad[37..45].fill(0);
    assert!(wire::decode_request(&bad, 7).is_err());
    let mut bad = wire::encode_request(7, &cases[1]).unwrap();
    bad[29] = 0;
    assert!(wire::decode_request(&bad, 7).is_err());
    let mut bad = wire::encode_request(7, &cases[1]).unwrap();
    bad[30] = b'/';
    assert!(wire::decode_request(&bad, 7).is_err());
    assert!(wire::decode_request(&[0; wire::MAX_REQUEST_BYTES + 1], 7).is_err());
}

#[test]
fn private_views_validate_nodes_and_monotonic_closed_or_open_state_without_aliasing() {
    let profile = profile();
    let actor = key(7, 2);
    let closed = View {
        actor,
        profile_hash: profile.hash(),
        revision: 0,
        open: None,
    };
    let opened = View {
        revision: 1,
        open: Some(Open {
            guide: key(7, 1),
            node: profile.profile().start().id().clone(),
        }),
        ..closed.clone()
    };
    let finished = View {
        revision: 2,
        open: None,
        ..closed.clone()
    };
    for view in [&closed, &opened, &finished] {
        let bytes = wire::encode_view(view, &profile).unwrap();
        assert!(bytes.len() <= wire::MAX_VIEW_BYTES);
        assert_eq!(&wire::decode_view(&bytes, &profile).unwrap(), view);
        for end in 0..bytes.len() {
            assert!(wire::decode_view(&bytes[..end], &profile).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(wire::decode_view(&trailing, &profile).is_err());
        for at in [0, 20, 60] {
            let mut bad = bytes.clone();
            bad[at] = 255;
            assert!(wire::decode_view(&bad, &profile).is_err());
        }
        let mut zero = bytes.clone();
        zero[4..12].fill(0);
        assert!(wire::decode_view(&zero, &profile).is_err());
        let mut zero = bytes.clone();
        zero[12..20].fill(0);
        assert!(wire::decode_view(&zero, &profile).is_err());
    }
    assert!(wire::validate_successor(&closed, actor, None).is_ok());
    assert!(wire::validate_successor(&opened, actor, Some(&closed)).is_ok());
    assert!(wire::validate_successor(&opened, actor, Some(&opened)).is_ok());
    assert!(wire::validate_successor(&finished, actor, Some(&opened)).is_ok());
    assert!(wire::validate_successor(&opened, actor, Some(&finished)).is_err());
    assert!(wire::validate_successor(&opened, key(7, 3), None).is_err());
    let alias = View {
        open: None,
        ..opened.clone()
    };
    assert!(wire::validate_successor(&alias, actor, Some(&opened)).is_err());
    let foreign = View {
        actor: key(8, 2),
        ..finished.clone()
    };
    assert!(wire::validate_successor(&finished, actor, Some(&foreign)).is_err());
    let different = View {
        profile_hash: [0; 32],
        ..finished.clone()
    };
    assert!(wire::validate_successor(&different, actor, Some(&opened)).is_err());
    assert!(wire::encode_view(&different, &profile).is_err());
    for invalid in [
        View {
            revision: 0,
            ..opened.clone()
        },
        View {
            revision: u64::MAX,
            ..opened.clone()
        },
        View {
            open: Some(Open {
                guide: key(8, 1),
                node: profile.profile().start().id().clone(),
            }),
            ..opened.clone()
        },
        View {
            open: Some(Open {
                guide: actor,
                node: profile.profile().start().id().clone(),
            }),
            ..opened.clone()
        },
        View {
            open: Some(Open {
                guide: key(7, 1),
                node: id("missing"),
            }),
            ..opened.clone()
        },
    ] {
        assert!(wire::encode_view(&invalid, &profile).is_err());
    }
    let maximum_id = "a".repeat(96);
    let maximum_profile=Profile::from_json(&serde_json::to_vec(&serde_json::json!({"schema":1,"id":"dialogue","npc":"guide","name":"Guide","start":maximum_id,"nodes":[{"id":maximum_id,"text":"Goodbye.","choices":[]}]})).unwrap()).unwrap();
    let maximum = View {
        profile_hash: maximum_profile.hash(),
        open: Some(Open {
            guide: key(7, 1),
            node: id(&maximum_id),
        }),
        ..opened
    };
    let bytes = wire::encode_view(&maximum, &maximum_profile).unwrap();
    assert_eq!(bytes.len(), wire::MAX_VIEW_BYTES);
    assert_eq!(
        wire::decode_view(&bytes, &maximum_profile).unwrap(),
        maximum
    );
    assert!(wire::decode_view(&[0; wire::MAX_VIEW_BYTES + 1], &profile).is_err());
}

#[test]
fn guide_presence_is_private_bounded_and_uses_checked_global_coordinates() {
    let profile = profile();
    let actor = key(7, 2);
    let presence = Presence {
        actor,
        profile_hash: profile.hash(),
        guide: Some(Guide {
            key: key(7, 1),
            position: GlobalPosition::new([15000.25, 24.5, -12000.75]).unwrap(),
        }),
    };
    let absent = Presence {
        guide: None,
        ..presence.clone()
    };
    for state in [&presence, &absent] {
        let bytes = wire::encode_presence(state, &profile).unwrap();
        assert_eq!(
            &wire::decode_presence(&bytes, &profile, actor).unwrap(),
            state
        );
        assert!(bytes.len() <= wire::MAX_GUIDE_BYTES);
        assert!(wire::decode_presence(&bytes, &profile, key(7, 3)).is_err());
        assert!(wire::decode_presence(&bytes, &profile, key(8, 2)).is_err());
        for end in 0..bytes.len() {
            assert!(wire::decode_presence(&bytes[..end], &profile, actor).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(wire::decode_presence(&trailing, &profile, actor).is_err());
        for at in [0, 20, 52] {
            let mut bad = bytes.clone();
            bad[at] = 255;
            assert!(wire::decode_presence(&bad, &profile, actor).is_err());
        }
    }
    let bytes = wire::encode_presence(&presence, &profile).unwrap();
    assert_eq!(bytes.len(), wire::MAX_GUIDE_BYTES);
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        MAX_GLOBAL_METRES + 1.,
    ] {
        for at in [69, 77, 85] {
            let mut bad = bytes.clone();
            bad[at..at + 8].copy_from_slice(&value.to_le_bytes());
            assert!(wire::decode_presence(&bad, &profile, actor).is_err());
        }
    }
    for invalid in [
        Presence {
            profile_hash: [0; 32],
            ..presence.clone()
        },
        Presence {
            guide: Some(Guide {
                key: actor,
                ..presence.guide.unwrap()
            }),
            ..presence.clone()
        },
        Presence {
            guide: Some(Guide {
                key: key(8, 1),
                ..presence.guide.unwrap()
            }),
            ..presence.clone()
        },
    ] {
        assert!(wire::encode_presence(&invalid, &profile).is_err());
    }
    assert!(wire::decode_presence(&[0; wire::MAX_GUIDE_BYTES + 1], &profile, actor).is_err());
}

#[test]
fn independent_replies_bound_request_acknowledgement_without_predicting_node_changes() {
    use theia_network::dialogue::Rejection;
    let profile = profile();
    let actor = key(7, 2);
    let accepted = wire::Reply {
        actor,
        profile_hash: profile.hash(),
        sequence: 1,
        view_revision: 1,
        rejection: None,
    };
    for rejection in [
        None,
        Some(Rejection::Unavailable),
        Some(Rejection::RateLimited),
        Some(Rejection::StaleSequence),
        Some(Rejection::StaleRevision),
        Some(Rejection::InvalidAction),
        Some(Rejection::OutOfRange),
        Some(Rejection::Exhausted),
    ] {
        let reply = wire::Reply {
            rejection,
            ..accepted.clone()
        };
        let bytes = wire::encode_reply(&reply, &profile).unwrap();
        assert_eq!(bytes.len(), wire::REPLY_BYTES);
        assert_eq!(wire::decode_reply(&bytes, &profile).unwrap(), reply);
        assert!(wire::validate_reply(&reply, actor, 1, None).is_ok());
        for end in 0..bytes.len() {
            assert!(wire::decode_reply(&bytes[..end], &profile).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(wire::decode_reply(&trailing, &profile).is_err());
        for at in [0, 20, 68] {
            let mut bad = bytes.clone();
            bad[at] = 255;
            assert!(wire::decode_reply(&bad, &profile).is_err());
        }
    }
    let rejected = wire::Reply {
        sequence: 2,
        rejection: Some(Rejection::InvalidAction),
        ..accepted.clone()
    };
    assert!(wire::validate_reply(&rejected, actor, 2, Some(&accepted)).is_ok());
    assert_eq!(
        rejected.view_revision, accepted.view_revision,
        "Rejected action has an acknowledgement despite unchanged node state"
    );
    assert!(wire::validate_reply(&accepted, actor, 1, Some(&accepted)).is_ok());
    assert!(wire::validate_reply(&accepted, actor, 0, None).is_err());
    assert!(wire::validate_reply(&accepted, key(7, 3), 1, None).is_err());
    assert!(wire::validate_reply(&accepted, actor, 2, Some(&rejected)).is_err());
    let alias = wire::Reply {
        rejection: Some(Rejection::InvalidAction),
        ..accepted.clone()
    };
    assert!(wire::validate_reply(&alias, actor, 1, Some(&accepted)).is_err());
    let regressed = wire::Reply {
        sequence: 2,
        view_revision: 0,
        ..accepted.clone()
    };
    assert!(wire::validate_reply(&regressed, actor, 2, Some(&accepted)).is_err());
    assert!(
        wire::encode_reply(
            &wire::Reply {
                sequence: 0,
                ..accepted.clone()
            },
            &profile
        )
        .is_err()
    );
    assert!(
        wire::encode_reply(
            &wire::Reply {
                profile_hash: [0; 32],
                ..accepted
            },
            &profile
        )
        .is_err()
    );
}
