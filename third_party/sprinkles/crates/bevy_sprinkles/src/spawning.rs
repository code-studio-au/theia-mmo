use bevy::{
    light::NotShadowCaster, pbr::ExtendedMaterial, prelude::*, render::storage::ShaderBuffer,
};

use crate::{
    asset::{DrawPassMaterial, EmitterData, EmitterTrail, ParticlesAsset},
    material::{ParticleEmitterUniforms, ParticleMaterialExtension, TRAIL_THICKNESS_CURVE_SAMPLES},
    mesh::{ParticleMeshCache, checked_mesh_arithmetic},
    runtime::{
        ColliderEntity, CurrentMaterialConfig, CurrentMeshConfig, EditorMode, EmitterEntity,
        EmitterRuntime, ParticleBufferHandle, ParticleData, ParticleMaterial,
        ParticleMaterialHandle, ParticleMeshHandle, ParticleSystemRuntime, Particles3d,
        ParticlesCollider3D, SimulationStep, SubEmitterBufferHandle, TrailHistoryEntry,
    },
};

// Bevy 0.20 ShaderBuffer accepts initialized bytes rather than ShaderType::set_data.
// Retain encase storage layout for the WGSL uniform structure.
fn set_emitter_uniforms(buffer: &mut ShaderBuffer, uniforms: ParticleEmitterUniforms) {
    let mut encoded = bevy::render::render_resource::encase::StorageBuffer::new(Vec::<u8>::new());
    encoded
        .write(&uniforms)
        .expect("particle emitter uniform layout");
    buffer.clear();
    buffer.extend_from_slice(&encoded.into_inner());
}

const MAX_FRAME_DELTA: f32 = 0.1;
const INACTIVE_GRACE_FACTOR: f32 = 1.2;
const MAX_TRAIL_HISTORY_FPS: f32 = 240.0;
fn create_trail_history_buffer(
    amount: u32,
    frames: u32,
    buffers: &mut Assets<ShaderBuffer>,
) -> Option<Handle<ShaderBuffer>> {
    if frames > 0 {
        let data = vec![TrailHistoryEntry::default(); (amount * frames) as usize];
        Some(buffers.add(ShaderBuffer::from(data)))
    } else {
        None
    }
}

fn compute_trail_history_frames(emitter: &EmitterData) -> u32 {
    let trail_size = emitter.trail_size();
    if trail_size <= 1 {
        return 0;
    }
    let effective_fps = if emitter.time.fixed_fps > 0 {
        emitter.time.fixed_fps as f32
    } else {
        MAX_TRAIL_HISTORY_FPS
    };
    let from_stretch = (emitter.trail.stretch_time * effective_fps).ceil() as u32;
    trail_size.max(from_stretch).max(2)
}

// Arithmetic validity only: these checks are not a memory or performance budget.
pub(crate) fn checked_emitter_layout(emitter: &EmitterData) -> Option<(u32, u32)> {
    let amount = emitter.emission.particles_amount;
    if !checked_mesh_arithmetic(&emitter.draw_pass.mesh, amount) {
        return None;
    }
    let trail_size = emitter.trail_size();
    let total_slots = amount.checked_mul(trail_size)?;
    if trail_size > 1 {
        let stretch = emitter.trail.stretch_time;
        if !stretch.is_finite() || stretch < 0.0 {
            return None;
        }
        let fps = if emitter.time.fixed_fps > 0 {
            emitter.time.fixed_fps as f64
        } else {
            MAX_TRAIL_HISTORY_FPS as f64
        };
        if (f64::from(stretch) * fps).ceil() > f64::from(u32::MAX) {
            return None;
        }
    }
    let frames = compute_trail_history_frames(emitter);
    amount.checked_mul(frames)?;
    usize::try_from(amount)
        .ok()?
        .checked_mul(12)?
        .checked_add(4)?;
    Some((total_slots, frames))
}

fn get_particle_asset<'a>(
    parent_system: Entity,
    particle_systems: &Query<&Particles3d>,
    assets: &'a Assets<ParticlesAsset>,
) -> Option<&'a ParticlesAsset> {
    let particle_system = particle_systems.get(parent_system).ok()?;
    assets.get(particle_system)
}

fn get_emitter_data<'a>(
    parent_system: Entity,
    emitter_index: usize,
    particle_systems: &Query<&Particles3d>,
    assets: &'a Assets<ParticlesAsset>,
) -> Option<&'a EmitterData> {
    get_particle_asset(parent_system, particle_systems, assets)
        .and_then(|asset| asset.emitters.get(emitter_index))
}

fn get_editor_assets_folders<'a>(
    parent_system: Entity,
    is_editor: bool,
    particle_systems: &Query<&Particles3d>,
    assets: &'a Assets<ParticlesAsset>,
) -> &'a [String] {
    if !is_editor {
        return &[];
    }
    get_particle_asset(parent_system, particle_systems, assets)
        .map(|a| a.sprinkles_editor.assets_folder.as_slice())
        .unwrap_or(&[])
}

