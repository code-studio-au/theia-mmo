//! Versioned bounded static-world geometry with Avian mesh/solid queries;
//! no player or NPC geometry enters this module.
use crate::{
    Result,
    content::{self, MAX_COLLISION_BYTES, Manifest},
    physics::{self, PlacedCollider, StaticScene},
    static_shapes::{MAX_INPUT_TRIANGLES, ShapeSet},
    wire::Intent,
    world::{PAWN_RADIUS, PAWN_Y},
};
use avian3d::prelude::Collider;
use bevy::prelude::Vec3;
use ring::digest::{SHA256, digest};
use serde::Deserialize;
use std::path::Path;

pub const GRAYBOX_ID: [u8; 32] = [0; 32];
const MAX_PROFILE_BYTES: u64 = MAX_COLLISION_BYTES;
const CELL_SIZE: f32 = 8.;
const MAX_COVERAGE_CELLS: usize = 2_000_000;
fn patch_bounds() -> [[f32; 2]; 2] {
    [[-19.; 2], [19.; 2]]
}
const MAX_SLOPE: f32 = 0.83909965; // tan(40 degrees)

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    schema: u32,
    #[serde(default)]
    bounds: Option<[[f32; 2]; 2]>,
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
    #[serde(default)]
    spawns: Vec<[f32; 3]>,
    #[serde(default)]
    static_shapes: Option<ShapeSet>,
}

/// Static conservative box for movement and camera clearance. Schema-1 fixtures
/// use blocking columns; schema-2 maps filter boxes by standing body height.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Solid {
    pub center: [f32; 3],
    pub size: [f32; 3],
}

