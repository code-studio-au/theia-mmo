//! Native composer: draft ownership is separate from authoritative chat history.
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::{FocusCause, InputFocus, InputFocusSystems},
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::EditableText,
    ui::InteractionDisabled,
    ui_widgets::{Activate, Button, TextInput},
};
use std::{collections::HashSet, time::Instant};
use theia_core::chat::{Channel, MAX_TEXT_BYTES, Text as ChatText, TextError};
use theia_network::{chat_client::Compose, chat_wire::Outcome};

#[derive(Component)]
struct Input;
#[derive(Component)]
struct Status;
#[derive(Component)]
struct ChannelMenu;
#[derive(Component)]
struct Placeholder;
#[derive(Component, Clone, Copy)]
enum Action {
    Channel,
    Choose(Channel),
}
impl Action {
    fn name(self) -> &'static str {
        match self {
            Self::Channel => "channel",
            Self::Choose(Channel::Say) => "channel_say",
            Self::Choose(Channel::Area) => "channel_area",
        }
    }
}
#[derive(Resource)]
pub(crate) struct Ui {
    pub(crate) typing: bool,
    channel: Channel,
    channel_menu: bool,
    submit: bool,
    queued: Option<(u64, Instant)>,
    status: &'static str,
    held: HashSet<KeyCode>,
}
impl Default for Ui {
    fn default() -> Self {
        Self {
            typing: false,
            channel: Channel::Say,
            channel_menu: false,
            submit: false,
            queued: None,
            status: "",
            held: default(),
        }
    }
}
impl Ui {
    pub(crate) fn active(&self) -> bool {
        self.typing || self.channel_menu
    }
}
pub(crate) fn install(app: &mut App) {
    app.init_resource::<Ui>()
        .add_observer(activate)
        .add_systems(
            PreUpdate,
            keyboard
                .after(bevy::input::InputSystems)
                .after(crate::loading::shield_keyboard)
                .before(InputFocusSystems::Dispatch)
                .before(bevy::picking::PickingSystems::ProcessInput)
                .before(crate::fantasy_ui::settings::keyboard)
                .before(crate::party::keyboard)
                .before(crate::combat::book::keyboard)
                .before(crate::camera::mode::input_policy),
        )
        // Text edits are applied in PostUpdate. Submit the complete draft,
        // including the final character if it arrived with Enter this frame.
        .add_systems(Last, (finish, paint).chain());
}
fn label(value: &str) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(14.),
        TextColor(Color::WHITE),
        Pickable::IGNORE,
    )
}
pub(crate) fn row(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        })
        .with_children(|column| {
            column
                .spawn(Node {
                    width: percent(100),
                    column_gap: px(5),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Action::Channel,
                        Button,
                        bevy::input_focus::tab_navigation::TabIndex(0),
                        Hovered::default(),
                        crate::ui_theme::Style::Secondary,
                        AccessibleLabel::new("Choose chat channel"),
                        Node {
                            width: px(37),
                            height: px(37),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_child((
                        crate::ui_theme::Icon(29),
                        Node {
                            width: px(24),
                            height: px(24),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    channel_menu(row);
                    row.spawn((
                        Input,
                        TextInput,
                        bevy::input_focus::tab_navigation::TabIndex(0),
                        EditableText {
                            max_characters: Some(MAX_TEXT_BYTES),
                            ..default()
                        },
                        AccessibleLabel::new("Chat message"),
                        Node {
                            // Allocate remaining row space independently of
                            // draft length; the text widget scrolls internally.
                            width: px(0),
                            flex_basis: px(0),
                            flex_grow: 1.,
                            ..crate::ui_theme::input_box(false, false)
                        },
                        BackgroundColor(crate::ui_theme::PANEL),
                        BorderColor::all(crate::ui_theme::GOLD),
                        TextFont::from_font_size(16.),
                        TextColor(Color::WHITE),
                        TextLayout::no_wrap(),
                    ))
                    .with_child((
                        Placeholder,
                        label("Message…"),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(14),
                            top: px(12),
                            ..default()
                        },
                    ));
                });
            column.spawn((
                Status,
                label(""),
                Node {
                    display: Display::None,
                    flex_shrink: 0.,
                    ..default()
                },
            ));
        });
}
fn channel_menu(row: &mut ChildSpawnerCommands) {
    row.spawn((
        ChannelMenu,
        crate::ui_theme::Panel,
        GlobalZIndex(72),
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(50),
            width: px(132),
            padding: UiRect::all(px(7)),
            row_gap: px(4),
            flex_direction: FlexDirection::Column,
            display: Display::None,
            ..default()
        },
        BackgroundColor(crate::ui_theme::PANEL),
    ))
    .with_children(|menu| {
        for (channel, title) in [(Channel::Say, "Say"), (Channel::Area, "Area")] {
            menu.spawn((
                Action::Choose(channel),
                Button,
                Hovered::default(),
                bevy::input_focus::tab_navigation::TabIndex(0),
                crate::ui_theme::Style::Secondary,
                AccessibleLabel::new(title),
                Node {
                    min_height: px(30),
                    padding: UiRect::horizontal(px(8)),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_child(label(title));
        }
    });
}
fn available(network: Option<&crate::network::Network>) -> bool {
    network.is_some_and(|n| !n.loading_blocked && n.connected() && n.state().chat.is_some())
}
fn discard(input: &mut EditableText, ui: &mut Ui, focus: &mut InputFocus, entity: Entity) {
    input.pending_edits.clear();
    input.pending_paste = None;
    input.editor.set_text("");
    ui.typing = false;
    ui.channel_menu = false;
    ui.submit = false;
    if focus.get() == Some(entity) {
        focus.clear();
    }
}
/// Latched keys cannot resume movement after closing chat until released.
fn shield(keys: &mut ButtonInput<KeyCode>, held: &mut HashSet<KeyCode>, typing: bool) {
    if typing {
        held.extend(keys.get_pressed().copied());
        let modifiers: Vec<_> = [
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::AltLeft,
            KeyCode::AltRight,
        ]
        .into_iter()
        .filter(|key| keys.pressed(*key))
        .collect();
        keys.reset_all();
        for key in modifiers {
            keys.press(key);
        }
    } else {
        for key in held.iter() {
            keys.reset(*key);
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn keyboard(
    mut ui: ResMut<Ui>,
    mut focus: ResMut<InputFocus>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut events: MessageReader<KeyboardInput>,
    windows: Query<&Window>,
    mut inputs: Query<(Entity, &mut EditableText), With<Input>>,
    actions: Query<(), With<Action>>,
    network: Option<Res<crate::network::Network>>,
    settings: Option<Res<crate::fantasy_ui::settings::Ui>>,
) {
    for event in events.read() {
        if event.state == ButtonState::Released {
            ui.held.remove(&event.key_code);
        }
    }
    let Ok((entity, mut input)) = inputs.single_mut() else {
        return;
    };
    let owned = focus.get() == Some(entity);
    let was_typing = ui.active() || owned;
    let allowed = available(network.as_deref())
        && windows.single().is_ok_and(|w| w.focused)
        && !settings.is_some_and(|s| s.open);
    if ui.channel_menu {
        if !allowed || !focus.get().is_some_and(|e| actions.contains(e)) {
            ui.channel_menu = false;
        } else if keys.just_pressed(KeyCode::Escape) {
            ui.channel_menu = false;
            keys.reset(KeyCode::Escape);
            if ui.typing {
                focus.set(entity, FocusCause::Navigated);
            } else {
                focus.clear();
            }
        }
    }
    if !allowed || (ui.typing && !owned && !focus.get().is_some_and(|e| actions.contains(e))) {
        discard(&mut input, &mut ui, &mut focus, entity);
    } else if owned {
        ui.typing = true;
        if keys.just_pressed(KeyCode::Escape) {
            discard(&mut input, &mut ui, &mut focus, entity);
        } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            ui.submit = true;
        }
    } else if allowed && focus.get().is_none() && keys.just_pressed(KeyCode::Enter) {
        focus.set(entity, FocusCause::Navigated);
        ui.typing = true;
    }
    let typing = was_typing || ui.active();
    shield(&mut keys, &mut ui.held, typing);
}
fn activate(
    event: On<Activate>,
    actions: Query<&Action>,
    inputs: Query<Entity, With<Input>>,
    mut ui: ResMut<Ui>,
    mut focus: ResMut<InputFocus>,
    network: Option<Res<crate::network::Network>>,
    windows: Query<&Window>,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    if !available(network.as_deref()) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Ok(input) = inputs.single() else {
        return;
    };
    match action {
        Action::Channel => {
            ui.channel_menu = !ui.channel_menu;
            if ui.channel_menu {
                focus.set(event.entity, FocusCause::Navigated);
                return;
            }
            if !ui.typing {
                focus.clear();
                return;
            }
        }
        Action::Choose(channel) if ui.channel_menu => ui.channel = *channel,
        Action::Choose(_) => return,
    }
    ui.channel_menu = false;
    focus.set(input, FocusCause::Navigated);
    ui.typing = true;
}
fn validation(value: &str) -> Result<ChatText, &'static str> {
    ChatText::parse(value).map_err(|error| match error {
        TextError::Length if value.is_empty() => "Enter a message",
        TextError::Length => "Message exceeds 256 bytes",
        TextError::Empty => "Enter a message",
        TextError::Control => "Use a single line without control characters",
    })
}
fn outcome(value: Outcome) -> &'static str {
    match value {
        Outcome::Accepted => "",
        Outcome::RateLimited => "Too many messages — please wait",
        Outcome::Muted => "You cannot send messages right now",
        Outcome::NoArea => "Area chat is unavailable here",
        Outcome::Unavailable => "Chat is unavailable right now",
        Outcome::Sequence | Outcome::Time => "Message refused — please try again",
    }
}
fn finish(
    mut ui: ResMut<Ui>,
    mut focus: ResMut<InputFocus>,
    mut inputs: Query<(Entity, &mut EditableText), With<Input>>,
    network: Option<Res<crate::network::Network>>,
    windows: Query<&Window>,
) {
    let Ok((entity, mut input)) = inputs.single_mut() else {
        return;
    };
    if !available(network.as_deref()) {
        discard(&mut input, &mut ui, &mut focus, entity);
        ui.queued = None;
        ui.status = "Chat unavailable";
        return;
    }
    if !windows.single().is_ok_and(|w| w.focused) {
        // Losing input focus cancels only the draft. The connection and any
        // already submitted message still receive authoritative acknowledgements.
        discard(&mut input, &mut ui, &mut focus, entity);
    }
    if ui.status == "Chat unavailable" {
        ui.status = "";
    }
    let network = network.unwrap();
    let state = network.state();
    let view = state.chat.as_ref().unwrap();
    if let Some((sequence, at)) = ui.queued {
        if view.submitted > sequence {
            ui.queued = None;
        } else if at.elapsed().as_secs() >= 6 {
            // No resend; acceptance is unknown. The worker also has a five-second timeout.
            ui.queued = None;
            ui.status = "No reply — delivery unknown";
        }
    }
    if view.pending || ui.queued.is_some() {
        ui.status = "Sending…";
    } else if view.timed_out {
        ui.status = "No reply — delivery unknown";
    } else if view.exhausted {
        ui.status = "Chat unavailable for this session";
    } else if let Some(reply) = view.inbox.reply().filter(|r| r.sequence == view.submitted) {
        // Preserve local validation until a new explicit submission.
        if ui.status == "Sending…" {
            ui.status = outcome(reply.outcome);
        }
    }
    if !std::mem::take(&mut ui.submit) {
        return;
    }
    if ui.queued.is_some() || view.pending || view.exhausted {
        return;
    }
    let message = match validation(&input.editor.text().to_string()) {
        Ok(message) => message,
        Err(status) => {
            ui.status = status;
            return;
        }
    };
    if network.chat_action(Compose {
        channel: ui.channel,
        text: message,
    }) {
        ui.queued = Some((view.submitted, Instant::now()));
        ui.status = "Sending…";
        discard(&mut input, &mut ui, &mut focus, entity);
    } else {
        ui.status = "Chat busy — please try again";
    }
}
#[allow(clippy::too_many_arguments)]
fn paint(
    mut commands: Commands,
    ui: Res<Ui>,
    network: Option<Res<crate::network::Network>>,
    inputs: Query<(Entity, &EditableText), With<Input>>,
    actions: Query<(Entity, &Action, Has<crate::ui_theme::Selected>)>,
    mut status: Query<(&mut Text, &mut Node), With<Status>>,
    mut menu: Query<&mut Node, (With<ChannelMenu>, Without<Status>)>,
    mut placeholder: Query<&mut Visibility, With<Placeholder>>,
) {
    let enabled = available(network.as_deref());
    for entity in inputs
        .iter()
        .map(|(e, _)| e)
        .chain(actions.iter().map(|(e, _, _)| e))
    {
        if enabled {
            commands.entity(entity).remove::<InteractionDisabled>();
        } else {
            commands.entity(entity).insert(InteractionDisabled);
        }
    }
    for (mut text, mut node) in &mut status {
        if text.0 != ui.status {
            text.0 = ui.status.into();
        }
        let display = if ui.status.is_empty() {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != display {
            node.display = display;
        }
    }
    let menu_display = if enabled && ui.channel_menu {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut menu {
        if node.display != menu_display {
            node.display = menu_display;
        }
    }
    for (entity, action, marked) in &actions {
        let selected = matches!(action, Action::Choose(channel) if *channel == ui.channel);
        if selected != marked {
            if selected {
                commands.entity(entity).insert(crate::ui_theme::Selected);
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::ui_theme::Selected>();
            }
        }
    }
    let empty = inputs
        .single()
        .is_ok_and(|(_, input)| input.editor.text().to_string().is_empty());
    for mut visible in &mut placeholder {
        *visible = if empty && !ui.typing {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Opt-in native qualification reports state/lengths, never draft or message bodies.
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let Some(ui) = world.get_resource::<Ui>() else {
        return serde_json::Value::Null;
    };
    let mut result = serde_json::json!({"typing":ui.typing,"channel_menu":ui.channel_menu,"channel":format!("{:?}",ui.channel),"status":ui.status,"queued":ui.queued.is_some()});
    if let Some(view) = world
        .get_resource::<crate::network::Network>()
        .and_then(|n| n.state().chat)
    {
        result["submitted"] = view.submitted.into();
        result["pending"] = view.pending.into();
        result["history"] = view.inbox.history().map(|d| serde_json::json!({"serial":d.serial,"channel":format!("{:?}",d.channel),"sender":d.name.as_str(),"bytes":d.text.as_str().len()})).collect::<Vec<_>>().into();
        result["reply"] = view
            .inbox
            .reply()
            .map(|r| format!("{:?}", r.outcome))
            .into();
    }
    result["draft_bytes"] = world
        .query_filtered::<&EditableText, With<Input>>()
        .single(world)
        .map_or(0, |i| i.editor.text().to_string().len())
        .into();
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let widgets: Vec<_> = world.query::<(Option<&Input>,Option<&Action>,&ComputedNode,&bevy::ui::UiGlobalTransform)>().iter(world)
        .filter(|(input,action,node,_)| (input.is_some()||action.is_some()) && node.size().min_element()>0.)
        .map(|(input,action,node,transform)|serde_json::json!({"action":if input.is_some(){"input"}else{action.unwrap().name()},"point":(transform.affine().translation * crate::fantasy_ui::window_factor(node, scale)).to_array(),"size":node.size().to_array()})).collect();
    result["widgets"] = widgets.into();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draft_validation_preserves_unicode_and_rejects_bytes_controls_and_blank() {
        assert_eq!(validation("你好 <Say>").unwrap().as_str(), "你好 <Say>");
        for value in [
            "",
            "  ",
            "\u{200b}",
            "one\ntwo",
            "x\u{202e}",
            &"é".repeat(129),
        ] {
            assert!(validation(value).is_err());
        }
        assert!(validation(&"é".repeat(128)).is_ok());
    }
    #[test]
    fn typing_consumes_actions_and_latches_held_movement_until_release() {
        let mut keys = ButtonInput::default();
        let mut held = HashSet::new();
        for key in [
            KeyCode::KeyW,
            KeyCode::Digit1,
            KeyCode::F10,
            KeyCode::Escape,
            KeyCode::ShiftLeft,
        ] {
            keys.press(key);
        }
        shield(&mut keys, &mut held, true);
        assert!(!keys.pressed(KeyCode::KeyW));
        assert!(!keys.just_pressed(KeyCode::Digit1));
        assert!(!keys.just_pressed(KeyCode::F10));
        assert!(!keys.just_pressed(KeyCode::Escape));
        assert!(keys.pressed(KeyCode::ShiftLeft));
        keys.press(KeyCode::KeyW); // OS repeat while closing cannot resume motion.
        shield(&mut keys, &mut held, false);
        assert!(!keys.pressed(KeyCode::KeyW));
        held.remove(&KeyCode::KeyW); // Actual key-up, then a new key-down.
        keys.press(KeyCode::KeyW);
        shield(&mut keys, &mut held, false);
        assert!(keys.just_pressed(KeyCode::KeyW));
    }
}
