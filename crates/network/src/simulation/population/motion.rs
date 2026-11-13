//! Independent nonblocking creature bodies using the ordinary checked controller.
use super::super::{
    World,
    creatures::{PursuitMotion, motion},
};
use crate::{Result, chunks::Coverage, kinematic, procedural::Patch, wire::Intent};
use std::sync::Arc;
use theia_core::{coordinates::GlobalPosition, facing::Facing, identity::EntityKey, pursuit::Mode};

pub(in crate::simulation) struct Actor {
    pub key: EntityKey,
    pub blocked: bool,
    pub facing: Facing,
    home: GlobalPosition,
    position: GlobalPosition,
    body: kinematic::Motion,
    patch: Arc<Patch>,
    coverage: Coverage,
}
impl Actor {
    pub fn new(key: EntityKey, position: GlobalPosition, patch: Arc<Patch>) -> Result<Self> {
        let coverage = Coverage::procedural_cells(patch.frame, patch.ids.iter().copied())?;
        Ok(Self {
            key,
            blocked: false,
            facing: Facing::default(),
            home: position,
            position,
            body: kinematic::Motion::default(),
            patch,
            coverage,
        })
    }
    pub fn respawn(&mut self, key: EntityKey) -> GlobalPosition {
        self.key = key;
        self.position = self.home;
        self.facing = Facing::default();
        self.body = kinematic::Motion::default();
        self.blocked = false;
        self.home
    }
    pub fn advance(&mut self, world: &mut World, directive: PursuitMotion) {
        if !world.targets.contains_key(&self.key.entity.value()) || directive.mode == Mode::Dead {
            self.body = kinematic::Motion::default();
            return;
        }
        if directive.mode == Mode::Blocked {
            self.blocked = true;
            return;
        }
        self.blocked = false;
        let mut intent = Intent::default();
        let mut distance = 0.;
        if let Some(goal) = directive.goal {
            let a = self.position.metres();
            let b = goal.metres();
            let dx = b[0] - a[0];
            let dz = b[2] - a[2];
            distance = dx.hypot(dz);
            let seconds = f64::from(world.rate.seconds());
            let scale = distance.min(f64::from(directive.speed) * seconds) / (5. * seconds);
            if distance > 0.000001 {
                intent.axes = [
                    (dx / distance * scale * 1000.).round() as i16,
                    (dz / distance * scale * 1000.).round() as i16,
                ];
            }
        }
        let before = self.body;
        let Ok(position) = motion::step(
            &self.patch,
            &self.coverage,
            &mut self.body,
            self.position,
            intent,
            world.rate,
        ) else {
            self.blocked = true;
            return;
        };
        let a = self.position.metres();
        let b = position.metres();
        if directive.goal.is_some() && distance > 0.01 && (b[0] - a[0]).hypot(b[2] - a[2]) < 0.00001
        {
            self.body = before;
            self.blocked = true;
            return;
        }
        if !world.move_creature_target(self.key, position) {
            self.body = before;
            return;
        }
        if let Some(facing) = directive
            .look_at
            .and_then(|target| Facing::towards(position, target))
            .or_else(|| {
                directive
                    .goal
                    .and_then(|_| Facing::towards(self.position, position))
            })
        {
            self.facing = facing;
        }
        self.position = position;
    }
}
