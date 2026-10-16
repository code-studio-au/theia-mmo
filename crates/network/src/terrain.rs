//! Bounded terrain walking fixture with authored static solid envelopes;
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
    #[serde(default)]
    solids: Vec<Solid>,
    #[serde(default)]
    walk_surfaces: Vec<[[f32; 3]; 3]>,
    #[serde(default)]
    camera_solids: Vec<Solid>,
    #[serde(default)]
    step_height: f32,
}

/// Authored ground-blocking column with a finite box for camera obstruction.
/// These envelopes do not support walking underneath or inside an object.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Solid {
    pub center: [f32; 3],
    pub size: [f32; 3],
}

pub struct Terrain {
    triangles: Vec<[[f32; 3]; 3]>,
    pub identity: [u8; 32],
    pub visual_sha256: [u8; 32],
    pub solids: Vec<Solid>,
    walk_surfaces: Vec<[[f32; 3]; 3]>,
    camera_solids: Vec<Solid>,
    step_height: f32,
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
            || profile.triangles.len() + profile.walk_surfaces.len() > 8192
            || !profile.step_height.is_finite()
            || !(0.0..=0.25).contains(&profile.step_height)
            || profile.source_origin.iter().any(|v| !v.is_finite())
            || profile
                .triangles
                .iter()
                .chain(profile.walk_surfaces.iter())
                .flatten()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 1023.)
        {
            return Err("Invalid bounded terrain profile".into());
        }
        if profile.solids.len() + profile.camera_solids.len() > 64
            || profile
                .solids
                .iter()
                .chain(profile.camera_solids.iter())
                .any(|b| {
                    b.center.iter().any(|v| !v.is_finite() || v.abs() > 1023.)
                        || b.size
                            .iter()
                            .any(|v| !v.is_finite() || *v <= 0. || *v > 128.)
                        || (0..3).any(|i| b.center[i].abs() + b.size[i] / 2. > 1023.)
                })
        {
            return Err("Invalid bounded static solid envelopes".into());
        }
        for t in profile.triangles.iter().chain(profile.walk_surfaces.iter()) {
            let determinant = cross(t[0], t[1], t[2]);
            if determinant >= -1e-6 {
                return Err("Terrain must have upward nonvertical triangles".into());
            }
        }
        let terrain = Self {
            triangles: profile.triangles,
            identity: sha256(bytes),
            visual_sha256: profile.visual_sha256,
            solids: profile.solids,
            walk_surfaces: profile.walk_surfaces,
            camera_solids: profile.camera_solids,
            step_height: profile.step_height,
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
            .chain(self.walk_surfaces.iter())
            .filter_map(|t| sample(t, p))
            .max_by(|a, b| a.0.total_cmp(&b.0))
    }
    fn standing(&self, p: [f32; 2]) -> Option<f32> {
        if p.iter()
            .any(|v| !v.is_finite() || v.abs() > 19. - PAWN_RADIUS)
        {
            return None;
        }
        if self.solids.iter().any(|b| {
            (p[0] - b.center[0]).abs() < b.size[0] / 2. + PAWN_RADIUS
                && (p[1] - b.center[2]).abs() < b.size[2] / 2. + PAWN_RADIUS
        }) {
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
                for solid in &self.solids {
                    next[axis] =
                        crate::world::sweep_box_axis(p, next[axis], axis, solid.center, solid.size);
                }
                if let (Some(before), Some(after)) = (self.standing(p), self.standing(next))
                    && (after - before).abs()
                        <= ((next[axis] - p[axis]).abs() * MAX_SLOPE + 0.0001).max(self.step_height)
                {
                    p = next;
                }
            }
        }
        p
    }
    /// Sampled camera clearance for the terrain patch, excluding all actors.
    pub fn camera_fraction(&self, target: [f32; 3], eye: [f32; 3]) -> f32 {
        let static_fraction = self
            .solids
            .iter()
            .chain(self.camera_solids.iter())
            .map(|s| crate::world::box_camera_fraction(target, eye, s.center, s.size))
            .fold(1., f32::min);
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
                return static_fraction.min((i - 1) as f32 / steps as f32);
            }
        }
        static_fraction
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
    fn authored_solids_stop_slide_and_obstruct_camera_on_slopes() {
        let mut profile: serde_json::Value = serde_json::from_slice(&plane(0.1)).unwrap();
        profile["solids"] = serde_json::json!([{"center":[-8,3,-8],"size":[8.6,10,8.6]}]);
        let t = Terrain::decode(&serde_json::to_vec(&profile).unwrap()).unwrap();
        let mut p = [-8., 0.];
        for _ in 0..90 {
            p = t.move_player(
                p,
                Intent {
                    axes: [0, -1000],
                    sprint: true,
                },
                1. / 30.,
            );
        }
        assert!((p[1] + 3.35).abs() < 1e-5);
        let slid = t.move_player(
            p,
            Intent {
                axes: [1000, -1000],
                sprint: true,
            },
            1. / 30.,
        );
        assert!(slid[0] > p[0]);
        assert_eq!(slid[1], p[1]);
        assert!(
            t.move_player(
                p,
                Intent {
                    axes: [0, 1000],
                    sprint: false
                },
                1. / 30.
            )[1] > p[1]
        );
        let target = [p[0], t.spawn_y(p), p[1]];
        assert!(t.camera_fraction(target, [target[0], target[1] + 7., target[2] - 12.]) < 0.1);
        assert_eq!(
            t.camera_fraction(target, [target[0], target[1] + 7., target[2] + 12.]),
            1.
        );
        for invalid in [
            serde_json::json!({"center":[0,3,0],"size":[8,8,8]}),
            serde_json::json!({"center":[-8,3,-8],"size":[0,8,8]}),
        ] {
            profile["solids"] = serde_json::json!([invalid]);
            assert!(Terrain::decode(&serde_json::to_vec(&profile).unwrap()).is_err());
        }
    }
    #[test]
    fn authored_floor_and_ramp_allow_entry_exit_and_keep_walls_solid() {
        let mut profile: serde_json::Value = serde_json::from_slice(&plane(0.)).unwrap();
        let quad = |x0: f32, x1: f32, z0: f32, z1: f32, y0: f32, y1: f32| {
            let a = [x0, y0, z0];
            let b = [x1, y0, z0];
            let c = [x1, y1, z1];
            let d = [x0, y1, z1];
            vec![[a, b, c], [a, c, d]]
        };
        let mut surfaces = quad(-3., 3., -2., -8., 1.5, 1.5);
        surfaces.extend(quad(-1., 1., 2., -2., 0., 1.5));
        profile["walk_surfaces"] = serde_json::json!(surfaces);
        profile["step_height"] = serde_json::json!(0.2);
        profile["solids"] = serde_json::json!([
            {"center":[-3.1,2,-5],"size":[0.2,4,6]},
            {"center":[3.1,2,-5],"size":[0.2,4,6]},
            {"center":[-2,2,-2],"size":[2,4,0.2]},
            {"center":[2,2,-2],"size":[2,4,0.2]},
            {"center":[0,2,-8.1],"size":[6.4,4,0.2]}]);
        profile["camera_solids"] = serde_json::json!([{"center":[0,4.1,-5],"size":[6,1,6]}]);
        let t = Terrain::decode(&serde_json::to_vec(&profile).unwrap()).unwrap();
        let mut p = [0., 3.];
        for _ in 0..48 {
            p = t.move_player(
                p,
                Intent {
                    axes: [0, -1000],
                    sprint: false,
                },
                1. / 30.,
            );
        }
        assert!(p[1] < -4.9 && p[1] > -5.1);
        assert!((t.height(p).unwrap() - 1.5).abs() < 1e-5);
        assert!(t.camera_fraction([p[0], 2.4, p[1]], [p[0], 9.4, p[1] + 12.]) < 0.5);
        for _ in 0..40 {
            p = t.move_player(
                p,
                Intent {
                    axes: [1000, 0],
                    sprint: true,
                },
                1. / 30.,
            );
        }
        assert!((p[0] - 2.65).abs() < 1e-5);
        for _ in 0..16 {
            p = t.move_player(
                p,
                Intent {
                    axes: [-1000, 0],
                    sprint: false,
                },
                1. / 30.,
            );
        }
        assert!(p[0].abs() < 0.1);
        for _ in 0..48 {
            p = t.move_player(
                p,
                Intent {
                    axes: [0, 1000],
                    sprint: false,
                },
                1. / 30.,
            );
        }
        assert!(p[1] > 2.8 && t.height(p).unwrap().abs() < 1e-5);
        profile["step_height"] = serde_json::json!(1.0);
        assert!(Terrain::decode(&serde_json::to_vec(&profile).unwrap()).is_err());
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
