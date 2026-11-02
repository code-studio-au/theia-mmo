//! Explicit PL02 owner-only life plus combat membership. Old PL01 never gains
//! implicit membership; clients require an exact policy and actor/life context.
use crate::{Result, player_life_wire as life, simulation::combat::player_life::Observation};
use theia_core::{
    combat_membership::{Profile as Policy, Summary},
    identity::EntityKey,
    time::TickRate,
};
pub const TAG: &[u8; 4] = b"PL02";
const TRAILER_BYTES: usize = 34;
pub const MIN_FRAME_BYTES: usize = life::MIN_FRAME_BYTES + TRAILER_BYTES;
pub const MAX_FRAME_BYTES: usize = life::MAX_FRAME_BYTES + TRAILER_BYTES;
#[derive(Clone, Debug)]
pub struct Profile {
    hash: [u8; 32],
    policy: Policy,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let policy = Policy::from_json(bytes)?;
        Ok(Self {
            hash: crate::terrain::sha256(bytes),
            policy,
        })
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}
pub fn is_state(bytes: &[u8]) -> bool {
    bytes.starts_with(TAG)
}
pub fn frame_rate(bytes: &[u8]) -> std::result::Result<TickRate, &'static str> {
    if !(MIN_FRAME_BYTES..=MAX_FRAME_BYTES).contains(&bytes.len()) || !is_state(bytes) {
        return Err("Invalid combat-membership life frame header");
    }
    TickRate::from_hz(u32::from(bytes[4])).ok_or("Unsupported membership tick rate")
}
fn valid(profile: &Profile, value: &Observation, summary: Summary) -> bool {
    let any = summary.outgoing_hostility || summary.incoming_hostility || summary.active_encounter;
    summary.opponents <= profile.policy.maximum_opponents()
        && (summary.opponents > 0) == any
        && (value.health > 0 || summary == Summary::default())
}
pub fn encode(
    profile: &life::Profile,
    membership: &Profile,
    rate: TickRate,
    value: &Observation,
) -> std::result::Result<Vec<u8>, &'static str> {
    let summary = value
        .combat
        .ok_or("Missing explicit membership observation")?;
    if !valid(membership, value, summary) {
        return Err("Invalid membership summary");
    }
    let mut legacy = *value;
    legacy.combat = None;
    let mut bytes = life::encode(profile, rate, &legacy)?;
    bytes[..4].copy_from_slice(TAG);
    bytes.extend(membership.hash);
    bytes.push(u8::try_from(summary.opponents).map_err(|_| "Membership count overflow")?);
    bytes.push(
        u8::from(summary.outgoing_hostility)
            | (u8::from(summary.incoming_hostility) << 1)
            | (u8::from(summary.active_encounter) << 2),
    );
    debug_assert!(bytes.len() <= MAX_FRAME_BYTES);
    Ok(bytes)
}
pub fn decode(
    profile: &life::Profile,
    membership: &Profile,
    actor: EntityKey,
    rate: TickRate,
    bytes: &[u8],
) -> std::result::Result<Observation, &'static str> {
    if frame_rate(bytes)? != rate {
        return Err("Membership tick rate mismatch");
    }
    let split = bytes.len() - TRAILER_BYTES;
    if bytes[split..split + 32] != membership.hash {
        return Err("Membership profile mismatch");
    }
    let count = bytes[split + 32];
    let flags = bytes[split + 33];
    if flags & !7 != 0 {
        return Err("Unknown membership flags");
    }
    let mut legacy = bytes[..split].to_vec();
    legacy[..4].copy_from_slice(life::TAG);
    let mut value = life::decode(profile, actor, rate, &legacy)?;
    let summary = Summary {
        opponents: usize::from(count),
        outgoing_hostility: flags & 1 != 0,
        incoming_hostility: flags & 2 != 0,
        active_encounter: flags & 4 != 0,
    };
    if !valid(membership, &value, summary) {
        return Err("Invalid membership summary");
    }
    value.combat = Some(summary);
    Ok(value)
}