pub fn update_particle_time(
    time: Res<Time>,
    assets: Res<Assets<ParticlesAsset>>,
    system_query: Query<(&Particles3d, &ParticleSystemRuntime)>,
    mut emitter_query: Query<(&EmitterEntity, &mut EmitterRuntime)>,
) {
    for (emitter, mut runtime) in emitter_query.iter_mut() {
        let Ok((particle_system, system_runtime)) = system_query.get(emitter.parent_system) else {
            continue;
        };

        let Some(asset) = assets.get(particle_system) else {
            continue;
        };

        let Some(emitter_data) = asset.emitters.get(runtime.emitter_index) else {
            continue;
        };

        runtime.simulation_steps.clear();

        let clear_requested = runtime.clear_requested;
        runtime.clear_requested = false;
        let world_rebase = std::mem::take(&mut runtime.pending_world_rebase);
        // A clear discards existing positions, so it supersedes translation.
        let world_rebase = if clear_requested {
            Vec3::ZERO
        } else {
            world_rebase
        };

        if runtime.inactive || system_runtime.paused {
            if clear_requested || world_rebase != Vec3::ZERO {
                let step = SimulationStep {
                    prev_system_time: runtime.system_time,
                    system_time: runtime.system_time,
                    cycle: runtime.cycle,
                    delta_time: 0.0,
                    clear_requested,
                    world_rebase,
                    trail_history_write_index: runtime.trail_history_write_index,
                };
                runtime.simulation_steps.push(step);
            }
            continue;
        }

        let fixed_fps = emitter_data.time.fixed_fps;
        let total_duration = emitter_data.time.total_duration();

        if fixed_fps > 0 {
            let fixed_delta = 1.0 / fixed_fps as f32;
            let frame_delta = time.delta_secs().min(MAX_FRAME_DELTA);
            runtime.accumulated_delta += frame_delta;

            while runtime.accumulated_delta >= fixed_delta
                || (clear_requested && runtime.simulation_steps.is_empty())
            {
                runtime.accumulated_delta -= fixed_delta;

                let prev_time = runtime.system_time;
                runtime.system_time += fixed_delta;

                if runtime.system_time >= total_duration && total_duration > 0.0 {
                    runtime.system_time %= total_duration;
                    runtime.cycle += 1;
                }

                let step = SimulationStep {
                    prev_system_time: prev_time,
                    system_time: runtime.system_time,
                    cycle: runtime.cycle,
                    delta_time: fixed_delta,
                    world_rebase: if runtime.simulation_steps.is_empty() {
                        world_rebase
                    } else {
                        Vec3::ZERO
                    },
                    clear_requested: if runtime.simulation_steps.is_empty() {
                        clear_requested
                    } else {
                        false
                    },
                    trail_history_write_index: runtime.trail_history_write_index,
                };
                runtime.advance_trail_history();
                runtime.simulation_steps.push(step);
            }

            if !runtime.simulation_steps.is_empty() {
                runtime.prev_system_time = runtime.simulation_steps[0].prev_system_time;
            } else if world_rebase != Vec3::ZERO {
                let step = SimulationStep {
                    prev_system_time: runtime.system_time,
                    system_time: runtime.system_time,
                    cycle: runtime.cycle,
                    delta_time: 0.,
                    clear_requested: false,
                    world_rebase,
                    trail_history_write_index: runtime.trail_history_write_index,
                };
                runtime.simulation_steps.push(step);
            }
        } else {
            let delta = time.delta_secs();
            let prev_time = runtime.system_time;
            runtime.prev_system_time = runtime.system_time;
            runtime.system_time += delta;

            if runtime.system_time >= total_duration && total_duration > 0.0 {
                runtime.system_time %= total_duration;
                runtime.cycle += 1;
            }

            let step = SimulationStep {
                prev_system_time: prev_time,
                system_time: runtime.system_time,
                cycle: runtime.cycle,
                delta_time: delta,
                clear_requested,
                world_rebase,
                trail_history_write_index: runtime.trail_history_write_index,
            };
            runtime.advance_trail_history();
            runtime.simulation_steps.push(step);
        }

        if emitter_data.time.one_shot && runtime.cycle > 0 && !runtime.one_shot_completed {
            runtime.set_emitting(false);
            runtime.one_shot_completed = true;
        }

        if !runtime.emitting {
            runtime.inactive_time += time.delta_secs();
            let grace = emitter_data.time.lifetime * INACTIVE_GRACE_FACTOR;
            if runtime.inactive_time > grace {
                runtime.inactive = true;
            }
        } else {
            runtime.inactive_time = 0.0;
        }
    }
}

fn transform_align_to_u32(align: Option<crate::asset::TransformAlign>) -> u32 {
    use crate::asset::TransformAlign;
    match align {
        None => 0,
        Some(TransformAlign::Billboard) => 1,
        Some(TransformAlign::YToVelocity) => 2,
        Some(TransformAlign::BillboardYToVelocity) => 3,
        Some(TransformAlign::BillboardFixedY) => 4,
    }
}

fn create_particle_material_from_config(
    config: &DrawPassMaterial,
    sorted_particles_buffer: Handle<ShaderBuffer>,
    emitter_uniforms_buffer: Handle<ShaderBuffer>,
    asset_server: &AssetServer,
    assets_folders: &[String],
) -> Option<ParticleMaterial> {
    let base = match config {
        DrawPassMaterial::Standard(mat) => mat.to_standard_material(asset_server, assets_folders),
        DrawPassMaterial::CustomShader { .. } => return None,
    };

    Some(ExtendedMaterial {
        base,
        extension: ParticleMaterialExtension {
            sorted_particles: sorted_particles_buffer,
            emitter_uniforms: emitter_uniforms_buffer,
        },
    })
}

fn bake_thickness_curve(trail: &EmitterTrail) -> [f32; TRAIL_THICKNESS_CURVE_SAMPLES] {
    let mut samples = [1.0f32; TRAIL_THICKNESS_CURVE_SAMPLES];
    if let Some(ref curve) = trail.thickness_curve {
        for (i, sample) in samples.iter_mut().enumerate() {
            let t = i as f32 / (TRAIL_THICKNESS_CURVE_SAMPLES - 1) as f32;
            *sample = curve.sample(t);
        }
    }
    samples
}

#[derive(Component)]
pub(crate) struct InvalidParticleAsset;

