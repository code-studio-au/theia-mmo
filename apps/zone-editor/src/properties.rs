//! Numeric object, plant and marker transforms; standard text/focus widgets, existing commands.
use super::*;
#[path = "properties_area.rs"]
mod area;
#[path = "properties_fields.rs"]
mod fields_ui;
#[path = "properties_inline.rs"]
mod inline;
#[path = "properties_scrub.rs"]
mod numeric;
#[path = "properties_summary.rs"]
mod summary;
use bevy::input_focus::{
    FocusCause, InputFocus,
    tab_navigation::{NavAction, TabGroup, TabIndex, TabNavigation},
};
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::text::EditableText;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{
    SelectAllOnFocus, Slider, SliderPrecision, SliderRange, SliderStep, SliderThumb, SliderValue,
    TextInput, TrackClick, ValueChange,
};
pub(super) use inline::Input as InlineInput;
pub(super) use numeric::scrub;
pub(super) use summary::Frame as InlineFrame;
pub(super) use summary::Name as InlineName;
pub(super) use summary::Value as SummaryValue;
use theia_zone::{
    markers::Marker, particles::ParticlePlacement, paths::GuidePath, vegetation::Plant,
};

const LABELS: [&str; 7] = [
    "World X (m)",
    "World Y (m)",
    "World Z (m)",
    "Yaw (degrees)",
    "Size X (m)",
    "Size Y (m)",
    "Size Z (m)",
];
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Caption;
#[derive(Component)]
pub(super) struct Field(pub usize);
#[derive(Component)]
pub(super) struct FieldRow(usize);
#[derive(Component)]
pub(super) struct YawRow;
#[derive(Component)]
pub(super) struct YawSlider;
#[derive(Component)]
pub(super) struct YawThumb;
#[derive(Component)]
pub(super) struct YawLabel;
#[derive(Component)]
pub(super) struct SliderRow(Control);
#[derive(Component)]
pub(super) struct SliderLabel(Control);
#[derive(Component)]
pub(super) struct SliderHandle(Control);
#[derive(Clone, Copy)]
struct SliderSpec {
    field: usize,
    min: f32,
    max: f32,
    step: f32,
    label: &'static str,
    yaw: bool,
}
impl SliderSpec {
    fn display(self, text: &str) -> Option<f32> {
        if self.yaw {
            return displayed_yaw(text);
        }
        let value = text.parse::<f64>().ok()?;
        (value.is_finite() && (value as f32).is_finite()).then_some(value as f32)
    }
    fn valid(self, text: &str) -> bool {
        text.parse::<f64>().is_ok_and(|value| {
            value.is_finite()
                && if self.yaw {
                    value.abs() <= 3600.
                } else {
                    (f64::from(self.min)..=f64::from(self.max)).contains(&value)
                }
        })
    }
}
#[derive(Component)]
pub(super) struct FieldLabel(pub(super) usize);
#[derive(Component)]
pub(super) struct FramedInput;
#[derive(Component)]
pub(super) struct FieldUnit(usize);
#[derive(Component)]
pub(super) enum FieldGroup {
    PositionHeading,
    SizeHeading,
    Tail,
}
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Spawn([f64; 3]),
    Object(Pose),
    Plant(Plant),
    Particle(ParticlePlacement),
    Scenery(theia_zone::scenery::SceneryPlacement),
    TestPlayer(theia_zone::test_players::TestPlayer),
    QuestArea(theia_zone::quest_areas::QuestArea),
    Marker(Marker),
    Path {
        path: GuidePath,
        point: Option<usize>,
    },
}
impl Target {
    fn slider(&self, control: Control, snap: bool) -> Option<SliderSpec> {
        let (field, min, max, step, label, yaw) = match (control, self) {
            (Control::YawSlider, _) if self.has_yaw() => (
                3,
                -180.,
                180.,
                if snap { 15. } else { 1. },
                if snap {
                    "Yaw slider | 15 deg snap"
                } else {
                    "Yaw slider | 1 deg step"
                },
                true,
            ),
            (Control::ValueSlider, Self::Plant(_)) => {
                (4, 0.25, 8., 0.25, "Plant scale | step 0.25", false)
            }
            (Control::ValueSlider, Self::Particle(_)) => {
                (4, 0.01, 64., 0.25, "Particle scale | step 0.25", false)
            }
            (Control::ValueSlider, Self::Marker(_)) => {
                (4, 0.25, 64., 0.25, "Radius (m) | step 0.25", false)
            }
            (Control::ValueSlider, Self::Path { point, .. }) => (
                if point.is_some() { 3 } else { 0 },
                0.25,
                32.,
                0.25,
                "Guide width (m) | step 0.25",
                false,
            ),
            _ => return None,
        };
        Some(SliderSpec {
            field,
            min,
            max,
            step,
            label,
            yaw,
        })
    }
    fn has_yaw(&self) -> bool {
        matches!(
            self,
            Self::Object(_)
                | Self::Plant(_)
                | Self::Marker(_)
                | Self::Particle(_)
                | Self::Scenery(_)
        )
    }
    fn labels(&self) -> &'static [&'static str] {
        match self {
            Self::Object(_) => &LABELS,
            Self::QuestArea(_) => &[
                "Min X (m)",
                "Min Y (m)",
                "Min Z (m)",
                "Max X (m)",
                "Max Y (m)",
                "Max Z (m)",
            ],
            Self::TestPlayer(_) => &[
                "World X (m)",
                "World Y (m)",
                "World Z (m)",
                "Delay (ms/way)",
                "Jitter (ms)",
                "Packet loss (%)",
                "Random seed",
            ],
            Self::Spawn(_) => &["World X (m)", "World Y (m)", "World Z (m)"],
            Self::Plant(_) | Self::Particle(_) | Self::Scenery(_) => &[
                "World X (m)",
                "World Y (m)",
                "World Z (m)",
                "Yaw (degrees)",
                "Scale",
            ],
            Self::Marker(_) => &[
                "World X (m)",
                "World Y (m)",
                "World Z (m)",
                "Yaw (degrees)",
                "Radius (m)",
            ],
            Self::Path { point: None, .. } => &["Width (m)"],
            Self::Path { point: Some(_), .. } => {
                &["World X (m)", "World Y (m)", "World Z (m)", "Width (m)"]
            }
        }
    }
    fn values(&self) -> Vec<String> {
        match self {
            Self::Object(pose) => values(pose).to_vec(),
            Self::QuestArea(area) => area
                .min
                .into_iter()
                .chain(area.max)
                .map(|v| v.to_string())
                .collect(),
            Self::TestPlayer(p) => [
                p.position[0],
                p.position[1],
                p.position[2],
                f64::from(p.network.delay_ms),
                f64::from(p.network.jitter_ms),
                p.network.loss_percent,
                f64::from(p.network.seed),
            ]
            .map(|v| v.to_string())
            .to_vec(),
            Self::Spawn(position) => position.map(|v| v.to_string()).to_vec(),
            Self::Plant(p) => [
                p.position[0],
                p.position[1],
                p.position[2],
                p.yaw_degrees,
                p.scale,
            ]
            .map(|v| v.to_string())
            .to_vec(),
            Self::Scenery(p) => [
                p.position[0],
                p.position[1],
                p.position[2],
                p.yaw_degrees,
                p.scale,
            ]
            .map(|v| v.to_string())
            .to_vec(),
            Self::Particle(p) => [
                p.position[0],
                p.position[1],
                p.position[2],
                p.yaw_degrees,
                p.scale,
            ]
            .map(|v| v.to_string())
            .to_vec(),
            Self::Marker(m) => [
                m.position[0],
                m.position[1],
                m.position[2],
                m.yaw_degrees,
                m.radius_metres,
            ]
            .map(|v| v.to_string())
            .to_vec(),
            Self::Path { path, point } => {
                let mut values =
                    point.map_or_else(Vec::new, |i| path.points[i].map(|v| v.to_string()).to_vec());
                values.push(path.width_metres.to_string());
                values
            }
        }
    }
    fn edit(&self, id: Option<u64>, fields: &[String]) -> Result<Option<Edit>, String> {
        match self {
            Self::QuestArea(original) => {
                let v = numbers(fields, self.labels())?;
                let mut area = original.clone();
                area.min = [v[0], v[1], v[2]];
                area.max = [v[3], v[4], v[5]];
                Ok((area != *original).then_some(Edit::PutQuestArea { area }))
            }
            Self::TestPlayer(original) => {
                let v = numbers(fields, self.labels())?;
                let mut player = original.clone();
                player.position = [v[0], v[1], v[2]];
                if [3, 4, 6]
                    .iter()
                    .any(|&i| v[i].fract() != 0. || !(0. ..=f64::from(u32::MAX)).contains(&v[i]))
                {
                    return Err("Delay, jitter and seed require nonnegative whole numbers".into());
                }
                player.network.delay_ms = v[3] as u32;
                player.network.jitter_ms = v[4] as u32;
                player.network.loss_percent = v[5];
                player.network.seed = v[6] as u32;
                player.network.validate()?;
                Ok((player != *original).then_some(Edit::PutTestPlayer { player }))
            }
            Self::Spawn(original) => {
                let v = numbers(fields, self.labels())?;
                let position = [v[0], v[1], v[2]];
                Ok((position != *original).then_some(Edit::SetSpawn { position }))
            }
            Self::Object(original) => {
                let pose = parse(fields)?;
                Ok((pose != *original).then_some(Edit::Transform {
                    id: id.ok_or("Missing object ID")?,
                    pose,
                }))
            }
            Self::Plant(original) => {
                let v = numbers(fields, self.labels())?;
                let mut plant = original.clone();
                plant.position = [v[0], v[1], v[2]];
                plant.yaw_degrees = v[3];
                plant.scale = v[4];
                Ok((plant != *original).then_some(Edit::PutPlant { plant }))
            }
            Self::Scenery(original) => {
                let v = numbers(fields, self.labels())?;
                let mut scenery = original.clone();
                scenery.position = [v[0], v[1], v[2]];
                scenery.yaw_degrees = v[3];
                scenery.scale = v[4];
                Ok((scenery != *original).then_some(Edit::PutScenery { scenery }))
            }
            Self::Particle(original) => {
                let v = numbers(fields, self.labels())?;
                let mut particle = original.clone();
                particle.position = [v[0], v[1], v[2]];
                particle.yaw_degrees = v[3];
                particle.scale = v[4];
                Ok((particle != *original).then_some(Edit::PutParticle { particle }))
            }
            Self::Marker(original) => {
                let v = numbers(fields, self.labels())?;
                let mut marker = original.clone();
                marker.position = [v[0], v[1], v[2]];
                marker.yaw_degrees = v[3];
                marker.radius_metres = v[4];
                Ok((marker != *original).then_some(Edit::PutMarker { marker }))
            }
            Self::Path {
                path: original,
                point,
            } => {
                let v = numbers(fields, self.labels())?;
                let mut path = original.clone();
                path.width_metres = *v.last().unwrap();
                if let Some(index) = point {
                    *path
                        .points
                        .get_mut(*index)
                        .ok_or("Unknown guide point index")? = [v[0], v[1], v[2]];
                }
                Ok((path != *original).then_some(Edit::PutPath { path }))
            }
        }
    }
}
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Control {
    Apply,
    Reset,
    Cancel,
    YawSlider,
    ValueSlider,
}
pub(super) struct Draft {
    pub id: Option<u64>,
    revision: u64,
    original: Target,
    subject: String,
    pub fields: Vec<String>,
    pub active: usize,
    pub selection: [usize; 2],
    action: Option<Control>,
}
fn values(pose: &Pose) -> [String; 7] {
    [
        pose.position[0],
        pose.position[1],
        pose.position[2],
        pose.yaw_degrees,
        pose.size[0],
        pose.size[1],
        pose.size[2],
    ]
    .map(|v| v.to_string())
}
fn numbers(fields: &[String], labels: &[&str]) -> Result<Vec<f64>, String> {
    if fields.len() != labels.len() {
        return Err("Incorrect transform field count".into());
    }
    fields
        .iter()
        .zip(labels)
        .map(|(value, label)| {
            value
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("{label} must be a finite number"))
        })
        .collect()
}
fn parse(fields: &[String]) -> Result<Pose, String> {
    let values = numbers(fields, &LABELS)?;
    Ok(Pose {
        position: [values[0], values[1], values[2]],
        yaw_degrees: values[3],
        size: [values[4], values[5], values[6]],
    })
}

pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            Root,
            panels::Region::Inspector,
            TabGroup::modal(),
            GlobalZIndex(20),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: px(sidebar::WIDTH),
                top: px(workspace::HEADER),
                bottom: px(0),
                width: px(workspace::INSPECTOR_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexStart,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme::PANEL),
        ))
        .with_children(|root| {
            scrolling::pane(root, 0., scrolling::Pane::Attributes, |root| {
                root.spawn((
                    Node {
                        width: percent(100),
                        padding: px(12).all(),
                        row_gap: px(6),
                        flex_direction: FlexDirection::Column,
                        border: px(1).all(),
                        ..default()
                    },
                    BorderColor::all(theme::BORDER),
                    BackgroundColor(theme::PANEL),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Caption,
                        Text::new("Properties"),
                        TextFont {
                            weight: FontWeight::SEMIBOLD,
                            ..theme::font(12.)
                        },
                        TextColor(theme::DISPLAY),
                        TextLayout::no_wrap(),
                    ));
                    panel.spawn(text("Drag labels | click values for exact input", 13.));
                    fields_ui::spawn(panel);
                    for control in [Control::YawSlider, Control::ValueSlider] {
                        let yaw = control == Control::YawSlider;
                        let mut container = panel.spawn((
                            SliderRow(control),
                            Visibility::Hidden,
                            Node {
                                display: Display::None,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(4),
                                ..default()
                            },
                        ));
                        if yaw {
                            container.insert(YawRow);
                        }
                        container.with_children(|row| {
                            let mut label =
                                row.spawn((SliderLabel(control), theme::field_label("")));
                            if yaw {
                                label.insert(YawLabel);
                            }
                            let mut slider = row.spawn((
                                control,
                                Slider {
                                    track_click: TrackClick::Snap,
                                    ..default()
                                },
                                SliderValue(if yaw { 0. } else { 1. }),
                                SliderRange::new(
                                    if yaw { -180. } else { 0.25 },
                                    if yaw { 180. } else { 8. },
                                ),
                                SliderStep(if yaw { 15. } else { 0.25 }),
                                SliderPrecision(if yaw { 0 } else { 2 }),
                                TabIndex(if yaw { 7 } else { 8 }),
                                Hovered::default(),
                                theme::slider::node(),
                                BackgroundColor(Color::NONE),
                            ));
                            if yaw {
                                slider.insert(YawSlider);
                            }
                            slider.with_children(|track| {
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
                                        let mut thumb = travel.spawn((
                                            SliderHandle(control),
                                            SliderThumb,
                                            theme::slider::thumb(50.),
                                            BackgroundColor(theme::PRIMARY),
                                        ));
                                        if yaw {
                                            thumb.insert(YawThumb);
                                        }
                                    });
                            });
                        });
                    }
                    panel.spawn(text(
                        "Enter applies all fields.\nEscape cancels. Tab changes field.",
                        13.,
                    ));
                    panel
                        .spawn(Node {
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            for (control, label) in [
                                (Control::Apply, "Apply"),
                                (Control::Reset, "Reset fields"),
                                (Control::Cancel, "Cancel"),
                            ] {
                                row.spawn((
                                    control,
                                    Button,
                                    Hovered::default(),
                                    theme::button_node(),
                                    BackgroundColor(theme::Variant::Default.background(false)),
                                ))
                                .with_child(theme::label(
                                    label,
                                    12.,
                                    theme::BODY,
                                ));
                            }
                        });
                });
            });
        });
}

