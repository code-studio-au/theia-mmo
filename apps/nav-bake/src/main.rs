//! Offline saved-terrain navigation tool. No renderer or server process is started.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalFrame, LocalPosition};
use theia_network::{
    Result,
    navigation::{self as nav, Agent, Manifest, Route, TileRecord},
    procedural::{self, Geometry},
    saved_terrain,
};
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn geometry(store: &saved_terrain::Store, id: ChunkId) -> Result<Geometry> {
    let frame = LocalFrame::new(id.origin(128).ok_or("Invalid bake origin")?);
    let mut result = Geometry {
        positions: vec![],
        triangles: vec![],
    };
    // Full-resolution collision triangles and neighboring geometry precede erosion.
    for neighbour in procedural::neighbours(id, 1) {
        let g = store.geometry(neighbour, 32)?;
        let offset = frame
            .to_local(neighbour.origin(128).ok_or("Invalid source tile origin")?)
            .ok_or("Source tile outside bake frame")?
            .metres();
        let base = result.positions.len() as u32;
        result.positions.extend(
            g.positions
                .iter()
                .map(|p| std::array::from_fn(|i| p[i] + offset[i])),
        );
        result
            .triangles
            .extend(g.triangles.iter().map(|t| t.map(|i| i + base)));
    }
    Ok(result)
}
fn surface(store: &saved_terrain::Store, x: f64, z: f64) -> Result<GlobalPosition> {
    let point = GlobalPosition::new([x, 0., z]).ok_or("Invalid route endpoint")?;
    let id = procedural::owner(point).ok_or("Endpoint outside saved world")?;
    let frame = LocalFrame::new(id.origin(128).ok_or("Invalid endpoint frame")?);
    let p = frame
        .to_local(point)
        .ok_or("Endpoint outside local frame")?
        .metres();
    let gx = p[0] / 4.;
    let gz = p[2] / 4.;
    let cx = (gx.floor() as usize).min(31);
    let cz = (gz.floor() as usize).min(31);
    let fx = gx - cx as f32;
    let fz = gz - cz as f32;
    let g = store.geometry(id, 32)?;
    let a = cz * 33 + cx;
    let [a, b, c, d] = [a, a + 1, a + 33, a + 34].map(|i| g.positions[i][1]);
    let height = if fx + fz <= 1. {
        a + (b - a) * fx + (c - a) * fz
    } else {
        d + (c - d) * (1. - fx) + (b - d) * (1. - fz)
    };
    GlobalPosition::new([x, f64::from(height), z])
        .ok_or_else(|| "Invalid supported endpoint".into())
}
fn bake(source: &Path, output: &Path, centre: ChunkId) -> Result<()> {
    let started = Instant::now();
    if output.exists() {
        return Err("Navigation output already exists; choose a fresh package directory".into());
    }
    if !procedural::contains(centre) {
        return Err("Navigation centre is outside the saved world".into());
    }
    let store = saved_terrain::Store::open(source)?;
    let agent = Agent::development();
    let mut tiles = Vec::new();
    for id in procedural::neighbours(centre, 1) {
        let tile_started = Instant::now();
        let tile = theia_nav_bake::bake(id, store.identity(), agent, geometry(&store, id)?)?;
        eprintln!(
            "navigation_tile_baked x={} z={} polygons={} detail_vertices={} elapsed_ms={}",
            id.x,
            id.z,
            tile.polygons.len(),
            tile.detail.vertices.len(),
            tile_started.elapsed().as_millis()
        );
        tiles.push(tile);
    }
    let query = theia_nav_bake::Query::new(&tiles)?;
    let origin = centre.origin(128).ok_or("Invalid patrol origin")?.metres();
    let endpoints = [[72., 64.], [104., 88.], [48., 96.], [72., 64.]]
        .map(|[x, z]| surface(&store, origin[0] + x, origin[2] + z))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let mut points = Vec::new();
    for endpoints in endpoints.windows(2) {
        for point in query.route(endpoints[0], endpoints[1])? {
            let point = point.metres();
            let point = surface(&store, point[0], point[2])?.metres();
            if points.last().is_none_or(|p| p != &point) {
                points.push(point);
            }
            if points.len() > nav::MAX_ROUTE_POINTS {
                return Err("Patrol route exceeds point budget".into());
            }
        }
    }
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
            id: "theia.dev.training-patrol".into(),
            points,
        }],
    };
    let manifest_bytes = manifest.encode()?;
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(output)?;
    for (tile, bytes) in tiles.iter().zip(encoded) {
        write_new(&output.join(nav::tile_name(tile.id)), &bytes)?;
    }
    // The final manifest is the publication point; failed earlier writes leave
    // no loadable package. Existing output files are never overwritten.
    let partial = output.join("navigation.manifest.partial");
    write_new(&partial, &manifest_bytes)?;
    fs::rename(&partial, output.join(nav::MANIFEST))?;
    if let Err(error) = nav::Store::open(output, store.identity()).and_then(|s| s.verify()) {
        fs::rename(output.join(nav::MANIFEST), &partial)?;
        return Err(error);
    }
    println!(
        "{}",
        serde_json::json!({"navigation_bake":{"passed":true,"terrain_only":true,"tiles":tiles.len(),"polygons":tiles.iter().map(|t|t.polygons.len()).sum::<usize>(),"vertices":tiles.iter().map(|t|t.vertices.len()).sum::<usize>(),"route_points":manifest.routes[0].points.len(),"source":store.identity(),"elapsed_ms":started.elapsed().as_millis(),"rerecast":"0.5.0","landmass":"0.9.3","runtime_integration":false}})
    );
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|v| v == "--help") {
        println!(
            "theia-nav-bake bake SOURCE OUTPUT [CENTRE_X CENTRE_Z]\ntheia-nav-bake verify SOURCE OUTPUT\nBakes at most nine 128m saved-terrain tiles and one development patrol route."
        );
        return Ok(());
    }
    match args.first().map(String::as_str) {
        Some("bake") if args.len() == 3 || args.len() == 5 => {
            let centre = if args.len() == 5 {
                ChunkId {
                    x: args[3].parse()?,
                    z: args[4].parse()?,
                }
            } else {
                ChunkId { x: 62, z: 62 }
            };
            bake(Path::new(&args[1]), Path::new(&args[2]), centre)
        }
        Some("verify") if args.len() == 3 => {
            let source = saved_terrain::Store::open(Path::new(&args[1]))?;
            let nav = nav::Store::open(Path::new(&args[2]), source.identity())?;
            nav.verify()?;
            let tiles = nav
                .manifest()
                .tiles
                .iter()
                .map(|t| nav.tile(t.id))
                .collect::<Result<Vec<_>>>()?;
            let query = theia_nav_bake::Query::new(&tiles)?;
            let mut sampled = 0;
            let mut missed = 0;
            let mut maximum_height_error = 0_f64;
            for tile in &tiles {
                let id = ChunkId {
                    x: tile.id[0],
                    z: tile.id[1],
                };
                let frame = LocalFrame::new(id.origin(128).ok_or("Invalid verification frame")?);
                let geometry = source.geometry(id, 32)?;
                for z in (2..33).step_by(4) {
                    for x in (2..33).step_by(4) {
                        let point = frame
                            .to_global(
                                LocalPosition::new(geometry.positions[z * 33 + x])
                                    .ok_or("Invalid source sample")?,
                            )
                            .ok_or("Invalid global source sample")?;
                        match query.sample(point) {
                            Ok(p) => {
                                sampled += 1;
                                maximum_height_error = maximum_height_error
                                    .max((p.metres()[1] - point.metres()[1]).abs());
                            }
                            Err(_) => missed += 1,
                        }
                    }
                }
            }
            println!(
                "{}",
                serde_json::json!({"navigation_verify":{"passed":true,"tiles":nav.manifest().tiles.len(),"identity":nav.identity(),"runtime_integration":false,"height_samples":sampled,"height_misses":missed,"max_height_error_metres":maximum_height_error}})
            );
            Ok(())
        }
        _ => Err("Use --help for the offline navigation bake/verify commands".into()),
    }
}