pub fn setup_particle_systems(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Particles3d,
            Has<EditorMode>,
            Has<InvalidParticleAsset>,
            Option<&bevy::camera::visibility::RenderLayers>,
        ),
        Without<ParticleSystemRuntime>,
    >,
    assets: Res<Assets<ParticlesAsset>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_cache: ResMut<ParticleMeshCache>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    for (system_entity, particle_system, is_editor, unsupported, layers) in query.iter() {
        let Some(asset) = assets.get(particle_system) else {
            continue;
        };

        let invalid = asset
            .unsupported_material_index()
            .map(|index| (index, "unsupported custom shader"))
            .or_else(|| {
                asset
                    .emitters
                    .iter()
                    .position(|emitter| checked_emitter_layout(emitter).is_none())
                    .map(|index| (index, "invalid particle, trail or mesh arithmetic"))
            });
        if let Some((index, reason)) = invalid {
            if !unsupported {
                error!("Particle system {system_entity:?}, emitter {index}: {reason}");
                commands
                    .entity(system_entity)
                    .try_insert(InvalidParticleAsset);
            }
            continue;
        }
        if unsupported {
            commands
                .entity(system_entity)
                .remove::<InvalidParticleAsset>();
        }
        if asset.emitters.is_empty() {
            continue;
        }

        let assets_folders = if is_editor {
            asset.sprinkles_editor.assets_folder.as_slice()
        } else {
            &[]
        };

        let initial_transform = asset.initial_transform.to_transform();

        commands
            .entity(system_entity)
            .queue_silenced(move |mut entity: EntityWorldMut| {
                entity
                    .insert(ParticleSystemRuntime::default())
                    .insert_if_new((initial_transform, Visibility::default()));
            });

        let mut emitter_entities: Vec<Entity> = Vec::new();

        for (emitter_index, emitter) in asset.emitters.iter().enumerate() {
            let amount = emitter.emission.particles_amount;
            let trail_size = emitter.trail_size();
            let Some((total_slots, trail_history_frames)) = checked_emitter_layout(emitter) else {
                continue;
            };

            let particles: Vec<ParticleData> =
                (0..total_slots).map(|_| ParticleData::default()).collect();

            let mut particle_buffer = ShaderBuffer::from(particles.clone());
            particle_buffer.buffer_usage |= bevy::render::render_resource::BufferUsages::COPY_SRC;
            let particle_buffer_handle = buffers.add(particle_buffer);

            let indices: Vec<u32> = (0..total_slots).collect();
            let indices_buffer_handle = buffers.add(ShaderBuffer::from(indices));

            let sorted_particles_buffer_handle = buffers.add(ShaderBuffer::from(particles));

            let trail_history_buffer =
                create_trail_history_buffer(amount, trail_history_frames, &mut buffers);

            let emitter_uniforms = ParticleEmitterUniforms {
                emitter_transform: Mat4::IDENTITY,
                max_particles: total_slots,
                particle_flags: emitter.particle_flags.bits(),
                trail_size,
                transform_align: transform_align_to_u32(emitter.draw_pass.transform_align),
                ..default()
            };
            let mut emitter_uniforms_ssbo = ShaderBuffer::default();
            set_emitter_uniforms(&mut emitter_uniforms_ssbo, emitter_uniforms);
            let emitter_uniforms_buffer_handle = buffers.add(emitter_uniforms_ssbo);

            let current_mesh = emitter.draw_pass.mesh.clone();
            let current_material = emitter.draw_pass.material.clone();
            let shadow_caster = emitter.draw_pass.shadow_caster;

            let particle_mesh_handle = mesh_cache.get_or_create(&current_mesh, amount, &mut meshes);

            let Some(material) = create_particle_material_from_config(
                &current_material,
                sorted_particles_buffer_handle.clone(),
                emitter_uniforms_buffer_handle.clone(),
                &asset_server,
                assets_folders,
            ) else {
                continue;
            };
            let material_handle = materials.add(material);

            let mut runtime = EmitterRuntime::new(emitter_index, emitter.time.fixed_seed);
            runtime.trail_history_frames = trail_history_frames;

            let mut emitter_cmds = commands.spawn((
                EmitterEntity {
                    parent_system: system_entity,
                },
                runtime,
                ParticleBufferHandle {
                    particle_buffer: particle_buffer_handle.clone(),
                    indices_buffer: indices_buffer_handle.clone(),
                    sorted_particles_buffer: sorted_particles_buffer_handle.clone(),
                    emitter_uniforms_buffer: emitter_uniforms_buffer_handle,
                    max_particles: total_slots,
                    amount,
                    trail_size,
                    trail_history_buffer,
                    trail_history_frames,
                },
                Mesh3d(particle_mesh_handle.clone()),
                MeshMaterial3d(material_handle.clone()),
                CurrentMeshConfig(current_mesh),
                CurrentMaterialConfig(current_material),
                ParticleMeshHandle(particle_mesh_handle),
                ParticleMaterialHandle(material_handle),
                emitter.initial_transform.to_transform(),
                Visibility::default(),
            ));

            if let Some(layers) = layers {
                emitter_cmds.insert(layers.clone());
            }

            if !shadow_caster {
                emitter_cmds.insert(NotShadowCaster);
            }

            let emitter_entity = emitter_cmds.id();

            emitter_entities.push(emitter_entity);
            commands
                .entity(system_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.add_child(emitter_entity);
                });
        }

        for (emitter_index, emitter) in asset.emitters.iter().enumerate() {
            if let Some(ref sub_config) = emitter.sub_emitter {
                let target_index = sub_config.target_emitter;
                if target_index == emitter_index || target_index >= asset.emitters.len() {
                    continue;
                }

                let target_amount = asset.emitters[target_index].emission.particles_amount;
                let buffer_len = 4 + 12 * target_amount as usize;
                let mut initial_data = vec![0u32; buffer_len];
                initial_data[1] = target_amount;
                let mut buffer = ShaderBuffer::from(initial_data);
                buffer.buffer_usage |= bevy::render::render_resource::BufferUsages::COPY_DST;

                let buffer_handle = buffers.add(buffer);
                let target_entity = emitter_entities[target_index];
                let parent_entity = emitter_entities[emitter_index];

                commands
                    .entity(parent_entity)
                    .insert(SubEmitterBufferHandle {
                        buffer: buffer_handle,
                        target_emitter: target_entity,
                        max_particles: target_amount,
                    });
            }
        }

        for (collider_index, collider_data) in asset.colliders.iter().enumerate() {
            let collider_entity = commands
                .spawn((
                    ColliderEntity {
                        parent_system: system_entity,
                        collider_index,
                    },
                    ParticlesCollider3D {
                        enabled: collider_data.enabled,
                        shape: collider_data.shape.clone(),
                    },
                    collider_data.initial_transform.to_transform(),
                    Name::new(collider_data.name.clone()),
                ))
                .id();

            commands
                .entity(system_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.add_child(collider_entity);
                });
        }
    }
}

pub fn cleanup_particle_entities(
    mut commands: Commands,
    mut removed_systems: RemovedComponents<Particles3d>,
    emitter_entities: Query<Entity, With<EmitterEntity>>,
    emitter_parent_query: Query<&EmitterEntity>,
    collider_entities: Query<(Entity, &ColliderEntity)>,
) {
    for removed_system in removed_systems.read() {
        for emitter_entity in emitter_entities.iter() {
            if let Ok(emitter) = emitter_parent_query.get(emitter_entity)
                && emitter.parent_system == removed_system
            {
                commands.entity(emitter_entity).despawn();
            }
        }

        for (entity, collider) in collider_entities.iter() {
            if collider.parent_system == removed_system {
                commands.entity(entity).despawn();
            }
        }
    }
}

