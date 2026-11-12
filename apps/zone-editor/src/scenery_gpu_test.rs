//! Opt-in real model/handle rendering without an OS window or input injection.
use super::*;
use std::time::{Duration, Instant};

#[test]
#[ignore = "requires THEIA_SCENERY_LIBRARY and an offscreen Vulkan adapter"]
fn imported_model_gizmos_render_and_preview_without_window() {
    let path = std::env::var_os("THEIA_SCENERY_LIBRARY").expect("Local scenery catalog");
    let library = scenery_library::Library::load(std::path::Path::new(&path)).unwrap();
    let asset = library
        .assets
        .values()
        .find(|a| a.entry.id.contains("archway"))
        .unwrap();
    let placement = theia_zone::scenery::SceneryPlacement {
        id: 1,
        name: "Archway transform qualification".into(),
        asset: asset.entry.id.clone(),
        position: [128_000_000_000., 0., -128_000_000_000.],
        yaw_degrees: 0.,
        scale: 1.,
    };
    let zone = Zone {
        schema: 10,
        origin: placement.position,
        spawn: placement.position,
        scenery: vec![placement],
        ..default()
    };
    let mut session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        "unused.zone.json".into(),
        Some(zone.clone()),
    );
    session.scenery_selected = Some(1);
    let mut app = App::new();
    library.register(&mut app);
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
    app.insert_resource(library)
        .insert_resource(session)
        .init_resource::<scenery_scene::Scene>()
        .insert_resource(GlobalAmbientLight {
            brightness: 350.,
            ..default()
        })
        .add_systems(Startup, gizmo::setup)
        .add_systems(Update, (scenery_scene::sync, gizmo::sync).chain());
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            1024,
            768,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.world_mut().spawn((
        Camera3d::default(),
        viewport::ZoneCamera,
        bevy::core_pipeline::tonemapping::Tonemapping::Reinhard,
        bevy::camera::RenderTarget::Image(image.clone().into()),
        Transform::from_xyz(16., 12., 20.).looking_at(Vec3::new(0., 3., 0.), Vec3::Y),
    ));
    app.world_mut().spawn((
        DirectionalLight::default(),
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
    loop {
        app.update();
        let models = app
            .world_mut()
            .query_filtered::<&Mesh3d, Without<gizmo::TransformHandle>>()
            .iter(app.world())
            .count();
        if models > 0 && start.elapsed() > Duration::from_secs(3) {
            break;
        }
        assert!(
            start.elapsed() < Duration::from_secs(60),
            "Model scene did not load"
        );
        std::thread::sleep(Duration::from_millis(16));
    }
    #[derive(Resource, Default)]
    struct Capture(Option<Image>);
    app.init_resource::<Capture>();
    let mut captures = vec![];
    for tool in [gizmo::Tool::Move, gizmo::Tool::Rotate, gizmo::Tool::Scale] {
        app.world_mut().resource_mut::<Session>().gizmo_tool = tool;
        for _ in 0..10 {
            app.update();
        }
        let visible = app
            .world_mut()
            .query::<(&gizmo::TransformHandle, &Visibility)>()
            .iter(app.world())
            .filter(|(_, v)| **v == Visibility::Visible)
            .map(|(h, _)| h.tool)
            .collect::<Vec<_>>();
        assert_eq!(
            visible.len(),
            match tool {
                gizmo::Tool::Move => 6,
                gizmo::Tool::Rotate => 1,
                gizmo::Tool::Scale => 2,
            }
        );
        assert!(visible.iter().all(|t| *t == tool));
        app.world_mut().resource_mut::<Capture>().0 = None;
        let screenshot = app
            .world_mut()
            .spawn(bevy::render::view::screenshot::Screenshot::image(
                image.clone(),
            ))
            .id();
        app.world_mut().entity_mut(screenshot).observe(
            |event: On<bevy::render::view::screenshot::ScreenshotCaptured>,
             mut capture: ResMut<Capture>| {
                capture.0 = Some(event.image.clone());
            },
        );
        if let Some(dir) = std::env::var_os("THEIA_SCENERY_CAPTURE_DIR") {
            let path = std::path::PathBuf::from(dir).join(format!("model-{tool:?}.png"));
            app.world_mut()
                .entity_mut(screenshot)
                .observe(bevy::render::view::screenshot::save_to_disk(path));
        }
        let start = Instant::now();
        while app.world().resource::<Capture>().0.is_none() {
            app.update();
            assert!(
                start.elapsed() < Duration::from_secs(30),
                "Capture timed out"
            );
            std::thread::sleep(Duration::from_millis(16));
        }
        captures.push(
            app.world_mut()
                .resource_mut::<Capture>()
                .0
                .take()
                .unwrap()
                .data
                .unwrap(),
        );
    }
    assert_ne!(captures[0], captures[1]);
    assert_ne!(captures[1], captures[2]);
    // A Window component supplies geometry only: Winit is disabled, so this
    // drives the actual ray-picking system without opening or moving anything.
    use bevy::ecs::system::RunSystemOnce;
    app.world_mut().resource_mut::<Session>().gizmo_tool = gizmo::Tool::Move;
    for _ in 0..3 {
        app.update();
    }
    let handle = app
        .world_mut()
        .query::<(&gizmo::TransformHandle, &Transform)>()
        .iter(app.world())
        .find(|(h, _)| h.tool == gizmo::Tool::Move && h.axis == 0 && h.head)
        .unwrap()
        .1
        .translation;
    let offset = Vec2::new(
        app.world().resource::<Session>().ui.canvas_left(),
        workspace::HEADER,
    );
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            resolution: (1600, 900).into(),
            ..default()
        })
        .id();
    for (point, release) in [(handle, false), (handle + Vec3::X * 2., true)] {
        let (camera, camera_transform) = app
            .world_mut()
            .query_filtered::<(&Camera, &GlobalTransform), With<viewport::ZoneCamera>>()
            .single(app.world())
            .unwrap();
        let screen = camera.world_to_viewport(camera_transform, point).unwrap();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(screen + offset));
        let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        buttons.clear();
        if release {
            buttons.release(MouseButton::Left);
        } else {
            buttons.press(MouseButton::Left);
        }
        app.world_mut().run_system_once(viewport::pointer).unwrap();
        if !release {
            assert!(
                app.world().resource::<Session>().drag.is_some(),
                "Rendered handle was not pickable"
            );
        }
    }
    assert_eq!(app.world().resource::<Session>().editor.revision(), 1);
    let moved = app.world().resource::<Session>().editor.zone().scenery[0].position;
    assert!((moved[0] - zone.scenery[0].position[0] - 2.).abs() < 0.001);
    app.world_mut()
        .resource_mut::<Session>()
        .execute(Command::Undo {
            expected_revision: 1,
        });
    assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
    app.world_mut().despawn(window);
    let (id, before) = transform_target::selected(app.world().resource::<Session>()).unwrap();
    let mut preview = before.clone();
    preview.position[0] += 5.;
    preview.size = [1.5; 3];
    preview.yaw_degrees = 45.;
    app.world_mut().resource_mut::<Session>().drag = Some(Drag {
        id,
        before,
        preview,
        revision: 0,
        gesture: gizmo::Gesture::plane(Ray3d::new(Vec3::Y * 10., Dir3::NEG_Y), Vec3::ZERO).unwrap(),
    });
    for _ in 0..5 {
        app.update();
    }
    let pose = app
        .world_mut()
        .query_filtered::<&Transform, With<scenery_scene::SceneryRoot>>()
        .single(app.world())
        .unwrap();
    assert_eq!(pose.translation, Vec3::X * 5.);
    assert_eq!(pose.scale, Vec3::splat(1.5));
    app.world_mut().resource_mut::<Session>().drag = None;
    for _ in 0..5 {
        app.update();
    }
    let pose = app
        .world_mut()
        .query_filtered::<&Transform, With<scenery_scene::SceneryRoot>>()
        .single(app.world())
        .unwrap();
    assert_eq!(pose.translation, Vec3::ZERO);
    assert_eq!(pose.scale, Vec3::ONE);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
    assert_eq!(
        app.world_mut().query::<&Window>().iter(app.world()).count(),
        0
    );
    println!(
        "three_tools_rendered=true live_preview=true cancel_restored=true source_unchanged=true windows=0"
    );
}
