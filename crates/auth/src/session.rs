//! Bounded process-local account sessions. These do not mint transport tickets or
//! bypass character ownership. One active session per owner is the initial policy.
use crate::{Error, Result, VerifiedIdentity};
use ring::{
    digest::{SHA256, digest},
    rand::{SecureRandom, SystemRandom},
};
use std::{
    collections::BTreeMap,
    fmt,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const CAPACITY: usize = 32;
const MAX_AGE: Duration = Duration::from_secs(300);
/// Opaque bearer credential. Never serialize/debug it implicitly; the HTTP
/// adapter must deliberately return it only to the owning login challenge.
pub struct SessionSecret(String);
/// Server-created session context, not a bearer credential. Private fields and
/// no deserializer prevent a client from inventing an authenticated owner.
#[derive(Clone)]
pub struct SessionAuthorization<Owner> {
    key: [u8; 32],
    owner: Owner,
}
impl<Owner> SessionAuthorization<Owner> {
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
}
impl<Owner> fmt::Debug for SessionAuthorization<Owner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionAuthorization([redacted])")
    }
}
impl SessionSecret {
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for SessionSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionSecret([redacted])")
    }
}
#[derive(Clone, Debug, Default)]
pub struct SessionStats {
    pub issued: u64,
    pub rejected: u64,
    pub expired: u64,
    pub revoked: u64,
    pub capacity_rejections: u64,
}
struct Entry<Owner> {
    owner: Owner,
    deadline: Instant,
}
pub struct Sessions<Owner> {
    entries: BTreeMap<[u8; 32], Entry<Owner>>,
    stats: SessionStats,
}
impl<Owner: Clone + Eq> Default for Sessions<Owner> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            stats: SessionStats::default(),
        }
    }
}
impl<Owner: Clone + Eq> Sessions<Owner> {
    pub fn stats(&self) -> &SessionStats {
        &self.stats
    }
    pub fn active(&mut self) -> usize {
        self.expire_at(Instant::now());
        self.entries.len()
    }
    /// Owner must come from the server's durable (issuer, subject) mapping.
    /// Successful login supersedes the owner's previous account credential.
    pub fn issue(&mut self, identity: &VerifiedIdentity, owner: Owner) -> Result<SessionSecret> {
        let unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Session)?
            .as_secs()
            // Round conservatively so a session cannot outlive the verified
            // token because the wall-clock sample had a fractional second.
            .checked_add(1)
            .ok_or(Error::Session)?;
        self.issue_at(identity, owner, Instant::now(), unix)
    }
    fn issue_at(
        &mut self,
        identity: &VerifiedIdentity,
        owner: Owner,
        now: Instant,
        unix: u64,
    ) -> Result<SessionSecret> {
        let remaining = identity
            .expires_unix
            .checked_sub(unix)
            .filter(|n| *n > 0)
            .ok_or(Error::Session)?;
        self.expire_at(now);
        if self.entries.len() == CAPACITY
            && !self.entries.values().any(|entry| entry.owner == owner)
        {
            self.stats.capacity_rejections = self.stats.capacity_rejections.saturating_add(1);
            return Err(Error::Capacity);
        }
        let mut random = [0; 32];
        SystemRandom::new()
            .fill(&mut random)
            .map_err(|_| Error::Session)?;
        let key = key(&random);
        if self.entries.contains_key(&key) {
            return Err(Error::Session);
        }
        self.revoke_owner(&owner);
        let deadline = now + MAX_AGE.min(Duration::from_secs(remaining));
        self.entries.insert(key, Entry { owner, deadline });
        self.stats.issued = self.stats.issued.saturating_add(1);
        Ok(SessionSecret(
            random.iter().map(|b| format!("{b:02x}")).collect(),
        ))
    }
    pub fn authenticate(&mut self, credential: &str) -> Result<Owner> {
        self.authenticate_at(credential, Instant::now())
    }
    pub fn authorize(&mut self, credential: &str) -> Result<SessionAuthorization<Owner>> {
        self.expire_at(Instant::now());
        if let Some((key, entry)) =
            decode_key(credential).and_then(|key| self.entries.get(&key).map(|entry| (key, entry)))
        {
            return Ok(SessionAuthorization {
                key,
                owner: entry.owner.clone(),
            });
        }
        self.stats.rejected = self.stats.rejected.saturating_add(1);
        Err(Error::Session)
    }
    /// Recheck after awaited owner/character work and before granting access.
    pub fn validate(&mut self, authorization: &SessionAuthorization<Owner>) -> Result<Owner> {
        self.expire_at(Instant::now());
        if let Some(entry) = self.entries.get(&authorization.key)
            && entry.owner == authorization.owner
        {
            return Ok(entry.owner.clone());
        }
        self.stats.rejected = self.stats.rejected.saturating_add(1);
        Err(Error::Session)
    }
    fn authenticate_at(&mut self, credential: &str, now: Instant) -> Result<Owner> {
        self.expire_at(now);
        if let Some(entry) = decode_key(credential).and_then(|key| self.entries.get(&key)) {
            return Ok(entry.owner.clone());
        }
        self.stats.rejected = self.stats.rejected.saturating_add(1);
        Err(Error::Session)
    }
    pub fn revoke(&mut self, credential: &str) {
        if decode_key(credential).is_some_and(|key| self.entries.remove(&key).is_some()) {
            self.stats.revoked = self.stats.revoked.saturating_add(1);
        }
    }
    pub fn revoke_owner(&mut self, owner: &Owner) {
        let before = self.entries.len();
        self.entries.retain(|_, entry| &entry.owner != owner);
        self.stats.revoked = self
            .stats
            .revoked
            .saturating_add((before - self.entries.len()) as u64);
    }
    fn expire_at(&mut self, now: Instant) {
        let before = self.entries.len();
        self.entries.retain(|_, entry| entry.deadline > now);
        self.stats.expired = self
            .stats
            .expired
            .saturating_add((before - self.entries.len()) as u64);
    }
}
fn key(bytes: &[u8]) -> [u8; 32] {
    digest(&SHA256, bytes).as_ref().try_into().unwrap()
}
fn decode_key(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut bytes = [0; 32];
    for (i, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let digit = |b| match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            _ => None,
        };
        bytes[i] = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Some(key(&bytes))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn identity(expires_unix: u64) -> VerifiedIdentity {
        VerifiedIdentity {
            issuer: "http://127.0.0.1:123/realm".into(),
            subject: "unit-subject".into(),
            expires_unix,
        }
    }
    #[test]
    fn credentials_expire_revoke_and_supersede_without_leaking_secrets() {
        let now = Instant::now();
        let mut sessions = Sessions::default();
        let id = identity(1000);
        let first = sessions.issue_at(&id, 7u64, now, 100).unwrap();
        assert_eq!(
            sessions
                .authenticate_at(first.expose_secret(), now)
                .unwrap(),
            7
        );
        let second = sessions.issue_at(&id, 7, now, 100).unwrap();
        assert_ne!(first.expose_secret(), second.expose_secret());
        assert!(
            sessions
                .authenticate_at(first.expose_secret(), now)
                .is_err()
        );
        assert_eq!(sessions.active(), 1);
        assert!(!format!("{second:?}").contains(second.expose_secret()));
        assert!(
            sessions
                .authenticate_at(second.expose_secret(), now + MAX_AGE)
                .is_err()
        );
        let third = sessions.issue_at(&identity(110), 8, now, 100).unwrap();
        assert!(
            sessions
                .authenticate_at(third.expose_secret(), now + Duration::from_secs(10))
                .is_err()
        );
        let fourth = sessions.issue_at(&id, 9, now, 100).unwrap();
        sessions.revoke(fourth.expose_secret());
        assert!(
            sessions
                .authenticate_at(fourth.expose_secret(), now)
                .is_err()
        );
        assert_eq!(sessions.stats().expired, 2);
        assert_eq!(sessions.stats().revoked, 2);
        assert!(sessions.issue_at(&identity(100), 1, now, 100).is_err());
    }
    #[test]
    fn opaque_server_contexts_revalidate_owner_revocation_and_replacement() {
        let now = Instant::now();
        let mut sessions = Sessions::default();
        let identity = identity(1000);
        let first = sessions.issue_at(&identity, 7u64, now, 100).unwrap();
        let context = sessions.authorize(first.expose_secret()).unwrap();
        assert_eq!(*context.owner(), 7);
        assert_eq!(sessions.validate(&context).unwrap(), 7);
        let forged = SessionAuthorization {
            key: context.key,
            owner: 8,
        };
        assert!(sessions.validate(&forged).is_err());
        assert!(!format!("{context:?}").contains(first.expose_secret()));
        let replacement = sessions.issue_at(&identity, 7, now, 100).unwrap();
        assert!(sessions.validate(&context).is_err());
        let current = sessions.authorize(replacement.expose_secret()).unwrap();
        sessions.revoke(replacement.expose_secret());
        assert!(sessions.validate(&current).is_err());
    }
    #[test]
    fn capacity_rejects_new_owners_without_evicting_live_sessions_and_inputs_are_bounded() {
        let now = Instant::now();
        let mut sessions = Sessions::default();
        let id = identity(1000);
        let mut credentials = Vec::new();
        for owner in 0..CAPACITY {
            credentials.push(sessions.issue_at(&id, owner, now, 100).unwrap());
        }
        assert!(matches!(
            sessions.issue_at(&id, CAPACITY, now, 100),
            Err(Error::Capacity)
        ));
        for (owner, credential) in credentials.iter().enumerate() {
            assert_eq!(
                sessions
                    .authenticate_at(credential.expose_secret(), now)
                    .unwrap(),
                owner
            );
        }
        for bad in [
            "x".repeat(100000),
            "0".repeat(64),
            "F".repeat(64),
            "é".repeat(32),
        ] {
            assert!(sessions.authenticate_at(&bad, now).is_err());
        }
        let replacement = sessions.issue_at(&id, 0, now, 100).unwrap();
        assert_eq!(sessions.active(), CAPACITY);
        sessions.revoke_owner(&0);
        assert!(
            sessions
                .authenticate_at(replacement.expose_secret(), now)
                .is_err()
        );
        assert_eq!(sessions.active(), CAPACITY - 1);
        assert!(sessions.issue_at(&id, CAPACITY, now, 100).is_ok());
    }
}
