//! Static scenery instances use standard Bevy glTF world assets.
use super::*;
use std::collections::BTreeMap;

#[derive(Component)]
pub(super) struct SceneryRoot(pub u64);
#[derive(Resource, Default)]
pub(super) struct Scene {
    roots: BTreeMap<u64, (Entity, String)>,
    signature: Option<(u64, u64)>,
}
pub(super) fn transform(
    zone: &Zone,
    placement: &theia_zone::scenery::SceneryPlacement,
    entry: &scenery_library::Entry,
) -> Result<Transform, String> {
    let (sin, cos) = placement.yaw_degrees.to_radians().sin_cos();
    for corner in 0..8 {
        let p: [f64; 3] =
            std::array::from_fn(|i| entry.bounds[(corner >> i) & 1][i] * placement.scale);
        let offset = [cos * p[0] + sin * p[2], p[1], -sin * p[0] + cos * p[2]];
        if (0..3).any(|i| {
            (placement.position[i] - zone.origin[i] + offset[i]).abs() > theia_zone::ZONE_EXTENT
        }) {
            return Err("Scenery bounds leave the authoring zone".into());
        }
    }
    Ok(
        Transform::from_translation(Vec3::from_array(zone.local(placement.position)?))
            .with_rotation(Quat::from_rotation_y(
                placement.yaw_degrees.to_radians() as f32
            ))
            .with_scale(Vec3::splat(placement.scale as f32)),
    )
}

pub(super) fn add_requested(
    mut requests: ResMut<workspace::Requests>,
    mut session: ResMut<Session>,
    library: Res<scenery_library::Library>,
    mut tabs: Option<ResMut<property_tabs::State>>,
) {
    if let Some(request) = session.scenery_ground_requested.take()
        && let Err(error) = ground(&mut session, &library, request)
    {
        session.status = error;
    }
    let Some(index) = requests.add_scenery.take() else {
        return;
    };
    if workspace::busy(&session) {
        return;
    }
    let Some(asset) = library.assets.values().nth(index) else {
        session.status = "Scenery library changed; select a model again".into();
        return;
    };
    add(&mut session, &asset.entry);
    if session.scenery_selected.is_some()
        && let Some(tabs) = tabs.as_mut()
    {
        tabs.active = property_tabs::Tab::Attributes;
    }
}
/// Ground the model's lowest local bound at the terrain sample under its anchor.
/// This is an authoring convenience, not a collision/contact qualification.
fn grounded(
    zone: &Zone,
    placement: &theia_zone::scenery::SceneryPlacement,
    entry: &scenery_library::Entry,
) -> Result<theia_zone::scenery::SceneryPlacement, String> {
    let mut result = placement.clone();
    let surface = zone.surface_position([placement.position[0], placement.position[2]])?;
    result.position[1] = surface[1] - entry.bounds[0][1] * placement.scale;
    result.validate(zone)?;
    transform(zone, &result, entry)?;
    Ok(result)
}
fn ground(
    session: &mut Session,
    library: &scenery_library::Library,
    (document, revision, id): (u64, u64, u64),
) -> Result<(), String> {
    if workspace::busy(session)
        || session.document_id != document
        || session.editor.revision() != revision
        || session.scenery_selected != Some(id)
    {
        return Err("Selection changed; ground the current scenery again".into());
    }
    let zone = session.editor.zone();
    let placement = zone
        .scenery
        .iter()
        .find(|p| p.id == id)
        .ok_or("Selected scenery no longer exists")?;
    let asset = library
        .assets
        .get(&placement.asset)
        .ok_or("Load the scenery library before grounding this model")?;
    let grounded = grounded(zone, placement, &asset.entry)?;
    if &grounded != placement {
        session.edit(vec![Edit::PutScenery { scenery: grounded }]);
    }
    Ok(())
}

fn add(session: &mut Session, entry: &scenery_library::Entry) {
    let id = session.next_id;
    if id == u64::MAX {
        session.status = "Authoring IDs exhausted".into();
        return;
    }
    let zone = session.editor.zone();
    let placement = theia_zone::scenery::SceneryPlacement {
        id,
        name: entry.name.clone(),
        asset: entry.id.clone(),
        position: zone.spawn,
        yaw_degrees: 0.,
        scale: 1.,
    };
    let placement = match grounded(zone, &placement, entry) {
        Ok(placement) => placement,
        Err(error) => {
            session.status = error;
            return;
        }
    };
    let revision = session.editor.revision();
    session.edit(vec![Edit::PutScenery { scenery: placement }]);
    if session.editor.revision() != revision {
        session.set_mode(EditorMode::Objects);
        session.scenery_selected = Some(id);
        session.ui.properties_collapsed = false;
        session.status =
            "Scenery added at spawn · edit position, rotation and scale in Attributes".into();
    }
}

