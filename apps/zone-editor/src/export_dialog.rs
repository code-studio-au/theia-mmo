//! Export the current zone snapshot to a new, manifest-bound world directory.
use super::*;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::tasks::{IoTaskPool, Task, futures::check_ready};
use bevy::text::{EditableText, TextEdit};
use bevy::ui_widgets::{SelectAllOnFocus, TextInput};
use bevy_sprinkles::asset::ParticlesAsset;
use bevy_sprinkles_editor::{
    project::ProjectOperation,
    state::{DirtyState, EditorState},
};

pub(super) struct Draft {
    mode: Mode,
    revision: u64,
    task: Option<Task<Result<serde_json::Value, String>>>,
    finished: bool,
    target: Option<PathBuf>,
    player_assets: Option<PathBuf>,
    account_catalogues: Option<(PathBuf, PathBuf)>,
    account_gameplay: bool,
    message: String,
    action: Option<Control>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    World,
    NetworkTest,
    Population,
    PopulationTest,
    AccountPopulationTest,
    ParticleChunks,
    ReplaceParticleChunks,
}
impl Mode {
    fn world(self) -> bool {
        matches!(
            self,
            Self::World
                | Self::NetworkTest
                | Self::Population
                | Self::PopulationTest
                | Self::AccountPopulationTest
        )
    }
    fn population(self) -> bool {
        matches!(
            self,
            Self::Population | Self::PopulationTest | Self::AccountPopulationTest
        )
    }
    fn network(self) -> bool {
        matches!(
            self,
            Self::NetworkTest | Self::PopulationTest | Self::AccountPopulationTest
        )
    }
    fn fields(self) -> &'static [usize] {
        match self {
            Self::World => &[0, 1, 2, 3],
            Self::NetworkTest => &[0, 3, 4],
            Self::Population => &[0, 1, 2, 3, 4, 5, 7, 10, 11],
            Self::PopulationTest => &[0, 1, 2, 3, 4, 5, 6, 7, 11],
            Self::AccountPopulationTest => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            _ => &[0, 1],
        }
    }
    fn visible_fields(self, template: bool) -> &'static [usize] {
        if self == Self::AccountPopulationTest && template {
            &[0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12]
        } else {
            self.fields()
        }
    }
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) struct Field(usize);
#[derive(Component)]
pub(super) struct Message;
#[derive(Component)]
pub(super) struct FieldRow(usize);
#[derive(Component)]
pub(super) struct FieldCaption(usize);
#[derive(Component)]
pub(super) struct DialogTitle;
#[derive(Component)]
pub(super) struct SubmitLabel;
#[derive(Component)]
pub(super) struct DialogDescription;
type DialogTexts<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Text,
        Option<&'static FieldCaption>,
        Has<DialogTitle>,
        Has<SubmitLabel>,
        Has<DialogDescription>,
        Has<Message>,
    ),
>;
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    Export,
    Close,
}

pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            Root,
            GlobalZIndex(30),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(theme::dialog::BACKDROP),
        ))
        .with_children(|root| {
            root.spawn(theme::dialog::panel(520.))
                .with_children(|panel| {
                    panel.spawn(theme::dialog::header()).with_child((
                        DialogTitle,
                        theme::label("Export world", 18., theme::DISPLAY),
                    ));
                    scrolling::pane(panel, 24., scrolling::Pane::Dialog, |body| {
                        body.spawn((
                            DialogDescription,
                            theme::label(
                                "Exports the current zone snapshot. Save particle edits first.",
                                12.,
                                theme::MUTED,
                            ),
                        ));
                        for (index, caption) in [
                            "New output folder",
                            "Content ID",
                            "Revision",
                            "Gameplay template (optional)",
                            "Player asset directory",
                            "Minimum spacing (metres)",
                            "Player asset directory",
                            "Creature appearance GLB (optional)",
                            "Item catalogue",
                            "Reward / corpse-loot catalogue",
                            "Gameplay template (optional)",
                            "Access road guide ID (optional)",
                            "AI test road ID (optional; starts paused, F7)",
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            body.spawn((
                                FieldRow(index),
                                Node {
                                    width: percent(100),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                            ))
                            .with_children(|row| {
                                row.spawn((
                                    FieldCaption(index),
                                    theme::label(caption, 12., theme::BODY),
                                ));
                                row.spawn((
                                    Field(index),
                                    bevy::input_focus::tab_navigation::TabIndex(0),
                                    TextInput,
                                    SelectAllOnFocus,
                                    EditableText {
                                        max_characters: Some(if index == 2 { 96 } else { 512 }),
                                        allow_newlines: false,
                                        ..default()
                                    },
                                    Node {
                                        width: percent(100),
                                        ..theme::input_node()
                                    },
                                    BorderColor::all(theme::BORDER),
                                    BackgroundColor(Color::NONE),
                                    theme::input_font(),
                                    TextColor(theme::BODY),
                                    TextLayout::no_wrap(),
                                    theme::text_cursor(),
                                ));
                            });
                        }
                        body.spawn((Message, theme::label("", 12., theme::MUTED)));
                    });
                    panel.spawn(theme::dialog::footer()).with_children(|row| {
                        for (control, caption) in
                            [(Control::Close, "Close"), (Control::Export, "Export world")]
                        {
                            let mut button = row.spawn((
                                control,
                                Button,
                                Hovered::default(),
                                theme::button_node(),
                                BackgroundColor(theme::Variant::Default.background(false)),
                            ));
                            button.with_children(|button| {
                                let mut label =
                                    button.spawn(theme::label(caption, 12., theme::BODY));
                                if matches!(control, Control::Export) {
                                    label.insert(SubmitLabel);
                                }
                            });
                        }
                    });
                });
        });
}

