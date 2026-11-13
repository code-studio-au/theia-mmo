//! Startup-only adapter for saved editor populations. No filesystem work or
//! navigation search belongs in the fixed simulation step.
use super::Binding;
use crate::{
    Result, authored_world,
    navigation::{self, spawn_candidates::Request, spawn_preparation},
    simulation::creatures::Definition,
};
use theia_zone::{Zone, spawn_groups::SpawnGroup};

fn group(zone: &Zone, id: u64) -> Result<&SpawnGroup> {
    zone.validate()?;
    let group = zone
        .spawn_groups
        .iter()
        .find(|g| g.id == id)
        .ok_or("Unknown saved spawn group ID")?;
    if !group.enabled {
        return Err("Selected spawn group is disabled".into());
    }
    Ok(group)
}

/// Placement-only request; creature/model/lifecycle admission is separate.
pub fn request(zone: &Zone, id: u64, spacing: f64, access: [f64; 3]) -> Result<Request> {
    let group = group(zone, id)?;
    let request = Request {
        schema: 1,
        id: format!("theia.zone.spawn.{}", group.id),
        volume: group.volume.clone(),
        count: usize::from(group.target_alive),
        seed: group.seed,
        minimum_spacing_metres: spacing,
        access_from: access,
    };
    request.validate()?;
    Ok(request)
}

