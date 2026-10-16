//! Bounded terrain-only walking fixture. Static mesh triangles are authority;
//! no player or NPC geometry enters this module.
use crate::{
    Result,
    wire::Intent,
    world::{PAWN_RADIUS, PAWN_Y},
};
use ring::digest::{SHA256, digest};
use serde::Deserialize;
use std::{fs, path::Path};

pub const GRAYBOX_ID: [u8; 32] = [0; 32];
const MAX_PROFILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SLOPE: f32 = 0.83909965; // tan(40 degrees)

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    schema: u32,
    source_origin: [f32; 3],
    visual_sha256: [u8; 32],
    triangles: Vec<[[f32; 3]; 3]>,
}

pub struct Terrain {
    triangles: Vec<[[f32; 3]; 3]>,
    pub identity: [u8; 32],
    pub visual_sha256: [u8; 32],
}

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    digest(&SHA256, bytes).as_ref().try_into().unwrap()
}
impl Terrain {
    pub fn load(path: &Path) -> Result<Self> {
        if fs::metadata(path)?.len() > MAX_PROFILE_BYTES {
            return Err("Terrain profile exceeds bounded fixture budget".into());
        }
        Self::decode(&fs::read(path)?)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() as u64 > MAX_PROFILE_BYTES {
            return Err("Terrain profile too large".into());
        }
        let profile: Profile = serde_json::from_slice(bytes)?;
        if profile.schema != 1
            || profile.triangles.is_empty()
            || profile.triangles.len() > 8192
            || profile.source_origin.iter().any(|v| !v.is_finite())
            || profile
                .triangles
                .iter()
                .flatten()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 1023.)
        {
            return Err("Invalid bounded terrain profile".into());
        }
        for t in &profile.triangles {
            let determinant = cross(t[0], t[1], t[2]);
            if determinant >= -1e-6 {
                return Err("Terrain must have upward nonvertical triangles".into());
            }
        }
        let terrain = Self {
            triangles: profile.triangles,
            identity: sha256(bytes),
            visual_sha256: profile.visual_sha256,
        };
        for x in [0., 2., 4., 6.] {
            if terrain.standing([x, 0.]).is_none() {
                return Err("Unsafe terrain spawn".into());
            }
        }
        Ok(terrain)
    }
    /// Centre-foot height on the highest surface. Intended only for height terrain,
    /// not caves, bridges, walls, arbitrary overlapping mesh colliders or gravity.
    pub fn height(&self, p: [f32; 2]) -> Option<f32> {
        self.surface(p).map(|(height, _)| height)
    }
    fn surface(&self, p: [f32; 2]) -> Option<(f32, bool)> {
        if p.iter().any(|v| !v.is_finite()) {
            return None;
        }
        self.triangles
            .iter()
            .filter_map(|t| sample(t, p))
            .max_by(|a, b| a.0.total_cmp(&b.0))
    }
    fn standing(&self, p: [f32; 2]) -> Option<f32> {
        if p.iter()
            .any(|v| !v.is_finite() || v.abs() > 19. - PAWN_RADIUS)
        {
            return None;
        }
        for dx in [-PAWN_RADIUS, 0., PAWN_RADIUS] {
            for dz in [-PAWN_RADIUS, 0., PAWN_RADIUS] {
                if !self.surface([p[0] + dx, p[1] + dz])?.1 {
                    return None;
                }
            }
        }
        self.height(p)
    }
    pub fn spawn_y(&self, xz: [f32; 2]) -> f32 {
        self.standing(xz).expect("Qualified terrain spawn") + PAWN_Y
    }
    pub fn move_player(&self, start: [f32; 2], intent: Intent, seconds: f32) -> [f32; 2] {
        if !intent.valid() || !seconds.is_finite() || seconds <= 0. || seconds > 0.1 {
            return start;
        }
        let mut d = intent.axes.map(|v| f32::from(v) / 1000.);
        let length = d[0].hypot(d[1]).max(1.);
        let speed = if intent.sprint { 8. } else { 5. };
        for v in &mut d {
            *v *= speed * seconds / length;
        }
        // Small bounded steps and footprint probes reject steep/missing ground.
        // This is a conservative sampled walker, not a general swept capsule.
        let steps = (d[0].hypot(d[1]) / 0.05).ceil().max(1.) as usize;
        let mut p = start;
        for _ in 0..steps {
            for axis in 0..2 {
                if d[axis] == 0. {
                    continue;
                }
                let mut next = p;
                next[axis] += d[axis] / steps as f32;
                if let (Some(before), Some(after)) = (self.standing(p), self.standing(next))
                    && (after - before).abs() <= (next[axis] - p[axis]).abs() * MAX_SLOPE + 0.0001
                {
                    p = next;
                }
            }
        }
        p
    }
    /// Sampled camera clearance for the terrain patch, excluding all actors.
    pub fn camera_fraction(&self, target: [f32; 3], eye: [f32; 3]) -> f32 {
        let d = [eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]];
        let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let steps = (length / 0.05).ceil().max(1.) as usize;
        for i in 1..=steps {
            let f = i as f32 / steps as f32;
            let p = [target[0] + d[0] * f, target[2] + d[2] * f];
            if self
                .height(p)
                .is_none_or(|h| target[1] + d[1] * f < h + 0.15)
            {
                return (i - 1) as f32 / steps as f32;
            }
        }
        1.
    }
}
fn cross(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    (b[0] - a[0]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[0] - a[0])
}
fn sample(t: &[[f32; 3]; 3], p: [f32; 2]) -> Option<(f32, bool)> {
    if p[0] < t[0][0].min(t[1][0]).min(t[2][0]) - 1e-5
        || p[0] > t[0][0].max(t[1][0]).max(t[2][0]) + 1e-5
        || p[1] < t[0][2].min(t[1][2]).min(t[2][2]) - 1e-5
        || p[1] > t[0][2].max(t[1][2]).max(t[2][2]) + 1e-5
    {
        return None;
    }
    let p3 = [p[0], 0., p[1]];
    let area = cross(t[0], t[1], t[2]);
    let u = cross(p3, t[1], t[2]) / area;
    let v = cross(t[0], p3, t[2]) / area;
    let w = 1. - u - v;
    if u < -1e-5 || v < -1e-5 || w < -1e-5 {
        return None;
    }
    let dx1 = t[1][0] - t[0][0];
    let dz1 = t[1][2] - t[0][2];
    let dy1 = t[1][1] - t[0][1];
    let dx2 = t[2][0] - t[0][0];
    let dz2 = t[2][2] - t[0][2];
    let dy2 = t[2][1] - t[0][1];
    let gx = (dy1 * dz2 - dz1 * dy2) / area;
    let gz = (dx1 * dy2 - dy1 * dx2) / area;
    Some((
        u * t[0][1] + v * t[1][1] + w * t[2][1],
        gx.hypot(gz) <= MAX_SLOPE,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn plane(slope: f32) -> Vec<u8> {
        let a = [-24., -24. * slope, -24.];
        let b = [24., 24. * slope, -24.];
        let c = [24., 24. * slope, 24.];
        let d = [-24., -24. * slope, 24.];
        serde_json::to_vec(&serde_json::json!({"schema":1,"source_origin":[0,0,0],"visual_sha256":vec![0;32],"triangles":[[a,c,b],[a,d,c]]})).unwrap()
    }
    #[test]
    fn barycentric_heights_speed_and_bounds() {
        let t = Terrain::decode(&plane(0.3)).unwrap();
        assert!((t.height([3., 4.]).unwrap() - 0.9).abs() < 1e-5);
        let intent = Intent {
            axes: [1000, 0],
            sprint: false,
        };
        let mut p = [0., 0.];
        for _ in 0..30 {
            p = t.move_player(p, intent, 1. / 30.);
        }
        assert!((p[0] - 5.).abs() < 1e-4);
        assert!((t.spawn_y(p) - 2.4).abs() < 1e-4);
        assert_eq!(t.move_player(p, intent, f32::NAN), p);
        for _ in 0..200 {
            p = t.move_player(p, intent, 1. / 30.);
        }
        assert!(p[0] <= 18.65 && p[0] > 18.59);
    }
    #[test]
    fn blocked_steep_ground_and_holes_allow_retreat_and_sliding() {
        for mode in ["steep", "hole", "step"] {
            let mut profile: serde_json::Value = serde_json::from_slice(&plane(0.)).unwrap();
            let rectangle = |lo: f32, hi: f32, ylo: f32, yhi: f32| {
                let a = [lo, ylo, -24.];
                let b = [hi, yhi, -24.];
                let c = [hi, yhi, 24.];
                let d = [lo, ylo, 24.];
                vec![[a, c, b], [a, d, c]]
            };
            let mut triangles = rectangle(-24., 8., 0., 0.);
            if mode == "steep" {
                triangles.extend(rectangle(8., 24., 0., 16.));
            }
            if mode == "step" {
                triangles.extend(rectangle(8., 24., 1., 1.));
            }
            if mode == "hole" {
                triangles.extend(rectangle(8.3, 24., 0., 0.));
            }
            profile["triangles"] = serde_json::json!(triangles);
            let t = Terrain::decode(&serde_json::to_vec(&profile).unwrap()).unwrap();
            let mut p = [6., 0.];
            for _ in 0..60 {
                p = t.move_player(
                    p,
                    Intent {
                        axes: [1000, 0],
                        sprint: true,
                    },
                    1. / 30.,
                );
            }
            assert!(p[0] < 8.01, "{mode}: {p:?}");
            let z = p[1];
            p = t.move_player(
                p,
                Intent {
                    axes: [1000, 1000],
                    sprint: false,
                },
                1. / 30.,
            );
            assert!(p[1] > z, "Slide along {mode}");
            let x = p[0];
            p = t.move_player(
                p,
                Intent {
                    axes: [-1000, 0],
                    sprint: false,
                },
                1. / 30.,
            );
            assert!(p[0] < x, "Retreat from {mode}");
        }
    }
    #[test]
    fn steep_missing_degenerate_and_unsafe_spawn_are_rejected() {
        assert!(Terrain::decode(&plane(1.)).is_err());
        assert!(Terrain::decode(b"{}").is_err());
        let mut p: serde_json::Value = serde_json::from_slice(&plane(0.)).unwrap();
        p["triangles"][0][0] = serde_json::json!([24., 0., 24.]);
        assert!(Terrain::decode(&serde_json::to_vec(&p).unwrap()).is_err());
        let t = Terrain::decode(&plane(0.3)).unwrap();
        assert!(t.height([25., 0.]).is_none());
        assert!(t.height([f32::NAN, 0.]).is_none());
        assert!(t.camera_fraction([0., 1., 0.], [10., 1., 0.]) < 1.);
    }
}
