//! Quest volumes use shared authoring transactions; they are not marker annotations.
use super::*;

pub(super) fn add(session: &mut Session) {
    let id = session.next_id;
    if id == u64::MAX {
        session.status = "Authoring IDs exhausted".into();
        return;
    }
    let zone = session.editor.zone();
    let base = format!("theia.area.{id}");
    let definition = (0..=zone.quest_areas.len())
        .map(|suffix| {
            if suffix == 0 {
                base.clone()
            } else {
                format!("{base}.{suffix}")
            }
        })
        .find(|candidate| !zone.quest_areas.iter().any(|a| &a.definition == candidate))
        .expect("More candidate names than existing areas");
    let bounds = zone.bounds();
    let min = std::array::from_fn::<_, 3, _>(|i| {
        (zone.spawn[i] - 4.).clamp(bounds[0][i], bounds[1][i] - 8.)
    });
    let max = min.map(|v| v + 8.);
    let revision = session.editor.revision();
    session.edit(vec![Edit::PutQuestArea {
        area: theia_zone::quest_areas::QuestArea {
            id,
            definition,
            name: format!("Quest area {id}"),
            min,
            max,
            map_objectives: Vec::new(),
        },
    }]);
    if session.editor.revision() != revision {
        session.set_mode(EditorMode::Objects);
        session.quest_area_selected = Some(id);
        session.ui.properties_collapsed = false;
        session.status =
            "Quest area added at spawn · edit bounds and quest links in Attributes".into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_areas_contain_edge_spawns_choose_unique_ids_and_share_undo() {
        for offset in [-510., 0., 510.] {
            let origin = [128_000_000_000.; 3];
            let zone = Zone {
                origin,
                spawn: origin.map(|v| v + offset),
                ..default()
            };
            let mut session = Session::new(
                Editor::new(zone.clone()).unwrap(),
                PathBuf::new(),
                Some(zone.clone()),
            );
            add(&mut session);
            let first = session.editor.zone().quest_areas[0].clone();
            for i in 0..3 {
                assert!(first.min[i] <= zone.spawn[i] && first.max[i] >= zone.spawn[i]);
                assert_eq!(first.max[i] - first.min[i], 8.);
            }
            assert_eq!(session.quest_area_selected, Some(first.id));
            assert_eq!(session.saved_zone.as_ref(), Some(&zone));
            // User-chosen definition names need not match their authoring IDs.
            let mut changed = first;
            changed.definition = format!("theia.area.{}", session.next_id);
            session.edit(vec![Edit::PutQuestArea {
                area: changed.clone(),
            }]);
            add(&mut session);
            assert_eq!(session.editor.zone().quest_areas.len(), 2);
            assert_ne!(
                session.editor.zone().quest_areas[1].definition,
                changed.definition
            );
            session.execute(Command::Undo {
                expected_revision: session.editor.revision(),
            });
            assert_eq!(session.editor.zone().quest_areas, vec![changed]);
            assert_eq!(session.quest_area_selected, None);
        }
    }
}
