//! Actual UI renderer, offscreen only; no window, mouse or keyboard backend.
use super::*;
use std::time::{Duration, Instant};

#[test]
#[ignore = "requires offscreen Vulkan; optional THEIA_CONTACT_CAPTURE"]
fn npc_reference_field_renders_aligned_inside_its_frame() {
    let zone = Zone {
        schema: 4,
        markers: vec![theia_zone::markers::Marker {
            id: 1,
            name: "Village guide".into(),
            kind: theia_zone::markers::MarkerKind::Npc,
            position: [2., 0.03, 0.],
            yaw_degrees: 0.,
            radius_metres: 1.,
            definition: Some("theia.dev.guide".into()),
        }],
        ..default()
    };
    let mut session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        "unused.json".into(),
        Some(zone.clone()),
    );
    session.marker_mode = true;
    session.marker_selected = Some(1);
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
    )
    .insert_resource(session)
    .add_plugins((theme::plugin, plugin));
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            420,
            580,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    let camera = app
        .world_mut()
        .spawn((
            Camera2d,
            bevy::camera::RenderTarget::Image(image.clone().into()),
        ))
        .id();
    app.world_mut()
        .commands()
        .spawn((
            Node {
                width: px(420),
                height: px(580),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme::PANEL),
            bevy::ui::UiTargetCamera(camera),
        ))
        .with_children(|root| {
            root.spawn(theme::label("Village guide", 18., theme::DISPLAY));
            spawn(root);
        });
    app.world_mut().flush();
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
    while start.elapsed() < Duration::from_secs(3) {
        app.update();
        std::thread::sleep(Duration::from_millis(16));
    }
    let (field, frame) = app
        .world_mut()
        .query::<(Entity, &super::super::inline::Input, &ChildOf)>()
        .iter(app.world())
        .find_map(|(e, input, parent)| {
            matches!(input, super::super::inline::Input::MarkerDefinition)
                .then_some((e, parent.parent()))
        })
        .unwrap();
    assert_eq!(
        app.world().get::<EditableText>(field).unwrap().value(),
        "theia.dev.guide"
    );
    let size = app.world().get::<ComputedNode>(field).unwrap().size();
    let bounds = app.world().get::<ComputedNode>(frame).unwrap().size();
    assert!(size.min_element() > 0. && size.x <= bounds.x - 12. && size.y <= bounds.y - 2.);
    let y = |entity| {
        app.world()
            .get::<bevy::ui::UiGlobalTransform>(entity)
            .unwrap()
            .affine()
            .translation
            .y
    };
    assert!((y(field) - y(frame)).abs() < 0.5);
    #[derive(Resource, Default)]
    struct Capture(bool);
    app.init_resource::<Capture>();
    let screenshot = app
        .world_mut()
        .spawn(bevy::render::view::screenshot::Screenshot::image(image))
        .id();
    app.world_mut().entity_mut(screenshot).observe(
        |_: On<bevy::render::view::screenshot::ScreenshotCaptured>,
         mut capture: ResMut<Capture>| capture.0 = true,
    );
    if let Some(path) = std::env::var_os("THEIA_CONTACT_CAPTURE") {
        app.world_mut().entity_mut(screenshot).observe(
            bevy::render::view::screenshot::save_to_disk(std::path::PathBuf::from(path)),
        );
    }
    let start = Instant::now();
    while !app.world().resource::<Capture>().0 {
        app.update();
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "Capture timed out"
        );
        std::thread::sleep(Duration::from_millis(16));
    }
    assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
    assert_eq!(
        app.world_mut().query::<&Window>().iter(app.world()).count(),
        0
    );
    println!("catalog_id_visible=true frame_alignment=true source_unchanged=true windows=0");
}
