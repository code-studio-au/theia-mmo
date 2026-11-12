//! Synthetic input and optional offscreen rendering; never creates an OS window.
use super::*;
use bevy::ecs::system::RunSystemOnce;
use std::time::Duration;

fn session() -> Session {
    let zone = Zone::default();
    let mut session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        "unused.json".into(),
        Some(zone.clone()),
    );
    session.play = Some(playtest::Playtest::new(&zone).unwrap());
    session
}
fn settle(app: &mut App) {
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            ..default()
        })
        .id();
    for _ in 0..5 {
        app.world_mut().run_system_once(playtest::step).unwrap();
    }
    app.world_mut().despawn(window);
}

#[test]
fn automatic_pose_follows_character_motion_and_stops_when_unfocused() {
    let mut app = App::new();
    app.insert_resource(session());
    settle(&mut app);
    let mut session = app.world_mut().resource_mut::<Session>();
    let play = session.play.as_mut().unwrap();
    assert_eq!(automatic(play, 0., None), ("Idle", None));
    for (velocity, heading, expected) in [
        (Vec3::NEG_Z * 7., 0., "Run"),
        (Vec3::X * 7., 0., "RunRight"),
        (Vec3::Z * 4.5, 0., "RunBack"),
        (Vec3::NEG_X * 2.5, 0., "WalkLeft"),
        (Vec3::NEG_X * 7., std::f32::consts::FRAC_PI_2, "Run"),
    ] {
        play.velocity = velocity;
        assert_eq!(automatic(play, heading, None).0, expected);
    }
    app.world_mut().run_system_once(playtest::step).unwrap();
    assert_eq!(
        automatic(
            app.world().resource::<Session>().play.as_ref().unwrap(),
            0.,
            None
        )
        .0,
        "Idle"
    );
}

#[test]
fn preview_controls_require_available_art_and_preserve_authoring_gates() {
    use workspace::Control;
    let mut session = session();
    assert!(!Control::PlayerModel.enabled(&session));
    session.player_clips = vec!["Idle".into(), "SwordAttack".into()];
    assert!(Control::PlayerModel.enabled(&session));
    assert!(Control::PlayerAnimation(Preview::SwordAttack).enabled(&session));
    assert!(!Control::PlayerAnimation(Preview::SpellCast).enabled(&session));
    assert!(!Control::Command(Action::Delete).enabled(&session));
    session.ui.input_owned = true;
    assert!(!Control::PlayerModel.enabled(&session));
    session.ui.input_owned = false;
    session.player_model = false;
    assert!(!Control::PlayerAnimation(Preview::Auto).enabled(&session));
    assert!(Control::PlayerModel.enabled(&session));
}