pub(super) fn activate(
    event: On<PointerPress>,
    controls: Query<&Control>,
    mut session: ResMut<Session>,
    windows: Query<&Window>,
) {
    if event.button == PointerButton::Primary
        && windows.single().is_ok_and(|window| window.focused)
        && let Ok(control) = controls.get(event.entity)
        && let Some(draft) = &mut session.transform_edit
    {
        draft.action = Some(*control);
    }
}

pub(super) fn summary_plugin(app: &mut App) {
    app.add_plugins(summary::plugin);
}
pub(super) fn setup_summary(parent: &mut ChildSpawnerCommands) {
    summary::spawn(parent);
}
pub(super) fn summary_subject(session: &Session) -> Option<String> {
    summary::description(session)
}

type InlineFields<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static summary::Value>,
        &'static EditableText,
    ),
    (With<inline::Input>, Without<Field>),
>;
#[allow(clippy::too_many_arguments)]
pub(super) fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    nav: TabNavigation,
    mut fields: Query<(Entity, &Field, &mut EditableText)>,
    mut requests: ResMut<workspace::Requests>,
    inline: Option<Res<inline::State>>,
    inputs: InlineFields,
    mut commands: Commands,
    mut tabs: Option<ResMut<property_tabs::State>>,
) {
    let requested = std::mem::take(&mut requests.transform);
    let requested_field = std::mem::take(&mut requests.transform_field);
    let spawn_requested = std::mem::take(&mut requests.spawn_transform);
    if !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if session.inline_editing && keys.just_pressed(KeyCode::Tab) {
        if focus
            .get()
            .and_then(|entity| inputs.get(entity).ok())
            .is_some_and(|(_, _, text)| !text.is_composing())
        {
            let previous = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            match nav.navigate(
                &focus,
                if previous {
                    NavAction::Previous
                } else {
                    NavAction::Next
                },
            ) {
                Ok(entity) => {
                    focus.set(entity, FocusCause::Navigated);
                    commands.trigger(bevy::ui_widgets::ScrollIntoView { entity });
                }
                Err(error) => session.status = format!("Field navigation: {error}"),
            }
        }
        return;
    }
    if session.transform_edit.is_some() {
        if keys.just_pressed(KeyCode::Tab)
            && !focus
                .get()
                .and_then(|e| fields.get(e).ok())
                .is_some_and(|(_, _, f)| f.is_composing())
        {
            let previous = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            match nav.navigate(
                &focus,
                if previous {
                    NavAction::Previous
                } else {
                    NavAction::Next
                },
            ) {
                Ok(next) => focus.set(next, FocusCause::Navigated),
                Err(error) => session.status = format!("Field navigation: {error}"),
            }
        }
        return;
    }
    if !(keys.just_pressed(KeyCode::F9) || requested || spawn_requested) || session.has_modal() {
        return;
    }
    if session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
    {
        session.status = "Finish or cancel the preview before editing properties".into();
        return;
    }
    if spawn_requested
        && inline.is_some()
        && outliner::selection(&session) != Some(outliner::Target::Spawn)
    {
        // Let the summary refresh to the spawn snapshot before focusing a field.
        session.set_mode(EditorMode::Objects);
        session.inspect_spawn = true;
        session.ui.properties_collapsed = false;
        if let Some(tabs) = tabs.as_deref_mut() {
            tabs.active = property_tabs::Tab::Attributes;
        }
        requests.transform = true;
        return;
    }
    let Some((id, original, subject)) = capture(
        &session,
        spawn_requested || outliner::selection(&session) == Some(outliner::Target::Spawn),
    ) else {
        session.status = "Select an object, plant, marker or guide to edit properties".into();
        return;
    };
    let active = requested_field
        .unwrap_or(0)
        .min(original.labels().len().saturating_sub(1));
    if inline.is_some() {
        if let Some((entity, _, _)) = inputs
            .iter()
            .find(|(_, input, _)| input.is_some_and(|value| value.0 == active))
        {
            session.ui.properties_collapsed = false;
            if let Some(mut tabs) = tabs {
                tabs.active = property_tabs::Tab::Attributes;
            }
            focus.set(entity, FocusCause::Navigated);
            commands.trigger(bevy::ui_widgets::ScrollIntoView { entity });
        }
        return;
    }
    let draft = Draft {
        id,
        revision: session.editor.revision(),
        fields: original.values(),
        original,
        subject,
        active,
        selection: [0; 2],
        action: None,
    };
    for (entity, field, mut text) in &mut fields {
        // Preserve the font/layout styles Bevy prepared for this reusable field.
        text.clear();
        text.editor_mut()
            .set_text(draft.fields.get(field.0).map_or("", String::as_str));
        if field.0 == active {
            focus.set(entity, FocusCause::Navigated);
        }
    }
    session.transform_edit = Some(draft);
    session.status = "Editing transform - Apply validates all fields together".into();
}

