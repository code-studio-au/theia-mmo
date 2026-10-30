//! Explicit fixture startup, outside the simulation and before opening endpoints.
use std::{sync::Arc, time::Duration};
use theia_core::coordinates::GlobalPosition;
use theia_network::{
    Result,
    dialogue::{Guide, Profile},
    simulation::World,
};
pub async fn prepare_guide(
    world: &mut World,
    profile: Arc<Profile>,
    xz: [f64; 2],
) -> Result<Guide> {
    let focus = GlobalPosition::new([xz[0], 0., xz[1]]).ok_or("Invalid guide position")?;
    if world.completed_tick() != 0
        || world.procedural().is_none()
        || theia_network::procedural::owner(focus).is_none()
    {
        return Err("Guide preparation requires inactive graybox and bounded position".into());
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        world.poll_terrain_keeping(&[focus])?;
        if world
            .procedural()
            .and_then(|runtime| runtime.patch(focus))
            .is_some()
        {
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("Guide terrain preparation timed out".into());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    world.spawn_development_guide(profile, xz)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn distant_guide_preparation_uses_the_existing_loader_before_simulation_starts() {
        let profile = Arc::new(
            Profile::from_json(include_bytes!(
                "../../../content/gameplay/development-dialogue.json"
            ))
            .unwrap(),
        );
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        let position = GlobalPosition::new([15040., 0., 15040.]).unwrap();
        assert!(!world.terrain_prepared_for(position));
        let guide = prepare_guide(&mut world, profile.clone(), [15040., 15040.])
            .await
            .unwrap();
        assert!(world.terrain_prepared_for(guide.position));
        assert_eq!(world.completed_tick(), 0);
        assert!(world.dialogue_profile().is_some());
        world.step();
        assert!(
            prepare_guide(&mut world, profile, [8000., 8000.])
                .await
                .is_err()
        );
    }
}
