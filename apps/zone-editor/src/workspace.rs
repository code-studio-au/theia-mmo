//! Application shell and standard Bevy controls over the retained editor operations.
use super::*;
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::ui::{Checked, InteractionDisabled};
use bevy::ui_widgets::{
    Slider, SliderPrecision, SliderRange, SliderStep, SliderThumb, SliderValue, TrackClick,
    ValueChange,
};

pub(super) const HEADER: f32 = 52.;
pub(super) const INSPECTOR_WIDTH: f32 = 360.;
const SETTINGS_HINT: &str = "Click sliders to focus.\nLeft/right, Home/End: adjust\nCtrl S saves the zone.\nCtrl Z / Ctrl Y: undo / redo";
#[derive(Component)]
pub(super) struct Inspector;
#[derive(Component)]
pub(super) struct SettingsHint;
#[derive(Resource, Default)]
pub(super) struct Requests {
    pub name: bool,
    pub name_zone: bool,
    pub find: bool,
    pub transform: bool,
    pub transform_field: Option<usize>,
    pub spawn_transform: bool,
    pub help: bool,
    pub file_open: bool,
    pub file_save_as: bool,
    pub particle_workspace: bool,
    pub particle_preview: bool,
    pub bind_particle: bool,
}
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    New,
    Mode(EditorMode),
    Command(Action),
    Frame,
    Rename,
    RenameZone,
    Transform,
    SpawnTransform,
    FileOpen,
    FileSaveAs,
    ParticleWorkspace,
    ParticlePreview,
    BindParticle,
    Point(i8),
    WholePath,
    Snap,
    Handles,
    Grid,
    GizmoTool(gizmo::Tool),
    View(camera_views::Preset),
    Help,
}
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Setting {
    MoveGrid,
    PlacementSize(usize),
    Radius,
    Strength,
    Blend,
    Level,
    ScatterRadius,
    ScatterCount,
}
#[derive(Component)]
pub(super) struct SettingRow(Setting);
#[derive(Component)]
pub(super) struct SettingLabel(Setting);
#[derive(Component)]
pub(super) struct SettingValue;
#[derive(Component)]
pub(super) struct Thumb;
#[derive(Component)]
pub(super) struct Caption;
#[derive(Component)]
pub(super) struct ButtonLabel;