// Apply authored visibility bounds to every emitter, independently of editor
// selection. Mesh bounds describe the unexpanded base mesh, not GPU particles.
pub(crate) fn sync_particle_visibility_bounds(
    mut commands: Commands,
    particle_systems: Query<&Particles3d>,
    assets: Res<Assets<ParticlesAsset>>,
    emitters: Query<(
        Entity,
        &EmitterEntity,
        &EmitterRuntime,
        Option<&bevy::camera::primitives::Aabb>,
    )>,
) {
    for (entity, emitter, runtime, current) in &emitters {
        let Some(data) = get_emitter_data(
            emitter.parent_system,
            runtime.emitter_index,
            &particle_systems,
            &assets,
        ) else {
            continue;
        };
        let bounds = data.draw_pass.visibility_aabb;
        if current.is_none_or(|aabb| {
            aabb.center != bounds.center || aabb.half_extents != bounds.half_extents
        }) {
            commands
                .entity(entity)
                .insert(bevy::camera::primitives::Aabb {
                    center: bounds.center,
                    half_extents: bounds.half_extents,
                });
        }
    }
}

pub fn sync_collider_data(
    particle_systems: Query<&Particles3d>,
    assets: Res<Assets<ParticlesAsset>>,
    mut collider_query: Query<(&ColliderEntity, &mut ParticlesCollider3D, &mut Transform)>,
) {
    if !assets.is_changed() {
        return;
    }

    for (collider, mut collider3d, mut transform) in collider_query.iter_mut() {
        let Some(collider_data) =
            get_particle_asset(collider.parent_system, &particle_systems, &assets)
                .and_then(|asset| asset.colliders.get(collider.collider_index))
        else {
            continue;
        };

        collider3d.enabled = collider_data.enabled;
        collider3d.shape = collider_data.shape.clone();
        *transform = collider_data.initial_transform.to_transform();
    }
}

pub fn sync_particle_mesh(
    particle_systems: Query<&Particles3d>,
    mut emitter_query: Query<(
        &EmitterEntity,
        &EmitterRuntime,
        &ParticleBufferHandle,
        &mut CurrentMeshConfig,
        &mut ParticleMeshHandle,
        &mut Mesh3d,
    )>,
    assets: Res<Assets<ParticlesAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_cache: ResMut<ParticleMeshCache>,
) {
    for (emitter, runtime, buffer_handle, mut current_config, mut mesh_handle, mut mesh3d) in
        emitter_query.iter_mut()
    {
        let Some(emitter_data) = get_emitter_data(
            emitter.parent_system,
            runtime.emitter_index,
            &particle_systems,
            &assets,
        ) else {
            continue;
        };

        if checked_emitter_layout(emitter_data).is_none() {
            continue;
        }
        let new_mesh = emitter_data.draw_pass.mesh.clone();

        if current_config.0 != new_mesh {
            let new_mesh_handle =
                mesh_cache.get_or_create(&new_mesh, buffer_handle.amount, &mut meshes);
            mesh3d.0 = new_mesh_handle.clone();
            current_config.0 = new_mesh;
            mesh_handle.0 = new_mesh_handle;
        }
    }
}

pub(crate) fn sync_particle_buffers(
    particle_systems: Query<&Particles3d>,
    editor_modes: Query<Has<EditorMode>>,
    mut emitter_query: Query<(
        &EmitterEntity,
        &mut EmitterRuntime,
        &mut ParticleBufferHandle,
        &mut ParticleMeshHandle,
        &mut Mesh3d,
        &mut CurrentMeshConfig,
        &mut ParticleMaterialHandle,
        Option<&mut MeshMaterial3d<ParticleMaterial>>,
        &mut CurrentMaterialConfig,
    )>,
    assets: Res<Assets<ParticlesAsset>>,
    asset_server: Res<AssetServer>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_cache: ResMut<ParticleMeshCache>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    for (
        emitter,
        mut runtime,
        mut buffer_handle,
        mut mesh_handle,
        mut mesh3d,
        mut current_config,
        mut material_handle,
        mut material3d,
        mut current_material_config,
    ) in emitter_query.iter_mut()
    {
        let Some(emitter_data) = get_emitter_data(
            emitter.parent_system,
            runtime.emitter_index,
            &particle_systems,
            &assets,
        ) else {
            continue;
        };

        // Leave existing buffers untouched while this emitter has no supported material.
        if matches!(
            emitter_data.draw_pass.material,
            DrawPassMaterial::CustomShader { .. }
        ) {
            continue;
        }
        let new_amount = emitter_data.emission.particles_amount;
        let new_trail_size = emitter_data.trail_size();
        let Some((new_total, new_trail_history_frames)) = checked_emitter_layout(emitter_data)
        else {
            continue;
        };

        if buffer_handle.amount == new_amount
            && buffer_handle.trail_size == new_trail_size
            && buffer_handle.trail_history_frames == new_trail_history_frames
        {
            continue;
        }

        let particles: Vec<ParticleData> =
            (0..new_total).map(|_| ParticleData::default()).collect();

        let mut new_particle_buffer = ShaderBuffer::from(particles.clone());
        new_particle_buffer.buffer_usage |= bevy::render::render_resource::BufferUsages::COPY_SRC;
        let new_particle_buf = buffers.add(new_particle_buffer);
        let new_indices_buf = buffers.add(ShaderBuffer::from((0..new_total).collect::<Vec<u32>>()));
        let new_sorted_buf = buffers.add(ShaderBuffer::from(particles));

        let emitter_uniforms = ParticleEmitterUniforms {
            max_particles: new_total,
            particle_flags: emitter_data.particle_flags.bits(),
            trail_size: new_trail_size,
            transform_align: transform_align_to_u32(emitter_data.draw_pass.transform_align),
            trail_thickness_curve: bake_thickness_curve(&emitter_data.trail),
            ..default()
        };
        let mut emitter_uniforms_ssbo = ShaderBuffer::default();
        set_emitter_uniforms(&mut emitter_uniforms_ssbo, emitter_uniforms);
        let new_uniforms_buf = buffers.add(emitter_uniforms_ssbo);

        buffer_handle.particle_buffer = new_particle_buf;
        buffer_handle.indices_buffer = new_indices_buf;
        buffer_handle.sorted_particles_buffer = new_sorted_buf.clone();
        buffer_handle.emitter_uniforms_buffer = new_uniforms_buf.clone();
        buffer_handle.max_particles = new_total;
        buffer_handle.amount = new_amount;
        buffer_handle.trail_size = new_trail_size;

        buffer_handle.trail_history_buffer =
            create_trail_history_buffer(new_amount, new_trail_history_frames, &mut buffers);
        buffer_handle.trail_history_frames = new_trail_history_frames;
        runtime.trail_history_write_index = 0;
        runtime.trail_history_frames = new_trail_history_frames;

        let is_editor = editor_modes.get(emitter.parent_system).unwrap_or(false);
        let assets_folders =
            get_editor_assets_folders(emitter.parent_system, is_editor, &particle_systems, &assets);

        let Some(material) = create_particle_material_from_config(
            &emitter_data.draw_pass.material,
            new_sorted_buf,
            new_uniforms_buf,
            &asset_server,
            assets_folders,
        ) else {
            continue;
        };
        let new_material = materials.add(material);
        if let Some(ref mut material3d) = material3d {
            material3d.0 = new_material.clone();
        }
        material_handle.0 = new_material;
        current_material_config.0 = emitter_data.draw_pass.material.clone();

        let new_mesh_handle =
            mesh_cache.get_or_create(&emitter_data.draw_pass.mesh, new_amount, &mut meshes);
        mesh3d.0 = new_mesh_handle.clone();
        mesh_handle.0 = new_mesh_handle.clone();
        current_config.0 = emitter_data.draw_pass.mesh.clone();
    }
}

