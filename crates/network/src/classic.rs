//! Strict classic movement/facing profile. No camera or target state is accepted.
use crate::{prediction::Pose, wire::Intent};
use std::collections::BTreeSet;
use theia_core::{coordinates::ChunkId, time::TickRate};

pub fn profile_identity(source: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-classic-ground-movement-v3\0".to_vec();
    bytes.extend(source);
    bytes.extend(include_bytes!(
        "../../../content/movement/classic-ground.json"
    ));
    crate::terrain::sha256(&bytes)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundSpeeds {
    schema: u32,
    pub walk_metres_per_second: f32,
    pub run_metres_per_second: f32,
    pub backward_run_metres_per_second: f32,
}
impl GroundSpeeds {
    fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > 1024 {
            return Err("Classic ground speeds exceed 1024 bytes");
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| "Invalid classic ground speeds")?;
        let [walk, backward, run] = [
            value.walk_metres_per_second,
            value.backward_run_metres_per_second,
            value.run_metres_per_second,
        ];
        if value.schema != 1
            || ![walk, backward, run]
                .into_iter()
                .all(|s| s.is_finite() && (0.1..=8.).contains(&s))
            || walk > backward
            || backward > run
        {
            return Err("Invalid classic ground speed bounds");
        }
        Ok(value)
    }
}

/// Read-only rates from the exact embedded, world-identity-bound movement data.
pub fn ground_speeds() -> &'static GroundSpeeds {
    static SPEEDS: std::sync::OnceLock<GroundSpeeds> = std::sync::OnceLock::new();
    SPEEDS.get_or_init(|| {
        GroundSpeeds::parse(include_bytes!(
            "../../../content/movement/classic-ground.json"
        ))
        .expect("Validated embedded classic ground speeds")
    })
}
/// The retained wire sprint bit means run / walk in v3. Accepted character
/// facing selects backward travel; one axis unit tolerates strafe quantization.
pub(crate) fn ground_speed(intent: Intent) -> Result<f32, &'static str> {
    let facing = intent.facing.ok_or("Missing classic movement facing")?;
    if !intent.valid() || intent.dodge.is_some() {
        return Err("Invalid classic ground movement");
    }
    let speeds = ground_speeds();
    let forward = facing.forward();
    let along = f64::from(intent.axes[0]) * forward[0] + f64::from(intent.axes[1]) * forward[1];
    Ok(if !intent.sprint {
        speeds.walk_metres_per_second
    } else if along < -1. {
        speeds.backward_run_metres_per_second
    } else {
        speeds.run_metres_per_second
    })
}

pub fn terrain_step(
    patch: &crate::procedural::Patch,
    ready: &BTreeSet<ChunkId>,
    pose: Pose,
    intent: Intent,
    rate: TickRate,
) -> Result<Option<Pose>, &'static str> {
    let facing = pose.facing.ok_or("Missing classic character facing")?;
    if pose.dodge.is_some() || intent.dodge.is_some() {
        return Err("Dodge is excluded from classic movement");
    }
    let mut base = pose;
    base.facing = None;
    let mut movement = intent;
    movement.facing = Some(intent.facing.unwrap_or(facing));
    Ok(
        crate::prediction::terrain_step_classic(patch, ready, base, movement, rate)?.map(|p| {
            Pose {
                facing: Some(intent.facing.unwrap_or(facing)),
                ..p
            }
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::facing::Facing;

    #[test]
    fn quantized_strafing_and_backpedalling_select_speeds_at_every_heading() {
        for units in 0..=u16::MAX {
            let facing = Facing::from_units(units);
            for (local, speed) in [
                ([0, -1000], 7.),
                ([1000, 0], 7.),
                ([-1000, 0], 7.),
                ([1000, -1000], 7.),
                ([0, 1000], 4.5),
                ([1000, 1000], 4.5),
            ] {
                let intent = Intent {
                    facing: Some(facing),
                    axes: facing.world_axes(local).unwrap(),
                    sprint: true,
                    ..Intent::default()
                };
                assert_eq!(
                    ground_speed(intent),
                    Ok(speed),
                    "heading={units}, {local:?}"
                );
                assert_eq!(
                    ground_speed(Intent {
                        sprint: false,
                        ..intent
                    }),
                    Ok(2.5)
                );
            }
        }
    }

    #[test]
    fn strict_bounded_speed_data_and_versioned_identity() {
        let bytes = include_bytes!("../../../content/movement/classic-ground.json");
        assert!(GroundSpeeds::parse(bytes).is_ok());
        for (key, value) in [
            ("schema", serde_json::json!(2)),
            ("walk_metres_per_second", serde_json::json!(0)),
            ("walk_metres_per_second", serde_json::json!(8)),
            ("run_metres_per_second", serde_json::json!(8.1)),
            ("backward_run_metres_per_second", serde_json::json!(8)),
            ("unknown", serde_json::json!(1)),
        ] {
            let mut data: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            data[key] = value;
            assert!(GroundSpeeds::parse(&serde_json::to_vec(&data).unwrap()).is_err());
        }
        assert!(GroundSpeeds::parse(&vec![b' '; 1025]).is_err());
        assert!(ground_speed(Intent::default()).is_err());
        let mut old = b"theia-classic-facing-targeting-v2\0".to_vec();
        old.extend([7; 32]);
        assert_ne!(profile_identity([7; 32]), crate::terrain::sha256(&old));
        assert_ne!(profile_identity([7; 32]), profile_identity([8; 32]));
    }
}
