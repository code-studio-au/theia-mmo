//! Application shell and standard Bevy controls over the retained editor operations.
use super::*;
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::ui::{Checked, InteractionDisabled};
use bevy::ui_widgets::{
    Checkbox, Slider, SliderPrecision, SliderRange, SliderStep, SliderThumb, SliderValue,
    TrackClick, ValueChange,
};

pub(super) const HEADER: f32 = 92.;
pub(super) const INSPECTOR: f32 = 600.;
#[derive(Resource, Default)]
pub(super) struct Requests {
    pub name: bool,
    pub transform: bool,
    pub help: bool,
}
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    Mode(EditorMode),
    Command(Action),
    Frame,
    Rename,
    Transform,
    Point(i8),
    WholePath,
    Snap,
    Help,
}
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Setting {
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
pub(super) struct Thumb;
#[derive(Component)]
pub(super) struct Caption;
#[derive(Component)]
pub(super) struct ButtonLabel;

fn busy(session: &Session) -> bool {
    session.has_modal()
        || session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
}
fn mode(session: &Session) -> EditorMode {
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
        Node {
            min_height: px(30),
            padding: UiRect::axes(px(10), px(6)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.10, 0.14, 0.19)),
        children![(ButtonLabel, text(caption, 15.))],
    )
}
pub(super) fn setup(mut commands: Commands) {
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
                padding: UiRect::axes(px(14), px(6)),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                ..default()
            },
            BackgroundColor(Color::srgb(0.025, 0.035, 0.05)),
        ))
        .with_children(|root| {
            root.spawn(Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn(text("THEIA  /  ZONE STUDIO", 18.));
                for (control, label) in [
                    (Control::Command(Action::Save), "Save"),
                    (Control::Command(Action::Load), "Open saved"),
                    (Control::Command(Action::Undo), "Undo"),
                    (Control::Command(Action::Redo), "Redo"),
                    (Control::Frame, "Frame selection"),
                    (Control::Command(Action::Spawn), "Spawn at view centre"),
                    (Control::Command(Action::Play), "Playtest"),
                    (Control::Help, "Shortcuts"),
                ] {
                    row.spawn(button(control, label));
                }
                row.spawn((
                    Control::Snap,
                    Checkbox,
                    Hovered::default(),
                    Node {
                        min_height: px(30),
                        padding: UiRect::axes(px(10), px(6)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.14, 0.19)),
                    children![(ButtonLabel, text("[x] Snap to 1 m", 15.))],
                ));
            });
            root.spawn(Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn(text("WORKSPACE", 13.));
                for (mode, label) in [
                    (EditorMode::Objects, "Objects"),
                    (EditorMode::Sculpt, "Sculpt"),
                    (EditorMode::Paint, "Paint"),
                    (EditorMode::Guides, "Paths / streams"),
                    (EditorMode::Markers, "NPC / quest markers"),
                    (EditorMode::Plants, "Vegetation"),
                ] {
                    row.spawn(button(Control::Mode(mode), label));
                }
            });
        });
    commands.spawn((GlobalZIndex(4), Node {
        position_type:PositionType::Absolute,right:px(0),bottom:px(0),width:px(outliner::WIDTH),height:px(INSPECTOR),
        padding:UiRect::all(px(12)),flex_direction:FlexDirection::Column,row_gap:px(8),overflow:Overflow::clip(),..default()
    },BackgroundColor(Color::srgb(0.045,0.06,0.08))))
    .with_children(|panel| {
        panel.spawn((Caption,text("PROPERTIES",19.)));
        panel.spawn((Info,text("",14.)));
        panel.spawn(Node { flex_wrap:FlexWrap::Wrap,column_gap:px(6),row_gap:px(6),..default() }).with_children(|row| {
            for (control,label) in [(Control::Rename,"Rename"),(Control::Transform,"Transform..."),
                (Control::Command(Action::Duplicate),"Duplicate"),(Control::Command(Action::Ground),"Ground"),
                (Control::Frame,"Frame")] { row.spawn(button(control,label)); }
        });
        panel.spawn(Node { flex_wrap:FlexWrap::Wrap,column_gap:px(6),row_gap:px(6),..default() }).with_children(|row| {
            for (control,label) in [(Control::Point(-1),"Previous point"),(Control::Point(1),"Next point"),
                (Control::WholePath,"Whole guide"),(Control::Command(Action::InsertPoint),"Insert point"),
                (Control::Command(Action::RemovePoint),"Remove point")] { row.spawn(button(control,label)); }
        });
        for setting in [Setting::Radius,Setting::Strength,Setting::Blend,Setting::Level,Setting::ScatterRadius,Setting::ScatterCount] {
            let (minimum,maximum) = setting.range();
            panel.spawn((SettingRow(setting), Node { flex_direction:FlexDirection::Column,row_gap:px(6),..default() }))
                .with_children(|row| {
                    row.spawn((SettingLabel(setting),text("",15.)));
                    row.spawn((setting, Slider { track_click:TrackClick::Snap,..default() },SliderValue(minimum),
                        SliderRange::new(minimum,maximum),SliderStep(match setting { Setting::ScatterCount=>1.,Setting::Level|Setting::ScatterRadius=>0.1,_=>0.05 }),SliderPrecision(if matches!(setting,Setting::ScatterCount){0}else{3}),
                        Hovered::default(),Node {width:percent(100),height:px(24),..default()},
                        BackgroundColor(Color::srgb(0.09,0.12,0.16))))
                    .with_children(|track| {
                        track.spawn(Node { position_type:PositionType::Absolute,left:px(0),right:px(14),top:px(0),bottom:px(0),..default() })
                            .with_children(|travel| {travel.spawn((Thumb,SliderThumb,Node {
                                position_type:PositionType::Absolute,width:px(14),height:px(24),left:percent(0),..default()
                            },BackgroundColor(Color::srgb(0.2,0.65,0.8))));});
                    });
                });
        }
        panel.spawn(text("Tool settings affect the next edit.\nCtrl S saves the zone.\nCtrl Z / Ctrl Y: undo / redo",13.));
    });
}
impl Control {
    fn enabled(self, session: &Session) -> bool {
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
            .is_some();
        match self {
            Self::Transform => session
                .selected
                .or(session.plant_selected)
                .or(session.marker_selected)
                .or(session.path_selected)
                .is_some(),
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
            _ => true,
        }
    }
}
impl Setting {
    fn range(self) -> (f32, f32) {
        match self {
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
        match self {
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
            Self::Radius => format!("Brush radius  {value:.1} m"),
            Self::Strength => format!("Strength  {value:.3} m / stamp"),
            Self::Blend => format!("Blend  {value:.2}%"),
            Self::Level => format!("Flatten level  {value:.2} m"),
            Self::ScatterRadius => format!("Scatter radius  {value:.1} m"),
            Self::ScatterCount => format!("Plants per click  {value:.0}"),
        }
    }
}
pub(super) fn activate(
    event: On<PointerPress>,
    controls: Query<&Control>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
    mut requests: ResMut<Requests>,
) {
    if event.button != PointerButton::Primary || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    if matches!(control, Control::Snap) {
        return;
    }
    if session.has_modal() {
        return;
    }
    if busy(&session)
        && !(session.play.is_some() && matches!(control, Control::Command(Action::Play)))
    {
        session.status =
            "Finish or cancel the preview / leave playtest before using workspace controls".into();
        return;
    }
    if !control.enabled(&session) {
        return;
    }
    match *control {
        Control::Mode(next) => {
            if mode(&session) != next {
                session.set_mode(next);
            }
        }
        Control::Command(action) => perform(action, &mut session, &mut view),
        Control::Frame => {
            if session.path_mode {
                paths::focus(&session, &mut view);
            } else {
                selection::frame(&mut session, &mut view);
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
        Control::Help => requests.help = !requests.help,
        Control::Rename => requests.name = true,
        Control::Transform => requests.transform = true,
        Control::Snap => {}
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
) {
    if busy(&session) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if matches!(controls.get(event.source), Ok(Control::Snap)) {
        session.snap = event.value;
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn sync(
    mut commands: Commands,
    session: Res<Session>,
    requests: Res<Requests>,
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
    mut labels: Query<(&SettingLabel, &mut Text), Without<ButtonLabel>>,
    mut captions: Query<&mut Text, (With<Caption>, Without<ButtonLabel>, Without<SettingLabel>)>,
) {
    for mut node in &mut help {
        node.display = if requests.help {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (entity, control, children, hovered, mut background, mut node, checked, disabled) in
        &mut controls
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
            Control::Help => requests.help,
            Control::WholePath => session.path_selected.is_some() && session.path_point.is_none(),
            Control::Command(Action::Play) => session.play.is_some(),
            _ => false,
        };
        background.0 = if !enabled {
            Color::srgb(0.055, 0.07, 0.09)
        } else if active {
            Color::srgb(0.13, 0.35, 0.46)
        } else if hovered.0 {
            Color::srgb(0.19, 0.25, 0.32)
        } else {
            Color::srgb(0.10, 0.14, 0.19)
        };
        if matches!(control, Control::Snap) && checked != session.snap {
            if session.snap {
                commands.entity(entity).insert(Checked);
            } else {
                commands.entity(entity).remove::<Checked>();
            }
        }
        let caption = match control {
            Control::Snap => Some(if session.snap {
                "[x] Snap to 1 m"
            } else {
                "[ ] Snap to 1 m"
            }),
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
                color.0 = if enabled {
                    Color::srgb(0.2, 0.65, 0.8)
                } else {
                    Color::srgb(0.2, 0.25, 0.3)
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
    for (setting, mut text) in &mut labels {
        **text = setting.0.label(&session);
    }
    for mut text in &mut captions {
        **text = format!("PROPERTIES  /  {:?}", mode(&session));
    }
}
