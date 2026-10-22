//! Finite gameplay rectangle, independent of package/contributor envelopes.
//! Global checked metres bind admission; local swept planes constrain only XZ.
use crate::{Result, kinematic, physics::SweepHit};
use bevy::prelude::Vec3;
use theia_core::coordinates::{GlobalPosition, LocalFrame, LocalPosition};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldBounds([[f64; 2]; 2]);
impl WorldBounds {
    pub fn new(bounds: [[f64; 2]; 2]) -> Option<Self> {
        let radius = f64::from(kinematic::FOOTPRINT_RADIUS);
        for p in bounds {
            GlobalPosition::new([p[0], 0., p[1]])?;
        }
        if (0..2).any(|axis| bounds[1][axis] - bounds[0][axis] <= 2. * radius) {
            return None;
        }
        Some(Self(bounds))
    }
    pub fn metres(self) -> [[f64; 2]; 2] {
        self.0
    }
    pub fn contains(self, feet: GlobalPosition) -> bool {
        let p = feet.metres();
        let radius = f64::from(kinematic::FOOTPRINT_RADIUS);
        (0..2).all(|axis| {
            p[axis * 2] >= self.0[0][axis] + radius && p[axis * 2] <= self.0[1][axis] - radius
        })
    }
    pub fn clamp(self, feet: GlobalPosition) -> GlobalPosition {
        let mut p = feet.metres();
        let radius = f64::from(kinematic::FOOTPRINT_RADIUS);
        for axis in 0..2 {
            p[axis * 2] = p[axis * 2].clamp(self.0[0][axis] + radius, self.0[1][axis] - radius);
        }
        GlobalPosition::new(p).expect("Checked finite rectangle and global position")
    }
    pub fn in_frame(self, frame: LocalFrame) -> Result<LocalBounds> {
        let origin = frame.origin().metres();
        let radius = f64::from(kinematic::FOOTPRINT_RADIUS);
        let mut centres = [[0f32; 2]; 2];
        for (side, row) in centres.iter_mut().enumerate() {
            let mut point = [0.; 3];
            for axis in 0..2 {
                let edge = self.0[side][axis] - origin[axis * 2];
                if edge.abs() > theia_core::coordinates::MAX_LOCAL_METRES {
                    return Err("World bounds outside bounded query frame".into());
                }
                point[axis * 2] = edge as f32;
                let exact = edge + if side == 0 { radius } else { -radius };
                let mut local = exact as f32;
                // Never round an admitted centre outwards across a global edge.
                if side == 0 && f64::from(local) < exact {
                    local = local.next_up();
                } else if side == 1 && f64::from(local) > exact {
                    local = local.next_down();
                }
                row[axis] = local;
            }
            LocalPosition::new(point).ok_or("World bounds outside bounded query frame")?;
        }
        if (0..2).any(|axis| centres[0][axis] >= centres[1][axis]) {
            return Err("World bounds collapse in local query frame".into());
        }
        Ok(LocalBounds(centres))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct LocalBounds([[f32; 2]; 2]);
impl kinematic::Boundary for LocalBounds {
    fn sweep(&self, feet: Vec3, delta: Vec3) -> Option<SweepHit> {
        let horizontal = Vec3::new(delta.x, 0., delta.z);
        if horizontal.length_squared() < 1e-12 {
            return None;
        }
        if (0..2).any(|axis| feet[axis * 2] < self.0[0][axis] || feet[axis * 2] > self.0[1][axis]) {
            return Some(SweepHit {
                distance: 0.,
                normal: -horizontal.normalize(),
            });
        }
        let mut nearest: Option<SweepHit> = None;
        for axis in 0..2 {
            let component = axis * 2;
            if delta[component] == 0. {
                continue;
            }
            let positive = delta[component] > 0.;
            let plane = self.0[usize::from(positive)][axis];
            let fraction = (plane - feet[component]) / delta[component];
            if !(0.0..=1.0).contains(&fraction) {
                continue;
            }
            let mut normal = Vec3::ZERO;
            normal[component] = if positive { -1. } else { 1. };
            let hit = SweepHit {
                distance: fraction * delta.length(),
                normal,
            };
            if nearest.is_none_or(|old| hit.distance < old.distance) {
                nearest = Some(hit);
            }
        }
        nearest
    }
}
