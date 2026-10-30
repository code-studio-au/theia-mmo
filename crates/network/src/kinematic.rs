//! Shared fixed-step capsule movement. The scene contains static world only;
//! actors are query shapes and never become blockers for another actor.
use crate::{
    physics::{GROUND, SOLID, StaticScene},
    wire::Intent,
};
use avian3d::prelude::Collider;
use bevy::prelude::{Vec3, Vec3Swizzles};
use theia_core::{coordinates::LocalPosition, time::TickRate};
const RADIUS: f32 = 0.35;
pub const FOOTPRINT_RADIUS: f32 = RADIUS + SKIN;
const HEIGHT: f32 = 1.8;
const SKIN: f32 = 0.005;
const CONTACT_PAD: f32 = 0.003;
const STEP: f32 = 0.35;
const SNAP: f32 = 0.10;
const GRAVITY: f32 = 20.;
const JUMP_SPEED: f32 = 7.;
const MIN_UP: f32 = 0.76604444; // cos(40 degrees)
const WORLD: u32 = SOLID | GROUND;
const MAX_SLIDES: usize = 5;
/// Shared static-world navigation clearance. Actors still never block actors.
#[derive(Clone, Copy, Debug)]
pub struct Clearance {
    pub radius: f32,
    pub height: f32,
    pub step: f32,
    pub minimum_up: f32,
}
pub fn clearance() -> Clearance {
    Clearance {
        radius: FOOTPRINT_RADIUS,
        height: HEIGHT,
        step: STEP,
        minimum_up: MIN_UP,
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Motion {
    vertical_speed: f32,
    grounded: bool,
    jump_held: bool,
    solver_rollbacks: u64,
}
/// Gameplay state required to replay this controller. Positions, world/tick
/// identity and input scheduling are separate correction contracts. Diagnostics
/// are intentionally excluded. The terrain correction profile sends this state
/// from authority to client; commands never supply it as gameplay truth.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionState {
    pub vertical_speed: f32,
    pub grounded: bool,
    pub jump_held: bool,
}
impl MotionState {
    pub fn valid(self) -> bool {
        self.vertical_speed.is_finite()
            && (-1024. ..=JUMP_SPEED).contains(&self.vertical_speed)
            && (!self.grounded || self.vertical_speed == 0.)
    }
}
/// An additional infinite-height movement boundary, independent of world
/// geometry. Sweeps use capsule-bottom positions and include footprint clearance.
pub trait Boundary {
    fn sweep(&self, feet: Vec3, delta: Vec3) -> Option<crate::physics::SweepHit>;
    fn on_hit(&self, _feet: Vec3, _delta: Vec3) {}
}
impl Motion {
    pub fn snapshot(self) -> MotionState {
        MotionState {
            vertical_speed: self.vertical_speed,
            grounded: self.grounded,
            jump_held: self.jump_held,
        }
    }
    /// Restore validated gameplay state. Replay diagnostics start separately;
    /// an acknowledgement cannot import or inflate solver counters.
    pub fn restore(state: MotionState) -> Result<Self, &'static str> {
        if !state.valid() {
            return Err("Invalid kinematic correction state");
        }
        Ok(Self {
            vertical_speed: if state.vertical_speed == 0. {
                0.
            } else {
                state.vertical_speed
            },
            grounded: state.grounded,
            jump_held: state.jump_held,
            solver_rollbacks: 0,
        })
    }
    /// Recovery preserves the observed held level so a reset cannot retrigger it.
    pub fn reset(&mut self, jump_held: bool) {
        *self = Self {
            jump_held,
            ..Self::default()
        };
    }

    pub fn grounded(self) -> bool {
        self.grounded
    }
    pub fn vertical_speed(self) -> f32 {
        self.vertical_speed
    }
    /// Conservative endpoint rejections, separate from world recovery/respawn.
    pub fn solver_rollbacks(self) -> u64 {
        self.solver_rollbacks
    }
}
fn shape(clearance: f32) -> Collider {
    Collider::capsule(RADIUS - clearance, HEIGHT - 2. * RADIUS)
}
fn centre(feet: Vec3) -> Vec3 {
    feet + Vec3::Y * (HEIGHT / 2.)
}
fn probe(scene: &StaticScene, feet: Vec3, distance: f32) -> Option<(Vec3, Vec3)> {
    let sole = Collider::sphere(RADIUS);
    let capsule = shape(0.);
    // Preserve ordinary capsule contacts. Its sole provides a standard Avian
    // fallback when convex CCD error makes a support candidate invalid.
    // Query offsets never move the actual character through headroom.
    for lift in [SKIN * 2. + 0.01, 0.01, SKIN, 0.] {
        for (shape, height) in [(&capsule, HEIGHT / 2.), (&sole, RADIUS)] {
            let Some(hit) = scene.sweep(
                shape,
                feet + Vec3::Y * (height + lift),
                Vec3::NEG_Y * (distance + lift),
                WORLD,
                SKIN,
            ) else {
                continue;
            };
            if hit.normal.y < MIN_UP {
                return None;
            }
            let ground = feet + Vec3::Y * (lift - hit.distance + CONTACT_PAD);
            if scene.penetrates(&capsule, centre(ground), WORLD, 0.0001)
                || scene.penetrates(&sole, ground + Vec3::Y * RADIUS, WORLD, 0.0001)
            {
                continue;
            }
            return Some((ground, hit.normal));
        }
    }
    None
}
fn slide(
    scene: &StaticScene,
    mut feet: Vec3,
    mut delta: Vec3,
    horizontal: bool,
    boundary: Option<&dyn Boundary>,
) -> Vec3 {
    let capsule = shape(0.);
    for _ in 0..MAX_SLIDES {
        let length = delta.length();
        // Discard solver-scale projection noise (10 micrometres). The
        // smallest valid input at 60 Hz still exceeds this threshold.
        if length < 0.00001 {
            break;
        }
        let physical = scene.sweep(&capsule, centre(feet), delta, WORLD, SKIN);
        let limited = boundary.and_then(|b| b.sweep(feet, delta));
        if limited.is_some_and(|b| physical.is_none_or(|a| b.distance <= a.distance))
            && let Some(boundary) = boundary
        {
            boundary.on_hit(feet, delta);
        }
        let hit = match (physical, limited) {
            (Some(a), Some(b)) => Some(if a.distance < b.distance { a } else { b }),
            (a, b) => a.or(b),
        };
        let Some(hit) = hit else {
            feet += delta;
            break;
        };
        let travelled = (hit.distance - 0.0005).max(0.);
        let travelled = if travelled < 0.00001 { 0. } else { travelled };
        let advance = delta * (travelled / length);
        feet += advance;
        delta -= advance;
        let mut normal = hit.normal;
        if horizontal && normal.y > 0. && normal.y < MIN_UP {
            normal.y = 0.;
            normal = normal.normalize_or_zero();
        }
        let into = delta.dot(normal);
        if into >= -0.000001 {
            break;
        }
        delta -= normal * into;
    }
    feet
}
fn step_up(
    scene: &StaticScene,
    start: Vec3,
    delta: Vec3,
    boundary: Option<&dyn Boundary>,
) -> Option<Vec3> {
    let capsule = shape(0.);
    let up = Vec3::Y * STEP;
    if scene
        .sweep(&capsule, centre(start), up, WORLD, SKIN)
        .is_some()
    {
        return None;
    }
    let raised = start + up;
    if let Some(boundary) = boundary
        && boundary.sweep(raised, delta).is_some()
    {
        boundary.on_hit(raised, delta);
        return None;
    }
    if scene
        .sweep(&capsule, centre(raised), delta, WORLD, SKIN)
        .is_some()
    {
        return None;
    }
    let (landed, _) = probe(scene, raised + delta, STEP + SNAP)?;
    ((landed.y - start.y) <= STEP + 0.001).then_some(landed)
}
/// Validate unchanged actor placement before a static-scene publication.
pub fn validate_pose(
    scene: &StaticScene,
    feet: LocalPosition,
    grounded: bool,
) -> Result<(), &'static str> {
    let p = Vec3::from_array(feet.metres());
    if scene.penetrates(&shape(0.), centre(p), WORLD, 0.0001) {
        return Err("Candidate geometry penetrates actor");
    }
    if grounded && probe(scene, p, SNAP).is_none_or(|(ground, _)| (ground.y - p.y).abs() > 0.02) {
        return Err("Candidate changes grounded actor support");
    }
    Ok(())
}
/// Bounded capsule landing probe, used to preserve an airborne actor's existing
/// landing surface during activation. This never moves the actor.
pub fn support_below(scene: &StaticScene, feet: LocalPosition) -> Option<f32> {
    probe(scene, Vec3::from_array(feet.metres()), 64.).map(|(p, _)| p.y)
}

