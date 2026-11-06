//! Standard Bevy item reader. Only a matching server answer opens exact local
//! content; selecting/hovering an item and changing pages grant no read credit.
use crate::{inventory, network::Network};
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use std::num::NonZeroU128;
use theia_core::{documents::OpenCommand, inventory::Bag};
use theia_network::{document_wire::Completion, documents::Profile, inventory_wire::Outcome};
#[derive(Resource)]
struct Config(Profile);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Ui {
    pub open: bool,
    pub closed_this_frame: bool,
    pub page: usize,
    pub title: String,
    pub status: String,
    pub read_clicks: u64,
    pub submitted: u64,
    pub approved: u64,
    pub rejected: u64,
    pub page_turns: u64,
    pub ignored_pending_clicks: u64,
    #[serde(skip)]
    waiting: Option<OpenCommand>,
    #[serde(skip)]
    document: Option<u8>,
}
impl Ui {
    fn close(&mut self) {
        self.closed_this_frame |= self.open || self.waiting.is_some();
        self.open = false;
        self.waiting = None;
        self.document = None;
        self.page = 0;
    }
    fn answer(&mut self, profile: &Profile, answer: Completion) -> bool {
        if self.waiting != Some(answer.command) || answer.profile_hash != profile.hash() {
            return false;
        }
        self.waiting = None;
        match answer.outcome {
            Outcome::Applied | Outcome::Replayed => {
                let Ok(document) = profile.resolve(answer.command) else {
                    return false;
                };
                self.open = true;
                self.document = Some(answer.command.document);
                self.page = 0;
                self.title = document.title().into();
                self.approved += 1;
                self.status = "Document opened.".into();
            }
            Outcome::Rejected(_) => {
                self.rejected += 1;
                self.status =
                    "Unable to open this item. Select the current stack and try again.".into();
            }
        }
        true
    }
}
#[derive(Resource, Default)]
struct Pending {
    read: Option<OpenCommand>,
    next: bool,
    previous: bool,
    close: bool,
}
#[derive(Component, Clone, Copy)]
enum Action {
    Read,
    Next,
    Previous,
    Close,
}
#[derive(Component)]
pub(crate) struct ReadButton;
#[derive(Component)]
pub(crate) struct NextButton;
#[derive(Component)]
pub(crate) struct PreviousButton;
#[derive(Component)]
pub(crate) struct CloseButton;
#[derive(Component)]
pub(crate) struct Pane;
#[derive(Component)]
pub(crate) struct BodyScroll;
#[derive(Component)]
pub(crate) struct BodyText;
#[derive(Component)]
enum Label {
    Title,
    Body,
    Page,
}
pub(crate) fn install(app: &mut App, profile: Profile) {
    app.insert_resource(Config(profile))
        .init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate)
        .add_systems(Startup, setup.after(inventory::Phase::Setup))
        .add_systems(
            Update,
            close_panel
                .before(crate::loot::close_panel)
                .before(crate::input),
        )
        .add_systems(
            Update,
            (present, paint, scroll)
                .chain()
                .after(inventory::Phase::Present)
                .before(inventory::Phase::Paint),
        );
}
fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        Pickable::IGNORE,
    )
}
fn button(action: Action, label: &str) -> impl Bundle {
    (
        action,
        Button,
        ActivateOnPress,
        Hovered::default(),
        Pickable::IGNORE,
        Node {
            min_height: px(32),
            padding: UiRect::all(px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.2, 0.25)),
        Name::new(label.to_string()),
        AccessibleLabel::new(label),
    )
}
fn setup(
    mut commands: Commands,
    panes: Query<Entity, With<inventory::DocumentActions>>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
) {
    for pane in &panes {
        commands.entity(pane).with_children(|parent| {
            parent
                .spawn((ReadButton, button(Action::Read, "Read selected item")))
                .insert(Node {
                    display: Display::None,
                    width: percent(100),
                    min_height: px(32),
                    padding: UiRect::all(px(6)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_child(text("Read selected item", 14.));
        });
    }
    commands
        .spawn((
            Pane,
            if skin.is_some() {
                crate::fantasy_ui::Panel::Paper
            } else {
                crate::fantasy_ui::Panel::Standard
            },
            Hovered::default(),
            GlobalZIndex(60),
            Node {
                position_type: PositionType::Absolute,
                left: if skin.is_some() { px(12) } else { percent(8) },
                top: if skin.is_some() { px(64) } else { percent(12) },
                width: px(if skin.is_some() { 420. } else { 560. }),
                max_width: percent(if skin.is_some() { 45. } else { 65. }),
                height: percent(if skin.is_some() { 45. } else { 68. }),
                max_height: if skin.is_some() {
                    px(crate::fantasy_ui::PAPER_MAX_HEIGHT)
                } else {
                    Val::Auto
                },
                padding: UiRect::all(px(if skin.is_some() { 14. } else { 16. })),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: px(if skin.is_some() { 10. } else { 12. }),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.04, 0.06, 0.99)),
        ))
        .with_children(|panel| {
            if skin.is_some() {
                crate::fantasy_ui::window_header(
                    panel,
                    (Label::Title, text("", 20.)),
                    button(Action::Close, "Close document"),
                    "Close document",
                );
            } else {
                panel.spawn((Label::Title, text("", 20.)));
            }
            if skin.is_some() {
                panel
                    .spawn((
                        crate::fantasy_ui::DarkSurface,
                        Node {
                            width: percent(100),
                            padding: UiRect::all(px(4)),
                            column_gap: px(8),
                            align_items: AlignItems::Center,
                            flex_shrink: 0.,
                            ..default()
                        },
                        BackgroundColor(crate::fantasy_ui::PANEL),
                    ))
                    .with_children(|row| {
                        row.spawn((PreviousButton, button(Action::Previous, "Previous page")))
                            .with_child(text("Prev", 14.));
                        row.spawn(Node {
                            flex_grow: 1.,
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_child((Label::Page, text("", 14.)));
                        row.spawn((NextButton, button(Action::Next, "Next page")))
                            .with_child(text("Next", 14.));
                    });
            }
            panel
                .spawn((
                    BodyScroll,
                    Hovered::default(),
                    ScrollPosition::default(),
                    Node {
                        flex_grow: 1.,
                        min_height: px(0),
                        width: percent(100),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .with_child((
                    Label::Body,
                    BodyText,
                    text("", 16.),
                    Node {
                        width: percent(100),
                        ..default()
                    },
                ));
            if skin.is_none() {
                panel.spawn((Label::Page, text("", 14.)));
            }
            panel
                .spawn(Node {
                    width: percent(100),
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|row| {
                    if skin.is_none() {
                        row.spawn((PreviousButton, button(Action::Previous, "Previous page")))
                            .with_child(text("Previous", 14.));
                        row.spawn((NextButton, button(Action::Next, "Next page")))
                            .with_child(text("Next", 14.));
                    }
                    row.spawn((CloseButton, button(Action::Close, "Close document")))
                        .with_child(text("Close [Esc]", 14.));
                });
        });
}
/// Actual reader widgets with explicit local fixture state only. No network,
/// open command, completion answer, observer or read credit is installed.
pub(crate) fn install_preview(app: &mut App) {
    let items = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-document-items.json"
    ))
    .unwrap();
    let profile = Profile::from_json(
        include_bytes!("../../../content/quests/development-documents.json"),
        &items,
    )
    .unwrap();
    app.insert_resource(Config(profile))
        .insert_resource(Ui {
            open: true,
            document: Some(0),
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, paint);
}
fn selected(profile: &Profile, bag: &Bag, slot: u8) -> Option<u8> {
    let stack = bag.slots().get(usize::from(slot))?.as_ref()?;
    let document = profile.catalog().for_item(stack.item())?;
    profile
        .catalog()
        .documents()
        .position(|d| d.id() == document.id())
        .and_then(|i| u8::try_from(i).ok())
}
fn activate(
    event: On<Activate>,
    actions: Query<&Action>,
    mut pending: ResMut<Pending>,
    mut ui: ResMut<Ui>,
    inventory: Res<inventory::Ui>,
    network: Res<Network>,
    config: Res<Config>,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    match action {
        Action::Read => {
            if !inventory.open
                || inventory.pending
                || ui.waiting.is_some()
                || pending.read.is_some()
            {
                ui.ignored_pending_clicks += 1;
                return;
            }
            let state = network.state();
            let Some(view) = state
                .inventory
                .as_ref()
                .filter(|_| network.connected() && !state.inventory_pending)
            else {
                return;
            };
            let Some(bag) = inventory.bag.as_ref().filter(|b| **b == view.bag) else {
                return;
            };
            let Some(slot) = inventory.selected else {
                return;
            };
            let Some(document) = selected(&config.0, bag, slot) else {
                return;
            };
            let Some(nonce) = ui.read_clicks.checked_add(1) else {
                return;
            };
            ui.read_clicks = nonce;
            let mut bytes = b"theia-document-ui-intent-v1\0".to_vec();
            bytes.extend(config.0.hash());
            bytes.extend(view.actor.epoch.value().to_le_bytes());
            bytes.extend(view.actor.entity.value().to_le_bytes());
            bytes.extend(nonce.to_le_bytes());
            let hash = theia_network::terrain::sha256(&bytes);
            pending.read = Some(OpenCommand {
                request_id: NonZeroU128::new(
                    u128::from_le_bytes(hash[..16].try_into().unwrap()).max(1),
                )
                .unwrap(),
                expected_revision: bag.revision(),
                slot,
                document,
            });
        }
        Action::Close => pending.close = true,
        Action::Next => pending.next = true,
        Action::Previous => pending.previous = true,
    }
}
pub(crate) fn close_panel(keys: Res<ButtonInput<KeyCode>>, mut ui: ResMut<Ui>) {
    ui.closed_this_frame = false;
    if keys.just_pressed(KeyCode::Escape) && (ui.open || ui.waiting.is_some()) {
        ui.close();
    }
}
fn present(
    config: Res<Config>,
    network: Res<Network>,
    mut inventory: ResMut<inventory::Ui>,
    mut pending: ResMut<Pending>,
    mut ui: ResMut<Ui>,
    mut scrolls: Query<&mut ScrollPosition, With<BodyScroll>>,
) {
    let input = std::mem::take(&mut *pending);
    if !network.connected() || !inventory.open {
        ui.close();
        return;
    }
    if input.close {
        ui.close();
    }
    if let Some(answer) = network.state().document_reply
        && ui.answer(&config.0, answer)
    {
        for mut scroll in &mut scrolls {
            scroll.y = 0.;
        }
    }
    if let Some(command) = input.read
        && !inventory.pending
        && ui.waiting.is_none()
        && network.request_document(command)
    {
        ui.waiting = Some(command);
        ui.open = false;
        ui.submitted += 1;
        ui.status = "Opening document...".into();
        inventory.pending = true;
        inventory.document_pending = true;
        inventory.status = ui.status.clone();
    }
    if ui.open
        && let Some(document) = ui
            .document
            .and_then(|i| config.0.catalog().documents().nth(usize::from(i)))
    {
        let previous = ui.page;
        if input.next {
            ui.page = (ui.page + 1).min(document.pages().len() - 1);
        }
        if input.previous {
            ui.page = ui.page.saturating_sub(1);
        }
        if previous != ui.page {
            ui.page_turns += 1;
            for mut scroll in &mut scrolls {
                scroll.y = 0.;
            }
        }
    }
}
type PanelQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static Pane>,
        Option<&'static ReadButton>,
    ),
    Or<(With<Pane>, With<ReadButton>)>,
>;
fn paint(
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    config: Res<Config>,
    ui: Res<Ui>,
    inventory: Res<inventory::Ui>,
    mut nodes: PanelQuery<'_, '_>,
    mut labels: Query<(&mut Text, &Label)>,
    mut buttons: Query<(
        &Action,
        Has<Pressed>,
        &Hovered,
        &mut BackgroundColor,
        &mut Pickable,
    )>,
) {
    let document = ui
        .document
        .and_then(|i| config.0.catalog().documents().nth(usize::from(i)));
    for (mut node, pane, read) in &mut nodes {
        let visible = if pane.is_some() {
            ui.open
        } else {
            read.is_some()
                && inventory.open
                && !inventory.pending
                && ui.waiting.is_none()
                && inventory.selected.is_some_and(|slot| {
                    inventory
                        .bag
                        .as_ref()
                        .is_some_and(|bag| selected(&config.0, bag, slot).is_some())
                })
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (mut value, label) in &mut labels {
        let text = match (label, document) {
            (Label::Title, Some(d)) => d.title().to_string(),
            (Label::Body, Some(d)) => d.pages().get(ui.page).cloned().unwrap_or_default(),
            (Label::Page, Some(d)) => format!("Page {} / {}", ui.page + 1, d.pages().len()),
            _ => String::new(),
        };
        if value.0 != text {
            value.0 = text;
        }
    }
    for (action, pressed, hovered, mut color, mut pickable) in &mut buttons {
        // Hidden controls must not retain a hit target while layout changes.
        let visible = match action {
            Action::Read => {
                inventory.open
                    && !inventory.pending
                    && ui.waiting.is_none()
                    && inventory.selected.is_some_and(|slot| {
                        inventory
                            .bag
                            .as_ref()
                            .is_some_and(|bag| selected(&config.0, bag, slot).is_some())
                    })
            }
            _ => ui.open,
        };
        *pickable = if visible {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
        let disabled = match action {
            Action::Read => inventory.pending || ui.waiting.is_some(),
            Action::Previous => ui.page == 0,
            Action::Next => document.is_none_or(|d| ui.page + 1 >= d.pages().len()),
            Action::Close => false,
        };
        color.0 = if skin.is_some() {
            crate::fantasy_ui::button_color(disabled, hovered.0, pressed)
        } else if disabled {
            Color::srgb(0.07, 0.08, 0.1)
        } else if pressed || hovered.0 {
            Color::srgb(0.15, 0.3, 0.36)
        } else {
            Color::srgb(0.08, 0.2, 0.25)
        };
    }
}
fn scroll(
    mut events: MessageReader<bevy::input::mouse::MouseWheel>,
    ui: Res<Ui>,
    mut panes: Query<(&Hovered, &mut ScrollPosition, &ComputedNode), With<BodyScroll>>,
) {
    let delta: f32 = events
        .read()
        .map(|e| {
            -e.y * if e.unit == bevy::input::mouse::MouseScrollUnit::Line {
                24.
            } else {
                1.
            }
        })
        .sum();
    if ui.open && delta != 0. {
        for (hovered, mut position, node) in &mut panes {
            if !hovered.0 {
                continue;
            }
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            position.y = (position.y + delta).clamp(0., maximum);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windowless_reader_fixture_paints_without_authority_or_read_credit() {
        let mut app = App::new();
        app.insert_resource(crate::fantasy_ui::Skin::fixture())
            .init_resource::<inventory::Ui>();
        install_preview(&mut app);
        app.update();
        let ui = app.world().resource::<Ui>();
        assert!(ui.open && ui.document.is_some());
        assert_eq!((ui.approved, ui.submitted, ui.read_clicks), (0, 0, 0));
        assert!(ui.waiting.is_none());
        assert!(!app.world().contains_resource::<Network>());
        assert!(!app.world().contains_resource::<Pending>());
    }
    fn profile() -> Profile {
        let items = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-document-items.json"
        ))
        .unwrap();
        Profile::from_json(
            include_bytes!("../../../content/quests/development-documents.json"),
            &items,
        )
        .unwrap()
    }
    fn command() -> OpenCommand {
        OpenCommand {
            request_id: NonZeroU128::new(1).unwrap(),
            expected_revision: 3,
            slot: 5,
            document: 0,
        }
    }
    #[test]
    fn reader_requires_its_original_answer_and_exact_text_profile_and_never_reopens_after_close() {
        let profile = profile();
        let mut ui = Ui {
            waiting: Some(command()),
            ..default()
        };
        let answer = Completion {
            command: command(),
            profile_hash: profile.hash(),
            outcome: Outcome::Applied,
        };
        let mut foreign = answer;
        foreign.profile_hash[0] ^= 1;
        assert!(!ui.answer(&profile, foreign));
        assert!(!ui.open);
        let mut changed = answer;
        changed.command.slot = 6;
        assert!(!ui.answer(&profile, changed));
        assert!(!ui.open);
        assert!(ui.answer(&profile, answer));
        assert!(ui.open);
        assert_eq!(ui.title, "Roadside Orders");
        assert_eq!(ui.page, 0);
        assert_eq!(ui.approved, 1);
        ui.close();
        assert!(!ui.answer(&profile, answer));
        assert!(!ui.open);
        assert_eq!(ui.approved, 1);
    }
    #[test]
    fn cancelled_pending_read_and_matching_refusal_never_display_note_text() {
        let profile = profile();
        let answer = Completion {
            command: command(),
            profile_hash: profile.hash(),
            outcome: Outcome::Replayed,
        };
        let mut ui = Ui {
            waiting: Some(command()),
            ..default()
        };
        ui.close();
        assert!(!ui.answer(&profile, answer));
        assert!(!ui.open);
        ui.waiting = Some(command());
        assert!(ui.answer(
            &profile,
            Completion {
                outcome: Outcome::Rejected(theia_network::inventory_wire::Reason::EmptySource),
                ..answer
            }
        ));
        assert!(!ui.open);
        assert_eq!(ui.rejected, 1);
        assert_eq!(ui.approved, 0);
    }
    #[test]
    fn escape_closes_reader_before_loot_or_backpack() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Ui>()
            .init_resource::<inventory::Ui>()
            .init_resource::<crate::loot::Ui>()
            .add_systems(
                Update,
                (
                    close_panel,
                    crate::loot::close_panel,
                    inventory::close_panel,
                )
                    .chain(),
            );
        app.world_mut().resource_mut::<Ui>().open = true;
        app.world_mut().resource_mut::<inventory::Ui>().open = true;
        app.world_mut().resource_mut::<crate::loot::Ui>().open = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(!app.world().resource::<Ui>().open);
        assert!(app.world().resource::<Ui>().closed_this_frame);
        assert!(app.world().resource::<inventory::Ui>().open);
        assert!(app.world().resource::<crate::loot::Ui>().open);
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::Escape);
        keys.clear();
        keys.press(KeyCode::Escape);
        app.update();
        assert!(!app.world().resource::<crate::loot::Ui>().open);
        assert!(app.world().resource::<inventory::Ui>().open);
    }
}