pub fn write_emitter_uniforms(
    particle_systems: Query<&Particles3d>,
    emitter_query: Query<(
        &EmitterEntity,
        &EmitterRuntime,
        &ParticleBufferHandle,
        &GlobalTransform,
    )>,
    assets: Res<Assets<ParticlesAsset>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    for (emitter, runtime, buffer_handle, global_transform) in emitter_query.iter() {
        let Some(emitter_data) = get_emitter_data(
            emitter.parent_system,
            runtime.emitter_index,
            &particle_systems,
            &assets,
        ) else {
            continue;
        };

        if checked_emitter_layout(emitter_data).is_none() {
            continue;
        }
        let trail_size = emitter_data.trail_size();
        let trail_thickness_curve = bake_thickness_curve(&emitter_data.trail);

        let uniforms = ParticleEmitterUniforms {
            emitter_transform: global_transform.to_matrix(),
            max_particles: buffer_handle.max_particles,
            particle_flags: emitter_data.particle_flags.bits(),
            use_local_coords: emitter_data.draw_pass.use_local_coords as u32,
            trail_size,
            transform_align: transform_align_to_u32(emitter_data.draw_pass.transform_align),
            trail_thickness_curve,
        };

        if let Some(mut buffer) = buffers.get_mut(&buffer_handle.emitter_uniforms_buffer) {
            set_emitter_uniforms(&mut buffer, uniforms);
        }
    }
}

#[derive(Component)]
pub(crate) struct InvalidEmitterSuppressed;

pub fn sync_particle_material(
    mut commands: Commands,
    particle_systems: Query<&Particles3d>,
    editor_modes: Query<Has<EditorMode>>,
    mut emitter_query: Query<(
        Entity,
        Has<InvalidEmitterSuppressed>,
        &EmitterEntity,
        &EmitterRuntime,
        &mut CurrentMaterialConfig,
        &mut ParticleMaterialHandle,
        Option<&mut MeshMaterial3d<ParticleMaterial>>,
    )>,
    assets: Res<Assets<ParticlesAsset>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    for (
        entity,
        suppressed,
        emitter,
        runtime,
        mut current_config,
        mut material_handle,
        mut material3d,
    ) in emitter_query.iter_mut()
    {
        let Some(emitter_data) = get_emitter_data(
            emitter.parent_system,
            runtime.emitter_index,
            &particle_systems,
            &assets,
        ) else {
            continue;
        };

        let new_material = emitter_data.draw_pass.material.clone();
        if matches!(new_material, DrawPassMaterial::CustomShader { .. })
            || checked_emitter_layout(emitter_data).is_none()
        {
            if material3d.is_some() {
                error!(
                    "Particle emitter {entity:?} has unsupported material or invalid allocation arithmetic; rendering disabled"
                );
                commands
                    .entity(entity)
                    .remove::<MeshMaterial3d<ParticleMaterial>>()
                    .try_insert(InvalidEmitterSuppressed);
            }
            continue;
        }

        if current_config.0.cache_key() != new_material.cache_key() {
            let (sorted_particles_handle, emitter_uniforms_handle) = {
                let Some(existing_material) = materials.get(&material_handle.0) else {
                    continue;
                };
                (
                    existing_material.extension.sorted_particles.clone(),
                    existing_material.extension.emitter_uniforms.clone(),
                )
            };

            let is_editor = editor_modes.get(emitter.parent_system).unwrap_or(false);
            let assets_folders = get_editor_assets_folders(
                emitter.parent_system,
                is_editor,
                &particle_systems,
                &assets,
            );

            let Some(material) = create_particle_material_from_config(
                &new_material,
                sorted_particles_handle,
                emitter_uniforms_handle,
                &asset_server,
                assets_folders,
            ) else {
                continue;
            };
            let new_material_handle = materials.add(material);
            current_config.0 = new_material;
            material_handle.0 = new_material_handle;
        }
        if let Some(ref mut material3d) = material3d {
            if material3d.0 != material_handle.0 {
                material3d.0 = material_handle.0.clone();
            }
        } else if suppressed {
            commands
                .entity(entity)
                .try_insert(MeshMaterial3d(material_handle.0.clone()))
                .remove::<InvalidEmitterSuppressed>();
        }
    }
}

#[cfg(test)]
mod theia_port_tests {
    use super::*;
    use bevy::render::{render_resource::ShaderType, storage::ShaderBufferData};

