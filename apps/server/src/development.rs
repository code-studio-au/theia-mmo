//! Explicit fixture startup, outside the simulation and before opening endpoints.
use std::{sync::Arc, time::Duration};
use theia_network::{
    Result,
    dialogue::{Guide, Placement, Profile},
    simulation::World,
};
pub async fn prepare_guide(
    world: &mut World,
    profile: Arc<Profile>,
    xz: [f64; 2],
) -> Result<Guide> {
    prepare_placed_guide(world, profile, Placement::Horizontal(xz)).await
}
pub async fn prepare_placed_guide(
    world: &mut World,
    profile: Arc<Profile>,
    placement: Placement,
) -> Result<Guide> {
    let focus = placement.focus()?;
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
    world.spawn_placed_guide(profile, placement)
}

/// Prepare all contact support outside the fixed step, then install atomically.
pub async fn prepare_contacts(
    world: &mut World,
    profile: Arc<Profile>,
    placements: Vec<(theia_core::content::ContentId, Placement)>,
) -> Result<Vec<theia_network::dialogue::ContactPresence>> {
    if world.completed_tick() != 0 || world.procedural().is_none() {
        return Err("Contacts require inactive prepared world".into());
    }
    let focuses = placements
        .iter()
        .map(|(_, p)| p.focus())
        .collect::<Result<Vec<_>>>()?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        world.poll_terrain_keeping(&focuses)?;
        if focuses
            .iter()
            .all(|p| world.procedural().and_then(|r| r.patch(*p)).is_some())
        {
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("Contact support preparation timed out".into());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    if profile.catalog().is_some() {
        world.spawn_placed_contacts(profile, placements)
    } else {
        if placements.len() != 1 || &placements[0].0 != profile.profile().npc() {
            return Err("Invalid legacy placement".into());
        }
        let guide = world.spawn_placed_guide(profile, placements[0].1)?;
        Ok(vec![theia_network::dialogue::ContactPresence {
            npc: placements[0].0.clone(),
            guide,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::coordinates::GlobalPosition;
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
