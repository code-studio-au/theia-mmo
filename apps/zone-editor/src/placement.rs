//! Read-only placement candidate and one standard Bevy mesh preview.
use super::*;

#[derive(Clone)]
pub(super) struct Preview {
    pub object: Object,
    pub error: Option<String>,
    revision: u64,
}
pub(super) fn pose(session: &Session, primitive: Primitive, hit: [f64; 3]) -> Result<Pose, String> {
    let point = session.snapped(hit);
    let mut position = session
        .editor
        .zone()
        .surface_position([point[0], point[2]])?;
    let size = session.placement_dimensions(primitive);
    position[1] += size[1] / 2.;
    Ok(Pose {
        position,
        yaw_degrees: 0.,
        size,
    })
}
pub(super) fn object(session: &Session, primitive: Primitive, pose: Pose) -> Object {
    let id = session.next_id;
    Object {
        id,
        name: format!("{primitive:?} {id}"),
        primitive,
        pose,
    }
}
pub(super) fn preview(
    session: &Session,
    primitive: Primitive,
    hit: [f64; 3],
    previous: Option<Preview>,
) -> Option<Preview> {
    let object = object(session, primitive, pose(session, primitive, hit).ok()?);
    let revision = session.editor.revision();
    if let Some(previous) = previous
        && previous.revision == revision
        && previous.object == object
    {
        return Some(previous);
    }
    let error = if object.id.checked_add(1).is_none() {
        Some("Object IDs exhausted".into())
    } else {
        Editor::new(session.editor.zone().clone())
            .and_then(|mut candidate| {
                candidate.execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::Place {
                        object: object.clone(),
                    }],
                })
            })
            .err()
    };
    Some(Preview {
        object,
        error,
        revision,
    })
}
#[derive(Component)]
pub(super) struct Ghost;
#[derive(Resource)]
pub(super) struct Materials([Handle<StandardMaterial>; 2]);
pub(super) fn setup(
    mut commands: Commands,
    palette: Res<viewport::Palette>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let handles = [
        Color::srgba(0.2, 0.8, 1., 0.35),
        Color::srgba(1., 0.15, 0.1, 0.45),
    ]
    .map(|base_color| {
        materials.add(StandardMaterial {
            base_color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })
    });
    commands.spawn((
        Ghost,
        Mesh3d(palette.meshes[0].clone()),
        MeshMaterial3d(handles[0].clone()),
        Transform::default(),
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
        bevy::picking::Pickable::IGNORE,
    ));
    commands.insert_resource(Materials(handles));
}
pub(super) fn sync(
    session: Res<Session>,
    palette: Res<viewport::Palette>,
    materials: Res<Materials>,
    mut ghosts: Query<
        (
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Transform,
            &mut Visibility,
        ),
        With<Ghost>,
    >,
) {
    let Ok((mut mesh, mut material, mut transform, mut visibility)) = ghosts.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    let Some(preview) = &session.placement_preview else {
        return;
    };
    let Ok(position) = session.editor.zone().local(preview.object.pose.position) else {
        return;
    };
    let size = preview.object.pose.size.map(|value| value as f32);
    if !size.iter().all(|value| value.is_finite() && *value > 0.) {
        return;
    }
    mesh.0 = palette.meshes[viewport::index(preview.object.primitive)].clone();
    material.0 = materials.0[usize::from(preview.error.is_some())].clone();
    *transform =
        Transform::from_translation(Vec3::from_array(position)).with_scale(Vec3::from_array(size));
    *visibility = Visibility::Visible;
}

#[cfg(test)]
#[path = "placement_preview_tests.rs"]
mod tests;
