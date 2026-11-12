use super::*;

fn library() -> scenery_library::Library {
    let mut library = scenery_library::Library::default();
    for (id, name, category) in [
        ("theia.hut", "Village hut", "Buildings"),
        ("theia.tree", "Forest tree", "Nature"),
    ] {
        library.assets.insert(
            id.into(),
            scenery_library::Asset {
                entry: scenery_library::Entry {
                    id: id.into(),
                    name: name.into(),
                    category: category.into(),
                    file: String::new(),
                    bytes: 1,
                    sha256: String::new(),
                    bounds: [[-1., 0., -1.], [1., 3., 1.]],
                    triangles: 1,
                    source: serde_json::json!({}),
                    collision: None,
                },
                bytes: std::sync::Arc::new(vec![]),
                collision: None,
            },
        );
    }
    library
}
fn session() -> Session {
    let zone = Zone {
        origin: [128_000_000_000., 0., -128_000_000_000.],
        spawn: [128_000_000_000., 0.1, -128_000_000_000.],
        ..default()
    };
    Session::new(
        Editor::new(zone.clone()).unwrap(),
        PathBuf::new(),
        Some(zone),
    )
}
fn fixture() -> App {
    let mut app = App::new();
    app.insert_resource(session())
        .insert_resource(library())
        .init_resource::<workspace::Requests>()
        .init_resource::<InputFocus>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_systems(Update, (sync, scenery_scene::add_requested).chain());
    for i in 0..2 {
        app.world_mut().spawn((Tile(i), Node::default()));
    }
    app.world_mut().spawn((Search, EditableText::new("")));
    app
}
#[test]
fn search_and_category_keep_browsing_read_only_and_clear_filtered_selection() {
    let mut app = fixture();
    let before = app.world().resource::<Session>().editor.zone().clone();
    app.world_mut().resource_mut::<Session>().ui.assets_open = true;
    app.world_mut().resource_mut::<Session>().assets.selected = Some(0);
    let field = app
        .world_mut()
        .query_filtered::<Entity, With<Search>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .get_mut::<EditableText>(field)
        .unwrap()
        .editor_mut()
        .set_text("FOREST nature");
    app.update();
    assert!(app.world().resource::<Session>().assets.selected.is_none());
    let shown: Vec<_> = app
        .world_mut()
        .query::<(&Tile, &Node)>()
        .iter(app.world())
        .filter_map(|(t, n)| (n.display != Display::None).then_some(t.0))
        .collect();
    assert_eq!(shown, [1]);
    app.world_mut().resource_mut::<Session>().assets.category = 1; // Buildings
    app.update();
    assert!(
        app.world_mut()
            .query::<(&Tile, &Node)>()
            .iter(app.world())
            .all(|(_, n)| n.display == Display::None)
    );
    assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 0));
    assert!(!app.world().resource::<Session>().is_dirty());
}
#[test]
fn placement_waits_for_input_then_uses_history_and_rejects_stale_document_requests() {
    let mut app = fixture();
    app.world_mut().resource_mut::<Session>().ui.assets_open = true;
    app.world_mut().resource_mut::<Session>().assets.selected = Some(0);
    let before = app.world().resource::<Session>().editor.zone().clone();
    let request = {
        let s = app.world().resource::<Session>();
        (s.document_id, s.editor.revision(), 0)
    };
    app.world_mut()
        .resource_mut::<workspace::Requests>()
        .add_scenery = Some(request);
    app.world_mut().resource_mut::<Session>().ui.input_owned = true;
    app.update();
    assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
    app.world_mut().resource_mut::<Session>().ui.input_owned = false;
    app.update();
    let mut s = app.world_mut().resource_mut::<Session>();
    assert_eq!(s.editor.zone().scenery.len(), 1);
    assert_eq!(s.editor.zone().scenery[0].asset, "theia.hut");
    assert_eq!(s.editor.zone().scenery[0].position, before.origin);
    let revision = s.editor.revision();
    s.execute(Command::Undo {
        expected_revision: revision,
    });
    assert!(s.editor.zone().scenery.is_empty());
    app.world_mut()
        .resource_mut::<workspace::Requests>()
        .add_scenery = Some(request);
    app.update();
    assert!(
        app.world()
            .resource::<Session>()
            .editor
            .zone()
            .scenery
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<Session>()
            .status
            .contains("Zone changed")
    );
}
#[test]
fn browser_pointer_gesture_stays_owned_outside_panel_until_release() {
    let mut app = fixture();
    app.add_systems(
        Update,
        (
            (|mut s: ResMut<Session>| s.ui.overlay_owned = false),
            panels::resize,
            ownership,
        )
            .chain()
            .before(sync),
    );
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            resolution: (1440, 900).into(),
            ..default()
        })
        .id();
    app.world_mut().resource_mut::<Session>().ui.assets_open = true;
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(900., 750.)));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(900., 300.)));
    app.update();
    assert!(app.world().resource::<Session>().ui.overlay_owned);
    assert!(
        canvas_cursor(
            app.world().get::<Window>(window).unwrap(),
            app.world().resource::<Session>()
        )
        .is_none()
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    app.update();
    assert!(app.world().resource::<Session>().ui.overlay_owned);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.update();
    assert!(!app.world().resource::<Session>().ui.overlay_owned);
}