pub(super) fn activate(
    event: On<PointerPress>,
    controls: Query<&Control>,
    mut session: ResMut<Session>,
) {
    if event.button == PointerButton::Primary
        && let Ok(control) = controls.get(event.entity)
        && let Some(draft) = &mut session.export_dialog
    {
        draft.action = Some(*control);
    }
}

pub(super) fn open(
    mut requests: ResMut<workspace::Requests>,
    mut session: ResMut<Session>,
    windows: Query<&Window>,
    mut fields: Query<(Entity, &Field, &mut EditableText)>,
    mut focus: ResMut<InputFocus>,
    manager: Option<Res<network_playtest::Manager>>,
) {
    let network = std::mem::take(&mut requests.network_playtest);
    let account_test = std::mem::take(&mut requests.account_population_playtest);
    let population_test = std::mem::take(&mut requests.population_playtest);
    let world = std::mem::take(&mut requests.export_world);
    let population = std::mem::take(&mut requests.export_population);
    let chunks = std::mem::take(&mut requests.export_particle_chunks);
    let replace = std::mem::take(&mut requests.replace_particle_chunks);
    if !(world || chunks || replace || network || population || population_test || account_test)
        || session.has_modal()
        || !windows.single().is_ok_and(|window| window.focused)
    {
        return;
    }
    if session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
    {
        session.status = "Finish the preview or leave playtest before exporting".into();
        return;
    }
    let slug = slug(&session.editor.zone().name);
    if (network || population_test)
        && (session.network_active || !session.editor.zone().test_players.iter().any(|p| p.enabled))
    {
        session.status = "Enable a test player and stop any existing network session first".into();
        return;
    }
    if account_test && session.network_active {
        session.status = "Stop the current playtest first".into();
        return;
    }
    let mode = if account_test {
        Mode::AccountPopulationTest
    } else if population_test {
        Mode::PopulationTest
    } else if network {
        Mode::NetworkTest
    } else if population {
        Mode::Population
    } else if replace {
        Mode::ReplaceParticleChunks
    } else if chunks {
        Mode::ParticleChunks
    } else {
        Mode::World
    };
    let values = if mode.population() {
        vec![
            if mode.network() {
                format!(
                    "build/{slug}-population-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_nanos())
                )
            } else {
                format!("build/{slug}-population-v1")
            },
            format!("theia.zone.{slug}"),
            "1".into(),
            session
                .creature_context
                .as_ref()
                .map_or_else(String::new, |c| c.path.display().to_string()),
            population_export::default_baker()
                .map_or_else(|_| String::new(), |p| p.display().to_string()),
            "3".into(),
            manager
                .as_ref()
                .and_then(|m| m.player_assets.as_ref())
                .map_or_else(String::new, |p| p.display().to_string()),
            manager
                .as_ref()
                .and_then(|m| m.creature_asset.as_ref())
                .map_or_else(String::new, |p| p.display().to_string()),
            "content/gameplay/development-equipment-items.json".into(),
            "content/gameplay/development-rewards.json".into(),
            if mode == Mode::PopulationTest {
                String::new()
            } else {
                session
                    .quest_context
                    .as_ref()
                    .map(|c| &c.path)
                    .or_else(|| manager.as_ref().and_then(|m| m.template.as_ref()))
                    .map_or_else(String::new, |p| p.display().to_string())
            },
            manager
                .as_ref()
                .and_then(|m| m.access_path)
                .map_or_else(String::new, |id| id.to_string()),
            String::new(),
        ]
    } else if mode.world() {
        vec![
            if network {
                format!(
                    "build/{slug}-network-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_nanos())
                )
            } else {
                format!("build/{slug}-v1")
            },
            format!("theia.zone.{slug}"),
            "1".to_string(),
            session
                .quest_context
                .as_ref()
                .map(|c| &c.path)
                .or_else(|| manager.as_ref().and_then(|m| m.template.as_ref()))
                .map_or_else(String::new, |p| p.display().to_string()),
            manager
                .as_ref()
                .and_then(|m| m.player_assets.as_ref())
                .map_or_else(String::new, |p| p.display().to_string()),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ]
    } else {
        vec![
            format!("build/{slug}-particles"),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ]
    };
    for (entity, field, mut text) in &mut fields {
        text.clear();
        text.editor_mut()
            .set_text(values.get(field.0).map_or("", String::as_str));
        if field.0 == 0 {
            text.queue_edit(TextEdit::SelectAll);
            focus.set(entity, FocusCause::Navigated);
        }
    }
    session.export_dialog = Some(Draft {
        mode,
        revision: session.editor.revision(),
        task: None,
        finished: false,
        target: None,
        player_assets: None,
        account_catalogues: None,
        account_gameplay: false,
        message: "Choose a new folder; existing exports are never overwritten.".into(),
        action: None,
    });
}
fn check_account_catalogues(items: &str, rewards: &str) -> Result<(PathBuf, PathBuf), String> {
    let read = |value: &str, limit| {
        if value.trim().is_empty() || value.contains(['\0', '\n', '\r']) {
            return Err("Choose existing item and reward catalogues".to_string());
        }
        let path = std::fs::canonicalize(value.trim()).map_err(|e| e.to_string())?;
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .map_err(|e| e.to_string())?
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > limit {
            return Err("Catalogue exceeds its size limit".into());
        }
        Ok((path, bytes))
    };
    let (items_path, items) = read(items, theia_core::inventory::MAX_CATALOG_BYTES as u64)?;
    let (rewards_path, rewards) = read(rewards, theia_core::rewards::MAX_CATALOG_BYTES as u64)?;
    let inventory = theia_network::renet_runtime::inventory::Profile::from_json(&items)
        .map_err(|e| format!("Item catalogue: {e}"))?;
    theia_network::reward_wire::Profile::from_json_with_creature_loot(&rewards, &inventory)
        .map_err(|e| format!("Reward catalogue: {e}"))?;
    Ok((items_path, rewards_path))
}
fn slug(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            result.push(c);
        } else if !result.is_empty() && !result.ends_with('-') {
            result.push('-');
        }
        if result.len() >= 48 {
            break;
        }
    }
    let result = result.trim_matches('-');
    if result.is_empty() {
        "untitled".into()
    } else {
        result.into()
    }
}
struct WorldInput {
    path: PathBuf,
    id: String,
    revision: u32,
    template: Option<PathBuf>,
}
fn values(fields: &Query<(Entity, &Field, &mut EditableText)>) -> Result<WorldInput, String> {
    let mut values: [String; 4] = std::array::from_fn(|_| String::new());
    for (_, field, text) in fields.iter() {
        if field.0 < values.len() {
            values[field.0] = text.value().to_string().trim().to_owned();
        }
    }
    if values[0].is_empty() || values[0].contains(['\0', '\n', '\r']) {
        return Err("Enter a valid new output folder".into());
    }
    if theia_core::content::ContentId::new(&values[1]).is_none() {
        return Err("Enter a valid lowercase content ID".into());
    }
    let revision = values[2]
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or("Revision must be a positive whole number")?;
    if values[3].contains(['\0', '\n', '\r']) {
        return Err("Enter a valid profile or gameplay manifest path".into());
    }
    Ok(WorldInput {
        path: PathBuf::from(&values[0]),
        id: values[1].clone(),
        revision,
        template: (!values[3].is_empty()).then(|| PathBuf::from(&values[3])),
    })
}

