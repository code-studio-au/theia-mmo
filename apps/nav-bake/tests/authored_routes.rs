//! Local proprietary starter-package qualification; no native renderer or actors.
use theia_core::coordinates::GlobalPosition;
use theia_network::{authored_world, navigation};

#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD and THEIA_AUTHORED_NAV starter packages"]
fn actual_starter_navigation_uses_the_single_opening_for_all_four_exit_routes() {
    let source = std::env::var_os("THEIA_AUTHORED_WORLD").expect("Authored world");
    let output = std::env::var_os("THEIA_AUTHORED_NAV").expect("Authored navigation");
    let store = authored_world::Store::open(std::path::Path::new(&source)).unwrap();
    let nav = navigation::Store::open(std::path::Path::new(&output), store.identity()).unwrap();
    let tiles = nav
        .manifest()
        .tiles
        .iter()
        .map(|t| nav.tile(t.id).unwrap())
        .collect::<Vec<_>>();
    let query = theia_nav_bake::Query::new(&tiles).unwrap();
    let start = GlobalPosition::new([8000., 0., 8000.]).unwrap();
    for [dx, dz] in [[70., 0.], [-70., 0.], [0., -70.], [0., 70.]] {
        let end = GlobalPosition::new([8000. + dx, 0., 8000. + dz]).unwrap();
        let route = query
            .route(start, end)
            .unwrap_or_else(|error| panic!("Flat exterior endpoint {dx},{dz}: {error}"));
        let mut crossing = None;
        for pair in route.windows(2) {
            let a = pair[0].metres();
            let b = pair[1].metres();
            let distance = ((b[0] - a[0]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
            let steps = (distance / 0.2).ceil().max(1.) as usize;
            for step in 0..=steps {
                let f = step as f64 / steps as f64;
                let x = a[0] + (b[0] - a[0]) * f - 8000.;
                let z = a[2] + (b[2] - a[2]) * f - 8000.;
                if x * x + z * z >= 48_f64.powi(2) {
                    crossing = Some([x, z]);
                    break;
                }
            }
            if crossing.is_some() {
                break;
            }
        }
        let [x, z] = crossing.expect("Route must leave the basin");
        assert!(
            z > 0. && x.abs() < 10.,
            "Exit {dx},{dz} bypassed opening at {x},{z}"
        );
        let last = route.last().unwrap().metres();
        assert!(
            (last[0] - end.metres()[0]).abs() < 0.25 && (last[2] - end.metres()[2]).abs() < 0.25
        );
        println!(
            "starter_navigation_exit target={dx},{dz} crossing={x},{z} route_points={}",
            route.len()
        );
    }
}

#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD and THEIA_AUTHORED_NAV starter packages"]
fn actual_authored_patrol_completes_a_loop_with_authoritative_static_collision() {
    use std::sync::Arc;
    use theia_network::{
        procedural,
        simulation::{World, creatures},
    };
    let source = std::env::var_os("THEIA_AUTHORED_WORLD").expect("Authored world");
    let output = std::env::var_os("THEIA_AUTHORED_NAV").expect("Authored navigation");
    let store = Arc::new(authored_world::Store::open(std::path::Path::new(&source)).unwrap());
    let source = procedural::Source::Authored(store);
    let nav = navigation::Store::open(std::path::Path::new(&output), source.identity()).unwrap();
    let profile = creatures::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-creature-patrol.json"
    ))
    .unwrap();
    let prepared = creatures::Prepared::load(profile, &nav, &source).unwrap();
    let mut world = World::with_terrain_source([8000., 8000.], 1., source).unwrap();
    world.enable_prediction().unwrap();
    world.enable_patrol(prepared).unwrap();
    world.enable_classic().unwrap();
    let bodies = || {
        world
            .procedural()
            .unwrap()
            .patches
            .values()
            .map(|p| p.scene.colliders.len())
            .sum::<usize>()
    };
    let before = bodies();
    for _ in 0..900 {
        world.step();
    }
    let after = world
        .procedural()
        .unwrap()
        .patches
        .values()
        .map(|p| p.scene.colliders.len())
        .sum::<usize>();
    assert_eq!(
        before, after,
        "Patrol actors must never become world collision"
    );
    let snapshot = world.patrol_snapshot().unwrap();
    assert_eq!(snapshot.blocked, 0);
    assert!(snapshot.loops >= 1 && snapshot.travelled_metres > 30.);
    println!(
        "{}",
        serde_json::json!({"authored_patrol":snapshot,"scope":"900 authoritative fixed steps with actual Synty static collision; no network, rendering, quest or persistence acceptance"})
    );
}

#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD dressed 512m valley near 8192,8192"]
fn actual_dressed_valley_hut_and_grove_tiles_bake_without_dropping_collision() {
    let source = std::env::var_os("THEIA_AUTHORED_WORLD").expect("Dressed valley");
    let store = authored_world::Store::open(std::path::Path::new(&source)).unwrap();
    for id in [
        theia_core::coordinates::ChunkId { x: 63, z: 64 },
        theia_core::coordinates::ChunkId { x: 64, z: 63 },
    ] {
        let patch = store.patch(id).unwrap();
        let geometry = theia_nav_bake::authored::geometry(&patch.scene.colliders).unwrap();
        println!(
            "tile={id:?} vertices={} triangles={}",
            geometry.positions.len(),
            geometry.triangles.len()
        );
        let tile = theia_nav_bake::bake(
            id,
            store.identity(),
            navigation::Agent::development(),
            geometry,
        )
        .unwrap();
        tile.validate().unwrap();
        println!("tile={id:?} polygons={}", tile.polygons.len());
    }
}

#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD valley and THEIA_AUTHORED_NAV west-grove package"]
fn actual_valley_patrol_completes_its_route_on_authoritative_scenery_collision() {
    use std::sync::Arc;
    use theia_network::{
        procedural,
        simulation::{World, creatures},
    };
    let world_path = std::env::var_os("THEIA_AUTHORED_WORLD").unwrap();
    let nav_path = std::env::var_os("THEIA_AUTHORED_NAV").unwrap();
    let store = Arc::new(authored_world::Store::open(std::path::Path::new(&world_path)).unwrap());
    let spawn = store.spawn(0).unwrap().metres();
    let source = procedural::Source::Authored(store);
    let nav = navigation::Store::open(std::path::Path::new(&nav_path), source.identity()).unwrap();
    let profile = creatures::Profile::from_json(include_bytes!(
        "../../../content/gameplay/starter-valley-wolf-patrol.json"
    ))
    .unwrap();
    let prepared = creatures::Prepared::load(profile, &nav, &source).unwrap();
    let mut world = World::with_terrain_source([spawn[0], spawn[2]], 1., source).unwrap();
    world.enable_prediction().unwrap();
    world.enable_classic().unwrap();
    world.enable_patrol(prepared).unwrap();
    for _ in 0..1200 {
        world.step();
    }
    let snapshot = world.patrol_snapshot().unwrap();
    assert_eq!(snapshot.blocked, 0);
    assert!(snapshot.loops >= 1 && snapshot.travelled_metres > 40.);
    println!(
        "{}",
        serde_json::json!({"valley_patrol":snapshot,
        "scope":"1200 authoritative fixed steps; no network, art, combat, quest or persistence acceptance"})
    );
}
