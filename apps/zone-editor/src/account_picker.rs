//! Native character choice for the private owned account-launcher handoff.
use super::*;
use theia_core::appearance::{Appearance, LABELS};
mod preview;
use bevy::input_focus::{
    FocusCause, InputFocus,
    tab_navigation::{TabGroup, TabIndex},
};
use bevy::text::EditableText;
use bevy::ui_widgets::{SelectAllOnFocus, TextInput};
pub(super) use preview::install;
use serde::Deserialize;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Character {
    id: String,
    name: String,
    #[serde(default)]
    appearance: Appearance,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Characters {
    schema: u32,
    characters: Vec<Character>,
    can_create: bool,
}
fn parse_snapshot(bytes: &[u8]) -> Result<Characters, String> {
    if bytes.len() > 4096 {
        return Err("Character list exceeds its size limit".into());
    }
    let data: Characters =
        serde_json::from_slice(bytes).map_err(|_| "Invalid account character list")?;
    if data.schema != 1
        || data.characters.len() > 8
        || data.can_create != (data.characters.len() < 8)
    {
        return Err("Invalid account character list".into());
    }
    let mut seen = std::collections::HashSet::new();
    for row in &data.characters {
        let id = uuid::Uuid::parse_str(&row.id).map_err(|_| "Invalid character identity")?;
        if id.is_nil()
            || id.to_string() != row.id
            || !seen.insert(id)
            || row.name.is_empty()
            || row.name.chars().count() > 80
            || row.name.chars().any(char::is_control)
        {
            return Err("Invalid account character list".into());
        }
    }
    Ok(data)
}
fn read_snapshot(directory: &Path) -> Result<Option<Characters>, String> {
    let file = match fs::File::open(directory.join("characters.json")) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Cannot read account character list: {e}")),
    };
    let mut bytes = Vec::new();
    file.take(4097)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    parse_snapshot(&bytes).map(Some)
}
#[derive(Component, Clone, Copy, Debug)]
enum Choice {
    Existing(usize),
    Create,
    Enter,
    New,
    Cycle(usize, bool),
    Rotate(bool),
    Cancel,
}
fn publish_choice(
    directory: &Path,
    data: &Characters,
    choice: Choice,
    name: &str,
    appearance: Appearance,
) -> Result<(), String> {
    let selection = match choice {
        Choice::Existing(index) => {
            let row = data
                .characters
                .get(index)
                .ok_or("Choose a listed character")?;
            serde_json::json!({"schema":1,"character_id":row.id})
        }
        Choice::Create if data.can_create => {
            let name = name.trim();
            theia_core::character_name::CharacterName::parse(name).map_err(|e| e.to_string())?;
            serde_json::json!({"schema":1,"create_name":name,"appearance":appearance})
        }
        _ => return Err("Choose an existing character".into()),
    };
    let destination = directory.join("selection.json");
    if destination.exists() {
        return Err("Character selection has already been sent".into());
    }
    let pending = directory.join("selection.pending");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| e.to_string())?;
    let result = file
        .write_all(&serde_json::to_vec(&selection).unwrap())
        .map_err(|e| e.to_string());
    drop(file);
    let result = result.and_then(|()| fs::rename(&pending, destination).map_err(|e| e.to_string()));
    if result.is_err() {
        let _ = fs::remove_file(pending);
    }
    result
}
struct Draft {
    directory: PathBuf,
    data: Characters,
    action: Option<Choice>,
    message: String,
    appearance: Appearance,
    selected: Option<usize>,
    yaw: f32,
}
#[derive(Resource, Default)]
struct Picker {
    draft: Option<Draft>,
    seen: Option<PathBuf>,
    elapsed: f32,
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Row(usize);
#[derive(Component)]
enum Label {
    Character(usize),
    Message,
    Option(usize),
}
#[derive(Component)]
struct NameInput;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Picker>()
        .add_systems(Startup, setup)
        .add_observer(choose)
        .add_systems(
            Update,
            (
                update.after(network_playtest::poll),
                preview::tick.after(update),
                sync_controls.after(update),
            ),
        );
}
fn button(parent: &mut ChildSpawnerCommands, choice: Choice, caption: &str) {
    parent
        .spawn((
            choice,
            Button,
            ActivateOnPress,
            Hovered::default(),
            TabIndex(0),
            theme::Variant::Default,
            theme::button_node(),
            BackgroundColor(theme::Variant::Default.background(false)),
        ))
        .with_child(theme::label(caption, 13., theme::BODY));
}
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let camera = preview::setup(&mut commands, &mut images);
    commands
        .spawn((
            Root,
            TabGroup::modal(),
            GlobalZIndex(35),
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
            root.spawn(theme::dialog::panel(940.))
                .with_children(|panel| {
                    panel
                        .spawn(theme::dialog::header())
                        .with_child(theme::label("Your adventurer", 18., theme::DISPLAY));
                    scrolling::pane(panel, 24., scrolling::Pane::Dialog, |body| {
                        body.spawn(theme::label(
                            "Choose an existing character or create one for this account.",
                            13.,
                            theme::BODY,
                        ));
                        body.spawn(Node {
                            width: percent(100),
                            column_gap: px(24),
                            ..default()
                        })
                        .with_children(|columns| {
                            preview::panel(columns, camera);
                            columns
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    flex_grow: 1.,
                                    row_gap: px(6),
                                    min_width: px(0),
                                    ..default()
                                })
                                .with_children(|body| {
                                    for index in 0..8 {
                                        body.spawn((
                                            Row(index),
                                            Choice::Existing(index),
                                            Button,
                                            ActivateOnPress,
                                            Hovered::default(),
                                            TabIndex(0),
                                            theme::Variant::Default,
                                            theme::button_node(),
                                            BackgroundColor(
                                                theme::Variant::Default.background(false),
                                            ),
                                        ))
                                        .with_child((
                                            Label::Character(index),
                                            theme::label("", 13., theme::BODY),
                                        ));
                                    }
                                    button(body, Choice::Enter, "Enter selected character");
                                    body.spawn((
                                        Row(8),
                                        Node {
                                            width: percent(100),
                                            flex_direction: FlexDirection::Column,
                                            row_gap: px(8),
                                            margin: UiRect::top(px(12)),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|form| {
                                        button(form, Choice::New, "Create a new adventurer");
                                        for (field, label) in LABELS.iter().enumerate() {
                                            form.spawn(Node {
                                                align_items: AlignItems::Center,
                                                column_gap: px(6),
                                                ..default()
                                            })
                                            .with_children(|row| {
                                                button(row, Choice::Cycle(field, false), "‹");
                                                row.spawn((
                                                    Label::Option(field),
                                                    theme::label(label, 13., theme::BODY),
                                                    Node {
                                                        flex_grow: 1.,
                                                        ..default()
                                                    },
                                                ));
                                                button(row, Choice::Cycle(field, true), "›");
                                            });
                                        }
                                        form.spawn(theme::label(
                                            "New character name",
                                            13.,
                                            theme::BODY,
                                        ));
                                        form.spawn((
                                            NameInput,
                                            TabIndex(0),
                                            TextInput,
                                            SelectAllOnFocus,
                                            EditableText {
                                                max_characters: Some(20),
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
                                        button(form, Choice::Create, "Create character and enter");
                                    });
                                });
                        });
                        body.spawn((Label::Message, theme::label("", 12., theme::MUTED)));
                    });
                    panel
                        .spawn(theme::dialog::footer())
                        .with_children(|footer| button(footer, Choice::Cancel, "Cancel playtest"));
                });
        });
}
fn choose(event: On<Activate>, choices: Query<&Choice>, mut picker: ResMut<Picker>) {
    if let Some(draft) = &mut picker.draft
        && let Ok(choice) = choices.get(event.entity)
    {
        draft.action = Some(*choice);
    }
}
#[allow(clippy::too_many_arguments)]
fn update(
    mut picker: ResMut<Picker>,
    mut manager: ResMut<network_playtest::Manager>,
    mut session: ResMut<Session>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut focus: ResMut<InputFocus>,
    mut roots: Query<&mut Node, (With<Root>, Without<Row>)>,
    mut rows: Query<(&Row, &mut Node), Without<Root>>,
    mut labels: Query<(&Label, &mut Text)>,
    mut names: Query<(Entity, &mut EditableText), With<NameInput>>,
    choices: Query<(Entity, &Choice)>,
) {
    picker.elapsed += time.delta_secs();
    if !manager.running() || !manager.account {
        picker.draft = None;
    }
    if picker.draft.is_none()
        && manager.running()
        && manager.account
        && !session.has_modal()
        && picker.elapsed >= 0.2
    {
        picker.elapsed = 0.;
        if let Some(directory) = manager.output.as_ref().map(|p| p.join("runtime"))
            && picker.seen.as_ref() != Some(&directory)
        {
            match read_snapshot(&directory) {
                Ok(Some(data)) => {
                    let empty = data.characters.is_empty();
                    picker.seen = Some(directory.clone());
                    let appearance = data
                        .characters
                        .first()
                        .map_or_else(Appearance::default, |c| c.appearance);
                    picker.draft = Some(Draft {
                        directory,
                        data,
                        action: None,
                        message: String::new(),
                        appearance,
                        selected: (!empty).then_some(0),
                        yaw: 0.,
                    });
                    if let Ok((entity, mut name)) = names.single_mut() {
                        // Preserve Bevy's initialized editor layout/style state.
                        name.clear();
                        if empty {
                            focus.set(entity, FocusCause::Navigated);
                        }
                    }
                    if !empty
                        && let Some((entity, _)) = choices
                            .iter()
                            .find(|(_, c)| matches!(c, Choice::Existing(0)))
                    {
                        focus.set(entity, FocusCause::Navigated);
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    picker.seen = Some(directory);
                    manager.stop();
                    session.status = error;
                }
            }
        }
    }
    let mut close = false;
    if let Some(draft) = &mut picker.draft {
        if windows.single().is_ok_and(|w| w.focused)
            && focus
                .get()
                .is_some_and(|entity| names.contains(entity) || choices.contains(entity))
        {
            if keys.just_pressed(KeyCode::Escape) {
                draft.action = Some(Choice::Cancel);
            }
            if keys.just_pressed(KeyCode::Enter)
                && draft.data.can_create
                && focus.get().is_some_and(|entity| names.contains(entity))
            {
                draft.action = Some(Choice::Create);
            }
        }
        if let Some(choice) = draft.action.take() {
            match choice {
                Choice::Cancel => {
                    manager.stop();
                    close = true;
                    session.status = "Stopping account playtest…".into();
                }
                Choice::Existing(index) => {
                    if let Some(character) = draft.data.characters.get(index) {
                        draft.selected = Some(index);
                        draft.appearance = character.appearance;
                        draft.message = format!("{} — ready to enter", character.name);
                    }
                }
                Choice::New if draft.data.can_create => {
                    draft.selected = None;
                    draft.appearance = Appearance::default();
                    draft.message = "Choose your appearance and enter a name".into();
                }
                Choice::Cycle(field, forward) if draft.data.can_create => {
                    if draft.selected.take().is_some() {
                        draft.message = "Creating a new adventurer".into();
                    }
                    draft.appearance = draft.appearance.cycle(field, forward);
                }
                Choice::Rotate(forward) => {
                    draft.yaw += if forward { 0.4 } else { -0.4 };
                }
                Choice::Create | Choice::Enter => {
                    let actual = match choice {
                        Choice::Enter => draft.selected.map(Choice::Existing),
                        _ => Some(Choice::Create),
                    };
                    if let Some(actual) = actual {
                        let name = names
                            .single()
                            .map_or_else(|_| String::new(), |(_, n)| n.value().to_string());
                        match publish_choice(
                            &draft.directory,
                            &draft.data,
                            actual,
                            &name,
                            draft.appearance,
                        ) {
                            Ok(()) => {
                                close = true;
                                session.status = "Entering account playtest…".into();
                            }
                            Err(error) => draft.message = error,
                        }
                    } else {
                        draft.message = "Select an existing character first".into();
                    }
                }
                _ => {}
            }
        }
    }

    if close {
        picker.draft = None;
    }
    if picker.draft.is_none()
        && session.account_picker_open
        && focus
            .get()
            .is_some_and(|e| names.contains(e) || choices.contains(e))
    {
        focus.clear();
    }
    session.account_picker_open = picker.draft.is_some();
    for mut root in &mut roots {
        root.display = if session.account_picker_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Some(draft) = &picker.draft {
        for (row, mut node) in &mut rows {
            node.display = if if row.0 == 8 {
                draft.data.can_create
            } else {
                row.0 < draft.data.characters.len()
            } {
                Display::Flex
            } else {
                Display::None
            };
        }
        for (label, mut text) in &mut labels {
            **text = match label {
                Label::Character(i) => draft
                    .data
                    .characters
                    .get(*i)
                    .map_or_else(String::new, |c| c.name.clone()),
                Label::Message => draft.message.clone(),
                Label::Option(i) => {
                    let value = draft.appearance.values()[*i];
                    let option = match *i {
                        0 => ["Masculine", "Feminine"][usize::from(value)].to_owned(),
                        3 if draft.appearance.values()[0] == 1 => "Unavailable".into(),
                        2 | 3 if value == 0 => "None".into(),
                        2 | 3 => format!("Style {value}"),
                        5 => ["Brown", "Black", "Blond", "Auburn"][usize::from(value)].into(),
                        6 => ["Traveller", "Adventurer"][usize::from(value)].into(),
                        _ => format!("{}", value + 1),
                    };
                    format!("{}: {}", LABELS[*i], option)
                }
            };
        }
    }
}

/// Read-only geometry for explicit native UI qualification; contains no session credentials.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Report<'w, 's> {
    picker: Option<Res<'w, Picker>>,
    preview: Option<Res<'w, preview::Preview>>,
    controls: Query<
        'w,
        's,
        (
            &'static Choice,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
        ),
    >,
    names: Query<
        'w,
        's,
        (
            Entity,
            &'static EditableText,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
        ),
        With<NameInput>,
    >,
}
impl Report<'_, '_> {
    pub(super) fn snapshot(&self, scale: f32, focus: &InputFocus) -> serde_json::Value {
        let Some(draft) = self.picker.as_ref().and_then(|p| p.draft.as_ref()) else {
            return serde_json::Value::Null;
        };
        let controls: Vec<_> = self.controls.iter().map(|(choice, node, transform)| {
            let factor = node.inverse_scale_factor() * scale;
            serde_json::json!({"action":format!("{choice:?}"),"centre":(transform.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()})
        }).collect();
        let name = self.names.iter().next().map(|(entity, text, node, transform)| {
            let factor = node.inverse_scale_factor() * scale;
            serde_json::json!({"text":text.value().to_string(),"focused":focus.get()==Some(entity),"centre":(transform.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()})
        });
        serde_json::json!({"characters":draft.data.characters.iter().map(|c| &c.name).collect::<Vec<_>>(),"preview":self.preview.as_ref().map(|p| serde_json::json!({"ready":p.ready,"error":p.error,"heroes":p.evidence})),"appearance":draft.appearance,"selected":draft.selected,"can_create":draft.data.can_create,"message":draft.message,"controls":controls,"name":name})
    }
}

fn sync_controls(
    mut commands: Commands,
    picker: Res<Picker>,
    mut controls: Query<(
        Entity,
        &Choice,
        &mut theme::Variant,
        Has<bevy::ui::InteractionDisabled>,
    )>,
) {
    let Some(draft) = &picker.draft else {
        return;
    };
    for (entity, choice, mut variant, disabled) in &mut controls {
        let enabled = match choice {
            Choice::Enter => draft.selected.is_some(),
            Choice::Cycle(3, _) => draft.data.can_create && draft.appearance.values()[0] == 0,
            Choice::Create | Choice::New | Choice::Cycle(_, _) => draft.data.can_create,
            _ => true,
        };
        if disabled == enabled {
            if enabled {
                commands
                    .entity(entity)
                    .remove::<bevy::ui::InteractionDisabled>();
            } else {
                commands
                    .entity(entity)
                    .insert(bevy::ui::InteractionDisabled);
            }
            *variant = if enabled {
                theme::Variant::Default
            } else {
                theme::Variant::Disabled
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot() -> serde_json::Value {
        serde_json::json!({"schema":1,"can_create":true,"characters":[{"id":"74686569-6100-0000-0000-000000000101","name":"Arden"}]})
    }
    #[test]
    fn picker_refuses_invalid_id_names_limits_and_unknown_fields() {
        let source = snapshot();
        assert!(parse_snapshot(&serde_json::to_vec(&source).unwrap()).is_ok());
        for (path, value) in [
            ("/schema", serde_json::json!(2)),
            ("/can_create", serde_json::json!(false)),
            ("/characters/0/id", serde_json::json!("invalid")),
            (
                "/characters/0/id",
                serde_json::json!("00000000-0000-0000-0000-000000000000"),
            ),
            ("/characters/0/name", serde_json::json!("Arden\n")),
            ("/characters/0/name", serde_json::json!("")),
        ] {
            let mut changed = source.clone();
            *changed.pointer_mut(path).unwrap() = value;
            assert!(
                parse_snapshot(&serde_json::to_vec(&changed).unwrap()).is_err(),
                "{path}"
            );
        }
        let mut changed = source.clone();
        changed["characters"] =
            serde_json::json!([source["characters"][0], source["characters"][0]]);
        assert!(parse_snapshot(&serde_json::to_vec(&changed).unwrap()).is_err());
        let mut changed = source;
        changed["unexpected"] = true.into();
        assert!(parse_snapshot(&serde_json::to_vec(&changed).unwrap()).is_err());
        assert!(parse_snapshot(&vec![b' '; 4097]).is_err());
    }
    #[test]
    fn selection_is_atomic_bounded_to_displayed_rows_and_cannot_overwrite_a_choice() {
        let path = std::env::temp_dir().join(format!(
            "theia-account-picker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        let data = parse_snapshot(&serde_json::to_vec(&snapshot()).unwrap()).unwrap();
        assert!(
            publish_choice(&path, &data, Choice::Existing(8), "", Appearance::default()).is_err()
        );
        assert!(
            publish_choice(
                &path,
                &data,
                Choice::Create,
                "Ar--den",
                Appearance::default()
            )
            .is_err()
        );
        assert!(!path.join("selection.json").exists());
        publish_choice(&path, &data, Choice::Existing(0), "", Appearance::default()).unwrap();
        let bytes = fs::read(path.join("selection.json")).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["character_id"],
            data.characters[0].id
        );
        assert!(
            publish_choice(
                &path,
                &data,
                Choice::Create,
                "Newhero",
                Appearance::default()
            )
            .is_err()
        );
        assert_eq!(fs::read(path.join("selection.json")).unwrap(), bytes);
        assert!(!path.join("selection.pending").exists());
        fs::remove_file(path.join("selection.json")).unwrap();
        publish_choice(
            &path,
            &data,
            Choice::Create,
            " Newhero ",
            Appearance::default(),
        )
        .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &fs::read(path.join("selection.json")).unwrap()
            )
            .unwrap()["create_name"],
            "Newhero"
        );
        fs::remove_dir_all(path).unwrap();
    }
}