struct PopulationInput {
    baker: PathBuf,
    spacing: f64,
    appearance: Option<PathBuf>,
    gameplay: Option<PathBuf>,
    access_path: Option<u64>,
    test_path: Option<u64>,
}
fn population_values(
    fields: &Query<(Entity, &Field, &mut EditableText)>,
    allow_gameplay: bool,
    allow_test_travel: bool,
) -> Result<PopulationInput, String> {
    let value = |index| {
        fields
            .iter()
            .find(|(_, f, _)| f.0 == index)
            .map(|(_, _, t)| t.value().to_string().trim().to_owned())
            .unwrap_or_default()
    };
    let creature = value(3);
    if creature.is_empty() || creature.contains(['\0', '\n', '\r']) {
        return Err("Choose a creature profile".into());
    }
    let baker = value(4);
    if baker.is_empty() || baker.contains(['\0', '\n', '\r']) {
        return Err("Choose a navigation baker executable".into());
    }
    let spacing = value(5)
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && (0.25..=32.).contains(n))
        .ok_or("Minimum spacing must be between 0.25 and 32 metres")?;
    let appearance = value(7);
    if appearance.contains(['\0', '\n', '\r']) {
        return Err("Choose a valid creature appearance path".into());
    }
    let appearance = (!appearance.is_empty()).then(|| PathBuf::from(appearance));
    let gameplay = if allow_gameplay {
        value(10)
    } else {
        String::new()
    };
    if gameplay.contains(['\0', '\n', '\r']) {
        return Err("Choose a valid gameplay template path".into());
    }
    let access_path = value(11);
    let access_path = if access_path.is_empty() {
        None
    } else {
        Some(
            access_path
                .parse::<u64>()
                .ok()
                .filter(|id| *id > 0)
                .ok_or("Access road guide ID must be a positive integer")?,
        )
    };
    Ok(PopulationInput {
        baker: PathBuf::from(baker),
        spacing,
        appearance,
        gameplay: (!gameplay.is_empty()).then(|| PathBuf::from(gameplay)),
        access_path,
        test_path: if allow_test_travel && !value(12).is_empty() {
            Some(
                value(12).parse::<u64>().ok().filter(|id| *id > 0).ok_or(
                    "AI test road ID must be a positive integer, or blank for manual play",
                )?,
            )
        } else {
            None
        },
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut roots: Query<&mut Node, With<Root>>,
    fields: Query<(Entity, &Field, &mut EditableText)>,
    mut texts: DialogTexts,
    mut rows: Query<(&FieldRow, &mut Node), Without<Root>>,
    mut focus: ResMut<InputFocus>,
    dirty: Res<DirtyState>,
    editor: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    operation: ProjectOperation,
    scenery: Option<Res<scenery_library::Library>>,
    mut manager: Option<ResMut<network_playtest::Manager>>,
) {
    let Some(mut draft) = session.export_dialog.take() else {
        for mut root in &mut roots {
            root.display = Display::None;
        }
        return;
    };
    for mut root in &mut roots {
        root.display = Display::Flex;
    }
    let has_template = matches!(draft.mode, Mode::Population | Mode::AccountPopulationTest)
        && fields
            .iter()
            .any(|(_, field, text)| field.0 == 10 && !text.value().to_string().trim().is_empty());
    let visible = draft.mode.visible_fields(has_template);
    for (row, mut node) in &mut rows {
        node.display = if !visible.contains(&row.0) {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Some(task) = &mut draft.task
        && let Some(result) = check_ready(task)
    {
        draft.task = None;
        match result {
            Ok(_) => {
                draft.finished = true;
                if draft.mode.network() {
                    let result = manager
                        .as_deref_mut()
                        .ok_or_else(|| "Network launcher is unavailable".to_string())
                        .and_then(|m| {
                            if draft.account_gameplay {
                                return m.start_account_gameplay(
                                    draft.target.as_deref().expect("Export target"),
                                    draft
                                        .player_assets
                                        .as_deref()
                                        .expect("Checked player assets"),
                                );
                            }
                            if let Some((items, rewards)) = &draft.account_catalogues {
                                return m.start_account_catalogues(
                                    draft.target.as_deref().expect("Export target"),
                                    draft
                                        .player_assets
                                        .as_deref()
                                        .expect("Checked player assets"),
                                    items,
                                    rewards,
                                );
                            }
                            let start = if draft.mode == Mode::AccountPopulationTest {
                                network_playtest::Manager::start_account_population
                            } else if draft.mode == Mode::PopulationTest {
                                network_playtest::Manager::start_population
                            } else {
                                network_playtest::Manager::start
                            };
                            start(
                                m,
                                draft.target.as_deref().expect("Export target"),
                                draft
                                    .player_assets
                                    .as_deref()
                                    .expect("Checked player assets"),
                            )
                        });
                    match result {
                        Ok(()) => {
                            session.network_active = true;
                            session.status="Network playtest starting; Zone -> Stop network playtest ends the server and clients".into();
                            focus.clear();
                            for mut root in &mut roots {
                                root.display = Display::None;
                            }
                            return;
                        }
                        Err(error) => {
                            draft.message =
                                format!("Network snapshot exported, but launch failed: {error}");
                            session.status = draft.message.clone();
                        }
                    }
                } else {
                    let target = draft
                        .target
                        .as_ref()
                        .map_or_else(String::new, |path| path.display().to_string());
                    draft.message = format!("Exported {target}. Close to return to the editor.");
                    session.status = format!("World exported to {target}");
                }
            }
            Err(error) => draft.message = format!("Export failed: {error}"),
        }
    }
    let focused = windows.single().is_ok_and(|window| window.focused);
    let action = draft.action.take();
    if focused
        && draft.task.is_none()
        && (matches!(action, Some(Control::Close)) || keys.just_pressed(KeyCode::Escape))
    {
        focus.clear();
        for mut root in &mut roots {
            root.display = Display::None;
        }
        return;
    }
    if focused && draft.task.is_none() && keys.just_pressed(KeyCode::Tab) {
        let current = focus.get();
        let index = fields
            .iter()
            .find(|(entity, _, _)| Some(*entity) == current)
            .map_or(0, |(_, field, _)| {
                let current = visible
                    .iter()
                    .position(|index| *index == field.0)
                    .unwrap_or(0);
                visible[if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                    (current + visible.len() - 1) % visible.len()
                } else {
                    (current + 1) % visible.len()
                }]
            });
        if let Some((entity, _, _)) = fields.iter().find(|(_, field, _)| field.0 == index) {
            focus.set(entity, FocusCause::Navigated);
        }
    }
    let enter = focused
        && (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter))
        && !fields.iter().any(|(_, _, text)| text.is_composing());
    if focused
        && !draft.finished
        && draft.task.is_none()
        && (enter || matches!(action, Some(Control::Export)))
    {
        if session.editor.revision() != draft.revision {
            draft.message = "Zone changed; close and reopen export.".into();
        } else if operation.busy()
            || (dirty.has_unsaved_changes && !editor.is_pristine_example(&assets))
        {
            draft.message = "Save particle edits before exporting.".into();
        } else {
            let input = if draft.mode.world() {
                values(&fields).map(|input| {
                    (
                        input.path,
                        Some((input.id, input.revision, input.template)),
                        None,
                    )
                })
            } else {
                let mut input = [String::new(), String::new()];
                for (_, field, text) in fields.iter() {
                    if field.0 < 2 {
                        input[field.0] = text.value().to_string().trim().to_owned();
                    }
                }
                if input
                    .iter()
                    .any(|value| value.is_empty() || value.contains(['\0', '\n', '\r']))
                {
                    Err("Enter a new output folder and an existing chunk world folder".into())
                } else {
                    Ok((
                        PathBuf::from(&input[0]),
                        None,
                        Some(PathBuf::from(&input[1])),
                    ))
                }
            };
            let population_settings = if draft.mode.population() {
                population_values(
                    &fields,
                    draft.mode != Mode::PopulationTest,
                    draft.mode == Mode::AccountPopulationTest,
                )
                .and_then(|settings| {
                    if let Some(id) = settings.test_path {
                        crate::account_test_route::road(session.editor.zone(), id)?;
                    }
                    Ok(Some(settings))
                })
            } else {
                Ok(None)
            };
            let input = input.and_then(|input| {
                if draft.mode.network() {
                    let manager = manager
                        .as_deref_mut()
                        .ok_or("Network launcher is unavailable")?;
                    if manager.running() {
                        return Err("Stop the current network playtest first".into());
                    }
                    let value = fields
                        .iter()
                        .find(|(_, f, _)| f.0 == if draft.mode.population() { 6 } else { 4 })
                        .map(|(_, _, t)| t.value().to_string())
                        .unwrap_or_default();
                    if value.trim().is_empty() || value.contains(['\0', '\n', '\r']) {
                        return Err("Choose a player asset directory".into());
                    }
                    let path = PathBuf::from(value.trim());
                    if !path.join("character.glb").is_file() {
                        return Err("Player asset directory must contain character.glb".into());
                    }
                    if draft.mode == Mode::AccountPopulationTest {
                        let path = |index| {
                            fields
                                .iter()
                                .find(|(_, f, _)| f.0 == index)
                                .map(|(_, _, t)| t.value().to_string())
                                .unwrap_or_default()
                        };
                        draft.account_gameplay = has_template;
                        draft.account_catalogues = if has_template {
                            None
                        } else {
                            Some(check_account_catalogues(&path(8), &path(9))?)
                        };
                    }
                    draft.player_assets = Some(path);
                    if draft.mode == Mode::NetworkTest {
                        manager.template = input
                            .1
                            .as_ref()
                            .and_then(|(_, _, template)| template.clone());
                    }
                }
                Ok(input)
            });
            match input.and_then(|input| population_settings.map(|settings| (input, settings))) {
                Ok(((path, world, chunks), population)) => {
                    let zone = session.editor.zone().clone();
                    let replace = draft.mode == Mode::ReplaceParticleChunks;
                    let network = draft.mode.network();
                    let account = draft.mode == Mode::AccountPopulationTest;
                    draft.message = format!("Exporting {}…", path.display());
                    draft.target = Some(path.clone());
                    let scenery = scenery.as_deref().cloned().unwrap_or_default();
                    draft.task = Some(IoTaskPool::get().spawn(async move {
                        if let Some((id, revision, template)) = world {
                            if let Some(population) = population {
                                let publish = if account {
                                    population_export::publish_account
                                } else if network {
                                    population_export::publish_network
                                } else {
                                    population_export::publish
                                };
                                publish(
                                    &zone,
                                    &path,
                                    &id,
                                    revision,
                                    population_export::Settings {
                                        creature: template
                                            .as_deref()
                                            .ok_or("Choose a creature profile")?,
                                        baker: &population.baker,
                                        spacing: population.spacing,
                                        appearance: population.appearance.as_deref(),
                                        gameplay: population.gameplay.as_deref(),
                                        access_path: population.access_path,
                                        test_path: population.test_path,
                                    },
                                    &scenery,
                                )
                            } else if network {
                                network_test_export::publish(
                                    &zone,
                                    &path,
                                    &id,
                                    revision,
                                    template.as_deref(),
                                    &scenery,
                                )
                            } else if !zone.scenery.is_empty() {
                                export::publish_with_assets(
                                    &zone,
                                    &path,
                                    &id,
                                    revision,
                                    template.as_deref(),
                                    &scenery,
                                )
                            } else if template.is_some() {
                                export::publish_with_gameplay(
                                    &zone,
                                    &path,
                                    &id,
                                    revision,
                                    template.as_deref(),
                                )
                            } else {
                                export::publish(&zone, &path, &id, revision)
                            }
                        } else {
                            export::publish_particles_to_chunks(
                                &zone,
                                &chunks.expect("chunk export folder"),
                                &path,
                                replace,
                            )
                            .map(|()| serde_json::Value::Null)
                        }
                    }));
                }
                Err(error) => draft.message = error,
            }
        }
    }
    for (mut text, caption, title, submit, description, message) in &mut texts {
        if message {
            **text = draft.message.clone();
        } else if title {
            **text = match draft.mode {
                Mode::NetworkTest => "Start network playtest",
                Mode::World => "Export world",
                Mode::Population => "Export population",
                Mode::PopulationTest => "Start population playtest",
                Mode::AccountPopulationTest => "Start account population playtest",
                Mode::ParticleChunks => "Export particles to chunks",
                Mode::ReplaceParticleChunks => "Replace all world particles",
            }
            .into();
        } else if submit {
            **text = match draft.mode {
                Mode::NetworkTest => "Start network playtest",
                Mode::World => "Export world",
                Mode::Population => "Export population",
                Mode::PopulationTest => "Start population playtest",
                Mode::AccountPopulationTest => "Start account population playtest",
                Mode::ParticleChunks => "Export particles",
                Mode::ReplaceParticleChunks => "Replace all particles",
            }
            .into();
        } else if description {
            **text = match draft.mode {
                Mode::NetworkTest => "Starts one server and a client per enabled Test Player from this zone snapshot. Stop from the Zone menu. Zone files change only with Save.",
                Mode::AccountPopulationTest => "Sign in and choose a persistent character. Select a gameplay template for authored quests. Leave AI test road blank for manual play; an optional saved Road starts paused and uses F7 to travel out and back. Requires configured account services.",
                Mode::PopulationTest => "Starts a checked population with one client per enabled Test Player. Stop through the Zone menu. Choose the matching creature profile and player assets.",
                Mode::Population => "Exports one enabled spawn group, its static world and checked navigation. Select a gameplay template to include authored quests and the guide. Does not start a server.",
                Mode::World => "Exports the current zone. Use a matching gameplay template for quest areas and NPC references. Save particle edits first.",
                Mode::ParticleChunks => "Copies a particle-free chunk world with these placements. Save particle edits first.",
                Mode::ReplaceParticleChunks => "Replaces ALL particle placements in a new copy of the chunk world with this zone's placements. An empty zone removes all world particles. Save particle edits first."
            }.into();
        } else if caption.is_some_and(|caption| caption.0 == 3) {
            **text = if draft.mode.population() {
                "Creature profile"
            } else {
                "Gameplay template (optional)"
            }
            .into();
        } else if caption.is_some_and(|caption| caption.0 == 4) {
            **text = if draft.mode.population() {
                "Navigation baker executable"
            } else {
                "Player asset directory"
            }
            .into();
        } else if caption.is_some_and(|caption| caption.0 == 1) {
            **text = if draft.mode.world() {
                "Content ID"
            } else {
                "Existing chunk world folder"
            }
            .into();
        }
    }
    session.export_dialog = Some(draft);
}

/// Opt-in, read-only native qualification geometry.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Report<'w, 's> {
    fields: Query<
        'w,
        's,
        (
            Entity,
            &'static Field,
            &'static EditableText,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
        ),
    >,
    buttons: Query<
        'w,
        's,
        (
            &'static Control,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
        ),
    >,
}
impl Report<'_, '_> {
    pub(super) fn snapshot(
        &self,
        session: &Session,
        scale: f32,
        focus: &InputFocus,
    ) -> serde_json::Value {
        let Some(draft) = &session.export_dialog else {
            return serde_json::Value::Null;
        };
        let fields: Vec<_> = self.fields.iter().map(|(entity, field, text, node, transform)| {
            let factor = node.inverse_scale_factor() * scale;
            serde_json::json!({"index":field.0,"text":text.value().to_string(),"focused":focus.get()==Some(entity),
                "centre":(transform.affine().translation * factor).to_array(),"size":(node.size()*factor).to_array()})
        }).collect();
        let buttons: Vec<_> = self.buttons.iter().map(|(control, node, transform)| {
            let factor = node.inverse_scale_factor() * scale;
            serde_json::json!({"action":format!("{control:?}"),
                "centre":(transform.affine().translation * factor).to_array(),"size":(node.size()*factor).to_array()})
        }).collect();
        serde_json::json!({"fields":fields,"buttons":buttons,"message":draft.message,
            "busy":draft.task.is_some(),"finished":draft.finished})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_catalogues_share_runtime_validation_and_refuse_mismatched_items() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let path = |name: &str| root.join(name).display().to_string();
        assert!(
            check_account_catalogues(
                &path("development-equipment-items.json"),
                &path("development-rewards.json")
            )
            .is_ok()
        );
        assert!(
            check_account_catalogues(
                &path("development-loot-items.json"),
                &path("development-creature-loot-rewards.json")
            )
            .is_ok()
        );
        assert!(
            check_account_catalogues(
                &path("development-equipment-items.json"),
                &path("development-creature-loot-rewards.json")
            )
            .is_err()
        );
        assert!(check_account_catalogues("", &path("development-rewards.json")).is_err());
        assert!(
            check_account_catalogues(
                &path("development-loot-items.json"),
                &path("development-quest-required-loot.json")
            )
            .is_err()
        );
    }
    #[test]
    fn slug_is_safe_and_bounded() {
        assert_eq!(slug("My New Zone!"), "my-new-zone");
        assert_eq!(slug("?!"), "untitled");
        assert!(slug(&"a".repeat(100)).len() <= 48);
    }
    #[test]
    fn chunk_export_dialog_uses_two_fields_and_requires_both_paths() {
        let mut app = App::new();
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .insert_resource(Session::new(
                Editor::new(Zone::default()).unwrap(),
                PathBuf::from("zone.json"),
                None,
            ))
            .init_resource::<workspace::Requests>()
            .init_resource::<DirtyState>()
            .init_resource::<EditorState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputFocus>()
            .add_systems(Startup, setup)
            .add_systems(Update, open)
            .add_systems(PostUpdate, finish);
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.update();
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .export_particle_chunks = true;
        app.update();
        let draft = app
            .world()
            .resource::<Session>()
            .export_dialog
            .as_ref()
            .unwrap();
        assert!(draft.mode == Mode::ParticleChunks);
        let mut rows = app.world_mut().query::<(&FieldRow, &Node)>();
        assert_eq!(
            rows.iter(app.world())
                .find(|(row, _)| row.0 == 2)
                .unwrap()
                .1
                .display,
            Display::None
        );
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Export);
        app.update();
        assert!(
            app.world()
                .resource::<Session>()
                .export_dialog
                .as_ref()
                .unwrap()
                .message
                .contains("existing chunk world")
        );
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Close);
        app.update();
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .replace_particle_chunks = true;
        app.update();
        assert!(
            app.world()
                .resource::<Session>()
                .export_dialog
                .as_ref()
                .unwrap()
                .mode
                == Mode::ReplaceParticleChunks
        );
        let mut descriptions = app
            .world_mut()
            .query_filtered::<&Text, With<DialogDescription>>();
        assert!(
            descriptions
                .single(app.world())
                .unwrap()
                .0
                .contains("ALL particle placements")
        );
    }
    #[test]
    fn population_dialog_validates_profile_and_spacing_without_starting_work() {
        let zone = Zone::default();
        let mut app = App::new();
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .insert_resource(Session::new(
                Editor::new(zone.clone()).unwrap(),
                PathBuf::from("zone.json"),
                None,
            ))
            .init_resource::<workspace::Requests>()
            .init_resource::<DirtyState>()
            .init_resource::<EditorState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputFocus>()
            .add_systems(Startup, setup)
            .add_systems(Update, open)
            .add_systems(PostUpdate, finish);
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.update();
        let context = crate::spawn_groups::Context::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../content/gameplay/development-creature-passive.json"),
        )
        .unwrap();
        let creature_path = context.path.display().to_string();
        app.world_mut().resource_mut::<Session>().creature_context = Some(context);
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .export_population = true;
        app.update();
        assert!(
            app.world_mut()
                .query::<(&Field, &EditableText)>()
                .iter(app.world())
                .any(|(field, text)| field.0 == 3 && text.value() == creature_path.as_str())
        );
        for expected in [1, 2, 3, 4, 5, 7, 10, 11, 0] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Tab);
            app.update();
            let focused = app.world().resource::<InputFocus>().get().unwrap();
            assert_eq!(app.world().get::<Field>(focused).unwrap().0, expected);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
        }
        for (profile, spacing, message) in [
            ("", "3", "Choose a creature profile"),
            ("creature.json", "NaN", "Minimum spacing"),
            ("creature.json", "33", "Minimum spacing"),
        ] {
            let mut fields = app.world_mut().query::<(&Field, &mut EditableText)>();
            for (field, mut text) in fields.iter_mut(app.world_mut()) {
                if field.0 == 3 {
                    text.editor_mut().set_text(profile);
                }
                if field.0 == 5 {
                    text.editor_mut().set_text(spacing);
                }
            }
            app.world_mut()
                .resource_mut::<Session>()
                .export_dialog
                .as_mut()
                .unwrap()
                .action = Some(Control::Export);
            app.update();
            let session = app.world().resource::<Session>();
            let draft = session.export_dialog.as_ref().unwrap();
            assert!(draft.message.contains(message), "{}", draft.message);
            assert!(draft.task.is_none());
            assert_eq!(session.editor.zone(), &zone);
        }
    }

    #[test]
    fn account_template_is_visible_switches_catalogues_and_does_not_leak_into_test_players() {
        let mut zone = Zone {
            schema: 12,
            ..default()
        };
        zone.test_players
            .push(theia_zone::test_players::TestPlayer {
                id: 1,
                name: "Test player".into(),
                client_id: 1,
                enabled: true,
                position: zone.spawn,
                controller: None,
                network: default(),
            });
        let mut app = App::new();
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .insert_resource(Session::new(
                Editor::new(zone.clone()).unwrap(),
                PathBuf::from("zone.json"),
                None,
            ))
            .init_resource::<network_playtest::Manager>()
            .init_resource::<workspace::Requests>()
            .init_resource::<DirtyState>()
            .init_resource::<EditorState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputFocus>()
            .add_systems(Startup, setup)
            .add_systems(Update, open)
            .add_systems(PostUpdate, finish);
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.update();
        let template = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/development-gameplay-classic-quest-loot-fixture.json");
        let context = crate::quest_inspector::Context::load(&template).unwrap();
        let selected = context.path.display().to_string();
        app.world_mut().resource_mut::<Session>().quest_context = Some(context);
        app.world_mut()
            .resource_mut::<network_playtest::Manager>()
            .access_path = Some(1201);
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .account_population_playtest = true;
        app.update();
        let mut fields = app.world_mut().query::<(&Field, &EditableText)>();
        assert!(
            fields
                .iter(app.world())
                .any(|(f, t)| f.0 == 10 && t.value() == selected.as_str())
        );
        let mut access_fields = app.world_mut().query::<(&Field, &EditableText)>();
        assert!(
            access_fields
                .iter(app.world())
                .any(|(field, text)| field.0 == 11 && text.value() == "1201")
        );
        let mut rows = app.world_mut().query::<(&FieldRow, &Node)>();
        for (row, node) in rows.iter(app.world()) {
            assert_eq!(node.display == Display::Flex, ![8, 9].contains(&row.0));
        }
        for expected in [1, 2, 3, 4, 5, 6, 7, 10, 11, 12, 0] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Tab);
            app.update();
            let focused = app.world().resource::<InputFocus>().get().unwrap();
            assert_eq!(app.world().get::<Field>(focused).unwrap().0, expected);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
        }
        let mut fields = app.world_mut().query::<(&Field, &mut EditableText)>();
        for (field, mut text) in fields.iter_mut(app.world_mut()) {
            if field.0 == 10 {
                text.editor_mut().set_text("");
            }
        }
        app.update();
        assert!(
            rows.iter(app.world())
                .all(|(_, n)| n.display == Display::Flex)
        );
        assert!(
            app.world()
                .resource::<Session>()
                .export_dialog
                .as_ref()
                .unwrap()
                .task
                .is_none()
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .population_playtest = true;
        app.update();
        assert!(
            rows.iter(app.world())
                .all(|(row, n)| (n.display == Display::Flex) == (row.0 <= 7 || row.0 == 11))
        );
        assert!(
            fields
                .iter(app.world())
                .any(|(f, t)| f.0 == 10 && t.value() == "")
        );
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &zone);
        assert_eq!(session.editor.revision(), 0);
    }

    #[test]
    fn network_dialog_filters_fields_and_rejects_missing_player_assets_before_export() {
        check_network_dialog(false);
        check_network_dialog(true);
    }
    fn check_network_dialog(population: bool) {
        let mut zone = Zone {
            schema: 12,
            ..default()
        };
        zone.test_players
            .push(theia_zone::test_players::TestPlayer {
                id: 1,
                name: "Test player".into(),
                client_id: 1,
                enabled: true,
                position: zone.spawn,
                controller: None,
                network: default(),
            });
        let original = zone.clone();
        let mut app = App::new();
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .insert_resource(Session::new(
                Editor::new(zone).unwrap(),
                PathBuf::from("zone.json"),
                None,
            ))
            .init_resource::<network_playtest::Manager>()
            .init_resource::<workspace::Requests>()
            .init_resource::<DirtyState>()
            .init_resource::<EditorState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputFocus>()
            .add_systems(Startup, setup)
            .add_systems(Update, open)
            .add_systems(PostUpdate, finish);
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.update();
        if population {
            app.world_mut()
                .resource_mut::<workspace::Requests>()
                .population_playtest = true;
        } else {
            app.world_mut()
                .resource_mut::<workspace::Requests>()
                .network_playtest = true;
        }
        app.update();
        let mut rows = app.world_mut().query::<(&FieldRow, &Node)>();
        for (row, node) in rows.iter(app.world()) {
            assert_eq!(
                node.display == Display::Flex,
                if population {
                    row.0 <= 7 || row.0 == 11
                } else {
                    [0, 3, 4].contains(&row.0)
                }
            );
        }
        // Keyboard navigation must skip the hidden manifest metadata fields.
        for expected in if population {
            vec![1, 2, 3, 4, 5, 6, 7, 11, 0]
        } else {
            vec![3, 4, 0]
        } {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Tab);
            app.update();
            let focused = app.world().resource::<InputFocus>().get().unwrap();
            assert_eq!(app.world().get::<Field>(focused).unwrap().0, expected);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
        }
        if population {
            let mut fields = app.world_mut().query::<(&Field, &mut EditableText)>();
            for (field, mut text) in fields.iter_mut(app.world_mut()) {
                if field.0 == 3 {
                    text.editor_mut().set_text("profile.json");
                }
            }
        }
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Export);
        app.update();
        let session = app.world().resource::<Session>();
        let draft = session.export_dialog.as_ref().unwrap();
        assert!(draft.message.contains("Choose a player asset directory"));
        assert!(draft.task.is_none());
        assert_eq!(session.editor.zone(), &original);
        assert!(
            !app.world()
                .resource::<network_playtest::Manager>()
                .running()
        );
    }
    #[test]
    fn dialog_validates_and_exports_without_overwriting_or_saving_zone() {
        check_dialog_export(false);
        check_dialog_export(true);
    }
    fn check_dialog_export(gameplay: bool) {
        use std::time::{Duration, Instant};
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "theia-export-dialog-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        let output = root.join("world");
        let template = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/development-gameplay-classic-quest-areas.json");
        let zone = if gameplay {
            serde_json::from_slice(include_bytes!(
                "../../../content/zones/quest-areas.zone.json"
            ))
            .unwrap()
        } else {
            Zone::default()
        };
        let mut app = App::new();
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .insert_resource(Session::new(
                Editor::new(zone.clone()).unwrap(),
                root.join("zone.json"),
                Some(zone),
            ))
            .init_resource::<workspace::Requests>()
            .init_resource::<DirtyState>()
            .init_resource::<EditorState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputFocus>()
            .add_systems(Startup, setup)
            .add_systems(Update, open)
            .add_systems(PostUpdate, finish);
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.update();
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .export_world = true;
        app.update();
        assert!(app.world().resource::<Session>().has_modal());
        let mut query = app.world_mut().query::<(&Field, &mut EditableText)>();
        for (field, mut text) in query.iter_mut(app.world_mut()) {
            text.clear();
            text.editor_mut().set_text(match field.0 {
                0 => output.to_str().unwrap(),
                1 => "theia.test.dialog",
                2 => "0",
                _ if gameplay => template.to_str().unwrap(),
                _ => "",
            });
        }
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Export);
        app.update();
        assert!(
            app.world()
                .resource::<Session>()
                .export_dialog
                .as_ref()
                .unwrap()
                .message
                .contains("positive")
        );
        assert!(!output.exists());
        for (field, mut text) in query.iter_mut(app.world_mut()) {
            if field.0 == 2 {
                text.clear();
                text.editor_mut().set_text("1");
            }
        }
        app.world_mut()
            .resource_mut::<DirtyState>()
            .has_unsaved_changes = true;
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Export);
        app.update();
        assert!(
            app.world()
                .resource::<Session>()
                .export_dialog
                .as_ref()
                .unwrap()
                .message
                .contains("Save particle")
        );
        assert!(!output.exists());
        app.world_mut()
            .resource_mut::<DirtyState>()
            .has_unsaved_changes = false;
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Export);
        app.update();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !app
            .world()
            .resource::<Session>()
            .export_dialog
            .as_ref()
            .unwrap()
            .finished
        {
            assert!(
                Instant::now() < deadline,
                "export worker did not finish: {}",
                app.world()
                    .resource::<Session>()
                    .export_dialog
                    .as_ref()
                    .unwrap()
                    .message
            );
            std::thread::sleep(Duration::from_millis(5));
            app.update();
        }
        assert!(output.join("world.manifest.json").is_file());
        assert_eq!(
            output.join("gameplay/gameplay.manifest.json").is_file(),
            gameplay
        );
        if gameplay {
            theia_network::gameplay_bundle::validate(
                &output.join("gameplay/gameplay.manifest.json"),
            )
            .unwrap();
        }
        assert!(
            app.world()
                .resource::<Session>()
                .export_dialog
                .as_ref()
                .unwrap()
                .message
                .contains(output.to_str().unwrap())
        );
        assert!(!root.join("zone.json").exists());
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Close);
        app.update();
        assert!(!app.world().resource::<Session>().has_modal());
        let original = std::fs::read(output.join("world.manifest.json")).unwrap();
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .export_world = true;
        app.update();
        for (field, mut text) in query.iter_mut(app.world_mut()) {
            text.clear();
            text.editor_mut().set_text(match field.0 {
                0 => output.to_str().unwrap(),
                1 => "theia.test.dialog",
                2 => "2",
                _ if gameplay => template.to_str().unwrap(),
                _ => "",
            });
        }
        app.world_mut()
            .resource_mut::<Session>()
            .export_dialog
            .as_mut()
            .unwrap()
            .action = Some(Control::Export);
        app.update();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let draft = app
                .world()
                .resource::<Session>()
                .export_dialog
                .as_ref()
                .unwrap();
            if draft.task.is_none() && draft.message.contains("new output directory") {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "existing destination was not rejected"
            );
            std::thread::sleep(Duration::from_millis(5));
            app.update();
        }
        assert_eq!(
            std::fs::read(output.join("world.manifest.json")).unwrap(),
            original
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
