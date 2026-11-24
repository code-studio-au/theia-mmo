use super::*;

fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(17).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn authority(rate: TickRate) -> Authority {
    let mut authority = Authority::new(key(1).epoch, rate);
    for id in 1..=MAX_ACTORS as u64 {
        authority.admit(key(id)).unwrap();
    }
    authority
}
fn send(authority: &mut Authority, id: u64, action: Action) -> Result<(), Refusal> {
    let view = authority.view(key(id)).unwrap();
    authority.request(
        key(id),
        Request {
            sequence: view.sequence + 1,
            revision: view.revision,
            action,
        },
    )
}
fn invite(authority: &mut Authority, from: u64, to: u64) -> u64 {
    send(authority, from, Action::Invite(key(to))).unwrap();
    authority
        .view(key(to))
        .unwrap()
        .membership
        .incoming
        .unwrap()
        .id
}
fn join(authority: &mut Authority, from: u64, to: u64) {
    let id = invite(authority, from, to);
    send(authority, to, Action::Accept(id)).unwrap();
}
fn group(authority: &Authority, id: u64) -> Party {
    authority.view(key(id)).unwrap().membership.party.unwrap()
}
fn advance(authority: &mut Authority) {
    authority
        .advance(authority.tick + REQUEST_WINDOW_SECONDS * u64::from(authority.rate.hz()))
        .unwrap();
}

#[test]
fn party_begins_on_accept_and_only_related_private_views_change() {
    let mut a = authority(TickRate::Hz30);
    let unrelated = a.view(key(8)).unwrap();
    let id = invite(&mut a, 1, 2);
    assert!(a.view(key(1)).unwrap().membership.party.is_none());
    assert!(a.view(key(2)).unwrap().membership.party.is_none());
    assert_eq!(a.view(key(8)).unwrap(), unrelated);
    send(&mut a, 2, Action::Accept(id)).unwrap();
    assert_eq!(group(&a, 1), group(&a, 2));
    assert_eq!(group(&a, 1).members, vec![key(1), key(2)]);
    assert_eq!(group(&a, 1).leader, key(1));
    assert!(a.view(key(1)).unwrap().membership.outgoing.is_empty());
    assert!(a.view(key(2)).unwrap().membership.incoming.is_none());
    assert_eq!(a.view(key(8)).unwrap(), unrelated);
}

#[test]
fn reserved_slots_and_out_of_order_accepts_never_exceed_five() {
    let mut a = authority(TickRate::Hz30);
    let ids: Vec<_> = (2..=5).map(|to| invite(&mut a, 1, to)).collect();
    advance(&mut a);
    assert_eq!(send(&mut a, 1, Action::Invite(key(6))), Err(Refusal::Full));
    for offset in [3, 0, 2, 1] {
        send(&mut a, offset + 2, Action::Accept(ids[offset as usize])).unwrap();
    }
    assert_eq!(
        group(&a, 1).members,
        vec![key(1), key(5), key(2), key(4), key(3)]
    );
    assert_eq!(send(&mut a, 1, Action::Invite(key(6))), Err(Refusal::Full));
    send(&mut a, 1, Action::Kick(key(4))).unwrap();
    join(&mut a, 1, 6);
    assert_eq!(group(&a, 6).members.len(), MAX_MEMBERS);
    assert!(a.view(key(4)).unwrap().membership.party.is_none());
}

