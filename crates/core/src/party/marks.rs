//! Stable unit symbols shared only inside the owning party. No positions.
use crate::identity::EntityKey;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Symbol {
    Skull = 1,
    Cross,
    Square,
    Moon,
    Triangle,
    Diamond,
    Circle,
    Star,
}
impl Symbol {
    pub const ALL: [Self; 8] = [
        Self::Skull,
        Self::Cross,
        Self::Square,
        Self::Moon,
        Self::Triangle,
        Self::Diamond,
        Self::Circle,
        Self::Star,
    ];
    pub fn from_byte(value: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|s| *s as u8 == value)
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Skull => "Skull",
            Self::Cross => "Cross",
            Self::Square => "Square",
            Self::Moon => "Moon",
            Self::Triangle => "Triangle",
            Self::Diamond => "Diamond",
            Self::Circle => "Circle",
            Self::Star => "Star",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    pub target: EntityKey,
    pub symbol: Symbol,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        identity::{EntityId, WorldEpoch},
        party::{Action, Authority, Refusal, Request},
        time::TickRate,
    };
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn send(a: &mut Authority, id: u64, action: Action) -> Result<(), Refusal> {
        a.advance(a.tick + 3 * u64::from(a.rate.hz())).unwrap();
        let v = a.view(key(id)).unwrap();
        a.request_with_targets(
            key(id),
            Request {
                sequence: v.sequence + 1,
                revision: v.revision,
                action,
            },
            |k| k.entity.value() < 200,
        )
    }
    fn join(a: &mut Authority, leader: u64, member: u64) {
        send(a, leader, Action::Invite(key(member))).unwrap();
        let invitation = a.view(key(member)).unwrap().membership.incoming.unwrap().id;
        send(a, member, Action::Accept(invitation)).unwrap();
    }
    fn marks(a: &Authority, id: u64) -> Vec<Mark> {
        a.view(key(id))
            .unwrap()
            .membership
            .party
            .map_or_else(Vec::new, |p| p.marks)
    }
    fn set(target: u64, symbol: Symbol) -> Action {
        Action::Mark {
            target: key(target),
            symbol: Some(symbol),
        }
    }
    #[test]
    fn assignments_are_bounded_unique_private_and_canonical_at_all_rates() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let mut a = Authority::new(key(1).epoch, rate);
            for id in 1..=6 {
                a.admit(key(id)).unwrap();
            }
            join(&mut a, 1, 2);
            join(&mut a, 1, 3);
            join(&mut a, 4, 5);
            for (i, symbol) in Symbol::ALL.into_iter().enumerate().rev() {
                send(&mut a, 1, set(100 + i as u64, symbol)).unwrap();
            }
            assert_eq!(marks(&a, 1).len(), 8);
            assert_eq!(marks(&a, 1), marks(&a, 2));
            assert!(marks(&a, 4).is_empty() && marks(&a, 6).is_empty());
            send(&mut a, 4, set(100, Symbol::Circle)).unwrap();
            assert_eq!(
                marks(&a, 4),
                vec![Mark {
                    target: key(100),
                    symbol: Symbol::Circle
                }]
            );
            assert_eq!(
                marks(&a, 1)[0],
                Mark {
                    target: key(100),
                    symbol: Symbol::Skull
                }
            );
            assert_eq!(
                marks(&a, 1).iter().map(|m| m.symbol).collect::<Vec<_>>(),
                Symbol::ALL
            );
            send(&mut a, 1, set(101, Symbol::Skull)).unwrap();
            assert_eq!(marks(&a, 1).len(), 7);
            assert_eq!(
                marks(&a, 1)[0],
                Mark {
                    target: key(101),
                    symbol: Symbol::Skull
                }
            );
            let revision = a.view(key(1)).unwrap().revision;
            send(&mut a, 1, set(101, Symbol::Skull)).unwrap();
            assert_eq!(a.view(key(1)).unwrap().revision, revision);
            send(
                &mut a,
                1,
                Action::Mark {
                    target: key(101),
                    symbol: None,
                },
            )
            .unwrap();
            assert_eq!(marks(&a, 1).len(), 6);
            send(&mut a, 1, Action::ClearMarks).unwrap();
            assert!(marks(&a, 1).is_empty());
        }
    }
    #[test]
    fn marker_requests_share_the_bounded_membership_rate_window() {
        let mut a = Authority::new(key(1).epoch, TickRate::Hz30);
        a.admit(key(1)).unwrap();
        a.admit(key(2)).unwrap();
        join(&mut a, 1, 2);
        for (i, symbol) in Symbol::ALL.into_iter().take(4).enumerate() {
            let v = a.view(key(1)).unwrap();
            a.request_with_targets(
                key(1),
                Request {
                    sequence: v.sequence + 1,
                    revision: v.revision,
                    action: set(100 + i as u64, symbol),
                },
                |_| true,
            )
            .unwrap();
        }
        let v = a.view(key(1)).unwrap();
        assert_eq!(
            a.request_with_targets(
                key(1),
                Request {
                    sequence: v.sequence + 1,
                    revision: v.revision,
                    action: Action::ClearMarks
                },
                |_| true
            ),
            Err(Refusal::RateLimited)
        );
        assert_eq!(marks(&a, 1).len(), 4);
        assert_eq!(a.view(key(1)).unwrap().sequence, v.sequence + 1);
        assert_eq!(a.view(key(1)).unwrap().revision, v.revision);
    }
    #[test]
    fn permissions_stale_replay_and_retirement_do_not_leak_or_resurrect_assignments() {
        let mut a = Authority::new(key(1).epoch, TickRate::Hz30);
        for id in 1..=4 {
            a.admit(key(id)).unwrap();
        }
        join(&mut a, 1, 2);
        join(&mut a, 1, 3);
        assert_eq!(
            send(&mut a, 4, set(100, Symbol::Skull)),
            Err(Refusal::NotMember)
        );
        assert_eq!(
            send(&mut a, 2, set(100, Symbol::Skull)),
            Err(Refusal::NotLeader)
        );
        assert_eq!(
            send(&mut a, 1, set(200, Symbol::Skull)),
            Err(Refusal::MissingTarget)
        );
        let foreign = EntityKey {
            epoch: WorldEpoch::new(2).unwrap(),
            ..key(100)
        };
        assert_eq!(
            send(
                &mut a,
                1,
                Action::Mark {
                    target: foreign,
                    symbol: Some(Symbol::Skull)
                }
            ),
            Err(Refusal::MissingTarget)
        );
        send(&mut a, 1, set(2, Symbol::Star)).unwrap();
        send(&mut a, 1, set(100, Symbol::Skull)).unwrap();
        let v = a.view(key(1)).unwrap();
        let request = Request {
            sequence: v.sequence + 1,
            revision: v.revision - 1,
            action: Action::ClearMarks,
        };
        assert_eq!(
            a.request_with_targets(key(1), request, |_| true),
            Err(Refusal::Stale)
        );
        assert_eq!(
            a.request_with_targets(key(1), request, |_| true),
            Err(Refusal::Sequence)
        );
        assert_eq!(marks(&a, 1).len(), 2);
        let before = a.view(key(1)).unwrap();
        a.retain_mark_targets(|k| k != key(100)).unwrap();
        assert_eq!(a.view(key(1)).unwrap().sequence, before.sequence);
        assert!(a.view(key(1)).unwrap().revision > before.revision);
        assert_eq!(
            marks(&a, 1),
            vec![Mark {
                target: key(2),
                symbol: Symbol::Star
            }]
        );
        send(&mut a, 1, Action::Promote(key(3))).unwrap();
        assert_eq!(marks(&a, 3), marks(&a, 1));
        assert_eq!(send(&mut a, 1, Action::ClearMarks), Err(Refusal::NotLeader));
        send(&mut a, 3, Action::Kick(key(2))).unwrap();
        assert!(marks(&a, 3).is_empty());
        send(&mut a, 3, set(100, Symbol::Skull)).unwrap();
        a.retire(key(1)).unwrap();
        assert!(marks(&a, 3).is_empty());
        a.admit(key(5)).unwrap();
        join(&mut a, 3, 5);
        assert!(marks(&a, 5).is_empty());
    }
}
