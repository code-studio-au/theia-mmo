//! Stationary, noncombat informational guide. Pure fixed-step state only;
//! no database, file or transport I/O, and no physics body for the NPC.
use super::World;
use crate::dialogue::{
    Action, Guide, Open, Placement, Profile, Rejection, Request, View, in_range,
};
use std::{collections::BTreeMap, sync::Arc};
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey},
};

pub(super) struct Authority {
    pub(super) profile: Arc<Profile>,
    contacts: BTreeMap<ContentId, Contact>,
    actors: BTreeMap<u64, Session>,
}
struct Contact {
    placement: Placement,
    guide: Guide,
    available: bool,
}
#[derive(Default)]
struct Session {
    sequence: u64,
    revision: u64,
    open: Option<Open>,
    window: u64,
    requests: u8,
}
impl Session {
    fn replace(&mut self, open: Option<Open>) -> Result<(), Rejection> {
        let revision = self.revision.checked_add(1).ok_or(Rejection::Exhausted)?;
        // Reserve the final revision for closure so exhaustion cannot change an
        // open view into a closed view at the same revision.
        if open.is_some() && revision == u64::MAX {
            return Err(Rejection::Exhausted);
        }
        self.revision = revision;
        self.open = open;
        Ok(())
    }
    fn close(&mut self) {
        if self.open.is_some() {
            self.replace(None)
                .expect("Open session reserves final closure revision");
        }
    }
}
impl Authority {
    pub(super) fn home(&self, npc: &ContentId) -> Option<Guide> {
        self.contacts
            .get(npc)
            .filter(|c| c.available)
            .map(|c| c.guide)
    }
    pub(super) fn identity(&self, mut base: [u8; 32]) -> [u8; 32] {
        // Sorted content IDs make allocation and identity independent of input order.
        for contact in self.contacts.values() {
            base = contact.placement.identity(base, self.profile.hash());
        }
        base
    }
    pub(super) fn available(&self) -> bool {
        self.contacts.values().all(|c| c.available)
    }
    pub(super) fn leave(&mut self, actor: u64) {
        self.actors.remove(&actor);
    }
    fn contact(&self, key: EntityKey) -> Option<(&ContentId, &Contact)> {
        self.contacts.iter().find(|(_, c)| c.guide.key == key)
    }
    fn eligible(&self, world: &World, actor: u64, key: EntityKey) -> bool {
        self.contact(key).is_some_and(|(npc, c)| {
            c.available
                && world.player_alive(actor)
                && world.global_position(actor).is_some_and(|p| {
                    in_range(p, world.follow.position(actor, npc, c.guide.position))
                })
        })
    }
    fn prune(&mut self, world: &World) {
        let contacts = &self.contacts;
        self.actors.retain(|&id, state| {
            let Some(position) = world.global_position(id) else {
                return false;
            };
            if state.open.as_ref().is_some_and(|open| {
                !world.player_alive(id)
                    || !contacts.iter().any(|(npc, c)| {
                        c.available
                            && c.guide.key == open.guide
                            && in_range(position, world.follow.position(id, npc, c.guide.position))
                    })
            }) {
                state.close();
            }
            true
        });
    }
    fn request(&mut self, world: &World, actor: u64, request: &Request) -> Result<(), Rejection> {
        if world.entity_key(actor).is_none() {
            return Err(Rejection::Unavailable);
        }
        let key = match request.action {
            Action::Open(key) => Some(key),
            _ => self
                .actors
                .get(&actor)
                .and_then(|s| s.open.as_ref())
                .map(|o| o.guide),
        };
        let eligible = key.is_some_and(|key| self.eligible(world, actor, key));
        let contact = key
            .and_then(|key| self.contact(key))
            .map(|(id, _)| id.clone());
        let state = self.actors.entry(actor).or_default();
        let window = world.tick / u64::from(world.rate.hz());
        if window != state.window {
            state.window = window;
            state.requests = 0;
        }
        if state.requests >= crate::dialogue::REQUESTS_PER_SECOND {
            return Err(Rejection::RateLimited);
        }
        state.requests += 1;
        if request.sequence == 0 || request.sequence <= state.sequence {
            return Err(Rejection::StaleSequence);
        }
        state.sequence = request.sequence;
        if contact.is_none() {
            return Err(Rejection::InvalidAction);
        }
        if !eligible {
            state.close();
            return Err(Rejection::OutOfRange);
        }
        if request.expected_revision != state.revision {
            return Err(Rejection::StaleRevision);
        }
        let open = match &request.action {
            Action::Open(key) if state.open.is_none() => Some(Open {
                guide: *key,
                node: self
                    .profile
                    .contact(contact.as_ref().ok_or(Rejection::InvalidAction)?)
                    .ok_or(Rejection::InvalidAction)?
                    .start()
                    .id()
                    .clone(),
            }),
            Action::Choose(choice) => {
                let open = state.open.as_ref().ok_or(Rejection::InvalidAction)?;
                let node = self
                    .profile
                    .contact(contact.as_ref().ok_or(Rejection::InvalidAction)?)
                    .ok_or(Rejection::InvalidAction)?
                    .choose(&open.node, choice)
                    .map_err(|_| Rejection::InvalidAction)?;
                Some(Open {
                    guide: open.guide,
                    node: node.id().clone(),
                })
            }
            Action::Close if state.open.is_some() => None,
            _ => return Err(Rejection::InvalidAction),
        };
        state.replace(open)
    }
}
impl World {
    pub fn enable_vendor(
        &mut self,
        profile: Arc<crate::vendor_wire::Profile>,
    ) -> crate::Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.vendor.is_some()
            || self
                .dialogue
                .as_ref()
                .is_none_or(|d| d.profile.contact(profile.catalog().npc()).is_none())
        {
            return Err("Vendor requires inactive world and matching guide".into());
        }
        self.vendor = Some(profile);
        Ok(())
    }
    pub fn vendor_profile(&self) -> Option<&Arc<crate::vendor_wire::Profile>> {
        self.vendor.as_ref()
    }
    /// Called before accepting an item intent. Completion does not repeat this
    /// check after close, walkaway or logout.
    pub fn vendor_eligible(&self, actor: u64, command: theia_core::vendors::Command) -> bool {
        self.vendor
            .as_ref()
            .is_some_and(|p| p.purchase(command).is_ok())
            && self
                .vendor
                .as_ref()
                .is_some_and(|p| self.contact_definition(command.npc) == Some(p.catalog().npc()))
            && self.guide_interaction_eligible(actor, command.dialogue_revision, command.npc)
    }

    /// Shared current own-session/incarnation/3D-range entitlement. This does not
    /// commit an item or quest operation and is not rechecked after acceptance.
    pub fn guide_interaction_eligible(
        &self,
        actor: u64,
        dialogue_revision: u64,
        npc: EntityKey,
    ) -> bool {
        self.dialogue
            .as_ref()
            .is_some_and(|d| d.eligible(self, actor, npc))
            && self.dialogue_view(actor).is_some_and(|v| {
                v.revision == dialogue_revision && v.open.is_some_and(|o| o.guide == npc)
            })
    }

    /// Trusted fixture placement before admission. A guide is not a combat target
    /// or fake player and constructs no collision object.
    pub fn spawn_development_guide(
        &mut self,
        profile: Arc<Profile>,
        xz: [f64; 2],
    ) -> crate::Result<Guide> {
        self.spawn_placed_guide(profile, Placement::Horizontal(xz))
    }
    /// Install one trusted, supported contact before admitting players. This
    /// retains the existing single-guide budget and creates no actor collision.
    pub fn spawn_placed_guide(
        &mut self,
        profile: Arc<Profile>,
        placement: Placement,
    ) -> crate::Result<Guide> {
        if profile.catalog().is_some() {
            return Err("Contact catalogues require explicit collection placement".into());
        }
        let npc = profile.profile().npc().clone();
        Ok(self.install_contacts(profile, vec![(npc, placement)])?[0].guide)
    }
    pub fn spawn_placed_contacts(
        &mut self,
        profile: Arc<Profile>,
        placements: Vec<(ContentId, Placement)>,
    ) -> crate::Result<Vec<crate::dialogue::ContactPresence>> {
        if profile.catalog().is_none() {
            return Err("Collection placement requires a contact catalogue".into());
        }
        self.install_contacts(profile, placements)
    }
    fn install_contacts(
        &mut self,
        profile: Arc<Profile>,
        placements: Vec<(ContentId, Placement)>,
    ) -> crate::Result<Vec<crate::dialogue::ContactPresence>> {
        if self.tick != 0 || !self.players.is_empty() || self.dialogue.is_some() {
            return Err("Contacts require an inactive world and unused contact budget".into());
        }
        if placements.len() != profile.contacts().count() {
            return Err("Contact placements must match every catalogue entry".into());
        }
        let mut sorted = BTreeMap::new();
        for (npc, placement) in placements {
            if profile.contact(&npc).is_none() || sorted.insert(npc, placement).is_some() {
                return Err("Unknown or duplicate contact placement".into());
            }
        }
        let runtime = self
            .procedural
            .as_ref()
            .ok_or("Contacts require prepared world")?;
        let mut next = self.next_id;
        let mut contacts = BTreeMap::new();
        for (npc, placement) in sorted {
            let focus = placement.focus()?;
            let patch = runtime
                .patch(focus)
                .ok_or("Contact collision not prepared")?;
            let mut position = match placement {
                Placement::Horizontal([x, z]) => patch.try_spawn(x, z),
                Placement::Feet(feet) => patch.try_spawn_near(feet),
            }
            .ok_or("Contact unsupported at requested position")?
            .metres();
            position[1] += f64::from(crate::world::PAWN_Y);
            let position =
                GlobalPosition::new(position).ok_or("Invalid supported contact position")?;
            next = next.checked_add(1).ok_or("Entity identity exhausted")?;
            contacts.insert(
                npc,
                Contact {
                    placement,
                    available: true,
                    guide: Guide {
                        key: EntityKey {
                            epoch: self.epoch,
                            entity: EntityId::new(next).unwrap(),
                        },
                        position,
                    },
                },
            );
        }
        let presence = contacts
            .iter()
            .map(|(npc, c)| crate::dialogue::ContactPresence {
                npc: npc.clone(),
                guide: c.guide,
            })
            .collect();
        // Commit only after every support/reference check succeeds.
        self.next_id = next;
        self.dialogue = Some(Authority {
            profile,
            contacts,
            actors: BTreeMap::new(),
        });
        Ok(presence)
    }
    pub fn dialogue_profile(&self) -> Option<&Arc<Profile>> {
        Some(&self.dialogue.as_ref()?.profile)
    }
    pub fn contact_definition(&self, key: EntityKey) -> Option<&ContentId> {
        let (id, contact) = self.dialogue.as_ref()?.contact(key)?;
        contact.available.then_some(id)
    }
    pub fn nearby_guide(&self, actor: u64) -> Option<Guide> {
        let authority = self.dialogue.as_ref()?;
        // Legacy convenience only. Commands always validate their exact entity.
        authority
            .contacts
            .iter()
            .find(|(_, c)| authority.eligible(self, actor, c.guide.key))
            .map(|(npc, c)| Guide {
                position: self.follow.position(actor, npc, c.guide.position),
                ..c.guide
            })
    }
    pub fn visible_contacts(&self, actor: u64) -> Vec<crate::dialogue::ContactPresence> {
        let Some(authority) = &self.dialogue else {
            return vec![];
        };
        let Some(position) = self.global_position(actor) else {
            return vec![];
        };
        authority
            .contacts
            .iter()
            .filter(|(npc, c)| {
                c.available
                    && crate::targeting::relevant(
                        position,
                        self.follow.position(actor, npc, c.guide.position),
                    )
            })
            .map(|(npc, c)| crate::dialogue::ContactPresence {
                npc: npc.clone(),
                guide: Guide {
                    position: self.follow.position(actor, npc, c.guide.position),
                    ..c.guide
                },
            })
            .collect()
    }
    pub fn visible_guide(&self, actor: u64) -> Option<Guide> {
        let authority = self.dialogue.as_ref()?;
        if authority.profile.catalog().is_some() {
            return None;
        }
        self.visible_contacts(actor).first().map(|c| c.guide)
    }
    /// Actor is supplied by the admitted owner, never decoded from a request.
    /// This library boundary is not yet an adopted network endpoint.
    pub fn interact(&mut self, actor: u64, request: &Request) -> Result<(), Rejection> {
        let mut authority = self.dialogue.take().ok_or(Rejection::Unavailable)?;
        let result = authority.request(self, actor, request);
        self.dialogue = Some(authority);
        result
    }
    pub fn dialogue_view(&self, actor: u64) -> Option<View> {
        let authority = self.dialogue.as_ref()?;
        let actor_key = self.entity_key(actor)?;
        let state = authority.actors.get(&actor);
        Some(View {
            actor: actor_key,
            profile_hash: authority.profile.hash(),
            revision: state.map_or(0, |s| s.revision),
            open: state.and_then(|s| s.open.clone()),
        })
    }
    /// Retain the bound content/placement identity; retiring this incarnation
    /// does not silently change a live transport's world profile.
    pub fn retire_development_guide(&mut self, key: EntityKey) -> bool {
        let Some(authority) = &mut self.dialogue else {
            return false;
        };
        let Some(contact) = authority
            .contacts
            .values_mut()
            .find(|c| c.guide.key == key && c.available)
        else {
            return false;
        };
        contact.available = false;
        for state in authority.actors.values_mut() {
            if state.open.as_ref().is_some_and(|o| o.guide == key) {
                state.close();
            }
        }
        true
    }
    pub(super) fn advance_dialogue(&mut self) {
        if let Some(mut authority) = self.dialogue.take() {
            authority.prune(self);
            self.dialogue = Some(authority);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{content::ContentId, identity::WorldEpoch, time::TickRate};
    const DATA: &[u8] = include_bytes!("../../../../content/gameplay/development-dialogue.json");
    fn profile() -> Arc<Profile> {
        Arc::new(Profile::from_json(DATA).unwrap())
    }
    fn setup() -> (World, Guide, u64) {
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        let guide = world
            .spawn_development_guide(profile(), [8062., 8000.])
            .unwrap();
        let actor = world.join().unwrap();
        world.players.get_mut(&actor).unwrap().position = guide.position;
        (world, guide, actor)
    }
    fn request(sequence: u64, revision: u64, action: Action) -> Request {
        Request {
            sequence,
            expected_revision: revision,
            action,
        }
    }
    #[test]
    fn purchases_require_current_own_session_guide_range_and_enabled_offer() {
        use theia_core::vendors::Command;
        let (mut world, guide, actor) = setup();
        let items = crate::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let vendor = Arc::new(
            crate::vendor_wire::Profile::from_json(
                include_bytes!("../../../../content/gameplay/development-vendor.json"),
                &items,
            )
            .unwrap(),
        );
        let command = Command {
            expected_revision: 0,
            dialogue_revision: 1,
            npc: guide.key,
            offer: 0,
        };
        assert!(!world.vendor_eligible(actor, command));
        // Configuration is restricted to before player admission.
        assert!(world.enable_vendor(vendor.clone()).is_err());
        world.vendor = Some(vendor.clone());
        let identity = world.identity();
        assert!(!world.vendor_eligible(actor, command));
        world
            .interact(actor, &request(1, 0, Action::Open(guide.key)))
            .unwrap();
        assert!(world.vendor_eligible(actor, command));
        let b = world.join().unwrap();
        world.players.get_mut(&b).unwrap().position = guide.position;
        assert!(!world.vendor_eligible(b, command));
        assert!(!world.vendor_eligible(
            actor,
            Command {
                dialogue_revision: 2,
                ..command
            }
        ));
        assert!(!world.vendor_eligible(
            actor,
            Command {
                offer: 1,
                ..command
            }
        ));
        assert!(!world.vendor_eligible(
            actor,
            Command {
                npc: world.entity_key(actor).unwrap(),
                ..command
            }
        ));
        let p = guide.position.metres();
        world.players.get_mut(&actor).unwrap().position =
            GlobalPosition::new([p[0], p[1] + 3.0001, p[2]]).unwrap();
        assert!(
            !world.vendor_eligible(actor, command),
            "Range rechecks the current position even before a pruning step"
        );
        world.players.get_mut(&actor).unwrap().position = guide.position;
        world
            .interact(actor, &request(2, 1, Action::Close))
            .unwrap();
        assert!(!world.vendor_eligible(actor, command));
        world
            .interact(actor, &request(3, 2, Action::Open(guide.key)))
            .unwrap();
        assert!(!world.vendor_eligible(actor, command));
        assert!(world.vendor_eligible(
            actor,
            Command {
                dialogue_revision: 3,
                ..command
            }
        ));
        world.retire_development_guide(guide.key);
        assert!(!world.vendor_eligible(
            actor,
            Command {
                dialogue_revision: 3,
                ..command
            }
        ));
        assert_eq!(
            world.identity(),
            identity,
            "Retiring NPC keeps the adopted vendor world identity"
        );
        let mut inactive = World::with_procedural([8060., 8000.], 1.).unwrap();
        assert!(inactive.enable_vendor(vendor.clone()).is_err());
        inactive
            .spawn_development_guide(profile(), [8062., 8000.])
            .unwrap();
        let base = inactive.identity();
        inactive.enable_vendor(vendor.clone()).unwrap();
        assert_eq!(
            inactive.identity(),
            crate::vendor_wire::profile_identity(base, vendor.hash())
        );
        assert!(inactive.enable_vendor(vendor).is_err());
    }
    #[test]
    fn guide_identity_binds_exact_content_and_placement_and_rejects_live_or_unsupported_setup() {
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        let base = world.identity();
        for xz in [
            [f64::NAN, 8000.],
            [f64::INFINITY, 8000.],
            [-1., 8000.],
            [16001., 8000.],
            [100., 100.],
        ] {
            assert!(world.spawn_development_guide(profile(), xz).is_err());
            assert_eq!(world.identity(), base);
        }
        let guide = world
            .spawn_development_guide(profile(), [8062., 8000.])
            .unwrap();
        let identity = world.identity();
        assert_ne!(identity, base);
        assert_eq!(
            identity,
            crate::dialogue::profile_identity(base, profile().hash(), [8062., 8000.])
        );
        assert!(
            world
                .spawn_development_guide(profile(), [8064., 8000.])
                .is_err()
        );
        let actor = world.join().unwrap();
        assert!(
            world
                .spawn_development_guide(profile(), [8064., 8000.])
                .is_err()
        );
        assert_eq!(world.dialogue_view(actor).unwrap().revision, 0);
        assert!(world.retire_development_guide(guide.key));
        assert_eq!(
            world.identity(),
            identity,
            "Retirement keeps the adopted profile identity"
        );
        assert!(!world.retire_development_guide(guide.key));
        let mut padded = DATA.to_vec();
        padded.push(b' ');
        assert_ne!(
            Profile::from_json(&padded).unwrap().hash(),
            profile().hash()
        );
        assert!(
            World::default()
                .spawn_development_guide(profile(), [0., 0.])
                .is_err()
        );
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        world.step();
        assert!(
            world
                .spawn_development_guide(profile(), [8062., 8000.])
                .is_err()
        );
    }
    #[test]
    fn exact_three_dimensional_range_and_current_guide_identity_are_authoritative() {
        let (mut world, guide, actor) = setup();
        let p = guide.position.metres();
        for offset in [[3., 0., 0.], [0., 3., 0.], [0., 0., -3.], [1., 2., 2.]] {
            world.players.get_mut(&actor).unwrap().position =
                GlobalPosition::new(std::array::from_fn(|i| p[i] + offset[i])).unwrap();
            assert_eq!(world.nearby_guide(actor), Some(guide));
        }
        for offset in [[3.00001, 0., 0.], [0., 3.00001, 0.], [3., 0., 3.]] {
            world.players.get_mut(&actor).unwrap().position =
                GlobalPosition::new(std::array::from_fn(|i| p[i] + offset[i])).unwrap();
            assert_eq!(world.nearby_guide(actor), None);
            assert_eq!(world.visible_guide(actor), Some(guide));
        }
        assert_eq!(
            world.interact(actor, &request(1, 0, Action::Open(guide.key))),
            Err(Rejection::OutOfRange)
        );
        world.players.get_mut(&actor).unwrap().position = guide.position;
        let foreign = EntityKey {
            epoch: WorldEpoch::new(if world.epoch() == 1 { 2 } else { 1 }).unwrap(),
            ..guide.key
        };
        assert_eq!(
            world.interact(actor, &request(2, 0, Action::Open(foreign))),
            Err(Rejection::InvalidAction)
        );
        assert_eq!(
            world.interact(
                actor,
                &request(3, 0, Action::Open(world.entity_key(actor).unwrap()))
            ),
            Err(Rejection::InvalidAction)
        );
        assert_eq!(
            world.interact(u64::MAX, &request(1, 0, Action::Open(guide.key))),
            Err(Rejection::Unavailable)
        );
        assert_eq!(world.dialogue.as_ref().unwrap().actors.len(), 1);
        assert_eq!(
            world.interact(actor, &request(4, 0, Action::Open(guide.key))),
            Ok(())
        );
    }
    #[test]
    fn player_sessions_are_independent_and_stale_or_unoffered_choices_cannot_advance_them() {
        let (mut world, guide, a) = setup();
        let b = world.join().unwrap();
        world.players.get_mut(&b).unwrap().position = guide.position;
        let open = request(1, 0, Action::Open(guide.key));
        world.interact(a, &open).unwrap();
        let initial = world.dialogue_view(a).unwrap();
        assert_eq!(world.dialogue_view(b).unwrap().open, None);
        assert_eq!(world.interact(a, &open), Err(Rejection::StaleSequence));
        assert_eq!(
            world.interact(
                a,
                &request(
                    2,
                    0,
                    Action::Choose(ContentId::new("theia.dev.guide.ask-movement").unwrap())
                )
            ),
            Err(Rejection::StaleRevision)
        );
        assert_eq!(
            world.interact(
                a,
                &request(
                    3,
                    1,
                    Action::Choose(ContentId::new("theia.dev.guide.finish-training").unwrap())
                )
            ),
            Err(Rejection::InvalidAction)
        );
        assert_eq!(world.dialogue_view(a).unwrap(), initial);
        world
            .interact(
                a,
                &request(
                    4,
                    1,
                    Action::Choose(ContentId::new("theia.dev.guide.ask-movement").unwrap()),
                ),
            )
            .unwrap();
        world.interact(b, &open).unwrap();
        assert_eq!(
            world.dialogue_view(a).unwrap().open.unwrap().node.as_str(),
            "theia.dev.guide.movement"
        );
        assert_eq!(
            world.dialogue_view(b).unwrap().open.unwrap().node.as_str(),
            "theia.dev.guide.welcome"
        );
        assert_ne!(
            world.dialogue_view(a).unwrap().actor,
            world.dialogue_view(b).unwrap().actor
        );
        world.interact(a, &request(5, 2, Action::Close)).unwrap();
        assert_eq!(world.dialogue_view(a).unwrap().revision, 3);
        assert!(world.dialogue_view(a).unwrap().open.is_none());
        assert!(world.dialogue_view(b).unwrap().open.is_some());
        assert_eq!(
            world.interact(a, &request(6, 2, Action::Open(guide.key))),
            Err(Rejection::StaleRevision)
        );
        world
            .interact(a, &request(7, 3, Action::Open(guide.key)))
            .unwrap();
        assert_eq!(world.dialogue_view(a).unwrap().revision, 4);
    }
    #[test]
    fn walkaway_retirement_and_disconnect_close_without_reusing_actor_sessions() {
        let (mut world, guide, a) = setup();
        world
            .interact(a, &request(1, 0, Action::Open(guide.key)))
            .unwrap();
        let p = guide.position.metres();
        world.players.get_mut(&a).unwrap().position =
            GlobalPosition::new([p[0] + 5., p[1], p[2]]).unwrap();
        world.step();
        assert_eq!(world.dialogue_view(a).unwrap().revision, 2);
        assert!(world.dialogue_view(a).unwrap().open.is_none());
        world.players.get_mut(&a).unwrap().position = guide.position;
        world
            .interact(a, &request(2, 2, Action::Open(guide.key)))
            .unwrap();
        assert!(world.retire_development_guide(guide.key));
        assert_eq!(world.dialogue_view(a).unwrap().revision, 4);
        assert!(world.dialogue_view(a).unwrap().open.is_none());
        world.leave(a);
        assert_eq!(world.dialogue_view(a), None);
        assert!(world.dialogue.as_ref().unwrap().actors.is_empty());
        let b = world.join().unwrap();
        assert_ne!(a, b);
        assert_eq!(world.dialogue_view(b).unwrap().revision, 0);
        assert!(world.dialogue_view(b).unwrap().open.is_none());
    }
    #[test]
    fn request_budgets_follow_simulation_rate_and_revision_exhaustion_reserves_closure() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (mut world, guide, a) = setup();
            // Configuration is checked before time advances; no player clock input exists.
            world.rate = rate;
            for sequence in 1..=u64::from(crate::dialogue::REQUESTS_PER_SECOND) {
                let expected = if sequence == 1 {
                    Ok(())
                } else {
                    Err(Rejection::InvalidAction)
                };
                assert_eq!(
                    world.interact(
                        a,
                        &request(sequence, 1.min(sequence - 1), Action::Open(guide.key))
                    ),
                    expected
                );
            }
            assert_eq!(
                world.interact(a, &request(9, 1, Action::Close)),
                Err(Rejection::RateLimited)
            );
            for _ in 0..rate.hz() {
                world.step();
            }
            world.interact(a, &request(9, 1, Action::Close)).unwrap();
            assert!(world.dialogue_view(a).unwrap().open.is_none());
        }
        let (mut world, guide, a) = setup();
        world
            .interact(a, &request(1, 0, Action::Open(guide.key)))
            .unwrap();
        world
            .dialogue
            .as_mut()
            .unwrap()
            .actors
            .get_mut(&a)
            .unwrap()
            .revision = u64::MAX - 1;
        assert_eq!(
            world.interact(
                a,
                &request(
                    2,
                    u64::MAX - 1,
                    Action::Choose(ContentId::new("theia.dev.guide.ask-movement").unwrap())
                )
            ),
            Err(Rejection::Exhausted)
        );
        assert_eq!(world.dialogue_view(a).unwrap().revision, u64::MAX - 1);
        world
            .interact(a, &request(3, u64::MAX - 1, Action::Close))
            .unwrap();
        assert_eq!(world.dialogue_view(a).unwrap().revision, u64::MAX);
        assert_eq!(
            world.interact(a, &request(4, u64::MAX, Action::Open(guide.key))),
            Err(Rejection::Exhausted)
        );
        assert!(world.dialogue_view(a).unwrap().open.is_none());
    }
    #[test]
    fn guide_is_not_a_player_or_attack_target_and_does_not_block_actual_movement() {
        let mut combat = World::with_procedural([8060., 8000.], 1.).unwrap();
        let target = combat.spawn_development_target([8068., 8000.]).unwrap();
        combat
            .enable_combat(
                include_bytes!("../../../../content/gameplay/development-abilities.json"),
                100,
                5000,
            )
            .unwrap();
        let guide = combat
            .spawn_development_guide(profile(), [8062., 8000.])
            .unwrap();
        let actor = combat.join().unwrap();
        let snapshot = combat.combat_snapshot(actor).unwrap();
        assert_eq!(snapshot.targets.len(), 1);
        assert_eq!(snapshot.targets[0].key, target);
        assert!(!combat.select_target(actor, Some(guide.key)));
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
            assert!(world.set_tick_rate(rate));
            let guide = world
                .spawn_development_guide(profile(), [8064., 8000.])
                .unwrap();
            let a = world.join().unwrap();
            let initial = world.global_position(a).unwrap().metres();
            assert_eq!(world.targeting_snapshot(a).unwrap().targets.len(), 0);
            assert!(!world.select_target(a, Some(guide.key)));
            let chunks = world
                .procedural()
                .unwrap()
                .patch(world.global_position(a).unwrap())
                .unwrap()
                .ids
                .iter()
                .map(|id| [id.x, id.z])
                .collect();
            assert!(world.client_ready(
                a,
                &crate::wire::ChunkReady {
                    version: crate::wire::VERSION,
                    epoch: world.epoch(),
                    sequence: 1,
                    chunks
                }
            ));
            for sequence in 1..=u64::from(rate.hz()) {
                assert!(world.input(
                    a,
                    crate::wire::Input {
                        version: crate::wire::VERSION,
                        epoch: world.epoch(),
                        sequence,
                        intent: crate::wire::Intent {
                            facing: None,
                            axes: [1000, 0],
                            sprint: true,
                            jump: false,
                            dodge: None,
                        }
                    }
                ));
                assert_eq!(world.step().players.len(), 1);
            }
            assert!(initial[0] < 8064. && world.global_position(a).unwrap().metres()[0] > 8066.);
            assert_eq!(world.movement_recoveries(), 0);
        }
    }
}

#[cfg(test)]
mod multi_contact_tests;
