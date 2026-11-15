//! Bounded session-party control frames. Caller ownership is never serialized
//! in requests; the admitted transport resolves it. Roster names are server-owned.
use crate::Result;
use theia_core::{
    character_name::CharacterName,
    identity::{EntityId, EntityKey, WorldEpoch},
    party::{self, Action, Invitation, Membership, Party, Refusal, View},
};

pub const REQUEST_TAG: &[u8; 4] = b"PQ01";
pub const STATE_TAG: &[u8; 4] = b"PS01";
pub const REQUEST_BYTES: usize = 37;
pub const MAX_STATE_BYTES: usize = 80
    + 16
    + 8 * party::MAX_MEMBERS
    + 40 * party::MAX_MEMBERS
    + party::MAX_ACTORS * (9 + theia_core::character_name::MAX_BYTES);

pub fn profile_identity(source: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-session-party-v2\0".to_vec();
    bytes.extend(source);
    for value in [
        party::MAX_ACTORS as u64,
        party::MAX_MEMBERS as u64,
        party::INVITATION_SECONDS,
        party::REQUEST_BURST as u64,
        party::REQUEST_WINDOW_SECONDS,
        party::MAX_SEQUENCE_ADVANCE,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(REQUEST_TAG);
    bytes.extend(STATE_TAG);
    crate::terrain::sha256(&bytes)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub epoch: WorldEpoch,
    pub intent: party::Request,
}
impl Request {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let (kind, value) = match self.intent.action {
            Action::Invite(key) | Action::Kick(key) | Action::Promote(key) => {
                if key.epoch != self.epoch {
                    return Err("Foreign party target epoch".into());
                }
                let kind = match self.intent.action {
                    Action::Invite(_) => 1,
                    Action::Kick(_) => 6,
                    _ => 7,
                };
                (kind, key.entity.value())
            }
            Action::Accept(id) => (2, id),
            Action::Decline(id) => (3, id),
            Action::Cancel(id) => (4, id),
            Action::Leave => (5, 0),
        };
        if self.intent.sequence == 0 || self.intent.revision == 0 || (kind != 5 && value == 0) {
            return Err("Invalid party request identifiers".into());
        }
        let mut bytes = Vec::with_capacity(REQUEST_BYTES);
        bytes.extend(REQUEST_TAG);
        bytes.extend(self.epoch.value().to_le_bytes());
        bytes.extend(self.intent.sequence.to_le_bytes());
        bytes.extend(self.intent.revision.to_le_bytes());
        bytes.push(kind);
        bytes.extend(value.to_le_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES || !bytes.starts_with(REQUEST_TAG) {
            return Err("Invalid party request frame".into());
        }
        let mut reader = Reader(&bytes[4..]);
        let epoch = WorldEpoch::new(reader.u64()?).ok_or("Invalid party epoch")?;
        let sequence = reader.u64()?;
        let revision = reader.u64()?;
        let kind = reader.byte()?;
        let value = reader.u64()?;
        let action = match kind {
            1 | 6 | 7 => {
                let target = EntityKey {
                    epoch,
                    entity: EntityId::new(value).ok_or("Invalid party target")?,
                };
                match kind {
                    1 => Action::Invite(target),
                    6 => Action::Kick(target),
                    _ => Action::Promote(target),
                }
            }
            2 => Action::Accept(value),
            3 => Action::Decline(value),
            4 => Action::Cancel(value),
            5 if value == 0 => Action::Leave,
            _ => return Err("Invalid party action".into()),
        };
        let request = Self {
            epoch,
            intent: party::Request {
                sequence,
                revision,
                action,
            },
        };
        request.encode()?;
        Ok(request)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedActor {
    pub key: EntityKey,
    pub name: CharacterName,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub world: [u8; 32],
    pub tick: u64,
    pub view: View,
    /// Outcome of view.sequence; None means success (or initial sequence zero).
    pub refusal: Option<Refusal>,
    /// Bounded same-world named admissions, ordered by incarnation ID. No
    /// account IDs, credentials, positions or stats are exposed here.
    pub roster: Vec<NamedActor>,
}
impl State {
    pub fn validate(&self) -> Result<()> {
        let owner = self.view.owner;
        if self.world == [0; 32]
            || self.tick == 0
            || self.view.revision == 0
            || self.roster.is_empty()
            || self.roster.len() > party::MAX_ACTORS
            || (self.view.sequence == 0 && self.refusal.is_some())
        {
            return Err("Invalid party state header".into());
        }
        let mut prior = 0;
        for actor in &self.roster {
            if actor.key.epoch != owner.epoch || actor.key.entity.value() <= prior {
                return Err("Invalid party roster order/incarnation".into());
            }
            prior = actor.key.entity.value();
        }
        let known = |key| self.roster.iter().any(|a| a.key == key);
        if !known(owner) {
            return Err("Party roster omits owner".into());
        }
        let membership = &self.view.membership;
        if let Some(group) = &membership.party
            && (group.id == 0
                || !(2..=party::MAX_MEMBERS).contains(&group.members.len())
                || !group.members.contains(&group.leader)
                || !group.members.contains(&owner)
                || group
                    .members
                    .iter()
                    .enumerate()
                    .any(|(i, key)| !known(*key) || group.members[..i].contains(key)))
        {
            return Err("Invalid party member list".into());
        }
        let valid_invitation = |invitation: &Invitation| {
            invitation.id != 0
                && invitation.party != Some(0)
                && invitation.expires_at > self.tick
                && invitation.inviter != invitation.recipient
                && known(invitation.inviter)
                && known(invitation.recipient)
        };
        if let Some(incoming) = &membership.incoming
            && (membership.party.is_some()
                || incoming.recipient != owner
                || !valid_invitation(incoming))
        {
            return Err("Invalid incoming party invitation".into());
        }
        let members = membership.party.as_ref().map_or(1, |p| p.members.len());
        if members + membership.outgoing.len() > party::MAX_MEMBERS
            || (!membership.outgoing.is_empty()
                && membership.party.as_ref().is_some_and(|p| p.leader != owner))
        {
            return Err("Party invitations exceed leader capacity".into());
        }
        for (index, invitation) in membership.outgoing.iter().enumerate() {
            if !valid_invitation(invitation)
                || invitation.inviter != owner
                || invitation.party != membership.party.as_ref().map(|p| p.id)
                || membership
                    .party
                    .as_ref()
                    .is_some_and(|p| p.members.contains(&invitation.recipient))
                || membership
                    .incoming
                    .as_ref()
                    .is_some_and(|i| i.id == invitation.id)
                || membership.outgoing[..index]
                    .iter()
                    .any(|i| i.id == invitation.id || i.recipient == invitation.recipient)
            {
                return Err("Invalid outgoing party invitation".into());
            }
        }
        Ok(())
    }
    pub fn same_view(&self, old: &Self) -> bool {
        self.world == old.world
            && self.view == old.view
            && self.refusal == old.refusal
            && self.roster == old.roster
    }
    pub fn validate_successor(
        &self,
        owner: EntityKey,
        world: [u8; 32],
        prior: Option<&Self>,
    ) -> Result<()> {
        self.validate()?;
        if self.view.owner != owner || self.world != world {
            return Err("Party state outside admitted owner/world".into());
        }
        if let Some(old) = prior {
            old.validate()?;
            if old.world != world
                || old.view.owner != owner
                || self.tick < old.tick
                || self.view.revision < old.view.revision
                || self.view.sequence < old.view.sequence
                || (self.view.revision == old.view.revision
                    && self.view.membership != old.view.membership)
                || (self.view.sequence == old.view.sequence && self.refusal != old.refusal)
            {
                return Err("Regressing or conflicting party state".into());
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = Vec::with_capacity(MAX_STATE_BYTES);
        bytes.extend(STATE_TAG);
        bytes.extend(self.world);
        for value in [
            self.tick,
            self.view.owner.epoch.value(),
            self.view.owner.entity.value(),
            self.view.revision,
            self.view.sequence,
        ] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.push(refusal_byte(self.refusal));
        if let Some(party) = &self.view.membership.party {
            bytes.push(party.members.len() as u8);
            bytes.extend(party.id.to_le_bytes());
            bytes.extend(party.leader.entity.value().to_le_bytes());
            for member in &party.members {
                bytes.extend(member.entity.value().to_le_bytes());
            }
        } else {
            bytes.push(0);
        }
        bytes.push(u8::from(self.view.membership.incoming.is_some()));
        if let Some(invitation) = &self.view.membership.incoming {
            encode_invitation(&mut bytes, invitation);
        }
        bytes.push(self.view.membership.outgoing.len() as u8);
        for invitation in &self.view.membership.outgoing {
            encode_invitation(&mut bytes, invitation);
        }
        bytes.push(self.roster.len() as u8);
        for actor in &self.roster {
            bytes.extend(actor.key.entity.value().to_le_bytes());
            bytes.push(actor.name.as_str().len() as u8);
            bytes.extend(actor.name.as_str().as_bytes());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_STATE_BYTES || !bytes.starts_with(STATE_TAG) {
            return Err("Invalid party state frame".into());
        }
        let mut r = Reader(&bytes[4..]);
        let world = r.take(32)?.try_into()?;
        let tick = r.u64()?;
        let epoch = WorldEpoch::new(r.u64()?).ok_or("Invalid party epoch")?;
        let owner = r.key(epoch)?;
        let revision = r.u64()?;
        let sequence = r.u64()?;
        let refusal = decode_refusal(r.byte()?)?;
        let count = usize::from(r.byte()?);
        let party = if count == 0 {
            None
        } else {
            if !(2..=party::MAX_MEMBERS).contains(&count) {
                return Err("Invalid party member count".into());
            }
            let id = r.u64()?;
            let leader = r.key(epoch)?;
            let mut members = Vec::with_capacity(count);
            for _ in 0..count {
                members.push(r.key(epoch)?);
            }
            Some(Party {
                id,
                leader,
                members,
            })
        };
        let incoming = match r.byte()? {
            0 => None,
            1 => Some(r.invitation(epoch)?),
            _ => return Err("Invalid incoming party flag".into()),
        };
        let count = usize::from(r.byte()?);
        if count >= party::MAX_MEMBERS {
            return Err("Invalid party invitation count".into());
        }
        let mut outgoing = Vec::with_capacity(count);
        for _ in 0..count {
            outgoing.push(r.invitation(epoch)?);
        }
        let count = usize::from(r.byte()?);
        if count == 0 || count > party::MAX_ACTORS {
            return Err("Invalid party roster count".into());
        }
        let mut roster = Vec::with_capacity(count);
        for _ in 0..count {
            let key = r.key(epoch)?;
            let length = usize::from(r.byte()?);
            if length > theia_core::character_name::MAX_BYTES {
                return Err("Party name exceeds limit".into());
            }
            let name = CharacterName::parse(std::str::from_utf8(r.take(length)?)?)?;
            roster.push(NamedActor { key, name });
        }
        if !r.0.is_empty() {
            return Err("Trailing party state bytes".into());
        }
        let state = Self {
            world,
            tick,
            view: View {
                owner,
                revision,
                sequence,
                membership: Membership {
                    party,
                    incoming,
                    outgoing,
                },
            },
            refusal,
            roster,
        };
        state.validate()?;
        Ok(state)
    }
}
fn encode_invitation(bytes: &mut Vec<u8>, invitation: &Invitation) {
    for value in [
        invitation.id,
        invitation.inviter.entity.value(),
        invitation.recipient.entity.value(),
        invitation.party.unwrap_or(0),
        invitation.expires_at,
    ] {
        bytes.extend(value.to_le_bytes());
    }
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let (value, rest) = self
            .0
            .split_at_checked(length)
            .ok_or("Truncated party frame")?;
        self.0 = rest;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into()?))
    }
    fn key(&mut self, epoch: WorldEpoch) -> Result<EntityKey> {
        Ok(EntityKey {
            epoch,
            entity: EntityId::new(self.u64()?).ok_or("Invalid party entity")?,
        })
    }
    fn invitation(&mut self, epoch: WorldEpoch) -> Result<Invitation> {
        let id = self.u64()?;
        let inviter = self.key(epoch)?;
        let recipient = self.key(epoch)?;
        let group = self.u64()?;
        let expires_at = self.u64()?;
        Ok(Invitation {
            id,
            inviter,
            recipient,
            party: (group != 0).then_some(group),
            expires_at,
        })
    }
}
fn refusal_byte(refusal: Option<Refusal>) -> u8 {
    match refusal {
        None => 0,
        Some(Refusal::Context) => 1,
        Some(Refusal::Sequence) => 2,
        Some(Refusal::Stale) => 3,
        Some(Refusal::RateLimited) => 4,
        Some(Refusal::MissingPlayer) => 5,
        Some(Refusal::SelfTarget) => 6,
        Some(Refusal::AlreadyGrouped) => 7,
        Some(Refusal::InvitationPending) => 8,
        Some(Refusal::NotLeader) => 9,
        Some(Refusal::Full) => 10,
        Some(Refusal::MissingInvitation) => 11,
        Some(Refusal::NotMember) => 12,
        Some(Refusal::Exhausted) => 13,
    }
}
fn decode_refusal(value: u8) -> Result<Option<Refusal>> {
    Ok(match value {
        0 => None,
        1 => Some(Refusal::Context),
        2 => Some(Refusal::Sequence),
        3 => Some(Refusal::Stale),
        4 => Some(Refusal::RateLimited),
        5 => Some(Refusal::MissingPlayer),
        6 => Some(Refusal::SelfTarget),
        7 => Some(Refusal::AlreadyGrouped),
        8 => Some(Refusal::InvitationPending),
        9 => Some(Refusal::NotLeader),
        10 => Some(Refusal::Full),
        11 => Some(Refusal::MissingInvitation),
        12 => Some(Refusal::NotMember),
        13 => Some(Refusal::Exhausted),
        _ => return Err("Invalid party refusal".into()),
    })
}

#[cfg(test)]
mod tests;
