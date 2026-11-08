//! Editor-only selection and framing, independent of document history.
use super::*;

pub(super) const MAX_VIEW_DISTANCE: f32 = 512.;

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
        || session.terrain_mode
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
    {
        return false;
    }
    if keys.just_pressed(KeyCode::Tab) && !session.marker_mode && !session.vegetation_mode {
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
    let zone = session.editor.zone();
    let framing = if session.vegetation_mode {
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
        session.status = "Select a placement before focusing".into();
    }
    true
}
