//! Explicit, temporary preview of the currently open Sprinkles project at an anchor.
//! This does not resolve the authored effect ID or add assets to a world package.
use bevy_sprinkles::{prelude::*, runtime::EditorMode};
use bevy_sprinkles_editor::{hosting::AuthoringHost, state::EditorState};

use super::*;

#[derive(Component)]
pub(super) struct ZoneParticlePreview;

struct Preview {
    document_id: u64,
    placement_id: u64,
    project: Handle<ParticlesAsset>,
}

#[derive(Resource, Default)]
pub(super) struct State {
    active: Option<Preview>,
    entity: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync(
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut requests: ResMut<workspace::Requests>,
    editor: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    host: Option<Res<AuthoringHost>>,
    mut state: ResMut<State>,
    mut previews: Query<&mut Transform, With<ZoneParticlePreview>>,
) {
    let host_active = host.as_ref().is_some_and(|host| host.active);
    if std::mem::take(&mut requests.particle_preview)
        && !host_active
        && !workspace::busy(&session)
        && let Some(id) = session.particle_selected
    {
        if state.active.as_ref().is_some_and(|preview| {
            preview.placement_id == id && preview.document_id == session.document_id
        }) {
            state.active = None;
            session.status = "Stopped open-project particle preview".into();
        } else if let Some(project) = editor
            .current_project
            .as_ref()
            .filter(|handle| assets.get(*handle).is_some())
        {
            state.active = Some(Preview {
                document_id: session.document_id,
                placement_id: id,
                project: project.clone(),
            });
            session.status =
                "Previewing open particle project · Effect ID is not linked or packaged".into();
        } else {
            session.status = "Open a particle project before previewing it".into();
        }
        if let Some(entity) = state.entity.take() {
            commands.entity(entity).despawn();
        }
    }
    let valid = state.active.as_ref().is_some_and(|preview| {
        preview.document_id == session.document_id
            && editor.current_project.as_ref() == Some(&preview.project)
            && assets.get(&preview.project).is_some()
            && session
                .editor
                .zone()
                .particles
                .iter()
                .any(|p| p.id == preview.placement_id)
    });
    if !valid {
        if state.active.is_some() {
            session.status =
                "Open-project particle preview stopped because its project or placement changed"
                    .into();
        }
        state.active = None;
    }
    if !valid || host_active || session.play.is_some() {
        if let Some(entity) = state.entity.take() {
            commands.entity(entity).despawn();
        }
        return;
    }
    let preview = state.active.as_ref().expect("validated preview");
    let project_handle = preview.project.clone();
    let placement = session
        .editor
        .zone()
        .particles
        .iter()
        .find(|p| p.id == preview.placement_id)
        .expect("validated placement");
    let project = assets.get(&preview.project).expect("validated project");
    let position = Vec3::from_array(
        session
            .editor
            .zone()
            .local(placement.position)
            .expect("validated position"),
    );
    let anchor = Transform::from_translation(position)
        .with_rotation(Quat::from_rotation_y(
            placement.yaw_degrees.to_radians() as f32
        ))
        .with_scale(Vec3::splat(placement.scale as f32));
    let transform = anchor.mul_transform(project.initial_transform.to_transform());
    if let Some(entity) = state.entity {
        if let Ok(mut current) = previews.get_mut(entity) {
            if *current != transform {
                *current = transform;
            }
            return;
        }
        state.entity = None;
    }
    let entity = commands
        .spawn((
            Particles3d(project_handle),
            EditorMode,
            transform,
            Visibility::default(),
            bevy::camera::visibility::RenderLayers::layer(0),
            ZoneParticlePreview,
            Name::new("Temporary particle project preview"),
        ))
        .id();
    state.entity = Some(entity);
}

pub(super) fn playback(
    assets: Res<Assets<ParticlesAsset>>,
    mut systems: Query<
        (Entity, &Particles3d, &mut ParticleSystemRuntime),
        With<ZoneParticlePreview>,
    >,
    mut emitters: Query<(&EmitterEntity, &mut EmitterRuntime)>,
) {
    for (entity, effect, mut runtime) in &mut systems {
        if let Some(asset) = assets.get(effect) {
            bevy_sprinkles_editor::viewport::sync_one_shot_preview(
                asset,
                entity,
                &mut runtime,
                &mut emitters,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_zone::particles::ParticlePlacement;

    fn fixture() -> (App, Handle<ParticlesAsset>) {
        let origin = [128_000_000_000., 4., -128_000_000_000.];
        let particle = ParticlePlacement {
            id: 7,
            name: "Fire".into(),
            effect: "vfx.fire".into(),
            position: [origin[0] + 0.125, origin[1], origin[2] + 1.],
            yaw_degrees: 90.,
            scale: 2.,
        };
        let zone = Zone {
            schema: 8,
            origin,
            spawn: origin,
            particles: vec![particle],
            ..Zone::default()
        };
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone),
        );
        session.particle_selected = Some(7);
        let mut assets = Assets::<ParticlesAsset>::default();
        let mut effect = ParticlesAsset::new(
            "Preview".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        effect.initial_transform.translation = Vec3::X;
        let handle = assets.add(effect);
        let mut editor = EditorState::default();
        editor.current_project = Some(handle.clone());
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(assets)
            .insert_resource(editor)
            .insert_resource(AuthoringHost::embedded(1))
            .init_resource::<workspace::Requests>()
            .init_resource::<State>()
            .add_systems(Update, sync);
        (app, handle)
    }
    fn preview(app: &mut App) -> Option<(Entity, Transform, Handle<ParticlesAsset>)> {
        app.world_mut()
            .query_filtered::<(Entity, &Transform, &Particles3d), With<ZoneParticlePreview>>()
            .iter(app.world())
            .next()
            .map(|(entity, transform, effect)| (entity, *transform, effect.0.clone()))
    }
    fn toggle(app: &mut App) {
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .particle_preview = true;
        app.update();
    }

    #[test]
    fn preview_tracks_anchor_and_open_project_without_editing_the_zone() {
        let (mut app, effect) = fixture();
        let before = app.world().resource::<Session>().editor.zone().clone();
        toggle(&mut app);
        let (entity, transform, selected) = preview(&mut app).unwrap();
        assert_eq!(selected, effect);
        assert!((transform.translation - Vec3::new(0.125, 0., -1.)).length() < 0.0001);
        assert_eq!(transform.scale, Vec3::splat(2.));
        assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
        assert!(!app.world().resource::<Session>().is_dirty());
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::Translate {
                id: 7,
                offset: [0.25, 0., 0.],
            }]);
        app.update();
        let (same, moved, _) = preview(&mut app).unwrap();
        assert_eq!(same, entity);
        assert!((moved.translation - Vec3::new(0.375, 0., -1.)).length() < 0.0001);
        toggle(&mut app);
        assert!(preview(&mut app).is_none());
    }

    #[test]
    fn preview_suspends_in_particle_workspace_and_playtest_and_stops_on_project_or_document_change()
    {
        let (mut app, _) = fixture();
        toggle(&mut app);
        assert!(preview(&mut app).is_some());
        app.world_mut().resource_mut::<AuthoringHost>().active = true;
        app.update();
        assert!(preview(&mut app).is_none());
        app.world_mut().resource_mut::<AuthoringHost>().active = false;
        app.update();
        assert!(preview(&mut app).is_some());
        let play =
            playtest::Playtest::new(app.world().resource::<Session>().editor.zone()).unwrap();
        app.world_mut().resource_mut::<Session>().play = Some(play);
        app.update();
        assert!(preview(&mut app).is_none());
        app.world_mut().resource_mut::<Session>().play = None;
        app.update();
        assert!(preview(&mut app).is_some());
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = None;
        app.update();
        assert!(preview(&mut app).is_none());
        assert!(app.world().resource::<State>().active.is_none());

        let (mut app, _) = fixture();
        toggle(&mut app);
        app.world_mut().resource_mut::<Session>().document_id += 1;
        app.update();
        assert!(preview(&mut app).is_none());
        assert!(app.world().resource::<State>().active.is_none());
    }

    #[test]
    fn completed_one_shot_preview_reuses_sprinkles_restart_behavior() {
        let mut app = App::new();
        let mut assets = Assets::<ParticlesAsset>::default();
        let mut emitter = EmitterData::default();
        emitter.time.one_shot = true;
        emitter.time.fixed_seed = Some(42);
        let effect = assets.add(ParticlesAsset::new(
            "Burst".into(),
            ParticlesDimension::D3,
            default(),
            vec![emitter],
            vec![],
            false,
            default(),
        ));
        app.insert_resource(assets).add_systems(Update, playback);
        let system = app
            .world_mut()
            .spawn((
                ZoneParticlePreview,
                Particles3d(effect),
                ParticleSystemRuntime::default(),
            ))
            .id();
        let mut runtime = EmitterRuntime::new(0, Some(42));
        runtime.one_shot_completed = true;
        runtime.set_emitting(false);
        let emitter = app
            .world_mut()
            .spawn((
                EmitterEntity {
                    parent_system: system,
                },
                runtime,
            ))
            .id();
        app.update();
        let runtime = app.world().get::<EmitterRuntime>(emitter).unwrap();
        assert!(runtime.is_emitting());
        assert!(!runtime.one_shot_completed);
        assert_eq!(runtime.random_seed, 42);
        assert!(runtime.clear_requested);
    }
}
