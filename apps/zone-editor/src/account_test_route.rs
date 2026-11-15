//! Explicit, paused account playtest travel from a saved road guide.
use theia_network::{authored_world, navigation};
use theia_zone::{
    Zone,
    paths::{GuidePath, PathKind},
};

pub(super) fn road(zone: &Zone, id: u64) -> Result<&GuidePath, String> {
    let mut matching = zone.paths.iter().filter(|p| p.id == id);
    let road = matching.next().ok_or("AI test road guide ID is absent")?;
    if road.kind != PathKind::Road || matching.next().is_some() {
        return Err("AI test travel needs one saved Road guide".into());
    }
    road.validate(zone)?;
    Ok(road)
}

/// The manual choice participates in cycling, but is never replaced on open.
pub(super) fn cycle(zone: &Zone, current: &str, forward: bool) -> Option<u64> {
    let mut ids: Vec<_> = zone
        .paths
        .iter()
        .filter(|p| p.kind == PathKind::Road)
        .map(|p| p.id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let choices: Vec<_> = std::iter::once(None)
        .chain(ids.into_iter().map(Some))
        .collect();
    let current = current.trim().parse().ok();
    let index = choices.iter().position(|id| *id == current).unwrap_or(0);
    choices[if forward {
        (index + 1) % choices.len()
    } else {
        (index + choices.len() - 1) % choices.len()
    }]
}

pub(super) fn choice_label(zone: &Zone, current: &str) -> String {
    let value = current.trim();
    if value.is_empty() {
        return "Manual play".into();
    }
    match value.parse::<u64>().ok().and_then(|id| road(zone, id).ok()) {
        Some(path) => format!("{} (#{})", path.name, path.id),
        None => "No matching saved Road guide".into(),
    }
}

pub(super) fn prepare(
    zone: &Zone,
    id: u64,
    world: &authored_world::Store,
    nav: &navigation::Store,
) -> Result<(Vec<u8>, serde_json::Value), String> {
    let road = road(zone, id)?;
    let route = navigation::authored_checks::grounded_route(world, nav, &road.points)
        .map_err(|e| format!("AI test road '{}' (#{id}): {e}", road.name))?;
    let bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema":1, "name":road.name, "navigation":"navigation", "waypoints":road.points,
        "start_paused":true, "return_trip":true,
    }))
    .map_err(|e| e.to_string())?;
    if bytes.len() > 32 * 1024 {
        return Err("AI test road exceeds 32 KiB".into());
    }
    Ok((
        bytes,
        serde_json::json!({"road":id,"name":road.name,"authored_points":road.points.len(),
        "steering_points":route.len(),"start_paused":true,"return_trip":true}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_choices_cycle_roads_in_id_order_with_explicit_manual_and_invalid_entry_recovery() {
        let mut zone = Zone {
            schema: 3,
            ..Default::default()
        };
        for (id, kind) in [
            (12, PathKind::Road),
            (3, PathKind::Patrol),
            (7, PathKind::Road),
        ] {
            zone.paths.push(GuidePath {
                id,
                name: format!("Guide {id}"),
                kind,
                width_metres: 1.,
                points: vec![[0.; 3], [8., 0., 0.]],
            });
        }
        let original = zone.clone();
        assert_eq!(cycle(&zone, "", true), Some(7));
        assert_eq!(cycle(&zone, "7", true), Some(12));
        assert_eq!(cycle(&zone, "12", true), None);
        assert_eq!(cycle(&zone, "", false), Some(12));
        assert_eq!(cycle(&zone, "12", false), Some(7));
        assert_eq!(cycle(&zone, "7", false), None);
        for unknown in ["3", "999", "text", "0"] {
            assert_eq!(cycle(&zone, unknown, true), Some(7));
            assert_eq!(cycle(&zone, unknown, false), Some(12));
            assert_eq!(choice_label(&zone, unknown), "No matching saved Road guide");
        }
        assert_eq!(choice_label(&zone, " 12 "), "Guide 12 (#12)");
        assert_eq!(choice_label(&zone, ""), "Manual play");
        assert_eq!(zone, original);
        assert_eq!(cycle(&Zone::default(), "", true), None);
        assert_eq!(cycle(&Zone::default(), "", false), None);
    }
    #[test]
    fn test_travel_requires_an_explicit_unique_valid_road() {
        let mut zone = Zone {
            schema: 3,
            paths: vec![GuidePath {
                id: 12,
                name: "Compass trail".into(),
                kind: PathKind::Road,
                width_metres: 2.,
                points: vec![[0., 0., 0.], [8., 0., 0.]],
            }],
            ..Default::default()
        };
        let original = zone.clone();
        assert_eq!(road(&zone, 12).unwrap().points.len(), 2);
        assert_eq!(zone, original);
        assert!(road(&zone, 0).is_err());
        assert!(road(&zone, 13).is_err());
        zone.paths[0].kind = PathKind::Patrol;
        assert!(road(&zone, 12).is_err());
        zone.paths[0].kind = PathKind::Stream;
        assert!(road(&zone, 12).is_err());
        zone = original.clone();
        zone.paths.push(zone.paths[0].clone());
        assert!(road(&zone, 12).is_err());
        zone = original;
        zone.paths[0].points[1] = [f64::NAN, 0., 0.];
        assert!(road(&zone, 12).is_err());
    }
    #[test]
    #[ignore = "requires THEIA_QUEST_TRAVEL_PACKAGE with actual authored collision/navigation"]
    fn authored_compass_road_preserves_minimal_points_and_rejects_wrong_floor() {
        let root =
            std::path::PathBuf::from(std::env::var_os("THEIA_QUEST_TRAVEL_PACKAGE").unwrap());
        let mut zone = Zone::load(&root.join("zone.json")).unwrap();
        let original = zone.clone();
        let world = authored_world::Store::open(&root.join("world")).unwrap();
        let nav = navigation::Store::open(&root.join("navigation"), world.identity()).unwrap();
        let (bytes, report) = prepare(&zone, 1203, &world, &nav).unwrap();
        let descriptor: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(report["authored_points"], 2);
        assert!(report["steering_points"].as_u64().unwrap() >= 2);
        assert_eq!(
            descriptor["waypoints"],
            serde_json::json!(road(&zone, 1203).unwrap().points)
        );
        assert_eq!(descriptor["navigation"], "navigation");
        assert_eq!(descriptor["start_paused"], true);
        assert_eq!(descriptor["return_trip"], true);
        assert_eq!(zone, original);
        zone.paths.iter_mut().find(|p| p.id == 1203).unwrap().points[1][1] += 8.;
        assert!(
            prepare(&zone, 1203, &world, &nav)
                .unwrap_err()
                .contains("AI test road")
        );
        println!("{report}");
    }
}
