//! GPU readback qualification for paused world-space particle and trail rebases.
use bevy::{
    app::AppExit,
    prelude::*,
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{BufferUsages, ShaderType},
        storage::ShaderBuffer,
    },
};
use bevy_sprinkles::{
    ParticleSimulationStep, ParticleSystemSetup, SprinklesPlugin,
    asset::{EmitterData, ParticleMesh, ParticlesAsset, ParticlesDimension, RibbonTrailShape},
    runtime::{
        EmitterRuntime, ParticleBufferHandle, ParticleData, ParticleSystemRuntime, Particles3d,
    },
};

#[derive(Clone, Default, ShaderType)]
struct History {
    position: Vec4,
    velocity: Vec4,
}
#[derive(Component)]
enum ReadKind {
    Head,
    History,
}
#[derive(Resource, Default)]
struct Probe {
    frame: u32,
    stage: u8,
    since: u32,
    attached: bool,
    particles: Option<Vec<ParticleData>>,
    history: Option<Vec<History>>,
    baseline_particles: Option<Vec<ParticleData>>,
    baseline_history: Option<Vec<History>>,
}
const OFFSET: Vec3 = Vec3::new(7., -2., 3.);

fn setup(mut commands: Commands, mut assets: ResMut<Assets<ParticlesAsset>>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 4., 12.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let mut emitter = EmitterData::default();
    emitter.time.lifetime = 3.;
    emitter.time.explosiveness = 0.;
    emitter.time.fixed_seed = Some(7);
    emitter.emission.particles_amount = 1;
    emitter.accelerations.gravity = Vec3::ZERO;
    emitter.trail.enabled = true;
    emitter.trail.stretch_time = 0.5;
    emitter.draw_pass.use_local_coords = false;
    emitter.draw_pass.mesh = ParticleMesh::RibbonTrail {
        size: 0.2,
        sections: 4,
        section_rings: 1,
        shape: RibbonTrailShape::Flat,
    };
    let handle = assets.add(ParticlesAsset::new(
        "GPU rebase probe".into(),
        ParticlesDimension::D3,
        default(),
        vec![emitter],
        vec![],
        false,
        default(),
    ));
    commands.spawn((Particles3d(handle), Transform::from_xyz(1., 2., 3.)));
}
fn attach(
    mut commands: Commands,
    mut probe: ResMut<Probe>,
    buffers: Query<&ParticleBufferHandle>,
    mut assets: ResMut<Assets<ShaderBuffer>>,
) {
    if probe.attached {
        return;
    }
    let Ok(buffer) = buffers.single() else {
        return;
    };
    let Some(history) = buffer.trail_history_buffer.clone() else {
        return;
    };
    let Some(mut asset) = assets.get_mut(&history) else {
        return;
    };
    asset.buffer_usage |= BufferUsages::COPY_SRC;
    commands
        .spawn((
            ReadKind::Head,
            Readback::buffer(buffer.particle_buffer.clone()),
        ))
        .observe(sample);
    commands
        .spawn((ReadKind::History, Readback::buffer(history)))
        .observe(sample);
    probe.attached = true;
}
fn sample(event: On<ReadbackComplete>, kinds: Query<&ReadKind>, mut probe: ResMut<Probe>) {
    match kinds.get(event.entity) {
        Ok(ReadKind::Head) => {
            probe.particles = Some(event.to_shader_type());
        }
        Ok(ReadKind::History) => {
            probe.history = Some(event.to_shader_type::<Vec<History>>());
        }
        Err(_) => {}
    }
}
fn drive(
    mut probe: ResMut<Probe>,
    mut systems: Query<&mut ParticleSystemRuntime>,
    mut emitters: Query<&mut EmitterRuntime>,
    mut exit: MessageWriter<AppExit>,
) {
    probe.frame += 1;
    if probe.frame > 1200 {
        error!(
            "GPU rebase probe timed out: stage={}, attached={}, head={}, history={:?}",
            probe.stage,
            probe.attached,
            probe
                .particles
                .as_ref()
                .is_some_and(|p| p.first().is_some_and(ParticleData::is_active)),
            probe.history.as_ref().map(Vec::len)
        );
        exit.write(AppExit::error());
        return;
    }
    if probe.stage == 0
        && probe.frame >= 120
        && probe
            .particles
            .as_ref()
            .is_some_and(|p| p.first().is_some_and(ParticleData::is_active))
        && probe.history.is_some()
    {
        if let Ok(mut system) = systems.single_mut() {
            system.pause();
            probe.stage = 1;
            probe.since = probe.frame;
        }
    } else if probe.stage == 1 && probe.frame >= probe.since + 30 {
        let (Some(particles), Some(history)) = (probe.particles.clone(), probe.history.clone())
        else {
            return;
        };
        if history.is_empty() {
            return;
        }
        probe.baseline_particles = Some(particles);
        probe.baseline_history = Some(history);
        let Ok(mut emitter) = emitters.single_mut() else {
            return;
        };
        if !emitter.queue_world_rebase(OFFSET) {
            error!("Cannot queue rebase");
            exit.write(AppExit::error());
            return;
        }
        probe.stage = 2;
        probe.since = probe.frame;
    } else if probe.stage == 2 && probe.frame >= probe.since + 30 {
        let (Some(before), Some(after), Some(old), Some(new)) = (
            probe.baseline_particles.as_ref(),
            probe.particles.as_ref(),
            probe.baseline_history.as_ref(),
            probe.history.as_ref(),
        ) else {
            return;
        };
        if before.len() < 2 || before.len() != after.len() {
            error!("GPU trail segment count mismatch");
            exit.write(AppExit::error());
            return;
        }
        let particle_delta = |a: &ParticleData, b: &ParticleData| {
            Vec3::from_array(b.position[..3].try_into().unwrap())
                - Vec3::from_array(a.position[..3].try_into().unwrap())
        };
        let head_delta = particle_delta(&before[0], &after[0]);
        let segment_error = before
            .iter()
            .zip(after)
            .skip(1)
            .map(|(a, b)| (particle_delta(a, b) - OFFSET).abs().max_element())
            .fold(0., f32::max);
        let max_error = old
            .iter()
            .zip(new)
            .map(|(a, b)| {
                ((b.position - a.position).truncate() - OFFSET)
                    .abs()
                    .max_element()
            })
            .fold(0., f32::max);
        if old.len() != new.len()
            || (head_delta - OFFSET).abs().max_element() > 0.001
            || max_error > 0.001
            || segment_error > 0.001
        {
            error!(
                "GPU rebase mismatch: head={head_delta:?}, expected={OFFSET:?}, history_error={max_error}, segment_error={segment_error}"
            );
            exit.write(AppExit::error());
            return;
        }
        let report = serde_json::json!({"passed":true,"head_delta":head_delta.to_array(),"history_entries":old.len(),"max_history_error":max_error,"trail_segments":before.len() - 1,"max_segment_error":segment_error,"paused":true});
        let path = std::env::var_os("THEIA_REBASE_PROBE").expect("THEIA_REBASE_PROBE");
        if let Err(error) = std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()) {
            error!("{error}");
            exit.write(AppExit::error());
            return;
        }
        exit.write(AppExit::Success);
        probe.stage = 3;
    }
}
fn main() {
    let result = App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Particle GPU rebase qualification".into(),
                resolution: (640, 480).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SprinklesPlugin)
        .init_resource::<Probe>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                attach
                    .after(ParticleSystemSetup)
                    .before(ParticleSimulationStep),
                drive.after(ParticleSimulationStep),
            ),
        )
        .run();
    if let AppExit::Error(code) = result {
        std::process::exit(i32::from(code.get()));
    }
}
