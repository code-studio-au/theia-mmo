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
    saw_invalid_layout_suppressed: bool,
    saw_valid_layout_restored: bool,
    saw_flat_cylinder: bool,
    saw_flat_prism: bool,
    saw_zero_sphere: bool,
    saw_invalid_quad: bool,
    saw_repaired_quad: bool,
    saw_invalid_sphere: bool,
    saw_repaired_sphere: bool,
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
        saw_invalid_layout_suppressed: false,
        saw_valid_layout_restored: false,
        saw_flat_cylinder: false,
        saw_flat_prism: false,
        saw_zero_sphere: false,
        saw_invalid_quad: false,
        saw_repaired_quad: false,
        saw_invalid_sphere: false,
        saw_repaired_sphere: false,
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
        80 => {
            let mut effect = effects.get_mut(&fixture.asset).unwrap();
            let emitter = &mut effect.emitters[0];
            emitter.trail.enabled = true;
            emitter.draw_pass.mesh = ParticleMesh::RibbonTrail {
                size: 1.0,
                sections: 2,
                section_rings: 1,
                shape: RibbonTrailShape::Flat,
            };
            emitter.emission.particles_amount = u32::MAX;
        }
        85 => {
            fixture.saw_invalid_layout_suppressed = emitters
                .iter()
                .any(|(material, buffers)| material.is_none() && buffers.amount == 11)
        }
        95 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .emission
                .particles_amount = 5
        }
        105 => {
            fixture.saw_valid_layout_restored = emitters
                .iter()
                .any(|(material, buffers)| material.is_some() && buffers.amount == 5)
        }
        110 => {
            let mut effect = effects.get_mut(&fixture.asset).unwrap();
            let emitter = &mut effect.emitters[0];
            emitter.trail.enabled = false;
            emitter.draw_pass.mesh = ParticleMesh::Cylinder {
                top_radius: 1.0,
                bottom_radius: 1.0,
                height: 0.0,
                radial_segments: 8,
                rings: 1,
                cap_top: true,
                cap_bottom: true,
            };
        }
        115 => {
            fixture.saw_flat_cylinder = emitters
                .iter()
                .any(|(material, buffers)| material.is_some() && buffers.amount == 5)
        }
        120 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .draw_pass
                .mesh = ParticleMesh::Prism {
                left_to_right: 0.5,
                size: Vec3::new(0.0, 0.0, 1.0),
                subdivide: Vec3::new(2.0, 2.0, 1.0),
            };
        }
        125 => {
            fixture.saw_flat_prism = emitters
                .iter()
                .any(|(material, buffers)| material.is_some() && buffers.amount == 5)
        }
        130 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .draw_pass
                .mesh = ParticleMesh::Sphere {
                radius: 0.0,
                segments: 0,
                rings: 0,
            };
        }
        135 => {
            fixture.saw_zero_sphere = emitters
                .iter()
                .any(|(material, buffers)| material.is_some() && buffers.amount == 5)
        }
        140 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .draw_pass
                .mesh = ParticleMesh::Quad {
                orientation: default(),
                size: Vec2::ONE,
                subdivide: Vec2::splat(f32::MAX),
            };
        }
        145 => {
            fixture.saw_invalid_quad = emitters.iter().any(|(material, _)| material.is_none());
        }
        150 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .draw_pass
                .mesh = ParticleMesh::default();
        }
        160 => {
            fixture.saw_repaired_quad = emitters
                .iter()
                .any(|(material, buffers)| material.is_some() && buffers.amount == 5);
        }
        170 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .draw_pass
                .mesh = ParticleMesh::Sphere {
                radius: 1.0,
                segments: u32::MAX,
                rings: 2,
            };
        }
        175 => {
            fixture.saw_invalid_sphere = emitters.iter().any(|(material, _)| material.is_none());
        }
        180 => {
            effects.get_mut(&fixture.asset).unwrap().emitters[0]
                .draw_pass
                .mesh = ParticleMesh::default();
        }
        190 => {
            fixture.saw_repaired_sphere = emitters
                .iter()
                .any(|(material, buffers)| material.is_some() && buffers.amount == 5);
        }
        200 => {
            if fixture.saw_suppressed
                && fixture.saw_restored
                && fixture.saw_invalid_layout_suppressed
                && fixture.saw_valid_layout_restored
                && fixture.saw_flat_cylinder
                && fixture.saw_flat_prism
                && fixture.saw_zero_sphere
                && fixture.saw_invalid_quad
                && fixture.saw_repaired_quad
                && fixture.saw_invalid_sphere
                && fixture.saw_repaired_sphere
            {
                println!("headless particle material transition passed");
                exits.write(AppExit::Success);
            } else {
                eprintln!(
                    "headless particle transitions failed: shader_suppressed={} shader_restored={} layout_suppressed={} layout_restored={} cylinder={} prism={} sphere={} invalid_quad={} repaired_quad={} invalid_sphere={} repaired_sphere={}",
                    fixture.saw_suppressed,
                    fixture.saw_restored,
                    fixture.saw_invalid_layout_suppressed,
                    fixture.saw_valid_layout_restored,
                    fixture.saw_flat_cylinder,
                    fixture.saw_flat_prism,
                    fixture.saw_zero_sphere,
                    fixture.saw_invalid_quad,
                    fixture.saw_repaired_quad,
                    fixture.saw_invalid_sphere,
                    fixture.saw_repaired_sphere
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