    #[test]
    fn world_rebase_is_delivered_once_even_when_paused_or_between_fixed_steps() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::ZERO,
            ))
            .init_resource::<Assets<ParticlesAsset>>()
            .add_systems(Update, update_particle_time);
        let mut emitter = EmitterData::default();
        emitter.time.fixed_fps = 60;
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "test".into(),
                crate::asset::ParticlesDimension::D3,
                default(),
                vec![emitter],
                vec![],
                false,
                default(),
            ));
        let parent = app
            .world_mut()
            .spawn((Particles3d(handle), ParticleSystemRuntime::default()))
            .id();
        let entity = app
            .world_mut()
            .spawn((
                EmitterEntity {
                    parent_system: parent,
                },
                EmitterRuntime::new(0, Some(7)),
            ))
            .id();
        app.update();
        {
            let mut runtime = app.world_mut().get_mut::<EmitterRuntime>(entity).unwrap();
            assert!(runtime.queue_world_rebase(Vec3::new(3., 0., 0.)));
            assert!(runtime.queue_world_rebase(Vec3::new(-1., 0., 0.)));
            assert!(!runtime.queue_world_rebase(Vec3::splat(f32::NAN)));
        }
        app.update();
        let runtime = app.world().get::<EmitterRuntime>(entity).unwrap();
        assert_eq!(runtime.system_time, 0.);
        assert_eq!(runtime.simulation_steps.len(), 1);
        assert_eq!(
            runtime.simulation_steps[0].world_rebase,
            Vec3::new(2., 0., 0.)
        );
        assert_eq!(runtime.simulation_steps[0].delta_time, 0.);
        assert!(!runtime.simulation_steps[0].clear_requested);
        app.update();
        assert!(
            app.world()
                .get::<EmitterRuntime>(entity)
                .unwrap()
                .simulation_steps
                .is_empty()
        );
        app.world_mut()
            .get_mut::<ParticleSystemRuntime>(parent)
            .unwrap()
            .pause();
        app.world_mut()
            .get_mut::<EmitterRuntime>(entity)
            .unwrap()
            .queue_world_rebase(Vec3::Y);
        app.update();
        let step = app
            .world()
            .get::<EmitterRuntime>(entity)
            .unwrap()
            .simulation_steps[0];
        assert_eq!(step.world_rebase, Vec3::Y);
        assert!(!step.clear_requested);
        assert_eq!(step.delta_time, 0.);
        {
            let mut runtime = app.world_mut().get_mut::<EmitterRuntime>(entity).unwrap();
            runtime.queue_world_rebase(Vec3::Z);
            runtime.clear_requested = true;
        }
        app.update();
        let step = app
            .world()
            .get::<EmitterRuntime>(entity)
            .unwrap()
            .simulation_steps[0];
        assert!(step.clear_requested);
        assert_eq!(step.world_rebase, Vec3::ZERO);
    }

    #[test]
    fn overflowing_trail_layout_is_rejected_and_live_edits_resume_after_repair() {
        use crate::asset::{ParticleMesh, RibbonTrailShape};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ShaderBuffer>>()
            .init_resource::<Assets<ParticleMaterial>>()
            .init_resource::<ParticleMeshCache>()
            .add_systems(
                Update,
                (
                    setup_particle_systems,
                    sync_particle_buffers.after(setup_particle_systems),
                    sync_particle_mesh.after(sync_particle_buffers),
                    sync_particle_material.after(sync_particle_mesh),
                ),
            );
        let mut emitter = EmitterData::default();
        emitter.trail.enabled = true;
        emitter.draw_pass.mesh = ParticleMesh::RibbonTrail {
            size: 1.0,
            sections: 2,
            section_rings: 1,
            shape: RibbonTrailShape::Flat,
        };
        emitter.emission.particles_amount = u32::MAX;
        assert!(checked_emitter_layout(&emitter).is_none());
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "Overflow".into(),
                crate::asset::ParticlesDimension::D3,
                default(),
                vec![emitter],
                vec![],
                false,
                default(),
            ));
        let owner = app.world_mut().spawn(Particles3d(handle.clone())).id();
        app.update();
        assert!(app.world().get::<InvalidParticleAsset>(owner).is_some());
        assert!(app.world().get::<ParticleSystemRuntime>(owner).is_none());
        assert_eq!(app.world().resource::<Assets<ShaderBuffer>>().len(), 0);
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .emitters[0]
            .emission
            .particles_amount = 4;
        app.update();
        assert!(app.world().get::<ParticleSystemRuntime>(owner).is_some());
        let mut query = app.world_mut().query::<(Entity, &EmitterEntity)>();
        let emitter_entity = query
            .iter(app.world())
            .find(|(_, e)| e.parent_system == owner)
            .unwrap()
            .0;
        assert_eq!(
            app.world()
                .get::<ParticleBufferHandle>(emitter_entity)
                .unwrap()
                .amount,
            4
        );
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .emitters[0]
            .emission
            .particles_amount = u32::MAX;
        app.update();
        assert!(
            app.world()
                .get::<MeshMaterial3d<ParticleMaterial>>(emitter_entity)
                .is_none()
        );
        assert_eq!(
            app.world()
                .get::<ParticleBufferHandle>(emitter_entity)
                .unwrap()
                .amount,
            4
        );
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .emitters[0]
            .emission
            .particles_amount = 5;
        app.update();
        assert!(
            app.world()
                .get::<MeshMaterial3d<ParticleMaterial>>(emitter_entity)
                .is_some()
        );
        assert_eq!(
            app.world()
                .get::<ParticleBufferHandle>(emitter_entity)
                .unwrap()
                .amount,
            5
        );
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .emitters[0]
            .draw_pass
            .mesh = ParticleMesh::Quad {
            orientation: default(),
            size: Vec2::ONE,
            subdivide: Vec2::splat(f32::MAX),
        };
        app.update();
        assert!(
            app.world()
                .get::<MeshMaterial3d<ParticleMaterial>>(emitter_entity)
                .is_none()
        );
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .emitters[0]
            .draw_pass
            .mesh = ParticleMesh::default();
        app.update();
        assert!(
            app.world()
                .get::<MeshMaterial3d<ParticleMaterial>>(emitter_entity)
                .is_some()
        );
    }

    #[test]
    fn quad_mesh_arithmetic_rejects_overflow_before_allocation() {
        use crate::asset::ParticleMesh;
        let mut emitter = EmitterData::default();
        emitter.emission.particles_amount = 2;
        for subdivide in [
            Vec2::splat(f32::MAX),
            Vec2::splat(65_534.0),
            Vec2::new(30_000.0, 30_000.0),
        ] {
            emitter.draw_pass.mesh = ParticleMesh::Quad {
                orientation: default(),
                size: Vec2::ONE,
                subdivide,
            };
            assert!(checked_emitter_layout(&emitter).is_none());
        }
        emitter.draw_pass.mesh = ParticleMesh::default_quad();
        emitter.emission.particles_amount = u32::MAX / 4 + 1;
        assert!(checked_emitter_layout(&emitter).is_none());
        emitter.emission.particles_amount = 5;
        assert!(checked_emitter_layout(&emitter).is_some());
    }

    #[test]
    fn sphere_mesh_arithmetic_rejects_overflow_before_allocation() {
        use crate::asset::ParticleMesh;
        let mut emitter = EmitterData::default();
        emitter.emission.particles_amount = 2;
        for (segments, rings) in [(u32::MAX, 2), (3, u32::MAX), (65_535, 65_535)] {
            emitter.draw_pass.mesh = ParticleMesh::Sphere {
                radius: 1.0,
                segments,
                rings,
            };
            assert!(checked_emitter_layout(&emitter).is_none());
        }
        emitter.draw_pass.mesh = ParticleMesh::Sphere {
            radius: 0.0,
            segments: 0,
            rings: 0,
        };
        emitter.emission.particles_amount = u32::MAX / 12 + 1;
        assert!(checked_emitter_layout(&emitter).is_none());
        emitter.emission.particles_amount = 5;
        assert!(checked_emitter_layout(&emitter).is_some());
    }

    #[test]
    fn cylinder_mesh_arithmetic_rejects_overflow_before_allocation() {
        use crate::asset::ParticleMesh;
        let mut emitter = EmitterData::default();
        emitter.emission.particles_amount = 1;
        let cylinder = |radial_segments, rings, cap_top, cap_bottom| ParticleMesh::Cylinder {
            top_radius: 1.0,
            bottom_radius: 1.0,
            height: 1.0,
            radial_segments,
            rings,
            cap_top,
            cap_bottom,
        };
        for mesh in [
            cylinder(u32::MAX, 1, false, false),
            cylinder(3, u32::MAX, false, false),
            cylinder(65_535, 65_535, false, false),
        ] {
            emitter.draw_pass.mesh = mesh;
            assert!(checked_emitter_layout(&emitter).is_none());
        }
        emitter.draw_pass.mesh = cylinder(3, 1, true, true);
        // Eight side vertices and five for each cap.
        emitter.emission.particles_amount = u32::MAX / 18 + 1;
        assert!(checked_emitter_layout(&emitter).is_none());
        emitter.draw_pass.mesh = cylinder(3, 1, false, false);
        assert!(checked_emitter_layout(&emitter).is_some());
        emitter.emission.particles_amount = 5;
        emitter.draw_pass.mesh = cylinder(0, 0, true, true);
        assert!(checked_emitter_layout(&emitter).is_some());
    }

    #[test]
    fn trail_mesh_arithmetic_rejects_overflow_before_allocation() {
        use crate::asset::{ParticleMesh, RibbonTrailShape};
        let mut emitter = EmitterData::default();
        emitter.emission.particles_amount = 1;
        for mesh in [
            ParticleMesh::TubeTrail {
                radius: 1.0,
                radial_steps: 3,
                sections: u32::MAX,
                section_rings: 2,
            },
            ParticleMesh::TubeTrail {
                radius: 1.0,
                radial_steps: u32::MAX,
                sections: 2,
                section_rings: 1,
            },
            ParticleMesh::RibbonTrail {
                size: 1.0,
                sections: u32::MAX,
                section_rings: 2,
                shape: RibbonTrailShape::Flat,
            },
            ParticleMesh::RibbonTrail {
                size: 1.0,
                sections: u32::MAX / 2,
                section_rings: 1,
                shape: RibbonTrailShape::Cross,
            },
        ] {
            emitter.draw_pass.mesh = mesh;
            assert!(checked_emitter_layout(&emitter).is_none());
        }
        emitter.draw_pass.mesh = ParticleMesh::RibbonTrail {
            size: 1.0,
            sections: 2,
            section_rings: 1,
            shape: RibbonTrailShape::Cross,
        };
        // Three rows, two vertices per row, two strips.
        emitter.emission.particles_amount = u32::MAX / 12 + 1;
        assert!(checked_emitter_layout(&emitter).is_none());
        emitter.emission.particles_amount = 5;
        assert!(checked_emitter_layout(&emitter).is_some());
        emitter.draw_pass.mesh = ParticleMesh::TubeTrail {
            radius: 1.0,
            radial_steps: 0,
            sections: 0,
            section_rings: 0,
        };
        assert!(checked_emitter_layout(&emitter).is_some());
    }

    #[test]
    fn trail_history_rejects_nonfinite_and_overflowing_frame_counts() {
        use crate::asset::{ParticleMesh, RibbonTrailShape};
        let mut emitter = EmitterData::default();
        emitter.trail.enabled = true;
        emitter.draw_pass.mesh = ParticleMesh::RibbonTrail {
            size: 1.0,
            sections: 2,
            section_rings: 1,
            shape: RibbonTrailShape::Flat,
        };
        for invalid in [f32::NAN, f32::INFINITY, -1.0, f32::MAX] {
            emitter.trail.stretch_time = invalid;
            assert!(checked_emitter_layout(&emitter).is_none());
        }
        emitter.trail.stretch_time = 0.3;
        emitter.emission.particles_amount = u32::MAX / 2;
        assert!(checked_emitter_layout(&emitter).is_none());
        emitter.emission.particles_amount = 5;
        assert!(checked_emitter_layout(&emitter).is_some());
    }

    #[test]
    fn unsupported_materials_never_panic_and_recover_without_stale_buffers() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ShaderBuffer>>()
            .init_resource::<Assets<ParticleMaterial>>()
            .init_resource::<ParticleMeshCache>()
            .add_systems(
                Update,
                (
                    setup_particle_systems,
                    sync_particle_buffers.after(setup_particle_systems),
                    sync_particle_material.after(sync_particle_buffers),
                ),
            );
        let mut effect = ParticlesAsset::new(
            "Material test".into(),
            crate::asset::ParticlesDimension::D3,
            default(),
            vec![EmitterData::default(), EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let original = effect.emitters[0].draw_pass.material.clone();
        effect.emitters[0].draw_pass.material = DrawPassMaterial::CustomShader {
            vertex_shader: None,
            fragment_shader: None,
        };
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(effect);
        let owner = app.world_mut().spawn(Particles3d(handle.clone())).id();
        app.update();
        assert!(app.world().get::<ParticleSystemRuntime>(owner).is_none());
        assert!(app.world().get::<InvalidParticleAsset>(owner).is_some());
        assert_eq!(app.world().resource::<Assets<ShaderBuffer>>().len(), 0);
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .emitters[0]
            .draw_pass
            .material = original.clone();
        app.update();
        assert!(app.world().get::<ParticleSystemRuntime>(owner).is_some());
        assert!(app.world().get::<InvalidParticleAsset>(owner).is_none());
        let mut query = app
            .world_mut()
            .query::<(Entity, &EmitterEntity, &EmitterRuntime)>();
        let mut emitters: Vec<_> = query
            .iter(app.world())
            .filter(|(_, emitter, _)| emitter.parent_system == owner)
            .map(|(entity, _, runtime)| (runtime.emitter_index, entity))
            .collect();
        emitters.sort_by_key(|(index, _)| *index);
        assert_eq!(emitters.len(), 2);
        let target = emitters[0].1;
        let other = emitters[1].1;
        let previous = app
            .world()
            .get::<ParticleBufferHandle>(target)
            .unwrap()
            .amount;
        {
            let mut assets = app.world_mut().resource_mut::<Assets<ParticlesAsset>>();
            let mut effect = assets.get_mut(&handle).unwrap();
            effect.emitters[0].draw_pass.material = DrawPassMaterial::CustomShader {
                vertex_shader: None,
                fragment_shader: None,
            };
            effect.emitters[0].emission.particles_amount = previous + 3;
        }
        app.update();
        assert!(
            app.world()
                .get::<MeshMaterial3d<ParticleMaterial>>(target)
                .is_none()
        );
        assert!(
            app.world()
                .get::<MeshMaterial3d<ParticleMaterial>>(other)
                .is_some()
        );
        assert_eq!(
            app.world()
                .get::<ParticleBufferHandle>(target)
                .unwrap()
                .amount,
            previous
        );
        app.update();
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .emitters[0]
            .draw_pass
            .material = original;
        app.update();
        let buffers = app.world().get::<ParticleBufferHandle>(target).unwrap();
        assert_eq!(buffers.amount, previous + 3);
        let material_handle = &app
            .world()
            .get::<MeshMaterial3d<ParticleMaterial>>(target)
            .unwrap()
            .0;
        let material = app
            .world()
            .resource::<Assets<ParticleMaterial>>()
            .get(material_handle)
            .unwrap();
        assert_eq!(
            material.extension.sorted_particles,
            buffers.sorted_particles_buffer
        );
        assert_eq!(
            material.extension.emitter_uniforms,
            buffers.emitter_uniforms_buffer
        );
        // A caller that deliberately removes its material is not overridden.
        app.world_mut()
            .entity_mut(other)
            .remove::<MeshMaterial3d<ParticleMaterial>>();
        app.update();
        assert!(
            app.world()
                .get::<MeshMaterial3d<ParticleMaterial>>(other)
                .is_none()
        );
    }

    #[test]
    fn particle_meshes_keep_each_owners_render_layers_and_default_layer_behavior() {
        use bevy::camera::visibility::RenderLayers;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ShaderBuffer>>()
            .init_resource::<Assets<ParticleMaterial>>()
            .init_resource::<ParticleMeshCache>()
            .add_systems(Update, setup_particle_systems);
        let asset = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "Layer test".into(),
                crate::asset::ParticlesDimension::D3,
                default(),
                vec![EmitterData::default()],
                vec![],
                false,
                default(),
            ));
        let default_owner = app.world_mut().spawn(Particles3d(asset.clone())).id();
        let isolated_owner = app
            .world_mut()
            .spawn((Particles3d(asset), RenderLayers::layer(7)))
            .id();
        app.update();
        let mut emitters = app
            .world_mut()
            .query::<(&EmitterEntity, Option<&RenderLayers>, &Mesh3d)>();
        let actual: Vec<_> = emitters
            .iter(app.world())
            .map(|(emitter, layers, _)| {
                (
                    emitter.parent_system,
                    layers
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(actual.len(), 2);
        assert!(actual.contains(&(default_owner, vec![0])));
        assert!(actual.contains(&(isolated_owner, vec![7])));
        app.update();
        assert_eq!(emitters.iter(app.world()).count(), 2);
    }

    #[test]
    fn all_emitters_receive_authored_bounds_and_live_updates_without_an_inspector() {
        use bevy::{camera::primitives::Aabb, math::Vec3A};
        let mut app = App::new();
        app.init_resource::<Assets<ParticlesAsset>>()
            .add_systems(Update, sync_particle_visibility_bounds);
        let mut emitters = vec![EmitterData::default(), EmitterData::default()];
        for (index, emitter) in emitters.iter_mut().enumerate() {
            emitter.draw_pass.visibility_aabb.center = Vec3A::splat(index as f32 + 2.);
            emitter.draw_pass.visibility_aabb.half_extents = Vec3A::splat(index as f32 + 20.);
        }
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "Bounds test".into(),
                crate::asset::ParticlesDimension::D3,
                default(),
                emitters,
                vec![],
                false,
                default(),
            ));
        let parent = app.world_mut().spawn(Particles3d(handle.clone())).id();
        let children: Vec<_> = (0..2)
            .map(|index| {
                app.world_mut()
                    .spawn((
                        EmitterEntity {
                            parent_system: parent,
                        },
                        EmitterRuntime::new(index, None),
                        Aabb::default(),
                    ))
                    .id()
            })
            .collect();
        for change in 0..3 {
            app.world_mut()
                .resource_mut::<Assets<ParticlesAsset>>()
                .get_mut(&handle)
                .unwrap()
                .emitters[1]
                .draw_pass
                .visibility_aabb
                .half_extents = Vec3A::splat(30. + change as f32);
            app.update();
            let assets = app.world().resource::<Assets<ParticlesAsset>>();
            for (index, entity) in children.iter().enumerate() {
                let actual = app.world().get::<Aabb>(*entity).unwrap();
                let expected = assets.get(&handle).unwrap().emitters[index]
                    .draw_pass
                    .visibility_aabb;
                assert_eq!(actual.center, expected.center);
                assert_eq!(actual.half_extents, expected.half_extents);
            }
        }
    }

    #[test]
    fn replaced_emitter_uniform_bytes_keep_wgsl_storage_layout_and_values() {
        let mut buffer = ShaderBuffer::default();
        for count in [8, 256, 1] {
            let expected = ParticleEmitterUniforms {
                emitter_transform: Mat4::from_translation(Vec3::new(16., -4., 8.)),
                max_particles: count,
                particle_flags: 5,
                use_local_coords: 1,
                trail_size: 4,
                transform_align: 2,
                trail_thickness_curve: [0.5; TRAIL_THICKNESS_CURVE_SAMPLES],
            };
            set_emitter_uniforms(&mut buffer, expected);
            let ShaderBufferData::Initialized { data, buffer_size } = &buffer.data else {
                panic!("uniform buffer must contain initialized storage bytes");
            };
            assert_eq!(*buffer_size, ParticleEmitterUniforms::min_size().get());
            assert_eq!(data.len() as u64, *buffer_size);
            let mut actual = ParticleEmitterUniforms::default();
            bevy::render::render_resource::encase::StorageBuffer::new(&data[..])
                .read(&mut actual)
                .unwrap();
            assert_eq!(actual.emitter_transform, expected.emitter_transform);
            assert_eq!(actual.max_particles, count);
            assert_eq!(actual.particle_flags, 5);
            assert_eq!(actual.use_local_coords, 1);
            assert_eq!(actual.trail_size, 4);
            assert_eq!(actual.transform_align, 2);
            assert_eq!(actual.trail_thickness_curve, expected.trail_thickness_curve);
        }
    }
}
