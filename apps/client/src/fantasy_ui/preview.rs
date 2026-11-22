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
    ui_scale: f32,
    preferred_scale: f32,
    reader: bool,
    target: Handle<Image>,
    started: Instant,
    ready_frames: u32,
    requested: bool,
}
#[derive(Component)]
struct ProjectedAnchor;
#[derive(Component)]
struct HeaderFixture;
const ANCHOR_RATIO: Vec2 = Vec2::new(0.73, 0.35);
type AnchorGeometry<'a> = (&'a ComputedNode, &'a bevy::ui::UiGlobalTransform);
type FeedbackGeometry<'a> = (
    &'a ComputedNode,
    &'a bevy::ui::UiGlobalTransform,
    Option<&'a crate::combat::ui::ActionLabel>,
    Has<crate::combat::ui::CastLabel>,
);
type ButtonGeometry<'a> = (
    &'a crate::combat::ui::ActionButton,
    &'a ComputedNode,
    &'a bevy::ui::UiGlobalTransform,
);
type UnitGeometry<'a> = (
    &'a super::UnitFrame,
    &'a ComputedNode,
    &'a bevy::ui::UiGlobalTransform,
);
type PaperSample = Or<(With<HeaderFixture>, With<crate::documents::Pane>)>;
type ReaderButtons = Or<(
    With<crate::documents::NextButton>,
    With<crate::documents::PreviousButton>,
    With<crate::documents::CloseButton>,
)>;
type BagActions = Or<(
    With<crate::inventory::EquipButton>,
    With<crate::inventory::UnequipButton>,
    With<crate::inventory::SplitButton>,
    With<crate::inventory::CancelButton>,
    With<crate::inventory::DestroyButton>,
)>;
#[derive(bevy::ecs::system::SystemParam)]
struct Layout<'w, 's> {
    windows: Query<'w, 's, &'static Window>,
    buttons: Query<'w, 's, ButtonGeometry<'static>>,
    text: Query<'w, 's, &'static bevy::text::TextLayoutInfo>,
    feedback: Query<'w, 's, FeedbackGeometry<'static>>,
    anchor: Query<'w, 's, AnchorGeometry<'static>, With<ProjectedAnchor>>,
    minimap: Query<'w, 's, AnchorGeometry<'static>, With<crate::minimap::Root>>,
    interface: Query<'w, 's, AnchorGeometry<'static>, With<super::settings::Pane>>,
    map_tooltip: Query<'w, 's, AnchorGeometry<'static>, With<crate::minimap::hints::Tooltip>>,
    map_surface: Query<'w, 's, AnchorGeometry<'static>, With<crate::minimap::Surface>>,
    header: Query<'w, 's, AnchorGeometry<'static>, PaperSample>,
    units: Query<'w, 's, UnitGeometry<'static>>,
    xp_popup: Query<'w, 's, AnchorGeometry<'static>, With<crate::progression::feedback::Label>>,
    xp: Query<'w, 's, AnchorGeometry<'static>, With<crate::progression::Root>>,
    dock: Query<'w, 's, AnchorGeometry<'static>, With<crate::inventory::BackpackDock>>,
    reward_icons: Query<
        'w,
        's,
        (
            &'static super::RewardIcon,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
            &'static Children,
        ),
    >,
    reward_text: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static bevy::text::TextLayoutInfo,
            &'static Text,
        ),
    >,
    reader_body: Query<'w, 's, AnchorGeometry<'static>, With<crate::documents::BodyScroll>>,
    reader_text: Query<
        'w,
        's,
        (
            &'static Text,
            &'static bevy::text::TextLayoutInfo,
            &'static TextColor,
        ),
        With<crate::documents::BodyText>,
    >,
    reader_buttons: Query<'w, 's, AnchorGeometry<'static>, ReaderButtons>,
    bag_actions: Query<'w, 's, AnchorGeometry<'static>, BagActions>,
    bag_slots: Query<'w, 's, AnchorGeometry<'static>, With<crate::inventory::SlotButton>>,
    bag_icons:
        Query<'w, 's, (&'static Node, &'static ComputedNode), With<crate::inventory::skin::Icon>>,
    bag_tooltip: Query<'w, 's, AnchorGeometry<'static>, With<crate::inventory::skin::Tooltip>>,
    plates: Query<
        'w,
        's,
        (
            &'static Node,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
        ),
        With<crate::nameplates::Root>,
    >,
}
pub(crate) fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !(6..=9).contains(&args.len()) || args[2] != "--output" || args[4] != "--size" {
        return Err(
            "Expected --preview-fantasy-ui HUD_DIRECTORY --output DIRECTORY --size WIDTHxHEIGHT [--ui-scale FACTOR] [--reader | --interface | --graphics]"
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
    let mut scale_args = args[6..].to_vec();
    let reader_flags = scale_args
        .iter()
        .filter(|arg| arg.as_str() == "--reader")
        .count();
    let interface_flags = scale_args
        .iter()
        .filter(|a| a.as_str() == "--interface")
        .count();
    let graphics_flags = scale_args
        .iter()
        .filter(|a| a.as_str() == "--graphics")
        .count();
    if reader_flags + interface_flags + graphics_flags > 1 {
        return Err("Choose at most one --reader, --interface or --graphics preview flag".into());
    }
    let reader = reader_flags == 1;
    let interface = interface_flags == 1 || graphics_flags == 1;
    scale_args.retain(|arg| arg != "--reader" && arg != "--interface" && arg != "--graphics");
    let preferred_scale = super::parse_scale(&mut scale_args, true)?;
    let ui_scale = super::fit_scale(preferred_scale, size.as_vec2());
    if !scale_args.is_empty() {
        return Err("Unexpected preview arguments".into());
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
    app.insert_resource(bevy::ui::UiScale(ui_scale));
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
            player_level: Some(2),
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
            ui_scale,
            preferred_scale,
            reader,
            target,
            started: Instant::now(),
            ready_frames: 0,
            requested: false,
        });
    crate::inventory::skin::install_preview(&mut app);
    if reader {
        crate::documents::install_preview(&mut app);
    }
    if interface {
        super::settings::install_preview(&mut app, preferred_scale);
        app.world_mut()
            .resource_mut::<super::settings::Ui>()
            .graphics_open = graphics_flags == 1;
    }
    crate::combat::ui::install_preview(&mut app);
    crate::minimap::install_preview(&mut app);
    crate::progression::install_preview(&mut app);
    crate::progression::feedback::install_preview(
        &mut app,
        size.as_vec2() * Vec2::new(0.5, 0.45) / ui_scale,
    );
    crate::nameplates::install_preview(&mut app, size.as_vec2() * Vec2::new(0.5, 0.55) / ui_scale);
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
fn labels(mut commands: Commands, run: Res<Capture>, skin: Res<super::Skin>) {
    if !run.reader {
        // Validated metadata for a labelled UI sample; never a live quest/reward.
        let items = theia_core::inventory::Catalog::from_json(include_bytes!(
            "../../../../content/gameplay/development-document-items.json"
        ))
        .unwrap();
        let mut metadata: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-rewards.json"
        ))
        .unwrap();
        metadata["rewards"][0]["grants"] = serde_json::json!([
            {"item":"theia.dev.training-token","count":3},
            {"item":"theia.dev.training-sword","count":1}
        ]);
        let rewards = theia_core::rewards::Catalog::from_json(
            &serde_json::to_vec(&metadata).unwrap(),
            &items,
        )
        .unwrap();
        let grants = rewards.definitions().next().unwrap().grants();
        commands
            .spawn((
                HeaderFixture,
                super::Panel::Paper,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(12),
                    top: px(220),
                    width: px(276),
                    padding: UiRect::all(px(12)),
                    row_gap: px(6),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(super::PANEL),
            ))
            .with_children(|panel| {
                super::window_header(
                    panel,
                    label("Window header (fixture)", 18.),
                    (
                        bevy::ui_widgets::Button,
                        bevy::picking::hover::Hovered::default(),
                        BackgroundColor(Color::srgb(0.08, 0.12, 0.16)),
                    ),
                    "Close fixture window",
                );
                panel.spawn(label(
                    "Rewards (fixture)\n3 x Training Token, 1 x Training Sword",
                    14.,
                ));
                // Deliberately narrow: exercise wrapping as well as the normal
                // full-width offer layout, without inventing extra rewards.
                panel
                    .spawn(Node {
                        width: px(140),
                        ..default()
                    })
                    .with_children(|row| super::reward_icons(row, &skin, grants));
            });
    }
    let point = super::projected_ui_point(run.size.as_vec2() * ANCHOR_RATIO, run.ui_scale).unwrap();
    commands
        .spawn((
            ProjectedAnchor,
            Node {
                position_type: PositionType::Absolute,
                left: px(point.x - 5.),
                top: px(point.y - 5.),
                width: px(10),
                height: px(10),
                ..default()
            },
            BackgroundColor(super::GOLD),
            bevy::picking::Pickable::IGNORE,
        ))
        .with_child((
            label("Projected anchor (fixture)", 14.),
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                top: px(-4),
                width: px(220),
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
        ));
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
    map_metrics: Res<crate::minimap::Metrics>,
) {
    assert!(
        layout.windows.is_empty(),
        "Offscreen preview must never create an OS window"
    );
    assert!(
        run.started.elapsed() < Duration::from_secs(90),
        "Offscreen UI preview timed out"
    );
    for handle in [&skin.sword, &skin.arcane, &skin.fallback]
        .into_iter()
        .chain(skin.item_icons.iter())
    {
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
            let point = transform.affine().translation;
            let size = node.size();
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
            assert!(
                (size.x - super::SLOT * run.ui_scale).abs() <= 1.
                    && (size.y - super::SLOT * run.ui_scale).abs() <= 1.,
                "Action must use the selected Bevy UI scale"
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
    let (anchor_node, anchor_transform) = layout.anchor.single().expect("Projected anchor fixture");
    assert!(anchor_node.size().min_element() > 0.);
    let projected_anchor = anchor_transform.affine().translation;
    let expected_anchor = run.size.as_vec2() * ANCHOR_RATIO;
    assert!(
        projected_anchor.distance(expected_anchor) <= 1.5,
        "Scaled label must retain its window-space world anchor"
    );
    let (map_node, map_transform) = layout
        .minimap
        .single()
        .expect("Actual minimap widget fixture");
    let map_size = map_node.size();
    let map_centre = map_transform.affine().translation;
    assert!(map_size.min_element() > 0.);
    assert!(
        map_centre.x - map_size.x * 0.5 >= 0.
            && map_centre.y - map_size.y * 0.5 >= 0.
            && map_centre.x + map_size.x * 0.5 <= run.size.x as f32
            && map_centre.y + map_size.y * 0.5 <= run.size.y as f32,
        "Minimap must remain within the rendered viewport"
    );
    let (surface_node, surface_transform) = layout.map_surface.single().expect("Map image surface");
    let surface_size = surface_node.size();
    let surface_centre = surface_transform.affine().translation;
    assert!(
        surface_node
            .border_radius()
            .top_left
            .distance(Vec2::splat(surface_size.x * 0.5))
            <= 1.,
        "Map image must use standard Bevy circular corners"
    );
    let (tip_node, tip_transform) = layout.map_tooltip.single().expect("Map tooltip fixture");
    let tip_size = tip_node.size();
    let tip_centre = tip_transform.affine().translation;
    assert!(
        tip_size.min_element() > 0.
            && tip_centre.x - tip_size.x * 0.5 >= 0.
            && tip_centre.y - tip_size.y * 0.5 >= 0.
            && tip_centre.x + tip_size.x * 0.5 <= run.size.x as f32
            && tip_centre.y + tip_size.y * 0.5 <= run.size.y as f32,
        "Map tooltip stays inside viewport"
    );
    assert!(
        tip_centre.x + tip_size.x * 0.5 <= map_centre.x - map_size.x * 0.5,
        "Tooltip leaves map visible"
    );
    let minimap = serde_json::json!({"centre":map_centre.to_array(),"size":map_size.to_array(),"surface":{"centre":surface_centre.to_array(),"size":surface_size.to_array(),"shape":"circle"},"tooltip":{"centre":tip_centre.to_array(),"size":tip_size.to_array(),"text":map_metrics.hover_text},"raster":"synthetic checker fixture; no world data"});
    let (header_node, header_transform) = layout
        .header
        .single()
        .expect("Actual shared title-row fixture");
    let header_size = header_node.size();
    let header_centre = header_transform.affine().translation;
    assert!(
        header_size.min_element() > 0.
            && header_centre.x - header_size.x * 0.5 >= 0.
            && header_centre.y - header_size.y * 0.5 >= 0.
            && header_centre.x + header_size.x * 0.5 <= run.size.x as f32
            && header_centre.y + header_size.y * 0.5 <= run.size.y as f32,
        "Shared window header must remain in the viewport"
    );
    let header = serde_json::json!({"centre":header_centre.to_array(),"size":header_size.to_array(),"fixture":true});
    let mut cast = None;
    let mut tooltips = Vec::new();
    for (node, transform, tooltip, is_cast) in &layout.feedback {
        let size = node.size();
        if size.min_element() <= 0. {
            continue;
        }
        let centre = transform.affine().translation;
        let rect = Rect::from_center_size(centre, size);
        if is_cast {
            cast = Some(rect);
        }
        if tooltip.is_some() {
            tooltips.push(rect);
        }
    }
    let cast = cast.expect("Cast/rejection feedback must be laid out");
    assert!(
        !tooltips.is_empty(),
        "The preview must display the hover description fixture"
    );
    for tooltip in &tooltips {
        assert!(
            tooltip.intersect(cast).is_empty(),
            "Tooltip must not obscure cast/rejection feedback"
        );
    }
    let feedback = serde_json::json!({"cast":[cast.min.to_array(),cast.max.to_array()],"tooltips":tooltips.iter().map(|r| [r.min.to_array(),r.max.to_array()]).collect::<Vec<_>>()});
    let viewport = Rect::from_corners(Vec2::ZERO, run.size.as_vec2());
    let centre_x = run.size.x as f32 * 0.5;
    let interface: Vec<_> = layout
        .interface
        .iter()
        .map(|(node, transform)| {
            let rect = Rect::from_center_size(transform.affine().translation, node.size());
            assert!(
                node.size().min_element() > 0.
                    && viewport.contains(rect.min)
                    && viewport.contains(rect.max),
                "Interface settings stays in viewport"
            );
            assert!(
                rect.intersect(cast).is_empty(),
                "Settings leaves combat feedback visible"
            );
            serde_json::json!({"fixture":true,"rect":[rect.min.to_array(),rect.max.to_array()]})
        })
        .collect();
    let unit_frames: Vec<_> = layout
        .units
        .iter()
        .map(|(side, node, transform)| {
            let rect = Rect::from_center_size(transform.affine().translation, node.size());
            assert!(
                node.size().min_element() > 0.
                    && viewport.contains(rect.min)
                    && viewport.contains(rect.max)
            );
            assert!(
                if side.0 {
                    rect.max.x < centre_x
                } else {
                    rect.min.x > centre_x
                },
                "Unit frames must flank the central world view"
            );
            assert!(
                rect.intersect(cast).is_empty(),
                "Unit frame must not cover casting feedback"
            );
            for tooltip in &tooltips {
                assert!(
                    rect.intersect(*tooltip).is_empty(),
                    "Tooltip must not obscure unit frames"
                );
            }
            serde_json::json!({"player":side.0,"rect":[rect.min.to_array(),rect.max.to_array()]})
        })
        .collect();
    assert_eq!(unit_frames.len(), 2);
    let (xp_node, xp_transform) = layout.xp.single().expect("Actual progression widgets");
    let xp_rect = Rect::from_center_size(xp_transform.affine().translation, xp_node.size());
    let (popup_node, popup_transform) = layout.xp_popup.single().expect("Shared XP popup fixture");
    let popup_rect =
        Rect::from_center_size(popup_transform.affine().translation, popup_node.size());
    assert!(
        popup_node.size().min_element() > 0.
            && viewport.contains(popup_rect.min)
            && viewport.contains(popup_rect.max),
        "XP popup stays inside viewport"
    );
    assert!(
        popup_rect.intersect(cast).is_empty() && popup_rect.intersect(xp_rect).is_empty(),
        "XP popup leaves feedback and experience bar clear"
    );
    let xp_popup = serde_json::json!({"fixture":true,"rect":[popup_rect.min.to_array(),popup_rect.max.to_array()],"amount":25});

    assert!(
        xp_node.size().min_element() > 0.
            && viewport.contains(xp_rect.min)
            && viewport.contains(xp_rect.max)
    );
    assert!(
        xp_rect.intersect(cast).is_empty(),
        "XP must not obscure casting feedback"
    );
    let xp =
        serde_json::json!({"rect":[xp_rect.min.to_array(),xp_rect.max.to_array()],"fixture":true});
    let (dock_node, dock_transform) = layout.dock.single().expect("Actual backpack toggle widget");
    let dock_rect = Rect::from_center_size(dock_transform.affine().translation, dock_node.size());
    assert!(viewport.contains(dock_rect.min) && viewport.contains(dock_rect.max));
    for tooltip in &tooltips {
        assert!(
            dock_rect.intersect(*tooltip).is_empty(),
            "Tooltip must not cover backpack access"
        );
    }
    for (_, node, transform) in &layout.buttons {
        let button_rect = Rect::from_center_size(transform.affine().translation, node.size());
        assert!(
            dock_rect.intersect(button_rect).is_empty(),
            "Backpack access must not cover actions"
        );
    }
    let dock = serde_json::json!({"rect":[dock_rect.min.to_array(),dock_rect.max.to_array()],"fixture":true});
    let nameplates: Vec<_> = layout
        .plates
        .iter()
        .filter(|(node, _, _)| node.display != Display::None)
        .map(|(_, node, transform)| {
            let rect = Rect::from_center_size(transform.affine().translation, node.size());
            assert!(
                node.size().min_element() > 0.
                    && viewport.contains(rect.min)
                    && viewport.contains(rect.max)
            );
            serde_json::json!({"rect":[rect.min.to_array(),rect.max.to_array()],"fixture":true})
        })
        .collect();
    assert_eq!(
        nameplates.len(),
        1,
        "One real nameplate widget with fixture data"
    );
    run.requested = true;
    let bag_slots: Vec<_> = layout
        .bag_slots
        .iter()
        .map(|(node, transform)| {
            let size = node.size();
            let centre = transform.affine().translation;
            assert!(size.min_element() > 0.);
            assert!(
                centre.x - size.x / 2. >= 0.
                    && centre.y - size.y / 2. >= 0.
                    && centre.x + size.x / 2. <= run.size.x as f32
                    && centre.y + size.y / 2. <= run.size.y as f32
            );
            serde_json::json!({"centre":centre.to_array(), "size":size.to_array()})
        })
        .collect();
    assert_eq!(bag_slots.len(), 16);
    let visible_icons = layout
        .bag_icons
        .iter()
        .filter(|(node, computed)| {
            node.display != Display::None && computed.size().min_element() > 0.
        })
        .count();
    assert_eq!(visible_icons, 6, "Only occupied fixture slots show art");
    let (tooltip_node, tooltip_transform) = layout.bag_tooltip.single().unwrap();
    let tooltip_size = tooltip_node.size();
    let tooltip_centre = tooltip_transform.affine().translation;
    let tooltip_rect = Rect::from_center_size(tooltip_centre, tooltip_size);
    assert!(
        tooltip_size.min_element() > 0.
            && tooltip_rect.min.min_element() >= 0.
            && tooltip_rect.max.x <= run.size.x as f32
            && tooltip_rect.max.y <= run.size.y as f32
    );
    assert!(
        tooltip_rect.intersect(xp_rect).is_empty(),
        "Item tooltip must not cover XP"
    );
    assert!(
        tooltip_rect.intersect(cast).is_empty(),
        "Item tooltip must not cover cast feedback"
    );
    for (_, node, transform) in &layout.units {
        assert!(
            tooltip_rect
                .intersect(Rect::from_center_size(
                    transform.affine().translation,
                    node.size()
                ))
                .is_empty(),
            "Item tooltip must not cover unit frames"
        );
    }
    for (_, node, transform) in &layout.buttons {
        assert!(
            tooltip_rect
                .intersect(Rect::from_center_size(
                    transform.affine().translation,
                    node.size()
                ))
                .is_empty(),
            "Item tooltip must not cover combat actions"
        );
    }
    for (node, transform) in &layout.bag_slots {
        assert!(
            tooltip_rect
                .intersect(Rect::from_center_size(
                    transform.affine().translation,
                    node.size()
                ))
                .is_empty(),
            "Item tooltip must not cover bag slots"
        );
    }
    let bag_actions: Vec<_> = layout
        .bag_actions
        .iter()
        .map(|(node, transform)| {
            let rect = Rect::from_center_size(transform.affine().translation, node.size());
            assert!(
                node.size().min_element() > 0.
                    && rect.min.min_element() >= 0.
                    && rect.max.x <= run.size.x as f32
                    && rect.max.y <= run.size.y as f32,
                "Backpack actions must fit the windowless fixture"
            );
            serde_json::json!([rect.min.to_array(), rect.max.to_array()])
        })
        .collect();
    assert_eq!(bag_actions.len(), 5);
    let backpack = serde_json::json!({"fixture":true, "slots":bag_slots,"actions":bag_actions,"visible_icons":visible_icons,"tooltip":{"centre":tooltip_centre.to_array(),"size":tooltip_size.to_array()}});
    let mut reward_rows = Vec::<f32>::new();
    let reward_icons: Vec<_> = layout.reward_icons.iter().map(|(reward, node, transform, children)| {
        let rect = Rect::from_center_size(transform.affine().translation, node.size());
        if !reward_rows.iter().any(|y| (*y - rect.min.y).abs() < 1.) { reward_rows.push(rect.min.y); }
        let header_rect = Rect::from_center_size(header_centre, header_size);
        assert!(node.size().min_element() > 0. && header_rect.contains(rect.min) && header_rect.contains(rect.max));
        let mut text_seen = false;
        for child in children.iter() {
            if let Ok((node, glyphs, text)) = layout.reward_text.get(child) {
                assert_eq!(text.0, reward.count.to_string());
                assert!(glyphs.size.x <= node.size().x + 1. && glyphs.size.y <= node.size().y + 1., "Reward count must fit its rendered cell");
                text_seen = true;
            }
        }
        assert!(text_seen);
        serde_json::json!({"item":reward.item.as_str(),"count":reward.count,"rect":[rect.min.to_array(),rect.max.to_array()]})
    }).collect();
    assert_eq!(reward_icons.len(), if run.reader { 0 } else { 2 });
    assert_eq!(
        reward_rows.len(),
        if run.reader { 0 } else { 2 },
        "Reward cells must wrap in the narrow fixture"
    );
    let reader = if run.reader {
        let pane_rect = Rect::from_center_size(header_centre, header_size);
        assert!(
            header_size.x <= 420. * run.ui_scale + 1.
                && header_size.y <= super::PAPER_MAX_HEIGHT * run.ui_scale + 1.
        );
        for (_, node, transform) in &layout.units {
            assert!(
                pane_rect
                    .intersect(Rect::from_center_size(
                        transform.affine().translation,
                        node.size()
                    ))
                    .is_empty(),
                "Reader must leave the lower combat frames visible"
            );
        }
        let (body_node, body_transform) = layout.reader_body.single().unwrap();
        let body_rect =
            Rect::from_center_size(body_transform.affine().translation, body_node.size());
        assert!(
            body_node.size().min_element() > 0.
                && pane_rect.contains(body_rect.min)
                && pane_rect.contains(body_rect.max)
        );
        let (body_text, glyphs, color) = layout.reader_text.single().unwrap();
        assert!(
            body_text.0.starts_with("Keep watch on the eastern road.") && !glyphs.glyphs.is_empty()
        );
        assert_eq!(color.0, super::INK);
        assert!(
            glyphs.size.y <= body_node.size().y + 1.,
            "The short first-page fixture should fit without scrolling"
        );
        let controls: Vec<_> = layout
            .reader_buttons
            .iter()
            .map(|(node, transform)| {
                let rect = Rect::from_center_size(transform.affine().translation, node.size());
                assert!(
                    node.size().min_element() > 0.
                        && pane_rect.contains(rect.min)
                        && pane_rect.contains(rect.max)
                );
                assert!(
                    rect.intersect(body_rect).is_empty(),
                    "Page/close controls must stay outside document scrolling"
                );
                serde_json::json!([rect.min.to_array(), rect.max.to_array()])
            })
            .collect();
        assert_eq!(controls.len(), 3);
        serde_json::json!({"fixture":true,"body":[body_rect.min.to_array(),body_rect.max.to_array()],"controls":controls,"glyphs":glyphs.glyphs.len()})
    } else {
        serde_json::Value::Null
    };
    assert_eq!(map_metrics.objective_regions, 1);
    assert!(map_metrics.objective_pixels > 0);
    let map_objectives = serde_json::json!({"fixture":true,"regions":map_metrics.objective_regions,"tinted_pixels":map_metrics.objective_pixels});
    let directory = run.directory.clone();
    let size = run.size;
    let ui_scale = run.ui_scale;
    let preferred_scale = run.preferred_scale;
    let gpu = serde_json::json!({"name":adapter.name,"backend":format!("{:?}",adapter.backend),"device_id":adapter.device});
    commands.spawn(Screenshot::image(run.target.clone())).observe(move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
        assert_eq!([event.image.width(), event.image.height()], [size.x,size.y]);
        event.image.clone().try_into_dynamic().unwrap().save(directory.join("hud.png")).unwrap();
        std::fs::write(directory.join("report.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "completed":true,"ui_scale":ui_scale,"preferred_ui_scale":preferred_scale,"geometry_units":"physical_pixels","offscreen":true,"windows":0,"fixture":true,"extent":[size.x,size.y],"gpu":gpu,"buttons":buttons,"glyphs":glyphs,"feedback":feedback,"minimap":minimap,"header":header,"unit_frames":unit_frames,"xp":xp,"xp_popup":xp_popup,"backpack_dock":dock,"backpack":backpack,"reward_icons":reward_icons,"reader":reader,"map_objectives":map_objectives,"interface":interface,"nameplates":nameplates,"projected_anchor":{"actual":projected_anchor.to_array(),"expected":expected_anchor.to_array()},
            "scope":"Actual Bevy combat HUD widgets with fixture values; no desktop input, account, network, world or live gameplay qualification"
        })).unwrap()).unwrap();
        exit.write(AppExit::Success);
    });
}
