//! Authoring requests only; the offline baker must resolve and validate them.
use std::{fs, path::Path};
use theia_network::navigation::Route;
use theia_zone::Zone;

pub(super) fn write(zone: &Zone, stage: &Path) -> Result<serde_json::Value, String> {
    let mut report = Vec::new();
    for path in &zone.paths {
        let Some(id) = path.patrol_route_id() else {
            continue;
        };
        let route = Route {
            id: id.clone(),
            points: path.points.clone(),
        };
        let file = format!("navigation-requests/patrol-{}.json", path.id);
        let output = stage.join(&file);
        fs::create_dir_all(output.parent().ok_or("Missing route parent")?)
            .map_err(|e| e.to_string())?;
        fs::write(
            output,
            serde_json::to_vec_pretty(&route).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        report.push(serde_json::json!({"id": id, "name": path.name, "file": file, "points": route.points.len(),
            "navigation_baked": false}));
    }
    Ok(serde_json::json!(report))
}
