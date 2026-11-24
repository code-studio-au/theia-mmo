//! Primitive and imported-model gizmos share gestures, validation and history.
use super::*;

pub(super) fn selected(session: &Session) -> Option<(u64, Pose)> {
    if let Some(pivot) = group_selection::pivot(session) {
        return Some((0, pivot));
    }
    let zone = session.editor.zone();
    if let Some(id) = session.test_player_selected {
        let player = zone.test_players.iter().find(|p| p.id == id)?;
        Some((
            id,
            Pose {
                position: player.position,
                yaw_degrees: 0.,
                size: [1.; 3],
            },
        ))
    } else if let Some(id) = session.scenery_selected {
        let placement = zone.scenery.iter().find(|p| p.id == id)?;
        Some((id, scenery_pose(placement)))
    } else {
        let object = zone
            .objects
            .iter()
            .find(|o| Some(o.id) == session.selected)?;
        Some((object.id, object.pose.clone()))
    }
}
pub(super) fn scenery_pose(p: &theia_zone::scenery::SceneryPlacement) -> Pose {
    Pose {
        position: p.position,
        yaw_degrees: p.yaw_degrees,
        size: [p.scale; 3],
    }
}
pub(super) fn scenery_preview(
    placement: &theia_zone::scenery::SceneryPlacement,
    pose: &Pose,
) -> theia_zone::scenery::SceneryPlacement {
    let mut result = placement.clone();
    result.position = pose.position;
    result.yaw_degrees = pose.yaw_degrees;
    result.scale = pose.size[0];
    result
}
pub(super) fn edit(
    zone: &Zone,
    id: u64,
    pose: &Pose,
    library: Option<&scenery_library::Library>,
) -> Result<Edit, String> {
    if let Some(placement) = zone.scenery.iter().find(|p| p.id == id) {
        if pose.size != [pose.size[0]; 3] {
            return Err("Scenery requires uniform scale".into());
        }
        let placement = scenery_preview(placement, pose);
        placement.validate(zone)?;
        let entry = library
            .and_then(|l| l.assets.get(&placement.asset))
            .ok_or("Load the scenery library before moving this model")?;
        scenery_scene::transform(zone, &placement, &entry.entry)?;
        Ok(Edit::PutScenery { scenery: placement })
    } else if let Some(original) = zone.plants.iter().find(|p| p.id == id) {
        let mut plant = original.clone();
        plant.position = pose.position;
        plant.yaw_degrees = pose.yaw_degrees;
        plant.scale = pose.size[0];
        Ok(Edit::PutPlant { plant })
    } else if let Some(original) = zone.markers.iter().find(|p| p.id == id) {
        let mut marker = original.clone();
        marker.position = pose.position;
        marker.yaw_degrees = pose.yaw_degrees;
        marker.radius_metres = pose.size[0];
        Ok(Edit::PutMarker { marker })
    } else if let Some(original) = zone.particles.iter().find(|p| p.id == id) {
        let mut particle = original.clone();
        particle.position = pose.position;
        particle.yaw_degrees = pose.yaw_degrees;
        particle.scale = pose.size[0];
        Ok(Edit::PutParticle { particle })
    } else if let Some(original) = zone.test_players.iter().find(|p| p.id == id) {
        if pose.yaw_degrees != 0. || pose.size != [1.; 3] {
            return Err("Test players currently support position only".into());
        }
        let mut player = original.clone();
        player.position = pose.position;
        player.validate(zone)?;
        Ok(Edit::PutTestPlayer { player })
    } else if zone.objects.iter().any(|o| o.id == id) {
        Ok(Edit::Transform {
            id,
            pose: pose.clone(),
        })
    } else {
        Err("Transform target no longer exists".into())
    }
}
/// Include off-centre source pivots when sizing handles around a model.
pub(super) fn extent(
    zone: &Zone,
    id: u64,
    pose: &Pose,
    library: Option<&scenery_library::Library>,
) -> f32 {
    zone.scenery
        .iter()
        .find(|p| p.id == id)
        .and_then(|p| library?.assets.get(&p.asset))
        .map(|asset| {
            asset
                .entry
                .bounds
                .iter()
                .flatten()
                .copied()
                .map(f64::abs)
                .fold(0., f64::max)
                * pose.size[0]
        })
        .unwrap_or_else(|| pose.size.iter().copied().fold(0., f64::max) * 0.5) as f32
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    pub(crate) fn fixture(origin: [f64; 3]) -> (Session, scenery_library::Library) {
        let placement = theia_zone::scenery::SceneryPlacement {
            id: 1,
            name: "Arch".into(),
            asset: "theia.test.arch".into(),
            position: [origin[0] + 2.125, origin[1], origin[2] - 3.25],
            yaw_degrees: 17.25,
            scale: 1.25,
        };
        let zone = Zone {
            origin,
            spawn: origin,
            schema: 10,
            scenery: vec![placement],
            ..default()
        };
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            "unused.zone.json".into(),
            Some(zone),
        );
        session.scenery_selected = Some(1);
        let entry = scenery_library::Entry {
            id: "theia.test.arch".into(),
            name: "Arch".into(),
            category: "Structures".into(),
            file: "unused.glb".into(),
            bytes: 1,
            sha256: "fixture".into(),
            bounds: [[-2., 0., -1.], [2., 6., 1.]],
            triangles: 1,
            source: serde_json::json!({}),
            collision: None,
        };
        let mut library = scenery_library::Library::default();
        library.assets.insert(
            entry.id.clone(),
            scenery_library::Asset {
                entry,
                bytes: std::sync::Arc::new(vec![]),
                collision: None,
            },
        );
        (session, library)
    }
    fn ray(point: Vec3) -> Ray3d {
        let eye = point + Vec3::new(7., 8., 10.);
        Ray3d::new(eye, Dir3::new(point - eye).unwrap())
    }
    #[test]
    fn model_xyz_plane_yaw_and_scale_use_shared_gestures_and_exact_history() {
        for origin in [[0.; 3], [128_000_000_000., 0., -128_000_000_000.]] {
            for (tool, axis) in [
                (gizmo::Tool::Move, 0),
                (gizmo::Tool::Move, 1),
                (gizmo::Tool::Move, 2),
                (gizmo::Tool::Rotate, 1),
                (gizmo::Tool::Scale, 1),
            ] {
                let (mut session, library) = fixture(origin);
                assert!(workspace::Control::GizmoTool(tool).enabled(&session));
                let original = session.editor.zone().clone();
                let (id, before) = selected(&session).unwrap();
                let anchor = Vec3::from_array(original.local(before.position).unwrap());
                let (start, end) = if tool == gizmo::Tool::Rotate {
                    (anchor + Vec3::X * 4., anchor - Vec3::Z * 4.)
                } else {
                    (anchor, anchor + Vec3::AXES[axis] * 2.)
                };
                let mut gesture =
                    gizmo::Gesture::begin(tool, ray(start), anchor, axis, 4.).unwrap();
                let preview = gesture.preview(ray(end), &before, false).unwrap();
                assert_ne!(preview, before);
                let command = edit(&original, id, &preview, Some(&library)).unwrap();
                assert_eq!(session.editor.zone(), &original);
                session.edit(vec![command]);
                let applied = session.editor.zone().clone();
                let model = &applied.scenery[0];
                assert_eq!(scenery_pose(model), preview);
                assert_eq!(model.name, original.scenery[0].name);
                assert_eq!(model.asset, original.scenery[0].asset);
                session.execute(Command::Undo {
                    expected_revision: session.editor.revision(),
                });
                assert_eq!(session.editor.zone(), &original);
                session.execute(Command::Redo {
                    expected_revision: session.editor.revision(),
                });
                assert_eq!(session.editor.zone(), &applied);
                assert_eq!(session.saved_zone.as_ref(), Some(&original));
                let mut plane = gizmo::Gesture::plane(ray(anchor), anchor).unwrap();
                let moved = plane
                    .preview(ray(anchor + Vec3::new(3., 0., 2.)), &before, false)
                    .unwrap();
                assert_eq!(moved.position[1], before.position[1]);
                assert!((moved.position[0] - before.position[0] - 3.).abs() < 0.0001);
            }
        }
    }
    #[test]
    fn model_bounds_scale_and_missing_assets_refuse_invalid_previews() {
        let (session, library) = fixture([0.; 3]);
        let (id, before) = selected(&session).unwrap();
        assert_eq!(
            extent(session.editor.zone(), id, &before, Some(&library)),
            7.5
        );
        assert!(edit(session.editor.zone(), id, &before, None).is_err());
        for size in [[-1.; 3], [0.; 3], [65.; 3], [1., 2., 1.], [f64::NAN; 3]] {
            let mut pose = before.clone();
            pose.size = size;
            assert!(edit(session.editor.zone(), id, &pose, Some(&library)).is_err());
        }
        let mut pose = before;
        pose.position[0] = theia_zone::ZONE_EXTENT - 0.01;
        assert!(
            edit(session.editor.zone(), id, &pose, Some(&library))
                .unwrap_err()
                .contains("bounds")
        );
    }
}
