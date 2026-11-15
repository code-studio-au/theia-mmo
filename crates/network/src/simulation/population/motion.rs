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
    roaming: Vec<Vec<GlobalPosition>>,
    excursion: usize,
    waypoint: usize,
    wander_after: u64,
    wander_cycle: u64,
}
impl Actor {
    pub fn new(
        key: EntityKey,
        position: GlobalPosition,
        patch: Arc<Patch>,
        roaming: Vec<Vec<GlobalPosition>>,
    ) -> Result<Self> {
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
            roaming,
            excursion: 0,
            waypoint: 0,
            wander_after: 0,
            wander_cycle: 0,
        })
    }
    pub fn respawn(&mut self, key: EntityKey) -> GlobalPosition {
        self.key = key;
        self.position = self.home;
        self.facing = Facing::default();
        self.body = kinematic::Motion::default();
        self.blocked = false;
        self.waypoint = 0;
        self.wander_after = 0;
        self.home
    }
    pub(super) fn position(&self) -> GlobalPosition {
        self.position
    }
    pub fn advance(
        &mut self,
        world: &mut World,
        mut directive: PursuitMotion,
        neighbours: &[(EntityKey, GlobalPosition)],
    ) {
        if directive.mode != Mode::Patrol {
            // Combat returns to home, not the interrupted waypoint. Restart the
            // checked excursion there rather than cutting across its corners.
            self.waypoint = 0;
            self.wander_after = 0;
        }
        if !world.targets.contains_key(&self.key.entity.value()) || directive.mode == Mode::Dead {
            self.body = kinematic::Motion::default();
            return;
        }
        if directive.mode == Mode::Blocked {
            self.blocked = true;
            return;
        }
        self.blocked = false;
        if directive.mode == Mode::Patrol && !self.roaming.is_empty() {
            if self.wander_after == 0 {
                self.wander_after = world
                    .tick
                    .saturating_add(u64::from(world.rate.hz()) * (2 + self.key.entity.value() % 5));
            }
            if world.tick >= self.wander_after {
                let route = &self.roaming[self.excursion];
                while self.waypoint < route.len() {
                    let mut point = route[self.waypoint].metres();
                    point[1] += f64::from(crate::world::PAWN_Y);
                    let p = self.position.metres();
                    if (p[0] - point[0]).hypot(p[2] - point[2]) > 0.1 {
                        directive.goal = GlobalPosition::new(point);
                        directive.speed = 1.5;
                        break;
                    }
                    self.waypoint += 1;
                }
                if self.waypoint == route.len() {
                    self.wander_cycle = self.wander_cycle.wrapping_add(1);
                    let random = self
                        .key
                        .entity
                        .value()
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(self.wander_cycle.wrapping_mul(1442695040888963407));
                    self.excursion = (random as usize) % self.roaming.len();
                    self.waypoint = 0;
                    self.wander_after = world
                        .tick
                        .saturating_add(u64::from(world.rate.hz()) * (2 + random % 5));
                }
            }
        }
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
        // Soft steering, never actor collision or displacement of another body.
        // Only steer an established chase route or an attacking creature; return
        // paths, dead actors and unavailable navigation keep their old behavior.
        if directive.mode == Mode::Attack
            || (directive.mode == Mode::Chase && directive.goal.is_some())
        {
            let push = separation(self.key, self.position, neighbours);
            let base = intent.axes.map(|v| f64::from(v) / 200.);
            let velocity = [base[0] + push[0], base[1] + push[1]];
            let speed = velocity[0].hypot(velocity[1]);
            let limit = if directive.mode == Mode::Attack {
                1.0
            } else {
                f64::from(directive.speed)
            };
            let scale = if speed > limit { limit / speed } else { 1. };
            intent.axes = velocity.map(|v| (v * scale * 200.).round() as i16);
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

// Theia crowd-readability policy: 1.6 m preferred spacing, at most 1 m/s
// avoidance. Snapshot neighbours ensure deterministic, order-independent intent.
fn separation(
    key: EntityKey,
    position: GlobalPosition,
    neighbours: &[(EntityKey, GlobalPosition)],
) -> [f64; 2] {
    let p = position.metres();
    let mut force = [0.; 2];
    for (other, position) in neighbours {
        if *other == key {
            continue;
        }
        let q = position.metres();
        if (p[1] - q[1]).abs() > 1.5 {
            continue;
        }
        let mut delta = [p[0] - q[0], p[2] - q[2]];
        let distance = delta[0].hypot(delta[1]);
        if distance >= 1.6 {
            continue;
        }
        if distance < 0.0001 {
            let low = key.entity.value().min(other.entity.value());
            let high = key.entity.value().max(other.entity.value());
            let angle = ((low.wrapping_mul(31) ^ high.wrapping_mul(17)) % 360) as f64
                * std::f64::consts::PI
                / 180.;
            let sign = if key.entity.value() < other.entity.value() {
                1.
            } else {
                -1.
            };
            delta = [angle.cos() * sign, angle.sin() * sign];
        } else {
            delta = delta.map(|v| v / distance);
        }
        let weight = (1.6 - distance) / 1.6;
        for i in 0..2 {
            force[i] += delta[i] * weight;
        }
    }
    let length = force[0].hypot(force[1]);
    if length > 1. {
        force = force.map(|v| v / length);
    }
    force
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::identity::{EntityId, WorldEpoch};
    #[test]
    fn interrupted_roaming_restarts_at_home_without_skipping_checked_waypoints() {
        let (mut world, key) =
            crate::simulation::creatures::tests::fixture(theia_core::time::TickRate::Hz30);
        let home = GlobalPosition::new([8006., 1., 8000.]).unwrap();
        let patch = world
            .procedural
            .as_ref()
            .unwrap()
            .patch(home)
            .unwrap()
            .clone();
        let mut actor = Actor::new(key, home, patch, vec![vec![home; 4]]).unwrap();
        actor.waypoint = 2;
        actor.wander_after = 1;
        actor.advance(
            &mut world,
            PursuitMotion {
                mode: Mode::Return,
                goal: None,
                speed: 0.,
                look_at: None,
            },
            &[],
        );
        assert_eq!(actor.waypoint, 0);
        assert_eq!(actor.wander_after, 0);
    }

    #[test]
    fn overlap_separates_symmetrically_and_force_is_bounded() {
        let key = |id| EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        };
        let at = GlobalPosition::new([1e9, 0.9, -1e9]).unwrap();
        let neighbours = vec![(key(1), at), (key(2), at)];
        let a = separation(key(1), at, &neighbours);
        let b = separation(key(2), at, &neighbours);
        assert!((a[0] + b[0]).abs() < 1e-9 && (a[1] + b[1]).abs() < 1e-9);
        assert!((a[0].hypot(a[1]) - 1.).abs() < 1e-9);
        let crowded: Vec<_> = (1..=16).map(|i| (key(i), at)).collect();
        let force = separation(key(1), at, &crowded);
        assert!(force[0].hypot(force[1]) <= 1.000001);
        let remote = GlobalPosition::new([1e9 + 2., 0.9, -1e9]).unwrap();
        assert_eq!(separation(key(1), at, &[(key(2), remote)]), [0.; 2]);
    }
}
