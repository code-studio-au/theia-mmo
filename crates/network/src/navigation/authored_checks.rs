//! Shared loader/worker capsule checks for authored routes and spawn candidates.
//! May load collision patches; never call from the fixed simulation step.
use super::query::Query;
use crate::{Result, authored_world::Store, procedural};
use std::{collections::BTreeMap, sync::Arc};
use theia_core::coordinates::GlobalPosition;

pub struct Grounding<'a> {
    store: &'a Store,
    patches: BTreeMap<theia_core::coordinates::ChunkId, Arc<procedural::Patch>>,
}
impl<'a> Grounding<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self {
            store,
            patches: BTreeMap::new(),
        }
    }
    pub(crate) fn into_patches(self) -> Vec<Arc<procedural::Patch>> {
        self.patches.into_values().collect()
    }
    pub fn at(&mut self, point: GlobalPosition) -> Result<GlobalPosition> {
        let id = procedural::owner(point).ok_or("Route outside prediction domain")?;
        if let std::collections::btree_map::Entry::Vacant(entry) = self.patches.entry(id) {
            entry.insert(Arc::new(self.store.patch(id)?));
        }
        let patch = &self.patches[&id];
        let mut local = patch
            .frame
            .to_local(point)
            .ok_or("Route outside local frame")?
            .metres();
        let (height, walkable) = patch
            .scene
            .ground([local[0], local[2]], local[1] + 0.3)
            .ok_or("Route has no close collision surface")?;
        if !walkable || (height - local[1]).abs() > 0.3 {
            return Err(format!(
                "Navigation route at {:?} disagrees with collision height {} (walkable={walkable})",
                point.metres(),
                height
            )
            .into());
        }
        local[1] = height + 0.03;
        let local =
            theia_core::coordinates::LocalPosition::new(local).ok_or("Invalid route pose")?;
        let local = crate::kinematic::spawn(&patch.scene, local)?;
        crate::kinematic::validate_pose(&patch.scene, local, true).map_err(|error| {
            format!(
                "Route grounding at {:?}, local feet {:?}: {error}",
                point.metres(),
                local.metres()
            )
        })?;
        patch
            .frame
            .to_global(local)
            .ok_or("Invalid global route".into())
    }
}
pub fn clearance(
    query: &Query,
    grounding: &mut Grounding<'_>,
    points: &[[f64; 3]],
    samples: &mut usize,
) -> Result<()> {
    for segment in points.windows(2) {
        let distance = (0..3)
            .map(|i| (segment[1][i] - segment[0][i]).powi(2))
            .sum::<f64>()
            .sqrt();
        let steps = (distance / 0.2).ceil().max(1.) as usize;
        *samples = samples
            .checked_add(steps + 1)
            .filter(|n| *n <= 8192)
            .ok_or("Authored route clearance sample budget exceeded")?;
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            let point = GlobalPosition::new(std::array::from_fn(|i| {
                segment[0][i] + (segment[1][i] - segment[0][i]) * fraction
            }))
            .ok_or("Invalid route interpolation")?;
            query
                .sample(point)
                .map_err(|error| format!("Route clearance at {:?}: {error}", point.metres()))?;
            grounding.at(point)?;
        }
    }
    Ok(())
}

/// Prepare bounded, continuously grounded static travel against the exact source.
/// Loader/editor-worker work only: loads navigation tiles and collision patches.
/// The returned waypoints describe a checked route, never authoritative motion.
pub fn grounded_route(
    world: &Store,
    navigation: &super::Store,
    waypoints: &[[f64; 3]],
) -> Result<Vec<GlobalPosition>> {
    if navigation.manifest().source != world.identity() {
        return Err("Travel navigation belongs to another authored world".into());
    }
    if !(2..=super::MAX_ROUTE_POINTS).contains(&waypoints.len()) {
        return Err("Travel requires 2..256 waypoints".into());
    }
    navigation.verify()?;
    let tiles = navigation
        .manifest()
        .tiles
        .iter()
        .map(|t| navigation.tile(t.id))
        .collect::<Result<Vec<_>>>()?;
    let query = Query::new(&tiles)?;
    let mut ground = Grounding::new(world);
    let waypoints = waypoints
        .iter()
        .map(|&p| GlobalPosition::new(p).ok_or("Invalid travel waypoint".into()))
        .collect::<Result<Vec<_>>>()?;
    let mut points = Vec::new();
    let mut samples = 0;
    for pair in waypoints.windows(2) {
        let path = query.route(pair[0], pair[1])?;
        for segment in path.windows(2) {
            project_segment(
                &query,
                &mut ground,
                segment[0],
                segment[1],
                &mut points,
                &mut samples,
            )?;
        }
    }
    if !(2..=super::MAX_ROUTE_POINTS).contains(&points.len()) {
        return Err("Travel needs a nonempty checked route".into());
    }
    clearance(
        &query,
        &mut ground,
        &points.iter().map(|p| p.metres()).collect::<Vec<_>>(),
        &mut 0,
    )?;
    Ok(points)
}

/// Follow the same floor continuously, rather than interpolating a long segment's
/// endpoint heights across rolling terrain. All original strict layer tolerances
/// and collision checks remain in force. Keep metre-spaced ground points for the
/// follower, then run ordinary clearance again over their connecting segments.
fn project_segment(
    query: &Query,
    ground: &mut Grounding<'_>,
    start: GlobalPosition,
    end: GlobalPosition,
    points: &mut Vec<GlobalPosition>,
    samples: &mut usize,
) -> Result<()> {
    let mut previous = ground.at(start)?;
    let expected_end = ground.at(end)?;
    if points.last() != Some(&previous) {
        points.push(previous);
    }
    let a = start.metres();
    let b = end.metres();
    let steps = ((b[0] - a[0]).hypot(b[2] - a[2]) / 0.1).ceil().max(1.) as usize;
    *samples = samples
        .checked_add(steps)
        .filter(|n| *n <= 8192)
        .ok_or("Test route grounding sample budget exceeded")?;
    for step in 1..=steps {
        let fraction = step as f64 / steps as f64;
        let x = a[0] + (b[0] - a[0]) * fraction;
        let z = a[2] + (b[2] - a[2]) * fraction;
        let candidate = GlobalPosition::new([x, previous.metres()[1], z])
            .ok_or("Invalid projected route point")?;
        let sampled = query.sample(candidate)?;
        previous = ground
            .at(GlobalPosition::new([x, sampled.metres()[1], z]).ok_or("Invalid route height")?)?;
        if step == steps && route_distance(previous, expected_end) > 0.3 {
            return Err("Test route projection changed floor".into());
        }
        let last = points.last().unwrap().metres();
        if step == steps || (x - last[0]).hypot(z - last[2]) >= 1. {
            if points.last() != Some(&previous) {
                points.push(previous);
            }
            if points.len() > super::MAX_ROUTE_POINTS {
                return Err("Test route exceeds waypoint budget".into());
            }
        }
    }
    Ok(())
}

fn route_distance(a: GlobalPosition, b: GlobalPosition) -> f64 {
    a.metres()
        .into_iter()
        .zip(b.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