pub(super) fn busy(session: &Session) -> bool {
    session.ui.input_owned
        || session.has_modal()
        || session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
}
pub(super) fn new_document(
    session: &mut Session,
    view: &mut View,
    content: &mut outliner::Content,
) {
    match session.new_document(view) {
        Ok(true) => *content = outliner::Content::default(),
        Ok(false) => {}
        Err(error) => session.status = error,
    }
}
pub(super) fn mode(session: &Session) -> EditorMode {
    if session.terrain_paint {
        EditorMode::Paint
    } else if session.terrain_mode {
        EditorMode::Sculpt
    } else if session.path_mode {
        EditorMode::Guides
    } else if session.marker_mode {
        EditorMode::Markers
    } else if session.vegetation_mode {
        EditorMode::Plants
    } else {
        EditorMode::Objects
    }
}
fn button(control: Control, caption: &str) -> impl Bundle {
    (
        control,
        Button,
        Hovered::default(),
        theme::button_node(),
        BackgroundColor(theme::Variant::Default.background(false)),
        children![(ButtonLabel, theme::label(caption, 12., theme::BODY))],
    )
}
pub(super) fn setup(
    mut commands: Commands,
    icons: Option<Res<sidebar::Icons>>,
    server: Option<Res<AssetServer>>,
    images: Option<Res<Assets<Image>>>,
) {
    sidebar::setup(&mut commands, icons.as_deref());
    commands.init_resource::<Requests>();
    commands
        .spawn((
            GlobalZIndex(5),
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                right: px(0),
                height: px(HEADER),
                padding: UiRect::all(px(12)),
                border: UiRect::bottom(px(1)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: percent(100),
                column_gap: px(12),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                menus::spawn_select(row, menus::SelectKind::Document);
                menus::spawn(
                    row,
                    "File",
                    &[
                        (Control::New, "New zone", "Ctrl N"),
                        (Control::FileOpen, "Open…", "Ctrl O"),
                        (Control::Command(Action::Save), "Save", "Ctrl S"),
                        (Control::FileSaveAs, "Save as…", "Ctrl Shift S"),
                        (Control::Command(Action::Load), "Reload", "Ctrl Shift O"),
                        (Control::ParticleWorkspace, "Particles workspace…", ""),
                    ],
                );
                menus::spawn(
                    row,
                    "Edit",
                    &[
                        (Control::Command(Action::Undo), "Undo", "Ctrl Z"),
                        (Control::Command(Action::Redo), "Redo", "Ctrl Y"),
                        (Control::Rename, "Rename…", "F8"),
                        (Control::Transform, "Focus attributes", "F9"),
                        (Control::Command(Action::Duplicate), "Duplicate", "Ctrl D"),
                        (Control::Command(Action::Delete), "Delete", "Del"),
                    ],
                );
                menus::spawn(
                    row,
                    "Add",
                    &[(
                        Control::Command(Action::AddParticle),
                        "Particle reference at spawn",
                        "",
                    )],
                );
                menus::spawn(
                    row,
                    "View",
                    &[
                        (Control::Frame, "Frame selection", "F"),
                        (
                            Control::View(camera_views::Preset::Perspective),
                            "Perspective",
                            "",
                        ),
                        (Control::View(camera_views::Preset::Top), "Top", ""),
                        (Control::View(camera_views::Preset::Front), "Front", ""),
                        (Control::View(camera_views::Preset::Right), "Right", ""),
                        (Control::Help, "Keyboard shortcuts", ""),
                    ],
                );
                menus::spawn(
                    row,
                    "Zone",
                    &[
                        (Control::Command(Action::Spawn), "Spawn at view centre", ""),
                        (Control::SpawnTransform, "Edit spawn…", ""),
                    ],
                );
                row.spawn(Node {
                    flex_grow: 1.,
                    ..default()
                });
                row.spawn(button(Control::Command(Action::Play), "Playtest"));
                row.spawn(button(Control::Command(Action::Save), "Save"));
            });
        });
    commands
        .spawn((
            GlobalZIndex(4),
            Inspector,
            panels::Region::Inspector,
            Node {
                position_type: PositionType::Absolute,
                left: px(sidebar::WIDTH),
                top: px(HEADER),
                bottom: px(0),
                width: px(INSPECTOR_WIDTH),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme::PANEL),
        ))
        .with_children(|root| {
            let caption = panels::header(root, panels::Region::Inspector, "Properties");
            root.commands().entity(caption).insert(Caption);
            panels::body(root, panels::Region::Inspector, |root| {
                property_tabs::layout(
                    root,
                    server.as_deref().filter(|_| images.is_some()),
                    |root| {
                        scrolling::pane(root, 12., scrolling::Pane::Workspace, |panel| {
                            property_tabs::group(panel, property_tabs::Tab::Attributes, |panel| {
                                panel
                                    .spawn(Node {
                                        width: percent(100),
                                        align_items: AlignItems::Center,
                                        column_gap: px(8),
                                        ..default()
                                    })
                                    .with_children(|row| {
                                        row.spawn((
                                            Info,
                                            text("", 14.),
                                            Node {
                                                width: px(0),
                                                min_width: px(0),
                                                flex_grow: 1.,
                                                ..default()
                                            },
                                        ));
                                        menus::spawn_property_actions(
                                            row,
                                            server.as_deref().filter(|_| images.is_some()),
                                        );
                                    });
                                properties::setup_summary(panel);
                                panel
                                    .spawn((
                                        menus::GuideControls,
                                        Visibility::Hidden,
                                        Node {
                                            display: Display::None,
                                            width: percent(100),
                                            flex_direction: FlexDirection::Column,
                                            ..default()
                                        },
                                    ))
                                    .with_children(|panel| {
                                        sections::spawn(panel, "Path points", |panel| {
                                            panel
                                                .spawn(Node {
                                                    flex_wrap: FlexWrap::Wrap,
                                                    column_gap: px(6),
                                                    row_gap: px(6),
                                                    ..default()
                                                })
                                                .with_children(|row| {
                                                    for (control, label) in [
                                                        (Control::Point(-1), "Previous point"),
                                                        (Control::Point(1), "Next point"),
                                                        (Control::WholePath, "Whole guide"),
                                                        (
                                                            Control::Command(Action::InsertPoint),
                                                            "Insert point",
                                                        ),
                                                        (
                                                            Control::Command(Action::RemovePoint),
                                                            "Remove point",
                                                        ),
                                                    ] {
                                                        row.spawn(button(control, label));
                                                    }
                                                });
                                        });
                                    });
                            });
                            property_tabs::group(panel, property_tabs::Tab::View, |panel| {
                                sections::spawn(panel, "Viewport", |panel| {
                                    menus::spawn_select(panel, menus::SelectKind::Camera);
                                    panel
                                        .spawn(Node {
                                            flex_wrap: FlexWrap::Wrap,
                                            column_gap: px(12),
                                            row_gap: px(6),
                                            ..default()
                                        })
                                        .with_children(|row| {
                                            theme::checkbox::spawn(row, Control::Grid, "Grid");
                                            theme::checkbox::spawn(
                                                row,
                                                Control::Handles,
                                                "Handles",
                                            );
                                        });
                                    theme::checkbox::spawn(
                                        panel,
                                        Control::Snap,
                                        "Snap (1 m / 15 deg / 10%)",
                                    );
                                    menus::spawn_select(panel, menus::SelectKind::Gizmo);
                                });
                            });
                            property_tabs::group(panel, property_tabs::Tab::Tools, |panel| {
                                setup_tools(panel);
                                sections::spawn(panel, "Tool settings", |panel| {
                                    for setting in [
                                        Setting::MoveGrid,
                                        Setting::PlacementSize(0),
                                        Setting::PlacementSize(1),
                                        Setting::PlacementSize(2),
                                        Setting::Radius,
                                        Setting::Strength,
                                        Setting::Blend,
                                        Setting::Level,
                                        Setting::ScatterRadius,
                                        Setting::ScatterCount,
                                    ] {
                                        let (minimum, maximum) = setting.range();
                                        panel
                                            .spawn((
                                                SettingRow(setting),
                                                Node {
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: px(6),
                                                    ..default()
                                                },
                                            ))
                                            .with_children(|row| {
                                                row.spawn(Node {
                                                    justify_content: JustifyContent::SpaceBetween,
                                                    align_items: AlignItems::Center,
                                                    column_gap: px(12),
                                                    ..default()
                                                })
                                                .with_children(|header| {
                                                    header.spawn((
                                                        SettingLabel(setting),
                                                        theme::field_label(""),
                                                    ));
                                                    header.spawn((
                                                        SettingLabel(setting),
                                                        SettingValue,
                                                        Text::new(""),
                                                        theme::input_font(),
                                                        TextColor(theme::BODY),
                                                        TextLayout::no_wrap(),
                                                        Pickable::IGNORE,
                                                        Node {
                                                            flex_shrink: 0.,
                                                            ..default()
                                                        },
                                                    ));
                                                });
                                                row.spawn((
                                                    bevy::input_focus::tab_navigation::TabIndex(-1),
                                                    setting,
                                                    Slider {
                                                        track_click: TrackClick::Snap,
                                                        ..default()
                                                    },
                                                    SliderValue(minimum),
                                                    SliderRange::new(minimum, maximum),
                                                    SliderStep(match setting {
                                                        Setting::ScatterCount
                                                        | Setting::MoveGrid => 1.,
                                                        Setting::Level | Setting::ScatterRadius => {
                                                            0.1
                                                        }
                                                        Setting::PlacementSize(_) => 0.25,
                                                        _ => 0.05,
                                                    }),
                                                    SliderPrecision(
                                                        if matches!(
                                                            setting,
                                                            Setting::ScatterCount
                                                                | Setting::MoveGrid
                                                        ) {
                                                            0
                                                        } else {
                                                            3
                                                        },
                                                    ),
                                                    Hovered::default(),
                                                    theme::slider::node(),
                                                    BackgroundColor(Color::NONE),
                                                ))
                                                .with_children(|track| {
                                                    track.spawn(theme::slider::track());
                                                    track
                                                        .spawn(Node {
                                                            position_type: PositionType::Absolute,
                                                            left: px(0),
                                                            right: px(14),
                                                            top: px(0),
                                                            bottom: px(0),
                                                            ..default()
                                                        })
                                                        .with_children(|travel| {
                                                            travel.spawn((
                                                                Thumb,
                                                                SliderThumb,
                                                                theme::slider::thumb(0.),
                                                                BackgroundColor(theme::PRIMARY),
                                                            ));
                                                        });
                                                });
                                            });
                                    }
                                });
                                panel.spawn((SettingsHint, text(SETTINGS_HINT, 13.)));
                            });
                        });
                    },
                );
            });
        });
}
impl Control {
    pub(super) fn enabled(self, session: &Session) -> bool {
        if matches!(self, Self::Command(Action::Play)) && session.play.is_some() {
            return !session.has_modal();
        }
        if busy(session) {
            return false;
        }
        let selected = session
            .selected
            .or(session.path_selected)
            .or(session.marker_selected)
            .or(session.plant_selected)
            .or(session.particle_selected)
            .is_some();
        match self {
            Self::GizmoTool(_) => {
                mode(session) == EditorMode::Objects
                    && session.particle_selected.is_none()
                    && outliner::selection(session) != Some(outliner::Target::Spawn)
            }
            Self::Transform => {
                session.inspect_spawn
                    || session
                        .selected
                        .or(session.plant_selected)
                        .or(session.marker_selected)
                        .or(session.path_selected)
                        .or(session.particle_selected)
                        .is_some()
            }
            Self::ParticlePreview | Self::BindParticle => session.particle_selected.is_some(),
            Self::Point(_) | Self::WholePath => {
                session.path_mode && session.path_selected.is_some()
            }
            Self::Command(Action::InsertPoint) => {
                session.path_point.is_some()
                    && session
                        .editor
                        .zone()
                        .paths
                        .iter()
                        .find(|p| Some(p.id) == session.path_selected)
                        .is_some_and(|p| p.points.len() < theia_zone::paths::MAX_POINTS)
            }
            Self::Command(Action::RemovePoint) => {
                session.path_point.is_some()
                    && session
                        .editor
                        .zone()
                        .paths
                        .iter()
                        .find(|p| Some(p.id) == session.path_selected)
                        .is_some_and(|p| p.points.len() > 2)
            }
            Self::Frame => {
                (session.terrain_mode && session.editor.zone().terrain.is_some()) || selected
            }
            Self::Command(Action::Duplicate | Action::Ground) => selected,
            Self::Command(Action::Undo) => session.editor.history().0 > 0,
            Self::Command(Action::Redo) => session.editor.history().1 > 0,
            Self::Command(Action::Load) => session.path.is_some(),
            _ => true,
        }
    }
}
impl Setting {
    fn range(self) -> (f32, f32) {
        match self {
            Self::MoveGrid => (0., (snapping::STEPS.len() - 1) as f32),
            Self::PlacementSize(_) => (0.25, 32.),
            Self::Radius => (4., 32.),
            Self::Strength => (0.05, 8.),
            Self::Blend => (5., 100.),
            Self::Level => (-64., 64.),
            Self::ScatterRadius => (1., 32.),
            Self::ScatterCount => (1., 64.),
        }
    }
    fn value(self, session: &Session) -> f32 {
        match self {
            Self::MoveGrid => snapping::STEPS
                .iter()
                .position(|&step| step == session.move_step)
                .unwrap_or(3) as f32,
            Self::PlacementSize(axis) => session.placement_size[axis] as f32,
            Self::Radius => session.brush_radius as f32,
            Self::Strength => session.brush_strength,
            Self::Blend => {
                100. * if session.terrain_paint {
                    session.paint_blend
                } else {
                    session.sculpt_blend
                }
            }
            Self::Level => session.flatten_height as f32,
            Self::ScatterRadius => session.scatter_radius as f32,
            Self::ScatterCount => session.scatter_count as f32,
        }
    }
    fn visible(self, session: &Session) -> bool {
        if session.play.is_some() {
            return false;
        }
        match self {
            Self::MoveGrid => true,
            Self::PlacementSize(axis) => {
                mode(session) == EditorMode::Objects
                    && match session.palette {
                        Some(Primitive::Box) => true,
                        Some(Primitive::Sphere) => axis == 0,
                        Some(Primitive::Cylinder) => axis < 2,
                        None => false,
                    }
            }
            Self::ScatterRadius | Self::ScatterCount => {
                session.vegetation_mode && session.plant_scatter && session.plant_palette.is_some()
            }
            Self::Radius => session.terrain_mode,
            Self::Strength => {
                session.terrain_mode
                    && matches!(
                        session.brush_tool,
                        terrain::Tool::Raise | terrain::Tool::Lower
                    )
            }
            Self::Blend => {
                session.terrain_mode
                    && !matches!(
                        session.brush_tool,
                        terrain::Tool::Raise | terrain::Tool::Lower
                    )
            }
            Self::Level => {
                session.terrain_mode && matches!(session.brush_tool, terrain::Tool::Flatten)
            }
        }
    }
    fn label(self, session: &Session) -> String {
        let value = self.value(session);
        match self {
            Self::MoveGrid => format!("Move / placement step  {} m", session.move_step),
            Self::PlacementSize(axis) => match session.palette {
                Some(Primitive::Sphere) => format!("New sphere diameter  {value:.2} m"),
                Some(Primitive::Cylinder) => format!(
                    "New cylinder {}  {value:.2} m",
                    if axis == 0 { "diameter" } else { "height" }
                ),
                _ => format!("New box {}  {value:.2} m", ["X", "Y", "Z"][axis]),
            },
            Self::Radius => format!("Brush radius  {value:.1} m"),
            Self::Strength => format!("Strength  {value:.3} m / stamp"),
            Self::Blend => format!("Blend  {value:.2}%"),
            Self::Level => format!("Flatten level  {value:.2} m"),
            Self::ScatterRadius => format!("Scatter radius  {value:.1} m"),
            Self::ScatterCount => format!("Plants per click  {value:.0}"),
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn activate(
    event: On<PointerPress>,
    controls: Query<
        &Control,
        (
            Without<bevy::ui_widgets::MenuItem>,
            Without<sidebar::RailButton>,
        ),
    >,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
    mut requests: ResMut<Requests>,
    mut content: ResMut<outliner::Content>,
) {
    if event.button != PointerButton::Primary || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    execute(
        *control,
        &mut session,
        &mut view,
        &mut requests,
        &mut content,
    );
}
pub(super) fn execute(
    control: Control,
    session: &mut Session,
    view: &mut View,
    requests: &mut Requests,
    content: &mut outliner::Content,
) {
    if matches!(control, Control::Snap | Control::Handles | Control::Grid) {
        return;
    }
    if session.has_modal() {
        return;
    }
    if busy(session)
        && !(session.play.is_some() && matches!(control, Control::Command(Action::Play)))
    {
        session.status =
            "Finish or cancel the preview / leave playtest before using workspace controls".into();
        return;
    }
    if !control.enabled(session) {
        return;
    }
    match control {
        Control::New => new_document(session, view, content),
        Control::Mode(next) => {
            if mode(session) != next {
                session.set_mode(next);
            }
        }
        Control::Command(action) => perform(action, session, view),
        Control::Frame => {
            if session.path_mode {
                paths::focus(session, view);
            } else {
                selection::frame(session, view);
            }
        }
        Control::Point(direction) => {
            if let Some(path) = session
                .editor
                .zone()
                .paths
                .iter()
                .find(|p| Some(p.id) == session.path_selected)
            {
                let count = path.points.len();
                let slot = session.path_point.map_or(0, |i| i + 1);
                let next = if direction < 0 {
                    (slot + count) % (count + 1)
                } else {
                    (slot + 1) % (count + 1)
                };
                session.path_point = next.checked_sub(1);
            }
        }
        Control::WholePath => session.path_point = None,
        Control::View(preset) => preset.select(view),
        Control::GizmoTool(tool) => {
            session.gizmo_tool = tool;
            session.palette = None;
        }
        Control::Help => requests.help = !requests.help,
        Control::Rename => requests.name = true,
        Control::RenameZone => requests.name_zone = true,
        Control::Transform => requests.transform = true,
        Control::SpawnTransform => requests.spawn_transform = true,
        Control::FileOpen => requests.file_open = true,
        Control::FileSaveAs => requests.file_save_as = true,
        Control::ParticleWorkspace => requests.particle_workspace = true,
        Control::ParticlePreview => requests.particle_preview = true,
        Control::BindParticle => requests.bind_particle = true,
        Control::Snap | Control::Handles | Control::Grid => {}
    }
}
pub(super) fn change(
    event: On<ValueChange<f32>>,
    sliders: Query<&Setting>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
) {
    if busy(&session) || !windows.single().is_ok_and(|w| w.focused) || !event.value.is_finite() {
        return;
    }
    let Ok(setting) = sliders.get(event.source) else {
        return;
    };
    if !setting.visible(&session) {
        return;
    }
    let (minimum, maximum) = setting.range();
    let value = event.value.clamp(minimum, maximum);
    match setting {
        Setting::MoveGrid => session.move_step = snapping::STEPS[value.round() as usize],
        Setting::PlacementSize(axis) => {
            session.placement_size[*axis] = f64::from((value * 4.).round() / 4.);
        }
        Setting::Radius => session.brush_radius = f64::from(value),
        Setting::Strength => session.brush_strength = value,
        Setting::Blend => {
            if session.terrain_paint {
                session.paint_blend = value / 100.;
            } else {
                session.sculpt_blend = value / 100.;
            }
        }
        Setting::Level => session.flatten_height = f64::from(value),
        Setting::ScatterRadius => session.scatter_radius = f64::from(value),
        Setting::ScatterCount => session.scatter_count = value.round() as u32,
    }
    session.status = setting.label(&session);
}
pub(super) fn toggle(
    event: On<ValueChange<bool>>,
    controls: Query<&Control>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
) {
    if busy(&session) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    match controls.get(event.source) {
        Ok(Control::Grid) => view.show_grid = event.value,
        Ok(Control::Snap) => session.snap = event.value,
        Ok(Control::Handles) => session.show_move_handles = event.value,
        _ => {}
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn sync(
    mut commands: Commands,
    session: Res<Session>,
    requests: Res<Requests>,
    view: Res<View>,
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    mut help: Query<
        &mut Node,
        (
            With<Help>,
            Without<Thumb>,
            Without<SettingRow>,
            Without<Control>,
        ),
    >,
    mut controls: Query<
        (
            Entity,
            &Control,
            &Children,
            &Hovered,
            &mut BackgroundColor,
            &mut Node,
            Has<Checked>,
            Has<InteractionDisabled>,
            Has<sidebar::RailButton>,
            Has<bevy::ui_widgets::MenuItem>,
        ),
        Without<Thumb>,
    >,
    mut button_labels: Query<&mut Text, With<ButtonLabel>>,
    sliders: Query<(
        Entity,
        &Setting,
        &SliderValue,
        &SliderRange,
        Has<InteractionDisabled>,
    )>,
    hierarchy: Query<&Children>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor), With<Thumb>>,
    mut rows: Query<(&SettingRow, &mut Node), (Without<Thumb>, Without<Control>)>,
    mut labels: Query<(&SettingLabel, &mut Text, Has<SettingValue>), Without<ButtonLabel>>,
    mut captions: Query<&mut Text, (With<Caption>, Without<ButtonLabel>, Without<SettingLabel>)>,
    mut section_roots: Query<
        &mut Node,
        (
            With<sections::Root>,
            Without<Help>,
            Without<Thumb>,
            Without<SettingRow>,
            Without<Control>,
        ),
    >,
    mut hints: Query<
        &mut Text,
        (
            With<SettingsHint>,
            Without<ButtonLabel>,
            Without<SettingLabel>,
            Without<Caption>,
        ),
    >,
) {
    for mut node in &mut section_roots {
        node.display = if session.play.is_some() {
            Display::None
        } else {
            Display::Flex
        };
    }
    for mut node in &mut help {
        node.display = if requests.help {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (
        entity,
        control,
        children,
        hovered,
        mut background,
        mut node,
        checked,
        disabled,
        rail,
        menu_item,
    ) in &mut controls
    {
        node.display = if matches!(
            control,
            Control::Point(_)
                | Control::WholePath
                | Control::Command(Action::InsertPoint | Action::RemovePoint)
        ) && !session.path_mode
        {
            Display::None
        } else {
            Display::Flex
        };
        let enabled = control.enabled(&session);
        if disabled == enabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
        let active = match *control {
            Control::Mode(m) => m == mode(&session),
            Control::Snap => session.snap,
            Control::Handles => session.show_move_handles,
            Control::Grid => view.show_grid,
            Control::GizmoTool(tool) => session.gizmo_tool == tool,
            Control::View(preset) => view.preset == preset,
            Control::Help => requests.help,
            Control::WholePath => session.path_selected.is_some() && session.path_point.is_none(),
            Control::Command(Action::Play) => session.play.is_some(),
            _ => false,
        };
        background.0 =
            if rail || matches!(control, Control::Snap | Control::Handles | Control::Grid) {
                Color::NONE
            } else if !enabled {
                theme::Variant::Disabled.background(false)
            } else if active {
                theme::Variant::Active.background(false)
            } else if menu_item {
                theme::Variant::Ghost.background(hovered.0)
            } else if hovered.0 {
                theme::Variant::Default.background(true)
            } else {
                theme::Variant::Default.background(false)
            };
        if matches!(control, Control::Snap | Control::Handles | Control::Grid) && checked != active
        {
            if active {
                commands.entity(entity).insert(Checked);
            } else {
                commands.entity(entity).remove::<Checked>();
            }
        }
        let grid_caption = format!(
            "Grid ({} m{})",
            authoring_grid::spacing(&session, &view),
            if authoring_grid::surface(&session, &view) {
                " surface"
            } else {
                ""
            }
        );
        let snap_caption = format!("Snap ({} m / 15 deg / 10%)", session.move_step);
        let caption = match control {
            Control::Grid => Some(grid_caption.as_str()),
            Control::Snap => Some(snap_caption.as_str()),
            Control::Handles => Some("Handles"),
            Control::Command(Action::Play) => Some(if session.play.is_some() {
                "Return to editing"
            } else {
                "Playtest"
            }),
            _ => None,
        };
        if let Some(caption) = caption {
            for child in children.iter() {
                if let Ok(mut text) = button_labels.get_mut(child) {
                    **text = caption.into();
                }
            }
        }
    }
    for (entity, setting, value, range, disabled) in &sliders {
        let enabled = !busy(&session) && setting.visible(&session);
        if disabled == enabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
        let actual = setting.value(&session);
        if value.0 != actual {
            commands.entity(entity).insert(SliderValue(actual));
        }
        for child in hierarchy.iter_descendants(entity) {
            if let Ok((mut node, mut color)) = thumbs.get_mut(child) {
                color.0 = if !enabled {
                    theme::MUTED
                } else if focus
                    .as_ref()
                    .is_some_and(|focus| focus.get() == Some(entity))
                {
                    theme::DISPLAY
                } else {
                    theme::PRIMARY
                };
                node.left = percent(range.thumb_position(actual) * 100.);
            }
        }
    }
    for (setting, mut node) in &mut rows {
        node.display = if setting.0.visible(&session) {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (setting, mut text, value) in &mut labels {
        let caption = setting.0.label(&session);
        let (label, number) = caption.split_once("  ").unwrap_or((&caption, ""));
        **text = if value { number } else { label }.into();
    }
    for mut text in &mut captions {
        **text = if session.play.is_some() {
            "Playtest controls".into()
        } else {
            format!("Properties · {:?}", mode(&session))
        };
    }
    for mut text in &mut hints {
        **text = if session.play.is_some() {
            "F5 returns to editing.\nAuthoring pauses during playtest."
        } else {
            SETTINGS_HINT
        }
        .into();
    }
}

#[cfg(test)]
#[path = "workspace_keyboard_tests.rs"]
mod keyboard_tests;
#[cfg(test)]
#[path = "placement_tests.rs"]
mod placement_tests;
