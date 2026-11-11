//! Editor-only badges for unresolved particle references, not particle playback.
use std::collections::BTreeMap;

use super::*;

#[derive(Component)]
pub(super) struct ParticleAnchor(pub u64);

#[derive(Resource)]
pub(super) struct Scene {
    mesh: Handle<Mesh>,
    normal: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
    entities: BTreeMap<u64, Entity>,
    signature: Option<(u64, u64, Option<u64>, bool)>,
}

pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Scene {
        mesh: meshes.add(Sphere::new(0.4)),
        normal: materials.add(StandardMaterial {
            base_color: Color::srgb(0.72, 0.25, 0.95),
            unlit: true,
            ..default()
        }),
        selected: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.65, 0.1),
            unlit: true,
            ..default()
        }),
        entities: BTreeMap::new(),
        signature: None,
    });
}

pub(super) fn sync(mut commands: Commands, session: Res<Session>, mut scene: ResMut<Scene>) {
    let signature = (
        session.document_id,
        session.editor.revision(),
        session.particle_selected,
        session.play.is_some(),
    );
    if scene.signature == Some(signature) {
        return;
    }
    let removed: Vec<_> = scene
        .entities
        .keys()
        .filter(|id| !session.editor.zone().particles.iter().any(|p| p.id == **id))
        .copied()
        .collect();
    for id in removed {
        if let Some(entity) = scene.entities.remove(&id) {
            commands.entity(entity).despawn();
        }
    }
    for particle in &session.editor.zone().particles {
        let position = Vec3::from_array(
            session
                .editor
                .zone()
                .local(particle.position)
                .expect("validated particle position"),
        );
        let material = if session.particle_selected == Some(particle.id) {
            scene.selected.clone()
        } else {
            scene.normal.clone()
        };
        let visibility = if session.play.is_some() {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if let Some(entity) = scene.entities.get(&particle.id) {
            commands.entity(*entity).insert((
                Transform::from_translation(position),
                MeshMaterial3d(material),
                visibility,
            ));
        } else {
            let entity = commands
                .spawn((
                    ParticleAnchor(particle.id),
                    Mesh3d(scene.mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(position),
                    visibility,
                    bevy::light::NotShadowCaster,
                ))
                .id();
            scene.entities.insert(particle.id, entity);
        }
    }
    scene.signature = Some(signature);
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_zone::particles::ParticlePlacement;

    #[test]
    fn anchor_tracks_exact_local_position_selection_deletion_and_undo() {
        let origin = [128_000_000_000., 4., -128_000_000_000.];
        let particle = ParticlePlacement {
            id: 7,
            name: "Fire".into(),
            effect: "vfx.fire".into(),
            position: [origin[0] + 0.125, origin[1], origin[2] + 1.],
            yaw_degrees: 0.,
            scale: 1.,
        };
        let zone = Zone {
            schema: 8,
            origin,
            spawn: origin,
            particles: vec![particle],
            ..Zone::default()
        };
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(Session::new(
                Editor::new(zone.clone()).unwrap(),
                PathBuf::new(),
                Some(zone),
            ))
            .add_systems(Startup, setup)
            .add_systems(Update, sync);
        app.update();
        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<ParticleAnchor>>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            app.world().get::<Transform>(entity).unwrap().translation,
            Vec3::new(0.125, 0., 1.)
        );
        let normal = app.world().resource::<Scene>().normal.clone();
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .unwrap()
                .0,
            normal
        );
        app.world_mut().resource_mut::<Session>().particle_selected = Some(7);
        app.update();
        let selected = app.world().resource::<Scene>().selected.clone();
        assert_eq!(
            app.world()
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .unwrap()
                .0,
            selected
        );
        let play =
            playtest::Playtest::new(app.world().resource::<Session>().editor.zone()).unwrap();
        app.world_mut().resource_mut::<Session>().play = Some(play);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Hidden
        );
        app.world_mut().resource_mut::<Session>().play = None;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Visible
        );
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::Translate {
                id: 7,
                offset: [0.25, 0., 0.],
            }]);
        app.update();
        assert_eq!(
            app.world().get::<Transform>(entity).unwrap().translation,
            Vec3::new(0.375, 0., 1.)
        );
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::DeleteParticle { id: 7 }]);
        app.update();
        assert!(app.world().get_entity(entity).is_err());
        let revision = app.world().resource::<Session>().editor.revision();
        app.world_mut()
            .resource_mut::<Session>()
            .execute(Command::Undo {
                expected_revision: revision,
            });
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ParticleAnchor>()
                .iter(app.world())
                .count(),
            1
        );
    }
}