#[test]
#[ignore = "requires THEIA_SCENERY_LIBRARY and offscreen Vulkan; optional THEIA_ASSET_BROWSER_CAPTURE"]
fn actual_scenery_browser_renders_search_tiles_and_resizable_dock() {
    use std::time::{Duration, Instant};
    let library = scenery_library::Library::load(std::path::Path::new(
        &std::env::var_os("THEIA_SCENERY_LIBRARY").expect("Local scenery catalog"),
    ))
    .unwrap();
    let count = library.assets.len();
    let mut s = session();
    s.ui.assets_open = true;
    s.assets.selected = Some(0);
    let original = s.editor.zone().clone();
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
    )
    .insert_resource(s)
    .insert_resource(library)
    .init_resource::<workspace::Requests>()
    .insert_resource(View {
        target: Vec3::ZERO,
        yaw: 0.,
        pitch: 0.8,
        distance: 32.,
        show_grid: false,
        preset: camera_views::Preset::Perspective,
    })
    .add_plugins((theme::plugin, scrolling::plugin, plugin))
    .add_systems(
        Update,
        (panels::resize, panels::apply, workspace::sync).chain(),
    );
    // A Window component supplies layout metrics; Winit is absent, so no OS window exists.
    app.world_mut().spawn(Window {
        focused: true,
        resolution: (1440, 900).into(),
        ..default()
    });
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            1440,
            900,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.world_mut().spawn((
        Camera2d,
        bevy::ui::IsDefaultUiCamera,
        bevy::camera::RenderTarget::Image(image.clone().into()),
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
    while start.elapsed() < Duration::from_secs(3) {
        app.update();
        std::thread::sleep(Duration::from_millis(16));
    }
    let preview_camera = app.world().resource::<asset_preview::Preview>().camera;
    assert!(app.world().get::<Camera>(preview_camera).unwrap().is_active);
    let first_root = app
        .world()
        .resource::<asset_preview::Preview>()
        .root
        .unwrap()
        .1;
    let descendants: Vec<_> = app
        .world()
        .get::<Children>(first_root)
        .unwrap()
        .iter()
        .collect();
    assert!(!descendants.is_empty());
    assert!(
        app.world_mut()
            .query::<(&Mesh3d, &bevy::camera::visibility::RenderLayers, &Pickable)>()
            .iter(app.world())
            .all(|(_, layers, pick)| {
                !layers.intersects(&bevy::camera::visibility::RenderLayers::layer(0))
                    && !layers.intersects(&bevy::camera::visibility::RenderLayers::layer(1))
                    && layers.intersects(&bevy::camera::visibility::RenderLayers::layer(2))
                    && !pick.is_hoverable
            })
    );
    let (entity, node) = app
        .world_mut()
        .query::<(Entity, &panels::Region)>()
        .iter(app.world())
        .find_map(|(e, r)| {
            (*r == panels::Region::Assets)
                .then_some((e, app.world().get::<ComputedNode>(e).unwrap()))
        })
        .unwrap();
    assert_eq!(node.size(), Vec2::new(1008., 280.));
    let top = app
        .world()
        .get::<bevy::ui::UiGlobalTransform>(entity)
        .unwrap()
        .affine()
        .translation
        .y
        - node.size().y / 2.;
    assert!((top - (900. - status_bar::HEIGHT - 280.)).abs() < 0.1);
    assert_eq!(
        app.world_mut().query::<&Tile>().iter(app.world()).count(),
        count
    );
    let field = app
        .world_mut()
        .query_filtered::<Entity, With<Search>>()
        .single(app.world())
        .unwrap();
    assert!(app.world().get::<ComputedNode>(field).unwrap().size().x >= 80.);
    // Actual widget value changes drive the same filtered tile layout.
    app.world_mut()
        .get_mut::<EditableText>(field)
        .unwrap()
        .editor_mut()
        .set_text("hut");
    for _ in 0..5 {
        app.update();
    }
    let shown = app
        .world_mut()
        .query::<(&Tile, &Node)>()
        .iter(app.world())
        .filter(|(_, n)| n.display != Display::None)
        .count();
    assert!(shown > 0 && shown < count);
    app.world_mut()
        .get_mut::<EditableText>(field)
        .unwrap()
        .editor_mut()
        .set_text("");
    for _ in 0..5 {
        app.update();
    }
    assert!(app.world().get_entity(first_root).is_err());
    assert!(
        descendants
            .iter()
            .all(|e| app.world().get_entity(*e).is_err())
    );
    assert!(!app.world().get::<Camera>(preview_camera).unwrap().is_active);
    // Select a different actual GLB after filtering removed the first preview.
    let hut = app
        .world()
        .resource::<scenery_library::Library>()
        .assets
        .values()
        .position(|a| a.entry.name.to_lowercase().contains("hut"))
        .unwrap();
    app.world_mut().resource_mut::<Session>().assets.selected = Some(hut);
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(4) {
        app.update();
        std::thread::sleep(Duration::from_millis(16));
    }
    assert!(app.world().get::<Camera>(preview_camera).unwrap().is_active);
    // The bottom-dock dropdown uses native popover placement and stays on screen.
    let trigger = app
        .world_mut()
        .query::<(Entity, &menus::Trigger)>()
        .iter(app.world())
        .find_map(|(e, t)| (t.0 == "Category").then_some(e))
        .unwrap();
    app.world_mut().trigger(bevy::ui_widgets::MenuEvent {
        source: trigger,
        action: bevy::ui_widgets::MenuAction::Toggle,
    });
    for _ in 0..5 {
        app.update();
    }
    let popup = app
        .world_mut()
        .query::<(Entity, &menus::Popup)>()
        .iter(app.world())
        .find_map(|(e, p)| (p.0 == "Category").then_some(e))
        .unwrap();
    let size = app.world().get::<ComputedNode>(popup).unwrap().size();
    let centre = app
        .world()
        .get::<bevy::ui::UiGlobalTransform>(popup)
        .unwrap()
        .affine()
        .translation;
    assert!(
        size.x > 0.
            && size.y > 0.
            && centre.y - size.y / 2. >= 0.
            && centre.y + size.y / 2. <= 900.
    );
    #[derive(Resource, Default)]
    struct Captured(bool);
    app.init_resource::<Captured>();
    let capture = app
        .world_mut()
        .spawn(bevy::render::view::screenshot::Screenshot::image(image))
        .id();
    app.world_mut().entity_mut(capture).observe(
        |_: On<bevy::render::view::screenshot::ScreenshotCaptured>, mut c: ResMut<Captured>| {
            c.0 = true
        },
    );
    if let Some(path) = std::env::var_os("THEIA_ASSET_BROWSER_CAPTURE") {
        app.world_mut()
            .entity_mut(capture)
            .observe(bevy::render::view::screenshot::save_to_disk(PathBuf::from(
                path,
            )));
    }
    let start = Instant::now();
    while !app.world().resource::<Captured>().0 {
        app.update();
        assert!(start.elapsed() < Duration::from_secs(30));
        std::thread::sleep(Duration::from_millis(16));
    }
    // Hiding the browser releases its instantiated model and stops the camera.
    let root = app
        .world()
        .resource::<asset_preview::Preview>()
        .root
        .unwrap()
        .1;
    app.world_mut().resource_mut::<Session>().ui.assets_open = false;
    for _ in 0..5 {
        app.update();
    }
    assert!(!app.world().get::<Camera>(preview_camera).unwrap().is_active);
    assert!(app.world().get_entity(root).is_err());
    assert!(
        app.world()
            .resource::<asset_preview::Preview>()
            .root
            .is_none()
    );
    assert_eq!(app.world().resource::<Session>().editor.zone(), &original);
    println!("asset_tiles={count} filtered_tiles={shown} source_unchanged=true os_windows=0");
}

#[test]
fn changing_filter_before_placement_settles_refuses_hidden_model() {
    let mut app = fixture();
    app.world_mut().resource_mut::<Session>().ui.assets_open = true;
    app.world_mut().resource_mut::<Session>().assets.selected = Some(0);
    let request = {
        let s = app.world().resource::<Session>();
        (s.document_id, s.editor.revision(), 0)
    };
    app.world_mut()
        .resource_mut::<workspace::Requests>()
        .add_scenery = Some(request);
    let field = app
        .world_mut()
        .query_filtered::<Entity, With<Search>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .get_mut::<EditableText>(field)
        .unwrap()
        .editor_mut()
        .set_text("tree");
    app.update();
    assert!(
        app.world()
            .resource::<Session>()
            .editor
            .zone()
            .scenery
            .is_empty()
    );
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 0));
}

#[test]
fn ending_search_does_not_run_world_escape_or_other_editor_shortcuts() {
    let mut app = fixture();
    app.init_resource::<outliner::Content>()
        .insert_resource(View {
            target: Vec3::ZERO,
            yaw: 0.,
            pitch: 0.8,
            distance: 32.,
            show_grid: false,
            preset: camera_views::Preset::Perspective,
        })
        .add_systems(Update, keyboard.after(sync))
        .add_systems(PostUpdate, finish_search);
    app.world_mut().spawn(Window {
        focused: true,
        ..default()
    });
    app.world_mut().resource_mut::<Session>().ui.assets_open = true;
    app.world_mut().resource_mut::<Session>().palette = Some(Primitive::Box);
    let field = app
        .world_mut()
        .query_filtered::<Entity, With<Search>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(field, bevy::input_focus::FocusCause::Navigated);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert_eq!(
        app.world().resource::<Session>().palette,
        Some(Primitive::Box)
    );
    assert!(app.world().resource::<InputFocus>().get().is_none());
}