pub(super) fn sync(
    mut commands: Commands,
    mut session: ResMut<Session>,
    library: Res<scenery_library::Library>,
    server: Res<AssetServer>,
    mut scene: ResMut<Scene>,
) {
    let signature = (session.document_id, session.editor.revision());
    if scene.signature == Some(signature) {
        return;
    }
    let mut old = std::mem::take(&mut scene.roots);
    let mut errors = 0;
    for placement in &session.editor.zone().scenery {
        let Some(asset) = library.assets.get(&placement.asset) else {
            errors += 1;
            continue;
        };
        let Ok(transform) = transform(session.editor.zone(), placement, &asset.entry) else {
            errors += 1;
            continue;
        };
        let asset_key = asset.entry.sha256.clone();
        let existing = old.remove(&placement.id).and_then(|(entity, key)| {
            if key == asset_key {
                Some(entity)
            } else {
                commands.entity(entity).despawn();
                None
            }
        });
        let root = if let Some(entity) = existing {
            commands.entity(entity).insert(transform);
            entity
        } else {
            commands
                .spawn((
                    SceneryRoot(placement.id),
                    WorldAssetRoot(server.load(format!("scenery://{}#Scene0", asset.entry.file))),
                    transform,
                    Visibility::Visible,
                ))
                .id()
        };
        scene.roots.insert(placement.id, (root, asset_key));
    }
    for (entity, _) in old.into_values() {
        commands.entity(entity).despawn();
    }
    if errors > 0 {
        session.status =
            format!("{errors} scenery placements unavailable · check library IDs and model bounds");
    }
    scene.signature = Some(signature);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_duplicate_delete_scenery_share_history_and_preserve_saved_document() {
        let zone = Zone::default();
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone.clone()),
        );
        let entry = scenery_library::Entry {
            id: "synty.forest.archway".into(),
            name: "Archway".into(),
            category: "Structures".into(),
            file: String::new(),
            bytes: 1,
            sha256: String::new(),
            bounds: [[-1., 0., -1.], [1., 3., 1.]],
            triangles: 1,
            source: serde_json::json!({}),
        };
        add(&mut session, &entry);
        let first = session.scenery_selected.unwrap();
        assert_eq!(
            outliner::selection(&session),
            Some(outliner::Target::Scenery(first))
        );
        let first_zone = session.editor.zone().clone();
        selection::duplicate(&mut session);
        let second = session.scenery_selected.unwrap();
        assert_ne!(first, second);
        assert_eq!(session.editor.zone().scenery.len(), 2);
        let mut view = View {
            target: Vec3::ZERO,
            yaw: 0.,
            pitch: -0.5,
            distance: 20.,
            preset: default(),
            show_grid: true,
        };
        perform(Action::Delete, &mut session, &mut view);
        assert_eq!(session.editor.zone(), &first_zone);
        assert_eq!(session.scenery_selected, None);
        assert_eq!(session.saved_zone.as_ref(), Some(&zone));
        for _ in 0..3 {
            session.execute(Command::Undo {
                expected_revision: session.editor.revision(),
            });
        }
        assert_eq!(session.editor.zone(), &zone);
        assert!(!session.is_dirty());
    }
    #[test]
    fn scenery_grounding_uses_scaled_bottom_and_refuses_stale_or_missing_assets() {
        let origin = [128_000_000_000., 12., -128_000_000_000.];
        let zone = Zone {
            origin,
            spawn: origin,
            ..default()
        };
        let mut session = Session::new(Editor::new(zone).unwrap(), PathBuf::new(), None);
        let entry = scenery_library::Entry {
            id: "theia.asset.hanging".into(),
            name: "Hanging model".into(),
            category: "Props".into(),
            file: String::new(),
            bytes: 1,
            sha256: String::new(),
            bounds: [[-0.1, -0.25, -0.1], [0.1, 0., 0.1]],
            triangles: 1,
            source: serde_json::json!({}),
        };
        add(&mut session, &entry);
        let original = session.editor.zone().scenery[0].clone();
        assert_eq!(original.position[1], 12.25);
        let mut changed = original.clone();
        changed.scale = 4.;
        changed.position[1] = 20.;
        changed.yaw_degrees = 90.;
        session.edit(vec![Edit::PutScenery {
            scenery: changed.clone(),
        }]);
        let request = (session.document_id, session.editor.revision(), original.id);
        let mut library = scenery_library::Library::default();
        assert!(ground(&mut session, &library, request).is_err());
        library.assets.insert(
            entry.id.clone(),
            scenery_library::Asset {
                entry,
                bytes: std::sync::Arc::new(vec![]),
            },
        );
        assert!(
            ground(
                &mut session,
                &library,
                (request.0 + 1, request.1, request.2)
            )
            .is_err()
        );
        assert_eq!(session.editor.zone().scenery[0], changed);
        ground(&mut session, &library, request).unwrap();
        assert_eq!(
            session.editor.zone().scenery[0].position,
            [origin[0], 13., origin[2]]
        );
        assert_eq!(session.scenery_selected, Some(original.id));
        session.execute(Command::Undo {
            expected_revision: session.editor.revision(),
        });
        assert_eq!(session.editor.zone().scenery[0], changed);
        assert!(ground(&mut session, &library, request).is_err());
    }

    #[test]
    fn far_model_transforms_preserve_anchor_and_reject_rotated_geometry_outside_zone() {
        let origin = [128_000_000_000., -4., -128_000_000_000.];
        let zone = Zone {
            origin,
            spawn: origin,
            ..Zone::default()
        };
        let mut placement = theia_zone::scenery::SceneryPlacement {
            id: 1,
            name: "Arch".into(),
            asset: "theia.arch".into(),
            position: [origin[0] + 0.125, origin[1], origin[2] - 0.25],
            yaw_degrees: 90.,
            scale: 2.,
        };
        let entry = scenery_library::Entry {
            id: "theia.arch".into(),
            name: "Arch".into(),
            category: "Structures".into(),
            file: String::new(),
            bytes: 1,
            sha256: String::new(),
            bounds: [[-1., 0., -4.], [1., 3., 4.]],
            triangles: 1,
            source: serde_json::json!({}),
        };
        let value = transform(&zone, &placement, &entry).unwrap();
        assert_eq!(value.translation, Vec3::new(0.125, 0., -0.25));
        assert_eq!(value.scale, Vec3::splat(2.));
        assert!((value.rotation * Vec3::Z - Vec3::X).length() < 1e-6);
        placement.position[0] = origin[0] + 508.;
        assert!(transform(&zone, &placement, &entry).is_err());
        placement.yaw_degrees = 0.;
        assert!(transform(&zone, &placement, &entry).is_ok());
    }
}
