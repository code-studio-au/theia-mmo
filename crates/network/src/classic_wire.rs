//! Explicit TC03/TP03 envelopes; legacy encoders refuse independent facing.
use crate::prediction_wire::{Commands, Corrected, MAX_COMMANDS};
use theia_core::facing::Facing;
pub const COMMAND_TAG: &[u8; 4] = b"TC03";
pub const CORRECTION_TAG: &[u8; 4] = b"TP03";
pub const MAX_COMMAND_BYTES: usize = 29 + MAX_COMMANDS * 23;
pub const CORRECTION_BYTES: usize = 133;

pub fn encode_commands(value: &Commands) -> Result<Vec<u8>, &'static str> {
    if !(1..=MAX_COMMANDS).contains(&value.commands.len()) {
        return Err("Invalid classic command count");
    }
    let mut base = value.clone();
    let mut headings = Vec::with_capacity(MAX_COMMANDS * 2);
    for command in &mut base.commands {
        if command.intent.dodge.is_some() {
            return Err("Classic command cannot dodge");
        }
        let facing = command
            .intent
            .facing
            .take()
            .ok_or("Classic command requires facing")?;
        headings.extend(facing.units().to_le_bytes());
    }
    let mut bytes = base.encode()?;
    bytes[..4].copy_from_slice(COMMAND_TAG);
    bytes.extend(headings);
    Ok(bytes)
}
pub fn decode_commands(bytes: &[u8]) -> Result<Commands, &'static str> {
    if bytes.len() < 29 || bytes.len() > MAX_COMMAND_BYTES || &bytes[..4] != COMMAND_TAG {
        return Err("Invalid classic command envelope");
    }
    let count = usize::from(bytes[28]);
    if !(1..=MAX_COMMANDS).contains(&count) || bytes.len() != 29 + count * 23 {
        return Err("Invalid classic command length");
    }
    let end = 29 + count * 21;
    let mut base = bytes[..end].to_vec();
    base[..4].copy_from_slice(crate::prediction_wire::COMMAND_TAG);
    let mut value = Commands::decode(&base)?;
    for (command, units) in value
        .commands
        .iter_mut()
        .zip(bytes[end..].as_chunks::<2>().0)
    {
        command.intent.facing = Some(Facing::from_units(u16::from_le_bytes(*units)));
    }
    Ok(value)
}
pub fn encode_correction(value: Corrected) -> Result<Vec<u8>, &'static str> {
    if !value.state.valid()
        || value.speed != 1
        || value.state.pose.dodge.is_some()
        || value.state.input.intent.dodge.is_some()
    {
        return Err("Invalid classic correction profile");
    }
    let mut base = value;
    let facing = base
        .state
        .pose
        .facing
        .take()
        .ok_or("Missing classic correction facing")?;
    let held = base.state.input.intent.facing.take();
    if held.is_none() && base.state.input.intent != crate::wire::Intent::default() {
        return Err("Classic held movement requires facing");
    }
    let mut bytes = base.encode()?;
    bytes[..4].copy_from_slice(CORRECTION_TAG);
    bytes.extend(facing.units().to_le_bytes());
    bytes.push(u8::from(held.is_some()));
    bytes.extend(held.unwrap_or_default().units().to_le_bytes());
    Ok(bytes)
}
pub fn decode_correction(bytes: &[u8]) -> Result<Corrected, &'static str> {
    if bytes.len() != CORRECTION_BYTES || &bytes[..4] != CORRECTION_TAG {
        return Err("Invalid classic correction envelope");
    }
    let mut base = bytes[..128].to_vec();
    base[..4].copy_from_slice(crate::prediction_wire::CORRECTION_TAG);
    let mut value = Corrected::decode(&base)?;
    value.state.pose.facing = Some(Facing::from_units(u16::from_le_bytes(
        bytes[128..130].try_into().unwrap(),
    )));
    let held = u16::from_le_bytes(bytes[131..133].try_into().unwrap());
    value.state.input.intent.facing = match bytes[130] {
        0 if held == 0 => None,
        1 => Some(Facing::from_units(held)),
        _ => return Err("Noncanonical classic held facing"),
    };
    // Restore validation after adding the held intent: neutral bootstrap and
    // expiry may not hide an unacknowledged input or stale facing command.
    if !value.state.valid()
        || crate::commands::Timeline::restore(value.state.rate, value.state.input).is_err()
    {
        return Err("Invalid classic correction state");
    }
    encode_correction(value)?;
    Ok(value)
}
