//! The initial objective consumes the same trusted XP kill facts and queue.
use super::World;
use crate::{Result, quest_wire::Profile};
use std::sync::Arc;
impl World {
    pub fn enable_quest(&mut self, profile: Arc<Profile>) -> Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.quest.is_some()
            || self.quest_lifecycle.is_some()
        {
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

    pub fn enable_quest_lifecycle(
        &mut self,
        profile: Arc<crate::quest_lifecycle_wire::Profile>,
    ) -> Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.quest.is_some()
            || self.quest_lifecycle.is_some()
            || self.dialogue.is_none()
        {
            return Err(
                "Explicit quests require inactive guide world and exclusive quest mode".into(),
            );
        }
        let xp = self
            .progression
            .as_ref()
            .ok_or("Explicit quests require progression")?;
        let patrol = self
            .patrol
            .as_ref()
            .ok_or("Explicit quests require creature authority")?;
        profile.check_progression(xp)?;
        if profile.profile().creature() != patrol.definition()
            || xp
                .profile()
                .kill_experience(profile.profile().creature())
                .is_none()
        {
            return Err("Explicit quest/progression/creature definitions differ".into());
        }
        self.quest_lifecycle = Some(profile);
        Ok(())
    }

    pub fn quest_lifecycle_profile(&self) -> Option<&Arc<crate::quest_lifecycle_wire::Profile>> {
        self.quest_lifecycle.as_ref()
    }

    /// Admission-time entitlement, never a durable success check. Accepted work
    /// must retain its original intent through later close/walkaway/logout.
    /// This initial quest belongs to the world's sole informational guide.
    pub fn quest_lifecycle_eligible(
        &self,
        actor: u64,
        request: crate::quest_lifecycle_wire::Request,
    ) -> bool {
        self.quest_lifecycle.is_some()
            && self.entity_key(actor) == Some(request.actor)
            && self.guide_interaction_eligible(actor, request.dialogue_revision, request.npc)
    }
}
