//! Standalone bounded local-chat codec; runtime routing is enabled separately.
//! Client intents carry no sender name, position, area or recipient list.
use crate::Result;
use theia_core::{
    character_name::{CharacterName, MAX_BYTES as NAME_BYTES},
    chat::{Channel, MAX_TEXT_BYTES, Request as Intent, Text},
    identity::{EntityId, EntityKey, WorldEpoch},
};
pub const REQUEST_TAG: &[u8; 4] = b"CQ01";
pub const DELIVERY_TAG: &[u8; 4] = b"CD01";
pub const REQUEST_HEADER_BYTES: usize = 23;
pub const MAX_REQUEST_BYTES: usize = REQUEST_HEADER_BYTES + MAX_TEXT_BYTES;
pub const DELIVERY_HEADER_BYTES: usize = 72;
pub const MAX_DELIVERY_BYTES: usize = DELIVERY_HEADER_BYTES + NAME_BYTES + MAX_TEXT_BYTES;
#[derive(Clone, Debug)]
pub struct Request {
    pub epoch: WorldEpoch,
    pub intent: Intent,
}
#[derive(Clone, Debug)]
pub struct Delivery {
    pub world: [u8; 32],
    pub sender: EntityKey,
    pub sequence: u64,
    pub tick: u64,
    pub channel: Channel,
    pub name: CharacterName,
    pub text: Text,
}
fn channel(value: u8) -> Result<Channel> {
    match value {
        0 => Ok(Channel::Say),
        1 => Ok(Channel::Area),
        _ => Err("Invalid chat channel".into()),
    }
}
fn channel_byte(value: Channel) -> u8 {
    match value {
        Channel::Say => 0,
        Channel::Area => 1,
    }
}
fn text(bytes: &[u8]) -> Result<Text> {
    Text::parse(std::str::from_utf8(bytes)?).map_err(|_| "Invalid chat text".into())
}
impl Request {
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.intent.sequence == 0 {
            return Err("Invalid chat request sequence".into());
        }
        let mut bytes = Vec::with_capacity(MAX_REQUEST_BYTES);
        bytes.extend(REQUEST_TAG);
        bytes.extend(self.epoch.value().to_le_bytes());
        bytes.extend(self.intent.sequence.to_le_bytes());
        bytes.push(channel_byte(self.intent.channel));
        bytes.extend((self.intent.text.as_str().len() as u16).to_le_bytes());
        bytes.extend(self.intent.text.as_str().as_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if !(REQUEST_HEADER_BYTES + 1..=MAX_REQUEST_BYTES).contains(&bytes.len())
            || !bytes.starts_with(REQUEST_TAG)
        {
            return Err("Invalid chat request frame".into());
        }
        let epoch = WorldEpoch::new(u64::from_le_bytes(bytes[4..12].try_into()?))
            .ok_or("Invalid chat epoch")?;
        let sequence = u64::from_le_bytes(bytes[12..20].try_into()?);
        let channel = channel(bytes[20])?;
        let length = usize::from(u16::from_le_bytes(bytes[21..23].try_into()?));
        if sequence == 0 || bytes.len() != REQUEST_HEADER_BYTES + length {
            return Err("Invalid chat request length/sequence".into());
        }
        Ok(Self {
            epoch,
            intent: Intent {
                sequence,
                channel,
                text: text(&bytes[23..])?,
            },
        })
    }
}
impl Delivery {
    fn validate(&self) -> Result<()> {
        if self.world == [0; 32] || self.sequence == 0 || self.tick == 0 {
            return Err("Invalid chat delivery context".into());
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = Vec::with_capacity(MAX_DELIVERY_BYTES);
        bytes.extend(DELIVERY_TAG);
        bytes.extend(self.world);
        bytes.extend(self.sender.epoch.value().to_le_bytes());
        bytes.extend(self.sender.entity.value().to_le_bytes());
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.push(channel_byte(self.channel));
        bytes.push(self.name.as_str().len() as u8);
        bytes.extend((self.text.as_str().len() as u16).to_le_bytes());
        bytes.extend(self.name.as_str().as_bytes());
        bytes.extend(self.text.as_str().as_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if !(DELIVERY_HEADER_BYTES + 4..=MAX_DELIVERY_BYTES).contains(&bytes.len())
            || !bytes.starts_with(DELIVERY_TAG)
        {
            return Err("Invalid chat delivery frame".into());
        }
        let epoch = WorldEpoch::new(u64::from_le_bytes(bytes[36..44].try_into()?))
            .ok_or("Invalid chat epoch")?;
        let entity = EntityId::new(u64::from_le_bytes(bytes[44..52].try_into()?))
            .ok_or("Invalid chat sender")?;
        let name_len = usize::from(bytes[69]);
        let text_len = usize::from(u16::from_le_bytes(bytes[70..72].try_into()?));
        if !(3..=NAME_BYTES).contains(&name_len)
            || bytes.len() != DELIVERY_HEADER_BYTES + name_len + text_len
        {
            return Err("Invalid chat delivery lengths".into());
        }
        let split = DELIVERY_HEADER_BYTES + name_len;
        let delivery = Self {
            world: bytes[4..36].try_into()?,
            sender: EntityKey { epoch, entity },
            sequence: u64::from_le_bytes(bytes[52..60].try_into()?),
            tick: u64::from_le_bytes(bytes[60..68].try_into()?),
            channel: channel(bytes[68])?,
            name: CharacterName::parse(std::str::from_utf8(&bytes[72..split])?)?,
            text: text(&bytes[split..])?,
        };
        delivery.validate()?;
        Ok(delivery)
    }
    /// Admission check only: chat recipients need not have a current movement
    /// snapshot for this sender. Deduplication/history remain separate concerns.
    pub fn validate_context(&self, epoch: WorldEpoch, world: [u8; 32]) -> Result<()> {
        self.validate()?;
        if self.sender.epoch != epoch || self.world != world {
            return Err("Chat outside admitted world".into());
        }
        Ok(())
    }
}
/// Private bounded acknowledgement. Identity is the admitted recipient, never
/// the public sender of another message. No message body is echoed here.
pub const REPLY_TAG: &[u8; 4] = b"CR01";
pub const REPLY_BYTES: usize = 69;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Accepted,
    Unavailable,
    Sequence,
    Time,
    Muted,
    NoArea,
    RateLimited,
}
#[derive(Clone, Debug)]
pub struct Reply {
    pub world: [u8; 32],
    pub owner: EntityKey,
    pub sequence: u64,
    pub tick: u64,
    pub outcome: Outcome,
}
impl Reply {
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.world == [0; 32] || self.sequence == 0 || self.tick == 0 {
            return Err("Invalid chat reply context".into());
        }
        let mut bytes = Vec::with_capacity(REPLY_BYTES);
        bytes.extend(REPLY_TAG);
        bytes.extend(self.world);
        bytes.extend(self.owner.epoch.value().to_le_bytes());
        bytes.extend(self.owner.entity.value().to_le_bytes());
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.push(match self.outcome {
            Outcome::Accepted => 0,
            Outcome::Unavailable => 1,
            Outcome::Sequence => 2,
            Outcome::Time => 3,
            Outcome::Muted => 4,
            Outcome::NoArea => 5,
            Outcome::RateLimited => 6,
        });
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REPLY_BYTES || !bytes.starts_with(REPLY_TAG) {
            return Err("Invalid chat reply frame".into());
        }
        let reply = Self {
            world: bytes[4..36].try_into()?,
            owner: EntityKey {
                epoch: WorldEpoch::new(u64::from_le_bytes(bytes[36..44].try_into()?))
                    .ok_or("Invalid chat reply epoch")?,
                entity: EntityId::new(u64::from_le_bytes(bytes[44..52].try_into()?))
                    .ok_or("Invalid chat reply owner")?,
            },
            sequence: u64::from_le_bytes(bytes[52..60].try_into()?),
            tick: u64::from_le_bytes(bytes[60..68].try_into()?),
            outcome: match bytes[68] {
                0 => Outcome::Accepted,
                1 => Outcome::Unavailable,
                2 => Outcome::Sequence,
                3 => Outcome::Time,
                4 => Outcome::Muted,
                5 => Outcome::NoArea,
                6 => Outcome::RateLimited,
                _ => return Err("Invalid chat reply outcome".into()),
            },
        };
        if reply.world == [0; 32] || reply.sequence == 0 || reply.tick == 0 {
            return Err("Invalid chat reply context".into());
        }
        Ok(reply)
    }
    pub fn validate_context(&self, owner: EntityKey, world: [u8; 32]) -> Result<()> {
        if self.owner != owner
            || self.world != world
            || world == [0; 32]
            || self.sequence == 0
            || self.tick == 0
        {
            return Err("Chat reply outside admitted owner/world".into());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_replies_bound_exact_frames_and_admitted_owner_for_every_outcome() {
        let source = delivery();
        for outcome in [
            Outcome::Accepted,
            Outcome::Unavailable,
            Outcome::Sequence,
            Outcome::Time,
            Outcome::Muted,
            Outcome::NoArea,
            Outcome::RateLimited,
        ] {
            let reply = Reply {
                world: source.world,
                owner: source.sender,
                sequence: 1,
                tick: 2,
                outcome,
            };
            let bytes = reply.encode().unwrap();
            assert_eq!(bytes.len(), REPLY_BYTES);
            let decoded = Reply::decode(&bytes).unwrap();
            assert_eq!(decoded.outcome, outcome);
            assert!(
                decoded
                    .validate_context(source.sender, source.world)
                    .is_ok()
            );
            let mut foreign = source.sender;
            foreign.entity = EntityId::new(4).unwrap();
            assert!(decoded.validate_context(foreign, source.world).is_err());
            assert!(decoded.validate_context(source.sender, [2; 32]).is_err());
            for cut in 0..bytes.len() {
                assert!(Reply::decode(&bytes[..cut]).is_err());
            }
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(Reply::decode(&trailing).is_err());
            for offset in [0, 4, 36, 44, 52, 60, 68] {
                let mut bad = bytes.clone();
                match offset {
                    4 => bad[4..36].fill(0),
                    36 | 44 | 52 | 60 => bad[offset..offset + 8].fill(0),
                    68 => bad[68] = 7,
                    _ => bad[0] = 0,
                }
                assert!(Reply::decode(&bad).is_err());
            }
        }
    }
    fn delivery() -> Delivery {
        Delivery {
            world: [1; 32],
            sender: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(3).unwrap(),
            },
            sequence: 1,
            tick: 2,
            channel: Channel::Say,
            name: CharacterName::parse("Arden").unwrap(),
            text: Text::parse("Hello 🙂").unwrap(),
        }
    }
    #[test]
    fn request_and_delivery_round_trip_at_full_bounds_in_both_channels() {
        for channel in [Channel::Say, Channel::Area] {
            let mut delivery = delivery();
            delivery.channel = channel;
            delivery.name = CharacterName::parse(&"W".repeat(NAME_BYTES)).unwrap();
            delivery.text = Text::parse(&"é".repeat(MAX_TEXT_BYTES / 2)).unwrap();
            let request = Request {
                epoch: delivery.sender.epoch,
                intent: Intent {
                    sequence: u64::MAX,
                    channel,
                    text: delivery.text.clone(),
                },
            };
            let bytes = request.encode().unwrap();
            assert_eq!(bytes.len(), MAX_REQUEST_BYTES);
            let decoded = Request::decode(&bytes).unwrap();
            assert_eq!(decoded.encode().unwrap(), bytes);
            let bytes = delivery.encode().unwrap();
            assert_eq!(bytes.len(), MAX_DELIVERY_BYTES);
            assert_eq!(Delivery::decode(&bytes).unwrap().encode().unwrap(), bytes);
            assert!(
                delivery
                    .validate_context(delivery.sender.epoch, [1; 32])
                    .is_ok()
            );
            assert!(
                delivery
                    .validate_context(WorldEpoch::new(8).unwrap(), [1; 32])
                    .is_err()
            );
            assert!(
                delivery
                    .validate_context(delivery.sender.epoch, [2; 32])
                    .is_err()
            );
        }
    }
    #[test]
    fn exact_lengths_tags_utf8_text_and_identity_are_checked_before_acceptance() {
        let delivery = delivery();
        let request = Request {
            epoch: delivery.sender.epoch,
            intent: Intent {
                sequence: 1,
                channel: Channel::Say,
                text: delivery.text.clone(),
            },
        };
        let a = request.encode().unwrap();
        let b = delivery.encode().unwrap();
        for n in 0..a.len() {
            assert!(Request::decode(&a[..n]).is_err());
        }
        for n in 0..b.len() {
            assert!(Delivery::decode(&b[..n]).is_err());
        }
        let mut trailing = a.clone();
        trailing.push(0);
        assert!(Request::decode(&trailing).is_err());
        let mut trailing = b.clone();
        trailing.push(0);
        assert!(Delivery::decode(&trailing).is_err());
        for (offset, value) in [
            (0, 0),
            (4, 0),
            (12, 0),
            (20, 2),
            (21, 0),
            (23, b'\n'),
            (23, 255),
        ] {
            let mut bad = a.clone();
            if offset == 4 || offset == 12 {
                bad[offset..offset + 8].fill(0);
            } else {
                bad[offset] = value;
            }
            assert!(Request::decode(&bad).is_err(), "request offset {offset}");
        }
        for offset in [0, 4, 36, 44, 52, 60, 68, 69, 70, 72, 77] {
            let mut bad = b.clone();
            match offset {
                4 => bad[4..36].fill(0),
                36 | 44 | 52 | 60 => bad[offset..offset + 8].fill(0),
                68 => bad[offset] = 2,
                72 => bad[offset] = 255,
                77 => bad[offset] = b'\n',
                _ => bad[offset] = 0,
            }
            assert!(Delivery::decode(&bad).is_err(), "delivery offset {offset}");
        }
        assert!(Request::decode(&vec![0; MAX_REQUEST_BYTES + 1]).is_err());
        assert!(Delivery::decode(&vec![0; MAX_DELIVERY_BYTES + 1]).is_err());
        let mut zero = request;
        zero.intent.sequence = 0;
        assert!(zero.encode().is_err());
        for case in 0..3 {
            let mut zero = delivery.clone();
            match case {
                0 => zero.world = [0; 32],
                1 => zero.sequence = 0,
                _ => zero.tick = 0,
            }
            assert!(zero.encode().is_err());
        }
    }
}
