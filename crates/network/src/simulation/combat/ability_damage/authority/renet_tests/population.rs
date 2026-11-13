//! Checked authored assets for the encrypted population qualification.
use super::*;
use crate::{authored_world, navigation, procedural, simulation::population::Binding};

pub(super) fn prepare(rate: TickRate, creature: &[u8]) -> (World, EntityKey, [f64; 2]) {
    let store = Arc::new(
        authored_world::Store::open(std::path::Path::new(
            &std::env::var_os("THEIA_AUTHORED_WORLD").unwrap(),
        ))
        .unwrap(),
    );
    let nav = navigation::Store::open(
        std::path::Path::new(&std::env::var_os("THEIA_AUTHORED_NAV").unwrap()),
        store.identity(),
    )
    .unwrap();
    let request = serde_json::from_slice(
        &std::fs::read(std::env::var_os("THEIA_AUTHORED_SPAWNS").unwrap()).unwrap(),
    )
    .unwrap();
    let prepared = navigation::spawn_preparation::check_authored(&store, &nav, &request)
        .unwrap()
        .into_prepared()
        .unwrap();
    assert_eq!(prepared.points().len(), 5);
    let point = prepared.points()[0].metres();
    let placement = [point[0], point[2]];
    let saved = store.spawn(0).unwrap().metres();
    let mut world = World::with_terrain_source(
        [saved[0], saved[2]],
        1.,
        procedural::Source::Authored(store),
    )
    .unwrap();
    assert!(world.set_tick_rate(rate));
    let definition = creatures::Definition::from_json(creature).unwrap();
    let binding = Binding::new(definition.id(), definition.clone(), 30).unwrap();
    let keys = if definition.aggro().is_some() {
        world
            .enable_navigating_population(
                prepared,
                binding,
                navigation::worker::Worker::load(&nav).unwrap(),
            )
            .unwrap()
    } else {
        world.enable_defined_population(prepared, binding).unwrap()
    };
    (world, keys[0], placement)
}

/// Compose the client source before rewards/progression and Classic wrapping.
/// Keep the assertion against World::identity in the callers.
pub(super) fn combat_source(world: &World) -> [u8; 32] {
    let mut source = world.procedural.as_ref().unwrap().source.identity();
    if world.targeting_enabled {
        source = crate::targeting::profile_identity(source);
    }
    source = world.patrol_identity.unwrap_or(source);
    if let Some(prepared) = &world.population {
        source = prepared.profile_identity(source);
    }
    if let Some(binding) = &world.population_binding {
        source = binding.profile_identity(source);
    }
    world.combat.as_ref().unwrap().identity(source)
}
