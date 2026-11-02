//! Explicit World-to-owner routing. Normal Renet/manifest/client adoption is
//! separate. Profiles bind exact content bytes and the XP policy, not a story MD.
use crate::{
    Result,
    dialogue::{Rejection, Request},
    progression_wire,
    renet_runtime::progression::{Input, JournalInput, JournalOperation, Port},
    simulation::{World, combat::Event, quest_journal::Prepared},
};
use std::sync::Arc;
use theia_core::{
    identity::EntityKey,
    quests::{catalog::Catalog, journal::PROFILE_DOMAIN},
};
#[derive(Clone, Debug)]
pub struct Profile {
    catalog: Arc<Catalog>,
    hash: [u8; 32],
    xp: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8], xp: &progression_wire::Profile) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes)?);
        let mut bound = PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        bound.extend(xp.hash());
        Ok(Self {
            catalog,
            hash: crate::terrain::sha256(&bound),
            xp: xp.hash(),
        })
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    /// Prepare only after all World content/modes are configured, before join.
    pub fn prepare(self: &Arc<Self>, world: &World) -> Result<PreparedRouter> {
        Ok(PreparedRouter {
            profile: self.clone(),
            world: world.prepare_quest_journal(self.catalog.clone())?,
        })
    }
}
pub struct PreparedRouter {
    profile: Arc<Profile>,
    world: Prepared,
}
impl PreparedRouter {
    pub fn bind(self, actor: EntityKey) -> Router {
        Router {
            profile: self.profile,
            world: self.world,
            actor,
            sequence: 0,
        }
    }
}
/// One admitted actor and one trusted-producer sequence per retained owner.
/// Queue acceptance never acknowledges persistence; observe committed owner state.
pub struct Router {
    profile: Arc<Profile>,
    world: Prepared,
    actor: EntityKey,
    sequence: u64,
}
impl Router {
    fn ready(&self, world: &World, port: &Port) -> bool {
        if port.input.max_capacity() != crate::renet_runtime::progression::CAPACITY
            || world.entity_key(self.actor.entity.value()) != Some(self.actor)
            || port.profile.hash() != self.profile.xp
            || port.quest.is_some()
            || port.lifecycle.is_some()
        {
            return false;
        }
        let state = port.state.borrow();
        !state.failed
            && !state.closed
            && state
                .journal
                .as_ref()
                .is_some_and(|j| j.catalog == self.profile.hash)
    }
    /// Reserve owner capacity before the dialogue authority changes state.
    /// Full/closed/not-initialized/mismatched owners refuse without processing it.
    pub fn interact(
        &mut self,
        world: &mut World,
        request: &Request,
        port: &Port,
    ) -> std::result::Result<bool, Rejection> {
        if !self.ready(world, port) {
            return Err(Rejection::Unavailable);
        }
        let sequence = self.sequence.checked_add(1).ok_or(Rejection::Exhausted)?;
        let permit = port
            .input
            .try_reserve()
            .map_err(|_| Rejection::Unavailable)?;
        let Some(fact) =
            world.interact_quest_journal(&self.world, self.actor.entity.value(), request)?
        else {
            return Ok(false);
        };
        permit.send(Input::Journal(JournalInput {
            actor: fact.actor,
            catalog: self.profile.hash,
            sequence,
            operation: JournalOperation::Event {
                id: fact.id,
                event: fact.event,
            },
        }));
        self.sequence = sequence;
        Ok(true)
    }
    /// All-or-none queue reservation for this actor's trusted kill batch. On
    /// refusal, caller retains the original batch for retry or fails closed;
    /// it must never acknowledge or discard earned work as committed.
    pub fn forward_kills(&mut self, world: &World, events: &[Event], port: &Port) -> Result<usize> {
        if !self.ready(world, port) {
            return Err("Quest router owner binding unavailable".into());
        }
        let facts: Vec<_> = world
            .quest_kill_occurrences(&self.world, events)?
            .into_iter()
            .filter(|fact| fact.actor == self.actor)
            .collect();
        if facts.is_empty() {
            return Ok(0);
        }
        let count = facts.len();
        let end = self
            .sequence
            .checked_add(count as u64)
            .ok_or("Quest router sequence exhausted")?;
        let permits = port
            .input
            .try_reserve_many(count)
            .map_err(|_| "Quest router owner queue unavailable")?;
        for (offset, (permit, fact)) in permits.zip(facts).enumerate() {
            permit.send(Input::Journal(JournalInput {
                actor: fact.actor,
                catalog: self.profile.hash,
                sequence: self.sequence + offset as u64 + 1,
                operation: JournalOperation::Event {
                    id: fact.id,
                    event: fact.event,
                },
            }));
        }
        self.sequence = end;
        Ok(count)
    }
}
