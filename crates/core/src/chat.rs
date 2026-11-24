//! Bounded local-chat policy. Callers supply authenticated server-owned context;
//! this module has no transport, account lookup, persistence or rendering.
use crate::{
    character_name::CharacterName, coordinates::GlobalPosition, identity::EntityKey, time::TickRate,
};
pub const MAX_TEXT_BYTES: usize = 256;
/// Initial Theia tuning, not a verified Classic distance conversion.
pub const SAY_METRES: f64 = 25.;
pub const BURST: usize = 3;
pub const WINDOW_SECONDS: u64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextError {
    Length,
    Empty,
    Control,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Text(String);
impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatText")
            .field("bytes", &self.0.len())
            .finish()
    }
}
impl Text {
    /// One plain Unicode line. Preserve spelling and spacing; never truncate a
    /// code point, parse markup or silently normalize the submitted message.
    pub fn parse(value: &str) -> Result<Self, TextError> {
        if value.is_empty() || value.len() > MAX_TEXT_BYTES {
            return Err(TextError::Length);
        }
        if value.chars().any(|c| {
            c.is_control()
                || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{feff}')
        }) {
            return Err(TextError::Control);
        }
        if !value.chars().any(|c| !c.is_whitespace() && c != '\u{200b}') {
            return Err(TextError::Empty);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// Authored chat-area membership supplied by the server. This is neither a
/// render chunk nor a simulation region, and never comes from a chat request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area(std::num::NonZeroU32);
impl Area {
    pub fn new(id: u32) -> Option<Self> {
        std::num::NonZeroU32::new(id).map(Self)
    }
    pub fn value(self) -> u32 {
        self.0.get()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Say,
    Area,
}
/// Trusted context looked up at submission/delivery time, not client metadata.
#[derive(Clone, Debug)]
pub struct Speaker {
    pub key: EntityKey,
    pub name: CharacterName,
    pub position: GlobalPosition,
    pub area: Option<Area>,
}
#[derive(Clone, Copy, Debug)]
pub struct Listener {
    pub key: EntityKey,
    pub position: GlobalPosition,
    pub area: Option<Area>,
    /// Server-owned ignore/moderation result for this particular sender.
    pub blocks_sender: bool,
}
/// Client intent carries no sender name, position, recipient list or authority.
#[derive(Clone, Debug)]
pub struct Request {
    pub sequence: u64,
    pub channel: Channel,
    pub text: Text,
}
#[derive(Clone, Debug)]
pub struct Message {
    pub speaker: Speaker,
    pub sequence: u64,
    pub tick: u64,
    pub channel: Channel,
    pub text: Text,
}
impl Message {
    /// No client recipient list, position or area can widen this audience.
    /// Say uses 3-D global f64 distance; it does not require movement visibility.
    pub fn reaches(&self, listener: Listener) -> bool {
        if self.speaker.key.epoch != listener.key.epoch || listener.blocks_sender {
            return false;
        }
        match self.channel {
            Channel::Say => {
                let source = self.speaker.position.metres();
                let target = listener.position.metres();
                source
                    .into_iter()
                    .zip(target)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    <= SAY_METRES.powi(2)
            }
            Channel::Area => self.speaker.area.is_some() && self.speaker.area == listener.area,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Context,
    Sequence,
    Time,
    Muted,
    NoArea,
    RateLimited,
}
/// One bounded gate per admitted incarnation. The server owns tick/rate and
/// must retain account-wide moderation across reconnects independently.
pub struct Gate {
    owner: EntityKey,
    sequence: u64,
    tick: u64,
    accepted: [Option<u64>; BURST],
    rate: TickRate,
}
impl Gate {
    pub fn new(owner: EntityKey, rate: TickRate) -> Self {
        Self {
            owner,
            sequence: 0,
            tick: 0,
            accepted: [None; BURST],
            rate,
        }
    }
    pub fn submit(
        &mut self,
        speaker: Speaker,
        request: Request,
        tick: u64,
        muted: bool,
    ) -> Result<Message, Refusal> {
        let Request {
            sequence,
            channel,
            text,
        } = request;
        if speaker.key != self.owner {
            return Err(Refusal::Context);
        }
        if tick == 0 || tick < self.tick {
            return Err(Refusal::Time);
        }
        if sequence <= self.sequence || sequence.saturating_sub(self.sequence) > 128 {
            return Err(Refusal::Sequence);
        }
        // A refused request is consumed too: it cannot be replayed after unmute,
        // area change or bucket expiry. Refusals never spend accepted capacity.
        self.sequence = sequence;
        self.tick = tick;
        if muted {
            return Err(Refusal::Muted);
        }
        if channel == Channel::Area && speaker.area.is_none() {
            return Err(Refusal::NoArea);
        }
        let window = u64::from(self.rate.hz()) * WINDOW_SECONDS;
        let slot = self
            .accepted
            .iter_mut()
            .find(|old| old.is_none_or(|at| tick.saturating_sub(at) >= window))
            .ok_or(Refusal::RateLimited)?;
        *slot = Some(tick);
        Ok(Message {
            speaker,
            sequence,
            tick,
            channel,
            text,
        })
    }
}
#[cfg(test)]
mod tests;