#[test]
fn ownership_leadership_and_target_incarnations_are_enforced() {
    let mut a = authority(TickRate::Hz30);
    let id = invite(&mut a, 1, 2);
    assert_eq!(
        send(&mut a, 3, Action::Accept(id)),
        Err(Refusal::MissingInvitation)
    );
    assert_eq!(
        send(&mut a, 3, Action::Decline(id)),
        Err(Refusal::MissingInvitation)
    );
    assert_eq!(
        send(&mut a, 2, Action::Cancel(id)),
        Err(Refusal::MissingInvitation)
    );
    send(&mut a, 2, Action::Accept(id)).unwrap();
    assert_eq!(
        send(&mut a, 2, Action::Invite(key(3))),
        Err(Refusal::NotLeader)
    );
    assert_eq!(
        send(&mut a, 2, Action::Kick(key(1))),
        Err(Refusal::NotLeader)
    );
    advance(&mut a);
    assert_eq!(
        send(&mut a, 2, Action::Promote(key(1))),
        Err(Refusal::NotLeader)
    );
    assert_eq!(
        send(&mut a, 1, Action::Kick(key(1))),
        Err(Refusal::SelfTarget)
    );
    assert_eq!(
        send(&mut a, 1, Action::Promote(key(3))),
        Err(Refusal::NotMember)
    );
    let wrong_epoch = EntityKey {
        epoch: WorldEpoch::new(18).unwrap(),
        ..key(3)
    };
    assert_eq!(
        send(&mut a, 1, Action::Invite(wrong_epoch)),
        Err(Refusal::MissingPlayer)
    );
    let before = a.view(key(1)).unwrap();
    assert_eq!(
        a.request(
            EntityKey {
                epoch: wrong_epoch.epoch,
                ..key(1)
            },
            Request {
                sequence: before.sequence + 1,
                revision: before.revision,
                action: Action::Leave,
            }
        ),
        Err(Refusal::Context)
    );
    assert_eq!(a.view(key(1)).unwrap(), before);
}

#[test]
fn promotion_and_leader_departure_cancel_invites_and_preserve_join_order() {
    let mut a = authority(TickRate::Hz30);
    join(&mut a, 1, 2);
    join(&mut a, 1, 3);
    let pending = invite(&mut a, 1, 4);
    send(&mut a, 1, Action::Promote(key(3))).unwrap();
    assert_eq!(group(&a, 2).leader, key(3));
    assert_eq!(group(&a, 2).members, vec![key(1), key(2), key(3)]);
    assert_eq!(
        send(&mut a, 4, Action::Accept(pending)),
        Err(Refusal::MissingInvitation)
    );
    let next = invite(&mut a, 3, 4);
    send(&mut a, 3, Action::Leave).unwrap();
    assert_eq!(group(&a, 2).leader, key(1));
    assert_eq!(
        send(&mut a, 4, Action::Accept(next)),
        Err(Refusal::MissingInvitation)
    );
    advance(&mut a);
    send(&mut a, 1, Action::Leave).unwrap();
    assert!(a.view(key(2)).unwrap().membership.party.is_none());
}

#[test]
fn competing_invites_cannot_merge_groups_or_reanimate_cancelled_requests() {
    let mut a = authority(TickRate::Hz30);
    let outer = invite(&mut a, 3, 1);
    let first = invite(&mut a, 1, 2);
    let own = invite(&mut a, 2, 4);
    send(&mut a, 2, Action::Accept(first)).unwrap();
    assert!(a.view(key(1)).unwrap().membership.incoming.is_none());
    assert!(a.view(key(3)).unwrap().membership.outgoing.is_empty());
    assert!(a.view(key(4)).unwrap().membership.incoming.is_none());
    assert_eq!(
        send(&mut a, 1, Action::Accept(outer)),
        Err(Refusal::MissingInvitation)
    );
    assert_eq!(
        send(&mut a, 4, Action::Accept(own)),
        Err(Refusal::MissingInvitation)
    );
    assert_eq!(
        send(&mut a, 3, Action::Invite(key(1))),
        Err(Refusal::AlreadyGrouped)
    );
    assert_eq!(group(&a, 1).members, vec![key(1), key(2)]);
}

