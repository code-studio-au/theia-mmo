//! Full bounded capacity is actual disk/Avian coverage, not rendering evidence.
#[path = "support/wide_chunk_fixture.rs"]
mod fixture;
use bevy::prelude::Vec3;
use fixture::WideFixture;
use theia_core::coordinates::{ChunkId, GlobalPosition};
use theia_network::{
    chunks::{Index, MAX_RESIDENT, Residency},
    physics,
};

#[test]
fn full_capacity_loads_real_collision_and_missing_thirty_third_cell_stays_closed() {
    for base in [-7_812_500, 0, 7_812_500] {
        let f = WideFixture::new(base);
        let index = Index::load(&f.root).unwrap();
        let frame = index.frame();
        let mut active = Residency::new(index, MAX_RESIDENT).unwrap();
        let loaded = &f.ids[..MAX_RESIDENT];
        active.replace(loaded, &[]).unwrap();
        assert_eq!(active.loaded(), loaded);
        assert_eq!(active.metrics().max_active, MAX_RESIDENT);
        for id in loaded {
            let global =
                GlobalPosition::new([id.x as f64 * 128. + 64., 0., id.z as f64 * 128. + 64.])
                    .unwrap();
            let local = frame.to_local(global).unwrap().metres();
            assert!(
                active
                    .index()
                    .coverage(active.loaded())
                    .unwrap()
                    .contains(global)
            );
            assert!(
                (active
                    .scene()
                    .unwrap()
                    .ground([local[0], local[2]], local[1] + 4.)
                    .unwrap()
                    .0
                    - local[1])
                    .abs()
                    < 0.001
            );
        }
        let coverage = active.index().coverage(active.loaded()).unwrap();
        let old_generation = active.generation();
        assert!(active.replace(&f.ids[..MAX_RESIDENT + 1], &[]).is_err());
        assert_eq!(active.generation(), old_generation);
        // Last loaded cell is (5,1); (5,2) is indexed but not resident.
        let p = GlobalPosition::new([
            (base + 5) as f64 * 128. + 64.,
            0.,
            (base + 1) as f64 * 128. + 64.,
        ])
        .unwrap();
        let local = Vec3::from_array(frame.to_local(p).unwrap().metres());
        let hit = coverage.sweep(local, Vec3::Z * 128.).unwrap();
        assert!((hit.distance - (64. - theia_network::kinematic::FOOTPRINT_RADIUS)).abs() < 0.001);
        assert!(coverage.sweep(local, Vec3::NEG_Y * 8.).is_none());
        assert_eq!(hit.normal, Vec3::NEG_Z);
        assert!(
            active.scene().unwrap().cast(
                &physics::character_shape(0.),
                local + Vec3::Y * 2.,
                Vec3::NEG_Y * 4.,
                physics::GROUND
            ) < 1.
        );
        assert_eq!(
            f.ids[MAX_RESIDENT],
            ChunkId {
                x: base + 5,
                z: base + 2
            }
        );
    }
}

#[test]
#[ignore = "requires explicitly staged private partitioned-town packages"]
fn staged_town_initial_and_all_indexed_publication_preserve_live_actor_support() {
    use theia_network::{chunk_world::Config, simulation::World};
    let directory = std::path::PathBuf::from(
        std::env::var("THEIA_PARTITION_WORLD_FIXTURE").expect("Set THEIA_PARTITION_WORLD_FIXTURE"),
    );
    let started = std::time::Instant::now();
    let config = Config::load(&directory).unwrap();
    assert_eq!(config.budget(), 26);
    assert_eq!(config.initial().len(), 10);
    let mut world = World::with_chunk_config(&config).unwrap();
    let initial_ms = started.elapsed().as_secs_f64() * 1000.;
    let actor = world.join().unwrap();
    let before = world.global_position(actor).unwrap();
    let all = config.index().ids().collect::<Vec<_>>();
    assert_eq!(all.len(), 26);
    let started = std::time::Instant::now();
    let candidate = world.plan_chunks(&all).unwrap().load().unwrap();
    let prepare_ms = started.elapsed().as_secs_f64() * 1000.;
    let started = std::time::Instant::now();
    world.activate_chunks(candidate).unwrap();
    let publish_ms = started.elapsed().as_secs_f64() * 1000.;
    assert_eq!(world.global_position(actor), Some(before));
    assert_eq!(world.chunks().unwrap().residency().loaded(), all);
    assert_eq!(world.chunks().unwrap().residency().generation(), 2);
    for _ in 0..90 {
        world.step();
    }
    assert_eq!(world.movement_recoveries(), 0);
    let after = world.global_position(actor).unwrap().metres();
    assert!((after[1] - before.metres()[1]).abs() < 0.002);
    println!(
        "staged_partition_capacity_pass initial=10 resident=26 initial_ms={initial_ms:.3} prepare_ms={prepare_ms:.3} publish_ms={publish_ms:.3} recoveries=0"
    );
}
