//! Bounded copies in the shared authoring ID namespace.
use crate::{MAX_OBJECTS, Result, ZONE_EXTENT, Zone, require};

fn copy_name(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if result.len() + c.len_utf8() > 75 {
            break;
        }
        result.push(c);
    }
    result.push_str(" copy");
    result
}

pub(crate) fn apply(zone: &mut Zone, id: u64, new_id: u64, offset: [f64; 3]) -> Result<()> {
    require(new_id > 0, "Copy needs a positive new ID")?;
    let mut occupied = zone
        .objects
        .iter()
        .map(|o| o.id)
        .chain(zone.paths.iter().map(|p| p.id))
        .chain(zone.markers.iter().map(|m| m.id))
        .chain(zone.plants.iter().map(|p| p.id));
    require(!occupied.any(|id| id == new_id), "Copy ID already exists")?;
    require(
        offset
            .iter()
            .all(|v| v.is_finite() && v.abs() <= 2. * ZONE_EXTENT),
        "Copy offset must be finite and within +/-1024 metres",
    )?;
    let shift = |position: [f64; 3]| std::array::from_fn(|i| position[i] + offset[i]);
    if let Some(source) = zone.objects.iter().find(|o| o.id == id) {
        require(zone.objects.len() < MAX_OBJECTS, "Object budget exceeded")?;
        let mut copy = source.clone();
        copy.id = new_id;
        copy.name = copy_name(&copy.name);
        copy.pose.position = shift(copy.pose.position);
        zone.objects.push(copy);
    } else if let Some(source) = zone.paths.iter().find(|p| p.id == id) {
        require(
            zone.paths.len() < crate::paths::MAX_PATHS,
            "Path budget exceeded",
        )?;
        let mut copy = source.clone();
        copy.id = new_id;
        copy.name = copy_name(&copy.name);
        copy.points = copy.points.into_iter().map(shift).collect();
        zone.paths.push(copy);
    } else if let Some(source) = zone.markers.iter().find(|m| m.id == id) {
        require(
            zone.markers.len() < crate::markers::MAX_MARKERS,
            "Marker budget exceeded",
        )?;
        let mut copy = source.clone();
        copy.id = new_id;
        copy.name = copy_name(&copy.name);
        copy.position = shift(copy.position);
        zone.markers.push(copy);
    } else if let Some(source) = zone.plants.iter().find(|p| p.id == id) {
        require(
            zone.plants.len() < crate::vegetation::MAX_PLANTS,
            "Vegetation budget exceeded",
        )?;
        let mut copy = source.clone();
        copy.id = new_id;
        copy.name = copy_name(&copy.name);
        copy.position = shift(copy.position);
        zone.plants.push(copy);
    } else {
        return Err("Unknown authoring ID to copy".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn editor() -> Editor {
        let mut editor = Editor::new(Zone::default()).unwrap();
        let command = serde_json::from_slice(include_bytes!(
            "../../../content/zones/selection.commands.jsonl"
        ))
        .unwrap();
        editor.execute(command).unwrap();
        editor
    }
    fn duplicate(revision: u64, id: u64, new_id: u64, offset: [f64; 3]) -> Command {
        Command::Apply {
            expected_revision: revision,
            edits: vec![Edit::Duplicate { id, new_id, offset }],
        }
    }
    #[test]
    fn all_authoring_kinds_copy_in_one_transaction_and_undo_with_far_offsets() {
        for sign in [-1., 0., 1.] {
            let mut zone = editor().zone().clone();
            let delta = [sign * 128_000_000_000., 20., -sign * 128_000_000_000.];
            let shift = |p: [f64; 3]| std::array::from_fn(|i| p[i] + delta[i]);
            zone.origin = shift(zone.origin);
            zone.spawn = shift(zone.spawn);
            for o in &mut zone.objects {
                o.pose.position = shift(o.pose.position);
            }
            for p in &mut zone.paths {
                for point in &mut p.points {
                    *point = shift(*point);
                }
            }
            for m in &mut zone.markers {
                m.position = shift(m.position);
            }
            for p in &mut zone.plants {
                p.position = shift(p.position);
            }
            let original = zone.clone();
            let mut e = Editor::new(zone).unwrap();
            let offset = [1., 0.5, -2.];
            let edits = [1, 1000, 1001, 1002]
                .into_iter()
                .enumerate()
                .map(|(i, id)| Edit::Duplicate {
                    id,
                    new_id: 2001 + i as u64,
                    offset,
                })
                .collect();
            let command = Command::Apply {
                expected_revision: 0,
                edits,
            };
            e.execute(serde_json::from_slice(&serde_json::to_vec(&command).unwrap()).unwrap())
                .unwrap();
            let copied = e.zone().clone();
            let move_by = |p: [f64; 3]| std::array::from_fn(|i| p[i] + offset[i]);
            let mut expected = original.clone();
            let mut object = original.objects[0].clone();
            object.id = 2001;
            object.name.push_str(" copy");
            object.pose.position = move_by(object.pose.position);
            expected.objects.push(object);
            let mut marker = original.markers[0].clone();
            marker.id = 2002;
            marker.name.push_str(" copy");
            marker.position = move_by(marker.position);
            expected.markers.push(marker);
            let mut plant = original.plants[0].clone();
            plant.id = 2003;
            plant.name.push_str(" copy");
            plant.position = move_by(plant.position);
            expected.plants.push(plant);
            let mut path = original.paths[0].clone();
            path.id = 2004;
            path.name.push_str(" copy");
            path.points = path.points.into_iter().map(move_by).collect();
            expected.paths.push(path);
            assert_eq!(copied, expected);
            assert_eq!(e.history(), (1, 0));
            assert_eq!(copied.next_id(), Some(2005));
            e.execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
            assert_eq!(e.zone(), &original);
            e.execute(Command::Redo {
                expected_revision: 2,
            })
            .unwrap();
            assert_eq!(e.zone(), &copied);
        }
    }
    #[test]
    fn invalid_copy_preserves_candidate_revision_and_redo() {
        let mut e = editor();
        e.execute(duplicate(1, 1, 2000, [0.; 3])).unwrap();
        e.execute(Command::Undo {
            expected_revision: 2,
        })
        .unwrap();
        let original = e.zone().clone();
        let history = e.history();
        for (id, new_id, offset) in [
            (999, 2000, [0.; 3]),
            (1, 0, [0.; 3]),
            (1, 1, [0.; 3]),
            (1, 1000, [0.; 3]),
            (1, 1001, [0.; 3]),
            (1, 1002, [0.; 3]),
            (1, 2000, [f64::NAN, 0., 0.]),
            (1, 2000, [1024.1, 0., 0.]),
            (1, 2000, [200., 0., 0.]),
        ] {
            assert!(e.execute(duplicate(3, id, new_id, offset)).is_err());
            assert_eq!(e.zone(), &original);
            assert_eq!(e.revision(), 3);
            assert_eq!(e.history(), history);
        }
        assert!(e.execute(duplicate(2, 1, 2000, [0.; 3])).is_err());
        assert!(
            e.execute(Command::Apply {
                expected_revision: 3,
                edits: vec![
                    Edit::SetSpawn {
                        position: [1., 0.1, 6.]
                    },
                    Edit::Duplicate {
                        id: 999,
                        new_id: 2000,
                        offset: [0.; 3]
                    }
                ]
            })
            .is_err()
        );
        assert_eq!(e.zone(), &original);
        assert_eq!(e.history(), history);
        e.execute(Command::Redo {
            expected_revision: 3,
        })
        .unwrap();
        assert_eq!(e.zone().objects.len(), 4);
    }
    #[test]
    fn copied_name_is_utf8_bounded_and_all_metadata_is_retained() {
        let mut zone = editor().zone().clone();
        zone.markers[0].name = "界".repeat(26) + "é";
        zone.markers[0].definition = Some("npc.test.guard".into());
        let mut e = Editor::new(zone.clone()).unwrap();
        e.execute(duplicate(0, 1000, 2000, [0.; 3])).unwrap();
        let copy = &e.zone().markers[1];
        assert_eq!(copy.name.len(), 80);
        assert!(copy.name.ends_with(" copy"));
        let mut expected = zone.markers[0].clone();
        expected.id = 2000;
        expected.name = "界".repeat(25) + " copy";
        assert_eq!(copy, &expected);
        assert_eq!(e.zone().markers[0], zone.markers[0]);
    }
    #[test]
    fn every_copy_kind_respects_its_existing_count_budget() {
        let source = editor().zone().clone();
        for kind in 0..4 {
            let mut zone = source.clone();
            zone.objects.clear();
            zone.paths.clear();
            zone.markers.clear();
            zone.plants.clear();
            let limit = match kind {
                0 => MAX_OBJECTS,
                1 => crate::paths::MAX_PATHS,
                2 => crate::markers::MAX_MARKERS,
                _ => crate::vegetation::MAX_PLANTS,
            };
            for id in 1..=limit as u64 {
                match kind {
                    0 => {
                        let mut v = source.objects[0].clone();
                        v.id = id;
                        zone.objects.push(v);
                    }
                    1 => {
                        let mut v = source.paths[0].clone();
                        v.id = id;
                        zone.paths.push(v);
                    }
                    2 => {
                        let mut v = source.markers[0].clone();
                        v.id = id;
                        zone.markers.push(v);
                    }
                    _ => {
                        let mut v = source.plants[0].clone();
                        v.id = id;
                        zone.plants.push(v);
                    }
                }
            }
            let mut e = Editor::new(zone.clone()).unwrap();
            assert!(
                e.execute(duplicate(0, 1, limit as u64 + 1, [0.; 3]))
                    .is_err()
            );
            assert_eq!(e.zone(), &zone);
            assert_eq!(e.revision(), 0);
            assert_eq!(e.history(), (0, 0));
        }
    }
}
