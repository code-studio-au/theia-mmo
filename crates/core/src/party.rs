//! Bounded session-local party authority. Only trusted admission supplies callers;
//! names, transport and persistence are deliberately outside this module.
use crate::{
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use std::collections::BTreeMap;

pub const MAX_ACTORS: usize = 8;
pub const MAX_MEMBERS: usize = 5;
pub const INVITATION_SECONDS: u64 = 60;
pub const REQUEST_BURST: usize = 4;
pub const REQUEST_WINDOW_SECONDS: u64 = 2;
pub const MAX_SEQUENCE_ADVANCE: u64 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Invite(EntityKey),
    Accept(u64),
    Decline(u64),
    Cancel(u64),
    Leave,
    Kick(EntityKey),
    Promote(EntityKey),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub sequence: u64,
    pub revision: u64,
    pub action: Action,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Context,
    Sequence,
    Stale,
    RateLimited,
    MissingPlayer,
    SelfTarget,
    AlreadyGrouped,
    InvitationPending,
    NotLeader,
    Full,
    MissingInvitation,
    NotMember,
    Exhausted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Party {
    pub id: u64,
    pub leader: EntityKey,
    /// Stable join order, unaffected by leadership changes.
    pub members: Vec<EntityKey>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invitation {
    pub id: u64,
    pub inviter: EntityKey,
    pub recipient: EntityKey,
    pub party: Option<u64>,
    pub expires_at: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Membership {
    pub party: Option<Party>,
    pub incoming: Option<Invitation>,
    pub outgoing: Vec<Invitation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub owner: EntityKey,
    pub revision: u64,
    pub sequence: u64,
    pub membership: Membership,
}
struct Actor {
    revision: u64,
    sequence: u64,
    requests: [Option<u64>; REQUEST_BURST],
}
pub struct Authority {
    epoch: WorldEpoch,
    rate: TickRate,
    tick: u64,
    highest_actor: u64,
    serial: u64,
    actors: BTreeMap<EntityId, Actor>,
    parties: BTreeMap<u64, Party>,
    invitations: BTreeMap<u64, Invitation>,
}
impl Authority {
    pub fn new(epoch: WorldEpoch, rate: TickRate) -> Self {
        Self {
            epoch,
            rate,
            tick: 0,
            highest_actor: 0,
            serial: 0,
            actors: BTreeMap::new(),
            parties: BTreeMap::new(),
            invitations: BTreeMap::new(),
        }
    }
    /// Trusted admission. Incarnation numbers must increase, including after
    /// retirement; a late packet can never refer to a newly admitted player.
    pub fn admit(&mut self, key: EntityKey) -> Result<(), Refusal> {
        if key.epoch != self.epoch
            || key.entity.value() <= self.highest_actor
            || self.actors.len() == MAX_ACTORS
        {
            return Err(Refusal::Context);
        }
        self.actors.insert(
            key.entity,
            Actor {
                revision: 1,
                sequence: 0,
                requests: [None; REQUEST_BURST],
            },
        );
        self.highest_actor = key.entity.value();
        Ok(())
    }
    pub fn view(&self, key: EntityKey) -> Option<View> {
        let actor = self.actor(key)?;
        Some(View {
            owner: key,
            revision: actor.revision,
            sequence: actor.sequence,
            membership: self.membership(key),
        })
    }
    /// Only the simulation clock can expire invitations. Backward clocks refuse
    /// without mutations. Repeated publication at the same tick is harmless.
    pub fn advance(&mut self, tick: u64) -> Result<(), Refusal> {
        if tick < self.tick {
            return Err(Refusal::Context);
        }
        if self.invitations.values().all(|i| i.expires_at > tick) {
            self.tick = tick;
            return Ok(());
        }
        self.reserve_change()?;
        let before = self.capture();
        self.tick = tick;
        self.invitations
            .retain(|_, invitation| invitation.expires_at > tick);
        self.publish(before);
        Ok(())
    }
    pub fn retire(&mut self, key: EntityKey) -> Result<(), Refusal> {
        if self.actor(key).is_none() {
            return Err(Refusal::Context);
        }
        self.reserve_change()?;
        let before = self.capture();
        self.depart(key);
        self.invitations
            .retain(|_, invitation| invitation.inviter != key && invitation.recipient != key);
        self.actors.remove(&key.entity);
        self.publish(before);
        Ok(())
    }
    /// Caller comes from the admitted connection, never a field in a request.
    /// Valid sequences are consumed even on stale/rate/policy refusal.
    pub fn request(&mut self, caller: EntityKey, request: Request) -> Result<(), Refusal> {
        self.reserve_change()?;
        let actor = self.actor(caller).ok_or(Refusal::Context)?;
        if request.sequence <= actor.sequence
            || request.sequence - actor.sequence > MAX_SEQUENCE_ADVANCE
        {
            return Err(Refusal::Sequence);
        }
        let actor = self.actors.get_mut(&caller.entity).unwrap();
        actor.sequence = request.sequence;
        let window = REQUEST_WINDOW_SECONDS * u64::from(self.rate.hz());
        let slot = actor
            .requests
            .iter_mut()
            .find(|at| at.is_none_or(|at| self.tick.saturating_sub(at) >= window))
            .ok_or(Refusal::RateLimited)?;
        *slot = Some(self.tick);
        if request.revision != actor.revision {
            return Err(Refusal::Stale);
        }
        let before = self.capture();
        self.apply(caller, request.action)?;
        self.publish(before);
        Ok(())
    }
    fn actor(&self, key: EntityKey) -> Option<&Actor> {
        (key.epoch == self.epoch)
            .then(|| self.actors.get(&key.entity))
            .flatten()
    }
    fn group(&self, key: EntityKey) -> Option<&Party> {
        self.parties
            .values()
            .find(|party| party.members.contains(&key))
    }
    fn membership(&self, key: EntityKey) -> Membership {
        Membership {
            party: self.group(key).cloned(),
            incoming: self
                .invitations
                .values()
                .find(|i| i.recipient == key)
                .cloned(),
            outgoing: self
                .invitations
                .values()
                .filter(|i| i.inviter == key)
                .cloned()
                .collect(),
        }
    }
    fn reserve_change(&self) -> Result<(), Refusal> {
        if self.actors.values().any(|actor| actor.revision == u64::MAX) {
            Err(Refusal::Exhausted)
        } else {
            Ok(())
        }
    }
    fn capture(&self) -> Vec<(EntityId, Membership)> {
        self.actors
            .keys()
            .map(|&entity| {
                (
                    entity,
                    self.membership(EntityKey {
                        epoch: self.epoch,
                        entity,
                    }),
                )
            })
            .collect()
    }
    fn publish(&mut self, before: Vec<(EntityId, Membership)>) {
        for (entity, old) in before {
            let key = EntityKey {
                epoch: self.epoch,
                entity,
            };
            if self.actors.contains_key(&entity) && self.membership(key) != old {
                self.actors.get_mut(&entity).unwrap().revision += 1;
            }
        }
    }
    fn apply(&mut self, caller: EntityKey, action: Action) -> Result<(), Refusal> {
        match action {
            Action::Invite(target) => self.invite(caller, target),
            Action::Accept(id) => self.accept(caller, id),
            Action::Decline(id) | Action::Cancel(id) => {
                let invitation = self
                    .invitations
                    .get(&id)
                    .ok_or(Refusal::MissingInvitation)?;
                let owner = if matches!(action, Action::Decline(_)) {
                    invitation.recipient
                } else {
                    invitation.inviter
                };
                if caller != owner {
                    return Err(Refusal::MissingInvitation);
                }
                self.invitations.remove(&id);
                Ok(())
            }
            Action::Leave => {
                if self.group(caller).is_none() {
                    return Err(Refusal::NotMember);
                }
                self.depart(caller);
                Ok(())
            }
            Action::Kick(target) | Action::Promote(target) => {
                let party = self.group(caller).ok_or(Refusal::NotMember)?;
                if party.leader != caller {
                    return Err(Refusal::NotLeader);
                }
                if target == caller {
                    return Err(Refusal::SelfTarget);
                }
                if !party.members.contains(&target) {
                    return Err(Refusal::NotMember);
                }
                if matches!(action, Action::Kick(_)) {
                    self.depart(target);
                } else {
                    let id = party.id;
                    self.parties.get_mut(&id).unwrap().leader = target;
                    self.cancel_outgoing(caller);
                }
                Ok(())
            }
        }
    }
    fn invite(&mut self, caller: EntityKey, target: EntityKey) -> Result<(), Refusal> {
        if caller == target {
            return Err(Refusal::SelfTarget);
        }
        if self.actor(target).is_none() {
            return Err(Refusal::MissingPlayer);
        }
        let party = self.group(caller);
        if party.is_some_and(|party| party.leader != caller) {
            return Err(Refusal::NotLeader);
        }
        if self.group(target).is_some() {
            return Err(Refusal::AlreadyGrouped);
        }
        if self.invitations.values().any(|i| i.recipient == target) {
            return Err(Refusal::InvitationPending);
        }
        let members = party.map_or(1, |party| party.members.len());
        let reserved = self
            .invitations
            .values()
            .filter(|i| i.inviter == caller)
            .count();
        if members + reserved >= MAX_MEMBERS {
            return Err(Refusal::Full);
        }
        let party = party.map(|party| party.id);
        let id = self.serial.checked_add(1).ok_or(Refusal::Exhausted)?;
        let expires_at = self
            .tick
            .checked_add(INVITATION_SECONDS * u64::from(self.rate.hz()))
            .ok_or(Refusal::Exhausted)?;
        self.serial = id;
        self.invitations.insert(
            id,
            Invitation {
                id,
                inviter: caller,
                recipient: target,
                party,
                expires_at,
            },
        );
        Ok(())
    }
    fn accept(&mut self, caller: EntityKey, id: u64) -> Result<(), Refusal> {
        let invitation = self
            .invitations
            .get(&id)
            .ok_or(Refusal::MissingInvitation)?
            .clone();
        if invitation.recipient != caller {
            return Err(Refusal::MissingInvitation);
        }
        if self.group(caller).is_some() {
            return Err(Refusal::AlreadyGrouped);
        }
        // Invitations are cancelled on every inviter departure/leadership change.
        let group = self.group(invitation.inviter);
        if group.map(|party| party.id) != invitation.party
            || group.is_some_and(|party| party.leader != invitation.inviter)
        {
            return Err(Refusal::MissingInvitation);
        }
        if group.is_some_and(|party| party.members.len() >= MAX_MEMBERS) {
            return Err(Refusal::Full);
        }
        let group_id = invitation.party.unwrap_or(id);
        self.cancel_outgoing(caller);
        // An inviter who becomes a leader cannot still accept another party.
        self.invitations
            .retain(|_, i| i.recipient != caller && i.recipient != invitation.inviter);
        self.parties
            .entry(group_id)
            .or_insert_with(|| Party {
                id: group_id,
                leader: invitation.inviter,
                members: vec![invitation.inviter],
            })
            .members
            .push(caller);
        for pending in self
            .invitations
            .values_mut()
            .filter(|i| i.inviter == invitation.inviter)
        {
            pending.party = Some(group_id);
        }
        Ok(())
    }
    fn cancel_outgoing(&mut self, caller: EntityKey) {
        self.invitations.retain(|_, i| i.inviter != caller);
    }
    fn depart(&mut self, caller: EntityKey) {
        let Some(id) = self.group(caller).map(|party| party.id) else {
            return;
        };
        self.cancel_outgoing(caller);
        let party = self.parties.get_mut(&id).unwrap();
        party.members.retain(|&member| member != caller);
        if party.members.len() < 2 {
            self.parties.remove(&id);
            self.invitations.retain(|_, i| i.party != Some(id));
        } else if party.leader == caller {
            party.leader = party.members[0];
        }
    }
}

#[cfg(test)]
mod tests;
