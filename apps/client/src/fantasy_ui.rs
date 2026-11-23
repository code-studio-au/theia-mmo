//! Native game presentation using the approved regenerated Theia artwork.
//! The legacy HUD source/CLI remains compatible with retained playtest launchers.
mod debug;
mod developer;
pub(crate) mod display;
pub(crate) mod graphics;
pub(crate) mod kit;
pub(crate) mod preferences;
pub(crate) mod preview;
pub(crate) mod settings;
use bevy::{
    asset::{
        AssetApp,
        io::{AssetSourceBuilder, file::FileAssetReader},
    },
    picking::Pickable,
    prelude::*,
};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub(crate) const SWORD: &str = "Sprites/Icons_Weapons/ICON_SM_Wep_Sword_07_Clean.png";
pub(crate) const ARCANE: &str =
    "Sprites/Icons_Elements/ICON_FantasyWarrior_Element_Air02_Clean.png";
pub(crate) const FALLBACK: &str =
    "Sprites/Icons_Inventory/ICON_FantasyWarrior_Inventory_Spellbooks01_Clean.png";
const PANEL_FRAME: &str = "Sprites/HUD/SPR_HUD_FantasyWarrior_Frame_Box_Medium04.png";
const ITEM_ICONS: [&str; 6] = [
    "Sprites/Icons_Resources/ICON_SM_Item_Rock_01.png",
    "Sprites/Icons_Inventory/ICON_FantasyWarrior_Inventory_Currency01_Clean.png",
    "Sprites/Icons_Weapons/ICON_SM_Wep_Sword_07.png",
    "Sprites/Icons_Weapons/ICON_SM_Wep_Sceptre_01.png",
    "Sprites/Icons_Inventory/ICON_FantasyWarrior_Inventory_Crafting01_Clean.png",
    "Sprites/Icons_Resources/ICON_SM_Item_Parchment_01.png",
];
pub(crate) const FILES: [&str; 10] = [
    SWORD,
    ARCANE,
    FALLBACK,
    ITEM_ICONS[0],
    ITEM_ICONS[1],
    ITEM_ICONS[2],
    ITEM_ICONS[3],
    ITEM_ICONS[4],
    ITEM_ICONS[5],
    PANEL_FRAME,
];
const IMAGE_DIMENSIONS: [(u32, u32); FILES.len()] = [
    (512, 256),
    (256, 256),
    (256, 256),
    (1024, 1024),
    (256, 256),
    (1024, 1024),
    (1024, 1024),
    (256, 256),
    (1024, 1024),
    (256, 256),
];
pub(crate) const TITLE_FONT: &str = "Fonts/Grenze/Grenze-SemiBold.ttf";
pub(crate) const BODY_FONT: &str = "Fonts/Texturina/Texturina_18pt-SemiBold.ttf";
pub(crate) const GOLD: Color = crate::ui_theme::GOLD;
pub(crate) const PANEL: Color = crate::ui_theme::PANEL;
const PAPER: Color = PANEL;
const INK: Color = crate::ui_theme::CREAM;
#[derive(Component)]
pub(crate) struct DarkSurface;
pub(crate) fn button_color(disabled: bool, hovered: bool, pressed: bool) -> Color {
    if disabled {
        Color::srgb(0.16, 0.14, 0.12)
    } else if pressed {
        Color::srgb(0.30, 0.22, 0.10)
    } else if hovered {
        Color::srgb(0.06, 0.22, 0.30)
    } else {
        Color::srgb(0.025, 0.11, 0.15)
    }
}
/// Computed UI coordinates are physical target pixels; inverse_scale_factor
/// removes BOTH target DPI and UiScale (Bevy 0.20 propagate_ui_target_cameras).
/// OS pointer telemetry needs window logical pixels, so retain UI enlargement.
/// ScrollPosition remains in UI logical pixels and must not use this factor.
pub(crate) fn window_factor(node: &ComputedNode, ui_scale: f32) -> f32 {
    node.inverse_scale_factor() * ui_scale
}
#[derive(Component)]
pub(crate) struct RewardIcon {
    pub item: theia_core::content::ContentId,
    pub count: u32,
}
/// Decorative companions to the existing full reward names/counts. No widget,
/// item choice or command is installed; the enclosing offer retains its action.
pub(crate) fn reward_icons(
    parent: &mut ChildSpawnerCommands,
    skin: &Skin,
    grants: &[theia_core::rewards::Grant],
) {
    if grants.is_empty() {
        return;
    }
    parent
        .spawn((
            Pickable::IGNORE,
            Node {
                width: percent(100),
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(6),
                row_gap: px(6),
                flex_shrink: 0.,
                ..default()
            },
        ))
        .with_children(|row| {
            for grant in grants {
                row.spawn((
                    RewardIcon {
                        item: grant.item().clone(),
                        count: grant.count(),
                    },
                    DarkSurface,
                    Pickable::IGNORE,
                    Node {
                        width: px(80),
                        height: px(36),
                        flex_shrink: 0.,
                        padding: UiRect::all(px(2)),
                        column_gap: px(4),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|cell| {
                    cell.spawn((
                        skin.item_icon(grant.item().as_str()),
                        Pickable::IGNORE,
                        crate::ui_theme::slot::frame(Node {
                            width: px(32),
                            height: px(32),
                            flex_shrink: 0.,
                            ..default()
                        }),
                    ));
                    cell.spawn((
                        Text::new(grant.count().to_string()),
                        Pickable::IGNORE,
                        TextFont {
                            font_size: bevy::text::FontSize::Px(13.),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}
pub(crate) const PAPER_MAX_HEIGHT: f32 = 600.;
pub(crate) const SLOT: f32 = 44.;
pub(crate) const SLOTS: usize = 12;
pub(crate) const ACTION_WIDTH: f32 = SLOT * SLOTS as f32 + 4. * (SLOTS - 1) as f32 + 18.;
pub(crate) const ACTION_END_OUTSET: f32 = 44.;
pub(crate) const XP_WIDTH: f32 = ACTION_WIDTH + 24.;
pub(crate) const XP_HEIGHT: f32 = 20.;
#[derive(Component)]
pub(crate) struct UnitFrame(pub bool);
pub(crate) fn hud_shadow() -> bevy::ui::widget::TextShadow {
    bevy::ui::widget::TextShadow {
        offset: Vec2::splat(1.5),
        color: Color::BLACK,
    }
}
/// Gameplay readouts sit above projected overhead names but below menus.
pub(crate) const UNIT_LAYER: i32 = 5;
pub(crate) const UNIT_WIDTH: f32 = 326.;
pub(crate) const UNIT_INSET: f32 = 84.;

pub(crate) fn unit_frame(player: bool) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        margin: UiRect::left(px(unit_margin(player, 1280.))),
        top: px(12),
        width: px(UNIT_WIDTH),
        min_height: px(92),
        padding: UiRect::new(px(UNIT_INSET), px(0), px(7), px(0)),
        row_gap: px(1),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}
fn unit_margin(player: bool, logical_width: f32) -> f32 {
    let width = if logical_width.is_finite() {
        logical_width
    } else {
        1280.
    };
    let left = if player {
        14.
    } else {
        (width * 0.39).min(width - 560.)
    };
    left - width * 0.5
}
fn unobscured_unit_margin(player: bool, width: f32, dialogue: bool, bag: bool) -> f32 {
    if !width.is_finite() || (!dialogue && !bag) {
        return unit_margin(player, width);
    }
    let left = if dialogue { 444. } else { 14. };
    let right = width - if bag { 352. } else { 12. };
    let player_x = left;
    let target_x = width * 0.5 + unit_margin(false, width);
    let target_x = if right - left >= 2. * UNIT_WIDTH + 12. {
        target_x.clamp(left + UNIT_WIDTH + 12., right - UNIT_WIDTH)
    } else {
        right - UNIT_WIDTH
    };
    (if player { player_x } else { target_x }) - width * 0.5
}
fn responsive_units(
    windows: Query<&Window>,
    cameras: Query<&Camera>,
    scale: Res<bevy::ui::UiScale>,
    dialogue: Option<Res<crate::dialogue::Ui>>,
    bag: Option<Res<crate::inventory::Ui>>,
    mut frames: Query<(&UnitFrame, &mut Node)>,
    mut tooltips: Query<(&crate::combat::ui::ActionLabel, &mut Node), Without<UnitFrame>>,
) {
    let viewport = windows
        .single()
        .ok()
        .map(|w| Vec2::new(w.width(), w.height()))
        .or_else(|| cameras.iter().find_map(Camera::logical_viewport_size));
    let Some(viewport) = viewport.filter(|v| v.is_finite() && v.min_element() > 0.) else {
        return;
    };
    if !scale.0.is_finite() || scale.0 <= 0. {
        return;
    }
    let margin = |player| {
        unobscured_unit_margin(
            player,
            viewport.x / scale.0,
            dialogue.as_ref().is_some_and(|d| d.open),
            bag.as_ref().is_some_and(|b| b.open),
        )
    };
    for (side, mut node) in &mut frames {
        let margin = px(margin(side.0));
        if node.margin.left != margin {
            node.margin.left = margin;
        }
        let stacked = !side.0
            && dialogue.as_ref().is_some_and(|d| d.open)
            && bag.as_ref().is_some_and(|b| b.open)
            && viewport.x / scale.0 < 444. + 352. + 2. * UNIT_WIDTH + 12.;
        let top = px(if stacked { 180. } else { 12. });
        if node.top != top {
            node.top = top;
        }
    }
    // Keep hover details to the right of the cast bar, and within the window.
    let width = viewport.x / scale.0;
    let tooltip_x = (width * 0.5 + ACTION_WIDTH * 0.5 + 12.).min(width - 292.);
    for (slot, mut node) in &mut tooltips {
        let left = px(tooltip_x - (width - ACTION_WIDTH) * 0.5 - 9. - slot.0 as f32 * (SLOT + 4.));
        if node.left != left {
            node.left = left;
        }
    }
}
#[derive(Component)]
pub(crate) struct HeaderClose;
/// Ordinary Bevy title row; callers supply their existing close action bundle.
/// Insert Node replacements after spawning to support title/button bundles that
/// already contain Node without duplicate-component bundle panics.
pub(crate) fn window_header(
    title_row: &mut ChildSpawnerCommands,
    title: impl Bundle,
    close: impl Bundle,
    label: &str,
) {
    title_row
        .spawn((
            DarkSurface,
            Node {
                width: percent(100),
                column_gap: px(8),
                padding: UiRect::axes(px(8), px(6)),
                border: UiRect::bottom(px(1)),
                align_items: AlignItems::Center,
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor::all(GOLD),
        ))
        .with_children(|row| {
            row.spawn(title).insert((
                Node {
                    flex_grow: 1.,
                    min_width: px(0),
                    ..default()
                },
                TextFont {
                    font_size: bevy::text::FontSize::Px(22.),
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.85, 0.70)),
            ));
            row.spawn((HeaderClose, close))
                .insert((
                    AccessibleLabel::new(label),
                    Node {
                        width: px(28),
                        min_width: px(28),
                        height: px(28),
                        padding: UiRect::all(px(4)),
                        flex_shrink: 0.,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Outline::new(px(1), px(-1), GOLD),
                ))
                .with_child((
                    Text::new("X"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(16.),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
        });
}

#[derive(Component)]
pub(crate) enum Panel {
    Standard,
    Paper,
    QuestTracker,
    Connection,
}

#[derive(Resource)]
pub(crate) struct Skin {
    pub(crate) wolf: Handle<Image>,
    sword: ImageNode,
    arcane: ImageNode,
    fire: ImageNode,
    ice: ImageNode,
    fallback: ImageNode,
    item_icons: [ImageNode; 6],
    title_font: Handle<Font>,
    body_font: Handle<Font>,
    bold_font: Handle<Font>,
}
impl Skin {
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        Self {
            wolf: default(),
            sword: default(),
            arcane: default(),
            fire: default(),
            ice: default(),
            fallback: default(),
            item_icons: default(),
            title_font: default(),
            body_font: default(),
            bold_font: default(),
        }
    }
    /// Presentation mapping only. Unknown authored items keep a generic icon.
    /// Pelt/leather, stone/shard and wand/crystal are provisional art, not item metadata.
    pub(crate) fn item_icon(&self, id: &str) -> ImageNode {
        let index = match id {
            "theia.dev.training-stone" => 0,
            "theia.dev.training-token" => 1,
            "theia.dev.training-sword" => 2,
            "theia.dev.training-wand" => 3,
            "theia.dev.wolf-pelt" => 4,
            "theia.dev.roadside-orders" => 5,
            _ => return self.fallback.clone(),
        };
        self.item_icons[index].clone()
    }
    pub(crate) fn icon(&self, id: &str) -> ImageNode {
        match id {
            "theia.dev.sword-slash" => self.sword.clone(),
            "theia.dev.arcane-bolt" => self.arcane.clone(),
            "theia.dev.scorch" => self.fire.clone(),
            "theia.dev.frost-armor" => self.ice.clone(),
            _ => self.fallback.clone(),
        }
    }
}

pub(crate) fn parse(
    args: &mut Vec<String>,
    enabled: bool,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let Some(i) = args.iter().position(|s| s == "--fantasy-ui") else {
        return Ok(None);
    };
    args.remove(i);
    if !enabled || i >= args.len() || args[i].starts_with("--") {
        return Err("--fantasy-ui needs a local HUD directory and Classic combat".into());
    }
    let root = PathBuf::from(args.remove(i)).canonicalize()?;
    if args.iter().any(|s| s == "--fantasy-ui") {
        return Err("Choose --fantasy-ui once".into());
    }
    validate(&root)?;
    Ok(Some(root))
}
pub(crate) fn parse_scale(
    args: &mut Vec<String>,
    enabled: bool,
) -> Result<f32, Box<dyn std::error::Error>> {
    let Some(i) = args.iter().position(|arg| arg == "--ui-scale") else {
        return Ok(1.);
    };
    args.remove(i);
    if !enabled || i >= args.len() || args[i].starts_with("--") {
        return Err("--ui-scale requires --fantasy-ui and a factor from 0.75 to 1.5".into());
    }
    let scale: f32 = args.remove(i).parse()?;
    if !scale.is_finite()
        || !(0.75..=1.5).contains(&scale)
        || args.iter().any(|arg| arg == "--ui-scale")
    {
        return Err("Choose --ui-scale once, from 0.75 to 1.5".into());
    }
    Ok(scale)
}
#[derive(Resource)]
pub(crate) struct PreferredScale(pub(crate) f32);
/// Retain a 1280x720 logical layout when resizing/fullscreen reduces space.
/// This changes only Bevy's UI scale, never the window, camera or world frame.
pub(crate) fn fit_scale(preferred: f32, viewport: Vec2) -> f32 {
    if !viewport.is_finite() || viewport.min_element() <= 0. {
        return preferred;
    }
    preferred.min(viewport.x / 1280.).min(viewport.y / 720.)
}
/// Camera projection returns window coordinates; absolute UI Nodes use logical
/// UI coordinates before UiScale. Keep projected labels on their world anchor.
pub(crate) fn projected_ui_point(window_point: Vec2, scale: f32) -> Option<Vec2> {
    if !scale.is_finite() || scale <= 0. || !window_point.is_finite() {
        return None;
    }
    let point = window_point / scale;
    point.is_finite().then_some(point)
}
pub(crate) fn install_scale(app: &mut App, preferred: f32) {
    app.insert_resource(PreferredScale(preferred))
        .insert_resource(bevy::ui::UiScale(preferred))
        .add_systems(Update, fit_window);
    graphics::install(app);
    settings::install(app);
    debug::install(app);
}
fn fit_window(
    preferred: Res<PreferredScale>,
    windows: Query<&Window>,
    mut scale: ResMut<bevy::ui::UiScale>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    if viewport.min_element() <= 0. {
        return;
    }
    let value = fit_scale(preferred.0, viewport);
    if scale.0 != value {
        scale.0 = value;
    }
}
fn validate(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for (name, expected) in FILES.into_iter().zip(IMAGE_DIMENSIONS) {
        let path = root.join(name).canonicalize()?;
        if !path.starts_with(root) || !path.is_file() || path.metadata()?.len() > 4 * 1024 * 1024 {
            return Err(format!("UI image outside local pack or larger than 4 MiB: {name}").into());
        }
        let mut header = [0; 24];
        std::fs::File::open(path)?.read_exact(&mut header)?;
        let width = u32::from_be_bytes(header[16..20].try_into()?);
        let height = u32::from_be_bytes(header[20..24].try_into()?);
        if &header[..8] != b"\x89PNG\r\n\x1a\n"
            || &header[12..16] != b"IHDR"
            || (width, height) != expected
        {
            return Err(
                format!("UI image must be the expected source PNG dimensions: {name}").into(),
            );
        }
    }
    for name in [TITLE_FONT, BODY_FONT] {
        let path = root.join(name).canonicalize()?;
        if !path.starts_with(root) || !path.is_file() || path.metadata()?.len() > 256 * 1024 {
            return Err(format!("UI font outside pack or larger than 256 KiB: {name}").into());
        }
        let mut signature = [0; 4];
        std::fs::File::open(path)?.read_exact(&mut signature)?;
        if signature != [0, 1, 0, 0] {
            return Err(format!("Expected TrueType UI font: {name}").into());
        }
    }
    Ok(())
}

/// Register before DefaultPlugins. Load handles after DefaultPlugins, before startup.
pub(crate) fn register(app: &mut App, directory: PathBuf) {
    app.register_asset_source(
        "fantasy-ui",
        AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(directory.clone()))),
    );
}
pub(crate) fn load(app: &mut App) {
    crate::ui_theme::install(app);
    kit::install(app);
    let title_font = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(
            include_bytes!("../../../content/ui/fonts/LiberationSerif-Regular.ttf").to_vec(),
        ));
    let body_font = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(
            include_bytes!("../../../content/ui/fonts/LiberationSans-Regular.ttf").to_vec(),
        ));
    let bold_font = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(
            include_bytes!("../../../content/ui/fonts/LiberationSans-Bold.ttf").to_vec(),
        ));
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let mut image = |bytes: &[u8]| images.add(crate::ui_theme::sprites::decode(bytes));
    let wolf = image(include_bytes!("../../../content/ui/hud/wolf-portrait.png"));
    // Sample the artwork inside these tiles. The slot supplies its own frame.
    let mut tile = |bytes: &[u8], [left, top, right, bottom]: [f32; 4]| ImageNode {
        rect: Some(Rect::new(left, top, right, bottom)),
        ..ImageNode::new(image(bytes))
    };
    let book = tile(
        include_bytes!("../../../content/ui/sprites-v3/sprites/icon/book.png"),
        [7., 12., 177., 148.],
    );
    let skin = Skin {
        wolf,
        sword: tile(
            include_bytes!("../../../content/ui/sprites-v3/sprites/ability/sword.png"),
            [43., 39., 285., 281.],
        ),
        arcane: tile(
            include_bytes!("../../../content/ui/sprites-v3/sprites/ability/arcane-arrow.png"),
            [40., 38., 280., 278.],
        ),
        fire: tile(
            include_bytes!("../../../content/ui/sprites-v3/sprites/ability/flame-burst.png"),
            [32., 26., 300., 294.],
        ),
        ice: tile(
            include_bytes!("../../../content/ui/sprites-v3/sprites/ability/ice.png"),
            [30., 27., 290., 287.],
        ),
        fallback: book.clone(),
        item_icons: [
            tile(
                include_bytes!("../../../content/ui/sprites-v3/sprites/item/shard.png"),
                [34., 46., 226., 238.],
            ),
            tile(
                include_bytes!("../../../content/ui/sprites-v3/sprites/icon/coins.png"),
                [14., 4., 162., 152.],
            ),
            tile(
                include_bytes!("../../../content/ui/sprites-v3/sprites/item/sword-common.png"),
                [38., 40., 278., 280.],
            ),
            tile(
                include_bytes!("../../../content/ui/sprites-v3/sprites/item/crystal.png"),
                [37., 45., 291., 299.],
            ),
            tile(
                include_bytes!("../../../content/ui/sprites-v3/sprites/item/leather.png"),
                [34., 41., 226., 233.],
            ),
            book,
        ],
        title_font,
        body_font,
        bold_font,
    };
    app.insert_resource(skin);
    app.add_systems(PostStartup, style_panels)
        .add_systems(PostUpdate, style_text.before(bevy::ui::UiSystems::Content))
        .add_systems(
            PostUpdate,
            responsive_units.before(bevy::ui::UiSystems::Layout),
        );
}
type Surface<'a> = (
    Option<&'a ChildOf>,
    Option<&'a Panel>,
    Has<DarkSurface>,
    Has<bevy::ui_widgets::Button>,
);
fn paper_text(mut entity: Entity, surfaces: &Query<Surface<'_>>) -> bool {
    for _ in 0..32 {
        let Ok((parent, panel, header, button)) = surfaces.get(entity) else {
            return false;
        };
        if header || button {
            return false;
        }
        if let Some(panel) = panel {
            return matches!(panel, Panel::Paper);
        }
        let Some(parent) = parent else {
            return false;
        };
        entity = parent.parent();
    }
    false
}
type ThemeText<'a> = (Entity, &'a mut TextFont, &'a mut TextColor);
type NewText = (
    Or<(With<Text>, With<bevy::text::EditableText>)>,
    Added<TextFont>,
);
fn style_text(
    skin: Res<Skin>,
    surfaces: Query<Surface<'_>>,
    mut text: Query<ThemeText<'_>, NewText>,
) {
    for (entity, mut font, mut color) in &mut text {
        font.font = if font.weight == bevy::text::FontWeight::BOLD {
            skin.bold_font.clone().into()
        } else if matches!(font.font_size, bevy::text::FontSize::Px(size) if size >= 22.) {
            skin.title_font.clone().into()
        } else {
            skin.body_font.clone().into()
        };
        if paper_text(entity, &surfaces) {
            color.0 = INK;
        }
    }
}
type ThemePanel<'a> = (
    Entity,
    &'a Panel,
    &'a mut Node,
    &'a mut BackgroundColor,
    Option<&'a GlobalZIndex>,
    Has<crate::inventory::Pane>,
);
fn style_panels(mut commands: Commands, mut panels: Query<ThemePanel>) {
    for (entity, panel, mut node, mut background, layer, backpack) in &mut panels {
        background.0 = PANEL;
        // The shared nine-slice trim replaces rectangular outlines without shrinking content.
        commands.entity(entity).remove::<Outline>();
        if backpack || matches!(panel, Panel::Standard | Panel::Paper | Panel::QuestTracker) {
            commands.entity(entity).insert(crate::ui_theme::Panel);
        }
        match panel {
            Panel::Standard | Panel::Paper => {
                if matches!(panel, Panel::Paper) {
                    background.0 = PAPER;
                }
                if layer.is_none() {
                    commands.entity(entity).insert(GlobalZIndex(20));
                }
            }
            Panel::QuestTracker => {
                node.left = Val::Auto;
                node.right = px(12);
                node.top = px(267);
                node.width = px(300);
                node.max_height = percent(34);
            }
            Panel::Connection => {
                node.top = Val::Auto;
                node.bottom = px(12);
                node.width = px(240);
                node.padding = UiRect::all(px(6));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn widget_coordinates_retain_ui_enlargement_and_only_remove_window_dpi() {
        let physical = Vec2::new(1200., 900.);
        // Same physical coordinate at DPI 2 and UI 1.5: the OS point is
        // (600,450), whereas an uncorrected inverse gives (400,300).
        let node = ComputedNode {
            inverse_scale_factor: 1. / 3.,
            ..default()
        };
        assert_eq!(physical * window_factor(&node, 1.5), Vec2::new(600., 450.));
        let node = ComputedNode {
            inverse_scale_factor: 0.8,
            ..default()
        };
        assert_eq!(physical * window_factor(&node, 1.25), physical);
        let node = ComputedNode {
            inverse_scale_factor: 1.,
            ..default()
        };
        assert_eq!(physical * window_factor(&node, 1.), physical);
    }
    #[test]
    fn parchment_uses_ink_for_body_and_later_labels_but_preserves_header_and_buttons() {
        let mut app = App::new();
        app.insert_resource(Skin::fixture())
            .add_systems(PostUpdate, style_text);
        let pane = app.world_mut().spawn(Panel::Paper).id();
        let body = app
            .world_mut()
            .spawn((
                Text::new("Quest text"),
                TextColor(Color::WHITE),
                ChildOf(pane),
            ))
            .id();
        let header = app.world_mut().spawn((DarkSurface, ChildOf(pane))).id();
        let title = app
            .world_mut()
            .spawn((Text::new("Guide"), TextColor(GOLD), ChildOf(header)))
            .id();
        let button = app
            .world_mut()
            .spawn((bevy::ui_widgets::Button, ChildOf(pane)))
            .id();
        let action = app
            .world_mut()
            .spawn((
                Text::new("Complete quest"),
                TextColor(Color::WHITE),
                ChildOf(button),
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<TextColor>(body), Some(&TextColor(INK)));
        assert_eq!(app.world().get::<TextColor>(title), Some(&TextColor(GOLD)));
        assert_eq!(
            app.world().get::<TextColor>(action),
            Some(&TextColor(Color::WHITE))
        );
        let late = app
            .world_mut()
            .spawn((Text::new("Reward description"), ChildOf(pane)))
            .id();
        app.update();
        assert_eq!(app.world().get::<TextColor>(late), Some(&TextColor(INK)));
    }
    #[test]
    fn projected_world_labels_keep_the_same_window_anchor_at_each_ui_scale() {
        let window_point = Vec2::new(960., 540.);
        for scale in [0.5, 0.75, 1., 1.25, 1.5] {
            let ui = projected_ui_point(window_point, scale).unwrap();
            assert_eq!(ui * scale, window_point);
            // Marker clipping uses the same logical coordinate frame as placement.
            let viewport = projected_ui_point(Vec2::new(1920., 1080.), scale).unwrap();
            assert_eq!(ui / viewport, Vec2::splat(0.5));
        }
        for scale in [0., -1., f32::NAN, f32::INFINITY] {
            assert!(projected_ui_point(window_point, scale).is_none());
        }
        assert!(projected_ui_point(Vec2::new(f32::NAN, 1.), 1.).is_none());
    }
    #[test]
    fn theme_fonts_apply_to_later_spawned_text_as_well_as_initial_panels() {
        use bevy::text::FontSource;
        let mut app = App::new();
        let mut fonts = Assets::<Font>::default();
        let title = fonts.add(Font::from_bytes(Vec::new()));
        let body = fonts.add(Font::from_bytes(Vec::new()));
        let mut skin = Skin::fixture();
        skin.title_font = title.clone();
        skin.body_font = body.clone();
        app.insert_resource(skin)
            .add_systems(PostUpdate, style_text);
        let initial = app
            .world_mut()
            .spawn((
                Text::new("Initial title"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(24.),
                    ..default()
                },
            ))
            .id();
        app.update();
        assert!(
            matches!(&app.world().get::<TextFont>(initial).unwrap().font,FontSource::Handle(font) if font == &title)
        );
        let late = app
            .world_mut()
            .spawn((
                Text::new("Later label"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(16.),
                    ..default()
                },
            ))
            .id();
        app.update();
        assert!(
            matches!(&app.world().get::<TextFont>(late).unwrap().font,FontSource::Handle(font) if font == &body)
        );
        assert_eq!(app.world().get::<Text>(late).unwrap().0, "Later label");
    }
    #[test]
    fn scale_is_optional_finite_bounded_and_cannot_enable_a_skin_implicitly() {
        assert_eq!(parse_scale(&mut Vec::new(), false).unwrap(), 1.);
        assert!(parse_scale(&mut vec!["--ui-scale".into(), "1.25".into()], false).is_err());
        for value in [
            "NaN",
            "inf",
            "-inf",
            "0",
            "0.74",
            "1.51",
            "200",
            "--classic",
        ] {
            assert!(
                parse_scale(&mut vec!["--ui-scale".into(), value.into()], true).is_err(),
                "{value}"
            );
        }
        assert!(parse_scale(&mut vec!["--ui-scale".into()], true).is_err());
        assert!(
            parse_scale(
                &mut vec![
                    "--ui-scale".into(),
                    "1".into(),
                    "--ui-scale".into(),
                    "1.25".into()
                ],
                true
            )
            .is_err()
        );
        for value in ["0.75", "1", "1.25", "1.5"] {
            let mut args = vec!["--classic".into(), "--ui-scale".into(), value.into()];
            assert_eq!(
                parse_scale(&mut args, true).unwrap(),
                value.parse::<f32>().unwrap()
            );
            assert_eq!(args, ["--classic"]);
        }
    }
    #[test]
    fn conversation_and_backpack_leave_both_unit_frames_visible_at_supported_widths() {
        for width in [1280., 1920., 3440. / 1.5, 3440.] {
            let player = width * 0.5 + unobscured_unit_margin(true, width, true, true);
            let target = width * 0.5 + unobscured_unit_margin(false, width, true, true);
            assert!(player >= 444. && target + UNIT_WIDTH <= width - 352.);
            if width >= 444. + 352. + 2. * UNIT_WIDTH + 12. {
                assert!(target >= player + UNIT_WIDTH + 12.);
            }
        }
        assert_eq!(
            unobscured_unit_margin(true, 1280., false, false),
            unit_margin(true, 1280.)
        );
    }
    #[test]
    fn unit_frames_keep_top_left_anchors_across_window_and_ui_scale_changes() {
        let mut app = App::new();
        app.insert_resource(bevy::ui::UiScale(1.25));
        app.add_systems(Update, responsive_units);
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (3440, 1440).into(),
                ..default()
            })
            .id();
        let player = app
            .world_mut()
            .spawn((UnitFrame(true), unit_frame(true)))
            .id();
        let target = app
            .world_mut()
            .spawn((UnitFrame(false), unit_frame(false)))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Node>(player).unwrap().margin.left,
            px(14. - 3440. / 1.25 / 2.)
        );
        assert_eq!(
            app.world().get::<Node>(target).unwrap().margin.left,
            px(unit_margin(false, 3440. / 1.25))
        );
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(1920., 1200.);
        app.update();
        assert_eq!(
            app.world().get::<Node>(player).unwrap().margin.left,
            px(14. - 1920. / 1.25 / 2.)
        );
        assert_eq!(
            app.world().get::<Node>(target).unwrap().margin.left,
            px(unit_margin(false, 1920. / 1.25))
        );
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(3440., 1440.);
        app.world_mut().resource_mut::<bevy::ui::UiScale>().0 = 1.5;
        app.update();
        let expected = unit_margin(true, 3440. / 1.5);
        assert_eq!(
            app.world().get::<Node>(player).unwrap().margin.left,
            px(expected)
        );
        assert_eq!(3440. / 1.5 / 2. + expected, 14.);
        assert_eq!(unit_margin(true, 10000.), 14. - 5000.);
        assert_eq!(unit_margin(false, f32::NAN), unit_margin(false, 1280.));
    }
    #[test]
    fn preferred_scale_fits_a_smaller_window_then_restores_on_resize_without_changing_it() {
        let mut app = App::new();
        install_scale(&mut app, 1.5);
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: bevy::window::WindowResolution::new(1280, 720)
                    .with_scale_factor_override(1.),
                ..default()
            })
            .id();
        app.update();
        assert_eq!(app.world().resource::<bevy::ui::UiScale>().0, 1.);
        assert_eq!(app.world().get::<Window>(window).unwrap().width(), 1280.);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(3840., 2160.);
        app.update();
        assert_eq!(app.world().resource::<bevy::ui::UiScale>().0, 1.5);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(960., 540.);
        app.update();
        assert_eq!(app.world().resource::<bevy::ui::UiScale>().0, 0.75);
        assert_eq!(fit_scale(1.25, Vec2::new(3440., 1440.)), 1.25);
        assert_eq!(fit_scale(1.5, Vec2::ZERO), 1.5);
        assert_eq!(fit_scale(1.5, Vec2::new(f32::NAN, 720.)), 1.5);
    }
    #[test]
    fn absent_skin_preserves_arguments_and_invalid_requests_fail_before_app_creation() {
        let mut args = vec!["--classic".into()];
        assert!(parse(&mut args, true).unwrap().is_none());
        assert_eq!(args, ["--classic"]);
        for values in [vec!["--fantasy-ui"], vec!["--fantasy-ui", "--classic"]] {
            assert!(parse(&mut values.into_iter().map(String::from).collect(), true).is_err());
        }
        assert!(parse(&mut vec!["--fantasy-ui".into(), "/tmp".into()], false).is_err());
        assert!(
            parse(
                &mut vec![
                    "--fantasy-ui".into(),
                    "/tmp".into(),
                    "--fantasy-ui".into(),
                    "/tmp".into()
                ],
                true
            )
            .is_err()
        );
    }
    #[test]
    fn lfs_pointer_and_wrong_source_dimensions_are_not_ui_images() {
        let root = std::env::temp_dir().join(format!("theia-ui-header-{}", std::process::id()));
        let path = root.join(SWORD);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            b"version https://git-lfs.github.com/spec/v1\noid sha256:missing\n",
        )
        .unwrap();
        assert!(
            validate(&root)
                .unwrap_err()
                .to_string()
                .contains("source PNG")
        );
        let mut header = [0; 24];
        header[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        header[12..16].copy_from_slice(b"IHDR");
        header[16..20].copy_from_slice(&4096_u32.to_be_bytes());
        header[20..24].copy_from_slice(&4096_u32.to_be_bytes());
        std::fs::write(&path, header).unwrap();
        assert!(validate(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cosmetic_panel_skin_preserves_hidden_state_scroll_and_content_dimensions() {
        let mut app = App::new();
        app.insert_resource(Skin::fixture())
            .init_resource::<Assets<Image>>();
        crate::ui_theme::install(&mut app);
        let node = Node {
            width: px(340),
            max_height: percent(75),
            padding: UiRect::all(px(12)),
            display: Display::None,
            overflow: Overflow::scroll_y(),
            ..default()
        };
        let pane = app
            .world_mut()
            .spawn((
                Panel::Standard,
                crate::inventory::Pane,
                node.clone(),
                ScrollPosition::from(Vec2::new(0., 51.)),
                Visibility::Hidden,
                BackgroundColor(Color::BLACK),
                GlobalZIndex(60),
            ))
            .id();
        app.add_systems(Update, style_panels);
        app.update();
        let result = app.world().get::<Node>(pane).unwrap();
        assert_eq!(result.width, node.width);
        assert_eq!(result.padding, node.padding);
        assert_eq!(
            result.border, node.border,
            "Outline must not shrink minimap/scroll content"
        );
        assert_eq!(result.display, Display::None);
        assert_eq!(
            app.world().get::<Visibility>(pane),
            Some(&Visibility::Hidden)
        );
        assert_eq!(app.world().get::<ScrollPosition>(pane).unwrap().y, 51.);
        assert!(app.world().get::<Outline>(pane).is_none());
        assert!(app.world().get::<ImageNode>(pane).is_none());
        let frame = app.world().get::<Children>(pane).unwrap()[0];
        assert!(matches!(
            app.world().get::<ImageNode>(frame).unwrap().image_mode,
            NodeImageMode::Sliced(_)
        ));
        assert_eq!(
            app.world().get::<Node>(frame).unwrap().position_type,
            PositionType::Absolute
        );
        assert_eq!(app.world().get::<Pickable>(frame), Some(&Pickable::IGNORE));
        assert_eq!(
            app.world().get::<bevy::ui::IgnoreScroll>(frame).unwrap().0,
            BVec2::TRUE
        );
        assert_eq!(
            app.world().get::<GlobalZIndex>(pane),
            Some(&GlobalZIndex(60)),
            "Retain the document reader's existing priority"
        );
    }
    #[test]
    #[ignore = "Requires the user's local licensed SYNTY HUD directory"]
    fn local_pack_preflight_and_actual_bevy_png_decode() {
        let root = PathBuf::from(std::env::var_os("THEIA_UI_SOURCE").expect("Set THEIA_UI_SOURCE"));
        let root = root.canonicalize().unwrap();
        assert_eq!(
            parse(
                &mut vec!["--fantasy-ui".into(), root.to_str().unwrap().into()],
                true
            )
            .unwrap(),
            Some(root.clone())
        );
        for name in FILES {
            let bytes = std::fs::read(root.join(name)).unwrap();
            let image = Image::from_buffer(
                &bytes,
                bevy::image::ImageType::Extension("png"),
                bevy::image::CompressedImageFormats::NONE,
                true,
                bevy::image::ImageSampler::default(),
                bevy::asset::RenderAssetUsages::default(),
            )
            .unwrap();
            assert!(image.data.is_some_and(|data| !data.is_empty()));
        }
    }
}