/// Resolve a trusted spawn guess onto nearby support, with bounded lift/headroom
/// checks. This is separate from movement and never accepts a client position.
pub fn spawn(scene: &StaticScene, guess: LocalPosition) -> Result<LocalPosition, &'static str> {
    let guess = Vec3::from_array(guess.metres());
    for lift in [0., 0.02, 0.05, 0.1, 0.2, STEP] {
        let raised = guess + Vec3::Y * lift;
        if scene.penetrates(&shape(0.), centre(raised), WORLD, 0.0001) {
            continue;
        }
        if let Some((feet, _)) = probe(scene, raised, lift + SNAP)
            && !scene.penetrates(&shape(0.), centre(feet), WORLD, 0.0001)
        {
            return LocalPosition::new(feet.to_array()).ok_or("Spawn outside bounded frame");
        }
    }
    Err("No safe kinematic spawn near requested support")
}

/// `start` is the capsule's bottom, in a checked local physics frame. A jump is a
/// held input level: only a new press while grounded starts one jump. There is
/// no network protocol or caller-selected timestep in this shared operation.
/// Errors leave both input pose and state unchanged; depenetration is separate.
pub fn advance(
    scene: &StaticScene,
    state: &mut Motion,
    start: LocalPosition,
    intent: Intent,
    rate: TickRate,
) -> Result<LocalPosition, &'static str> {
    advance_bounded(scene, state, start, intent, rate, None)
}
pub fn advance_bounded(
    scene: &StaticScene,
    state: &mut Motion,
    start: LocalPosition,
    intent: Intent,
    rate: TickRate,
    boundary: Option<&dyn Boundary>,
) -> Result<LocalPosition, &'static str> {
    advance_development(scene, state, start, intent, rate, boundary, 1.)
}
/// Server-configured development travel multiplier; never taken from wire input.
pub fn advance_development(
    scene: &StaticScene,
    state: &mut Motion,
    start: LocalPosition,
    intent: Intent,
    rate: TickRate,
    boundary: Option<&dyn Boundary>,
    speed: f32,
) -> Result<LocalPosition, &'static str> {
    if !matches!(speed, 1. | 8. | 32.) {
        return Err("Invalid developer travel speed");
    }
    if !intent.valid() {
        return Err("Invalid movement axes");
    }
    let start_position = start;
    let start = Vec3::from_array(start.metres());
    if scene.penetrates(&shape(0.), centre(start), WORLD, 0.0001) {
        return Err("Kinematic spawn/state penetrates world");
    }
    let jump = intent.jump;
    let mut next = *state;
    let mut feet = start;
    if next.vertical_speed <= 0. {
        if let Some((ground, _)) = probe(scene, feet, SNAP) {
            feet = ground;
            next.grounded = true;
            next.vertical_speed = 0.;
        } else {
            next.grounded = false;
        }
    }
    if jump && !next.jump_held && next.grounded {
        next.vertical_speed = JUMP_SPEED;
        next.grounded = false;
    }
    next.jump_held = jump;
    let axes = Vec3::new(
        f32::from(intent.axes[0]) / 1000.,
        0.,
        f32::from(intent.axes[1]) / 1000.,
    );
    let delta =
        axes.clamp_length_max(1.) * if intent.sprint { 8. } else { 5. } * rate.seconds() * speed;
    let moved = slide(scene, feet, delta, true, boundary);
    let progress = (moved - feet).xz().length();
    feet = if next.grounded && progress + 0.0001 < delta.length() {
        step_up(scene, feet, delta, boundary)
            .filter(|stepped| (*stepped - feet).xz().length() > progress + 0.0001)
            .unwrap_or(moved)
    } else {
        moved
    };
    if next.grounded {
        if let Some((ground, _)) = probe(scene, feet, SNAP) {
            feet = ground;
        } else {
            next.grounded = false;
        }
    }
    if !next.grounded {
        let dt = rate.seconds();
        let dy = next.vertical_speed * dt - 0.5 * GRAVITY * dt * dt;
        next.vertical_speed = (next.vertical_speed - GRAVITY * dt).max(-50.);
        let before = feet;
        feet = slide(scene, feet, Vec3::Y * dy, false, boundary);
        if dy > 0. && feet.y - before.y < dy - 0.0001 {
            next.vertical_speed = 0.;
        }
        if next.vertical_speed <= 0.
            && let Some((ground, _)) = probe(scene, feet, SNAP)
        {
            feet = ground;
            next.grounded = true;
            next.vertical_speed = 0.;
        }
    }
    let mut result =
        LocalPosition::new(feet.to_array()).ok_or("Kinematic movement left bounded frame")?;
    // A step/slide can end inside overlapping surfaces when a CCD contact is
    // approximate. The admitted start is still valid: block this attempted
    // step there instead of turning a rejected endpoint into spawn recovery.
    if scene.penetrates(&shape(0.), centre(feet), WORLD, 0.0001) {
        feet = start;
        result = start_position;
        next = *state;
        next.jump_held = jump;
        next.solver_rollbacks = next.solver_rollbacks.saturating_add(1);
    }
    // Ground support must remain valid when queried at the returned endpoint.
    // Near a ledge or overlapping surface a snap's first contact may not be a
    // stable support at its resulting pose. Continue airborne in that case;
    // do not weaken the publication clearance/support checks.
    next.grounded = next.grounded
        && probe(scene, feet, SNAP).is_some_and(|(ground, _)| (ground.y - feet.y).abs() <= 0.02);
    if next.grounded {
        next.vertical_speed = 0.;
    }
    *state = next;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upward_near_surface_motion_remains_airborne() {
        let scene = StaticScene::new(vec![crate::physics::PlacedCollider {
            shape: Collider::cuboid(10., 0.5, 10.),
            center: Vec3::new(0., -0.25, 0.),
            layer: WORLD,
        }]);
        let mut motion = Motion {
            vertical_speed: 0.5,
            grounded: false,
            ..Default::default()
        };
        let feet = advance(
            &scene,
            &mut motion,
            LocalPosition::new([0., 0.01, 0.]).unwrap(),
            Intent::default(),
            TickRate::Hz60,
        )
        .unwrap();
        assert!(feet.metres()[1] > 0.01);
        assert!(!motion.grounded());
        assert!(motion.vertical_speed() > 0.);
        validate_pose(&scene, feet, false).unwrap();
    }
}
