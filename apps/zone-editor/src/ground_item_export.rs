//! Reject unsupported collection items before publishing a population package.
use theia_network::{authored_world, gameplay_bundle, ground_placement, procedural};

#[derive(Debug, serde::Serialize)]
pub(super) struct Checked {
    reward: String,
    quest: String,
    area: String,
    position: [f64; 3],
    supported_feet: [f64; 3],
}

/// Worker-only: opening patches may read geometry; simulation uses prepared ones.
pub(super) fn prepare(
    snapshot: &gameplay_bundle::Snapshot,
    world: &authored_world::Store,
) -> Result<Vec<Checked>, String> {
    let Some(raw) = snapshot.profile("rewards") else {
        return Ok(Vec::new());
    };
    let items = theia_core::inventory::Catalog::from_json(
        snapshot
            .profile("items")
            .ok_or("Ground rewards require items")?,
    )?;
    let rewards = theia_core::rewards::Catalog::from_json(raw, &items)?;
    rewards
        .definitions()
        .filter_map(|reward| reward.ground().map(|g| (reward, g)))
        .map(|(reward, ground)| {
            let check = || {
                let owner = procedural::owner(ground.position)
                    .ok_or("Ground item is outside the world domain")?;
                let patch = world.patch(owner)?;
                ground_placement::supported(&patch, ground.position)
            };
            let supported = check().map_err(|e| {
                format!(
                    "Ground collection '{}' in '{}': {e}",
                    reward.id().as_str(),
                    ground.area
                )
            })?;
            Ok(Checked {
                reward: reward.id().as_str().into(),
                quest: ground.quest.as_str().into(),
                area: ground.area.clone(),
                position: ground.position.metres(),
                supported_feet: supported.metres(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires THEIA_QUEST_TRAVEL_PACKAGE with actual authored collision"]
    fn authored_collection_preflight_checks_support_without_mutating_source() {
        let root =
            std::path::PathBuf::from(std::env::var_os("THEIA_QUEST_TRAVEL_PACKAGE").unwrap());
        let manifest = root.join("world/gameplay/gameplay.manifest.json");
        let snapshot = gameplay_bundle::snapshot(&manifest).unwrap();
        let world = authored_world::Store::open(&root.join("world")).unwrap();
        let checked = prepare(&snapshot, &world).unwrap();
        assert_eq!(checked.len(), 1);
        assert_eq!(checked[0].reward, "theia.dev.compass-ground");
        assert_eq!(checked[0].area, "Northern Meadow");
        let original = std::fs::read(&manifest).unwrap();
        let position = theia_core::coordinates::GlobalPosition::new(checked[0].position).unwrap();
        let patch = world.patch(procedural::owner(position).unwrap()).unwrap();
        let mut wrong_floor = position.metres();
        wrong_floor[1] += 8.;
        assert!(
            ground_placement::supported(
                &patch,
                theia_core::coordinates::GlobalPosition::new(wrong_floor).unwrap()
            )
            .is_err()
        );
        let mut outside = position.metres();
        outside[0] += 4096.;
        assert!(
            ground_placement::supported(
                &patch,
                theia_core::coordinates::GlobalPosition::new(outside).unwrap()
            )
            .is_err()
        );
        assert_eq!(original, std::fs::read(&manifest).unwrap());
        println!("{}", serde_json::to_string(&checked).unwrap());
    }
}