fn capture(session: &Session, spawn_requested: bool) -> Option<(Option<u64>, Target, String)> {
    let zone = session.editor.zone();
    let result = if spawn_requested {
        (
            None,
            Target::Spawn(zone.spawn),
            "Playtest spawn | world metres\nClearance checked on playtest".into(),
        )
    } else if let Some(object) = zone.objects.iter().find(|o| Some(o.id) == session.selected) {
        (
            Some(object.id),
            Target::Object(object.pose.clone()),
            format!("{:?} #{} | {}", object.primitive, object.id, object.name),
        )
    } else if let Some(model) = zone
        .scenery
        .iter()
        .find(|p| Some(p.id) == session.scenery_selected)
    {
        (
            Some(model.id),
            Target::Scenery(model.clone()),
            format!("Scenery #{} | {}", model.id, model.name),
        )
    } else if let Some(p) = zone
        .test_players
        .iter()
        .find(|p| Some(p.id) == session.test_player_selected)
    {
        (
            Some(p.id),
            Target::TestPlayer(p.clone()),
            format!("Test player {} | {}", p.client_id, p.name),
        )
    } else if let Some(area) = zone
        .quest_areas
        .iter()
        .find(|a| Some(a.id) == session.quest_area_selected)
    {
        (
            Some(area.id),
            Target::QuestArea(area.clone()),
            format!("Quest area #{} | {}", area.id, area.name),
        )
    } else if let Some(particle) = zone
        .particles
        .iter()
        .find(|p| Some(p.id) == session.particle_selected)
    {
        (
            Some(particle.id),
            Target::Particle(particle.clone()),
            format!("Particle #{} | {}", particle.id, particle.name),
        )
    } else if let Some(plant) = zone
        .plants
        .iter()
        .find(|p| Some(p.id) == session.plant_selected)
    {
        (
            Some(plant.id),
            Target::Plant(plant.clone()),
            format!("{:?} #{} | {}", plant.kind, plant.id, plant.name),
        )
    } else if let Some(marker) = zone
        .markers
        .iter()
        .find(|m| Some(m.id) == session.marker_selected)
    {
        (
            Some(marker.id),
            Target::Marker(marker.clone()),
            format!("{:?} #{} | {}", marker.kind, marker.id, marker.name),
        )
    } else {
        let path = zone
            .paths
            .iter()
            .find(|p| Some(p.id) == session.path_selected)?;
        let point = session.path_point.filter(|i| *i < path.points.len());
        (
            Some(path.id),
            Target::Path {
                path: path.clone(),
                point,
            },
            format!(
                "{:?} #{} | {}{}",
                path.kind,
                path.id,
                path.name,
                point.map_or_else(String::new, |i| format!("\nPoint {}", i + 1))
            ),
        )
    };
    Some(result)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    mut fields: Query<(Entity, &Field, &mut EditableText)>,
    mut roots: Query<&mut Node, With<Root>>,
    mut captions: Query<&mut Text, With<Caption>>,
    mut rows: Query<(&FieldRow, &mut Node, &mut Visibility), (Without<Root>, Without<FieldLabel>)>,
    mut labels: Query<(&FieldLabel, &mut Text, &mut Node), (Without<Caption>, Without<Root>)>,
    mut units: Query<(&FieldUnit, &mut Text), (Without<Caption>, Without<FieldLabel>)>,
    mut groups: Query<
        (&FieldGroup, &mut Node),
        (Without<Root>, Without<FieldRow>, Without<FieldLabel>),
    >,
    form_sliders: Query<(), (With<Slider>, With<Control>)>,
) {
    let Some(mut draft) = session.transform_edit.take() else {
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
        return;
    };
    let resetting = matches!(draft.action, Some(Control::Reset))
        && windows.single().is_ok_and(|window| window.focused);
    if resetting {
        draft.fields = draft.original.values();
        draft.active = 0;
        for (entity, field, mut text) in &mut fields {
            if let Some(value) = draft.fields.get(field.0) {
                text.clear();
                text.editor_mut().set_text(value);
                if field.0 == 0 {
                    focus.set(entity, FocusCause::Navigated);
                }
            }
        }
        session.status = "Transform fields reset".into();
    }
    let mut composing = false;
    let mut edited = false;
    for (row, mut node, mut visibility) in &mut rows {
        let visible = row.0 < draft.fields.len();
        // TabNavigation follows inherited visibility, not layout Display::None.
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (label, mut text, mut node) in &mut labels {
        if let Some((prefix, width, _)) = fields_ui::prefix(&draft.original, label.0) {
            **text = prefix.into();
            node.width = px(width);
        }
    }
    for (unit, mut text) in &mut units {
        if let Some((_, _, suffix)) = fields_ui::prefix(&draft.original, unit.0) {
            **text = suffix.into();
        }
    }
    for (group, mut node) in &mut groups {
        let visible = match group {
            FieldGroup::PositionHeading => {
                !matches!(draft.original, Target::Path { point: None, .. })
            }
            FieldGroup::SizeHeading => matches!(draft.original, Target::Object(_)),
            FieldGroup::Tail => draft.fields.len() > 4,
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (entity, field, text) in &fields {
        if field.0 >= draft.fields.len() {
            continue;
        }
        let value = text.value().to_string();
        edited |= value != draft.fields[field.0];
        draft.fields[field.0] = value;
        if focus.get() == Some(entity) {
            draft.active = field.0;
        }
        composing |= text.is_composing();
    }
    if edited {
        session.status = "Editing transform - Apply validates all fields together".into();
    }
    // Resolve after updating active, since query iteration order is unspecified.
    let active_entity =
        fields
            .iter()
            .find(|(_, f, _)| f.0 == draft.active)
            .map(|(entity, _, text)| {
                let range = text.editor().raw_selection().text_range();
                draft.selection = [range.start, range.end];
                entity
            });
    let cancel = !windows.single().is_ok_and(|w| w.focused)
        || keys.just_pressed(KeyCode::Escape)
        || matches!(draft.action, Some(Control::Cancel));
    let apply = !resetting
        && !composing
        && (keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || matches!(draft.action, Some(Control::Apply)));
    let close = if cancel {
        session.status = "Transform edit cancelled".into();
        true
    } else if apply {
        match draft
            .original
            .edit(draft.id, &draft.fields)
            .and_then(|edit| {
                let Some(edit) = edit else {
                    return Ok(false);
                };
                let count = session.selected_path_len();
                session.editor.execute(Command::Apply {
                    expected_revision: draft.revision,
                    edits: vec![edit],
                })?;
                session.reconcile_after_edit(count);
                Ok(true)
            }) {
            Ok(changed) => {
                session.status = if changed {
                    "Transform updated"
                } else {
                    "Transform unchanged"
                }
                .into();
                true
            }
            Err(error) => {
                session.status = error;
                false
            }
        }
    } else {
        false
    };
    if close {
        focus.clear();
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
    } else {
        if let Some(entity) = active_entity
            && focus.get() != Some(entity)
            && !focus
                .get()
                .is_some_and(|focused| form_sliders.contains(focused))
        {
            focus.set(entity, FocusCause::Navigated);
        }
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::Flex;
        }
        if let Ok(mut caption) = captions.single_mut() {
            caption.0 = format!("Properties\n{}", draft.subject);
        }
        draft.action = None;
        session.transform_edit = Some(draft);
    }
}

// Slider presentation wraps full turns; untouched exact numeric strings remain
// authoritative for Apply, including f64 yaw values outside this display range.
fn displayed_yaw(value: &str) -> Option<f32> {
    let value = value.parse::<f64>().ok()?;
    value.is_finite().then(|| {
        let wrapped = (value + 180.).rem_euclid(360.) - 180.;
        // Keep both inclusive track endpoints stable. Positive authored half
        // turns belong at +180; negative half turns belong at -180.
        if wrapped == -180. && value > 0. {
            180.
        } else {
            wrapped as f32
        }
    })
}

pub(super) fn slider_change(
    event: On<ValueChange<f32>>,
    sliders: Query<&Control, With<Slider>>,
    windows: Query<&Window>,
    session: Res<Session>,
    mut fields: Query<(&Field, &mut EditableText)>,
) {
    if !windows.single().is_ok_and(|w| w.focused)
        || !event.value.is_finite()
        || session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
        || session.name_edit.is_some()
        || session.file_dialog.is_some()
    {
        return;
    }
    let (Ok(control), Some(draft)) = (sliders.get(event.source), &session.transform_edit) else {
        return;
    };
    let Some(spec) = draft.original.slider(*control, session.snap) else {
        return;
    };
    if !(spec.min..=spec.max).contains(&event.value) {
        return;
    }
    let Some((_, mut field)) = fields.iter_mut().find(|(field, _)| field.0 == spec.field) else {
        return;
    };
    let current = field.value().to_string();
    if field.is_composing() || (spec.valid(&current) && spec.display(&current) == Some(event.value))
    {
        return;
    }
    let original = draft.original.values()[spec.field].clone();
    let value = if spec.display(&original) == Some(event.value) {
        original
    } else {
        f64::from(((event.value / spec.step).round() * spec.step).clamp(spec.min, spec.max))
            .to_string()
    };
    field.clear();
    field.editor_mut().set_text(&value);
}

type FormSliders<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Control,
        &'static SliderValue,
        &'static SliderRange,
        &'static SliderStep,
        Has<InteractionDisabled>,
    ),
    With<Slider>,
>;

pub(super) fn slider_sync(
    mut commands: Commands,
    session: Res<Session>,
    mut rows: Query<(&SliderRow, &mut Node, &mut Visibility)>,
    sliders: FormSliders,
    mut thumbs: Query<(&SliderHandle, &mut Node), Without<SliderRow>>,
    mut labels: Query<(&SliderLabel, &mut Text)>,
) {
    let spec = |control| {
        session
            .transform_edit
            .as_ref()
            .and_then(|draft| draft.original.slider(control, session.snap))
    };
    let value = |spec: SliderSpec| {
        session
            .transform_edit
            .as_ref()
            .and_then(|draft| spec.display(&draft.fields[spec.field]))
            .map(|value| value.clamp(spec.min, spec.max))
    };
    for (row, mut node, mut visibility) in &mut rows {
        let visible = spec(row.0).is_some();
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (entity, control, current, range, step, disabled) in &sliders {
        let actual = spec(*control);
        if disabled == actual.is_some() {
            if actual.is_some() {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
        if let Some(spec) = actual {
            if let Some(value) = value(spec)
                && current.0 != value
            {
                commands.entity(entity).insert(SliderValue(value));
            }
            if range.start() != spec.min || range.end() != spec.max {
                commands
                    .entity(entity)
                    .insert(SliderRange::new(spec.min, spec.max));
            }
            if step.0 != spec.step {
                commands.entity(entity).insert(SliderStep(spec.step));
            }
        }
    }
    for (thumb, mut node) in &mut thumbs {
        if let Some(spec) = spec(thumb.0)
            && let Some(value) = value(spec)
        {
            node.left = percent((value - spec.min) / (spec.max - spec.min) * 100.);
        }
    }
    for (label, mut text) in &mut labels {
        if let Some(spec) = spec(label.0) {
            **text = spec.label.into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spawn_fields_preserve_exact_world_values_and_share_bounds_and_history() {
        for origin in [[0.; 3], [128_000_000_000., 20., -128_000_000_000.]] {
            let zone = Zone {
                origin,
                spawn: [origin[0] + 8.125, origin[1] + 0.125, origin[2] + 6.5],
                ..default()
            };
            let target = Target::Spawn(zone.spawn);
            assert_eq!(target.labels().len(), 3);
            let mut fields = target.values();
            assert!(target.edit(None, &fields).unwrap().is_none());
            assert!(target.edit(None, &fields[..2]).is_err());
            for invalid in ["NaN", "inf", "", "one"] {
                fields[0] = invalid.into();
                assert!(target.edit(None, &fields).is_err());
            }
            fields = target.values();
            let mut editor = Editor::new(zone.clone()).unwrap();
            fields[0] = (origin[0] + 12.375).to_string();
            let edit = target.edit(None, &fields).unwrap().unwrap();
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![edit],
                })
                .unwrap();
            let changed = editor.zone().clone();
            assert_eq!(
                changed.spawn,
                [origin[0] + 12.375, zone.spawn[1], zone.spawn[2]]
            );
            assert_eq!(editor.history(), (1, 0));
            editor
                .execute(Command::Undo {
                    expected_revision: 1,
                })
                .unwrap();
            assert_eq!(editor.zone(), &zone);
            let history = editor.history();
            fields[0] = (origin[0] + 999.).to_string();
            assert!(
                editor
                    .execute(Command::Apply {
                        expected_revision: 2,
                        edits: vec![target.edit(None, &fields).unwrap().unwrap()]
                    })
                    .is_err()
            );
            assert_eq!(editor.zone(), &zone);
            assert_eq!(editor.revision(), 2);
            assert_eq!(editor.history(), history);
            assert!(target.edit(None, &target.values()).unwrap().is_none());
            editor
                .execute(Command::Redo {
                    expected_revision: 2,
                })
                .unwrap();
            assert_eq!(editor.zone(), &changed);
        }
    }
    #[test]
    fn guide_width_and_point_properties_share_validation_and_preserve_other_points() {
        use theia_zone::paths::PathKind;
        for origin in [[0.; 3], [128_000_000_000., 20., -128_000_000_000.]] {
            for kind in [PathKind::Road, PathKind::Stream] {
                let path = GuidePath {
                    id: 9,
                    name: "Walking route".into(),
                    kind,
                    width_metres: 4.,
                    points: vec![
                        [origin[0] + 8., origin[1] + 0.03, origin[2] + 8.],
                        [origin[0] + 24., origin[1] + 0.03, origin[2] + 8.],
                        [origin[0] + 24., origin[1] + 0.03, origin[2] + 24.],
                    ],
                };
                for point in [None, Some(1)] {
                    let target = Target::Path {
                        path: path.clone(),
                        point,
                    };
                    let mut fields = target.values();
                    assert!(target.edit(Some(9), &fields).unwrap().is_none());
                    assert!(target.edit(Some(9), &[]).is_err());
                    let mut zone = Zone {
                        schema: 7,
                        origin,
                        spawn: origin,
                        ..default()
                    };
                    zone.paths.push(path.clone());
                    let mut editor = Editor::new(zone.clone()).unwrap();
                    if point.is_some() {
                        fields[0] = (origin[0] + 24.125).to_string();
                        fields[1] = (origin[1] + 1.25).to_string();
                        fields[2] = (origin[2] + 8.5).to_string();
                    }
                    *fields.last_mut().unwrap() = "2.75".into();
                    editor
                        .execute(Command::Apply {
                            expected_revision: 0,
                            edits: vec![target.edit(Some(9), &fields).unwrap().unwrap()],
                        })
                        .unwrap();
                    let changed = editor.zone().clone();
                    assert_eq!(changed.paths[0].name, path.name);
                    assert_eq!(changed.paths[0].kind, kind);
                    assert_eq!(changed.paths[0].width_metres, 2.75);
                    assert_eq!(changed.paths[0].points[0], path.points[0]);
                    assert_eq!(changed.paths[0].points[2], path.points[2]);
                    assert_eq!(
                        changed.paths[0].points[1],
                        if point.is_some() {
                            [origin[0] + 24.125, origin[1] + 1.25, origin[2] + 8.5]
                        } else {
                            path.points[1]
                        }
                    );
                    assert_eq!(editor.history(), (1, 0));
                    editor
                        .execute(Command::Undo {
                            expected_revision: 1,
                        })
                        .unwrap();
                    assert_eq!(editor.zone(), &zone);
                    let mut invalid = fields.clone();
                    *invalid.last_mut().unwrap() = "0.1".into();
                    assert!(
                        editor
                            .execute(Command::Apply {
                                expected_revision: 2,
                                edits: vec![target.edit(Some(9), &invalid).unwrap().unwrap()]
                            })
                            .is_err()
                    );
                    if point.is_some() {
                        for (value, coordinate) in invalid.iter_mut().zip(path.points[0]) {
                            *value = coordinate.to_string();
                        }
                        *invalid.last_mut().unwrap() = "2.75".into();
                        assert!(
                            editor
                                .execute(Command::Apply {
                                    expected_revision: 2,
                                    edits: vec![target.edit(Some(9), &invalid).unwrap().unwrap()]
                                })
                                .is_err()
                        );
                    }
                    assert_eq!(editor.zone(), &zone);
                    assert_eq!(editor.history(), (0, 1));
                    assert_eq!(editor.revision(), 2);
                    editor
                        .execute(Command::Redo {
                            expected_revision: 2,
                        })
                        .unwrap();
                    assert_eq!(editor.zone(), &changed);
                }
            }
        }
    }
    #[test]
    fn plant_marker_fields_preserve_metadata_and_share_candidate_history() {
        use theia_zone::{markers::MarkerKind, vegetation::PlantKind};
        for origin in [[0.; 3], [128_000_000_000., 20., -128_000_000_000.]] {
            for target in [
                Target::Plant(Plant {
                    id: 7,
                    name: "Tree".into(),
                    kind: PlantKind::Tree,
                    position: origin,
                    yaw_degrees: 0.,
                    scale: 1.,
                }),
                Target::Plant(Plant {
                    id: 7,
                    name: "Shrub".into(),
                    kind: PlantKind::Shrub,
                    position: origin,
                    yaw_degrees: 0.,
                    scale: 1.,
                }),
                Target::Marker(Marker {
                    id: 7,
                    name: "NPC".into(),
                    kind: MarkerKind::Npc,
                    position: origin,
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: Some("npc.guide".into()),
                }),
                Target::Marker(Marker {
                    id: 7,
                    name: "Quest".into(),
                    kind: MarkerKind::Quest,
                    position: origin,
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: Some("quest.note".into()),
                }),
            ] {
                let mut zone = Zone {
                    schema: 7,
                    origin,
                    spawn: origin,
                    ..default()
                };
                match &target {
                    Target::Plant(p) => zone.plants.push(p.clone()),
                    Target::Marker(m) => zone.markers.push(m.clone()),
                    _ => unreachable!(),
                }
                let mut editor = Editor::new(zone.clone()).unwrap();
                let mut fields = target.values();
                assert!(target.edit(Some(7), &fields).unwrap().is_none());
                assert!(target.edit(Some(7), &fields[..4]).is_err());
                for index in 0..5 {
                    let mut invalid = fields.clone();
                    invalid[index] = "NaN".into();
                    assert!(
                        target
                            .edit(Some(7), &invalid)
                            .unwrap_err()
                            .contains(target.labels()[index])
                    );
                }
                fields[0] = (origin[0] + 8.125).to_string();
                fields[1] = (origin[1] + 2.25).to_string();
                fields[2] = (origin[2] - 8.5).to_string();
                fields[3] = "37.5".into();
                fields[4] = "1.25".into();
                editor
                    .execute(Command::Apply {
                        expected_revision: 0,
                        edits: vec![target.edit(Some(7), &fields).unwrap().unwrap()],
                    })
                    .unwrap();
                let changed = editor.zone().clone();
                assert_eq!(editor.history(), (1, 0));
                match &target {
                    Target::Plant(p) => {
                        let mut expected = p.clone();
                        expected.position = [origin[0] + 8.125, origin[1] + 2.25, origin[2] - 8.5];
                        expected.yaw_degrees = 37.5;
                        expected.scale = 1.25;
                        assert_eq!(changed.plants, [expected]);
                    }
                    Target::Marker(m) => {
                        let mut expected = m.clone();
                        expected.position = [origin[0] + 8.125, origin[1] + 2.25, origin[2] - 8.5];
                        expected.yaw_degrees = 37.5;
                        expected.radius_metres = 1.25;
                        assert_eq!(changed.markers, [expected]);
                    }
                    _ => unreachable!(),
                }
                editor
                    .execute(Command::Undo {
                        expected_revision: 1,
                    })
                    .unwrap();
                assert_eq!(editor.zone(), &zone);
                fields[4] = "65".into();
                assert!(
                    editor
                        .execute(Command::Apply {
                            expected_revision: 2,
                            edits: vec![target.edit(Some(7), &fields).unwrap().unwrap()]
                        })
                        .is_err()
                );
                assert_eq!(editor.history(), (0, 1));
                assert_eq!(editor.revision(), 2);
                assert_eq!(editor.zone(), &zone);
                editor
                    .execute(Command::Redo {
                        expected_revision: 2,
                    })
                    .unwrap();
                assert_eq!(editor.zone(), &changed);
            }
        }
    }
    #[test]
    fn numeric_fields_round_trip_far_f64_positions_without_display_rounding() {
        for sign in [-1., 1.] {
            let pose = Pose {
                position: [
                    sign * 128_000_000_000. + 0.012345,
                    20.123456789,
                    -sign * 128_000_000_000. - 0.987654,
                ],
                yaw_degrees: -37.123456789,
                size: [1.123456789, 2.123456789, 3.123456789],
            };
            assert_eq!(parse(&values(&pose)).unwrap(), pose);
            let mut fields = values(&pose);
            fields[4] = " 1.25e1 ".into();
            let changed = parse(&fields).unwrap();
            assert_eq!(changed.position, pose.position);
            assert_eq!(changed.yaw_degrees, pose.yaw_degrees);
            assert_eq!(changed.size, [12.5, pose.size[1], pose.size[2]]);
        }
    }
    #[test]
    fn malformed_and_nonfinite_fields_are_refused_before_commands() {
        let pose = Pose {
            position: [1., 2., 3.],
            yaw_degrees: 0.,
            size: [1.; 3],
        };
        for index in 0..7 {
            for invalid in ["", " ", "1,5", "2m", "NaN", "inf", "-inf", "1e999"] {
                let mut fields = values(&pose);
                fields[index] = invalid.into();
                assert!(parse(&fields).unwrap_err().contains(LABELS[index]));
            }
        }
    }
    #[test]
    fn parsed_fields_share_full_candidate_validation_and_one_entry_history_for_all_shapes() {
        for primitive in [Primitive::Box, Primitive::Sphere, Primitive::Cylinder] {
            for origin in [[0.; 3], [128_000_000_000., 20., -128_000_000_000.]] {
                let mut zone = Zone {
                    origin,
                    spawn: [origin[0], origin[1] + 0.1, origin[2] + 6.],
                    ..default()
                };
                let pose = Pose {
                    position: [origin[0] + 8., origin[1] + 1., origin[2] - 8.],
                    yaw_degrees: 0.,
                    size: [2.; 3],
                };
                zone.objects.push(Object {
                    id: 1,
                    name: "Numeric fixture".into(),
                    primitive,
                    pose: pose.clone(),
                });
                let mut editor = Editor::new(zone.clone()).unwrap();
                let mut fields = values(&pose);
                fields[0] = (origin[0] + 8.125).to_string();
                fields[1] = (origin[1] + 1.5).to_string();
                fields[2] = (origin[2] - 8.5).to_string();
                fields[3] = "37.5".into();
                for value in &mut fields[4..] {
                    *value = "3".into();
                }
                let changed = parse(&fields).unwrap();
                editor
                    .execute(Command::Apply {
                        expected_revision: 0,
                        edits: vec![Edit::Transform {
                            id: 1,
                            pose: changed.clone(),
                        }],
                    })
                    .unwrap();
                assert_eq!(editor.history(), (1, 0));
                assert_eq!(editor.zone().objects[0].pose, changed);
                editor
                    .execute(Command::Undo {
                        expected_revision: 1,
                    })
                    .unwrap();
                assert_eq!(editor.zone(), &zone);
                for (index, invalid) in [
                    (0, (origin[0] + 513.).to_string()),
                    (4, "-1".into()),
                    (3, "360001".into()),
                ] {
                    let mut invalid_fields = fields.clone();
                    invalid_fields[index] = invalid;
                    assert!(
                        editor
                            .execute(Command::Apply {
                                expected_revision: 2,
                                edits: vec![Edit::Transform {
                                    id: 1,
                                    pose: parse(&invalid_fields).unwrap()
                                }]
                            })
                            .is_err()
                    );
                    assert_eq!(editor.revision(), 2);
                    assert_eq!(editor.history(), (0, 1));
                    assert_eq!(editor.zone(), &zone);
                }
                if primitive != Primitive::Box {
                    let mut invalid_fields = fields.clone();
                    invalid_fields[4] = "4".into();
                    assert!(
                        editor
                            .execute(Command::Apply {
                                expected_revision: 2,
                                edits: vec![Edit::Transform {
                                    id: 1,
                                    pose: parse(&invalid_fields).unwrap()
                                }]
                            })
                            .is_err()
                    );
                    assert_eq!(editor.history(), (0, 1));
                }
                editor
                    .execute(Command::Redo {
                        expected_revision: 2,
                    })
                    .unwrap();
                assert_eq!(editor.zone().objects[0].pose, changed);
                assert_eq!(editor.zone().objects[0].name, "Numeric fixture");
            }
        }
    }
}

#[cfg(test)]
#[path = "properties_yaw_tests.rs"]
mod yaw_tests;

#[cfg(test)]
#[path = "properties_value_tests.rs"]
mod value_tests;

#[cfg(test)]
#[path = "properties_keyboard_tests.rs"]
mod keyboard_tests;

#[cfg(test)]
mod test_player_attributes {
    use super::*;
    #[test]
    fn inline_test_player_values_keep_configuration_and_validate_integer_network_settings() {
        let original = theia_zone::test_players::TestPlayer {
            id: 1,
            name: "Delayed player".into(),
            client_id: 2,
            enabled: false,
            position: [128_000_000_000.125, 0.1, -128_000_000_000.25],
            controller: Some("test.walk-turn".into()),
            network: theia_zone::test_players::NetworkConditions {
                outages: vec![[1000, 100]],
                ..default()
            },
        };
        let target = Target::TestPlayer(original.clone());
        let mut values = target.values();
        assert_eq!(values.len(), 7);
        assert!(!target.has_yaw());
        values[0] = "128000000000.375".into();
        values[3] = "80".into();
        values[4] = "20".into();
        values[5] = "1.25".into();
        values[6] = "4294967295".into();
        let Some(Edit::PutTestPlayer { player }) = target.edit(Some(1), &values).unwrap() else {
            panic!("Expected test-player command");
        };
        assert_eq!(player.position[0], 128_000_000_000.375);
        assert_eq!(player.position[2], original.position[2]);
        assert_eq!(player.network.added_base_rtt_ms(), 160);
        assert_eq!(player.network.outages, original.network.outages);
        assert_eq!(player.controller, original.controller);
        assert!(!player.enabled);
        for (field, value) in [
            (3, "80.5"),
            (4, "-1"),
            (3, "5001"),
            (5, "101"),
            (6, "4294967296"),
            (0, "NaN"),
        ] {
            let mut bad = values.clone();
            bad[field] = value.into();
            assert!(target.edit(Some(1), &bad).is_err());
        }
    }
}
