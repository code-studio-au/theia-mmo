//! Actual editor terrain surface rendered to an image, without a desktop window.
use super::*;
use std::time::{Duration, Instant};
#[test]
#[ignore = "requires offscreen Vulkan; optional THEIA_VALLEY_CAPTURE"]
fn expanded_editor_terrain_renders_without_window() {
    let mut editor = Editor::new(Zone {
        origin: [8192., 0., 8192.],
        spawn: [8192., 0.1, 8192.],
        ..default()
    })
    .unwrap();
    editor
        .execute(Command::Apply {
            expected_revision: 0,
            edits: vec![Edit::CreateStarterValley],
        })
        .unwrap();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    );
    app.insert_resource(GlobalAmbientLight {
        brightness: 350.,
        ..default()
    });
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(terrain::surface_mesh(editor.zone()).unwrap());
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            perceptual_roughness: 1.,
            ..default()
        });
    app.world_mut()
        .spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()));
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            1280,
            960,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.world_mut().spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            far: 4096.,
            ..default()
        }),
        bevy::core_pipeline::tonemapping::Tonemapping::Reinhard,
        bevy::camera::RenderTarget::Image(image.clone().into()),
        Transform::from_xyz(340., 440., 500.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    app.world_mut().spawn((
        DirectionalLight {
            illuminance: 14000.,
            ..default()
        },
        Transform::from_xyz(10., 20., 10.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    while app.plugins_state() != bevy::app::PluginsState::Ready {
        std::thread::yield_now();
    }
    app.finish();
    app.cleanup();
    let info = app
        .sub_app(bevy::render::RenderApp)
        .world()
        .resource::<bevy::render::renderer::RenderAdapterInfo>();
    println!(
        "adapter={} backend={:?} driver={}",
        info.name, info.backend, info.driver_info
    );
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(4) {
        app.update();
        std::thread::sleep(Duration::from_millis(16));
    }
    #[derive(Resource, Default)]
    struct Capture(Option<Image>);
    app.init_resource::<Capture>();
    let screenshot = app
        .world_mut()
        .spawn(bevy::render::view::screenshot::Screenshot::image(image))
        .id();
    app.world_mut().entity_mut(screenshot).observe(
        |event: On<bevy::render::view::screenshot::ScreenshotCaptured>,
         mut result: ResMut<Capture>| result.0 = Some(event.image.clone()),
    );
    if let Some(path) = std::env::var_os("THEIA_VALLEY_CAPTURE") {
        app.world_mut().entity_mut(screenshot).observe(
            bevy::render::view::screenshot::save_to_disk(std::path::PathBuf::from(path)),
        );
    }
    let start = Instant::now();
    while app.world().resource::<Capture>().0.is_none() {
        app.update();
        assert!(start.elapsed() < Duration::from_secs(30));
        std::thread::sleep(Duration::from_millis(16));
    }
    let pixels = app
        .world_mut()
        .resource_mut::<Capture>()
        .0
        .take()
        .unwrap()
        .data
        .unwrap();
    // Grass and road vertex colours must actually reach the rendered image.
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[1] > p[0].saturating_add(8) && p[1] > p[2])
            .count()
            > 10000
    );
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] > p[1].saturating_add(15) && p[1] > p[2])
            .count()
            > 500
    );
    assert_eq!(
        app.world_mut().query::<&Window>().iter(app.world()).count(),
        0
    );
    println!("terrain=512m samples=16641 triangles=32768 grass_and_roads_rendered=true windows=0");
}
