//! Selected authoring volume, using shared standard meshes and no physics/picking.
use super::*;

#[derive(Component)]
pub(super) struct Edge(usize);
#[derive(Resource)]
pub(super) struct Scene {
    signature: Option<(u64, u64, Option<u64>, bool)>,
}

pub(super) fn setup(
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
    for index in 0..12 {
        commands.spawn((
            Edge(index),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            Pickable::IGNORE,
            bevy::light::NotShadowCaster,
        ));
    }
    commands.insert_resource(Scene { signature: None });
}

fn edges(
    zone: &Zone,
    area: &theia_zone::quest_areas::QuestArea,
) -> Result<[Transform; 12], String> {
    // Subtract the checked origin before narrowing any coordinates.
    let min = Vec3::from_array(zone.local(area.min)?);
    let max = Vec3::from_array(zone.local(area.max)?);
    let size = max - min;
    let width = (size.max_element() * 0.002).clamp(0.04, 0.25);
    Ok(std::array::from_fn(|index| {
        let axis = index / 4;
        let side = index % 4;
        let mut centre = (min + max) * 0.5;
        let mut scale = Vec3::splat(width);
        scale[axis] = size[axis];
        for (bit, other) in (0..3).filter(|i| *i != axis).enumerate() {
            centre[other] = if side & (1 << bit) == 0 {
                min[other]
            } else {
                max[other]
            };
        }
        Transform::from_translation(centre).with_scale(scale)
    }))
}

pub(super) fn sync(
    session: Res<Session>,
    mut scene: ResMut<Scene>,
    mut preview: Query<(&Edge, &mut Transform, &mut Visibility)>,
) {
    let signature = (
        session.document_id,
        session.editor.revision(),
        session.quest_area_selected,
        session.play.is_some(),
    );
    if scene.signature == Some(signature) {
        return;
    }
    let zone = session.editor.zone();
    let transforms = if session.play.is_none() {
        zone.quest_areas
            .iter()
            .find(|a| Some(a.id) == session.quest_area_selected)
            .and_then(|area| edges(zone, area).ok())
    } else {
        None
    };
    for (edge, mut transform, mut visibility) in &mut preview {
        if let Some(transforms) = &transforms {
            *transform = transforms[edge.0];
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    scene.signature = Some(signature);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_volume_edges_preserve_bounds_without_expanding_collision_or_asset_counts() {
        let origin = [128_000_000_000.; 3];
        let area = theia_zone::quest_areas::QuestArea {
            id: 1,
            name: "Far bounds".into(),
            definition: "theia.area.bounds".into(),
            min: origin.map(|p| p + 0.125),
            max: origin.map(|p| p + 8.125),
            map_objectives: vec![],
        };
        let zone = Zone {
            schema: 9,
            origin,
            spawn: origin,
            quest_areas: vec![area],
            ..default()
        };
        let mut app = App::new();
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone.clone()),
        );
        session.quest_area_selected = Some(1);
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(session)
            .add_systems(Startup, setup)
            .add_systems(Update, sync);
        app.update();
        let transforms = edges(&zone, &zone.quest_areas[0]).unwrap();
        for (index, transform) in transforms.iter().enumerate() {
            let axis = index / 4;
            assert_eq!(transform.translation[axis], 4.125);
            assert_eq!(transform.scale[axis], 8.);
            for i in (0..3).filter(|i| *i != axis) {
                assert!([0.125, 8.125].contains(&transform.translation[i]));
            }
        }
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 1);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<avian3d::prelude::Collider>>()
                .iter(app.world())
                .count(),
            0
        );
        let mut preview = app
            .world_mut()
            .query::<(&Edge, &Transform, &Visibility, &Pickable)>();
        assert_eq!(preview.iter(app.world()).count(), 12);
        for (edge, transform, visibility, picking) in preview.iter(app.world()) {
            assert_eq!(*transform, transforms[edge.0]);
            assert_eq!(*visibility, Visibility::Visible);
            assert!(!picking.is_hoverable && !picking.should_block_lower);
        }
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::Translate {
                id: 1,
                offset: [0.25, 0., 0.],
            }]);
        app.update();
        assert_eq!(
            preview.iter(app.world()).next().unwrap().1.translation.x,
            4.375
        );
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::DeleteQuestArea { id: 1 }]);
        app.update();
        assert!(
            preview
                .iter(app.world())
                .all(|(_, _, visibility, _)| *visibility == Visibility::Hidden)
        );
        app.world_mut()
            .resource_mut::<Session>()
            .execute(Command::Undo {
                expected_revision: 2,
            });
        app.world_mut()
            .resource_mut::<Session>()
            .quest_area_selected = Some(1);
        app.update();
        assert!(
            preview
                .iter(app.world())
                .all(|(_, _, visibility, _)| *visibility == Visibility::Visible)
        );
        let play =
            playtest::Playtest::new(app.world().resource::<Session>().editor.zone()).unwrap();
        app.world_mut().resource_mut::<Session>().play = Some(play);
        app.update();
        assert!(
            preview
                .iter(app.world())
                .all(|(_, _, visibility, _)| *visibility == Visibility::Hidden)
        );
        app.world_mut().resource_mut::<Session>().play = None;
        app.update();
        assert!(
            preview
                .iter(app.world())
                .all(|(_, _, visibility, _)| *visibility == Visibility::Visible)
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 1);
    }
}
