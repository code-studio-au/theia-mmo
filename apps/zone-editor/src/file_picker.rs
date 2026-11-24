//! Bounded standard-Bevy zone browser, over the retained validated file APIs.
use super::*;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::text::{EditableText, TextEdit};
use bevy::ui_widgets::{SelectAllOnFocus, TextInput};

const ROWS: usize = 12;
const SCAN_LIMIT: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Open,
    Gameplay,
    Creature,
    SaveAs,
}
impl Mode {
    fn prompt(self) -> &'static str {
        match self {
            Self::Gameplay => "Choose a gameplay manifest",
            Self::Creature => "Choose a creature definition",
            _ => "Choose a zone file",
        }
    }
}
#[derive(Clone)]
pub(super) struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub directory: bool,
}
pub(super) struct Draft {
    pub mode: Mode,
    pub folder: PathBuf,
    pub text: String,
    pub selection: [usize; 2],
    pub page: usize,
    pub entries: Vec<Entry>,
    pub overwrite: bool,
    truncated: bool,
    revision: u64,
    action: Option<Control>,
}
impl Draft {
    fn refresh(&mut self, folder: PathBuf) -> Result<(), String> {
        let folder = std::fs::canonicalize(folder).map_err(|e| e.to_string())?;
        let mut entries = vec![];
        let mut truncated = false;
        let listing = std::fs::read_dir(&folder).map_err(|e| e.to_string())?;
        for (index, entry) in listing.take(SCAN_LIMIT + 1).enumerate() {
            if index == SCAN_LIMIT {
                truncated = true;
                break;
            }
            let entry = entry.map_err(|e| e.to_string())?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let path = entry.path();
            let directory = path.is_dir();
            if directory || is_json(&path) {
                entries.push(Entry {
                    name,
                    path,
                    directory,
                });
            }
        }
        entries.sort_by(|a, b| b.directory.cmp(&a.directory).then(a.name.cmp(&b.name)));
        self.folder = folder;
        self.entries = entries;
        self.truncated = truncated;
        self.page = 0;
        self.overwrite = false;
        Ok(())
    }
    fn browse(&mut self, folder: PathBuf) -> Result<(), String> {
        let name = if self.mode == Mode::SaveAs {
            std::path::Path::new(&self.text)
                .file_name()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
        } else {
            String::new()
        };
        self.overwrite = false;
        self.refresh(folder)?;
        self.text = name;
        Ok(())
    }
    pub fn pages(&self) -> usize {
        self.entries.len().div_ceil(ROWS).max(1)
    }
    pub fn row(&self, index: usize) -> Option<&Entry> {
        self.entries.get(self.page * ROWS + index)
    }
    fn path(&self) -> Result<PathBuf, String> {
        if self.text.trim().is_empty() || self.text.contains(['\n', '\r', '\0']) {
            return Err("Enter a zone filename or directory".into());
        }
        let mut path = self.folder.join(&self.text);
        if self.mode == Mode::SaveAs && path.extension().is_none() && !path.is_dir() {
            path.set_extension("zone.json");
        }
        if !path.is_dir() && !is_json(&path) {
            return Err("Choose a .json file".into());
        }
        Ok(path)
    }
}
fn tail(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.into()
    } else {
        format!(
            "…{}",
            value
                .chars()
                .rev()
                .take(limit)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>()
        )
    }
}
fn is_json(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("json"))
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Field;
#[derive(Component)]
pub(super) struct Caption;
#[derive(Component)]
pub(super) struct Description;
#[derive(Component)]
pub(super) struct Label(Control);
#[derive(Component)]
pub(super) struct RowIcon(usize);
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    Field,
    Row(usize),
    Up,
    Previous,
    Next,
    Apply,
    Cancel,
}
fn button_base(control: Control) -> impl Bundle {
    (
        control,
        Button,
        Hovered::default(),
        Node {
            justify_content: if matches!(control, Control::Row(_)) {
                JustifyContent::Start
            } else {
                JustifyContent::Center
            },
            overflow: Overflow::clip(),
            column_gap: px(6),
            ..theme::button_node()
        },
        BackgroundColor(theme::Variant::Default.background(false)),
    )
}
fn button(control: Control, label: &str) -> impl Bundle {
    (
        button_base(control),
        children![(
            Label(control),
            theme::label(label, 12., theme::BODY),
            TextLayout::no_wrap()
        )],
    )
}
pub(super) fn setup(mut commands: Commands, icons: Option<Res<browser_icons::Icons>>) {
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
            root.spawn(theme::dialog::panel(900.))
                .with_children(|panel| {
                    panel
                        .spawn(theme::dialog::header())
                        .with_child((Caption, theme::label("Zone files", 18., theme::DISPLAY)));
                    scrolling::pane(panel, 24., scrolling::Pane::Dialog, |body| {
                        body.spawn((Description, theme::label("", 14., theme::MUTED)));
                        body.spawn(Node {
                            column_gap: px(6),
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            for (control, label) in [
                                (Control::Up, "Up folder"),
                                (Control::Previous, "Previous page"),
                                (Control::Next, "Next page"),
                            ] {
                                row.spawn(button(control, label));
                            }
                        });
                        for index in 0..ROWS {
                            body.spawn(button_base(Control::Row(index)))
                                .with_children(|row| {
                                    row.spawn((
                                        RowIcon(index),
                                        Pickable::IGNORE,
                                        ImageNode {
                                            image: icons
                                                .as_ref()
                                                .map_or_else(Handle::default, |icons| {
                                                    icons.file.clone()
                                                }),
                                            color: theme::BODY,
                                            ..default()
                                        },
                                        Node {
                                            width: px(16),
                                            height: px(16),
                                            flex_shrink: 0.,
                                            ..default()
                                        },
                                    ));
                                    row.spawn((
                                        Label(Control::Row(index)),
                                        theme::label("", 12., theme::BODY),
                                        TextLayout::no_wrap(),
                                    ));
                                });
                        }
                        body.spawn((theme::label("Filename or path (.json)", 12., theme::MUTED),));
                        body.spawn((
                            Field,
                            Control::Field,
                            TextInput,
                            SelectAllOnFocus,
                            Hovered::default(),
                            EditableText {
                                max_characters: Some(4096),
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
                        body.spawn((theme::label(
                            "Enter opens/saves | Escape cancels",
                            12.,
                            theme::MUTED,
                        ),));
                    });
                    panel.spawn(theme::dialog::footer()).with_children(|row| {
                        row.spawn(button(Control::Cancel, "Cancel"));
                        row.spawn(button(Control::Apply, "Open"));
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
        && !matches!(control, Control::Field)
        && let Some(draft) = &mut session.file_dialog
    {
        draft.action = Some(*control);
    }
}
pub(super) fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    mut fields: Query<(Entity, &mut EditableText), With<Field>>,
    mut requests: ResMut<workspace::Requests>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let open = std::mem::take(&mut requests.file_open)
        || (ctrl && !shift && keys.just_pressed(KeyCode::KeyO));
    let implicit_save_as = std::mem::take(&mut session.save_as_requested);
    let save_as = std::mem::take(&mut requests.file_save_as)
        || implicit_save_as
        || (ctrl && shift && keys.just_pressed(KeyCode::KeyS));
    let gameplay = std::mem::take(&mut requests.gameplay_open);
    let creature = std::mem::take(&mut requests.creature_open);
    let mode = if creature {
        Some(Mode::Creature)
    } else if gameplay {
        Some(Mode::Gameplay)
    } else if open {
        Some(Mode::Open)
    } else if save_as {
        Some(Mode::SaveAs)
    } else {
        None
    };
    let Some(mode) = mode else {
        return;
    };
    if session.has_modal() || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
    {
        session.status =
            "Finish or cancel the preview, or leave playtest, before choosing a file".into();
        return;
    }
    let base = if mode == Mode::Creature {
        session
            .creature_context
            .as_ref()
            .map_or_else(|| session.picker_path(), |c| c.path.clone())
    } else if mode == Mode::Gameplay {
        session
            .quest_context
            .as_ref()
            .map_or_else(|| session.picker_path(), |c| c.path.clone())
    } else {
        session.picker_path()
    };
    let Some(folder) = base.parent() else {
        session.status = "Current save path has no folder".into();
        return;
    };
    let mut draft = Draft {
        mode,
        folder: folder.into(),
        text: if mode == Mode::SaveAs
            || (mode == Mode::Gameplay && session.quest_context.is_some())
            || (mode == Mode::Creature && session.creature_context.is_some())
        {
            base.file_name()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
        } else {
            String::new()
        },
        selection: [0; 2],
        page: 0,
        entries: vec![],
        overwrite: false,
        truncated: false,
        revision: session.editor.revision(),
        action: None,
    };
    let folder = if folder.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        folder.into()
    };
    if let Err(error) = draft.refresh(folder) {
        session.status = error;
        return;
    }
    let Ok((entity, mut field)) = fields.single_mut() else {
        return;
    };
    field.clear();
    field.editor_mut().set_text(&draft.text);
    field.queue_edit(TextEdit::SelectAll);
    focus.set(entity, FocusCause::Navigated);
    session.status = match mode {
        Mode::Open => "Choose a zone file; undo can restore the previous document",
        Mode::SaveAs => "Choose a new zone filename",
        Mode::Gameplay => "Choose a gameplay manifest; quest data is loaded read-only",
        Mode::Creature => "Choose a creature definition; assign it to a spawn group after loading",
    }
    .into();
    session.file_dialog = Some(draft);
}
fn submit(draft: &mut Draft, session: &mut Session, view: &mut View) -> Result<bool, String> {
    if session.editor.revision() != draft.revision {
        return Err("Stale file dialog; cancel and reopen".into());
    }
    let path = draft.path()?;
    if path.is_dir() {
        if draft.mode != Mode::SaveAs {
            draft.refresh(path)?;
            draft.text.clear();
            session.status = draft.mode.prompt().into();
            return Ok(false);
        }
        return Err("Save As needs a filename, not a directory".into());
    }
    match draft.mode {
        Mode::Creature => {
            let context = crate::spawn_groups::Context::load(&path)?;
            session.status = format!("Loaded creature definition {}", context.path.display());
            session.creature_context = Some(context);
        }
        Mode::Gameplay => {
            let context = crate::quest_inspector::Context::load(&path)?;
            session.status = format!("Loaded gameplay {}", context.path.display());
            session.quest_context = Some(context);
        }
        Mode::SaveAs => {
            if path.exists() && !draft.overwrite {
                draft.overwrite = true;
                session.status = format!(
                    "File exists: {}. Click Overwrite file to replace it.",
                    path.display()
                );
                return Ok(false);
            }
            session.editor.zone().save(&path)?;
            session.assign_path(path);
            session.saved_zone = Some(session.editor.zone().clone());
            session.status = format!("Saved {}", session.path_label());
        }
        Mode::Open => {
            let zone = Zone::load(&path)?;
            session.editor.replace(zone, draft.revision)?;
            session.set_mode(EditorMode::Objects);
            session.assign_path(path);
            session.reconcile();
            session.saved_zone = Some(session.editor.zone().clone());
            view.target =
                Vec3::from_array(session.editor.zone().local(session.editor.zone().spawn)?);
            view.distance = 32.;
            camera_views::Preset::Perspective.select(view);
            session.status = format!(
                "Opened {} (undo restores prior document)",
                session.path_label()
            );
        }
    }
    Ok(true)
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
    mut focus: ResMut<InputFocus>,
    mut fields: Query<(Entity, &mut EditableText), With<Field>>,
    mut roots: Query<&mut Node, With<Root>>,
    mut controls: Query<(&Control, &Hovered, &mut Node, &mut BackgroundColor), Without<Root>>,
    mut labels: Query<(&Label, &mut Text)>,
    mut captions: Query<&mut Text, (With<Caption>, Without<Label>)>,
    mut descriptions: Query<&mut Text, (With<Description>, Without<Caption>, Without<Label>)>,
    icons: Option<Res<browser_icons::Icons>>,
    mut row_icons: Query<(&RowIcon, &mut ImageNode)>,
) {
    let Some(mut draft) = session.file_dialog.take() else {
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
        return;
    };
    let Ok((entity, mut field)) = fields.single_mut() else {
        session.file_dialog = Some(draft);
        return;
    };
    let typed = field.value().to_string();
    if typed != draft.text {
        draft.text = typed;
        draft.overwrite = false;
        session.status = draft.mode.prompt().into();
    }
    let range = field.editor().raw_selection().text_range();
    draft.selection = [range.start, range.end];
    let cancel = !windows.single().is_ok_and(|w| w.focused)
        || keys.just_pressed(KeyCode::Escape)
        || matches!(draft.action, Some(Control::Cancel));
    let apply = !field.is_composing()
        && (keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || matches!(draft.action, Some(Control::Apply)));
    let mut close = cancel;
    let mut reset = false;
    if cancel {
        session.status = "File dialog cancelled".into();
    } else if apply {
        let old = draft.text.clone();
        match submit(&mut draft, &mut session, &mut view) {
            Ok(done) => close = done,
            Err(error) => session.status = error,
        }
        reset = draft.text != old;
    } else {
        let folder = match draft.action {
            Some(Control::Up) => draft.folder.parent().map(std::path::Path::to_path_buf),
            Some(Control::Row(index)) => draft
                .row(index)
                .filter(|e| e.directory)
                .map(|e| e.path.clone()),
            _ => None,
        };
        if let Some(folder) = folder {
            match draft.browse(folder) {
                Ok(()) => {
                    reset = true;
                    session.status = draft.mode.prompt().into();
                }
                Err(error) => session.status = error,
            }
        } else {
            match draft.action {
                Some(Control::Previous) => draft.page = draft.page.saturating_sub(1),
                Some(Control::Next) => draft.page = (draft.page + 1).min(draft.pages() - 1),
                Some(Control::Row(index)) => {
                    if let Some(entry) = draft.row(index) {
                        draft.text = entry.name.clone();
                        draft.overwrite = false;
                        reset = true;
                        session.status = draft.mode.prompt().into();
                    }
                }
                _ => {}
            }
        }
    }
    if close {
        focus.clear();
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
    } else {
        if reset {
            field.clear();
            field.editor_mut().set_text(&draft.text);
            field.queue_edit(TextEdit::SelectAll);
        }
        if focus.get() != Some(entity) {
            focus.set(entity, FocusCause::Navigated);
        }
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::Flex;
        }
        for (control, hovered, mut node, mut background) in &mut controls {
            if !matches!(control, Control::Field) {
                let selected = matches!(control,Control::Row(index) if draft.row(*index).is_some_and(|e|!e.directory && e.name==draft.text));
                let unavailable = matches!(control,Control::Previous if draft.page==0)
                    || matches!(control,Control::Next if draft.page+1==draft.pages())
                    || matches!(control,Control::Up if draft.folder.parent().is_none());
                background.0 = if unavailable {
                    theme::Variant::Disabled.background(false)
                } else if matches!(control, Control::Apply) && draft.overwrite {
                    theme::Variant::Destructive.background(hovered.0)
                } else if matches!(control, Control::Apply) {
                    theme::Variant::Primary.background(hovered.0)
                } else if selected {
                    theme::Variant::Active.background(false)
                } else if matches!(control, Control::Row(_)) {
                    theme::Variant::Ghost.background(hovered.0)
                } else if hovered.0 {
                    theme::Variant::Default.background(true)
                } else {
                    theme::Variant::Default.background(false)
                };
            }
            node.display = if matches!(control,Control::Row(index) if draft.row(*index).is_none()) {
                Display::None
            } else {
                Display::Flex
            };
        }
        for (RowIcon(index), mut image) in &mut row_icons {
            if let Some(entry) = draft.row(*index) {
                if let Some(icons) = &icons {
                    image.image = if entry.directory {
                        icons.folder.clone()
                    } else {
                        icons.file.clone()
                    };
                }
                image.color = if !entry.directory && entry.name == draft.text {
                    theme::Variant::Active.foreground()
                } else {
                    theme::BODY
                };
            }
        }
        for (label, mut text) in &mut labels {
            **text = match label.0 {
                Control::Row(index) => draft
                    .row(index)
                    .map_or_else(String::new, |e| e.name.clone()),
                Control::Apply => if draft.overwrite {
                    "Overwrite file"
                } else if draft.mode == Mode::Creature {
                    "Load creature definition"
                } else if draft.mode == Mode::Gameplay {
                    "Load gameplay"
                } else if draft.mode == Mode::Open {
                    "Open zone"
                } else {
                    "Save As"
                }
                .into(),
                Control::Up => "Up folder".into(),
                Control::Previous => "Previous page".into(),
                Control::Next => "Next page".into(),
                Control::Cancel => "Cancel".into(),
                Control::Field => String::new(),
            };
        }
        if let Ok(mut caption) = captions.single_mut() {
            **caption = if draft.mode == Mode::Creature {
                "Load creature definition"
            } else if draft.mode == Mode::Gameplay {
                "Load gameplay"
            } else if draft.mode == Mode::Open {
                "Open zone"
            } else {
                "Save zone as"
            }
            .into();
        }
        if let Ok(mut description) = descriptions.single_mut() {
            **description = format!(
                "Page {} / {}{}\n{}\n{}",
                draft.page + 1,
                draft.pages(),
                if draft.truncated {
                    " (directory scan limited)"
                } else {
                    ""
                },
                tail(&draft.folder.to_string_lossy(), 180),
                tail(&session.status, 240)
            );
        }
        draft.action = None;
        session.file_dialog = Some(draft);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "theia-zone-browser-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path.canonicalize().unwrap())
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn draft(mode: Mode, folder: &std::path::Path, text: &str) -> Draft {
        Draft {
            mode,
            folder: folder.into(),
            text: text.into(),
            selection: [0; 2],
            page: 0,
            entries: vec![],
            overwrite: false,
            truncated: false,
            revision: 0,
            action: None,
        }
    }
    #[test]
    fn zone_paths_preserve_literal_names_and_append_only_missing_save_extension() {
        let root = Scratch::new();
        let mut d = draft(Mode::SaveAs, &root.0, "my zone");
        assert_eq!(d.path().unwrap(), root.0.join("my zone.zone.json"));
        d.text = "existing.JSON".into();
        assert_eq!(d.path().unwrap(), root.0.join("existing.JSON"));
        for value in ["", " ", "bad.glb", "line\nfile.json", "null\0file.json"] {
            d.text = value.into();
            assert!(d.path().is_err());
        }
        d.mode = Mode::Open;
        d.text = "literal name.json".into();
        assert_eq!(d.path().unwrap(), root.0.join("literal name.json"));
        d.text = ".".into();
        assert!(d.path().unwrap().is_dir());
    }
    #[test]
    fn browser_pages_filters_navigates_and_retains_save_filename_without_partial_refresh() {
        let root = Scratch::new();
        let nested = root.0.join("nested");
        std::fs::create_dir(&nested).unwrap();
        for index in 0..20 {
            std::fs::write(root.0.join(format!("zone-{index:02}.json")), b"{}").unwrap();
        }
        std::fs::write(root.0.join("ignore.txt"), b"not a zone").unwrap();
        let mut d = draft(Mode::SaveAs, &root.0, "copy.zone.json");
        d.refresh(root.0.clone()).unwrap();
        assert_eq!(d.entries.len(), 21);
        assert_eq!(d.pages(), 2);
        assert!(d.row(0).unwrap().directory);
        assert_eq!(d.row(1).unwrap().name, "zone-00.json");
        d.page = 1;
        assert_eq!(d.row(0).unwrap().name, "zone-11.json");
        let folder = d.folder.clone();
        assert!(d.refresh(root.0.join("missing")).is_err());
        assert_eq!(d.folder, folder);
        assert_eq!(d.page, 1);
        d.overwrite = true;
        d.browse(nested.clone()).unwrap();
        assert_eq!(d.text, "copy.zone.json");
        assert!(!d.overwrite);
        assert_eq!(d.page, 0);
        assert_eq!(d.pages(), 1);
        assert!(d.row(0).is_none());
        assert_eq!(d.path().unwrap(), nested.join("copy.zone.json"));
        d.mode = Mode::Open;
        d.browse(root.0.clone()).unwrap();
        assert!(d.text.is_empty());
    }
}

#[cfg(test)]
#[path = "document_tests.rs"]
mod document_tests;
