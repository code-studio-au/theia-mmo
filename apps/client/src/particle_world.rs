//! Bounded, manifest-verified particle placements for a monolithic world.
use bevy::{
    picking::Pickable,
    prelude::*,
    ui_widgets::{Activate, Button},
};
use bevy_sprinkles::{SprinklesPlugin, asset::ParticlesAsset, runtime::Particles3d};
use std::{collections::BTreeMap, path::Path};
use theia_core::coordinates::ChunkId;
use theia_vfx::{BoundCatalog, BoundEffect, CatalogCounts, PreparedCatalog, PreparedResidency};

pub const LIMITS: CatalogCounts = CatalogCounts {
    placements: 32,
    emitters: 64,
    vertices: 1_000_000,
    indices: 3_000_000,
    particle_slots: 8192,
    trail_history_entries: 65_536,
    sub_emitter_words: 100_000,
};
pub const MAX_IMAGE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Resource)]
struct Pending(Option<PreparedCatalog>);
#[derive(Resource)]
struct AuthoredOrigin(theia_core::coordinates::GlobalPosition);
#[derive(Component)]
struct AuthoredParticle(theia_core::coordinates::GlobalPosition);
#[derive(Resource)]
struct KeepAlive {
    _effects: Vec<BoundEffect>,
    _textures: Vec<Handle<Image>>,
}
#[derive(Component)]
struct WorldParticle;
#[derive(Component)]
struct WaitingForScene;
#[derive(Component)]
struct Toggle;
#[derive(Component)]
struct ToggleLabel;
#[derive(Resource)]
pub(crate) struct VisibilityChoice(pub(crate) bool);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    placements: u64,
    live_roots: usize,
    running_emitter_frames: u64,
    visible_emitter_frames: u64,
    world_space_rebase_resets: u64,
    world_space_rebase_translations: u64,
    effects_enabled: bool,
    control_visible: bool,
    control_shown_after_ready: u64,
    control_hidden_after_ready: u64,
    control_visible_ready_frames: u64,
    control_hidden_ready_frames: u64,
    visibility_key_toggles: u64,
    visibility_button_toggles: u64,
    unready_emitter_frames: u64,
    released_systems: u64,
    chunk_publications: u64,
    chunk_retained_roots: u64,
    chunk_retired_roots: u64,
}
#[derive(Resource, Default)]
struct LastFrame(Option<([f64; 3], [f64; 3])>);
#[derive(Resource)]
struct BurstCapture(Option<std::path::PathBuf>);
#[derive(Resource)]
struct FinishMarker(Option<std::path::PathBuf>);

pub fn install(app: &mut App, prepared: PreparedCatalog, visible: bool) {
    install_shared(app, visible);
    app.insert_resource(Pending(Some(prepared)))
        .add_systems(Startup, spawn);
}

pub fn prepare_authored(
    store: &theia_network::authored_world::Store,
) -> Result<Option<PreparedCatalog>, String> {
    let Some((catalog, root)) = store.particles() else {
        return Ok(None);
    };
    let frame = theia_core::coordinates::LocalFrame::new(store.origin());
    theia_vfx::prepare_catalog_with_image_limit(
        catalog,
        &root.join("effects"),
        &root.join("textures"),
        Path::new("particles"),
        frame,
        frame,
        LIMITS,
        MAX_IMAGE_BYTES,
    )
    .map(Some)
}

pub fn install_authored(
    app: &mut App,
    prepared: PreparedCatalog,
    origin: theia_core::coordinates::GlobalPosition,
    visible: bool,
) {
    install(app, prepared, visible);
    app.insert_resource(AuthoredOrigin(origin)).add_systems(
        Update,
        place_authored
            .after(crate::floating_origin::select)
            .after(apply_visibility)
            .before(bevy_sprinkles::ParticleSimulationStep),
    );
}