#[test]
#[ignore = "requires THEIA_PLAYER_ASSETS and an offscreen Vulkan adapter"]
fn synty_player_renders_and_animates_without_changing_the_zone_or_capsule() {
    let path = std::env::var_os("THEIA_PLAYER_ASSETS").expect("Local qualified Synty player art");
    let mut app = App::new();
    install(&mut app, Path::new(&path)).unwrap();
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
    app.insert_resource(session())
        .insert_resource(GlobalAmbientLight {
            brightness: 350.,
            ..default()
        })
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f64(1. / 60.),
        ))
        .add_systems(Update, viewport::sync);
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
    settle(&mut app);
    let feet = app
        .world()
        .resource::<Session>()
        .play
        .as_ref()
        .unwrap()
        .feet;
    let zone = app.world().resource::<Session>().editor.zone().clone();
    app.world_mut().run_system_once(viewport::setup).unwrap();
    let redundant: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, Or<(With<Camera2d>, With<bevy::ui::widget::ViewportNode>)>>()
        .iter(app.world())
        .collect();
    for entity in redundant {
        app.world_mut().despawn(entity);
    }
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            768,
            768,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    let camera = app
        .world_mut()
        .query_filtered::<Entity, With<viewport::ZoneCamera>>()
        .single(app.world())
        .unwrap();
    let centre = Vec3::from_array(feet.metres()) + Vec3::Y;
    app.world_mut().entity_mut(camera).insert((
        bevy::camera::RenderTarget::Image(image.clone().into()),
        Transform::from_translation(centre + Vec3::new(2.5, 1., -4.)).looking_at(centre, Vec3::Y),
    ));
    let start = Instant::now();
    loop {
        app.update();
        assert!(
            app.world().resource::<Avatar>().error.is_none(),
            "{:?}",
            app.world().resource::<Avatar>().error
        );
        if app.world().resource::<Avatar>().active.is_some()
            && start.elapsed() > Duration::from_secs(3)
        {
            break;
        }
        assert!(
            start.elapsed() < Duration::from_secs(60),
            "Player did not become ready"
        );
        std::thread::sleep(Duration::from_millis(16));
    }
    assert_eq!(app.world().resource::<Session>().player_clips.len(), 22);
    assert!(
        !app.world()
            .resource::<Session>()
            .player_clips
            .iter()
            .any(|c| c == "SpellCast")
    );
    let root = app.world().resource::<Avatar>().root.unwrap();
    let actors: Vec<_> = descendants(app.world(), root)
        .into_iter()
        .filter(|e| app.world().get::<AnimationPlayer>(*e).is_some())
        .collect();
    assert_eq!(actors.len(), 1);
    let skins: Vec<_> = app
        .world_mut()
        .query::<&bevy::mesh::skinning::SkinnedMesh>()
        .iter(app.world())
        .map(|skin| skin.joints.len())
        .collect();
    assert!(!skins.is_empty());
    assert!(skins.iter().all(|count| *count == 52), "{skins:?}");
    let pawn = app
        .world_mut()
        .query_filtered::<Entity, With<viewport::Pawn>>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        *app.world().get::<Visibility>(pawn).unwrap(),
        Visibility::Hidden
    );
    let joints = app
        .world_mut()
        .query::<&bevy::mesh::skinning::SkinnedMesh>()
        .iter(app.world())
        .next()
        .unwrap()
        .joints
        .clone();
    let pose = |world: &World| {
        joints
            .iter()
            .map(|e| *world.get::<Transform>(*e).unwrap())
            .collect::<Vec<_>>()
    };
    let idle = pose(app.world());
    app.world_mut().resource_mut::<Session>().player_animation = Preview::SwordAttack;
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<Avatar>().active.as_deref(),
        Some("SwordAttack")
    );
    assert_ne!(pose(app.world()), idle);
    #[derive(Resource, Default)]
    struct Capture(Option<Image>);
    app.init_resource::<Capture>();
    let screenshot = app
        .world_mut()
        .spawn(bevy::render::view::screenshot::Screenshot::image(image))
        .id();
    app.world_mut().entity_mut(screenshot).observe(
        |event: On<bevy::render::view::screenshot::ScreenshotCaptured>,
         mut capture: ResMut<Capture>| {
            capture.0 = Some(event.image.clone());
        },
    );
    if let Some(path) = std::env::var_os("THEIA_PLAYER_CAPTURE") {
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
    let data = app
        .world()
        .resource::<Capture>()
        .0
        .as_ref()
        .unwrap()
        .data
        .as_ref()
        .unwrap();
    assert!(
        data.as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[0].abs_diff(pixel[1]) > 20),
        "No colored character pixels"
    );
    app.world_mut().resource_mut::<Session>().player_animation = Preview::Auto;
    app.world_mut()
        .resource_mut::<Session>()
        .play
        .as_mut()
        .unwrap()
        .velocity = Vec3::X * 7.;
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<Avatar>().active.as_deref(),
        Some("RunRight")
    );
    app.world_mut().resource_mut::<Session>().player_model = false;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(root).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        *app.world().get::<Visibility>(pawn).unwrap(),
        Visibility::Visible
    );
    app.world_mut().resource_mut::<Session>().player_model = true;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(root).unwrap(),
        Visibility::Visible
    );
    assert_eq!(
        app.world()
            .resource::<Session>()
            .play
            .as_ref()
            .unwrap()
            .feet,
        feet
    );
    assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
    assert_eq!(app.world().resource::<Session>().editor.revision(), 0);
    app.world_mut().resource_mut::<Session>().play = None;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(root).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        *app.world().get::<Visibility>(pawn).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        app.world_mut().query::<&Window>().iter(app.world()).count(),
        0
    );
    println!(
        "clips=22 joints=52 animated_sword=true directional_run=true capsule_toggle=true zone_unchanged=true windows=0"
    );
}