#[test]
fn pending_conflicts_decline_cancel_and_dissolution_release_slots() {
    let mut a = authority(TickRate::Hz30);
    let first = invite(&mut a, 1, 2);
    assert_eq!(
        send(&mut a, 3, Action::Invite(key(2))),
        Err(Refusal::InvitationPending)
    );
    send(&mut a, 2, Action::Decline(first)).unwrap();
    let second = invite(&mut a, 3, 2);
    send(&mut a, 3, Action::Cancel(second)).unwrap();
    join(&mut a, 1, 2);
    let third = invite(&mut a, 1, 4);
    send(&mut a, 2, Action::Leave).unwrap();
    assert!(a.view(key(1)).unwrap().membership.party.is_none());
    assert!(a.view(key(4)).unwrap().membership.incoming.is_none());
    assert_eq!(
        send(&mut a, 4, Action::Accept(third)),
        Err(Refusal::MissingInvitation)
    );
}

#[test]
fn disconnect_retires_all_links_without_reusing_an_incarnation() {
    let mut a = authority(TickRate::Hz30);
    join(&mut a, 1, 2);
    join(&mut a, 1, 3);
    invite(&mut a, 1, 4);
    a.retire(key(1)).unwrap();
    assert!(a.view(key(1)).is_none());
    assert_eq!(group(&a, 2).leader, key(2));
    assert!(a.view(key(4)).unwrap().membership.incoming.is_none());
    assert_eq!(a.admit(key(1)), Err(Refusal::Context));
    a.admit(key(9)).unwrap();
    assert_eq!(a.view(key(9)).unwrap().membership, Membership::default());
    a.retire(key(2)).unwrap();
    assert!(a.view(key(3)).unwrap().membership.party.is_none());
    assert_eq!(a.retire(key(2)), Err(Refusal::Context));
}

#[test]
fn expiry_uses_exact_simulation_seconds_at_all_supported_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut a = authority(rate);
        a.advance(11).unwrap();
        let id = invite(&mut a, 1, 2);
        let expires = 11 + INVITATION_SECONDS * u64::from(rate.hz());
        a.advance(expires - 1).unwrap();
        assert_eq!(a.view(key(2)).unwrap().membership.incoming.unwrap().id, id);
        let before = a.view(key(2)).unwrap();
        assert_eq!(a.advance(expires - 2), Err(Refusal::Context));
        assert_eq!(a.view(key(2)).unwrap(), before);
        a.advance(expires).unwrap();
        assert!(a.view(key(2)).unwrap().membership.incoming.is_none());
        assert_eq!(
            send(&mut a, 2, Action::Accept(id)),
            Err(Refusal::MissingInvitation)
        );
        join(&mut a, 1, 2);
        a.advance(expires + 100_000).unwrap();
        assert_eq!(group(&a, 1).members.len(), 2);
    }
}

#[test]
fn stale_refusals_consume_sequences_without_affecting_membership() {
    let mut a = authority(TickRate::Hz30);
    let stale = a.view(key(2)).unwrap();
    let id = invite(&mut a, 1, 2);
    let request = Request {
        sequence: 1,
        revision: stale.revision,
        action: Action::Accept(id),
    };
    assert_eq!(a.request(key(2), request), Err(Refusal::Stale));
    let before = a.view(key(2)).unwrap();
    assert_eq!(before.sequence, 1);
    assert_eq!(
        a.request(
            key(2),
            Request {
                revision: before.revision,
                ..request
            }
        ),
        Err(Refusal::Sequence)
    );
    assert_eq!(a.view(key(2)).unwrap(), before);
    send(&mut a, 2, Action::Accept(id)).unwrap();
    let next = Request {
        sequence: 2 + MAX_SEQUENCE_ADVANCE + 1,
        revision: group(&a, 2).id,
        action: Action::Leave,
    };
    assert_eq!(a.request(key(2), next), Err(Refusal::Sequence));
    assert_eq!(group(&a, 2).members.len(), 2);
}