fn place_authored(
    frame: Res<crate::floating_origin::Frame>,
    choice: Res<VisibilityChoice>,
    mut roots: Query<(&AuthoredParticle, &mut Transform, &mut Visibility)>,
) {
    for (anchor, mut transform, mut visibility) in &mut roots {
        if let Some(local) = frame.render.to_local(anchor.0) {
            transform.translation = Vec3::from_array(local.metres());
            *visibility = if choice.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn install_shared(app: &mut App, visible: bool) {
    if !app.is_plugin_added::<SprinklesPlugin>() {
        app.add_plugins(SprinklesPlugin);
    }
    let capture =
        std::env::var_os("THEIA_WORLD_PARTICLE_FIRST_CAPTURE").map(std::path::PathBuf::from);
    app.insert_resource(BurstCapture(capture));
    app.insert_resource(FinishMarker(
        std::env::var_os("THEIA_WORLD_PARTICLE_FINISHED_MARKER").map(std::path::PathBuf::from),
    ));
    app.insert_resource(VisibilityChoice(visible))
        .init_resource::<Metrics>()
        .init_resource::<LastFrame>()
        .add_observer(activate_toggle)
        .add_systems(Startup, setup_toggle)
        .add_systems(PreUpdate, keyboard_toggle.after(bevy::input::InputSystems))
        .add_systems(Update, apply_visibility)
        .add_systems(
            Update,
            gate_playback
                .after(bevy_sprinkles::ParticleSystemSetup)
                .before(bevy_sprinkles::ParticleSimulationStep),
        )
        .add_systems(
            Update,
            rebase_world_emitters
                .after(crate::floating_origin::place_static)
                .before(bevy_sprinkles::ParticleSimulationStep),
        )
        .add_systems(Last, (observe, mark_finished.after(observe), capture_first));
}

// Opt-in qualification marker: only after at least one system was released and
// all authored placements have actually despawned.
fn mark_finished(mut marker: ResMut<FinishMarker>, metrics: Res<Metrics>) {
    if metrics.placements > 0
        && metrics.released_systems >= metrics.placements
        && metrics.live_roots == 0
        && let Some(path) = marker.0.take()
        && let Err(error) = std::fs::write(path, b"finished\n")
    {
        error!("Cannot write particle completion marker: {error}");
    }
}

#[derive(Resource, Default)]
pub(crate) struct ChunkParticles {
    pending: Option<PreparedResidency>,
    chunks: BTreeMap<ChunkId, (BoundCatalog, Vec<Entity>)>,
    pub counts: CatalogCounts,
    pub image_bytes: usize,
}
impl ChunkParticles {
    pub(crate) fn retained_chunks(&self) -> std::collections::BTreeSet<ChunkId> {
        self.chunks.keys().copied().collect()
    }
}

pub fn install_chunks(app: &mut App, prepared: PreparedResidency, visible: bool) {
    install_shared(app, visible);
    app.insert_resource(ChunkParticles {
        pending: Some(prepared),
        ..default()
    })
    .add_systems(Startup, spawn_initial_chunks);
}

fn spawn_initial_chunks(world: &mut World) {
    let prepared = world
        .resource_mut::<ChunkParticles>()
        .pending
        .take()
        .unwrap();
    if let Err(error) = publish_chunks(world, prepared) {
        error!("Cannot bind initial chunk particles: {error}");
        world.write_message(AppExit::error());
    }
}

/// Bind new chunks before touching the active set; old roots survive any failure.
pub(crate) fn publish_chunks(world: &mut World, prepared: PreparedResidency) -> Result<(), String> {
    let next = prepared
        .chunks
        .iter()
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    let counts = prepared.counts;
    let image_bytes = prepared.decoded_image_bytes;
    let mut bound = Vec::new();
    for (id, catalog) in prepared.chunks {
        if world.resource::<ChunkParticles>().chunks.contains_key(&id) {
            continue;
        }
        let catalog = world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
            theia_vfx::bind_prepared_catalog(
                catalog,
                Path::new("particles"),
                &mut images,
                &mut world.resource_mut::<Assets<ParticlesAsset>>(),
            )
        })?;
        bound.push((id, catalog));
    }
    for (_, catalog) in &bound {
        if catalog.placements.iter().any(|placement| {
            !catalog
                .effects
                .iter()
                .any(|effect| effect.id == placement.effect)
        }) {
            return Err("Chunk particle placement references a missing effect".into());
        }
    }
    let visible = world.resource::<VisibilityChoice>().0;
    let mut new_chunks = Vec::new();
    for (id, catalog) in bound {
        let effects: BTreeMap<_, _> = catalog
            .effects
            .iter()
            .map(|e| (e.id.as_str(), e.handle.clone()))
            .collect();
        let mut roots = Vec::new();
        for placement in &catalog.placements {
            let effect = &effects[placement.effect.as_str()];
            roots.push(
                world
                    .spawn((
                        WorldParticle,
                        WaitingForScene,
                        Particles3d(effect.clone()),
                        crate::floating_origin::Anchored(
                            placement.transform.translation.to_array().map(f64::from),
                        ),
                        placement.transform,
                        if visible {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        },
                    ))
                    .id(),
            );
        }
        new_chunks.push((id, catalog, roots));
    }
    let mut retired = Vec::new();
    let mut retained_count = 0;
    world.resource_scope(|_, mut state: Mut<ChunkParticles>| {
        state.chunks.retain(|id, (_, roots)| {
            if next.contains(id) {
                retained_count += roots.len() as u64;
                true
            } else {
                retired.extend(roots.iter().copied());
                false
            }
        });
        for (id, catalog, roots) in new_chunks {
            state.chunks.insert(id, (catalog, roots));
        }
        state.counts = counts;
        state.image_bytes = image_bytes;
    });
    let retired_count = retired.len() as u64;
    for entity in retired {
        world.despawn(entity);
    }
    let mut metrics = world.resource_mut::<Metrics>();
    metrics.placements = counts.placements;
    metrics.chunk_publications += 1;
    metrics.chunk_retained_roots += retained_count;
    metrics.chunk_retired_roots += retired_count;
    Ok(())
}
#[allow(clippy::too_many_arguments)] // Bevy system resources and queries.
fn spawn(
    mut commands: Commands,
    mut pending: ResMut<Pending>,
    mut images: ResMut<Assets<Image>>,
    mut particles: ResMut<Assets<ParticlesAsset>>,
    mut metrics: ResMut<Metrics>,
    choice: Res<VisibilityChoice>,
    mut exit: MessageWriter<AppExit>,
    authored: Option<Res<AuthoredOrigin>>,
) {
    let Some(prepared) = pending.0.take() else {
        return;
    };
    let bound = match theia_vfx::bind_prepared_catalog(
        prepared,
        Path::new("particles"),
        &mut images,
        &mut particles,
    ) {
        Ok(bound) => bound,
        Err(error) => {
            error!("Cannot bind world particles: {error}");
            exit.write(AppExit::error());
            return;
        }
    };
    let effects: BTreeMap<_, _> = bound
        .effects
        .iter()
        .map(|effect| (effect.id.as_str(), &effect.handle))
        .collect();
    for placement in &bound.placements {
        let Some(effect) = effects.get(placement.effect.as_str()) else {
            error!("Particle placement references a missing effect");
            exit.write(AppExit::error());
            return;
        };
        let mut entity = commands.spawn((
            WorldParticle,
            WaitingForScene,
            Particles3d((*effect).clone()),
            crate::floating_origin::Anchored(
                placement.transform.translation.to_array().map(f64::from),
            ),
            placement.transform,
            if choice.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ));
        if let Some(origin) = &authored {
            let local = placement.transform.translation.to_array();
            let global = theia_core::coordinates::LocalFrame::new(origin.0)
                .to_global(
                    theia_core::coordinates::LocalPosition::new(local)
                        .expect("Prepared local placement"),
                )
                .expect("Prepared global placement");
            entity
                .remove::<crate::floating_origin::Anchored>()
                .insert(AuthoredParticle(global));
        }
    }
    metrics.placements = bound.counts.placements;
    info!(
        "World particle package admitted: {} placements, {} emitters",
        bound.counts.placements, bound.counts.emitters
    );
    commands.insert_resource(KeepAlive {
        _effects: bound.effects,
        _textures: bound.textures,
    });
}

#[allow(clippy::too_many_arguments)] // Bevy system resources and queries.
fn gate_playback(
    mut commands: Commands,
    scene: Option<Res<crate::terrain_scene::Ready>>,
    chunks: Option<Res<crate::chunk_world::Ready>>,
    authored: Option<Res<crate::authored_scene::Scene>>,
    pipelines: Res<crate::capture_ready::CaptureReady>,
    network: Option<Res<crate::network::Network>>,
    mut metrics: ResMut<Metrics>,
    mut systems: Query<
        (Entity, &mut bevy_sprinkles::runtime::ParticleSystemRuntime),
        With<WaitingForScene>,
    >,
) {
    let ready =
        (scene.is_some() || chunks.is_some() || authored.is_some_and(|scene| scene.ready()))
            && pipelines.ready()
            && network.as_ref().is_some_and(|network| {
                let state = network.state();
                state.error.is_none() && state.snapshot.is_some()
            });
    for (entity, mut runtime) in &mut systems {
        if ready {
            runtime.resume();
            commands.entity(entity).remove::<WaitingForScene>();
            metrics.released_systems += 1;
        } else {
            runtime.pause();
        }
    }
}

fn setup_toggle(mut commands: Commands, graybox: Option<Res<crate::graybox::Graybox>>) {
    commands
        .spawn((
            Toggle,
            Button,
            AccessibleLabel::new("Toggle world particle effects"),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: if graybox.is_some() { Val::Auto } else { px(12) },
                right: if graybox.is_some() { px(12) } else { Val::Auto },
                bottom: px(12),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
            GlobalZIndex(30),
        ))
        .with_child((
            ToggleLabel,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(14.),
                ..default()
            },
            Pickable::IGNORE,
        ));
}
fn activate_toggle(
    event: On<Activate>,
    buttons: Query<(), With<Toggle>>,
    windows: Query<&Window>,
    mut choice: ResMut<VisibilityChoice>,
    mut metrics: ResMut<Metrics>,
) {
    if metrics.placements > 0
        && buttons.contains(event.entity)
        && windows.single().is_ok_and(|window| window.focused)
    {
        choice.0 = !choice.0;
        metrics.visibility_button_toggles += 1;
    }
}
fn keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut choice: ResMut<VisibilityChoice>,
    mut metrics: ResMut<Metrics>,
) {
    if metrics.placements > 0
        && windows.single().is_ok_and(|window| window.focused)
        && keys.just_pressed(KeyCode::F9)
    {
        choice.0 = !choice.0;
        metrics.visibility_key_toggles += 1;
    }
}
#[allow(clippy::too_many_arguments)] // Bevy system resources and queries.
fn apply_visibility(
    choice: Res<VisibilityChoice>,
    mut metrics: ResMut<Metrics>,
    mut roots: Query<&mut Visibility, With<WorldParticle>>,
    mut labels: Query<&mut Text, With<ToggleLabel>>,
    mut buttons: Query<&mut Node, With<Toggle>>,
    terrain_ready: Option<Res<crate::terrain_scene::Ready>>,
    chunks_ready: Option<Res<crate::chunk_world::Ready>>,
    authored: Option<Res<crate::authored_scene::Scene>>,
) {
    metrics.effects_enabled = choice.0;
    let visible = metrics.placements > 0;
    let ready = terrain_ready.is_some()
        || chunks_ready.is_some()
        || authored.is_some_and(|scene| scene.ready());
    if ready && visible && !metrics.control_visible {
        metrics.control_shown_after_ready += 1;
    }
    if ready && !visible && metrics.control_visible {
        metrics.control_hidden_after_ready += 1;
    }
    if ready && visible {
        metrics.control_visible_ready_frames += 1;
    }
    if ready && !visible {
        metrics.control_hidden_ready_frames += 1;
    }
    metrics.control_visible = visible;
    for mut node in &mut buttons {
        let desired = if metrics.placements > 0 {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != desired {
            node.display = desired;
        }
    }
    let visibility = if choice.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut current in &mut roots {
        if *current != visibility {
            *current = visibility;
        }
    }
    let text = if choice.0 {
        "World effects: On [F9]"
    } else {
        "World effects: Off [F9]"
    };
    for mut label in &mut labels {
        if label.0 != text {
            **label = text.into();
        }
    }
}

// World-space GPU positions and trail histories refer to the previous render
// frame. Translate them on ordinary render-origin shifts; local-coordinate
// emitters follow their parent. Clear if the content frame changes or the
// translation cannot be represented within the bounded rendering frame.
fn rebase_world_emitters(
    frame: Res<crate::floating_origin::Frame>,
    mut previous: ResMut<LastFrame>,
    mut metrics: ResMut<Metrics>,
    systems: Query<&Particles3d, With<WorldParticle>>,
    assets: Res<Assets<ParticlesAsset>>,
    mut emitters: Query<(
        &bevy_sprinkles::runtime::EmitterEntity,
        &mut bevy_sprinkles::runtime::EmitterRuntime,
    )>,
) {
    let current = (
        frame.content.origin().metres(),
        frame.render.origin().metres(),
    );
    let Some(old) = previous.0.replace(current) else {
        return;
    };
    if old == current {
        return;
    }
    let offset = std::array::from_fn::<_, 3, _>(|i| old.1[i] - current.1[i]);
    let translation = (old.0 == current.0
        && offset
            .iter()
            .all(|v| v.is_finite() && v.abs() <= 2. * theia_core::coordinates::MAX_LOCAL_METRES))
    .then(|| Vec3::from_array(offset.map(|v| v as f32)));
    for (emitter, mut runtime) in &mut emitters {
        let Some(data) = systems
            .get(emitter.parent_system)
            .ok()
            .and_then(|system| assets.get(&system.0))
            .and_then(|asset| asset.emitters.get(runtime.emitter_index))
        else {
            continue;
        };
        if !data.draw_pass.use_local_coords {
            if translation.is_some_and(|offset| runtime.queue_world_rebase(offset)) {
                metrics.world_space_rebase_translations += 1;
            } else {
                // Do not rewind emission time and retrigger one-shot bursts.
                runtime.clear_requested = true;
                runtime.simulation_steps.clear();
                runtime.trail_history_write_index = 0;
                metrics.world_space_rebase_resets += 1;
            }
        }
    }
}

// Optional native qualification capture during early particle playback.
fn capture_first(
    mut commands: Commands,
    mut capture: ResMut<BurstCapture>,
    pipelines: Res<crate::capture_ready::CaptureReady>,
    roots: Query<(), With<WorldParticle>>,
    emitters: Query<(
        &bevy_sprinkles::runtime::EmitterEntity,
        &bevy_sprinkles::runtime::EmitterRuntime,
        &ViewVisibility,
    )>,
) {
    if capture.0.is_none() || !pipelines.ready() {
        return;
    }
    if emitters.iter().any(|(emitter, runtime, visible)| {
        roots.contains(emitter.parent_system) && runtime.system_time >= 0.6 && visible.get()
    }) {
        let path = capture.0.take().unwrap();
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path));
    }
}

