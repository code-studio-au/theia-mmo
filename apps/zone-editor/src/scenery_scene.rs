//! Static scenery instances use standard Bevy glTF world assets.
use super::*;
use std::collections::BTreeMap;

#[derive(Component)]
pub(super) struct SceneryRoot;
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
                    SceneryRoot,
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
