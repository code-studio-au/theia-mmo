//! Bounded informational dialogue frames. Encoding is a contract prerequisite;
//! endpoint adoption and actual encrypted exchange must be qualified separately.
use crate::{
    Result,
    dialogue::{Action, Guide, Open, Profile, Rejection, Request, View},
};
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
};

pub const REQUEST_TAG: &[u8; 4] = b"DI01";
pub const VIEW_TAG: &[u8; 4] = b"DV01";
pub const GUIDE_TAG: &[u8; 4] = b"DG01";
pub const REPLY_TAG: &[u8; 4] = b"DR01";
pub const REPLY_BYTES: usize = 69;
pub const MAX_REQUEST_BYTES: usize = 126;
pub const MAX_VIEW_BYTES: usize = 174;
pub const MAX_GUIDE_BYTES: usize = 93;

fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn key(bytes: &[u8], at: usize) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(word(bytes, at)).ok_or("Invalid dialogue epoch")?,
        entity: EntityId::new(word(bytes, at + 8)).ok_or("Invalid dialogue entity")?,
    })
}
fn push_key(bytes: &mut Vec<u8>, key: EntityKey) {
    bytes.extend(key.epoch.value().to_le_bytes());
    bytes.extend(key.entity.value().to_le_bytes());
}
fn content(bytes: &[u8]) -> Result<ContentId> {
    ContentId::new(std::str::from_utf8(bytes)?).ok_or_else(|| "Invalid dialogue content ID".into())
}
fn same_world(actor: EntityKey, guide: EntityKey) -> bool {
    actor.epoch == guide.epoch && actor.entity != guide.entity
}
pub fn encode_request(epoch: u64, request: &Request) -> Result<Vec<u8>> {
    if epoch == 0 || request.sequence == 0 {
        return Err("Invalid dialogue request identity".into());
    }
    let mut bytes = REQUEST_TAG.to_vec();
    bytes.extend(epoch.to_le_bytes());
    bytes.extend(request.sequence.to_le_bytes());
    bytes.extend(request.expected_revision.to_le_bytes());
    match &request.action {
        Action::Open(guide) => {
            if guide.epoch.value() != epoch {
                return Err("Foreign dialogue guide".into());
            }
            bytes.push(0);
            push_key(&mut bytes, *guide);
        }
        Action::Choose(choice) => {
            bytes.push(1);
            bytes.push(choice.as_str().len() as u8);
            bytes.extend(choice.as_str().as_bytes());
        }
        Action::Close => bytes.push(2),
    }
    Ok(bytes)
}
pub fn decode_request(bytes: &[u8], epoch: u64) -> Result<Request> {
    if !(29..=MAX_REQUEST_BYTES).contains(&bytes.len())
        || !bytes.starts_with(REQUEST_TAG)
        || epoch == 0
        || word(bytes, 4) != epoch
    {
        return Err("Invalid dialogue request frame".into());
    }
    let action = match bytes[28] {
        0 if bytes.len() == 45 => Action::Open(key(bytes, 29)?),
        1 if bytes.len() >= 31 && bytes.len() == 30 + usize::from(bytes[29]) => {
            Action::Choose(content(&bytes[30..])?)
        }
        2 if bytes.len() == 29 => Action::Close,
        _ => return Err("Invalid dialogue action frame".into()),
    };
    let request = Request {
        sequence: word(bytes, 12),
        expected_revision: word(bytes, 20),
        action,
    };
    encode_request(epoch, &request)?;
    Ok(request)
}
pub fn encode_view(view: &View, profile: &Profile) -> Result<Vec<u8>> {
    if view.profile_hash != profile.hash() {
        return Err("Dialogue view profile mismatch".into());
    }
    let mut bytes = VIEW_TAG.to_vec();
    push_key(&mut bytes, view.actor);
    bytes.extend(view.profile_hash);
    bytes.extend(view.revision.to_le_bytes());
    if let Some(open) = &view.open {
        if !same_world(view.actor, open.guide)
            || view.revision == 0
            || view.revision == u64::MAX
            || profile.profile().node(&open.node).is_none()
        {
            return Err("Invalid open dialogue view".into());
        }
        bytes.push(1);
        push_key(&mut bytes, open.guide);
        bytes.push(open.node.as_str().len() as u8);
        bytes.extend(open.node.as_str().as_bytes());
    } else {
        bytes.push(0);
    }
    Ok(bytes)
}
pub fn decode_view(bytes: &[u8], profile: &Profile) -> Result<View> {
    if !(61..=MAX_VIEW_BYTES).contains(&bytes.len()) || !bytes.starts_with(VIEW_TAG) {
        return Err("Invalid dialogue view frame".into());
    }
    let open = match bytes[60] {
        0 if bytes.len() == 61 => None,
        1 if bytes.len() >= 79 && bytes.len() == 78 + usize::from(bytes[77]) => Some(Open {
            guide: key(bytes, 61)?,
            node: content(&bytes[78..])?,
        }),
        _ => return Err("Invalid dialogue node frame".into()),
    };
    let view = View {
        actor: key(bytes, 4)?,
        profile_hash: bytes[20..52].try_into().unwrap(),
        revision: word(bytes, 52),
        open,
    };
    encode_view(&view, profile)?;
    Ok(view)
}
pub fn validate_successor(view: &View, actor: EntityKey, prior: Option<&View>) -> Result<()> {
    if view.actor != actor
        || prior.is_some_and(|old| {
            old.actor != actor
                || view.profile_hash != old.profile_hash
                || view.revision < old.revision
                || (view.revision == old.revision && view != old)
        })
    {
        return Err("Foreign/regressed/aliased dialogue view".into());
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq)]
pub struct Presence {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    pub guide: Option<Guide>,
}
pub fn encode_presence(presence: &Presence, profile: &Profile) -> Result<Vec<u8>> {
    if presence.profile_hash != profile.hash() {
        return Err("Guide presence profile mismatch".into());
    }
    let mut bytes = GUIDE_TAG.to_vec();
    push_key(&mut bytes, presence.actor);
    bytes.extend(presence.profile_hash);
    if let Some(guide) = presence.guide {
        if !same_world(presence.actor, guide.key) {
            return Err("Foreign guide presence".into());
        }
        bytes.push(1);
        push_key(&mut bytes, guide.key);
        for value in guide.position.metres() {
            bytes.extend(value.to_le_bytes());
        }
    } else {
        bytes.push(0);
    }
    Ok(bytes)
}
pub fn decode_presence(bytes: &[u8], profile: &Profile, actor: EntityKey) -> Result<Presence> {
    if !(53..=MAX_GUIDE_BYTES).contains(&bytes.len()) || !bytes.starts_with(GUIDE_TAG) {
        return Err("Invalid guide presence frame".into());
    }
    let guide = match bytes[52] {
        0 if bytes.len() == 53 => None,
        1 if bytes.len() == MAX_GUIDE_BYTES => Some(Guide {
            key: key(bytes, 53)?,
            position: GlobalPosition::new(std::array::from_fn(|i| {
                f64::from_bits(word(bytes, 69 + 8 * i))
            }))
            .ok_or("Invalid guide coordinates")?,
        }),
        _ => return Err("Invalid guide presence payload".into()),
    };
    let presence = Presence {
        actor: key(bytes, 4)?,
        profile_hash: bytes[20..52].try_into().unwrap(),
        guide,
    };
    if presence.actor != actor {
        return Err("Foreign private guide presence".into());
    }
    encode_presence(&presence, profile)?;
    Ok(presence)
}

