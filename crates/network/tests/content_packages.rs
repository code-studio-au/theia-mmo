//! Package byte contracts, fail-closed loading and real authenticated admission.
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use theia_network::{
    content::Manifest,
    terrain::{Terrain, sha256},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const GOLDEN: &[u8] = include_bytes!("fixtures/content-manifest.json");
const ID: &str = "779362a91793754893713aa35b709e773164332295f7ccb3fc49d7f5756b2d26";
fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
struct Fixture {
    root: PathBuf,
    profile: Vec<u8>,
    manifest: Value,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-content-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let visual = b"Binding-test payload; not a rendered GLB";
        let profile = serde_json::to_vec(&json!({"schema":2,"bounds":[[-19.,-19.],[19.,19.]],"source_origin":[0.,0.,0.],"visual_sha256":sha256(visual),"triangles":[[[-19.,0.,-19.],[-19.,0.,19.],[19.,0.,-19.]],[[19.,0.,-19.],[-19.,0.,19.],[19.,0.,19.]]]})).unwrap();
        let manifest = json!({"schema":1,"content_id":"theia.test.flat","revision":7,"collision_schema":2,"collision_bytes":profile.len(),"collision_sha256":sha256(&profile),"visual_bytes":visual.len(),"visual_sha256":sha256(visual)});
        std::fs::write(root.join("terrain.json"), &profile).unwrap();
        std::fs::write(root.join("terrain.glb"), visual).unwrap();
        let fixture = Self {
            root,
            profile,
            manifest,
        };
        fixture.write();
        fixture
    }
    fn write(&self) {
        std::fs::write(
            self.root.join("world.manifest.json"),
            serde_json::to_vec_pretty(&self.manifest).unwrap(),
        )
        .unwrap();
    }
    fn load(&self) -> theia_network::Result<Terrain> {
        Terrain::load(&self.root.join("terrain.json"))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn schema4_heightfield_package_has_bound_identity_and_validated_spawn() {
    let mut fixture = Fixture::new();
    fixture.profile = serde_json::to_vec(&json!({
        "schema":4,"bounds":[[-19,-19],[19,19]],"source_origin":[0,0,0],
        "visual_sha256":fixture.manifest["visual_sha256"],"triangles":[],"spawns":[[2,0,2]],
        "static_shapes":{"heightfields":[{"rows":2,"columns":2,"heights_column_major":[0,0,0,0],"center":[0,0,0],"scale":[38,1,38]}]}
    })).unwrap();
    fixture.manifest["collision_schema"] = json!(4);
    fixture.manifest["collision_bytes"] = json!(fixture.profile.len());
    fixture.manifest["collision_sha256"] = json!(sha256(&fixture.profile));
    std::fs::write(fixture.root.join("terrain.json"), &fixture.profile).unwrap();
    fixture.write();
    let terrain = fixture.load().unwrap();
    terrain
        .verify_visual(&fixture.root.join("terrain.glb"))
        .unwrap();
    assert_eq!(terrain.physics.colliders.len(), 1);
    assert_eq!(terrain.spawn(0).unwrap().metres()[0], 2.);
    assert!((terrain.spawn(0).unwrap().metres()[1] - 0.008).abs() < 0.001);
    fixture.manifest["collision_schema"] = json!(3);
    fixture.write();
    assert!(fixture.load().is_err());
}

#[test]
fn golden_python_rust_identity_is_format_independent_and_revision_sensitive() {
    let manifest = Manifest::decode(GOLDEN).unwrap();
    assert_eq!(hex(manifest.identity()), ID);
    let mut document: Value = serde_json::from_slice(GOLDEN).unwrap();
    assert_eq!(
        Manifest::decode(&serde_json::to_vec(&document).unwrap())
            .unwrap()
            .identity(),
        manifest.identity()
    );
    for (field, value) in [
        ("revision", json!(8)),
        ("content_id", json!("theia.test.other")),
        ("collision_sha256", json!(vec![0; 32])),
        ("visual_bytes", json!(457)),
    ] {
        let old = document[field].clone();
        document[field] = value;
        assert_ne!(
            Manifest::decode(&serde_json::to_vec(&document).unwrap())
                .unwrap()
                .identity(),
            manifest.identity()
        );
        document[field] = old;
    }
}
#[test]
fn unsupported_malformed_and_out_of_budget_manifests_fail_closed() {
    let source: Value = serde_json::from_slice(GOLDEN).unwrap();
    for (field, value) in [
        ("schema", json!(4)),
        ("revision", json!(0)),
        ("collision_schema", json!(5)),
        ("content_id", json!("../world")),
        ("collision_bytes", json!(33554433u64)),
        ("visual_bytes", json!(2147483649u64)),
        ("visual_bytes", json!(0)),
        ("visual_sha256", json!([0, 1])),
        ("unknown", json!(true)),
    ] {
        let mut document = source.clone();
        document[field] = value;
        assert!(
            Manifest::decode(&serde_json::to_vec(&document).unwrap()).is_err(),
            "{field}"
        );
    }
    assert!(Manifest::decode(&vec![b' '; 8193]).is_err());
    assert!(Manifest::decode(&GOLDEN[..GOLDEN.len() / 2]).is_err());
    let mut trailing = GOLDEN.to_vec();
    trailing.extend_from_slice(b"{}");
    assert!(Manifest::decode(&trailing).is_err());
}
#[test]
fn headless_load_binds_collision_schema_visual_and_revision_without_visual_io() {
    let mut fixture = Fixture::new();
    let terrain = fixture.load().unwrap();
    let identity = terrain.identity;
    assert_ne!(identity, sha256(&fixture.profile));
    terrain
        .verify_visual(&fixture.root.join("terrain.glb"))
        .unwrap();
    assert_eq!(
        terrain.package.as_ref().unwrap().content_id().as_str(),
        "theia.test.flat"
    );
    assert_eq!(terrain.package.as_ref().unwrap().revision(), 7);
    std::fs::remove_file(fixture.root.join("terrain.glb")).unwrap();
    assert_eq!(fixture.load().unwrap().identity, identity); // Headless server does not need art.
    assert!(
        terrain
            .verify_visual(&fixture.root.join("terrain.glb"))
            .is_err()
    );
    fixture.manifest["revision"] = json!(8);
    fixture.write();
    assert_ne!(fixture.load().unwrap().identity, identity);
    fixture.manifest["collision_schema"] = json!(1);
    fixture.write();
    assert!(fixture.load().is_err());
    fixture.manifest["collision_schema"] = json!(2);
    fixture.manifest["visual_sha256"] = json!(vec![0; 32]);
    fixture.write();
    assert!(fixture.load().is_err());
}
#[test]
fn corrupt_present_packages_never_fall_back_to_legacy() {
    let fixture = Fixture::new();
    std::fs::write(fixture.root.join("terrain.glb"), b"wrong payload").unwrap();
    assert!(
        fixture
            .load()
            .unwrap()
            .verify_visual(&fixture.root.join("terrain.glb"))
            .is_err()
    );
    let mut changed = fixture.profile.clone();
    changed.push(b' ');
    std::fs::write(fixture.root.join("terrain.json"), changed).unwrap();
    assert!(fixture.load().is_err());
    std::fs::write(fixture.root.join("terrain.json"), &fixture.profile).unwrap();
    std::fs::write(fixture.root.join("world.manifest.json"), b"{}").unwrap();
    assert!(fixture.load().is_err());
    std::fs::remove_file(fixture.root.join("world.manifest.json")).unwrap();
    assert_eq!(fixture.load().unwrap().identity, sha256(&fixture.profile));
}

#[tokio::test]
async fn secure_admission_uses_package_identity_and_rejects_a_stale_revision() {
    use theia_network::{client::State, renet_runtime as runtime, simulation::World, wire::Intent};
    use tokio::sync::watch;
    let mut fixture = Fixture::new();
    let old = fixture.load().unwrap().identity;
    fixture.manifest["revision"] = json!(8);
    fixture.write();
    let terrain = fixture.load().unwrap();
    let expected = terrain.identity;
    std::fs::remove_file(fixture.root.join("terrain.glb")).unwrap();
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let transport = runtime::issue_local(socket, &fixture.root.join("tickets")).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(runtime::run_server(
        transport,
        shutdown,
        World::with_terrain(terrain),
    ));
    let (_input, input) = watch::channel(Intent::default());
    let (state_tx, mut state) = watch::channel(State::default());
    let (client_stop, client_shutdown) = watch::channel(false);
    let bytes = std::fs::read(fixture.root.join("tickets/client-1.token")).unwrap();
    let client = tokio::spawn(async move {
        runtime::run_client(address, &bytes, input, state_tx, client_shutdown, expected).await
    });
    tokio::time::timeout(Duration::from_secs(5), async {
        while state.borrow().snapshot.is_none() {
            state.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    let (_wrong_input, input) = watch::channel(Intent::default());
    let (state_tx, _state) = watch::channel(State::default());
    let (_wrong_stop, shutdown) = watch::channel(false);
    let ticket = std::fs::read(fixture.root.join("tickets/client-2.token")).unwrap();
    let error = tokio::time::timeout(
        Duration::from_secs(5),
        runtime::run_client(address, &ticket, input, state_tx, shutdown, old),
    )
    .await
    .unwrap()
    .unwrap_err();
    assert!(error.to_string().contains("world mismatch"), "{error}");
    client_stop.send_replace(true);
    client.await.unwrap().unwrap();
    stop.send_replace(true);
    server.await.unwrap().unwrap();
}

#[test]
fn general_collision_schema_and_explicit_spawns_are_package_bound_without_visual_io() {
    let mut fixture = Fixture::new();
    let mut profile: Value = serde_json::from_slice(&fixture.profile).unwrap();
    profile["schema"] = json!(3);
    profile["spawns"] = json!([[2, 0, 2]]);
    fixture.profile = serde_json::to_vec(&profile).unwrap();
    fixture.manifest["collision_schema"] = json!(3);
    fixture.manifest["collision_bytes"] = json!(fixture.profile.len());
    fixture.manifest["collision_sha256"] = json!(sha256(&fixture.profile));
    std::fs::write(fixture.root.join("terrain.json"), &fixture.profile).unwrap();
    fixture.write();
    std::fs::remove_file(fixture.root.join("terrain.glb")).unwrap();
    let terrain = fixture.load().unwrap();
    assert!(terrain.has_spawns());
    let p = terrain.spawn(0).unwrap().metres();
    assert_eq!([p[0], p[2]], [2., 2.]);
    assert!((p[1] - 0.008).abs() < 0.001);
    fixture.manifest["collision_schema"] = json!(2);
    fixture.write();
    assert!(fixture.load().is_err());
}
