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
            "passed":passed,"id":request.id,"source":store.identity(),"navigation":navigation.identity(),
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
