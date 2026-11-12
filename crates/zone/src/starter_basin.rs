//! A small editable terrain starting point, using the existing height-patch format.
//! Geometry alone is not navigation or authoritative containment qualification.
use crate::{
    Result, Zone, require,
    terrain::{HeightPatch, Resolution},
};

pub fn available(zone: &Zone) -> bool {
    zone.terrain.is_none()
        && zone.objects.is_empty()
        && zone.paths.is_empty()
        && zone.plants.is_empty()
        && zone.markers.is_empty()
        && zone.particles.is_empty()
        && zone.quest_areas.is_empty()
        && zone.scenery.is_empty()
}

pub(crate) fn apply(zone: &mut Zone) -> Result<()> {
    require(
        available(zone),
        "Create a new blank zone before adding starter basin terrain",
    )?;
    let chunk = theia_core::coordinates::GlobalPosition::new(zone.origin)
        .and_then(|p| p.chunk(128))
        .ok_or("Invalid basin origin")?;
    let mut patch = HeightPatch {
        chunk: [chunk.x, chunk.z],
        resolution: Resolution::TwoMetres,
        heights: vec![0.; 65 * 65],
        weights: vec![crate::painting::GRASS; 65 * 65],
    };
    let start = patch.origin()?;
    for z in 1..64 {
        for x in 1..64 {
            let dx = x as f64 * 2. - 64.;
            let dz = z as f64 * 2. - 64.;
            let radius = dx.hypot(dz);
            let inner = ((radius - 36.) / 6.).clamp(0., 1.);
            let outer = ((60. - radius) / 8.).clamp(0., 1.);
            // One twelve-metre floor opening on +Z, with steep four-metre sides.
            let gate = if dz > 0. {
                ((dx.abs() - 6.) / 4.).clamp(0., 1.)
            } else {
                1.
            };
            let height = (16. * inner * outer * gate) as f32;
            let index = z * 65 + x;
            patch.heights[index] = height;
            patch.weights[index] = if height > 0. {
                [0, 0, 255, 0]
            } else if dx.abs() <= 4. && dz >= 0. {
                [0, 255, 0, 0]
            } else {
                crate::painting::GRASS
            };
        }
    }
    zone.schema = zone.schema.max(7);
    zone.spawn = [start[0] + 64., zone.origin[1] + 0.1, start[2] + 64.];
    zone.terrain = Some(patch);
    zone.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    #[test]
    fn basin_is_one_undo_preserves_borders_and_refuses_replacing_content() {
        for origin in [[0.; 3], [128_000_000_000.125, 20., -128_000_000_000.25]] {
            let source = Zone {
                origin,
                spawn: origin,
                ..Zone::default()
            };
            let mut editor = Editor::new(source.clone()).unwrap();
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::CreateStarterBasin],
                })
                .unwrap();
            let created = editor.zone().clone();
            created.validate().unwrap();
            let patch = created.terrain.as_ref().unwrap();
            assert_eq!(patch.axis(), 65);
            let start = patch.origin().unwrap();
            for z in [0., 16., 32., 48., 64.] {
                assert_eq!(
                    patch.height(start[0] + 64., start[2] + 64. + z).unwrap(),
                    Some(0.)
                );
            }
            assert_eq!(
                patch.height(start[0] + 16., start[2] + 64.).unwrap(),
                Some(16.)
            );
            assert!(
                editor
                    .execute(Command::Apply {
                        expected_revision: 1,
                        edits: vec![Edit::CreateStarterBasin]
                    })
                    .is_err()
            );
            assert_eq!(editor.zone(), &created);
            editor
                .execute(Command::Undo {
                    expected_revision: 1,
                })
                .unwrap();
            assert_eq!(editor.zone(), &source);
            editor
                .execute(Command::Redo {
                    expected_revision: 2,
                })
                .unwrap();
            assert_eq!(editor.zone(), &created);
        }
        let mut occupied = Zone::default();
        occupied.objects.push(crate::Object {
            id: 1,
            name: "Existing".into(),
            primitive: crate::Primitive::Box,
            pose: crate::Pose {
                position: [0., 1., 0.],
                yaw_degrees: 0.,
                size: [1.; 3],
            },
        });
        let before = occupied.clone();
        assert!(apply(&mut occupied).is_err());
        assert_eq!(occupied, before);
    }
}
