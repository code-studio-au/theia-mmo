//! Bounded, manifest-verified particle placements for a monolithic world.
use bevy::prelude::*;
use bevy_sprinkles::{SprinklesPlugin, asset::ParticlesAsset, runtime::Particles3d};
use std::{collections::BTreeMap, path::Path};
use theia_vfx::{BoundEffect, CatalogCounts, PreparedCatalog};

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
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    placements: u64,
    live_roots: usize,
    running_emitter_frames: u64,
    visible_emitter_frames: u64,
    world_space_rebase_resets: u64,
}
#[derive(Resource, Default)]
struct LastFrame(Option<([f64; 3], [f64; 3])>);

pub fn install(app: &mut App, prepared: PreparedCatalog) {
    if !app.is_plugin_added::<SprinklesPlugin>() {
        app.add_plugins(SprinklesPlugin);
    }
    app.insert_resource(Pending(Some(prepared)))
        .init_resource::<Metrics>()
        .init_resource::<LastFrame>()
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            rebase_world_emitters
                .after(crate::floating_origin::place_static)
                .before(bevy_sprinkles::ParticleSimulationStep),
        )
        .add_systems(Last, observe);
}
fn spawn(
    mut commands: Commands,
    mut pending: ResMut<Pending>,
    mut images: ResMut<Assets<Image>>,
    mut particles: ResMut<Assets<ParticlesAsset>>,
    mut metrics: ResMut<Metrics>,
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
            Particles3d((*effect).clone()),
            crate::floating_origin::Anchored(
                placement.transform.translation.to_array().map(f64::from),
            ),
            placement.transform,
            Visibility::Inherited,
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

fn observe(
    mut metrics: ResMut<Metrics>,
    roots: Query<(), With<WorldParticle>>,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
    use theia_vfx::{CatalogEffect, CatalogPlacement};

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
}
