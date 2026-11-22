//! Whole-zone cartography. Immutable presentation data, no terrain requests or gameplay actions.
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    text::FontSize,
    ui_widgets::{Activate, Button},
};
use serde::Deserialize;
use theia_network::procedural::Source;
mod qualification;
mod regions;

const MAP_JSON: &[u8] = include_bytes!("../../../content/zones/millhaven/overview.json");
const MAP_PNG: &[u8] = include_bytes!("../../../content/zones/millhaven/overview.png");
const GOLD: Color = Color::srgb(0.88, 0.73, 0.42);
const PANEL: Color = Color::srgb(0.10, 0.085, 0.065);

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Landmark {
    name: String,
    description: String,
    xz: [f64; 2],
    major: bool,
}
#[derive(Resource, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Data {
    schema: u32,
    title: String,
    terrain_identity: [u8; 32],
    bounds: [[f64; 2]; 2],
    north_positive_z: bool,
    landmarks: Vec<Landmark>,
    #[serde(default)]
    regions: Vec<regions::Region>,
}
impl Data {
    fn project(&self, xz: [f64; 2]) -> Option<Vec2> {
        if !(0..2)
            .all(|i| xz[i].is_finite() && (self.bounds[0][i]..=self.bounds[1][i]).contains(&xz[i]))
        {
            return None;
        }
        let x = (xz[0] - self.bounds[0][0]) / (self.bounds[1][0] - self.bounds[0][0]);
        let z = (xz[1] - self.bounds[0][1]) / (self.bounds[1][1] - self.bounds[0][1]);
        Some(Vec2::new(
            if self.north_positive_z { 1. - x } else { x } as f32,
            if self.north_positive_z { 1. - z } else { z } as f32,
        ))
    }
    fn valid(&self) -> bool {
        self.schema == 2
            && !self.title.is_empty()
            && self.title.len() <= 64
            && (1..=16).contains(&self.landmarks.len())
            && regions::valid(&self.regions)
            && (0..2).all(|i| {
                self.bounds[0][i].is_finite()
                    && self.bounds[1][i].is_finite()
                    && (1. ..=16000.).contains(&(self.bounds[1][i] - self.bounds[0][i]))
            })
            && self.landmarks.iter().all(|p| {
                !p.name.is_empty()
                    && p.name.len() <= 64
                    && p.description.len() <= 256
                    && self.project(p.xz).is_some()
            })
    }
}
#[derive(Resource, Default)]
pub(crate) struct Ui {
    pub open: bool,
    pub closed_this_frame: bool,
    selected: usize,
}
pub(crate) fn blocks_pointer(ui: Option<&Ui>) -> bool {
    ui.is_some_and(|s| s.open || s.closed_this_frame)
}
pub(crate) fn north_positive_z(world: &World) -> bool {
    world
        .get_resource::<Data>()
        .is_some_and(|d| d.north_positive_z)
}

#[derive(Component)]
struct Root;
#[derive(Component)]
struct Surface;
#[derive(Component)]
struct Player;
#[derive(Component)]
struct Pin(usize);
#[derive(Component)]
struct MapLabel(usize);
#[derive(Component)]
struct MapLabelText(usize);
#[derive(Component)]
struct Detail;
#[derive(Component)]
struct PositionText;
#[derive(Component)]
struct RowText(usize);
#[derive(Component, Clone, Copy)]
enum Control {
    Toggle,
    Close,
    Landmark(usize),
}

