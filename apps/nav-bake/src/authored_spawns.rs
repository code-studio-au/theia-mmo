//! Read-only candidate checks against the exact authored world/navigation pair.
use std::{fs, io::Read, path::Path};
use theia_network::{
    Result, authored_world,
    navigation::{self as nav, spawn_candidates},
};

pub(super) fn check(source: &Path, navigation: &Path, request_path: &Path) -> Result<()> {
    let mut bytes = Vec::new();
    fs::File::open(request_path)?
        .take(32 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32 * 1024 {
        return Err("Spawn candidate request exceeds byte budget".into());
    }
    let request: spawn_candidates::Request = serde_json::from_slice(&bytes)?;
    check_request(source, navigation, &request, None)
}

fn check_request(
    source: &Path,
    navigation: &Path,
    request: &spawn_candidates::Request,
    authoring: Option<serde_json::Value>,
) -> Result<()> {
    let store = authored_world::Store::open(source)?;
    let navigation = nav::Store::open(navigation, store.identity())?;
    let checked = nav::spawn_preparation::check_authored(&store, &navigation, request)?;
    let mut report = checked.report().clone();
    report["authored_spawn_candidates"]["authoring"] = serde_json::json!(authoring);
    println!("{report}");
    checked.into_prepared()?;
    Ok(())
}

/// Read saved editor bounds directly. Access and spacing are explicit checker
/// inputs; neither becomes a roaming rule or changes the saved source.
pub(super) fn check_zone(
    source: &Path,
    navigation: &Path,
    zone: &Path,
    id: u64,
    spacing: f64,
    access: [f64; 3],
) -> Result<()> {
    let zone = theia_zone::Zone::load(zone)?;
    let (request, authoring) = zone_request(&zone, id, spacing, access)?;
    check_request(source, navigation, &request, Some(authoring))
}
fn zone_request(
    zone: &theia_zone::Zone,
    id: u64,
    spacing: f64,
    access: [f64; 3],
) -> Result<(spawn_candidates::Request, serde_json::Value)> {
    let request =
        theia_network::simulation::population::authoring::request(zone, id, spacing, access)?;
    let group = zone.spawn_groups.iter().find(|g| g.id == id).unwrap();
    let authoring = serde_json::json!({
        "zone_digest": theia_network::terrain::sha256(&serde_json::to_vec(zone)?),
        "group": group,
        "scope": "Saved authoring request checked against the supplied world; zone geometry equivalence and creature/model binding are not validated"
    });
    Ok((request, authoring))
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::spawn_volume::PolygonVolume;
    fn zone() -> theia_zone::Zone {
        let origin = [128_000_000_000.125, 20.25, -128_000_000_000.5];
        theia_zone::Zone {
            schema: 14,
            origin,
            spawn: origin,
            spawn_groups: vec![theia_zone::spawn_groups::SpawnGroup {
                id: 7,
                name: "Grove".into(),
                enabled: true,
                creature: "theia.dev.training-creature".into(),
                model: Some("theia.model.wolf".into()),
                volume: PolygonVolume::new(
                    vec![[0., 0.], [8., 0.], [8., 3.], [3., 3.], [3., 8.], [0., 8.]],
                    -2.,
                    8.,
                )
                .unwrap()
                .translated(origin)
                .unwrap(),
                target_alive: 5,
                respawn_seconds: 30,
                roam_radius_metres: 0.,
                patrol_path: None,
                seed: u64::MAX,
            }],
            ..Default::default()
        }
    }
    #[test]
    fn saved_spawn_requests_keep_exact_bounds_seed_and_population_metadata() {
        let zone = zone();
        let (request, report) = zone_request(&zone, 7, 3., zone.spawn).unwrap();
        assert_eq!(request.volume, zone.spawn_groups[0].volume);
        assert_eq!(request.seed, u64::MAX);
        assert_eq!(request.count, 5);
        assert_eq!(request.id, "theia.zone.spawn.7");
        assert_eq!(
            report["group"],
            serde_json::to_value(&zone.spawn_groups[0]).unwrap()
        );
        let mut renamed = zone.clone();
        renamed.spawn_groups[0].name = "Renamed".into();
        let (other, other_report) = zone_request(&renamed, 7, 3., zone.spawn).unwrap();
        assert_eq!(request.id, other.id);
        assert_ne!(report["zone_digest"], other_report["zone_digest"]);
        for (id, spacing, access) in [
            (8, 3., zone.spawn),
            (7, 0., zone.spawn),
            (7, f64::NAN, zone.spawn),
            (7, 3., [f64::INFINITY; 3]),
        ] {
            assert!(zone_request(&zone, id, spacing, access).is_err());
        }
        let mut disabled = zone.clone();
        disabled.spawn_groups[0].enabled = false;
        assert!(zone_request(&disabled, 7, 3., zone.spawn).is_err());
        assert_eq!(zone.spawn_groups[0].seed, u64::MAX);
    }
}
