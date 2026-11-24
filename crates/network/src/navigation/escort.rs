//! Shared editor/server preparation for destination-only escort travel.
//! All navigation and collision I/O belongs outside simulation and admission.
use crate::{Result, authored_world, navigation};
use theia_core::{coordinates::GlobalPosition, quests::catalog::Follow};
use theia_zone::markers::{Marker, MarkerKind};

pub fn destination(
    home: GlobalPosition,
    policy: &Follow,
    markers: &[Marker],
) -> Result<GlobalPosition> {
    let mut matches = markers.iter().filter(|m| m.id == policy.destination);
    let marker = matches
        .next()
        .ok_or("Follow destination marker is absent")?;
    if matches.next().is_some() || marker.kind != MarkerKind::Quest {
        return Err("Follow destination must identify exactly one Quest marker".into());
    }
    let end = GlobalPosition::new(marker.position).ok_or("Invalid follow destination")?;
    let distance = home
        .metres()
        .into_iter()
        .zip(end.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    if distance > 512. {
        return Err("Follow destination exceeds 512 metre budget".into());
    }
    Ok(end)
}

pub fn prepare(
    world: &authored_world::Store,
    nav: &navigation::Store,
    home: GlobalPosition,
    policy: &Follow,
    markers: &[Marker],
) -> Result<Vec<GlobalPosition>> {
    let owner = crate::procedural::owner(home).ok_or("Follow home outside terrain")?;
    let home = world
        .patch(owner)?
        .try_spawn_near(home)
        .ok_or("Follow home has no supported placement")?;
    let end = destination(home, policy, markers)?;
    navigation::authored_checks::grounded_route(world, nav, &[home.metres(), end.metres()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_requires_unique_quest_marker_in_bounded_metric_range() {
        let home = GlobalPosition::new([8000., 0., 8000.]).unwrap();
        let policy = Follow {
            destination: 42,
            max_distance_metres: 10,
            speed_millimetres_per_second: 7000,
        };
        let marker = Marker {
            id: 42,
            name: "Ancient tree".into(),
            kind: MarkerKind::Quest,
            position: [8100., 0., 8000.],
            yaw_degrees: 0.,
            radius_metres: 1.,
            definition: None,
        };
        assert_eq!(
            destination(home, &policy, std::slice::from_ref(&marker))
                .unwrap()
                .metres(),
            marker.position
        );
        assert!(destination(home, &policy, &[]).is_err());
        assert!(destination(home, &policy, &[marker.clone(), marker.clone()]).is_err());
        for invalid in [
            Marker {
                kind: MarkerKind::Npc,
                ..marker.clone()
            },
            Marker {
                position: [8512.01, 0., 8000.],
                ..marker.clone()
            },
            Marker {
                position: [8100., f64::NAN, 8000.],
                ..marker
            },
        ] {
            assert!(destination(home, &policy, &[invalid]).is_err());
        }
    }
}
