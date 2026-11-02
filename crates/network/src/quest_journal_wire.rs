//! Bounded owner-private journal observations. Catalogue indices are meaningful
//! only under the exact validated profile digest. No credit upload command.
use crate::{Result, quest_journal::Profile};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::{
        catalog::{MAX_QUESTS, MAX_TOTAL_OBJECTIVES},
        journal::{Journal, Progress, Status},
    },
};
pub const STATE_TAG: &[u8; 4] = b"QJ01";
pub const HEADER_BYTES: usize = 61;
/// Header + at most 32 metadata records + at most 512 one-byte counts.
pub const MAX_STATE_BYTES: usize = HEADER_BYTES + MAX_QUESTS * 11 + MAX_TOTAL_OBJECTIVES;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    pub revision: u64,
    pub journal: Journal,
}
fn status_code(status: Status) -> u8 {
    match status {
        Status::Active => 1,
        Status::Ready => 2,
        Status::Completed => 3,
    }
}
impl State {
    fn validate(&self, profile: &Profile) -> Result<()> {
        if self.profile_hash != profile.hash()
            || self.profile_hash == [0; 32]
            || self.revision > i64::MAX as u64
            || self
                .journal
                .quests()
                .any(|(_, q)| q.revision() > self.revision)
        {
            return Err("Invalid journal observation binding/revision".into());
        }
        self.journal
            .validate(profile.catalog())
            .map_err(|_| "Invalid journal observation progress")?;
        Ok(())
    }
    pub fn encode(&self, profile: &Profile) -> Result<Vec<u8>> {
        self.validate(profile)?;
        let mut bytes = STATE_TAG.to_vec();
        bytes.extend(self.actor.epoch.value().to_le_bytes());
        bytes.extend(self.actor.entity.value().to_le_bytes());
        bytes.extend(self.profile_hash);
        bytes.extend(self.revision.to_le_bytes());
        bytes.push(u8::try_from(self.journal.quests().count())?);
        for (quest, progress) in self.journal.quests() {
            let index = profile
                .catalog()
                .quests()
                .position(|q| q.id() == quest)
                .ok_or("Journal quest missing")?;
            let definition = profile.catalog().quest(quest).unwrap();
            let stage = definition
                .stages()
                .position(|s| s.id() == progress.stage())
                .ok_or("Journal stage missing")?;
            bytes.push(u8::try_from(index)?);
            bytes.extend(progress.revision().to_le_bytes());
            bytes.push(status_code(progress.status()));
            bytes.push(u8::try_from(stage)?);
            for objective in definition.stages().flat_map(|s| s.objectives()) {
                bytes.push(u8::try_from(
                    progress
                        .count(objective.id())
                        .ok_or("Journal objective missing")?,
                )?);
            }
        }
        if bytes.len() > MAX_STATE_BYTES {
            return Err("Journal observation exceeds frame budget".into());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], profile: &Profile) -> Result<Self> {
        if !(HEADER_BYTES..=MAX_STATE_BYTES).contains(&bytes.len()) || !bytes.starts_with(STATE_TAG)
        {
            return Err("Invalid journal observation frame".into());
        }
        let mut reader = Reader { bytes, at: 4 };
        let actor = EntityKey {
            epoch: WorldEpoch::new(reader.word()?).ok_or("Invalid journal epoch")?,
            entity: EntityId::new(reader.word()?).ok_or("Invalid journal actor")?,
        };
        let profile_hash = reader.take(32)?.try_into().unwrap();
        if profile_hash != profile.hash() {
            return Err("Journal observation profile differs".into());
        }
        let revision = reader.word()?;
        let count = usize::from(reader.byte()?);
        if count > MAX_QUESTS {
            return Err("Journal quest count exceeds budget".into());
        }
        let definitions: Vec<_> = profile.catalog().quests().collect();
        let mut prior = None;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let index = usize::from(reader.byte()?);
            if prior.is_some_and(|old| index <= old) {
                return Err("Noncanonical journal quest order".into());
            }
            prior = Some(index);
            let definition = definitions
                .get(index)
                .ok_or("Unknown journal quest index")?;
            let quest_revision = reader.word()?;
            let status = match reader.byte()? {
                1 => Status::Active,
                2 => Status::Ready,
                3 => Status::Completed,
                _ => return Err("Invalid journal status".into()),
            };
            let stage = definition
                .stages()
                .nth(usize::from(reader.byte()?))
                .ok_or("Unknown journal stage index")?
                .id()
                .clone();
            let mut counts = Vec::new();
            for objective in definition.stages().flat_map(|s| s.objectives()) {
                counts.push((objective.id().clone(), u32::from(reader.byte()?)));
            }
            let progress = Progress::from_parts(definition, quest_revision, status, stage, counts)
                .map_err(|_| "Invalid journal quest progress")?;
            entries.push((definition.id().clone(), progress));
        }
        if reader.at != bytes.len() {
            return Err("Trailing journal observation bytes".into());
        }
        let state = Self {
            actor,
            profile_hash,
            revision,
            journal: Journal::from_progress(profile.catalog(), entries)
                .map_err(|_| "Invalid journal observation entries")?,
        };
        state.validate(profile)?;
        Ok(state)
    }
    pub fn validate_successor(
        &self,
        profile: &Profile,
        actor: EntityKey,
        prior: Option<&Self>,
    ) -> Result<()> {
        self.validate(profile)?;
        if self.actor != actor {
            return Err("Foreign journal actor observation".into());
        }
        if let Some(old) = prior {
            old.validate(profile)?;
            if old.actor != actor
                || self.profile_hash != old.profile_hash
                || self.revision < old.revision
                || (self.revision == old.revision && self != old)
            {
                return Err("Regressed or conflicting journal observation".into());
            }
            for (id, before) in old.journal.quests() {
                let after = self
                    .journal
                    .progress(id)
                    .ok_or("Journal history disappeared")?;
                if after.revision() < before.revision()
                    || status_code(after.status()) < status_code(before.status())
                    || (after.revision() == before.revision() && after != before)
                    || (before.status() == Status::Completed && after != before)
                {
                    return Err("Journal quest progress regressed".into());
                }
                for objective in profile
                    .catalog()
                    .quest(id)
                    .unwrap()
                    .stages()
                    .flat_map(|s| s.objectives())
                {
                    if after.count(objective.id()) < before.count(objective.id())
                        || (before.status() == Status::Ready
                            && after.count(objective.id()) != before.count(objective.id()))
                    {
                        return Err("Journal objective progress regressed".into());
                    }
                }
            }
        }
        Ok(())
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .ok_or("Journal frame offset overflow")?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or("Truncated journal observation")?;
        self.at = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn word(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
}
