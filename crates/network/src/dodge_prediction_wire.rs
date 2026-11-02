//! Explicit opt-in TC02/TP02 movement envelopes. These codecs are not enabled
//! by current sessions. Exact profile/world/owner/life admission remains the
//! transport caller's responsibility; corrections validate the bound policy.
use crate::{dodging, prediction_wire};
use theia_core::dodge;

pub const COMMAND_TAG: &[u8; 4] = b"TC02";
pub const CORRECTION_TAG: &[u8; 4] = b"TP02";
pub const MAX_COMMAND_BYTES: usize = 29 + prediction_wire::MAX_COMMANDS * 34;
pub const CORRECTION_BYTES: usize = 170;

fn request_bytes(request: Option<dodging::Request>) -> Result<[u8; 13], &'static str> {
    let mut bytes = [0; 13];
    if let Some(request) = request {
        if !request.valid() {
            return Err("Invalid dodge request");
        }
        bytes[0] = 1;
        bytes[1..9].copy_from_slice(&request.nonce.to_le_bytes());
        bytes[9..11].copy_from_slice(&request.axes[0].to_le_bytes());
        bytes[11..13].copy_from_slice(&request.axes[1].to_le_bytes());
    }
    Ok(bytes)
}
fn read_request(bytes: &[u8]) -> Result<Option<dodging::Request>, &'static str> {
    let bytes: &[u8; 13] = bytes
        .try_into()
        .map_err(|_| "Invalid dodge request length")?;
    let request = match bytes[0] {
        0 => None,
        1 => Some(dodging::Request {
            nonce: u64::from_le_bytes(bytes[1..9].try_into().unwrap()),
            axes: [
                i16::from_le_bytes(bytes[9..11].try_into().unwrap()),
                i16::from_le_bytes(bytes[11..13].try_into().unwrap()),
            ],
        }),
        _ => return Err("Invalid dodge request flag"),
    };
    if request_bytes(request)? != *bytes {
        return Err("Noncanonical dodge request");
    }
    Ok(request)
}
pub fn encode_commands(value: &prediction_wire::Commands) -> Result<Vec<u8>, &'static str> {
    let mut base = value.clone();
    for command in &mut base.commands {
        command.intent.dodge = None;
    }
    let mut bytes = base.encode()?;
    bytes[..4].copy_from_slice(COMMAND_TAG);
    for command in &value.commands {
        bytes.extend(request_bytes(command.intent.dodge)?);
    }
    Ok(bytes)
}
pub fn decode_commands(bytes: &[u8]) -> Result<prediction_wire::Commands, &'static str> {
    if bytes.len() < 29 || bytes.len() > MAX_COMMAND_BYTES || &bytes[..4] != COMMAND_TAG {
        return Err("Invalid dodge command envelope");
    }
    let count = usize::from(bytes[28]);
    if !(1..=prediction_wire::MAX_COMMANDS).contains(&count) || bytes.len() != 29 + count * 34 {
        return Err("Invalid dodge command length");
    }
    let base_length = 29 + count * 21;
    let mut base = bytes[..base_length].to_vec();
    base[..4].copy_from_slice(prediction_wire::COMMAND_TAG);
    let mut value = prediction_wire::Commands::decode(&base)?;
    for (command, suffix) in value
        .commands
        .iter_mut()
        .zip(bytes[base_length..].as_chunks::<13>().0)
    {
        command.intent.dodge = read_request(suffix)?;
    }
    Ok(value)
}
pub fn encode_correction(
    value: prediction_wire::Corrected,
    profile: &dodging::Profile,
) -> Result<Vec<u8>, &'static str> {
    let snapshot = value
        .state
        .pose
        .dodge
        .ok_or("Missing dodge correction state")?;
    if value.speed != 1 || !snapshot.valid_at(profile, value.state.rate, value.state.input.tick) {
        return Err("Invalid bound dodge correction");
    }
    let mut base = value;
    base.state.pose.dodge = None;
    base.state.input.intent.dodge = None;
    let mut bytes = base.encode()?;
    bytes[..4].copy_from_slice(CORRECTION_TAG);
    bytes.extend(request_bytes(value.state.input.intent.dodge)?);
    bytes.extend(snapshot.last_nonce.to_le_bytes());
    bytes.extend(snapshot.policy.ready_at.to_le_bytes());
    bytes.push(u8::from(snapshot.policy.active.is_some()));
    let (started_at, axes) = snapshot
        .policy
        .active
        .map_or((0, [0, 0]), |a| (a.started_at, a.axes));
    bytes.extend(started_at.to_le_bytes());
    for axis in axes {
        bytes.extend(axis.to_le_bytes());
    }
    Ok(bytes)
}
pub fn decode_correction(
    bytes: &[u8],
    profile: &dodging::Profile,
) -> Result<prediction_wire::Corrected, &'static str> {
    if bytes.len() != CORRECTION_BYTES || &bytes[..4] != CORRECTION_TAG {
        return Err("Invalid dodge correction envelope");
    }
    let mut base = bytes[..128].to_vec();
    base[..4].copy_from_slice(prediction_wire::CORRECTION_TAG);
    let mut value = prediction_wire::Corrected::decode(&base)?;
    value.state.input.intent.dodge = read_request(&bytes[128..141])?;
    let last_nonce = u64::from_le_bytes(bytes[141..149].try_into().unwrap());
    let ready_at = u64::from_le_bytes(bytes[149..157].try_into().unwrap());
    let started_at = u64::from_le_bytes(bytes[158..166].try_into().unwrap());
    let axes = [
        i16::from_le_bytes(bytes[166..168].try_into().unwrap()),
        i16::from_le_bytes(bytes[168..170].try_into().unwrap()),
    ];
    let active = match bytes[157] {
        0 if started_at == 0 && axes == [0, 0] => None,
        1 => Some(dodge::Active { started_at, axes }),
        _ => return Err("Invalid dodge active state"),
    };
    value.state.pose.dodge = Some(dodging::Snapshot {
        last_nonce,
        policy: dodge::Snapshot {
            tick: value.state.input.tick,
            ready_at,
            active,
        },
    });
    // Reuse policy validation and enforce canonical representation on all fields.
    if encode_correction(value, profile)? != bytes {
        return Err("Noncanonical dodge correction");
    }
    Ok(value)
}
