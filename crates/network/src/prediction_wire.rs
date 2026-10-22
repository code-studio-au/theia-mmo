//! Explicit opt-in movement profile 1, carried by Renet's unreliable channel.
//! Legacy wire-v11 snapshots/control remain unchanged. Fixed binary lengths keep
//! owner corrections separate from recipient-local multi-player snapshots.
use crate::{
    commands::{Command, State},
    kinematic::MotionState,
    prediction::{Correction, Pose},
    wire::Intent,
};
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
pub const COMMAND_TAG: &[u8; 4] = b"TC01";
pub const CORRECTION_TAG: &[u8; 4] = b"TP01";
pub const MAX_COMMANDS: usize = 4;
#[derive(Clone, Debug)]
pub struct Commands {
    pub epoch: u64,
    pub stream_epoch: u64,
    pub ready_sequence: u64,
    pub commands: Vec<Command>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corrected {
    pub state: Correction,
    pub speed: u8,
}
fn intent(out: &mut Vec<u8>, value: Intent) {
    for axis in value.axes {
        out.extend(axis.to_le_bytes());
    }
    out.push(u8::from(value.sprint) | (u8::from(value.jump) << 1));
}
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], &'static str> {
        let (value, rest) = self
            .0
            .split_at_checked(N)
            .ok_or("Truncated prediction packet")?;
        self.0 = rest;
        Ok(value.try_into().unwrap())
    }
    fn u64(&mut self) -> Result<u64, &'static str> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn intent(&mut self) -> Result<Intent, &'static str> {
        let axes = [
            i16::from_le_bytes(self.take()?),
            i16::from_le_bytes(self.take()?),
        ];
        let flags = self.take::<1>()?[0];
        let value = Intent {
            axes,
            sprint: flags & 1 != 0,
            jump: flags & 2 != 0,
        };
        if flags > 3 || !value.valid() {
            return Err("Invalid prediction intent");
        }
        Ok(value)
    }
}
impl Commands {
    pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
        if self.epoch == 0
            || self.stream_epoch == 0
            || self.commands.is_empty()
            || self.commands.len() > MAX_COMMANDS
            || self
                .commands
                .iter()
                .any(|c| c.sequence == 0 || !c.intent.valid())
            || self
                .commands
                .windows(2)
                .any(|c| c[0].tick >= c[1].tick || c[0].sequence >= c[1].sequence)
        {
            return Err("Invalid prediction commands");
        }
        let mut out = COMMAND_TAG.to_vec();
        for v in [self.epoch, self.stream_epoch, self.ready_sequence] {
            out.extend(v.to_le_bytes());
        }
        out.push(self.commands.len() as u8);
        for c in &self.commands {
            out.extend(c.sequence.to_le_bytes());
            out.extend(c.tick.to_le_bytes());
            intent(&mut out, c.intent);
        }
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > 29 + MAX_COMMANDS * 21 {
            return Err("Oversized prediction command packet");
        }
        let mut r = Reader(bytes);
        if &r.take::<4>()? != COMMAND_TAG {
            return Err("Unknown prediction command profile");
        }
        let (epoch, stream_epoch, ready_sequence) = (r.u64()?, r.u64()?, r.u64()?);
        let count = r.take::<1>()?[0] as usize;
        if !(1..=MAX_COMMANDS).contains(&count) {
            return Err("Invalid command count");
        }
        let mut commands = Vec::with_capacity(count);
        for _ in 0..count {
            commands.push(Command {
                sequence: r.u64()?,
                tick: r.u64()?,
                intent: r.intent()?,
            });
        }
        let value = Self {
            epoch,
            stream_epoch,
            ready_sequence,
            commands,
        };
        if !r.0.is_empty() {
            return Err("Trailing prediction command bytes");
        }
        value.encode()?;
        Ok(value)
    }
}
impl Corrected {
    pub fn encode(self) -> Result<Vec<u8>, &'static str> {
        let c = self.state;
        if !c.valid() || !matches!(self.speed, 1 | 8 | 32) {
            return Err("Invalid prediction correction");
        }
        let mut out = CORRECTION_TAG.to_vec();
        out.extend(c.world);
        for v in [c.actor.epoch.value(), c.actor.entity.value()] {
            out.extend(v.to_le_bytes());
        }
        out.push(c.rate.hz() as u8);
        for v in [
            c.input.stream_epoch,
            c.input.tick,
            c.input.applied_sequence,
            c.input.fresh_tick,
        ] {
            out.extend(v.to_le_bytes());
        }
        intent(&mut out, c.input.intent);
        for v in c.pose.feet.metres() {
            out.extend(v.to_le_bytes());
        }
        out.extend(c.pose.motion.vertical_speed.to_le_bytes());
        out.push(u8::from(c.pose.motion.grounded) | (u8::from(c.pose.motion.jump_held) << 1));
        out.extend(c.ready_sequence.to_le_bytes());
        out.push(self.speed);
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 128 {
            return Err("Invalid prediction correction length");
        }
        let mut r = Reader(bytes);
        if &r.take::<4>()? != CORRECTION_TAG {
            return Err("Unknown prediction correction profile");
        }
        let world = r.take()?;
        let actor = EntityKey {
            epoch: WorldEpoch::new(r.u64()?).ok_or("Invalid world epoch")?,
            entity: EntityId::new(r.u64()?).ok_or("Invalid actor")?,
        };
        let rate =
            TickRate::from_hz(u32::from(r.take::<1>()?[0])).ok_or("Invalid prediction rate")?;
        let input = State {
            stream_epoch: r.u64()?,
            tick: r.u64()?,
            applied_sequence: r.u64()?,
            fresh_tick: r.u64()?,
            intent: r.intent()?,
        };
        let feet = GlobalPosition::new([
            f64::from_le_bytes(r.take()?),
            f64::from_le_bytes(r.take()?),
            f64::from_le_bytes(r.take()?),
        ])
        .ok_or("Invalid correction feet")?;
        let vertical_speed = f32::from_le_bytes(r.take()?);
        let flags = r.take::<1>()?[0];
        if flags > 3 {
            return Err("Invalid correction motion flags");
        }
        let motion = MotionState {
            vertical_speed,
            grounded: flags & 1 != 0,
            jump_held: flags & 2 != 0,
        };
        let ready_sequence = r.u64()?;
        let speed = r.take::<1>()?[0];
        let value = Self {
            state: Correction {
                world,
                actor,
                rate,
                input,
                pose: Pose { feet, motion },
                ready_sequence,
            },
            speed,
        };
        value.encode()?;
        Ok(value)
    }
}
