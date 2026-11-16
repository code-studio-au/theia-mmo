//! Optional World-owned membership. No names, persistence or transport in ticks.
use super::World;
use theia_core::party::{self, Refusal, Request, View};

const _: () = assert!(party::MAX_ACTORS == crate::wire::MAX_PLAYERS);

pub(super) fn profile_identity(source: [u8; 32]) -> [u8; 32] {
    crate::party_wire::profile_identity(source)
}

impl World {
    /// Configure before any admission. Normal worlds stay unchanged until the
    /// runtime explicitly opts in; named authenticated transport is required by
    /// the integration layer, not inferred from this headless adapter.
    pub fn enable_parties(&mut self) -> crate::Result<()> {
        if self.tick != 0 || self.has_admitted_players || self.parties.is_some() || self.dev_tour {
            return Err("Parties require an inactive world before first admission".into());
        }
        self.parties = Some(party::Authority::new(self.epoch, self.rate));
        Ok(())
    }
    pub fn parties_enabled(&self) -> bool {
        self.parties.is_some()
    }
    pub fn party_view(&self, actor: u64) -> Option<View> {
        self.parties.as_ref()?.view(self.entity_key(actor)?)
    }
    pub fn party_member_vitals(&self, actor: u64) -> Option<Vec<crate::party_wire::MemberVitals>> {
        let view = self.party_view(actor)?;
        Some(view.membership.party.map_or_else(Vec::new, |party| {
            party
                .members
                .into_iter()
                .map(|key| crate::party_wire::MemberVitals {
                    key,
                    vitals: self.combat_vitals(key.entity.value()),
                })
                .collect()
        }))
    }
    /// The caller ID must be resolved from the admitted transport session.
    /// Death never prevents membership management; departure is explicit.
    pub fn party_request(&mut self, actor: u64, request: Request) -> Result<(), Refusal> {
        let key = self.entity_key(actor).ok_or(Refusal::Context)?;
        self.parties
            .as_mut()
            .ok_or(Refusal::Context)?
            .request(key, request)?;
        self.retire_invalid_selections();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{party::Action, time::TickRate};

    fn send(world: &mut World, actor: u64, action: Action) -> Result<(), Refusal> {
        let view = world.party_view(actor).unwrap();
        world.party_request(
            actor,
            Request {
                sequence: view.sequence + 1,
                revision: view.revision,
                action,
            },
        )
    }

    #[test]
    fn opt_in_preserves_default_identity_and_freezes_rate_before_admission() {
        let mut world = World::default();
        let baseline = world.identity();
        assert!(!world.parties_enabled());
        let request = Request {
            sequence: 1,
            revision: 1,
            action: Action::Leave,
        };
        assert_eq!(world.party_request(1, request), Err(Refusal::Context));
        assert!(world.set_tick_rate(TickRate::Hz60));
        world.enable_parties().unwrap();
        assert_eq!(world.identity(), profile_identity(baseline));
        assert_ne!(world.identity(), baseline);
        assert!(!world.set_tick_rate(TickRate::Hz20));
        assert!(world.enable_parties().is_err());
        let actor = world.join().unwrap();
        assert_eq!(
            world.party_view(actor).unwrap().owner,
            world.entity_key(actor).unwrap()
        );
        world.leave(actor);
        assert!(world.enable_parties().is_err());
        let mut late = World::default();
        let id = late.join().unwrap();
        late.leave(id);
        assert!(late.enable_parties().is_err());
        assert_eq!(late.identity(), baseline);
    }

    #[test]
    fn real_world_clock_expires_invites_and_departure_retires_membership() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let mut world = World::at_rate(rate);
            world.enable_parties().unwrap();
            let first = world.join().unwrap();
            let second = world.join().unwrap();
            let second_key = world.entity_key(second).unwrap();
            send(&mut world, first, Action::Invite(second_key)).unwrap();
            let invitation = world
                .party_view(second)
                .unwrap()
                .membership
                .incoming
                .unwrap();
            while world.completed_tick() + 1 < invitation.expires_at {
                world.step();
            }
            assert!(
                world
                    .party_view(second)
                    .unwrap()
                    .membership
                    .incoming
                    .is_some()
            );
            world.step();
            assert_eq!(
                send(&mut world, second, Action::Accept(invitation.id)),
                Err(Refusal::MissingInvitation)
            );
            send(&mut world, first, Action::Invite(second_key)).unwrap();
            let id = world
                .party_view(second)
                .unwrap()
                .membership
                .incoming
                .unwrap()
                .id;
            send(&mut world, second, Action::Accept(id)).unwrap();
            let party = world.party_view(first).unwrap().membership.party.unwrap();
            for _ in 0..100 {
                world.step();
            }
            assert_eq!(
                world.party_view(second).unwrap().membership.party.unwrap(),
                party
            );
            world.leave(first);
            assert!(world.party_view(first).is_none());
            assert!(world.party_view(second).unwrap().membership.party.is_none());
            let new = world.join().unwrap();
            assert_ne!(new, first);
            assert_eq!(
                world.party_view(new).unwrap().membership,
                party::Membership::default()
            );
            assert_eq!(
                world.party_request(
                    first,
                    Request {
                        sequence: 3,
                        revision: 1,
                        action: Action::Leave
                    }
                ),
                Err(Refusal::Context)
            );
        }
    }
}

#[cfg(test)]
pub(super) mod targeting_tests;
