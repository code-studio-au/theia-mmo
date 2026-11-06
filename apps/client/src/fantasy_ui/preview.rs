//! Windowless Bevy GPU image export. This mode never installs Winit or audio.
use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    asset::LoadState,
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        renderer::RenderAdapterInfo,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    text::FontSize,
    window::ExitCondition,
};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Resource)]
struct Capture {
    directory: PathBuf,
    size: UVec2,
    target: Handle<Image>,
    started: Instant,
    ready_frames: u32,
    requested: bool,
}
type ButtonGeometry<'a> = (
    &'a crate::combat::ui::ActionButton,
    &'a ComputedNode,
    &'a bevy::ui::UiGlobalTransform,
);
#[derive(bevy::ecs::system::SystemParam)]
struct Layout<'w, 's> {
    windows: Query<'w, 's, &'static Window>,
    buttons: Query<'w, 's, ButtonGeometry<'static>>,
    text: Query<'w, 's, &'static bevy::text::TextLayoutInfo>,
}
pub(crate) fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() != 6 || args[2] != "--output" || args[4] != "--size" {
        return Err(
            "Expected --preview-fantasy-ui HUD_DIRECTORY --output DIRECTORY --size WIDTHxHEIGHT"
                .into(),
        );
    }
    let mut paths = vec!["--fantasy-ui".into(), args[1].clone()];
    let directory = super::parse(&mut paths, true)?.unwrap();
    let (w, h) = args[5].split_once('x').ok_or("Expected WIDTHxHEIGHT")?;
    let size = UVec2::new(w.parse()?, h.parse()?);
    if !(640..=3840).contains(&size.x) || !(480..=2160).contains(&size.y) {
        return Err("Preview size outside 640x480 to 3840x2160".into());
    }
    let output = PathBuf::from(&args[3]);
    if output.join("hud.png").exists() || output.join("report.json").exists() {
        return Err("Preview output already exists".into());
    }
    std::fs::create_dir_all(&output)?;
    let mut app = App::new();
    super::register(&mut app, directory);
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::audio::AudioPlugin>(),
    )
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        1. / 60.,
    )))
    .insert_resource(ClearColor(Color::srgb(0.10, 0.12, 0.14)));
    super::load(&mut app);
    let target = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.world_mut().spawn((
        Camera2d,
        IsDefaultUiCamera,
        Camera::default(),
        RenderTarget::Image(target.clone().into()),
    ));
    let catalog = theia_core::abilities::Catalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-classic-abilities.json"
    ))?;
    let stats = theia_core::resources::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-character-stats.json"
    ))?;
    app.insert_resource(crate::combat::Config(Arc::new(catalog), Some(stats), true))
        .insert_resource(crate::combat::ui::Model {
            connected: true,
            has_target: true,
            vitals_ready: true,
            player_health: 0.73,
            mana: 0.65,
            health: 0.57,
            player_health_text: "Health 73 / 100".into(),
            mana_text: "Mana 65 / 100".into(),
            life_status: "In combat".into(),
            status: "Face the target".into(),
            actions: vec![
                crate::combat::ui::Action {
                    cooldown: 0.45,
                    status: "Cooldown".into(),
                },
                crate::combat::ui::Action {
                    cooldown: 0.,
                    status: "Ready".into(),
                },
            ],
            ..default()
        })
        .insert_resource(Capture {
            directory: output,
            size,
            target,
            started: Instant::now(),
            ready_frames: 0,
            requested: false,
        });
    crate::combat::ui::install_preview(&mut app);
    app.add_systems(Startup, labels).add_systems(Last, capture);
    app.run();
    Ok(())
}
fn label(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.85, 0.70)),
    )
}
fn labels(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(304),
                top: px(12),
                width: px(276),
                min_height: px(68),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(super::PANEL),
            BorderColor::all(super::GOLD),
        ))
        .with_child(label("Target: Wolf\n57 / 100 health", 18.));
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            left: percent(0),
            top: percent(43),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_child(label(
            "THEIA UI PREVIEW\nFixture values - no live game session",
            20.,
        ));
}
fn capture(
    mut commands: Commands,
    mut run: ResMut<Capture>,
    skin: Res<super::Skin>,
    server: Res<AssetServer>,
    layout: Layout,
    adapter: Res<RenderAdapterInfo>,
) {
    assert!(
        layout.windows.is_empty(),
        "Offscreen preview must never create an OS window"
    );
    assert!(
        run.started.elapsed() < Duration::from_secs(90),
        "Offscreen UI preview timed out"
    );
    for handle in [&skin.frame, &skin.sword, &skin.arcane, &skin.fallback] {
        if let Some(LoadState::Failed(error)) = server.get_load_state(handle.id()) {
            panic!("UI image failed: {error}");
        }
        if !server.is_loaded_with_dependencies(handle.id()) {
            return;
        }
    }
    for font in [&skin.title_font, &skin.body_font] {
        if let Some(LoadState::Failed(error)) = server.get_load_state(font.id()) {
            panic!("UI font failed: {error}");
        }
        if !server.is_loaded_with_dependencies(font.id()) {
            return;
        }
    }
    run.ready_frames += 1;
    if run.requested || run.ready_frames < 60 {
        return;
    }
    let buttons: Vec<_> = layout
        .buttons
        .iter()
        .map(|(slot, node, transform)| {
            let point = transform.affine().translation * node.inverse_scale_factor();
            let size = node.size() * node.inverse_scale_factor();
            assert!(
                size.x > 0. && size.y > 0.,
                "UI must actually lay out on the offscreen camera"
            );
            assert!(
                point.x >= size.x / 2.
                    && point.y >= size.y / 2.
                    && point.x + size.x / 2. <= run.size.x as f32
                    && point.y + size.y / 2. <= run.size.y as f32,
                "Action outside offscreen extent"
            );
            serde_json::json!({"slot":slot.0,"centre":point.to_array(),"size":size.to_array()})
        })
        .collect();
    assert_eq!(buttons.len(), 2);
    let glyphs: usize = layout.text.iter().map(|text| text.glyphs.len()).sum();
    assert!(
        glyphs > 40,
        "Offscreen UI must have real laid-out text glyphs"
    );
    run.requested = true;
    let directory = run.directory.clone();
    let size = run.size;
    let gpu = serde_json::json!({"name":adapter.name,"backend":format!("{:?}",adapter.backend),"device_id":adapter.device});
    commands.spawn(Screenshot::image(run.target.clone())).observe(move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
        assert_eq!([event.image.width(), event.image.height()], [size.x,size.y]);
        event.image.clone().try_into_dynamic().unwrap().save(directory.join("hud.png")).unwrap();
        std::fs::write(directory.join("report.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "completed":true,"offscreen":true,"windows":0,"fixture":true,"extent":[size.x,size.y],"gpu":gpu,"buttons":buttons,"glyphs":glyphs,
            "scope":"Actual Bevy combat HUD widgets with fixture values; no desktop input, account, network, world or live gameplay qualification"
        })).unwrap()).unwrap();
        exit.write(AppExit::Success);
    });
}
