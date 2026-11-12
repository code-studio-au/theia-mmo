//! Editor-only selection and framing, independent of document history.
use super::*;

pub(super) const MAX_VIEW_DISTANCE: f32 = 1536.;

pub(super) fn keyboard(
    keys: &ButtonInput<KeyCode>,
    session: &mut Session,
    view: &mut View,
) -> bool {
    if session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
        || session.path_mode
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
    {
        return false;
    }
    if keys.just_pressed(KeyCode::Tab)
        && !session.marker_mode
        && !session.vegetation_mode
        && !session.terrain_mode
    {
        session.inspect_spawn = false;
        session.particle_selected = None;
        session.quest_area_selected = None;
        session.scenery_selected = None;
        let objects = &session.editor.zone().objects;
        let reverse = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        session.selected = if objects.is_empty() {
            None
        } else {
            let current = objects.iter().position(|o| Some(o.id) == session.selected);
            let index = match (current, reverse) {
                (Some(i), false) => (i + 1) % objects.len(),
                (Some(i), true) => (i + objects.len() - 1) % objects.len(),
                (None, false) => 0,
                (None, true) => objects.len() - 1,
            };
            Some(objects[index].id)
        };
        session.palette = None;
        session.status = session
            .selected
            .map_or("No objects to select".into(), |id| {
                format!("Selected object #{id} - F focuses, H grounds")
            });
        return true;
    }
    if !keys.just_pressed(KeyCode::KeyF) {
        return false;
    }
    frame(session, view)
}

pub(super) fn frame(session: &mut Session, view: &mut View) -> bool {
    let zone = session.editor.zone();
    let framing = if session.terrain_mode {
        zone.terrain.as_ref().and_then(|patch| {
            let start = patch.origin().ok()?;
            let minimum = patch.heights.iter().copied().fold(f32::INFINITY, f32::min);
            let maximum = patch
                .heights
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            let half = patch.extent_metres() / 2.;
            let centre = [
                start[0] + half,
                zone.origin[1] + f64::from((minimum + maximum) / 2.),
                start[2] + half,
            ];
            let radius = Vec3::new(half as f32, (maximum - minimum) / 2., half as f32).length();
            Some((centre, radius))
        })
    } else if session.vegetation_mode {
        zone.plants
            .iter()
            .find(|p| Some(p.id) == session.plant_selected)
            .map(|p| {
                let mut centre = p.position;
                // Conservative bounds for the retained fixed greybox proxies.
                let (height, radius) = if p.kind == theia_zone::vegetation::PlantKind::Tree {
                    (2.5, 3.2)
                } else {
                    (0.6, 1.1)
                };
                centre[1] += height * p.scale;
                (centre, (radius * p.scale) as f32)
            })
    } else if session.marker_mode {
        zone.markers
            .iter()
            .find(|m| Some(m.id) == session.marker_selected)
            .map(|m| (m.position, m.radius_metres.max(2.) as f32))
    } else if let Some(id) = session.quest_area_selected {
        zone.quest_areas.iter().find(|a| a.id == id).map(|a| {
            let centre = std::array::from_fn(|i| a.min[i] + (a.max[i] - a.min[i]) / 2.);
            let radius =
                Vec3::from_array(std::array::from_fn(|i| ((a.max[i] - a.min[i]) / 2.) as f32))
                    .length();
            (centre, radius)
        })
    } else if let Some(id) = session.scenery_selected {
        zone.scenery
            .iter()
            .find(|p| p.id == id)
            .map(|p| (p.position, (p.scale * 4.) as f32))
    } else if let Some(id) = session.particle_selected {
        zone.particles
            .iter()
            .find(|p| p.id == id)
            .map(|p| (p.position, 2.))
    } else {
        zone.objects
            .iter()
            .find(|o| Some(o.id) == session.selected)
            .map(|o| {
                (
                    o.pose.position,
                    Vec3::from_array(o.pose.size.map(|v| v as f32)).length() / 2.,
                )
            })
    };
    if let Some((centre, radius)) = framing {
        match zone.local(centre) {
            Ok(local) => {
                view.target = Vec3::from_array(local);
                // A factor of three encloses a sphere at the standard 45 degree
                // vertical field of view, with margin for the editor UI.
                view.distance = (radius * 3. + 8.).clamp(8., MAX_VIEW_DISTANCE);
                session.status = "Focused selection".into();
            }
            Err(error) => session.status = error,
        }
    } else {
        session.status = if session.terrain_mode {
            "Create terrain before focusing"
        } else {
            "Select a placement before focusing"
        }
        .into();
    }
    true
}

pub(super) fn duplicate(session: &mut Session) {
    if session.path_draft.is_some() {
        session.status = "Finish or cancel the guide draft before duplicating".into();
        return;
    }
    let selected = if session.path_mode {
        session.path_selected
    } else if session.marker_mode {
        session.marker_selected
    } else if session.vegetation_mode {
        session.plant_selected
    } else if !session.terrain_mode {
        session
            .selected
            .or(session.particle_selected)
            .or(session.scenery_selected)
    } else {
        None
    };
    let Some(id) = selected else {
        session.status = "Select a placement or guide before duplicating".into();
        return;
    };
    let new_id = session.next_id;
    let Some(next) = new_id.checked_add(1) else {
        session.status = "Authoring IDs exhausted".into();
        return;
    };
    session.next_id = next;
    let step = if session.snap { 1. } else { 0.1 };
    let revision = session.editor.revision();
    session.edit(vec![Edit::Duplicate {
        id,
        new_id,
        offset: [step, 0., step],
    }]);
    if session.editor.revision() == revision {
        return;
    }
    if session.path_mode {
        session.path_selected = Some(new_id);
        session.path_point = None;
    } else if session.marker_mode {
        session.marker_selected = Some(new_id);
        session.marker_palette = None;
    } else if session.vegetation_mode {
        session.plant_selected = Some(new_id);
        session.plant_palette = None;
    } else if session.scenery_selected.is_some() {
        session.scenery_selected = Some(new_id);
    } else if session.particle_selected.is_some() {
        session.particle_selected = Some(new_id);
    } else {
        session.selected = Some(new_id);
        session.palette = None;
    }
}
