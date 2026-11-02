//! Private explicit quest journal/actions. Decoding establishes no NPC entitlement,
//! kill credit, durable transition or reward; those belong to the authority owner.
use crate::Result;
use std::sync::Arc;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::{
        self, Profile as CoreProfile,
        lifecycle::{self as life, Action, Command, Status},
    },
};
pub const STATE_TAG: &[u8; 4] = b"QL01";
pub const REQUEST_TAG: &[u8; 4] = b"QA01";
pub const REPLY_TAG: &[u8; 4] = b"QR01";
pub const STATE_BYTES: usize = 65;
pub const REQUEST_BYTES: usize = 61;
pub const REPLY_BYTES: usize = 69;
#[derive(Clone, Debug)]
pub struct Profile {
    profile: Arc<CoreProfile>,
    hash: [u8; 32],
    completion: Option<(Arc<quests::turn_in_xp::Recipe>, [u8; 32])>,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let profile = Arc::new(CoreProfile::from_json(bytes)?);
        let mut bound = life::PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        Ok(Self {
            profile,
            hash: crate::terrain::sha256(&bound),
            completion: None,
        })
    }
    pub fn from_json_with_turn_in_xp(
        quest: &[u8],
        recipe: &[u8],
        xp: &crate::progression_wire::Profile,
    ) -> Result<Self> {
        let mut profile = Self::from_json(quest)?;
        let checked =
            quests::turn_in_xp::Recipe::from_json(recipe, profile.profile(), xp.profile())?;
        let mut bytes = quests::turn_in_xp::PROFILE_DOMAIN.to_vec();
        bytes.extend(recipe);
        let recipe_hash = crate::terrain::sha256(&bytes);
        let mut bound = quests::turn_in_xp::LIFECYCLE_PROFILE_DOMAIN.to_vec();
        bound.extend(profile.hash);
        bound.extend(recipe_hash);
        bound.extend(xp.hash());
        profile.hash = crate::terrain::sha256(&bound);
        profile.completion = Some((Arc::new(checked), xp.hash()));
        Ok(profile)
    }
    pub fn turn_in_xp(&self) -> Option<&quests::turn_in_xp::Recipe> {
        self.completion.as_ref().map(|c| c.0.as_ref())
    }
    pub fn check_progression(&self, xp: &crate::progression_wire::Profile) -> Result<()> {
        if xp
            .profile()
            .kill_experience(self.profile.creature())
            .is_none()
            || self.completion.as_ref().is_some_and(|c| c.1 != xp.hash())
        {
            return Err("Explicit quest/progression profile mismatch".into());
        }
        Ok(())
    }
    pub fn profile(&self) -> &CoreProfile {
        &self.profile
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}
pub fn profile_identity(base: [u8; 32], profile: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-quest-lifecycle-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(profile);
    crate::terrain::sha256(&bytes)
}
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn key(bytes: &[u8], at: usize) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(word(bytes, at)).ok_or("Invalid quest lifecycle epoch")?,
        entity: EntityId::new(word(bytes, at + 8)).ok_or("Invalid quest lifecycle entity")?,
    })
}
fn push_key(bytes: &mut Vec<u8>, key: EntityKey) {
    bytes.extend(key.epoch.value().to_le_bytes());
    bytes.extend(key.entity.value().to_le_bytes());
}
fn status_code(status: Status) -> u8 {
    match status {
        Status::Available => 0,
        Status::Accepted => 1,
        Status::TurnedIn => 2,
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    pub progress: life::State,
}
impl State {
    pub fn encode(&self, profile: &Profile) -> Result<Vec<u8>> {
        if self.profile_hash == [0; 32] || self.profile_hash != profile.hash() {
            return Err("Quest lifecycle observation profile mismatch".into());
        }
        self.progress
            .progress(profile.profile())
            .map_err(|_| "Invalid quest lifecycle state")?;
        let mut bytes = STATE_TAG.to_vec();
        push_key(&mut bytes, self.actor);
        bytes.extend(self.profile_hash);
        bytes.extend(self.progress.revision().to_le_bytes());
        bytes.extend(self.progress.count().to_le_bytes());
        bytes.push(status_code(self.progress.status()));
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], profile: &Profile) -> Result<Self> {
        if bytes.len() != STATE_BYTES || !bytes.starts_with(STATE_TAG) {
            return Err("Invalid quest lifecycle observation frame".into());
        }
        let status = match bytes[64] {
            0 => Status::Available,
            1 => Status::Accepted,
            2 => Status::TurnedIn,
            _ => return Err("Invalid quest lifecycle status".into()),
        };
        let state = Self {
            actor: key(bytes, 4)?,
            profile_hash: bytes[20..52].try_into().unwrap(),
            progress: life::State::restore(
                profile.profile(),
                word(bytes, 52),
                status,
                u32::from_le_bytes(bytes[60..64].try_into().unwrap()),
            )
            .map_err(|_| "Invalid quest lifecycle progress")?,
        };
        state.encode(profile)?;
        Ok(state)
    }
    pub fn validate_successor(
        &self,
        profile: &Profile,
        actor: EntityKey,
        prior: Option<&Self>,
    ) -> Result<()> {
        self.encode(profile)?;
        if self.actor != actor {
            return Err("Foreign quest lifecycle observation".into());
        }
        if let Some(old) = prior {
            old.encode(profile)?;
            if old.actor != actor
                || self.profile_hash != old.profile_hash
                || self.progress.revision() < old.progress.revision()
                || self.progress.count() < old.progress.count()
                || status_code(self.progress.status()) < status_code(old.progress.status())
                || (self.progress.revision() == old.progress.revision() && self != old)
            {
                return Err("Regressed quest lifecycle observation".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub actor: EntityKey,
    pub sequence: u64,
    pub command: Command,
    pub dialogue_revision: u64,
    pub npc: EntityKey,
}
impl Request {
    pub fn encode(self) -> Result<Vec<u8>> {
        if self.sequence == 0
            || self.command.expected_revision > u64::from(quests::MAX_REQUIRED_KILLS) + 1
            || self.dialogue_revision == 0
            || self.npc.epoch != self.actor.epoch
            || self.npc.entity == self.actor.entity
        {
            return Err("Invalid quest lifecycle request context".into());
        }
        let mut bytes = REQUEST_TAG.to_vec();
        push_key(&mut bytes, self.actor);
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.command.expected_revision.to_le_bytes());
        bytes.extend(self.dialogue_revision.to_le_bytes());
        push_key(&mut bytes, self.npc);
        bytes.push(self.command.canonical_bytes()[8]);
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES || !bytes.starts_with(REQUEST_TAG) {
            return Err("Invalid quest lifecycle request frame".into());
        }
        let action = match bytes[60] {
            1 => Action::Accept,
            2 => Action::TurnIn,
            _ => return Err("Invalid quest lifecycle action".into()),
        };
        let request = Self {
            actor: key(bytes, 4)?,
            sequence: word(bytes, 20),
            command: Command {
                expected_revision: word(bytes, 28),
                action,
            },
            dialogue_revision: word(bytes, 36),
            npc: key(bytes, 44)?,
        };
        request.encode()?;
        Ok(request)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    NotEntitled,
    StaleRevision,
    NotAvailable,
    NotReady,
    AlreadyTurnedIn,
    Busy,
}
impl Rejection {
    fn code(self) -> u8 {
        match self {
            Self::NotEntitled => 1,
            Self::StaleRevision => 2,
            Self::NotAvailable => 3,
            Self::NotReady => 4,
            Self::AlreadyTurnedIn => 5,
            Self::Busy => 6,
        }
    }
    fn from_code(code: u8) -> Result<Option<Self>> {
        Ok(match code {
            0 => None,
            1 => Some(Self::NotEntitled),
            2 => Some(Self::StaleRevision),
            3 => Some(Self::NotAvailable),
            4 => Some(Self::NotReady),
            5 => Some(Self::AlreadyTurnedIn),
            6 => Some(Self::Busy),
            _ => return Err("Invalid quest lifecycle rejection".into()),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    pub sequence: u64,
    pub quest_revision: u64,
    pub rejection: Option<Rejection>,
}
impl Reply {
    pub fn encode(self, profile: &Profile) -> Result<Vec<u8>> {
        if self.profile_hash == [0; 32]
            || self.profile_hash != profile.hash()
            || self.sequence == 0
            || self.quest_revision > u64::from(profile.profile().required()) + 2
        {
            return Err("Invalid quest lifecycle reply context".into());
        }
        let mut bytes = REPLY_TAG.to_vec();
        push_key(&mut bytes, self.actor);
        bytes.extend(self.profile_hash);
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.quest_revision.to_le_bytes());
        bytes.push(self.rejection.map_or(0, Rejection::code));
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], profile: &Profile) -> Result<Self> {
        if bytes.len() != REPLY_BYTES || !bytes.starts_with(REPLY_TAG) {
            return Err("Invalid quest lifecycle reply frame".into());
        }
        let reply = Self {
            actor: key(bytes, 4)?,
            profile_hash: bytes[20..52].try_into().unwrap(),
            sequence: word(bytes, 52),
            quest_revision: word(bytes, 60),
            rejection: Rejection::from_code(bytes[68])?,
        };
        reply.encode(profile)?;
        Ok(reply)
    }
    pub fn validate_successor(
        self,
        profile: &Profile,
        actor: EntityKey,
        sent: u64,
        prior: Option<Self>,
    ) -> Result<()> {
        self.encode(profile)?;
        if let Some(old) = prior {
            old.encode(profile)?;
        }
        if self.actor != actor
            || self.sequence == 0
            || self.sequence > sent
            || prior.is_some_and(|old| {
                old.actor != actor
                    || old.profile_hash != self.profile_hash
                    || self.sequence < old.sequence
                    || self.quest_revision < old.quest_revision
                    || (self.sequence == old.sequence && self != old)
            })
        {
            return Err("Foreign or regressed quest lifecycle reply".into());
        }
        Ok(())
    }
}
