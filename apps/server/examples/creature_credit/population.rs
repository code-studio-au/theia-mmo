//! Real checked population setup for the owned SQL first-tag qualification.
use super::*;

pub(super) async fn prepare(
    package: PathBuf,
) -> theia_network::Result<(World, Vec<u8>, [f64; 2], u32)> {
    use theia_network::{authored_world, simulation::population};
    let store = Arc::new(authored_world::Store::open(&package.join("world"))?);
    let saved = store.spawn(0).ok_or("Missing authored spawn")?.metres();
    let source = Source::Authored(store);
    let loaded =
        population::configuration::Loaded::open(&package.join("population.json"), &source)?;
    assert_eq!(loaded.count(), 5);
    let creature = loaded.creature_bytes().to_vec();
    let respawn_ms = loaded.respawn_ms();
    let (prepared, binding) = loaded.into_parts();
    let feet = prepared.points()[0].metres();
    let placement = [feet[0], feet[2]];
    let mut world = World::with_terrain_source([saved[0], saved[2]], 1., source)?;
    world.enable_prediction()?;
    world.enable_classic()?;
    world.enable_defined_population(prepared, binding)?;
    theia_server::development::prepare_guide(
        &mut world,
        Arc::new(theia_network::dialogue::Profile::from_json(
            include_bytes!("../../../../content/gameplay/development-dialogue.json"),
        )?),
        placement,
    )
    .await?;
    Ok((world, creature, placement, respawn_ms))
}
