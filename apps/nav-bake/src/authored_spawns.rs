//! Read-only candidate checks against the exact authored world/navigation pair.
use crate::authored_cli::{Grounding, clearance};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
use theia_core::coordinates::GlobalPosition;
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
    request.validate()?;
    let store = authored_world::Store::open(source)?;
    let navigation = nav::Store::open(navigation, store.identity())?;
    navigation.verify()?;
    if navigation.manifest().agent != nav::Agent::development() {
        return Err(
            "Spawn collision checks currently require the development capsule profile".into(),
        );
    }
    let tiles = navigation
        .manifest()
        .tiles
        .iter()
        .map(|t| navigation.tile(t.id))
        .collect::<Result<Vec<_>>>()?;
    let query = theia_nav_bake::Query::new(&tiles)?;
    let mut grounding = Grounding::new(&store);
    let access = GlobalPosition::new(request.access_from).ok_or("Invalid spawn access point")?;
    query
        .sample(access)
        .map_err(|e| format!("Spawn access point is not on navigation: {e}"))?;
    let access = grounding
        .at(access)
        .map_err(|e| format!("Spawn access point has no valid standing pose: {e}"))?;
    let proposals = spawn_candidates::proposals(
        &tiles,
        &request.volume,
        request.seed,
        spawn_candidates::MAX_ATTEMPTS,
    )?;
    let mut accepted: Vec<[f64; 3]> = Vec::new();
    let mut rejected: BTreeMap<&str, usize> = BTreeMap::new();
    let mut examples: BTreeMap<&str, String> = BTreeMap::new();
    let mut examined = 0;
    let mut access_queries = 0;
    let mut clearance_samples = 0;
    let mut exhausted = false;
    for proposal in &proposals {
        if accepted.len() == request.count {
            break;
        }
        if access_queries >= 256 || clearance_samples >= 8192 {
            exhausted = true;
            break;
        }
        examined += 1;
        let point = match query.sample(*proposal).and_then(|p| grounding.at(p)) {
            Ok(p) => p,
            Err(error) => {
                reject(
                    &mut rejected,
                    &mut examples,
                    "support_or_clearance",
                    &error.to_string(),
                );
                continue;
            }
        };
        if !request.volume.contains(point) {
            reject(
                &mut rejected,
                &mut examples,
                "outside_final_bounds",
                "Grounded feet left the spawn bounds",
            );
            continue;
        }
        let metres = point.metres();
        if accepted.iter().any(|other| {
            (other[0] - metres[0]).hypot(other[2] - metres[2]) < request.minimum_spacing_metres
        }) {
            reject(
                &mut rejected,
                &mut examples,
                "spacing",
                "Candidate is too close to an accepted point",
            );
            continue;
        }
        access_queries += 1;
        let route = match query.route(access, point) {
            Ok(points) => points.iter().map(|p| p.metres()).collect::<Vec<_>>(),
            Err(error) => {
                reject(
                    &mut rejected,
                    &mut examples,
                    "inaccessible",
                    &error.to_string(),
                );
                continue;
            }
        };
        if let Err(error) = clearance(&query, &mut grounding, &route, &mut clearance_samples) {
            reject(
                &mut rejected,
                &mut examples,
                "access_clearance",
                &error.to_string(),
            );
            continue;
        }
        accepted.push(metres);
    }
    let passed = accepted.len() == request.count;
    println!(
        "{}",
        serde_json::json!({"authored_spawn_candidates": {
            "passed":passed,"id":request.id,"authoring":authoring,"source":store.identity(),"navigation":navigation.identity(),
            "seed":request.seed,"requested":request.count,"accepted":accepted,
            "proposal_budget":spawn_candidates::MAX_ATTEMPTS,"inside_proposals":proposals.len(),
            "examined":examined,"rejected":rejected,"rejection_examples":examples,
            "access_queries":access_queries,"clearance_samples":clearance_samples,"work_budget_exhausted":exhausted,
            "scope":"Read-only, bounded development-capsule candidate check with sampled access clearance; no actor spawns, population, respawn timing or roaming boundary"
        }})
    );
    if !passed {
        return Err("Insufficient validated spawn candidates within the bounded search; no population was created".into());
    }
    Ok(())
}
fn reject(
    counts: &mut BTreeMap<&'static str, usize>,
    examples: &mut BTreeMap<&'static str, String>,
    reason: &'static str,
    detail: &str,
) {
    *counts.entry(reason).or_default() += 1;
    examples
        .entry(reason)
        .or_insert_with(|| detail.chars().take(240).collect());
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
    zone.validate()?;
    let group = zone
        .spawn_groups
        .iter()
        .find(|g| g.id == id)
        .ok_or("Unknown saved spawn group ID")?;
    if !group.enabled {
        return Err("Selected spawn group is disabled".into());
    }
    let request = spawn_candidates::Request {
        schema: 1,
        id: format!("theia.zone.spawn.{}", group.id),
        volume: group.volume.clone(),
        count: usize::from(group.target_alive),
        seed: group.seed,
        minimum_spacing_metres: spacing,
        access_from: access,
    };
    request.validate()?;
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