#[derive(Resource)]
struct MapImage(Image);
pub(crate) fn cartography_pixel(
    world: &World,
    position: theia_core::coordinates::GlobalPosition,
) -> Option<[u8; 4]> {
    let data = world.get_resource::<Data>()?;
    let p = position.metres();
    let uv = data.project([p[0], p[2]])?;
    let image = &world.get_resource::<MapImage>()?.0;
    let x = ((uv.x * image.width() as f32) as usize).min(image.width() as usize - 1);
    let y = ((uv.y * image.height() as f32) as usize).min(image.height() as usize - 1);
    let i = (y * image.width() as usize + x) * 4;
    image.data.as_ref()?.get(i..i + 4)?.try_into().ok()
}
fn decode_map(json: &[u8], png: &[u8]) -> Result<(Data, Image), String> {
    let data: Data = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    if !data.valid() {
        return Err("Invalid cartography metadata".into());
    }
    Ok((data, decode_image(png)?))
}
fn decode_image(png: &[u8]) -> Result<Image, String> {
    if png.len() < 24 || &png[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Invalid cartography metadata or PNG header".into());
    }
    let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
    if !(1..=2048).contains(&width) || !(1..=2048).contains(&height) {
        return Err("Cartography image exceeds 2048 pixels per axis".into());
    }
    let image = Image::from_buffer(
        png,
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::default(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .map_err(|e| e.to_string())?;
    if image
        .data
        .as_ref()
        .is_none_or(|pixels| pixels.len() != width as usize * height as usize * 4)
    {
        return Err("Cartography must decode to RGBA8".into());
    }
    Ok(image)
}
fn local_map(path: &std::path::Path) -> Result<(Data, Image), String> {
    use std::io::Read;
    let read = |name: &str, limit: u64| -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        std::fs::File::open(path.join(name))
            .map_err(|e| e.to_string())?
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > limit {
            return Err("Cartography file exceeds byte budget".into());
        }
        Ok(bytes)
    };
    decode_map(
        &read("overview.json", 65536)?,
        &read("overview.png", 8 * 1024 * 1024)?,
    )
}
pub(crate) fn install(app: &mut App, source: &Source) {
    let Some(store) = source.saved() else {
        return;
    };
    let loaded = if let Some(path) = std::env::var_os("THEIA_ZONE_MAP_DIRECTORY") {
        local_map(std::path::Path::new(&path))
    } else {
        decode_map(MAP_JSON, MAP_PNG)
    };
    let (data, image) = match loaded {
        Ok(value) => value,
        Err(error) => {
            warn!("Zone map unavailable: {error}");
            return;
        }
    };
    if store.identity() != data.terrain_identity || store.bounds() != data.bounds {
        return;
    }
    if let Some(path) = std::env::var_os("THEIA_ZONE_MAP_DIRECTORY") {
        match regions::load(std::path::Path::new(&path), &data) {
            Ok(Some(mask)) => {
                app.insert_resource(mask);
            }
            Ok(None) => (),
            Err(error) => {
                warn!("Zone boundaries unavailable: {error}");
            }
        }
    }
    crate::world_input::install_gestures(app);
    app.insert_resource(data)
        .insert_resource(MapImage(image))
        .init_resource::<Ui>()
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            keyboard
                .after(bevy::input::InputSystems)
                .before(crate::fantasy_ui::settings::keyboard)
                .before(crate::camera::controls)
                .before(crate::input),
        )
        .add_observer(activate)
        .add_systems(PostUpdate, present.before(bevy::ui::UiSystems::Layout));
    qualification::install(app);
}
fn label(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    )
}
fn button(control: Control, name: &str) -> impl Bundle {
    (
        control,
        Button,
        Hovered::default(),
        AccessibleLabel::new(name),
        Node {
            padding: UiRect::axes(px(10), px(7)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.22, 0.18, 0.12)),
    )
}
fn arrow() -> Image {
    let mut rgba = vec![0; 24 * 24 * 4];
    for y in 2..22 {
        for x in 0..24 {
            let dx = (x as i32 - 12).abs();
            if dx <= y / 2 && (y < 16 || dx > y - 16) {
                let color = if dx >= y / 2 - 1 {
                    [30, 25, 18, 255]
                } else {
                    [255, 244, 185, 255]
                };
                let i = (y as usize * 24 + x) * 4;
                rgba[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    Image::new(
        bevy::render::render_resource::Extent3d {
            width: 24,
            height: 24,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}
fn setup(
    mut commands: Commands,
    data: Res<Data>,
    map_image: Res<MapImage>,
    mask: Option<Res<regions::Mask>>,
    mut images: ResMut<Assets<Image>>,
) {
    let terrain = images.add(map_image.0.clone());
    let overlay = mask
        .as_ref()
        .map(|mask| regions::setup(&mut commands, &mut images, mask));
    let player = images.add(arrow());
    commands
        .spawn(button(Control::Toggle, "Open zone map [M]"))
        .insert((
            GlobalZIndex(31),
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: px(12),
                margin: UiRect::left(px(-70)),
                width: px(140),
                height: px(32),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_child(label("Zone map [M]", 15., GOLD));
    commands
        .spawn((
            Root,
            crate::ui_theme::Panel,
            GlobalZIndex(200),
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            BackgroundColor(PANEL),
            Outline::new(px(2), px(-2), GOLD),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    height: px(32),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|header| {
                header.spawn(label(format!("World Map  ·  {}", data.title), 23., GOLD));
                crate::fantasy_ui::kit::disabled(header, "+");
                crate::fantasy_ui::kit::disabled(header, "−");
                header
                    .spawn(button(Control::Close, "Close zone map [M / Esc]"))
                    .with_child(label("Close [M / Esc]", 14., GOLD));
            });
            root.spawn((
                Node {
                    column_gap: px(16),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                body.spawn((
                    Surface,
                    Node {
                        position_type: PositionType::Relative,
                        overflow: Overflow::clip(),
                        flex_shrink: 0.,
                        ..default()
                    },
                    ImageNode::new(terrain),
                ))
                .with_children(|map| {
                    if let Some(overlay) = overlay {
                        map.spawn((
                            ImageNode::new(overlay),
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(100),
                                height: percent(100),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        map.spawn((
                            regions::HoverLabel,
                            label("Hover over a zone to see its boundary", 15., GOLD),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(12),
                                bottom: px(12),
                                padding: UiRect::all(px(6)),
                                ..default()
                            },
                            BackgroundColor(PANEL),
                        ));
                    }
                    map.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(10),
                            top: px(8),
                            padding: UiRect::all(px(5)),
                            ..default()
                        },
                        BackgroundColor(PANEL),
                    ))
                    .with_child(label("N", 16., GOLD));
                    for (i, landmark) in data.landmarks.iter().enumerate() {
                        map.spawn((
                            Pin(i),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(22),
                                height: px(22),
                                border_radius: BorderRadius::MAX,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(PANEL),
                            Outline::new(px(1), px(-1), GOLD),
                            Pickable::IGNORE,
                        ))
                        .with_child(label(
                            format!("{}", i + 1),
                            13.,
                            GOLD,
                        ));
                        if landmark.major {
                            map.spawn((
                                MapLabel(i),
                                Node {
                                    position_type: PositionType::Absolute,
                                    padding: UiRect::axes(px(4), px(2)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.08, 0.06, 0.03, 0.8)),
                                Pickable::IGNORE,
                            ))
                            .with_child((
                                MapLabelText(i),
                                label(&landmark.name, 13., Color::srgb(1., 0.92, 0.72)),
                            ));
                        }
                    }
                    map.spawn((
                        Player,
                        Node {
                            position_type: PositionType::Absolute,
                            width: px(24),
                            height: px(24),
                            ..default()
                        },
                        ImageNode::new(player),
                        ZIndex(10),
                        Pickable::IGNORE,
                    ));
                });
                body.spawn((
                    Node {
                        width: px(220),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(5),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|side| {
                    side.spawn(label(
                        if data.regions.is_empty() {
                            "LANDMARKS"
                        } else {
                            "ZONES"
                        },
                        16.,
                        GOLD,
                    ));
                    for (i, p) in data.landmarks.iter().enumerate() {
                        side.spawn(button(Control::Landmark(i), &p.name))
                            .with_child((
                                RowText(i),
                                label(format!("{}. {}", i + 1, p.name), 13., Color::WHITE),
                            ));
                    }
                    side.spawn((Detail, label("", 13., Color::srgb(0.86, 0.81, 0.69))));
                });
            });
            root.spawn((
                PositionText,
                label("Waiting for your position...", 14., GOLD),
            ));
            root.spawn(label(
                format!(
                    "{:.3} x {:.3} km | {} | Arrow: you",
                    (data.bounds[1][0] - data.bounds[0][0]) / 1000.,
                    (data.bounds[1][1] - data.bounds[0][1]) / 1000.,
                    if data.regions.is_empty() {
                        "Gold: roads | Blue: watercourses"
                    } else {
                        "Hover: zone boundary"
                    }
                ),
                12.,
                Color::srgb(0.75, 0.71, 0.61),
            ));
        });
}

fn keyboard(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut ui: ResMut<Ui>,
    loading: Option<Res<crate::loading::State>>,
) {
    ui.closed_this_frame = false;
    if loading.is_some_and(|s| s.blocks_input()) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && ui.open {
        ui.open = false;
        ui.closed_this_frame = true;
        // The top map owns this Escape; underlying panels/targets retain state.
        keys.clear_just_pressed(KeyCode::Escape);
    } else if keys.just_pressed(KeyCode::KeyM) {
        ui.closed_this_frame = ui.open;
        ui.open = !ui.open;
    }
}
fn activate(
    event: On<Activate>,
    controls: Query<&Control>,
    windows: Query<&Window>,
    loading: Option<Res<crate::loading::State>>,
    mut ui: ResMut<Ui>,
    data: Res<Data>,
) {
    if loading.is_some_and(|s| s.blocks_input()) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    match controls.get(event.entity) {
        Ok(Control::Toggle) => {
            ui.closed_this_frame = ui.open;
            ui.open = !ui.open;
        }
        Ok(Control::Close) if ui.open => {
            ui.open = false;
            ui.closed_this_frame = true;
        }
        Ok(Control::Landmark(i)) if ui.open && *i < data.landmarks.len() => ui.selected = *i,
        _ => (),
    }
}
fn layout(size: Vec2, aspect: f32) -> (Vec2, Vec2) {
    let map_width = (size.x - 284.)
        .min((size.y - 152.) * aspect)
        .clamp(160., 1024.);
    let map = Vec2::new(map_width, map_width / aspect);
    (map, Vec2::new(map.x + 268., map.y.max(320.) + 136.))
}
fn present(world: &mut World) {
    let data = world.resource::<Data>().clone();
    let ui = world.resource::<Ui>();
    let (open, selected) = (ui.open, ui.selected);
    let loading = world
        .get_resource::<crate::loading::State>()
        .is_some_and(|s| s.blocks_input());
    let state = world.resource::<crate::network::Network>().state();
    let position = crate::minimap::owner_position(world, &state);
    let heading = crate::minimap::player_heading(world, &state);
    let visible = open && position.is_some() && !loading;
    regions::present(world, visible);
    let scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |s| s.0);
    let size = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map_or(Vec2::new(1280., 720.), |w| {
            Vec2::new(w.width(), w.height()) / scale
        });
    let (map, panel) = layout(
        size,
        ((data.bounds[1][0] - data.bounds[0][0]) / (data.bounds[1][1] - data.bounds[0][1])) as f32,
    );
    for mut node in world
        .query_filtered::<&mut Node, With<Root>>()
        .iter_mut(world)
    {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        node.width = px(panel.x);
        node.height = px(panel.y);
        node.left = px((size.x - panel.x) * 0.5);
        node.top = px((size.y - panel.y) * 0.5);
    }
    if !visible {
        return;
    }
    for mut node in world
        .query_filtered::<&mut Node, With<Surface>>()
        .iter_mut(world)
    {
        node.width = px(map.x);
        node.height = px(map.y);
    }
    let p = position.unwrap().metres();
    let point = data.project([p[0], p[2]]);
    for (mut node, mut transform) in world
        .query_filtered::<(&mut Node, &mut UiTransform), With<Player>>()
        .iter_mut(world)
    {
        node.display = if point.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(q) = point {
            let q = q * map;
            node.left = px(q.x - 12.);
            node.top = px(q.y - 12.);
        }
        transform.rotation = crate::minimap::heading_rotation(heading, data.north_positive_z);
    }
    for (pin, mut node, mut color) in world
        .query::<(&Pin, &mut Node, &mut BackgroundColor)>()
        .iter_mut(world)
    {
        let landmark = &data.landmarks[pin.0];
        let q = data.project(landmark.xz).unwrap() * map;
        node.display = if landmark.major || pin.0 == selected {
            Display::Flex
        } else {
            Display::None
        };
        node.left = px(q.x - 11.);
        node.top = px(q.y - 11.);
        color.0 = if pin.0 == selected {
            Color::srgb(0.48, 0.25, 0.08)
        } else {
            PANEL
        };
    }
    for (row, mut text) in world.query::<(&RowText, &mut Text)>().iter_mut(world) {
        let l = &data.landmarks[row.0];
        **text = format!("{}. {}", row.0 + 1, l.name);
    }
    for (label, mut node) in world.query::<(&MapLabel, &mut Node)>().iter_mut(world) {
        let q = data.project(data.landmarks[label.0].xz).unwrap() * map;
        node.left = px(q.x + 14.);
        node.top = px(q.y - 11.);
    }
    for (label, mut text) in world.query::<(&MapLabelText, &mut Text)>().iter_mut(world) {
        let name = match (map.x < 600., data.landmarks[label.0].name.as_str()) {
            (true, "Southeast farm") => "SE farm",
            (true, "Southwest farm") => "SW farm",
            _ => &data.landmarks[label.0].name,
        };
        if text.0 != name {
            text.0 = name.to_owned();
        }
    }
    for (control, mut color) in world
        .query::<(&Control, &mut BackgroundColor)>()
        .iter_mut(world)
    {
        if let Control::Landmark(i) = control {
            color.0 = if *i == selected {
                Color::srgb(0.38, 0.28, 0.13)
            } else {
                Color::srgb(0.18, 0.15, 0.10)
            };
        }
    }
    for mut text in world
        .query_filtered::<&mut Text, With<Detail>>()
        .iter_mut(world)
    {
        let l = &data.landmarks[selected];
        **text = format!(
            "{:.0} m from you\nX {:.0} | Z {:.0}\n{}",
            (p[0] - l.xz[0]).hypot(p[2] - l.xz[1]),
            l.xz[0],
            l.xz[1],
            l.description
        );
    }
    for mut text in world
        .query_filtered::<&mut Text, With<PositionText>>()
        .iter_mut(world)
    {
        let centre = &data.landmarks[0];
        **text = format!(
            "You: X {:.0}, Z {:.0}  |  {}: {:.0} m away",
            p[0],
            p[2],
            centre.name,
            (p[0] - centre.xz[0]).hypot(p[2] - centre.xz[1])
        );
    }
}

#[cfg(test)]
mod tests;
