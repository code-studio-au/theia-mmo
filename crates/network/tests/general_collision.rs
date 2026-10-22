//! Schema-3 geometry and explicit spawns, independent of legacy height terrain.
use serde_json::{Value, json};
use theia_core::time::TickRate;
use theia_network::{simulation::World, terrain::Terrain, wire::*};
fn floor(y: f32) -> Vec<[[f32; 3]; 3]> {
    vec![
        [[20., y, 20.], [40., y, 40.], [40., y, 20.]],
        [[20., y, 20.], [20., y, 40.], [40., y, 40.]],
    ]
}
fn profile(ceiling: Option<f32>) -> Value {
    let mut triangles = floor(0.);
    if let Some(y) = ceiling {
        triangles.extend(floor(y).into_iter().map(|[a, b, c]| [a, c, b]));
    }
    triangles.extend([
        [[30., 0., 20.], [30., 4., 20.], [30., 4., 40.]],
        [[30., 0., 20.], [30., 4., 40.], [30., 0., 40.]],
    ]);
    json!({"schema":3,"bounds":[[20,20],[40,40]],"source_origin":[0,0,0],
        "visual_sha256":vec![0;32],"triangles":triangles,"spawns":[[24,0,24]]})
}
fn load(p: &Value) -> theia_network::Result<Terrain> {
    Terrain::decode(&serde_json::to_vec(p).unwrap())
}
fn step(world: &mut World, id: u64, seq: u64, axes: [i16; 2], jump: bool) -> Player {
    assert!(world.input(
        id,
        Input {
            version: VERSION,
            epoch: world.epoch(),
            sequence: seq,
            intent: Intent {
                axes,
                sprint: false,
                jump
            }
        }
    ));
    world
        .step()
        .players
        .into_iter()
        .find(|p| p.id == id)
        .unwrap()
}
#[test]
fn combined_floor_wall_ceiling_mesh_has_authoritative_headroom_and_wall_collision() {
    for hz in [20, 30, 60] {
        let mut world = World::with_terrain(load(&profile(Some(2.1))).unwrap());
        assert!(world.set_tick_rate(TickRate::from_hz(hz).unwrap()));
        let id = world.join().unwrap();
        let mut max_y = 0f32;
        for seq in 1..=hz * 2 {
            let p = step(&mut world, id, u64::from(seq), [1000, 0], true);
            max_y = max_y.max(p.y);
            assert!(p.y - 0.9 + 1.8 < 2.101, "Ceiling penetration: {p:?}");
            assert!(p.xz[0] < 29.651, "Mesh wall penetration: {p:?}");
        }
        assert!(
            max_y > 1.05 && max_y < 1.201,
            "Ceiling limits jump: {max_y}"
        );
        assert_eq!(world.movement_recoveries(), 0);
    }
}
#[test]
fn tight_combined_ceiling_does_not_hide_floor_support() {
    let terrain = load(&profile(Some(1.815))).unwrap();
    let mut world = World::with_terrain(terrain);
    let id = world.join().unwrap();
    for seq in 1..60 {
        let p = step(&mut world, id, seq, [0, 0], false);
        assert!((p.y - 0.908).abs() < 0.001, "{p:?}");
    }
    assert_eq!(world.movement_recoveries(), 0);
}
#[test]
fn explicit_lower_spawn_does_not_select_highest_floor_and_chunk_without_spawns_loads() {
    let mut p = profile(None);
    let mut triangles = floor(0.);
    triangles.extend(floor(3.));
    p["triangles"] = json!(triangles);
    p["spawns"] = json!([[24, 0, 24], [26, 3, 24]]);
    let mut world = World::with_terrain(load(&p).unwrap());
    let lower = world.join().unwrap();
    let upper = world.join().unwrap();
    for _ in 0..30 {
        world.step();
    }
    let s = world.step();
    assert!((s.players.iter().find(|p| p.id == lower).unwrap().y - 0.908).abs() < 0.001);
    assert!((s.players.iter().find(|p| p.id == upper).unwrap().y - 3.908).abs() < 0.001);
    p.as_object_mut().unwrap().remove("spawns");
    let chunk = load(&p).unwrap();
    assert!(!chunk.has_spawns());
    assert!(chunk.spawn(0).is_none());
    assert!(World::with_terrain(chunk).join().is_none());
}
#[test]
fn invalid_general_geometry_and_spawns_fail_closed() {
    let source = profile(None);
    for (field, value) in [
        ("bounds", Value::Null),
        ("spawns", json!([[0, 0, 0]])),
        ("spawns", json!([[24, 2, 24]])),
        ("spawns", json!([[30, 0, 24]])),
        ("spawns", json!(vec![[24, 0, 24]; 9])),
        (
            "triangles",
            json!([[[20, 0, 20], [21, 0, 21], [22, 0, 22]]]),
        ),
        ("step_height", json!(0.2)),
        ("walk_surfaces", json!(floor(0.))),
    ] {
        let mut p = source.clone();
        p[field] = value;
        assert!(load(&p).is_err(), "Accepted {p}");
    }
}

#[test]
fn mesh_room_camera_uses_nearby_ceiling_instead_of_highest_surface() {
    let terrain = load(&profile(Some(2.1))).unwrap();
    let target = [24., 0.908, 24.];
    let eye = [24., 5., 30.];
    let fraction = terrain.camera_fraction(target, eye);
    // Independent 15 cm camera-box clearance below the 2.1 m ceiling.
    let expected = (2.1 - 0.15 - target[1]) / (eye[1] - target[1]);
    assert!(
        (fraction - expected).abs() < 0.001,
        "{fraction} vs {expected}"
    );
    assert!(terrain.camera_fraction(target, [24., 1.1, 26.]) > 0.999);
}
