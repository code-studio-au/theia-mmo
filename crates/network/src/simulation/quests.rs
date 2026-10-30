//! The initial objective consumes the same trusted XP kill facts and queue.
use super::World;
use crate::{Result, quest_wire::Profile};
use std::sync::Arc;
impl World {
    pub fn enable_quest(&mut self, profile: Arc<Profile>) -> Result<()> {
        if self.tick != 0 || !self.players.is_empty() || self.quest.is_some() {
            return Err("Objectives require inactive world".into());
        }
        let xp = self
            .progression
            .as_ref()
            .ok_or("Objectives require progression")?;
        let patrol = self
            .patrol
            .as_ref()
            .ok_or("Objectives require authoritative creature")?;
        if profile.profile().creature() != patrol.definition()
            || xp
                .profile()
                .kill_experience(profile.profile().creature())
                .is_none()
        {
            return Err("Objective/progression/creature definitions differ".into());
        }
        self.quest = Some(profile);
        Ok(())
    }
    pub fn quest_profile(&self) -> Option<&Arc<Profile>> {
        self.quest.as_ref()
    }
}
