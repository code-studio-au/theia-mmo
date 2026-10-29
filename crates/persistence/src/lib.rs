//! Async PostgreSQL boundary. No renderer, transport or simulation dependencies.
//! Queries and commit acknowledgements belong to an owner worker, never World::step.
mod characters;
mod store;
mod worker;
pub use characters::{Character, CharacterName, CharacterSummary, MAX_CHARACTERS_PER_OWNER};
use std::fmt;
pub use store::Store;
use theia_core::coordinates::GlobalPosition;
use uuid::Uuid;
pub use worker::{Publisher, Worker, WorkerStatus};

pub type Result<T> = std::result::Result<T, Error>;
pub enum Error {
    Invalid(&'static str),
    Database(tokio_postgres::Error),
    Timeout,
    RealmBusy,
    Ownership,
    WorldMismatch,
    Migration,
    Durability,
    NameUnavailable,
    CharacterLimit,
    CharacterMissing,
    AccountMissing,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Connection strings/passwords and SQL parameters are never formatted.
        f.write_str(match self {
            Self::Invalid(message) => message,
            Self::Database(_) => "PostgreSQL operation failed",
            Self::Timeout => "PostgreSQL operation timed out",
            Self::RealmBusy => "Persistent realm already has an owner",
            Self::Ownership => "Persistent character owner mismatch",
            Self::WorldMismatch => "Persistent character world content mismatch",
            Self::Migration => "Persistence migration identity/version mismatch",
            Self::Durability => "PostgreSQL durability settings do not satisfy the contract",
            Self::NameUnavailable => "Character name unavailable in this realm",
            Self::CharacterLimit => "Character limit reached in this realm",
            Self::CharacterMissing => "Character unavailable",
            Self::AccountMissing => "Persistent account owner missing",
        })
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for Error {}
impl From<tokio_postgres::Error> for Error {
    fn from(value: tokio_postgres::Error) -> Self {
        Self::Database(value)
    }
}
macro_rules! persistent_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(Uuid);
        impl $name {
            pub fn new(value: Uuid) -> Option<Self> {
                (!value.is_nil()).then_some(Self(value))
            }
            pub fn parse(value: &str) -> Option<Self> {
                Self::new(Uuid::parse_str(value).ok()?)
            }
            pub fn random() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn value(self) -> Uuid {
                self.0
            }
        }
    };
}
persistent_id!(RealmId);
persistent_id!(CharacterId);
persistent_id!(OwnerId);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub character: CharacterId,
    pub owner: OwnerId,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedCharacter {
    pub binding: Binding,
    pub world: [u8; 32],
    /// Authoritative global body-centre position, Y-up metres, checked f64.
    pub position: GlobalPosition,
    pub sequence: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Saved {
    Committed,
    Stale,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn loopback_configuration_rejects_overriding_remote_addresses() {
        for connection in [
            "host=127.0.0.1 hostaddr=203.0.113.1 password=never-log-this",
            "host=::1 hostaddr=2001:db8::1 password=never-log-this",
            "host=127.0.0.1 hostaddr=127.0.0.1,127.0.0.2 password=never-log-this",
        ] {
            let error = Store::connect_loopback(connection, RealmId::random())
                .await
                .err()
                .unwrap();
            assert!(matches!(error, Error::Invalid(_)));
            assert!(!format!("{error:?}").contains("never-log-this"));
        }
    }
    #[test]
    fn persistent_ids_reject_nil_and_keep_domains_distinct() {
        assert!(RealmId::new(Uuid::nil()).is_none());
        assert!(CharacterId::new(Uuid::nil()).is_none());
        assert!(OwnerId::new(Uuid::nil()).is_none());
        let character = CharacterId::random();
        assert_eq!(CharacterId::new(character.value()), Some(character));
        assert_ne!(character, CharacterId::random());
        assert_eq!(
            Error::Ownership.to_string(),
            "Persistent character owner mismatch"
        );
    }
}
