//! Actual CLI checks against locally owned authored packages; no live actors.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use theia_core::spawn_volume::PolygonVolume;
use theia_network::navigation::spawn_candidates::Request;

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
fn authored_spawn_cli_is_repeatable_rejects_wrong_layers_and_preserves_packages() {
    let world = PathBuf::from(std::env::var_os("THEIA_AUTHORED_WORLD").expect("world"));
    let nav = PathBuf::from(std::env::var_os("THEIA_AUTHORED_NAV").expect("navigation"));
    let request_path = PathBuf::from(std::env::var_os("THEIA_AUTHORED_SPAWNS").expect("request"));
    let bytes = fs::read(&request_path).unwrap();
    let request: Request = serde_json::from_slice(&bytes).unwrap();
    let retained = [
        world.join("authoring.zone.json"),
        world.join("terrain.json"),
        nav.join("navigation.manifest.json"),
        request_path.clone(),
    ]
    .map(|p| {
        let b = fs::read(&p).unwrap();
        (p, b)
    });
    let scratch = Scratch(std::env::temp_dir().join(format!(
            "theia-spawn-cli-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&scratch.0).unwrap();
    let run = |path: &Path| {
        Command::new(env!("CARGO_BIN_EXE_theia-nav-bake"))
            .args(["spawns-authored"])
            .arg(&world)
            .arg(&nav)
            .arg(path)
            .output()
            .unwrap()
    };
    let first = run(&request_path);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let report: serde_json::Value = serde_json::from_str(
        String::from_utf8_lossy(&first.stdout)
            .lines()
            .last()
            .unwrap(),
    )
    .unwrap();
    let result = &report["authored_spawn_candidates"];
    assert_eq!(result["passed"], true);
    let points: Vec<[f64; 3]> = serde_json::from_value(result["accepted"].clone()).unwrap();
    assert_eq!(points.len(), request.count);
    for (i, point) in points.iter().enumerate() {
        assert!(
            request
                .volume
                .contains(theia_core::coordinates::GlobalPosition::new(*point).unwrap())
        );
        assert!(
            points[..i]
                .iter()
                .all(|other| (point[0] - other[0]).hypot(point[2] - other[2])
                    >= request.minimum_spacing_metres)
        );
    }
    assert_eq!(run(&request_path).stdout, first.stdout);
    let mut high = request.clone();
    high.volume = PolygonVolume::new(high.volume.polygon().to_vec(), 90., 100.).unwrap();
    let file = scratch.0.join("wrong-height.json");
    fs::write(&file, serde_json::to_vec(&high).unwrap()).unwrap();
    assert!(!run(&file).status.success());
    let mut inaccessible = request.clone();
    inaccessible.access_from[1] += 20.;
    fs::write(&file, serde_json::to_vec(&inaccessible).unwrap()).unwrap();
    assert!(!run(&file).status.success());
    let mut crowded = request.clone();
    crowded.count = 64;
    crowded.minimum_spacing_metres = 32.;
    fs::write(&file, serde_json::to_vec(&crowded).unwrap()).unwrap();
    let failed = run(&file);
    assert!(!failed.status.success());
    let partial: serde_json::Value = serde_json::from_str(
        String::from_utf8_lossy(&failed.stdout)
            .lines()
            .last()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(partial["authored_spawn_candidates"]["passed"], false);
    assert!(
        partial["authored_spawn_candidates"]["accepted"]
            .as_array()
            .unwrap()
            .len()
            < 64
    );
    let foreign = scratch.0.join("foreign-navigation");
    fs::create_dir(&foreign).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(nav.join("navigation.manifest.json")).unwrap()).unwrap();
    manifest["source"] = serde_json::json!(vec![0_u8; 32]);
    fs::write(
        foreign.join("navigation.manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let refused = Command::new(env!("CARGO_BIN_EXE_theia-nav-bake"))
        .arg("spawns-authored")
        .arg(&world)
        .arg(&foreign)
        .arg(&request_path)
        .output()
        .unwrap();
    assert!(!refused.status.success());
    for (path, bytes) in retained {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    println!(
        "authored_spawn_cli passed=true repeatable=true wrong_height_refused=true inaccessible_anchor_refused=true insufficient_candidates_reported=true source_preserved=true report={report}"
    );
}