#[test]
fn refusals_are_rate_bounded_and_replays_do_not_extend_the_window() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut a = authority(rate);
        for _ in 0..REQUEST_BURST {
            assert_eq!(
                send(&mut a, 1, Action::Invite(key(1))),
                Err(Refusal::SelfTarget)
            );
        }
        assert_eq!(
            send(&mut a, 1, Action::Invite(key(2))),
            Err(Refusal::RateLimited)
        );
        a.advance(REQUEST_WINDOW_SECONDS * u64::from(rate.hz()) - 1)
            .unwrap();
        assert_eq!(
            send(&mut a, 1, Action::Invite(key(2))),
            Err(Refusal::RateLimited)
        );
        a.advance(REQUEST_WINDOW_SECONDS * u64::from(rate.hz()))
            .unwrap();
        invite(&mut a, 1, 2);
        let view = a.view(key(1)).unwrap();
        assert_eq!(view.sequence, 7);
        for _ in 0..1000 {
            assert_eq!(
                a.request(
                    key(1),
                    Request {
                        sequence: view.sequence,
                        revision: view.revision,
                        action: Action::Leave
                    }
                ),
                Err(Refusal::Sequence)
            );
        }
        invite(&mut a, 1, 3);
        invite(&mut a, 1, 4);
        invite(&mut a, 1, 5);
    }
}

#[test]
fn exhausted_counters_and_capacity_fail_without_partial_mutation() {
    let mut a = authority(TickRate::Hz30);
    assert_eq!(a.admit(key(9)), Err(Refusal::Context));
    a.serial = u64::MAX;
    assert_eq!(
        send(&mut a, 1, Action::Invite(key(2))),
        Err(Refusal::Exhausted)
    );
    assert!(a.view(key(2)).unwrap().membership.incoming.is_none());
    a.serial = 0;
    a.advance(u64::MAX).unwrap();
    assert_eq!(
        send(&mut a, 1, Action::Invite(key(2))),
        Err(Refusal::Exhausted)
    );
    assert!(a.invitations.is_empty());
    assert_eq!(a.serial, 0);
    a.actors.get_mut(&key(2).entity).unwrap().revision = u64::MAX;
    let before = a.view(key(1)).unwrap();
    assert_eq!(a.retire(key(1)), Err(Refusal::Exhausted));
    assert_eq!(a.view(key(1)).unwrap(), before);
}

#[test]
fn mixed_commands_preserve_unique_membership_and_bounded_pending_links() {
    let mut a = authority(TickRate::Hz30);
    let mut random = 0x122337_u64;
    for step in 0..10_000 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let caller = 1 + ((random >> 32) % 8);
        let target = 1 + ((random >> 40) % 8);
        let view = a.view(key(caller)).unwrap();
        let incoming = view.membership.incoming.map_or(0, |i| i.id);
        let outgoing = view.membership.outgoing.first().map_or(0, |i| i.id);
        let action = match (random >> 48) % 7 {
            0 => Action::Invite(key(target)),
            1 => Action::Accept(incoming),
            2 => Action::Decline(incoming),
            3 => Action::Cancel(outgoing),
            4 => Action::Leave,
            5 => Action::Kick(key(target)),
            _ => Action::Promote(key(target)),
        };
        let _ = send(&mut a, caller, action);
        a.advance(step * 7).unwrap();
        for id in 1..=8 {
            let count = a
                .parties
                .values()
                .filter(|p| p.members.contains(&key(id)))
                .count();
            assert!(count <= 1);
            assert!(
                a.invitations
                    .values()
                    .filter(|i| i.recipient == key(id))
                    .count()
                    <= 1
            );
        }
        for p in a.parties.values() {
            assert!((2..=MAX_MEMBERS).contains(&p.members.len()));
            assert!(p.members.contains(&p.leader));
            let reserved = a
                .invitations
                .values()
                .filter(|i| i.party == Some(p.id))
                .count();
            assert!(p.members.len() + reserved <= MAX_MEMBERS);
        }
        assert!(a.invitations.len() <= MAX_ACTORS);
        for i in a.invitations.values() {
            assert!(i.expires_at > a.tick);
            assert!(a.group(i.recipient).is_none());
            assert_eq!(a.group(i.inviter).map(|p| p.id), i.party);
            assert!(a.group(i.inviter).is_none_or(|p| p.leader == i.inviter));
        }
    }
}
