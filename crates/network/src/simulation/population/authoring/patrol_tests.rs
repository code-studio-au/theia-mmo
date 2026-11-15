//! Real exported collision, navigation and authoritative movement qualification.
use super::*;
use crate::{procedural, simulation::World};
use std::{path::PathBuf, sync::Arc};
use theia_core::time::TickRate;

const CREATURE: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-creature-deflect-pursuit.json");
const STATS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-character-stats.json");

#[test]
#[ignore = "requires THEIA_PATROL_SOURCE and THEIA_PATROL_PACKAGE with exported world/navigation"]
fn authored_patrol_is_checked_and_completes_out_and_back_at_all_rates() {
    let package = PathBuf::from(std::env::var_os("THEIA_PATROL_PACKAGE").unwrap());
    let mut zone = Zone::load(&PathBuf::from(
        std::env::var_os("THEIA_PATROL_SOURCE").unwrap(),
    ))
    .unwrap();
    zone.schema = 15;
    let group = zone.spawn_groups.iter_mut().find(|g| g.id == 1200).unwrap();
    group.patrol_path = Some(1204);
    group.roam_radius_metres = 0.;
    zone.validate().unwrap();
    let original = zone.clone();
    let store = Arc::new(authored_world::Store::open(&package.join("world")).unwrap());
    let nav = navigation::Store::open(&package.join("navigation"), store.identity()).unwrap();
    let definition = Definition::from_json(CREATURE).unwrap();
    let access = access_waypoints(&zone, 1200, 1201, store.spawn(0).unwrap().metres()).unwrap();
    let access = navigation::authored_checks::grounded_route(&store, &nav, &access).unwrap();
    let access = access.last().unwrap().metres();
    let prepare = |zone: &Zone| prepare(zone, 1200, definition.clone(), &store, &nav, 3., access);
    let (_, binding) = prepare(&zone).unwrap();
    assert_eq!(binding.roaming.len(), 5);
    for routes in &binding.roaming {
        assert_eq!(routes.len(), 1);
        let route = &routes[0];
        assert!(route.len() >= 5 && route.len() <= 255);
        assert_eq!(route.first(), route.last());
        assert!(route.iter().eq(route.iter().rev()));
    }
    let mut invalid = zone.clone();
    invalid
        .paths
        .iter_mut()
        .find(|p| p.id == 1204)
        .unwrap()
        .points[1][0] += 80.;
    assert!(
        prepare(&invalid)
            .err()
            .unwrap()
            .to_string()
            .contains("64 m")
    );
    let mut invalid = zone.clone();
    invalid
        .paths
        .iter_mut()
        .find(|p| p.id == 1204)
        .unwrap()
        .points[1][1] += 8.;
    assert!(
        prepare(&invalid).is_err(),
        "Wrong-floor endpoint must be refused"
    );
    let passive = Definition::from_json(include_bytes!(
        "../../../../../../content/gameplay/development-creature-passive.json"
    ))
    .unwrap();
    assert!(super::binding(&zone, 1200, passive).is_err());
    let mut stationary = zone.clone();
    stationary.spawn_groups[0].patrol_path = None;
    assert_ne!(
        binding.profile_identity([0; 32]),
        prepare(&stationary).unwrap().1.profile_identity([0; 32])
    );

    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (prepared, binding) = prepare(&zone).unwrap();
        let ends: Vec<_> = binding
            .roaming
            .iter()
            .map(|routes| routes[0][routes[0].len() / 2].metres())
            .collect();
        let spawn = store.spawn(0).unwrap().metres();
        let mut world = World::with_terrain_source(
            [spawn[0], spawn[2]],
            1.,
            procedural::Source::Authored(store.clone()),
        )
        .unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        let keys = world
            .enable_navigating_population(
                prepared,
                binding,
                navigation::worker::Worker::load(&nav).unwrap(),
            )
            .unwrap();
        let homes: Vec<_> = keys
            .iter()
            .map(|k| world.targets[&k.entity.value()].position.metres())
            .collect();
        world
            .enable_status_combat(
                include_bytes!(
                    "../../../../../../content/gameplay/development-classic-abilities.json"
                ),
                100,
                30_000,
                STATS,
                include_bytes!("../../../../../../content/gameplay/development-statuses.json"),
            )
            .unwrap();
        world
            .enable_autoattack(
                include_bytes!("../../../../../../content/gameplay/development-melee-weapons.json"),
                include_bytes!(
                    "../../../../../../content/gameplay/development-equipment-items.json"
                ),
            )
            .unwrap();
        let life = crate::player_life_wire::Profile::from_json(
            STATS,
            CREATURE,
            include_bytes!(
                "../../../../../../content/gameplay/development-creature-retaliation.json"
            ),
            include_bytes!("../../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap();
        world.enable_population_player_life(Arc::new(life)).unwrap();
        let identity = world.identity();
        let mut visited = vec![false; keys.len()];
        let mut returned = visited.clone();
        let mut departed_again = visited.clone();
        let distance = |a: [f64; 3], b: [f64; 3]| (a[0] - b[0]).hypot(a[2] - b[2]);
        for _ in 0..u64::from(rate.hz()) * 120 {
            world.step();
            for (index, key) in keys.iter().enumerate() {
                let position = world.targets[&key.entity.value()].position.metres();
                assert!(
                    distance(position, homes[index]) < 64.,
                    "{rate:?}: escaped home bound"
                );
                assert!(
                    !world.population_motion[index].blocked,
                    "{rate:?}: blocked actor {index} at {position:?}"
                );
                visited[index] |= distance(position, ends[index]) < 0.3;
                returned[index] |= visited[index] && distance(position, homes[index]) < 0.3;
                departed_again[index] |= returned[index] && distance(position, homes[index]) > 1.;
            }
            if departed_again.iter().all(|done| *done) {
                break;
            }
        }
        assert!(
            departed_again.iter().all(|done| *done),
            "{rate:?}: visited={visited:?} returned={returned:?} departed={departed_again:?}"
        );
        assert_eq!(world.identity(), identity);
        println!(
            "{rate:?}: all five actors visited the authored endpoint, returned home and began another patrol"
        );
    }
    assert_eq!(
        zone, original,
        "Prepared steering must not rewrite sparse authoring"
    );
}
