//! Process-incarnation network identity; never a persistent account/character ID.
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldEpoch(NonZeroU64);
impl WorldEpoch {
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
    pub fn value(self) -> u64 {
        self.0.get()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(NonZeroU64);
impl EntityId {
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
    pub fn value(self) -> u64 {
        self.0.get()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityKey {
    pub epoch: WorldEpoch,
    pub entity: EntityId,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incarnations_disambiguate_reused_entity_numbers() {
        assert!(WorldEpoch::new(0).is_none());
        assert!(EntityId::new(0).is_none());
        let entity = EntityId::new(1).unwrap();
        assert_ne!(
            EntityKey {
                epoch: WorldEpoch::new(1).unwrap(),
                entity
            },
            EntityKey {
                epoch: WorldEpoch::new(2).unwrap(),
                entity
            }
        );
    }
}
