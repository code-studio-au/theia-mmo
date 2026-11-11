//! Explicit previews of open projects or local library effects at zone anchors.
//! Local library references are not yet packaged or delivered to clients.
use bevy_sprinkles::{prelude::*, runtime::EditorMode};
use bevy_sprinkles_editor::{hosting::AuthoringHost, state::EditorState};
use bevy_sprinkles_editor::{
    project::SaveProjectToLibraryEvent, ui::components::toasts::ToastEvent,
};

use super::*;

#[derive(Component)]
pub(super) struct ZoneParticlePreview;

struct Preview {
    document_id: u64,
    placement_id: u64,
    project: Handle<ParticlesAsset>,
    source: PreviewSource,
}

enum PreviewSource {
    OpenProject,
    Library(String),
}

#[derive(Resource)]
pub(super) struct LibraryRoot(pub PathBuf);
impl Default for LibraryRoot {
    fn default() -> Self {
        Self(bevy_sprinkles_editor::io::data_dir())
    }
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
    asset_server: Option<Res<AssetServer>>,
    library_root: Res<LibraryRoot>,
    host: Option<Res<AuthoringHost>>,
    mut state: ResMut<State>,
    mut previews: Query<&mut Transform, With<ZoneParticlePreview>>,
) {
    let host_active = host.as_ref().is_some_and(|host| host.active);
    if std::mem::take(&mut requests.bind_particle)
        && !host_active
        && !workspace::busy(&session)
        && let Some(effect) = session
            .particle_selected
            .and_then(|id| session.editor.zone().particles.iter().find(|p| p.id == id))
            .map(|p| p.effect.clone())
    {
        if editor
            .current_project
            .as_ref()
            .is_some_and(|handle| assets.get(handle).is_some())
        {
            session.status = format!("Saving open particle project as {effect}…");
            commands.trigger(SaveProjectToLibraryEvent(effect));
        } else {
            session.status = "Open a particle project before saving it to an Effect ID".into();
        }
    }
    if std::mem::take(&mut requests.particle_preview)
        && !host_active
        && !workspace::busy(&session)
        && let Some(id) = session.particle_selected
    {
        if state.active.as_ref().is_some_and(|preview| {
            preview.placement_id == id
                && preview.document_id == session.document_id
                && matches!(preview.source, PreviewSource::OpenProject)
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
                source: PreviewSource::OpenProject,
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
    if std::mem::take(&mut requests.preview_library_particle)
        && !host_active
        && !workspace::busy(&session)
        && let Some(placement) = session
            .particle_selected
            .and_then(|id| session.editor.zone().particles.iter().find(|p| p.id == id))
            .cloned()
    {
        if state.active.as_ref().is_some_and(|preview| {
            preview.placement_id == placement.id
                && preview.document_id == session.document_id
                && matches!(&preview.source, PreviewSource::Library(id) if id == &placement.effect)
        }) {
            state.active = None;
            if let Some(entity) = state.entity.take() {
                commands.entity(entity).despawn();
            }
            session.status = "Stopped local effect preview".into();
        } else if let Some(server) = asset_server.as_ref() {
            let path = bevy_sprinkles_editor::io::effect_library_path_in(
                &library_root.0,
                &placement.effect,
            );
            let resolved = path
                .and_then(|path| std::fs::canonicalize(path).ok())
                .filter(|path| path.is_file());
            if let Some(path) = resolved {
                let project: Handle<ParticlesAsset> = server.load(path.clone());
                if assets.get(&project).is_some() {
                    server.reload(path);
                }
                if let Some(entity) = state.entity.take() {
                    commands.entity(entity).despawn();
                }
                session.status =
                    format!("Loading local effect {} · not packaged", placement.effect);
                state.active = Some(Preview {
                    document_id: session.document_id,
                    placement_id: placement.id,
                    project,
                    source: PreviewSource::Library(placement.effect),
                });
            } else {
                session.status = format!("No local effect saved for {}", placement.effect);
            }
        } else {
            session.status = "The asset loader is unavailable".into();
        }
    }
    let valid = state.active.as_ref().is_some_and(|preview| {
        preview.document_id == session.document_id
            && session.editor.zone().particles.iter().any(|p| {
                p.id == preview.placement_id
                    && match &preview.source {
                        PreviewSource::OpenProject => {
                            editor.current_project.as_ref() == Some(&preview.project)
                                && assets.get(&preview.project).is_some()
                        }
                        PreviewSource::Library(id) => p.effect == *id,
                    }
            })
    });
    if !valid {
        if state.active.is_some() {
            session.status =
                "Particle preview stopped because its project or placement changed".into();
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
    let loaded_library = match &preview.source {
        PreviewSource::Library(id) => Some(id.clone()),
        PreviewSource::OpenProject => None,
    };
    if matches!(preview.source, PreviewSource::Library(_))
        && asset_server.as_ref().is_some_and(|server| {
            matches!(
                server.load_state(project_handle.id()),
                bevy::asset::LoadState::Failed(_)
            )
        })
    {
        session.status = "Could not load the local particle effect".into();
        state.active = None;
        if let Some(entity) = state.entity.take() {
            commands.entity(entity).despawn();
        }
        return;
    }
    let placement = session
        .editor
        .zone()
        .particles
        .iter()
        .find(|p| p.id == preview.placement_id)
        .expect("validated placement");
    let Some(project) = assets.get(&preview.project) else {
        return;
    };
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
    if let Some(id) = loaded_library {
        session.status = format!("Previewing local effect {id} · not packaged");
    }
}

pub(super) fn relay_toast(
    event: On<ToastEvent>,
    host: Option<Res<AuthoringHost>>,
    mut session: ResMut<Session>,
) {
    if host.as_ref().is_none_or(|host| !host.active) {
        session.status = event.content.clone();
    }
}

fn respawn_edited_preview(editor: &EditorState, state: &mut State, commands: &mut Commands) {
    if state
        .active
        .as_ref()
        .is_some_and(|preview| editor.current_project.as_ref() == Some(&preview.project))
        && let Some(entity) = state.entity.take()
    {
        commands.entity(entity).despawn();
    }
}

pub(super) fn respawn_emitters(
    _: On<bevy_sprinkles_editor::viewport::RespawnEmittersEvent>,
    editor: Res<EditorState>,
    mut state: ResMut<State>,
    mut commands: Commands,
) {
    respawn_edited_preview(&editor, &mut state, &mut commands);
}

pub(super) fn respawn_colliders(
    _: On<bevy_sprinkles_editor::viewport::RespawnCollidersEvent>,
    editor: Res<EditorState>,
    mut state: ResMut<State>,
    mut commands: Commands,
) {
    respawn_edited_preview(&editor, &mut state, &mut commands);
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

    #[derive(Resource, Default)]
    struct LibraryRequests(Vec<String>);

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
            .init_resource::<LibraryRoot>()
            .init_resource::<workspace::Requests>()
            .init_resource::<State>()
            .add_observer(respawn_emitters)
            .add_observer(respawn_colliders)
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
    fn original_edit_respawn_events_rebuild_only_the_matching_zone_preview() {
        let (mut app, effect) = fixture();
        toggle(&mut app);
        let old = preview(&mut app).unwrap().0;
        app.world_mut()
            .trigger(bevy_sprinkles_editor::viewport::RespawnEmittersEvent);
        app.update();
        let replaced = preview(&mut app).unwrap();
        assert_ne!(old, replaced.0);
        assert_eq!(replaced.2, effect);
        app.world_mut()
            .trigger(bevy_sprinkles_editor::viewport::RespawnCollidersEvent);
        app.update();
        let after = preview(&mut app).unwrap().0;
        assert_ne!(replaced.0, after);
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = None;
        app.world_mut()
            .trigger(bevy_sprinkles_editor::viewport::RespawnEmittersEvent);
        assert!(app.world().get_entity(after).is_ok());
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

    #[test]
    fn binding_uses_the_selected_id_and_reports_particle_save_results_in_zone() {
        let (mut app, _) = fixture();
        app.init_resource::<LibraryRequests>()
            .add_observer(
                |event: On<SaveProjectToLibraryEvent>, mut requests: ResMut<LibraryRequests>| {
                    requests.0.push(event.0.clone())
                },
            )
            .add_observer(relay_toast);
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .bind_particle = true;
        app.update();
        assert_eq!(app.world().resource::<LibraryRequests>().0, ["vfx.fire"]);
        assert!(
            app.world()
                .resource::<Session>()
                .status
                .contains("Saving open particle project")
        );
        app.world_mut()
            .trigger(ToastEvent::error("Could not save project: already exists"));
        assert!(
            app.world()
                .resource::<Session>()
                .status
                .contains("already exists")
        );
        app.world_mut().resource_mut::<AuthoringHost>().active = true;
        app.world_mut()
            .trigger(ToastEvent::success("Hidden workspace toast"));
        assert!(
            app.world()
                .resource::<Session>()
                .status
                .contains("already exists")
        );
    }

    #[test]
    fn local_effect_id_loads_through_bevy_without_an_open_project() {
        use bevy::asset::{AssetApp, AssetPlugin, UnapprovedPathMode};
        use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
        let dir = std::env::temp_dir().join(format!(
            "theia-particle-library-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("library")).unwrap();
        std::fs::write(dir.join("library/vfx.fire.ron"), include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/3d-explosion.ron")).unwrap();
        let (mut app, _) = fixture();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            })
            .init_asset::<ParticlesAsset>()
            .init_asset_loader::<bevy_sprinkles::asset::ParticlesAssetLoader>()
            .insert_resource(LibraryRoot(dir.clone()));
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = None;
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .preview_library_particle = true;
        let deadline = Instant::now() + Duration::from_secs(10);
        while preview(&mut app).is_none() {
            assert!(
                Instant::now() < deadline,
                "Local effect did not load: {}",
                app.world().resource::<Session>().status
            );
            app.update();
            std::thread::sleep(Duration::from_millis(2));
        }
        let (_, _, effect) = preview(&mut app).unwrap();
        assert_eq!(
            app.world()
                .resource::<Assets<ParticlesAsset>>()
                .get(&effect)
                .unwrap()
                .name,
            "Explosion"
        );
        assert!(
            app.world()
                .resource::<Session>()
                .status
                .contains("Previewing local effect vfx.fire")
        );
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .preview_library_particle = true;
        app.update();
        assert!(preview(&mut app).is_none());
        std::fs::write(dir.join("library/vfx.fire.ron"), include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/rain.ron")).unwrap();
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .preview_library_particle = true;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            app.update();
            if let Some((_, _, handle)) = preview(&mut app)
                && app
                    .world()
                    .resource::<Assets<ParticlesAsset>>()
                    .get(&handle)
                    .is_some_and(|asset| asset.name == "Rain")
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "Updated local effect did not reload"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        let mut changed = app.world().resource::<Session>().editor.zone().particles[0].clone();
        changed.effect = "vfx.other".into();
        app.world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::PutParticle { particle: changed }]);
        app.update();
        assert!(preview(&mut app).is_none());
        assert!(app.world().resource::<State>().active.is_none());
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .preview_library_particle = true;
        app.update();
        assert!(
            app.world()
                .resource::<Session>()
                .status
                .contains("No local effect saved")
        );
        std::fs::write(dir.join("library/vfx.other.ron"), b"invalid RON").unwrap();
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .preview_library_particle = true;
        app.update();
        let deadline = Instant::now() + Duration::from_secs(10);
        while app.world().resource::<State>().active.is_some() {
            assert!(
                Instant::now() < deadline,
                "Invalid local effect did not fail"
            );
            app.update();
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(
            app.world()
                .resource::<Session>()
                .status
                .contains("Could not load")
        );
        assert!(preview(&mut app).is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
