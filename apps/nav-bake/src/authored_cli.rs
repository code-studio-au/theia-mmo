//! Bounded offline authored-world navigation publication; existing format/runtime.
use std::{collections::BTreeMap, fs, io::Read, path::Path, time::Instant};
use theia_core::{content::ContentId, coordinates::GlobalPosition};
use theia_network::{
    Result,
    authored_world::Store,
    navigation::{self as nav, Agent, Manifest, Route, TileRecord},
    procedural,
};

pub(super) fn bake(source: &Path, output: &Path, route: &Path) -> Result<()> {
    let started = Instant::now();
    if output.exists() {
        return Err("Navigation output exists; choose a new directory".into());
    }
    let store = Store::open(source)?;
    let (request, endpoints) = request(route)?;
    let centre = procedural::owner(endpoints[0]).ok_or("Route outside prediction domain")?;
    let agent = Agent::development();
    let mut tiles = Vec::new();
    let mut inputs = Vec::new();
    for id in procedural::neighbours(centre, 1) {
        let patch = store.patch(id)?;
        let geometry = theia_nav_bake::authored::geometry(&patch.scene.colliders)?;
        inputs.push(serde_json::json!({"tile":[id.x,id.z],"vertices":geometry.positions.len(),"triangles":geometry.triangles.len()}));
        eprintln!(
            "authored_navigation_input x={} z={} vertices={} triangles={}",
            id.x,
            id.z,
            geometry.positions.len(),
            geometry.triangles.len()
        );
        let tile = theia_nav_bake::bake(id, store.identity(), agent, geometry)
            .map_err(|error| format!("Authored navigation tile [{},{}]: {error}", id.x, id.z))?;
        eprintln!(
            "authored_navigation_tile x={} z={} polygons={}",
            id.x,
            id.z,
            tile.polygons.len()
        );
        tiles.push(tile);
    }
    let query = theia_nav_bake::Query::new(&tiles)?;
    let points = routed_points(&store, &query, &endpoints)?;
    let encoded = tiles
        .iter()
        .map(nav::Tile::encode)
        .collect::<Result<Vec<_>>>()?;
    let manifest = Manifest {
        schema: 1,
        source: store.identity(),
        agent,
        tiles: tiles
            .iter()
            .zip(&encoded)
            .map(|(t, b)| TileRecord {
                id: t.id,
                sha256: theia_network::terrain::sha256(b),
            })
            .collect(),
        routes: vec![Route {
            id: request.id,
            points,
        }],
    };
    let bytes = manifest.encode()?;
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(output)?;
    let result = (|| -> Result<()> {
        for (tile, bytes) in tiles.iter().zip(encoded) {
            super::write_new(&output.join(nav::tile_name(tile.id)), &bytes)?;
        }
        let partial = output.join("navigation.manifest.partial");
        super::write_new(&partial, &bytes)?;
        fs::rename(&partial, output.join(nav::MANIFEST))?;
        verify(source, output)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(output);
    }
    result?;
    println!(
        "{}",
        serde_json::json!({"authored_navigation_bake":{"passed":true,"source":store.identity(),"tiles":tiles.len(),"inputs":inputs,"route_points":manifest.routes[0].points.len(),"elapsed_ms":started.elapsed().as_millis(),"scope":"Offline navigation from admitted authored collision; no creature runtime or exhaustive containment acceptance"}})
    );
    Ok(())
}
// Read-only queries reuse the same route limits, strict layer sampling and
// collision checks as published patrols. They never modify a package or actor.
fn request(route: &Path) -> Result<(Route, Vec<GlobalPosition>)> {
    let mut bytes = Vec::new();
    fs::File::open(route)?
        .take(32 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32 * 1024 {
        return Err("Authored route file exceeds budget".into());
    }
    let request: Route = serde_json::from_slice(&bytes)?;
    if ContentId::new(&request.id).is_none()
        || !(2..=nav::MAX_ROUTE_POINTS).contains(&request.points.len())
    {
        return Err("Authored route requires a logical ID and 2..256 global waypoints".into());
    }
    let endpoints = request
        .points
        .iter()
        .map(|&p| GlobalPosition::new(p).ok_or("Invalid route waypoint".into()))
        .collect::<Result<Vec<_>>>()?;
    Ok((request, endpoints))
}
fn routed_points(
    store: &Store,
    query: &theia_nav_bake::Query,
    endpoints: &[GlobalPosition],
) -> Result<Vec<[f64; 3]>> {
    let mut grounding = Grounding::new(store);
    let mut points = Vec::new();
    for pair in endpoints.windows(2) {
        for point in query.route(pair[0], pair[1]).map_err(|error| {
            format!(
                "Route {:?} -> {:?}: {error}",
                pair[0].metres(),
                pair[1].metres()
            )
        })? {
            let point = grounding.at(point)?.metres();
            if points.last() != Some(&point) {
                points.push(point);
            }
            if points.len() > nav::MAX_ROUTE_POINTS {
                return Err("Authored patrol route exceeds point budget".into());
            }
        }
    }
    Ok(points)
}
pub(super) fn route(source: &Path, navigation: &Path, request_path: &Path) -> Result<()> {
    let store = Store::open(source)?;
    let nav = nav::Store::open(navigation, store.identity())?;
    nav.verify()?;
    let (request, endpoints) = request(request_path)?;
    let tiles = nav
        .manifest()
        .tiles
        .iter()
        .map(|t| nav.tile(t.id))
        .collect::<Result<Vec<_>>>()?;
    let query = theia_nav_bake::Query::new(&tiles)?;
    let points = routed_points(&store, &query, &endpoints)?;
    let mut grounding = Grounding::new(&store);
    let mut samples = 0;
    clearance(&query, &mut grounding, &points, &mut samples)?;
    println!(
        "{}",
        serde_json::json!({"authored_navigation_route": {
            "passed": true, "source": store.identity(),
            "route": Route { id: request.id, points }, "clearance_samples": samples,
            "scope": "Read-only offline route with 0.2 m sampled capsule clearance; no actor movement or continuous sweep proof"
        }})
    );
    Ok(())
}
struct Grounding<'a> {
    store: &'a Store,
    patches: BTreeMap<theia_core::coordinates::ChunkId, procedural::Patch>,
}
impl<'a> Grounding<'a> {
    fn new(store: &'a Store) -> Self {
        Self {
            store,
            patches: BTreeMap::new(),
        }
    }
    fn at(&mut self, point: GlobalPosition) -> Result<GlobalPosition> {
        let id = procedural::owner(point).ok_or("Route outside prediction domain")?;
        if let std::collections::btree_map::Entry::Vacant(entry) = self.patches.entry(id) {
            entry.insert(self.store.patch(id)?);
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
        let local = theia_network::kinematic::spawn(&patch.scene, local)?;
        theia_network::kinematic::validate_pose(&patch.scene, local, true)?;
        patch
            .frame
            .to_global(local)
            .ok_or("Invalid global route".into())
    }
}
pub(super) fn verify(source: &Path, output: &Path) -> Result<()> {
    let store = Store::open(source)?;
    let nav = nav::Store::open(output, store.identity())?;
    nav.verify()?;
    let tiles = nav
        .manifest()
        .tiles
        .iter()
        .map(|t| nav.tile(t.id))
        .collect::<Result<Vec<_>>>()?;
    let query = theia_nav_bake::Query::new(&tiles)?;
    let mut points = 0;
    let mut samples = 0usize;
    let mut grounding = Grounding::new(&store);
    for route in &nav.manifest().routes {
        for p in &route.points {
            let p = GlobalPosition::new(*p).ok_or("Invalid route point")?;
            query.sample(p)?;
            grounding.at(p)?;
            points += 1;
        }
        clearance(&query, &mut grounding, &route.points, &mut samples)?;
    }
    println!(
        "{}",
        serde_json::json!({"authored_navigation_verify":{"passed":true,"tiles":tiles.len(),"route_points":points,"clearance_samples":samples,"source":store.identity(),"scope":"Package, navigation and 0.2 m sampled route capsule clearance; native creature acceptance remains separate"}})
    );
    Ok(())
}

fn clearance(
    query: &theia_nav_bake::Query,
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
