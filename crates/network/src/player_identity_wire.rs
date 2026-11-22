//! Bounded server-owned public player names and cosmetic appearance.
//! No account identity, credentials or gameplay stats.
use crate::Result;
use theia_core::{
    character_name::{CharacterName, MAX_BYTES as NAME_BYTES},
    identity::{EntityId, EntityKey, WorldEpoch},
};
pub const STATE_TAG: &[u8; 4] = b"PN02";
pub const HEADER_BYTES: usize = 53;
pub const ROW_BYTES: usize = 16;
pub const MAX_ACTORS: usize = crate::wire::MAX_PLAYERS;
pub const MAX_BYTES: usize = HEADER_BYTES + MAX_ACTORS * (ROW_BYTES + NAME_BYTES);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub key: EntityKey,
    pub name: Option<CharacterName>,
    pub appearance: theia_core::appearance::Appearance,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub epoch: u64,
    pub tick: u64,
    pub world: [u8; 32],
    pub actors: Vec<Actor>,
}
impl State {
    fn validate(&self) -> Result<()> {
        if self.epoch == 0
            || self.tick == 0
            || self.world == [0; 32]
            || self.actors.len() > MAX_ACTORS
        {
            return Err("Invalid player identity header/count".into());
        }
        let mut previous = 0;
        for actor in &self.actors {
            if actor.key.epoch.value() != self.epoch || actor.key.entity.value() <= previous {
                return Err("Player identities must be ordered in the current epoch".into());
            }
            previous = actor.key.entity.value();
        }
        Ok(())
    }
    /// Roster/name changes matter; movement ticks alone do not require a resend.
    pub fn same_view(&self, other: &Self) -> bool {
        self.epoch == other.epoch && self.world == other.world && self.actors == other.actors
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = Vec::with_capacity(MAX_BYTES);
        bytes.extend(STATE_TAG);
        bytes.extend(self.epoch.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.extend(self.world);
        bytes.push(self.actors.len() as u8);
        for actor in &self.actors {
            bytes.extend(actor.key.entity.value().to_le_bytes());
            let name = actor.name.as_ref().map_or("", CharacterName::as_str);
            bytes.push(name.len() as u8);
            bytes.extend(actor.appearance.values());
            bytes.extend(name.as_bytes());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if !(HEADER_BYTES..=MAX_BYTES).contains(&bytes.len()) || !bytes.starts_with(STATE_TAG) {
            return Err("Invalid player identity frame".into());
        }
        let epoch = WorldEpoch::new(u64::from_le_bytes(bytes[4..12].try_into()?))
            .ok_or("Invalid player identity epoch")?;
        let tick = u64::from_le_bytes(bytes[12..20].try_into()?);
        let world = bytes[20..52].try_into()?;
        let count = usize::from(bytes[52]);
        if tick == 0 || world == [0; 32] || count > MAX_ACTORS {
            return Err("Invalid player identity header/count".into());
        }
        let mut offset = HEADER_BYTES;
        let mut actors = Vec::with_capacity(count);
        for _ in 0..count {
            let entity =
                EntityId::new(u64::from_le_bytes(take(bytes, &mut offset, 8)?.try_into()?))
                    .ok_or("Invalid player identity actor")?;
            let length = usize::from(take(bytes, &mut offset, 1)?[0]);
            if length > NAME_BYTES {
                return Err("Player name exceeds its byte limit".into());
            }
            let appearance =
                theia_core::appearance::Appearance::new(take(bytes, &mut offset, 7)?.try_into()?)?;
            let name = if length == 0 {
                None
            } else {
                Some(CharacterName::parse(std::str::from_utf8(take(
                    bytes,
                    &mut offset,
                    length,
                )?)?)?)
            };
            actors.push(Actor {
                key: EntityKey { epoch, entity },
                name,
                appearance,
            });
        }
        if offset != bytes.len() {
            return Err("Trailing player identity bytes".into());
        }
        let result = Self {
            epoch: epoch.value(),
            tick,
            world,
            actors,
        };
        result.validate()?;
        Ok(result)
    }
    /// Called at the admitted client boundary before replacing its metadata.
    pub fn validate_successor(
        &self,
        owner: EntityKey,
        world: [u8; 32],
        prior: Option<&Self>,
    ) -> Result<()> {
        self.validate()?;
        if self.world != world
            || self.epoch != owner.epoch.value()
            || !self.actors.iter().any(|a| a.key == owner)
        {
            return Err("Player identities outside the admitted owner/world".into());
        }
        if let Some(old) = prior {
            old.validate()?;
            if old.epoch != self.epoch
                || old.world != self.world
                || self.tick < old.tick
                || (self.tick == old.tick && self != old)
            {
                return Err("Stale or conflicting player identities".into());
            }
        }
        Ok(())
    }
}
fn take<'a>(bytes: &'a [u8], offset: &mut usize, length: usize) -> Result<&'a [u8]> {
    let end = offset
        .checked_add(length)
        .ok_or("Player identity offset overflow")?;
    let value = bytes
        .get(*offset..end)
        .ok_or("Truncated player identity row")?;
    *offset = end;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn actor(id: u64, name: Option<&str>) -> Actor {
        Actor {
            key: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(id).unwrap(),
            },
            name: name.map(|name| CharacterName::parse(name).unwrap()),
            appearance: Default::default(),
        }
    }
    fn view() -> State {
        State {
            epoch: 7,
            tick: 10,
            world: [3; 32],
            actors: vec![actor(1, Some("Arden"))],
        }
    }
    #[test]
    fn empty_unknown_and_maximum_views_round_trip_with_bounded_size() {
        for actors in [
            vec![],
            vec![actor(1, None)],
            (1..=MAX_ACTORS as u64)
                .map(|id| actor(id, Some("Abcdefghijklmnopqrst")))
                .collect(),
        ] {
            let state = State { actors, ..view() };
            let bytes = state.encode().unwrap();
            assert_eq!(State::decode(&bytes).unwrap(), state);
            assert!(bytes.len() <= MAX_BYTES);
            if state.actors.len() == MAX_ACTORS {
                assert_eq!(bytes.len(), MAX_BYTES);
            }
        }
        assert_eq!(MAX_BYTES, 341);
        const { assert!(MAX_BYTES <= crate::wire::MAX_BYTES) };
    }
    #[test]
    fn distinct_appearances_roundtrip_and_invalid_catalogue_bytes_reject() {
        let mut state = view();
        state.actors.push(actor(2, Some("Elara")));
        state.actors[1].appearance =
            theia_core::appearance::Appearance::new([1, 3, 4, 0, 3, 2, 1]).unwrap();
        assert_eq!(State::decode(&state.encode().unwrap()).unwrap(), state);
        let before = state.clone();
        state.actors[0].appearance = state.actors[0].appearance.cycle(2, true);
        assert!(!state.same_view(&before));
        for index in 0..7 {
            let mut bytes = state.encode().unwrap();
            bytes[HEADER_BYTES + 9 + index] = 255;
            assert!(State::decode(&bytes).is_err());
        }
    }
    #[test]
    fn all_truncations_and_trailing_payload_are_rejected() {
        let bytes = view().encode().unwrap();
        for length in 0..bytes.len() {
            assert!(State::decode(&bytes[..length]).is_err(), "prefix {length}");
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(State::decode(&extra).is_err());
        let mut wrong_tag = bytes;
        wrong_tag[0] ^= 1;
        assert!(State::decode(&wrong_tag).is_err());
        assert!(State::decode(&vec![0; MAX_BYTES + 1]).is_err());
    }
    #[test]
    fn malformed_names_lengths_counts_and_zero_headers_fail_closed() {
        let valid = view().encode().unwrap();
        for name in [
            b"Ar--n".as_slice(),
            b"Ard2n",
            b"Ard n",
            &[0xff, b'r', b'd', b'e', b'n'],
        ] {
            let mut bytes = valid.clone();
            bytes[HEADER_BYTES + ROW_BYTES..].copy_from_slice(name);
            assert!(State::decode(&bytes).is_err());
        }
        for length in [1, 2, 21, 255] {
            let mut bytes = valid.clone();
            bytes[HEADER_BYTES + 8] = length;
            assert!(State::decode(&bytes).is_err());
        }
        let mut count = valid.clone();
        count[52] = MAX_ACTORS as u8 + 1;
        assert!(State::decode(&count).is_err());
        for range in [4..12, 12..20, 20..52, HEADER_BYTES..HEADER_BYTES + 8] {
            let mut bytes = valid.clone();
            bytes[range].fill(0);
            assert!(State::decode(&bytes).is_err());
        }
    }
    #[test]
    fn unordered_duplicate_foreign_epoch_and_excess_rows_are_rejected() {
        for actors in [
            vec![actor(2, None), actor(1, None)],
            vec![actor(1, None), actor(1, None)],
            vec![actor(1, None); MAX_ACTORS + 1],
        ] {
            assert!(State { actors, ..view() }.encode().is_err());
        }
        let mut foreign = view();
        foreign.actors[0].key.epoch = WorldEpoch::new(8).unwrap();
        assert!(foreign.encode().is_err());
        let mut bytes = State {
            actors: vec![actor(1, None), actor(2, None)],
            ..view()
        }
        .encode()
        .unwrap();
        bytes[HEADER_BYTES + ROW_BYTES..HEADER_BYTES + ROW_BYTES + 8]
            .copy_from_slice(&1u64.to_le_bytes());
        assert!(State::decode(&bytes).is_err());
    }
    #[test]
    fn admission_and_monotonicity_checks_preserve_the_valid_context() {
        let old = view();
        let owner = old.actors[0].key;
        old.validate_successor(owner, old.world, None).unwrap();
        old.validate_successor(owner, old.world, Some(&old))
            .unwrap();
        let mut next = old.clone();
        next.tick += 1;
        next.actors[0].name = Some(CharacterName::parse("Ar'den").unwrap());
        next.validate_successor(owner, old.world, Some(&old))
            .unwrap();
        assert!(
            old.validate_successor(owner, old.world, Some(&next))
                .is_err()
        );
        next.tick = old.tick;
        assert!(
            next.validate_successor(owner, old.world, Some(&old))
                .is_err()
        );
        assert!(
            old.validate_successor(actor(2, None).key, old.world, None)
                .is_err()
        );
        assert!(
            old.validate_successor(
                EntityKey {
                    epoch: WorldEpoch::new(8).unwrap(),
                    ..owner
                },
                old.world,
                None
            )
            .is_err()
        );
        assert!(old.validate_successor(owner, [4; 32], None).is_err());
        assert!(
            State {
                actors: vec![],
                ..old.clone()
            }
            .validate_successor(owner, old.world, None)
            .is_err()
        );
        let mut foreign = old.clone();
        foreign.world = [4; 32];
        assert!(
            old.validate_successor(owner, old.world, Some(&foreign))
                .is_err()
        );
    }
    #[test]
    fn stable_rosters_ignore_movement_ticks_but_detect_name_and_removal_changes() {
        let old = view();
        let mut next = old.clone();
        next.tick += 1;
        assert!(old.same_view(&next));
        next.actors[0].name = None;
        assert!(!old.same_view(&next));
        next.actors.clear();
        assert!(!old.same_view(&next));
    }
}
