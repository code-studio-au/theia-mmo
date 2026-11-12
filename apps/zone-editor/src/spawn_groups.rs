//! Population draft controls and selected polygon preview; no runtime spawning.
use super::*;
use theia_zone::spawn_groups::SpawnGroup;

pub(super) fn bounds(group: &SpawnGroup) -> ([f64; 3], [f64; 3]) {
    let points = group.volume.polygon();
    let min = [
        points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
        group.volume.min_y(),
        points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
    ];
    let max = [
        points
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max),
        group.volume.max_y(),
        points
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max),
    ];
    (min, max)
}
pub(super) fn centre(group: &SpawnGroup) -> [f64; 3] {
    let (min, max) = bounds(group);
    std::array::from_fn(|i| min[i] + (max[i] - min[i]) * 0.5)
}
pub(super) fn add(session: &mut Session) {
    let id = session.next_id;
    if id == u64::MAX {
        session.status = "Authoring IDs exhausted".into();
        return;
    }
    let zone = session.editor.zone();
    let lower = std::array::from_fn::<_, 3, _>(|i| {
        zone.origin[i]
            + (zone.spawn[i] - zone.origin[i] - 8.)
                .clamp(-theia_zone::ZONE_EXTENT, theia_zone::ZONE_EXTENT - 16.)
    });
    let volume = match theia_core::spawn_volume::PolygonVolume::new(
        vec![
            [lower[0], lower[2]],
            [lower[0] + 16., lower[2]],
            [lower[0] + 16., lower[2] + 16.],
            [lower[0], lower[2] + 16.],
        ],
        lower[1],
        lower[1] + 16.,
    ) {
        Ok(volume) => volume,
        Err(error) => {
            session.status = error;
            return;
        }
    };
    let revision = session.editor.revision();
    session.edit(vec![Edit::PutSpawnGroup {
        group: SpawnGroup {
            id,
            name: format!("NPC spawn zone {id}"),
            enabled: true,
            creature: "theia.dev.training-creature".into(),
            model: None,
            volume,
            target_alive: 3,
            respawn_seconds: 30,
            seed: 42,
        },
    }]);
    if session.editor.revision() != revision {
        session.set_mode(EditorMode::Objects);
        session.spawn_group_selected = Some(id);
        session.ui.properties_collapsed = false;
        session.status =
            "NPC spawn draft added · configure in Attributes; live population is not connected"
                .into();
    }
}
pub(super) fn toggle(session: &mut Session) {
    if let Some(original) = session
        .editor
        .zone()
        .spawn_groups
        .iter()
        .find(|g| Some(g.id) == session.spawn_group_selected)
    {
        let mut group = original.clone();
        group.enabled = !group.enabled;
        session.edit(vec![Edit::PutSpawnGroup { group }]);
    }
}
#[derive(Component)]
struct Edge(usize);
#[derive(Resource, Default)]
struct Scene {
    signature: Option<(u64, u64, Option<u64>, bool)>,
}
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_zone_systems(sync);
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1., 1., 1.));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1., 0.65, 0.1),
        unlit: true,
        ..default()
    });
    for i in 0..theia_core::spawn_volume::MAX_VERTICES * 3 {
        commands.spawn((
            Edge(i),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            Pickable::IGNORE,
            bevy::light::NotShadowCaster,
        ));
    }
    commands.init_resource::<Scene>();
}
fn edges(zone: &Zone, group: &SpawnGroup) -> Result<Vec<Transform>, String> {
    let polygon = group.volume.polygon();
    let low = polygon
        .iter()
        .map(|p| {
            zone.local([p[0], group.volume.min_y(), p[1]])
                .map(Vec3::from_array)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let high = polygon
        .iter()
        .map(|p| {
            zone.local([p[0], group.volume.max_y(), p[1]])
                .map(Vec3::from_array)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (min, max) = bounds(group);
    let width =
        ((0..3).map(|i| max[i] - min[i]).fold(0., f64::max) * 0.002).clamp(0.04, 0.25) as f32;
    let mut transforms = Vec::new();
    for i in 0..polygon.len() {
        let j = (i + 1) % polygon.len();
        for (a, b) in [(low[i], low[j]), (high[i], high[j]), (low[i], high[i])] {
            let delta = b - a;
            transforms.push(
                Transform::from_translation((a + b) * 0.5)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, delta.normalize()))
                    .with_scale(Vec3::new(width, delta.length(), width)),
            );
        }
    }
    Ok(transforms)
}
fn sync(
    session: Res<Session>,
    mut scene: ResMut<Scene>,
    mut preview: Query<(&Edge, &mut Transform, &mut Visibility)>,
) {
    let signature = (
        session.document_id,
        session.editor.revision(),
        session.spawn_group_selected,
        session.play.is_some(),
    );
    if scene.signature == Some(signature) {
        return;
    }
    let zone = session.editor.zone();
    let transforms = if session.play.is_none() {
        zone.spawn_groups
            .iter()
            .find(|g| Some(g.id) == session.spawn_group_selected)
            .and_then(|g| edges(zone, g).ok())
    } else {
        None
    };
    for (edge, mut transform, mut visibility) in &mut preview {
        if let Some(value) = transforms.as_ref().and_then(|v| v.get(edge.0)) {
            *transform = *value;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    scene.signature = Some(signature);
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Report<'w, 's> {
    edges: Query<'w, 's, &'static Visibility, With<Edge>>,
}
impl Report<'_, '_> {
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"visible_edges": self.edges.iter().filter(|v| **v == Visibility::Visible).count(), "pool_size": self.edges.iter().count()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn session(origin: [f64; 3], offset: f64) -> Session {
        let zone = Zone {
            origin,
            spawn: origin.map(|v| v + offset),
            ..default()
        };
        Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone),
        )
    }
    #[test]
    fn add_toggle_copy_delete_and_undo_preserve_original_document_and_edge_spawns() {
        for offset in [-510., 0., 510.] {
            let mut session = session([128_000_000_000.; 3], offset);
            let source = session.editor.zone().clone();
            add(&mut session);
            let original = session.editor.zone().spawn_groups[0].clone();
            assert!(
                original
                    .volume
                    .contains(theia_core::coordinates::GlobalPosition::new(source.spawn).unwrap())
            );
            assert_eq!(session.spawn_group_selected, Some(original.id));
            toggle(&mut session);
            assert!(!session.editor.zone().spawn_groups[0].enabled);
            if offset == 510. {
                let before = session.editor.zone().clone();
                selection::duplicate(&mut session);
                assert_eq!(session.editor.zone(), &before);
                session.edit(vec![Edit::Translate {
                    id: original.id,
                    offset: [-1., 0., -1.],
                }]);
            }
            selection::duplicate(&mut session);
            assert_eq!(session.editor.zone().spawn_groups.len(), 2);
            let copy = session.spawn_group_selected.unwrap();
            assert_ne!(copy, original.id);
            session.edit(vec![Edit::DeleteSpawnGroup { id: copy }]);
            assert_eq!(session.spawn_group_selected, None);
            let revision = session.editor.revision();
            session.execute(Command::Undo {
                expected_revision: revision,
            });
            assert_eq!(session.editor.zone().spawn_groups.len(), 2);
            assert_eq!(session.saved_zone.as_ref(), Some(&source));
        }
        let mut edge = session([theia_core::coordinates::MAX_GLOBAL_METRES; 3], 0.);
        add(&mut edge);
        assert!(edge.editor.zone().spawn_groups.is_empty());
    }
    #[test]
    fn polygon_preview_uses_bounded_shared_assets_no_collision_and_hides_with_selection() {
        let mut session = session([128_000_000_000.125; 3], 0.);
        add(&mut session);
        let original = session.editor.zone().clone();
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(session)
            .add_plugins(plugin);
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 1);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<avian3d::prelude::Collider>>()
                .iter(app.world())
                .count(),
            0
        );
        let mut query = app.world_mut().query::<(&Edge, &Transform, &Visibility)>();
        assert_eq!(
            query
                .iter(app.world())
                .filter(|(_, _, v)| **v == Visibility::Visible)
                .count(),
            12
        );
        for (_, t, v) in query.iter(app.world()) {
            if *v == Visibility::Visible {
                assert!(t.translation.abs().max_element() <= 8.);
                assert_eq!(t.scale.y, 16.);
            }
        }
        app.world_mut()
            .resource_mut::<Session>()
            .spawn_group_selected = None;
        app.update();
        assert!(
            query
                .iter(app.world())
                .all(|(_, _, v)| *v == Visibility::Hidden)
        );
        assert_eq!(app.world().resource::<Session>().editor.zone(), &original);
    }
}
