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
    if scene.signature == Some(signature) && !group_selection::active(&session) {
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
    for original in &session.editor.zone().particles {
        let pose = group_selection::preview(
            &session,
            original.id,
            &group_selection::pose(original.position, original.yaw_degrees, original.scale),
        );
        let particle = &theia_zone::particles::ParticlePlacement {
            position: pose.position,
            yaw_degrees: pose.yaw_degrees,
            scale: pose.size[0],
            ..original.clone()
        };
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
    scene.signature = if group_selection::active(&session) {
        None
    } else {
        Some(signature)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_zone::particles::ParticlePlacement;

    #[test]
    fn delete_key_removes_selected_particle_when_no_text_field_owns_focus() {
        let (mut app, _) = crate::outliner::tests::app([0.; 3]);
        app.add_systems(Update, crate::keyboard);
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::PutParticle {
                particle: ParticlePlacement {
                    id: 999,
                    name: "New particle".into(),
                    effect: "vfx.unassigned".into(),
                    position: [0.; 3],
                    yaw_degrees: 0.,
                    scale: 1.,
                },
            }]);
        app.world_mut().resource_mut::<Session>().particle_selected = Some(999);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Delete);
        app.update();
        assert!(
            app.world()
                .resource::<Session>()
                .editor
                .zone()
                .particles
                .is_empty()
        );
    }

    #[test]
    fn add_at_spawn_then_delete_removes_only_the_selected_placement_and_undo_restores_it() {
        let zone = Zone::default();
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone),
        );
        let mut view = View {
            target: Vec3::ZERO,
            yaw: 0.,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        };
        perform(Action::AddParticle, &mut session, &mut view);
        let id = session.particle_selected.expect("new particle is selected");
        assert_eq!(session.editor.zone().particles.len(), 1);
        assert_eq!(
            session.editor.zone().particles[0].position,
            session.editor.zone().spawn
        );
        perform(Action::Delete, &mut session, &mut view);
        assert!(session.editor.zone().particles.is_empty());
        assert_eq!(session.particle_selected, None);
        perform(Action::Undo, &mut session, &mut view);
        assert_eq!(session.editor.zone().particles[0].id, id);
    }

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
