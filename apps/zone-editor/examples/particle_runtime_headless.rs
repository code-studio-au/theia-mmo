//! Windowless GPU smoke test for unsupported particle material transitions.
use std::time::Duration;

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    camera::RenderTarget,
    prelude::*,
    render::render_resource::TextureFormat,
    window::ExitCondition,
    winit::WinitPlugin,
};
use bevy_sprinkles::{ParticleBufferHandle, prelude::*};

#[derive(Resource)]
struct Fixture {
    asset: Handle<ParticlesAsset>,
    original: DrawPassMaterial,
    frame: usize,
    saw_suppressed: bool,
    saw_restored: bool,
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut effects: ResMut<Assets<ParticlesAsset>>,
) {
    let image = images.add(Image::new_target_texture(
        256,
        256,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands.spawn((
        Camera3d::default(),
        RenderTarget::Image(image.into()),
        Transform::from_xyz(0., 2., 8.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(2., 5., 3.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let emitter = EmitterData::default();
    let original = emitter.draw_pass.material.clone();
    let effect = effects.add(ParticlesAsset::new(
        "Headless particle check".into(),
        ParticlesDimension::D3,
        default(),
        vec![emitter],
        vec![],
        false,
        default(),
    ));
    commands.spawn(Particles3d(effect.clone()));
    commands.insert_resource(Fixture {
        asset: effect,
        original,
        frame: 0,
        saw_suppressed: false,
        saw_restored: false,
    });
}

fn step(
    mut fixture: ResMut<Fixture>,
    mut effects: ResMut<Assets<ParticlesAsset>>,
    emitters: Query<(
        Option<&MeshMaterial3d<ParticleMaterial>>,
        &ParticleBufferHandle,
    )>,
    mut exits: MessageWriter<AppExit>,
) {
    fixture.frame += 1;
    match fixture.frame {
        30 => {
            let mut effect = effects.get_mut(&fixture.asset).unwrap();
            effect.emitters[0].draw_pass.material = DrawPassMaterial::CustomShader {
                vertex_shader: None,
                fragment_shader: None,
            };
            effect.emitters[0].emission.particles_amount = 11;
        }
        35 => fixture.saw_suppressed = emitters.iter().any(|(material, _)| material.is_none()),
        60 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .draw_pass
                .material = fixture.original.clone()
        }
        65 => {
            fixture.saw_restored = emitters
                .iter()
                .any(|(material, buffers)| material.is_some() && buffers.amount == 11)
        }
        120 => {
            if fixture.saw_suppressed && fixture.saw_restored {
                println!("headless particle material transition passed");
                exits.write(AppExit::Success);
            } else {
                eprintln!(
                    "headless particle material transition failed: suppressed={} restored={}",
                    fixture.saw_suppressed, fixture.saw_restored
                );
                exits.write(AppExit::error());
            }
        }
        _ => {}
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(SprinklesPlugin)
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(16)))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, step)
        .run();
}
