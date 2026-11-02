//! Independent authoritative character heading through the existing command clock.
use super::World;
impl World {
    pub fn enable_classic(&mut self) -> crate::Result<()> {
        if !self.predicted
            || self.tick != 0
            || !self.players.is_empty()
            || self.classic
            || self.dodge.is_some()
            || self.dev_tour
            || self.terrain_teleport.is_some()
            || self.procedural.as_ref().is_none_or(|r| r.speed != 1.)
        {
            return Err("Classic facing requires inactive normal-speed predicted terrain".into());
        }
        self.classic = true;
        Ok(())
    }
    pub fn classic_enabled(&self) -> bool {
        self.classic
    }
}
