//! Actual authored routes and Avian movement, independent of rendering.
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};
use theia_core::time::TickRate;
use theia_network::{
    authored_world, procedural,
    simulation::{World, population::configuration::Loaded},
};

#[test]
#[ignore = "requires THEIA_ROAMING_PACKAGE with a checked roaming population"]
fn idle_population_moves_inside_its_radius_at_all_tick_rates() {
    let package = PathBuf::from(std::env::var_os("THEIA_ROAMING_PACKAGE").unwrap());
    let store = Arc::new(authored_world::Store::open(&package.join("world")).unwrap());
    let spawn = store.spawn(0).unwrap().metres();
    let source = procedural::Source::Authored(store.clone());
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let loaded = Loaded::open(&package.join("population.json"), &source).unwrap();
        let worker = loaded.pursuit_worker().unwrap().unwrap();
        let life = Arc::new(
            theia_network::player_life_wire::Profile::from_json(
                include_bytes!("../../../content/gameplay/development-character-stats.json"),
                loaded.creature_bytes(),
                include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
                include_bytes!("../../../content/gameplay/development-player-respawn.json"),
            )
            .unwrap(),
        );
        let respawn = loaded.respawn_ms();
        let (prepared, binding) = loaded.into_parts();
        let homes = prepared.points().to_vec();
        let volume = prepared.request().volume.clone();
        let mut world =
            World::with_terrain_source([spawn[0], spawn[2]], 1., source.clone()).unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        let keys = world
            .enable_navigating_population(prepared, binding, worker)
            .unwrap();
        world
            .enable_resource_combat(
                include_bytes!("../../../content/gameplay/development-abilities.json"),
                100,
                respawn,
                include_bytes!("../../../content/gameplay/development-character-stats.json"),
            )
            .unwrap();
        world.enable_population_player_life(life).unwrap();
        let actor = (0..1000)
            .find_map(|_| {
                world.poll_terrain();
                let joined = world.join();
                if joined.is_none() {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                joined
            })
            .unwrap();
        let mut moved = BTreeSet::new();
        for _ in 0..rate.hz() * 30 {
            world.poll_terrain();
            world.step();
            for target in world.targeting_snapshot(actor).unwrap().targets {
                let index = keys.iter().position(|k| *k == target.key).unwrap();
                let p = target.position.metres();
                let home = homes[index].metres();
                let distance = (p[0] - home[0]).hypot(p[2] - home[2]);
                assert!(
                    distance <= 4.1,
                    "Roaming exceeded the 4 m fixture radius: {distance}"
                );
                if distance > 0.5 {
                    moved.insert(index);
                }
                let feet = theia_core::coordinates::GlobalPosition::new([
                    p[0],
                    p[1] - f64::from(theia_network::world::PAWN_Y),
                    p[2],
                ])
                .unwrap();
                assert!(volume.contains(feet));
            }
        }
        assert_eq!(
            moved.len(),
            keys.len(),
            "Not all idle creatures walked at {rate:?}"
        );
        assert_eq!(world.creature_pursuit_metrics().unwrap().acquisitions, 0);
        assert_eq!(world.player_life_metrics().unwrap().strikes, 0);
    }
}