/// A request acknowledgement is independent of node mutation: a refused choice
/// can leave the view unchanged. The adopter must publish that view before this
/// reply, and the client waits until it has at least `view_revision`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    pub sequence: u64,
    pub view_revision: u64,
    pub rejection: Option<Rejection>,
}
fn rejection_code(rejection: Option<Rejection>) -> u8 {
    match rejection {
        None => 0,
        Some(Rejection::Unavailable) => 1,
        Some(Rejection::RateLimited) => 2,
        Some(Rejection::StaleSequence) => 3,
        Some(Rejection::StaleRevision) => 4,
        Some(Rejection::InvalidAction) => 5,
        Some(Rejection::OutOfRange) => 6,
        Some(Rejection::Exhausted) => 7,
    }
}
pub fn encode_reply(reply: &Reply, profile: &Profile) -> Result<Vec<u8>> {
    if reply.profile_hash != profile.hash() || reply.sequence == 0 {
        return Err("Invalid dialogue reply identity".into());
    }
    let mut bytes = REPLY_TAG.to_vec();
    push_key(&mut bytes, reply.actor);
    bytes.extend(reply.profile_hash);
    bytes.extend(reply.sequence.to_le_bytes());
    bytes.extend(reply.view_revision.to_le_bytes());
    bytes.push(rejection_code(reply.rejection));
    Ok(bytes)
}
pub fn decode_reply(bytes: &[u8], profile: &Profile) -> Result<Reply> {
    if bytes.len() != REPLY_BYTES || !bytes.starts_with(REPLY_TAG) {
        return Err("Invalid dialogue reply frame".into());
    }
    let rejection = match bytes[68] {
        0 => None,
        1 => Some(Rejection::Unavailable),
        2 => Some(Rejection::RateLimited),
        3 => Some(Rejection::StaleSequence),
        4 => Some(Rejection::StaleRevision),
        5 => Some(Rejection::InvalidAction),
        6 => Some(Rejection::OutOfRange),
        7 => Some(Rejection::Exhausted),
        _ => return Err("Invalid dialogue reply outcome".into()),
    };
    let reply = Reply {
        actor: key(bytes, 4)?,
        profile_hash: bytes[20..52].try_into().unwrap(),
        sequence: word(bytes, 52),
        view_revision: word(bytes, 60),
        rejection,
    };
    encode_reply(&reply, profile)?;
    Ok(reply)
}
pub fn validate_reply(
    reply: &Reply,
    actor: EntityKey,
    sent: u64,
    prior: Option<&Reply>,
) -> Result<()> {
    if reply.actor != actor
        || reply.sequence == 0
        || reply.sequence > sent
        || prior.is_some_and(|old| {
            old.actor != actor
                || old.profile_hash != reply.profile_hash
                || reply.sequence < old.sequence
                || reply.view_revision < old.view_revision
                || (reply.sequence == old.sequence && reply != old)
        })
    {
        return Err("Foreign/regressed/unsolicited dialogue reply".into());
    }
    Ok(())
}