/// Explicit road guide from the actual player access pose into this population.
/// These are requests only; the loader must ground and clearance-check the route.
pub fn access_waypoints(
    zone: &Zone,
    group_id: u64,
    path_id: u64,
    access: [f64; 3],
) -> Result<Vec<[f64; 3]>> {
    let group = group(zone, group_id)?;
    let path = zone
        .paths
        .iter()
        .find(|p| p.id == path_id)
        .ok_or("Unknown population access path ID")?;
    if path.kind != theia_zone::paths::PathKind::Road {
        return Err("Population access path must be a road guide".into());
    }
    let access = theia_core::coordinates::GlobalPosition::new(access)
        .ok_or("Invalid population access pose")?;
    let start = path.points[0];
    let distance = start
        .into_iter()
        .zip(access.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    if distance > 0.5 {
        return Err("Population access path must start within 0.5 metres of player access".into());
    }
    let end = theia_core::coordinates::GlobalPosition::new(*path.points.last().unwrap())
        .ok_or("Invalid population access endpoint")?;
    if !group.volume.contains(end) {
        return Err("Population access path must end inside the spawn volume".into());
    }
    let mut points = path.points.clone();
    points[0] = access.metres();
    Ok(points)
}

/// Resolve the saved group's creature and lifecycle before expensive placement.
/// Explicit model bindings must wait for the presentation catalog adapter.
pub fn binding(zone: &Zone, id: u64, definition: Definition) -> Result<Binding> {
    let group = group(zone, id)?;
    if usize::from(group.target_alive) > crate::targeting::MAX_TARGETS {
        return Err("Saved population exceeds the runtime 16-target limit".into());
    }
    if group.model.is_some() {
        return Err("Saved population model binding is not connected".into());
    }
    let creature = theia_core::content::ContentId::new(&group.creature)
        .ok_or("Invalid population creature reference")?;
    Binding::new(&creature, definition, group.respawn_seconds)
}

/// Check one explicitly selected group against the supplied immutable world/nav
/// pair. This does not prove that the draft's other geometry was exported.
pub fn prepare(
    zone: &Zone,
    id: u64,
    definition: Definition,
    store: &authored_world::Store,
    navigation: &navigation::Store,
    spacing: f64,
    access: [f64; 3],
) -> Result<(spawn_preparation::Prepared, Binding)> {
    let binding = binding(zone, id, definition)?;
    let request = request(zone, id, spacing, access)?;
    let prepared =
        spawn_preparation::check_authored(store, navigation, &request)?.into_prepared()?;
    Ok((prepared, binding))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn definition() -> Definition {
        crate::simulation::creatures::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-creature-patrol.json"
        ))
        .unwrap()
        .definition()
        .clone()
    }
    fn zone() -> Zone {
        let mut zone = Zone {
            schema: 14,
            ..Default::default()
        };
        zone.spawn_groups.push(SpawnGroup {
            id: 1,
            name: "Grove".into(),
            enabled: true,
            creature: definition().id().as_str().into(),
            model: None,
            volume: theia_core::spawn_volume::PolygonVolume::new(
                vec![[0., 0.], [8., 0.], [8., 8.], [0., 8.]],
                -2.,
                8.,
            )
            .unwrap(),
            target_alive: 5,
            respawn_seconds: 61,
            seed: 17,
        });
        zone
    }
    #[test]
    fn access_road_requires_connected_start_and_endpoint_in_its_group() {
        let mut zone = zone();
        zone.paths.push(theia_zone::paths::GuidePath {
            id: 2,
            name: "Access road".into(),
            kind: theia_zone::paths::PathKind::Road,
            width_metres: 1.,
            points: vec![[0., 0., 0.], [4., 0., 4.]],
        });
        let original = zone.clone();
        let points = access_waypoints(&zone, 1, 2, [0., 0.1, 0.]).unwrap();
        assert_eq!(points, vec![[0., 0.1, 0.], [4., 0., 4.]]);
        assert_eq!(zone, original);
        assert!(access_waypoints(&zone, 1, 99, zone.spawn).is_err());
        assert!(access_waypoints(&zone, 1, 2, [1., 0., 0.]).is_err());
        assert!(access_waypoints(&zone, 1, 2, [f64::NAN, 0., 0.]).is_err());
        zone.paths[0].points[1] = [20., 0., 20.];
        assert!(access_waypoints(&zone, 1, 2, zone.spawn).is_err());
        zone = original;
        for kind in [
            theia_zone::paths::PathKind::Stream,
            theia_zone::paths::PathKind::Patrol,
        ] {
            zone.paths[0].kind = kind;
            assert!(access_waypoints(&zone, 1, 2, zone.spawn).is_err());
        }
    }

    #[test]
    fn saved_lifecycle_and_creature_are_resolved_exactly() {
        let mut zone = zone();
        let resolved = binding(&zone, 1, definition()).unwrap();
        assert_eq!(resolved.respawn_ms(), 61_000);
        assert_eq!(resolved.definition(), &definition());
        let request = request(&zone, 1, 3., zone.spawn).unwrap();
        assert_eq!(request.seed, 17);
        assert_eq!(request.count, 5);
        assert_eq!(request.volume, zone.spawn_groups[0].volume);
        zone.spawn_groups[0].respawn_seconds = 86400;
        let other = binding(&zone, 1, definition()).unwrap();
        assert_eq!(other.respawn_ms(), 86_400_000);
        assert_ne!(
            resolved.profile_identity([0; 32]),
            other.profile_identity([0; 32])
        );
    }
    #[test]
    fn unsupported_runtime_features_are_not_silently_discarded() {
        let mut zone = zone();
        assert!(binding(&zone, 99, definition()).is_err());
        zone.spawn_groups[0].model = Some("theia.model.wolf".into());
        assert!(
            binding(&zone, 1, definition())
                .err()
                .unwrap()
                .to_string()
                .contains("model")
        );
        // Placement-only checking may still inspect a model-bearing draft.
        assert!(request(&zone, 1, 3., zone.spawn).is_ok());
        zone.spawn_groups[0].model = None;
        zone.spawn_groups[0].target_alive = 17;
        assert!(
            binding(&zone, 1, definition())
                .err()
                .unwrap()
                .to_string()
                .contains("16-target")
        );
        zone.spawn_groups[0].target_alive = 5;
        zone.spawn_groups[0].creature = "theia.creature.unknown".into();
        assert!(binding(&zone, 1, definition()).is_err());
        zone.spawn_groups[0].creature = definition().id().as_str().into();
        zone.spawn_groups[0].enabled = false;
        assert!(binding(&zone, 1, definition()).is_err());
        assert!(request(&zone, 1, 3., zone.spawn).is_err());
    }
}