fn observe(
    mut metrics: ResMut<Metrics>,
    roots: Query<(), With<WorldParticle>>,
    scene_ready: Option<Res<crate::terrain_scene::Ready>>,
    chunks_ready: Option<Res<crate::chunk_world::Ready>>,
    authored: Option<Res<crate::authored_scene::Scene>>,
    emitters: Query<(
        &bevy_sprinkles::runtime::EmitterEntity,
        &bevy_sprinkles::runtime::EmitterRuntime,
        &ViewVisibility,
    )>,
) {
    metrics.live_roots = roots.iter().count();
    let ready = scene_ready.is_some()
        || chunks_ready.is_some()
        || authored.is_some_and(|scene| scene.ready());
    for (emitter, runtime, visible) in &emitters {
        if roots.contains(emitter.parent_system) && runtime.system_time > 0. {
            metrics.running_emitter_frames += 1;
            metrics.visible_emitter_frames += u64::from(visible.get());
            metrics.unready_emitter_frames += u64::from(!ready);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
    use theia_vfx::{CatalogEffect, CatalogPlacement};

    #[test]
    fn completion_marker_waits_for_released_systems_and_no_live_roots() {
        let path =
            std::env::temp_dir().join(format!("theia-particle-finished-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut app = App::new();
        app.insert_resource(FinishMarker(Some(path.clone())))
            .init_resource::<Metrics>()
            .add_systems(Update, mark_finished);
        app.update();
        assert!(!path.exists());
        {
            let mut m = app.world_mut().resource_mut::<Metrics>();
            m.placements = 1;
            m.released_systems = 1;
            m.live_roots = 1;
        }
        app.update();
        assert!(!path.exists());
        app.world_mut().resource_mut::<Metrics>().live_roots = 0;
        app.update();
        assert_eq!(std::fs::read(&path).unwrap(), b"finished\n");
        std::fs::remove_file(path).unwrap();
    }

    fn chunk_fixture(id: ChunkId, effect_name: &str) -> (ChunkId, PreparedCatalog) {
        let asset = ParticlesAsset::new(
            "test".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        (
            id,
            PreparedCatalog {
                effects: vec![CatalogEffect {
                    id: "vfx.test".into(),
                    asset,
                }],
                textures: vec![],
                placements: vec![CatalogPlacement {
                    id: 1,
                    effect: effect_name.into(),
                    transform: Transform::from_xyz(id.x as f32, 0., 0.),
                }],
                counts: CatalogCounts {
                    placements: 1,
                    emitters: 1,
                    ..default()
                },
            },
        )
    }

    #[test]
    fn chunk_publication_preserves_retained_roots_and_rejects_partial_candidate() {
        let mut world = World::new();
        world.insert_resource(ChunkParticles::default());
        world.insert_resource(VisibilityChoice(true));
        world.insert_resource(Metrics::default());
        world.insert_resource(Assets::<Image>::default());
        world.insert_resource(Assets::<ParticlesAsset>::default());
        let a = ChunkId { x: 0, z: 0 };
        let b = ChunkId { x: 1, z: 0 };
        let candidate = |chunks| PreparedResidency {
            chunks,
            counts: CatalogCounts {
                placements: 2,
                emitters: 2,
                ..default()
            },
            decoded_image_bytes: 0,
        };
        publish_chunks(&mut world, candidate(vec![chunk_fixture(a, "vfx.test")])).unwrap();
        let original = world.resource::<ChunkParticles>().chunks[&a].1[0];
        assert!(
            publish_chunks(
                &mut world,
                candidate(vec![
                    chunk_fixture(a, "vfx.test"),
                    chunk_fixture(b, "missing")
                ])
            )
            .is_err()
        );
        assert_eq!(world.resource::<ChunkParticles>().chunks[&a].1[0], original);
        assert_eq!(world.resource::<ChunkParticles>().chunks.len(), 1);
        publish_chunks(
            &mut world,
            candidate(vec![
                chunk_fixture(a, "vfx.test"),
                chunk_fixture(b, "vfx.test"),
            ]),
        )
        .unwrap();
        assert_eq!(world.resource::<ChunkParticles>().chunks[&a].1[0], original);
        let replacement = world.resource::<ChunkParticles>().chunks[&b].1[0];
        publish_chunks(&mut world, candidate(vec![chunk_fixture(b, "vfx.test")])).unwrap();
        assert!(world.get_entity(original).is_err());
        assert!(world.get_entity(replacement).is_ok());
    }

    #[test]
    fn installs_anchored_effects_with_authored_transform_and_shared_asset() {
        let asset = ParticlesAsset::new(
            "test".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let transform = Transform::from_xyz(2., 3., 4.)
            .with_rotation(Quat::from_rotation_y(0.5))
            .with_scale(Vec3::splat(2.));
        let prepared = PreparedCatalog {
            effects: vec![CatalogEffect {
                id: "vfx.test".into(),
                asset,
            }],
            textures: vec![],
            placements: vec![
                CatalogPlacement {
                    id: 1,
                    effect: "vfx.test".into(),
                    transform,
                },
                CatalogPlacement {
                    id: 2,
                    effect: "vfx.test".into(),
                    transform,
                },
            ],
            counts: CatalogCounts {
                placements: 2,
                emitters: 2,
                ..default()
            },
        };
        let mut app = App::new();
        app.insert_resource(Pending(Some(prepared)))
            .insert_resource(VisibilityChoice(true))
            .init_resource::<Metrics>()
            .insert_resource(Assets::<Image>::default())
            .insert_resource(Assets::<ParticlesAsset>::default())
            .add_message::<AppExit>()
            .add_systems(Startup, spawn);
        app.update();
        let world = app.world_mut();
        let mut query = world.query_filtered::<(&Particles3d, &Transform, &crate::floating_origin::Anchored), With<WorldParticle>>();
        let rows: Vec<_> = query.iter(world).collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0.0.id(), rows[1].0.0.id());
        assert_eq!(*rows[0].1, transform);
        assert_eq!(rows[0].2.0, [2., 3., 4.]);
    }
    #[test]
    fn authored_particles_preserve_global_position_rotation_and_scale_across_rebases() {
        use theia_core::coordinates::{GlobalPosition, LocalFrame};
        let mut app = App::new();
        app.init_resource::<crate::floating_origin::Frame>()
            .insert_resource(VisibilityChoice(true))
            .add_systems(Update, place_authored);
        let global = GlobalPosition::new([8192.125, 0.75, 8190.5]).unwrap();
        let rotation = Quat::from_rotation_y(0.5);
        let entity = app
            .world_mut()
            .spawn((
                AuthoredParticle(global),
                Transform::default()
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(4.)),
                Visibility::Inherited,
            ))
            .id();
        for origin in [[8192., 0., 8192.], [8064., 0., 8192.], [0.; 3]] {
            app.world_mut()
                .resource_mut::<crate::floating_origin::Frame>()
                .render = LocalFrame::new(GlobalPosition::new(origin).unwrap());
            app.update();
            let world = app.world();
            let transform = world.get::<Transform>(entity).unwrap();
            assert_eq!(transform.rotation, rotation);
            assert_eq!(transform.scale, Vec3::splat(4.));
            let frame = &world.resource::<crate::floating_origin::Frame>().render;
            if let Some(local) = frame.to_local(global) {
                assert_eq!(transform.translation.to_array(), local.metres());
            } else {
                assert_eq!(
                    *world.get::<Visibility>(entity).unwrap(),
                    Visibility::Hidden
                );
            }
        }
        app.world_mut().resource_mut::<VisibilityChoice>().0 = false;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Hidden
        );
    }

    #[test]
    fn anchored_particle_transform_tracks_render_frame_rebase() {
        let mut app = App::new();
        app.init_resource::<crate::floating_origin::Frame>()
            .add_systems(Update, crate::floating_origin::place_static);
        let rotation = Quat::from_rotation_y(0.5);
        let entity = app
            .world_mut()
            .spawn((
                WorldParticle,
                crate::floating_origin::Anchored([2., 3., 4.]),
                Transform::from_xyz(2., 3., 4.)
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(2.)),
            ))
            .id();
        app.update();
        let position = theia_core::coordinates::GlobalPosition::new([256., 0., 0.]).unwrap();
        app.world_mut()
            .resource_mut::<crate::floating_origin::Frame>()
            .focus(position);
        app.update();
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(transform.translation, Vec3::new(-254., 3., 4.));
        assert_eq!(transform.rotation, rotation);
        assert_eq!(transform.scale, Vec3::splat(2.));
    }
    #[test]
    fn rebase_translates_world_space_emitters_but_preserves_local_emitters() {
        use bevy_sprinkles::runtime::{EmitterEntity, EmitterRuntime};
        let mut app = App::new();
        app.init_resource::<crate::floating_origin::Frame>()
            .init_resource::<LastFrame>()
            .init_resource::<Metrics>()
            .insert_resource(Assets::<ParticlesAsset>::default())
            .add_systems(Update, rebase_world_emitters);
        let make = |local| {
            let mut emitter = EmitterData::default();
            emitter.draw_pass.use_local_coords = local;
            emitter.time.fixed_seed = Some(7);
            ParticlesAsset::new(
                "test".into(),
                ParticlesDimension::D3,
                default(),
                vec![emitter],
                vec![],
                false,
                default(),
            )
        };
        let world_asset = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(make(false));
        let local_asset = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(make(true));
        let world_root = app
            .world_mut()
            .spawn((WorldParticle, Particles3d(world_asset)))
            .id();
        let local_root = app
            .world_mut()
            .spawn((WorldParticle, Particles3d(local_asset)))
            .id();
        let mut world_runtime = EmitterRuntime::new(0, Some(7));
        world_runtime.system_time = 3.;
        world_runtime.cycle = 2;
        world_runtime.one_shot_completed = true;
        let world_emitter = app
            .world_mut()
            .spawn((
                EmitterEntity {
                    parent_system: world_root,
                },
                world_runtime,
            ))
            .id();
        let mut local_runtime = EmitterRuntime::new(0, Some(7));
        local_runtime.system_time = 3.;
        let local_emitter = app
            .world_mut()
            .spawn((
                EmitterEntity {
                    parent_system: local_root,
                },
                local_runtime,
            ))
            .id();
        app.update();
        assert!(
            !app.world()
                .get::<EmitterRuntime>(world_emitter)
                .unwrap()
                .clear_requested
        );
        app.world_mut()
            .resource_mut::<crate::floating_origin::Frame>()
            .focus(theia_core::coordinates::GlobalPosition::new([256., 0., 0.]).unwrap());
        app.update();
        let world = app.world().get::<EmitterRuntime>(world_emitter).unwrap();
        assert!(!world.clear_requested && world.is_emitting());
        assert_eq!(world.system_time, 3.);
        assert_eq!(world.cycle, 2);
        assert!(world.one_shot_completed);
        assert_eq!(world.random_seed, 7);
        let local = app.world().get::<EmitterRuntime>(local_emitter).unwrap();
        assert!(!local.clear_requested);
        assert_eq!(local.system_time, 3.);
        assert_eq!(
            app.world()
                .resource::<Metrics>()
                .world_space_rebase_translations,
            1
        );
        let mut stopped = app
            .world_mut()
            .get_mut::<EmitterRuntime>(world_emitter)
            .unwrap();
        stopped.stop(Some(7));
        stopped.clear_requested = false;
        app.world_mut()
            .resource_mut::<crate::floating_origin::Frame>()
            .focus(theia_core::coordinates::GlobalPosition::new([-256., 0., 0.]).unwrap());
        app.update();
        let stopped = app.world().get::<EmitterRuntime>(world_emitter).unwrap();
        assert!(!stopped.clear_requested && !stopped.is_emitting());
        let frame = &mut app
            .world_mut()
            .resource_mut::<crate::floating_origin::Frame>();
        frame.content = theia_core::coordinates::LocalFrame::new(
            theia_core::coordinates::GlobalPosition::new([10., 0., 0.]).unwrap(),
        );
        app.update();
        assert!(
            app.world()
                .get::<EmitterRuntime>(world_emitter)
                .unwrap()
                .clear_requested
        );
        assert_eq!(
            app.world().resource::<Metrics>().world_space_rebase_resets,
            1
        );
    }
    #[test]
    fn visibility_controls_affect_only_world_particles_and_require_focus() {
        let mut app = App::new();
        app.insert_resource(VisibilityChoice(true))
            .init_resource::<Metrics>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_observer(activate_toggle)
            .add_systems(Update, (keyboard_toggle, apply_visibility).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                focused: false,
                ..default()
            })
            .id();
        let root = app
            .world_mut()
            .spawn((WorldParticle, Visibility::Inherited))
            .id();
        let gameplay_cue = app.world_mut().spawn(Visibility::Inherited).id();
        let button = app.world_mut().spawn((Toggle, Node::default())).id();
        let label = app.world_mut().spawn((ToggleLabel, Text::new(""))).id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F9);
        app.update();
        assert!(app.world().resource::<VisibilityChoice>().0);
        assert_eq!(
            app.world().get::<Node>(button).unwrap().display,
            Display::None
        );
        app.insert_resource(crate::chunk_world::Ready);
        app.update();
        app.world_mut().resource_mut::<Metrics>().placements = 1;
        app.world_mut().trigger(Activate { entity: button });
        assert!(app.world().resource::<VisibilityChoice>().0);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.update();
        assert_eq!(
            app.world().get::<Node>(button).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().resource::<Metrics>().control_shown_after_ready,
            1
        );
        assert!(
            app.world()
                .resource::<Metrics>()
                .control_hidden_ready_frames
                > 0
        );
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(
            *app.world().get::<Visibility>(gameplay_cue).unwrap(),
            Visibility::Inherited
        );
        assert_eq!(
            app.world().get::<Text>(label).unwrap().0,
            "World effects: Off [F9]"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut().trigger(Activate { entity: button });
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Inherited
        );
        assert_eq!(
            app.world().get::<Text>(label).unwrap().0,
            "World effects: On [F9]"
        );
        app.world_mut().resource_mut::<VisibilityChoice>().0 = false;
        app.world_mut().resource_mut::<Metrics>().placements = 0;
        app.update();
        assert_eq!(
            app.world().get::<Node>(button).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().resource::<Metrics>().control_hidden_after_ready,
            1
        );
        assert!(
            app.world()
                .resource::<Metrics>()
                .control_visible_ready_frames
                > 0
        );
        assert!(!app.world().resource::<VisibilityChoice>().0);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::F9);
            keys.clear();
            keys.press(KeyCode::F9);
        }
        app.update();
        assert!(!app.world().resource::<VisibilityChoice>().0);
    }
    #[test]
    fn unready_scene_keeps_particle_system_paused() {
        let mut app = App::new();
        app.init_resource::<Metrics>()
            .init_resource::<crate::capture_ready::CaptureReady>()
            .add_systems(Update, gate_playback);
        let entity = app
            .world_mut()
            .spawn((
                WaitingForScene,
                bevy_sprinkles::runtime::ParticleSystemRuntime::default(),
            ))
            .id();
        app.update();
        assert!(
            app.world()
                .get::<bevy_sprinkles::runtime::ParticleSystemRuntime>(entity)
                .unwrap()
                .paused
        );
        assert!(app.world().get::<WaitingForScene>(entity).is_some());
        app.insert_resource(crate::terrain_scene::Ready);
        app.world_mut()
            .resource_mut::<crate::capture_ready::CaptureReady>()
            .clear_frames = 3;
        app.update();
        assert!(
            app.world()
                .get::<bevy_sprinkles::runtime::ParticleSystemRuntime>(entity)
                .unwrap()
                .paused
        );
        assert_eq!(app.world().resource::<Metrics>().released_systems, 0);
    }
}
