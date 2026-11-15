//! Validate escort bindings and prepare their runtime routes before publication.
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    quests::catalog::{Catalog, EventKey, Follow},
};
use theia_network::{authored_world, navigation};
use theia_zone::{Zone, markers::MarkerKind};

struct Binding {
    quest: ContentId,
    npc: ContentId,
    home: GlobalPosition,
    policy: Follow,
}

fn bindings(zone: &Zone, bytes: &[u8]) -> Result<Vec<Binding>, String> {
    let catalog = Catalog::from_json(bytes).map_err(str::to_owned)?;
    let mut bindings = Vec::new();
    for quest in catalog.quests() {
        for objective in quest.stages().flat_map(|s| s.objectives()) {
            let (EventKey::FollowNpc(npc), Some(policy)) = (objective.event(), objective.follow())
            else {
                continue;
            };
            let mut homes = zone.markers.iter().filter(|m| {
                m.kind == MarkerKind::Npc && m.definition.as_deref() == Some(npc.as_str())
            });
            let marker = homes.next().ok_or_else(|| {
                format!(
                    "Escort '{}' needs its NPC marker: {}",
                    quest.title(),
                    npc.as_str()
                )
            })?;
            if homes.next().is_some() {
                return Err(format!(
                    "Escort '{}' has duplicate NPC markers",
                    quest.title()
                ));
            }
            let home = GlobalPosition::new(marker.position).ok_or("Invalid escort NPC position")?;
            navigation::escort::destination(home, policy, &zone.markers).map_err(|e| {
                format!(
                    "Escort '{}' destination {}: {e}",
                    quest.title(),
                    policy.destination
                )
            })?;
            bindings.push(Binding {
                quest: quest.id().clone(),
                npc: npc.clone(),
                home,
                policy: policy.clone(),
            });
        }
    }
    Ok(bindings)
}

/// Cheap reference checks before any package directory or navigation is written.
pub(super) fn validate_bindings(zone: &Zone, bytes: &[u8]) -> Result<(), String> {
    bindings(zone, bytes).map(|_| ())
}

#[derive(serde::Serialize)]
pub(super) struct Checked {
    quest: String,
    npc: String,
    destination: u64,
    steering_points: usize,
}

/// Worker-only: use the same support, route and clearance checks as the server.
pub(super) fn prepare(
    zone: &Zone,
    bytes: &[u8],
    world: &authored_world::Store,
    nav: &navigation::Store,
) -> Result<Vec<Checked>, String> {
    bindings(zone, bytes)?
        .into_iter()
        .map(|binding| {
            let route = navigation::escort::prepare(
                world,
                nav,
                binding.home,
                &binding.policy,
                &zone.markers,
            )
            .map_err(|e| {
                format!(
                    "Escort '{}' to destination {}: {e}",
                    binding.quest.as_str(),
                    binding.policy.destination
                )
            })?;
            Ok(Checked {
                quest: binding.quest.as_str().into(),
                npc: binding.npc.as_str().into(),
                destination: binding.policy.destination,
                steering_points: route.len(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_zone::markers::Marker;
    const QUESTS: &[u8] = include_bytes!("../../../content/quests/development-starter-quests.json");

    fn fixture() -> Zone {
        Zone {
            markers: vec![
                Marker {
                    id: 1,
                    name: "Elowen".into(),
                    kind: MarkerKind::Npc,
                    position: [8194., 0., 8174.],
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: Some("theia.dev.ranger".into()),
                },
                Marker {
                    id: 1202,
                    name: "Ancient tree".into(),
                    kind: MarkerKind::Quest,
                    position: [8276., 1.75, 8128.],
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: None,
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn missing_mistyped_duplicate_and_distant_bindings_are_rejected_without_mutation() {
        let zone = fixture();
        let before = serde_json::to_vec(&zone).unwrap();
        validate_bindings(&zone, QUESTS).unwrap();
        assert_eq!(before, serde_json::to_vec(&zone).unwrap());
        for case in 0..5 {
            let mut bad = zone.clone();
            match case {
                0 => {
                    bad.markers.pop();
                }
                1 => bad.markers[1].kind = MarkerKind::Npc,
                2 => bad.markers[1].position[0] += 512.,
                3 => bad.markers[0].definition = Some("theia.dev.other".into()),
                _ => {
                    let mut duplicate = bad.markers[0].clone();
                    duplicate.id = 2;
                    bad.markers.push(duplicate);
                }
            }
            assert!(
                validate_bindings(&bad, QUESTS)
                    .unwrap_err()
                    .contains("Escort")
            );
        }
    }

    #[test]
    #[ignore = "requires THEIA_QUEST_TRAVEL_PACKAGE with actual authored collision/navigation"]
    fn authored_escort_preflight_checks_route_and_rejects_wrong_floor() {
        let root =
            std::path::PathBuf::from(std::env::var_os("THEIA_QUEST_TRAVEL_PACKAGE").unwrap());
        let source = std::fs::read(root.join("zone.json")).unwrap();
        let zone = Zone::load(&root.join("zone.json")).unwrap();
        let world = authored_world::Store::open(&root.join("world")).unwrap();
        let nav = navigation::Store::open(&root.join("navigation"), world.identity()).unwrap();
        let snapshot = theia_network::gameplay_bundle::snapshot(
            &root.join("world/gameplay/gameplay.manifest.json"),
        )
        .unwrap();
        let bytes = snapshot.profile("quest").unwrap();
        let checked = prepare(&zone, bytes, &world, &nav).unwrap();
        assert_eq!(checked.len(), 1);
        assert_eq!(checked[0].destination, 1202);
        assert!(checked[0].steering_points >= 2);
        let mut bad = zone.clone();
        bad.markers
            .iter_mut()
            .find(|m| m.id == 1202)
            .unwrap()
            .position[1] += 8.;
        let error = prepare(&bad, bytes, &world, &nav).err().unwrap();
        assert!(
            error.contains("Escort") && error.contains("1202"),
            "{error}"
        );
        assert_eq!(source, std::fs::read(root.join("zone.json")).unwrap());
        println!("{}", serde_json::to_string(&checked).unwrap());
    }
}
