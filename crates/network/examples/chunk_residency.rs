//! Actual bounded disk loading, startup and Avian queries. No renderer or networking.
use bevy::prelude::Vec3;
use theia_core::coordinates::{GlobalPosition, LocalPosition};
use theia_network::{
    chunks::{Index, Residency},
    physics,
};
fn main() -> theia_network::Result<()> {
    let directory = std::env::args_os()
        .nth(1)
        .ok_or("Expected prepared chunk fixture directory")?;
    let directory = std::path::Path::new(&directory);
    // Older residency-only evidence predates the shared startup descriptor.
    // When supplied, validate the actual generator-to-authority contract too.
    if directory.join("world.session.json").exists() {
        let config = theia_network::chunk_world::Config::load(directory)?;
        let mut world = theia_network::simulation::World::with_chunk_config(&config)?;
        assert_eq!(world.identity(), config.identity());
        let id = world.join().ok_or("No valid chunk-world spawn")?;
        let snapshot = world.step();
        assert_eq!(snapshot.players.len(), 1);
        assert_eq!(snapshot.players[0].id, id);
        assert_eq!(world.movement_recoveries(), 0);
        println!(
            "chunk_startup_pass world={:02x?} resident={} spawns={}",
            world.identity(),
            config.initial().len(),
            config.spawns().len()
        );
    }
    let index = Index::load(directory)?;
    let frame = index.frame();
    let point = |x, z| {
        frame
            .to_global(LocalPosition::new([x, 0., z]).unwrap())
            .unwrap()
    };
    let left = index.owner(point(-0.25, 32.)).ok_or("Missing left owner")?;
    let middle = index.owner(point(0., 32.)).ok_or("Missing middle owner")?;
    let right = index.owner(point(128., 32.)).ok_or("Missing right owner")?;
    assert_ne!(left, middle);
    assert_ne!(middle, right);
    let mut chunks = Residency::new(index, 2)?;
    chunks.replace(&[left, middle], &[point(-0.25, 32.)])?;
    let ground = |chunks: &Residency, p: GlobalPosition| {
        let p = chunks.index().frame().to_local(p).unwrap().metres();
        chunks.scene().unwrap().ground([p[0], p[2]], 5.).unwrap().0
    };
    assert!(ground(&chunks, point(-0.01, 32.)).abs() < 0.001);
    assert!(ground(&chunks, point(0.01, 32.)).abs() < 0.001);
    assert_eq!(chunks.visuals().len(), 2);
    // A standing capsule hits the actual thin wall in the loaded middle chunk.
    let fraction = chunks.scene().unwrap().cast(
        &physics::character_shape(0.),
        Vec3::new(63., 0.925, 64.),
        Vec3::X * 2.,
        physics::SOLID,
    );
    assert!(fraction > 0.1 && fraction < 0.5);
    assert!(chunks.replace(&[left, middle, right], &[]).is_err());
    assert!(
        chunks
            .replace(&[middle, right], &[point(-0.25, 32.)])
            .is_err()
    );
    chunks.replace(&[middle, right], &[point(64., 32.)])?;
    assert!(chunks.scene().unwrap().ground([-64., 32.], 5.).is_none());
    assert!(ground(&chunks, point(127.99, 32.)).abs() < 0.001);
    assert!(ground(&chunks, point(128.01, 32.)).abs() < 0.001);
    for visual in chunks.visuals() {
        assert!(visual.asset.is_file());
        assert_eq!(visual.origin.chunk(128), Some(visual.chunk));
    }
    chunks.replace(&[], &[])?;
    assert!(chunks.scene().is_none() && chunks.visuals().is_empty());
    let stats = chunks.metrics();
    assert_eq!((stats.loads, stats.unloads, stats.max_active), (3, 3, 2));
    println!(
        "chunk_residency_pass loads={} unloads={} max_active={} wall_fraction={fraction} query_origin={:?}",
        stats.loads,
        stats.unloads,
        stats.max_active,
        frame.origin().metres()
    );
    Ok(())
}