pub struct Terrain {
    schema: u32,
    bounds: [[f32; 2]; 2],
    pub identity: [u8; 32],
    pub visual_sha256: [u8; 32],
    pub package: Option<Manifest>,
    pub solids: Vec<Solid>,
    pub physics: StaticScene,
    ground_ceiling: f32,
    step_height: f32,
    spawns: Vec<[f32; 3]>,
    fall_limit: f32,
    world_envelope: bool,
}

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    digest(&SHA256, bytes).as_ref().try_into().unwrap()
}
impl Terrain {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = content::read_bounded(path, MAX_PROFILE_BYTES)?;
        let package = Manifest::load_optional(&path.with_file_name(content::MANIFEST_FILE))?;
        if let Some(package) = &package {
            package.verify_collision(&bytes)?;
        }
        let mut terrain = Self::decode(&bytes)?;
        if let Some(package) = package {
            package.verify_profile(terrain.schema, terrain.visual_sha256)?;
            terrain.identity = package.identity();
            println!(
                "world_package content_id={} revision={} collision_verified=true",
                package.content_id().as_str(),
                package.revision()
            );
            terrain.package = Some(package);
        }
        Ok(terrain)
    }
    pub fn verify_visual(&self, path: &Path) -> Result<()> {
        if let Some(package) = &self.package {
            package.verify_visual(path)
        } else {
            content::verify_visual(path, self.visual_sha256, None)
        }
    }
    pub fn verify_visual_in(&self, path: &Path, asset_root: &Path) -> Result<()> {
        if let Some(package) = &self.package {
            package.verify_visual_in(path, asset_root)
        } else {
            self.verify_visual(path)
        }
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() as u64 > MAX_PROFILE_BYTES {
            return Err("Terrain profile too large".into());
        }
        let profile: Profile = serde_json::from_slice(bytes)?;
        let modern = profile.schema >= 3;
        let shape_world = profile
            .static_shapes
            .as_ref()
            .is_some_and(ShapeSet::has_world_geometry);
        let bounds = match profile.bounds {
            Some(bounds) => bounds,
            None if profile.schema < 3 => patch_bounds(),
            None => return Err("General collision profiles require explicit bounds".into()),
        };
        if (profile.schema == 1 && bytes.len() > 4 * 1024 * 1024)
            || !matches!(profile.schema, 1..=4)
            || ((profile.schema == 4) != profile.static_shapes.is_some())
            || (profile.triangles.is_empty()
                && (!modern || (profile.solids.is_empty() && !shape_world)))
            || profile.triangles.len() + profile.walk_surfaces.len()
                > if profile.schema >= 2 {
                    MAX_INPUT_TRIANGLES
                } else {
                    8192
                }
            || bounds
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 1023.)
            || (0..2).any(|i| bounds[0][i] >= bounds[1][i])
            || (profile.schema == 1 && bounds != patch_bounds())
            || (profile.schema < 3 && !profile.spawns.is_empty())
            || (modern && (!profile.walk_surfaces.is_empty() || profile.step_height != 0.))
            || profile.spawns.len() > 8
            || profile.spawns.iter().any(|p| {
                p.iter().any(|v| !v.is_finite() || v.abs() > 1023.)
                    || p[0] < bounds[0][0] + PAWN_RADIUS + 0.005
                    || p[0] > bounds[1][0] - PAWN_RADIUS - 0.005
                    || p[2] < bounds[0][1] + PAWN_RADIUS + 0.005
                    || p[2] > bounds[1][1] - PAWN_RADIUS - 0.005
            })
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
        if profile.solids.len() + profile.camera_solids.len()
            > if profile.schema >= 2 { 4096 } else { 64 }
            || profile
                .solids
                .iter()
                .chain(profile.camera_solids.iter())
                .any(|b| {
                    b.center.iter().any(|v| !v.is_finite() || v.abs() > 1023.)
                        || b.size.iter().any(|v| {
                            !v.is_finite()
                                || *v <= 0.
                                || *v > if profile.schema >= 2 { 1024. } else { 128. }
                        })
                        || (0..3).any(|i| b.center[i].abs() + b.size[i] / 2. > 1023.)
                })
        {
            return Err("Invalid bounded static solid envelopes".into());
        }
        for t in profile.triangles.iter().chain(profile.walk_surfaces.iter()) {
            if modern {
                let [a, b, c] = t.map(Vec3::from_array);
                if (b - a).cross(c - a).length_squared() <= 1e-12 {
                    return Err("Degenerate general collision triangle".into());
                }
            } else if cross(t[0], t[1], t[2]) >= -1e-6 {
                return Err("Terrain must have upward nonvertical triangles".into());
            }
        }
        let mut entries = 0;
        for t in profile.triangles.iter().chain(profile.walk_surfaces.iter()) {
            let lo = [
                t[0][0].min(t[1][0]).min(t[2][0]),
                t[0][2].min(t[1][2]).min(t[2][2]),
            ];
            let hi = [
                t[0][0].max(t[1][0]).max(t[2][0]),
                t[0][2].max(t[1][2]).max(t[2][2]),
            ];
            validate_coverage_budget(lo.map(|v| v - 1e-4), hi.map(|v| v + 1e-4), &mut entries)?;
        }
        for b in &profile.solids {
            let lo = [
                b.center[0] - b.size[0] / 2. - PAWN_RADIUS - 0.1,
                b.center[2] - b.size[2] / 2. - PAWN_RADIUS - 0.1,
            ];
            let hi = [
                b.center[0] + b.size[0] / 2. + PAWN_RADIUS + 0.1,
                b.center[2] + b.size[2] / 2. + PAWN_RADIUS + 0.1,
            ];
            validate_coverage_budget(lo, hi, &mut entries)?;
        }
        let shapes = profile
            .static_shapes
            .as_ref()
            .map(|set| {
                set.compile(
                    profile.triangles.len() + profile.walk_surfaces.len(),
                    profile.solids.len()
                        + profile.camera_solids.len()
                        + usize::from(!profile.triangles.is_empty()),
                )
            })
            .transpose()?;
        if let Some(shapes) = &shapes {
            for bounds in &shapes.bounds {
                validate_coverage_budget(
                    [
                        bounds[0][0] - PAWN_RADIUS - 0.1,
                        bounds[0][2] - PAWN_RADIUS - 0.1,
                    ],
                    [
                        bounds[1][0] + PAWN_RADIUS + 0.1,
                        bounds[1][2] + PAWN_RADIUS + 0.1,
                    ],
                    &mut entries,
                )?;
            }
        }
        let mut colliders = Vec::new();
        let shape_min = shapes.as_ref().map_or(f32::INFINITY, |s| s.world_min_y);
        let shape_max = shapes.as_ref().map_or(f32::NEG_INFINITY, |s| s.world_max_y);
        let solid_min = if modern {
            profile
                .solids
                .iter()
                .map(|s| s.center[1] - s.size[1] / 2.)
                .fold(shape_min, f32::min)
        } else {
            f32::INFINITY
        };
        let solid_max = if modern {
            profile
                .solids
                .iter()
                .map(|s| s.center[1] + s.size[1] / 2.)
                .fold(shape_max, f32::max)
        } else {
            f32::NEG_INFINITY
        };
        let min_y = profile
            .triangles
            .iter()
            .chain(profile.walk_surfaces.iter())
            .flatten()
            .map(|v| v[1])
            .fold(solid_min, f32::min)
            - 2.;
        let max_y = profile
            .triangles
            .iter()
            .chain(profile.walk_surfaces.iter())
            .flatten()
            .map(|v| v[1])
            .fold(solid_max, f32::max)
            + crate::world::PAWN_HEIGHT
            + 2.;
        for b in &profile.solids {
            let mut size = b.size;
            let mut center = b.center;
            // Old schema-1 fixtures specified blocking columns, not 3D solids.
            if profile.schema == 1 {
                size[1] = max_y - min_y;
                center[1] = (max_y + min_y) / 2.;
            }
            colliders.push(PlacedCollider {
                shape: Collider::cuboid(size[0], size[1], size[2]),
                center: Vec3::from_array(center),
                layer: if profile.schema == 1 {
                    physics::SOLID
                } else {
                    physics::SOLID | physics::CAMERA
                },
            });
        }
        if profile.schema == 1 {
            for b in &profile.solids {
                colliders.push(PlacedCollider {
                    shape: Collider::cuboid(b.size[0], b.size[1], b.size[2]),
                    center: Vec3::from_array(b.center),
                    layer: physics::CAMERA,
                });
            }
        }
        for b in &profile.camera_solids {
            colliders.push(PlacedCollider {
                shape: Collider::cuboid(b.size[0], b.size[1], b.size[2]),
                center: Vec3::from_array(b.center),
                layer: physics::CAMERA,
            });
        }
        if let Some(shapes) = shapes {
            colliders.extend(shapes.colliders);
        }
        let vertices: Vec<Vec3> = profile
            .triangles
            .iter()
            .chain(profile.walk_surfaces.iter())
            .flatten()
            .copied()
            .map(Vec3::from_array)
            .collect();
        let ground_ceiling = vertices.iter().map(|v| v.y).fold(solid_max, f32::max) + 2.;
        let indices = (0..vertices.len() as u32)
            .step_by(3)
            .map(|i| [i, i + 1, i + 2])
            .collect();
        if !vertices.is_empty() {
            colliders.push(PlacedCollider {
                shape: Collider::trimesh(vertices, indices),
                center: Vec3::ZERO,
                layer: if modern {
                    physics::GROUND | physics::SOLID | physics::CAMERA
                } else {
                    physics::GROUND
                },
            });
        }
        let mut terrain = Self {
            schema: profile.schema,
            bounds,
            identity: sha256(bytes),
            visual_sha256: profile.visual_sha256,
            package: None,
            solids: profile.solids,
            physics: StaticScene::new(colliders),
            ground_ceiling,
            step_height: profile.step_height,
            spawns: Vec::new(),
            fall_limit: min_y - 16.,
            world_envelope: false,
        };
        if modern {
            for spawn in profile.spawns {
                let guess = theia_core::coordinates::LocalPosition::new(spawn)
                    .ok_or("Invalid explicit spawn frame")?;
                terrain.spawns.push(
                    crate::kinematic::spawn(&terrain.physics, guess)
                        .map_err(|e| format!("Unsafe explicit spawn: {e}"))?
                        .metres(),
                );
            }
        } else {
            // Legacy development worlds resolve trusted XZ to a floor once.
            for x in [0., 2., 4., 6.] {
                let y = terrain.support([x, 0.]).ok_or("Unsafe terrain spawn")?;
                let guess = theia_core::coordinates::LocalPosition::new([x, y, 0.])
                    .ok_or("Invalid terrain spawn frame")?;
                terrain.spawns.push(
                    crate::kinematic::spawn(&terrain.physics, guess)
                        .map_err(|e| format!("Unsafe terrain spawn: {e}"))?
                        .metres(),
                );
            }
        }
        Ok(terrain)
    }
    /// Chunks may have no spawn points. Normal-world consumers require one.
    pub fn has_spawns(&self) -> bool {
        !self.spawns.is_empty()
    }
    /// Trusted capsule-bottom spawn, validated during loading.
    pub fn spawn(&self, slot: usize) -> Option<theia_core::coordinates::LocalPosition> {
        if !self.has_spawns() {
            return None;
        }
        theia_core::coordinates::LocalPosition::new(self.spawns[slot % self.spawns.len()])
    }
    pub fn below_world(&self, feet: theia_core::coordinates::LocalPosition) -> bool {
        feet.metres()[1] < self.fall_limit
    }
    /// Add the finite development-world envelope only at normal-world adoption.
    /// Individual residency chunks retain their open ownership seams.
    pub fn into_world(mut self) -> Self {
        if self.world_envelope {
            return self;
        }
        let mut colliders = self.physics.colliders.clone();
        let bottom = self.fall_limit - 2.;
        let top = self
            .solids
            .iter()
            .map(|b| b.center[1] + b.size[1] / 2.)
            .fold(self.ground_ceiling, f32::max)
            + 16.;
        let height = top - bottom;
        let centre_y = (top + bottom) / 2.;
        let lo = self.bounds[0];
        let hi = self.bounds[1];
        for side in [lo[0] - 0.5, hi[0] + 0.5] {
            colliders.push(PlacedCollider {
                shape: Collider::cuboid(1., height, hi[1] - lo[1] + 2.),
                center: Vec3::new(side, centre_y, (lo[1] + hi[1]) / 2.),
                layer: physics::SOLID,
            });
        }
        for side in [lo[1] - 0.5, hi[1] + 0.5] {
            colliders.push(PlacedCollider {
                shape: Collider::cuboid(hi[0] - lo[0], height, 1.),
                center: Vec3::new((lo[0] + hi[0]) / 2., centre_y, side),
                layer: physics::SOLID,
            });
        }
        self.physics = StaticScene::new(colliders);
        self.world_envelope = true;
        self
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
        self.physics.ground(p, self.ground_ceiling)
    }

    #[cfg(test)]
    fn standing(&self, p: [f32; 2]) -> Option<f32> {
        let ground = self.support(p)?;
        (!self.physics.overlaps(
            &physics::character_shape(0.0001),
            physics::character_center(p, ground),
            physics::SOLID,
        ))
        .then_some(ground)
    }
    fn support(&self, p: [f32; 2]) -> Option<f32> {
        if (0..2).any(|i| {
            !p[i].is_finite()
                || p[i] < self.bounds[0][i] + PAWN_RADIUS
                || p[i] > self.bounds[1][i] - PAWN_RADIUS
        }) {
            return None;
        }
        let ground = self.height(p)?;
        for dx in [-PAWN_RADIUS, 0., PAWN_RADIUS] {
            for dz in [-PAWN_RADIUS, 0., PAWN_RADIUS] {
                if !self.surface([p[0] + dx, p[1] + dz])?.1 {
                    return None;
                }
            }
        }
        Some(ground)
    }
    pub fn spawn_y(&self, xz: [f32; 2]) -> f32 {
        self.support(xz).expect("Qualified terrain support") + PAWN_Y
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
        // Avian sweeps the capsule in 3D; support policy is still grounded-only.
        let steps = (d[0].hypot(d[1]) / 0.05).ceil().max(1.) as usize;
        let mut p = start;
        for _ in 0..steps {
            for axis in 0..2 {
                if d[axis] == 0. {
                    continue;
                }
                let mut next = p;
                next[axis] += d[axis] / steps as f32;
                if let (Some(before), Some(after)) = (self.support(p), self.support(next))
                    && (after - before).abs()
                        <= ((next[axis] - p[axis]).abs() * MAX_SLOPE + 0.0001).max(self.step_height)
                {
                    let delta = Vec3::new(next[0] - p[0], after - before, next[1] - p[1]);
                    let fraction = self.physics.cast(
                        &physics::character_shape(0.),
                        physics::character_center(p, before),
                        delta,
                        physics::SOLID,
                    );
                    next[axis] = p[axis] + (next[axis] - p[axis]) * fraction;
                    // Support handles slopes/holes; Avian's sweep owns obstacle
                    // clearance. A second overlap solver at contact can disagree
                    // with the sweep's skin and incorrectly reject wall sliding.
                    if self.support(next).is_some() {
                        p = next;
                    }
                }
            }
        }
        p
    }
    /// Sampled camera clearance for the terrain patch, excluding all actors.
    pub fn camera_fraction(&self, target: [f32; 3], eye: [f32; 3]) -> f32 {
        let static_fraction = self.physics.camera_fraction(target, eye);
        // General geometry already includes floors, walls and ceilings in its
        // sweep. A highest-surface sample would mistake roofs for lower-room
        // ground and collapse a valid indoor camera to the character.
        if self.schema >= 3 {
            return static_fraction;
        }
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
fn cell(p: [f32; 2]) -> [i32; 2] {
    p.map(|v| (v / CELL_SIZE).floor() as i32)
}
// Preserve the bounded source-geometry complexity guard without building the
// old gameplay grid. Avian owns the runtime broad phase.
fn validate_coverage_budget(lo: [f32; 2], hi: [f32; 2], entries: &mut usize) -> Result<()> {
    let lo = cell(lo);
    let hi = cell(hi);
    *entries += ((hi[0] - lo[0] + 1) as usize) * ((hi[1] - lo[1] + 1) as usize);
    if *entries > MAX_COVERAGE_CELLS {
        return Err("Terrain coverage exceeds budget".into());
    }
    Ok(())
}

fn cross(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    (b[0] - a[0]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[0] - a[0])
}
#[cfg(test)]
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
    #[test]
    fn avian_mesh_matches_reference_heights_and_respects_remote_obstacles() {
        let mut triangles = Vec::new();
        for x in -64..64 {
            for z in -64..64 {
                let x = x as f32 * 4.;
                let z = z as f32 * 4.;
                let a = [x, x * 0.1, z];
                let b = [x + 4., (x + 4.) * 0.1, z];
                let c = [x + 4., (x + 4.) * 0.1, z + 4.];
                let d = [x, x * 0.1, z + 4.];
                triangles.extend([[a, c, b], [a, d, c]]);
            }
        }
        let profile = serde_json::json!({"schema":2,"source_origin":[0,0,0],
            "visual_sha256":vec![0;32],"bounds":[[-256,-256],[256,256]],
            "triangles":triangles, "solids":[
                {"center":[100,12,0],"size":[2,10,4]},
                {"center":[50,30,0],"size":[4,2,4]},
                {"center":[-100,0,0],"size":[4,1024,4]}]});
        let terrain = Terrain::decode(&serde_json::to_vec(&profile).unwrap()).unwrap();
        for i in -64..64 {
            let p = [i as f32 * 3.73, i as f32 * -2.31];
            let brute = triangles
                .iter()
                .filter_map(|t| sample(t, p))
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .unwrap()
                .0;
            assert!((terrain.height(p).unwrap() - brute).abs() < 1e-5);
        }
        let go = Intent {
            jump: false,
            axes: [1000, 0],
            sprint: true,
        };
        let mut p = [0., 0.];
        for _ in 0..450 {
            p = terrain.move_player(p, go, 1. / 30.);
        }
        assert!(p[0] <= 98.651 && p[0] > 98.59, "{p:?}");
        assert!(
            terrain.standing([50., 0.]).is_some(),
            "overhead prop permits passage"
        );
        assert!(
            terrain.standing([-100., 0.]).is_none(),
            "water zone blocks ground movement"
        );
        let target = [p[0], terrain.spawn_y(p), p[1]];
        assert!(terrain.camera_fraction(target, [110., target[1], 0.]) < 0.1);
        let mut edge = [255., 20.];
        for _ in 0..30 {
            edge = terrain.move_player(edge, go, 1. / 30.);
        }
        assert!(edge[0] <= 255.65 && edge[0] > 255.59);
        let mut invalid = profile;
        invalid["bounds"][0][0] = serde_json::json!(300);
        assert!(Terrain::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
    }

    #[test]
    fn source_budget_rejects_excessive_triangle_coverage() {
        let mut profile: serde_json::Value = serde_json::from_slice(&plane(0.)).unwrap();
        profile["schema"] = serde_json::json!(2);
        profile["triangles"] = serde_json::json!(vec![
            [
                [-1000., 0., -1000.],
                [1000., 0., 1000.],
                [1000., 0., -1000.]
            ];
            40
        ]);
        let error = Terrain::decode(&serde_json::to_vec(&profile).unwrap())
            .err()
            .unwrap();
        assert!(error.to_string().contains("coverage exceeds budget"));
    }
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
            jump: false,
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
                        jump: false,
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
                    jump: false,
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
                    jump: false,
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
                    jump: false,
                    axes: [0, -1000],
                    sprint: true,
                },
                1. / 30.,
            );
        }
        assert!((p[1] + 3.35).abs() < 0.001, "{p:?}");
        let slid = t.move_player(
            p,
            Intent {
                jump: false,
                axes: [1000, -1000],
                sprint: true,
            },
            1. / 30.,
        );
        assert!(slid[0] > p[0], "start={p:?}, slid={slid:?}");
        // Capsule contacts include a 0.5 mm skin; allow 1 mm solver tolerance.
        assert!((slid[1] - p[1]).abs() < 0.001, "start={p:?} slid={slid:?}");
        assert!(
            t.move_player(
                p,
                Intent {
                    jump: false,
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
                    jump: false,
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
                    jump: false,
                    axes: [1000, 0],
                    sprint: true,
                },
                1. / 30.,
            );
        }
        assert!((p[0] - 2.65).abs() < 0.001, "{p:?}");
        for _ in 0..16 {
            p = t.move_player(
                p,
                Intent {
                    jump: false,
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
                    jump: false,
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
