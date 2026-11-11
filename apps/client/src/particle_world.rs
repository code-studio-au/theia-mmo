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
struct VisibilityChoice(bool);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    placements: u64,
    live_roots: usize,
    running_emitter_frames: u64,
    visible_emitter_frames: u64,
    world_space_rebase_resets: u64,
    effects_enabled: bool,
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

pub fn install(app: &mut App, prepared: PreparedCatalog, visible: bool) {
    install_shared(app, visible);
    app.insert_resource(Pending(Some(prepared)))
        .add_systems(Startup, spawn);
}

fn install_shared(app: &mut App, visible: bool) {
    if !app.is_plugin_added::<SprinklesPlugin>() {
        app.add_plugins(SprinklesPlugin);
    }
    let capture =
        std::env::var_os("THEIA_WORLD_PARTICLE_FIRST_CAPTURE").map(std::path::PathBuf::from);
    app.insert_resource(BurstCapture(capture));
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
        .add_systems(Last, (observe, capture_first));
}

#[derive(Resource, Default)]
pub(crate) struct ChunkParticles {
    pending: Option<PreparedResidency>,
    chunks: BTreeMap<ChunkId, (BoundCatalog, Vec<Entity>)>,
    pub counts: CatalogCounts,
    pub image_bytes: usize,
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
fn spawn(
    mut commands: Commands,
    mut pending: ResMut<Pending>,
    mut images: ResMut<Assets<Image>>,
    mut particles: ResMut<Assets<ParticlesAsset>>,
    mut metrics: ResMut<Metrics>,
    choice: Res<VisibilityChoice>,
    mut exit: MessageWriter<AppExit>,
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
        commands.spawn((
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

fn gate_playback(
    mut commands: Commands,
    scene: Option<Res<crate::terrain_scene::Ready>>,
    chunks: Option<Res<crate::chunk_world::Ready>>,
    pipelines: Res<crate::capture_ready::CaptureReady>,
    network: Option<Res<crate::network::Network>>,
    mut metrics: ResMut<Metrics>,
    mut systems: Query<
        (Entity, &mut bevy_sprinkles::runtime::ParticleSystemRuntime),
        With<WaitingForScene>,
    >,
) {
    let ready = (scene.is_some() || chunks.is_some())
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

fn setup_toggle(mut commands: Commands) {
    commands
        .spawn((
            Toggle,
            Button,
            AccessibleLabel::new("Toggle world particle effects"),
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
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
    if buttons.contains(event.entity) && windows.single().is_ok_and(|window| window.focused) {
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
    if windows.single().is_ok_and(|window| window.focused) && keys.just_pressed(KeyCode::F9) {
        choice.0 = !choice.0;
        metrics.visibility_key_toggles += 1;
    }
}
fn apply_visibility(
    choice: Res<VisibilityChoice>,
    mut metrics: ResMut<Metrics>,
    mut roots: Query<&mut Visibility, With<WorldParticle>>,
    mut labels: Query<&mut Text, With<ToggleLabel>>,
) {
    metrics.effects_enabled = choice.0;
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
// frame. Clear them when the origin changes; local-coordinate emitters follow
// their parent and do not need a restart.
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
    let changed = previous
        .0
        .replace(current)
        .is_some_and(|old| old != current);
    if !changed {
        return;
    }
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
            let emitting = runtime.is_emitting();
            runtime.stop(data.time.fixed_seed);
            if emitting {
                runtime.play();
            }
            metrics.world_space_rebase_resets += 1;
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
    emitters: Query<(
        &bevy_sprinkles::runtime::EmitterEntity,
        &bevy_sprinkles::runtime::EmitterRuntime,
        &ViewVisibility,
    )>,
) {
    metrics.live_roots = roots.iter().count();
    for (emitter, runtime, visible) in &emitters {
        if roots.contains(emitter.parent_system) && runtime.system_time > 0. {
            metrics.running_emitter_frames += 1;
            metrics.visible_emitter_frames += u64::from(visible.get());
            metrics.unready_emitter_frames +=
                u64::from(scene_ready.is_none() && chunks_ready.is_none());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
    use theia_vfx::{CatalogEffect, CatalogPlacement};

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
    fn rebase_resets_world_space_emitters_but_preserves_local_emitters() {
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
        assert!(world.clear_requested && world.is_emitting());
        assert_eq!(world.system_time, 0.);
        assert_eq!(world.random_seed, 7);
        let local = app.world().get::<EmitterRuntime>(local_emitter).unwrap();
        assert!(!local.clear_requested);
        assert_eq!(local.system_time, 3.);
        assert_eq!(
            app.world().resource::<Metrics>().world_space_rebase_resets,
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
        assert!(stopped.clear_requested && !stopped.is_emitting());
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
        let button = app.world_mut().spawn(Toggle).id();
        let label = app.world_mut().spawn((ToggleLabel, Text::new(""))).id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F9);
        app.update();
        assert!(app.world().resource::<VisibilityChoice>().0);
        app.world_mut().trigger(Activate { entity: button });
        assert!(app.world().resource::<VisibilityChoice>().0);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.update();
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
