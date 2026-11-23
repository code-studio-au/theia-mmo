//! Standard Bevy backpack widgets. Contents and outcomes come from the authority;
//! selecting slots never changes the displayed bag optimistically.
pub(crate) mod context_menu;
pub(crate) mod ground;
mod lifecycle_qualification;
mod money;
mod qualification;
pub(crate) mod skin;
mod vendor_qualification;
mod zone_qualification;
use crate::network::Network;
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use theia_core::inventory::{Bag, EquipmentOperation, Operation};
use theia_network::{
    inventory_wire::{Outcome, Reason},
    renet_runtime::inventory::Profile,
};
#[derive(Resource)]
struct Config(Profile);
#[derive(Component, Clone, Copy)]
enum Widget {
    Toggle,
    ToggleEquipment,
    Slot(u8),
    Split,
    Destroy,
    Cancel,
    Equip,
    Unequip,
    Claim,
    Buy(u8),
}
#[derive(Component)]
pub(crate) struct SlotButton(pub u8);
#[derive(Component)]
pub(crate) struct SplitButton;
#[derive(Component)]
pub(crate) struct DestroyButton;
#[derive(Component)]
pub(crate) struct CancelButton;
#[derive(Component)]
struct DestroyLabel;
#[derive(Component)]
pub(crate) struct EquipButton;
#[derive(Component)]
pub(crate) struct UnequipButton;
#[derive(Component)]
pub(crate) struct ClaimButton;
#[derive(Component)]
struct RewardLabel;
#[derive(Component)]
pub(crate) struct BuyButton(pub u8);
#[derive(Component)]
struct OfferLabel(u8);
#[derive(Component)]
pub(crate) struct Pane;
#[derive(Component)]
struct EquipmentPane;
#[derive(Component)]
struct VendorPane;
#[derive(Component)]
struct Feedback;
#[derive(Component)]
pub(crate) struct DocumentActions;
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Phase {
    Setup,
    Present,
    Paint,
}
#[derive(Component)]
struct SlotLabel(u8);
#[derive(Component)]
struct StatusLabel;
#[derive(Component)]
struct SplitLabel;
#[derive(Component)]
struct EquipmentLabel;
#[derive(Resource, Default)]
struct Pending {
    toggle: bool,
    toggle_equipment: bool,
    context: Option<context_menu::Choice>,
    slot: Option<u8>,
    split: bool,
    destroy: bool,
    cancel: bool,
    equip: bool,
    unequip: bool,
    claim: bool,
    buy: Option<theia_core::vendors::Command>,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Ui {
    pub open: bool,
    pub equipment_open: bool,
    pub equipment_enabled: bool,
    #[serde(skip)]
    menu: Option<context_menu::Selection>,
    #[serde(skip)]
    menu_dragged: bool,
    pub destroy_confirmation: Option<(u8, u64)>,
    pub selected: Option<u8>,
    pub split: bool,
    pub unequip: bool,
    pub pending: bool,
    pub status: String,
    pub last_reply: u64,
    pub revision: Option<u64>,
    pub ready_frames: u64,
    pub visible_frames: u64,
    pub slot_clicks: u64,
    pub submitted: u64,
    pub applied: u64,
    pub replayed: u64,
    pub rejected: u64,
    pub ignored_pending_clicks: u64,
    pub equipment_clicks: u64,
    pub reward_clicks: u64,
    pub reward_pending: bool,
    pub vendor_pending: bool,
    pub document_pending: bool,
    pub vendor_open: bool,
    pub vendor_clicks: u64,
    #[serde(skip)]
    offers: Vec<theia_core::vendors::Command>,
    #[serde(skip)]
    pub bag: Option<Bag>,
}
impl Ui {
    pub(crate) fn panel_open(&self) -> bool {
        self.open || self.equipment_open || self.vendor_open
    }
}
fn activate(
    event: On<Activate>,
    widgets: Query<&Widget>,
    mut pending: ResMut<Pending>,
    ui: Res<Ui>,
    windows: Query<&Window>,
) {
    if let Ok(widget) = widgets.get(event.entity) {
        if !windows.single().is_ok_and(|window| window.focused) {
            *pending = Pending::default();
            return;
        }
        match widget {
            Widget::Toggle => pending.toggle = true,
            Widget::ToggleEquipment => pending.toggle_equipment = true,
            Widget::Slot(slot) => pending.slot = Some(*slot),
            Widget::Split => pending.split = true,
            Widget::Destroy => pending.destroy = true,
            Widget::Cancel => pending.cancel = true,
            Widget::Equip => pending.equip = true,
            Widget::Unequip => pending.unequip = true,
            Widget::Claim => pending.claim = true,
            Widget::Buy(index) => {
                if (ui.open || ui.vendor_open) && !ui.pending {
                    pending.buy = ui.offers.get(usize::from(*index)).copied();
                }
            }
        }
    }
}
pub(crate) fn install(app: &mut App, profile: Profile) {
    assert!(
        app.is_plugin_added::<bevy::ui::picking_backend::UiPickingPlugin>(),
        "Backpack requires Bevy UI picking"
    );
    crate::loot::install(app, profile.clone());
    ground::install(app);
    if let Some(documents) = profile.documents() {
        crate::documents::install(app, documents.clone());
    }
    app.insert_resource(Ui {
        equipment_enabled: profile.catalog().equipment_enabled(),
        ..default()
    })
    .insert_resource(Config(profile))
    .init_resource::<Pending>()
    .add_observer(activate)
    .add_systems(Update, collect.before(present))
    .add_systems(Update, close_panel.before(present))
    .add_systems(Startup, setup.in_set(Phase::Setup))
    .add_systems(
        Update,
        (
            present.in_set(Phase::Present),
            (
                paint,
                paint_coins_and_offers,
                money::paint,
                paint_buttons,
                skin::present,
            )
                .chain()
                .in_set(Phase::Paint),
            scroll,
        )
            .chain()
            .after(crate::network::present),
    );
    qualification::install(app);
    context_menu::install(app);
    vendor_qualification::install(app);
    zone_qualification::install(app);
    lifecycle_qualification::install(app);
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
fn button(widget: Widget, label: &str, width: f32, height: f32) -> impl Bundle {
    (
        Button,
        ActivateOnPress,
        Hovered::default(),
        widget,
        AccessibleLabel::new(label),
        Node {
            width: px(width),
            min_height: px(height),
            padding: UiRect::all(px(4)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.12, 0.16)),
    )
}
#[derive(Component)]
pub(crate) struct BackpackDock;
/// Shared real toggle widget, also rendered by the labelled developer fixture.
pub(crate) fn spawn_toggle(commands: &mut Commands, skinned: bool) {
    let width = if skinned { 140. } else { 340. };
    commands
        .spawn((
            BackpackDock,
            button(Widget::Toggle, "Open backpack", width, 30.),
        ))
        .insert(Node {
            position_type: PositionType::Absolute,
            display: if skinned {
                Display::None
            } else {
                Display::Flex
            },
            right: px(12),
            bottom: px(12),
            width: px(width),
            min_height: px(30),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_child(text("Backpack [I]", 16.));
}
const SLOT_SIZE: f32 = 52.;
const SLOT_GAP: f32 = 4.;
const BAG_WIDTH: f32 = 4. * SLOT_SIZE + 3. * SLOT_GAP + 24.;

fn spawn_slots(grid: &mut ChildSpawnerCommands, profile: &Profile, skinned: bool) {
    for slot in 0..16 {
        let height = if profile.vendor().is_some() {
            40.
        } else if profile.rewards().is_some() {
            52.
        } else if profile.catalog().equipment_enabled() {
            64.
        } else {
            72.
        };
        let mut entity = grid.spawn((
            SlotButton(slot),
            button(
                Widget::Slot(slot),
                &format!("Backpack slot {}", slot + 1),
                if skinned { SLOT_SIZE } else { 76. },
                if skinned { SLOT_SIZE } else { height },
            ),
        ));
        if skinned {
            entity.remove::<(Button, ActivateOnPress)>().insert((
                crate::ui_theme::Style::Slot,
                crate::ui_theme::slot::frame(Node {
                    width: px(SLOT_SIZE),
                    height: px(SLOT_SIZE),
                    flex_shrink: 0.,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                }),
            ));
        }
        entity.with_children(|button| {
            if skinned {
                skin::slot_children(button, slot);
            } else {
                button.spawn((SlotLabel(slot), text("Loading", 12.)));
            }
        });
    }
}
fn setup(mut commands: Commands, config: Res<Config>, skin: Option<Res<crate::fantasy_ui::Skin>>) {
    spawn_toggle(&mut commands, skin.is_some());
    if skin.is_some() {
        skin::spawn_tooltip(&mut commands);
    }
    commands
        .spawn((
            Pane,
            crate::fantasy_ui::Panel::Standard,
            Hovered::default(),
            ScrollPosition::default(),
            Node {
                position_type: PositionType::Absolute,
                right: px(324),
                bottom: px(52),
                width: px(if skin.is_some() { BAG_WIDTH } else { 340. }),
                max_height: percent(75),
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(px(12)),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.96)),
        ))
        .with_children(|panel| {
            if skin.is_some() {
                crate::fantasy_ui::window_header(
                    panel,
                    text("Inventory", 22.),
                    button(Widget::Toggle, "Close backpack", 28., 28.),
                    "Close backpack",
                );
            }

            if skin.is_none() && config.0.catalog().equipment_enabled() {
                panel.spawn((EquipmentLabel, text("Main hand: loading...", 14.)));
            }
            panel
                .spawn((Node {
                    width: percent(100),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(SLOT_GAP),
                    row_gap: px(SLOT_GAP),
                    ..default()
                },))
                .with_children(|grid| spawn_slots(grid, &config.0, skin.is_some()));
            if skin.is_none() {
                spawn_actions(panel, &config.0, "");
            }
            if config.0.catalog().currency_enabled() {
                if skin.is_none() {
                    panel.spawn(Node {
                        height: px(44),
                        flex_shrink: 0.,
                        ..default()
                    });
                }
                money::row(panel, None, skin.is_none());
            }
        });
    if skin.is_some() {
        if config.0.catalog().equipment_enabled() {
            commands
                .spawn((
                    EquipmentPane,
                    crate::ui_theme::Panel,
                    GlobalZIndex(22),
                    side_panel(236.),
                ))
                .with_children(|panel| {
                    crate::fantasy_ui::window_header(
                        panel,
                        text("Character", 22.),
                        button(Widget::ToggleEquipment, "Close character", 28., 28.),
                        "Close character",
                    );
                    skin::equipment_slot(panel, "Main hand");
                });
        }
        if let Some(vendor) = config.0.vendor() {
            commands
                .spawn((
                    VendorPane,
                    crate::ui_theme::Panel,
                    GlobalZIndex(22),
                    side_panel(306.),
                ))
                .with_children(|panel| {
                    crate::ui_theme::heading(panel, "Trading");
                    spawn_offers(panel, vendor.catalog());
                    money::row(panel, None, false);
                });
        }
        commands.spawn((
            Feedback,
            StatusLabel,
            text("", 14.),
            crate::fantasy_ui::hud_shadow(),
            GlobalZIndex(24),
            Node {
                position_type: PositionType::Absolute,
                right: px(324. + BAG_WIDTH + 12.),
                bottom: px(100),
                max_width: px(BAG_WIDTH),
                display: Display::None,
                ..default()
            },
        ));
    }
}
fn side_panel(width: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: px(12),
        bottom: px(64),
        width: px(width),
        max_height: percent(75),
        overflow: Overflow::scroll_y(),
        padding: UiRect::all(px(12)),
        display: Display::None,
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        ..default()
    }
}
fn spawn_actions(panel: &mut ChildSpawnerCommands, profile: &Profile, status: &str) {
    if profile.catalog().equipment_enabled() {
        panel
            .spawn(Node {
                width: percent(100),
                column_gap: px(6),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    EquipButton,
                    button(Widget::Equip, "Equip selected item", 150., 30.),
                ))
                .with_child(text("Equip selected", 14.));
                row.spawn((
                    UnequipButton,
                    button(Widget::Unequip, "Choose unequip destination", 150., 30.),
                ))
                .with_child(text("Unequip", 14.));
            });
    }
    panel
        .spawn((Node {
            width: percent(100),
            column_gap: px(6),
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                SplitButton,
                button(Widget::Split, "Split one item", 150., 30.),
            ))
            .with_child((SplitLabel, text("Split one", 14.)));
            row.spawn((
                CancelButton,
                button(Widget::Cancel, "Cancel item selection", 150., 30.),
            ))
            .with_child(text("Cancel", 14.));
        });
    if profile.catalog().equipment_enabled() {
        panel
            .spawn((
                DestroyButton,
                button(
                    Widget::Destroy,
                    "Destroy selected stack with confirmation",
                    306.,
                    30.,
                ),
            ))
            .with_child((DestroyLabel, text("Destroy selected stack", 14.)));
    }
    if profile.documents().is_some() {
        panel.spawn((
            DocumentActions,
            Node {
                width: percent(100),
                ..default()
            },
        ));
    }
    panel.spawn((
        StatusLabel,
        text(status, 14.),
        Node {
            width: percent(100),
            min_height: px(40),
            ..default()
        },
    ));
    if let Some(vendor) = profile.vendor() {
        spawn_offers(panel, vendor.catalog());
    }
    if profile.rewards().is_some() {
        panel
            .spawn((
                ClaimButton,
                button(Widget::Claim, "Claim available reward", 316., 30.),
            ))
            .with_child((RewardLabel, text("Loading rewards...", 14.)));
    }
}
fn spawn_offers(panel: &mut ChildSpawnerCommands, catalog: &theia_core::vendors::Catalog) {
    for (index, offer) in catalog.offers().enumerate() {
        panel
            .spawn((
                BuyButton(index as u8),
                button(
                    Widget::Buy(index as u8),
                    &format!(
                        "{} {}",
                        if offer.is_sale() { "Sell" } else { "Buy" },
                        offer.name()
                    ),
                    316.,
                    58.,
                ),
            ))
            .insert(Node {
                width: percent(100),
                min_height: px(64),
                flex_shrink: 0.,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(6)),
                ..default()
            })
            .with_children(|button| {
                button.spawn((
                    OfferLabel(index as u8),
                    text("Talk to a vendor to trade.", 13.),
                ));
                let price = if offer.is_sale() {
                    offer.grant()
                } else {
                    offer.cost()
                };
                if let theia_core::vendors::Amount::Coins { copper } = price {
                    money::row(button, Some(*copper), false);
                }
            });
    }
}
fn reason(reason: Reason) -> &'static str {
    match reason {
        Reason::Stale | Reason::Conflict => "Backpack changed; choose the item again.",
        Reason::SameSlot => "Choose a different slot.",
        Reason::EmptySource => "That slot is empty.",
        Reason::Full => "Not enough backpack space.",
        Reason::Occupied => "Choose an empty slot to split into.",
        Reason::Amount => "This stack cannot be split that way.",
        Reason::Exhausted => "Backpack changes are unavailable.",
        Reason::Busy => "Backpack busy; try again.",
        Reason::Slot => "That slot is unavailable.",
        Reason::EquipmentDisabled => "Equipment is unavailable.",
        Reason::NotEquippable => "That item cannot be equipped.",
        Reason::EmptyHand => "No main-hand item is equipped.",
    }
}
fn collect(keys: Res<ButtonInput<KeyCode>>, windows: Query<&Window>, mut pending: ResMut<Pending>) {
    if !windows.single().is_ok_and(|window| window.focused) {
        *pending = Pending::default();
        return;
    }
    pending.toggle |= keys.just_pressed(KeyCode::KeyI);
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn close_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui: ResMut<Ui>,
    loot: Option<Res<crate::loot::Ui>>,
    documents: Option<Res<crate::documents::Ui>>,
    dialogue: Option<Res<crate::dialogue::Ui>>,
    network: Option<Res<Network>>,
    (interface, book): (
        Option<Res<crate::fantasy_ui::settings::Ui>>,
        Option<Res<crate::combat::book::Ui>>,
    ),
    party: Option<Res<crate::party::Ui>>,
    windows: Query<&Window>,
) {
    let talking = dialogue
        .as_ref()
        .is_some_and(|ui| ui.open || ui.pending || ui.close_when_open)
        || network.as_ref().is_some_and(|network| {
            let state = network.state();
            state.error.is_none()
                && (state.dialogue_pending
                    || state
                        .dialogue
                        .is_some_and(|dialogue| dialogue.open.is_some()))
        });
    if windows.single().is_ok_and(|window| window.focused)
        && keys.just_pressed(KeyCode::Escape)
        && !crate::party::owns_escape(party.as_deref())
        && !crate::combat::book::owns_escape(book.as_deref())
        && !crate::fantasy_ui::settings::owns_escape(interface.as_deref())
        && !talking
        && !documents
            .as_ref()
            .is_some_and(|u| u.open || u.closed_this_frame)
        && !loot.as_ref().is_some_and(|u| u.open || u.closed_this_frame)
    {
        if ui.menu.take().is_some() || ui.split || ui.unequip {
            ui.selected = None;
            ui.split = false;
            ui.unequip = false;
            ui.destroy_confirmation = None;
            ui.status.clear();
        } else if ui.open {
            ui.open = false;
        } else {
            ui.equipment_open = false;
        }
    }
}

fn present(
    network: Res<Network>,
    windows: Query<&Window>,
    mut pending: ResMut<Pending>,
    mut ui: ResMut<Ui>,
    config: Res<Config>,
) {
    let input = std::mem::take(&mut *pending);
    let mut input = if windows.single().is_ok_and(|window| window.focused) {
        input
    } else {
        Pending::default()
    };
    if input.toggle {
        ui.open = !ui.open;
        ui.status.clear();
    }
    if input.toggle_equipment {
        ui.equipment_open = !ui.equipment_open;
    }
    let state = network.state();
    if !ui.open {
        ui.destroy_confirmation = None;
    }
    if !network.connected() {
        ui.destroy_confirmation = None;
        ui.menu = None;
        ui.vendor_open = false;
        ui.status.clear();
        return;
    }
    let trading = config.0.vendor().is_some_and(|vendor| {
        vendor.catalog().offers().any(|o| !o.is_sale())
            && state
                .dialogue
                .as_ref()
                .and_then(|v| v.open.as_ref())
                .is_some_and(|o| {
                    state.guide.as_ref().is_some_and(|p| {
                        p.find(o.guide).is_some()
                            && (p.contacts.is_empty()
                                || p.npc(o.guide) == Some(vendor.catalog().npc()))
                    })
                })
    });
    if trading && !ui.vendor_open {
        ui.open = true;
        ui.equipment_open = false;
    }
    ui.vendor_open = trading;
    let Some(view) = &state.inventory else {
        ui.status = "Loading backpack...".into();
        return;
    };
    if ui.bag.as_ref() != Some(&view.bag) {
        ui.destroy_confirmation = None;
        ui.menu = None;
        ui.bag = Some(view.bag.clone());
    }
    if ui.status == "Loading backpack..." {
        ui.status.clear();
    }
    ui.revision = Some(view.bag.revision());
    ui.ready_frames += 1;
    ui.visible_frames += u64::from(ui.open);
    if let Some(reply) = view.reply
        && reply.sequence > ui.last_reply
    {
        ui.last_reply = reply.sequence;
        ui.pending = false;
        ui.selected = None;
        ui.split = false;
        ui.unequip = false;
        ui.status = match reply.outcome {
            Outcome::Applied => {
                ui.applied += 1;
                if ui.document_pending {
                    "Document opened."
                } else if ui.vendor_pending {
                    "Trade saved."
                } else if ui.reward_pending {
                    "Reward claimed."
                } else {
                    "Backpack saved."
                }
                .into()
            }
            Outcome::Replayed => {
                ui.replayed += 1;
                if ui.document_pending {
                    "Document opened."
                } else if ui.vendor_pending {
                    "Trade saved."
                } else if ui.reward_pending {
                    "Reward claimed."
                } else {
                    "Backpack saved."
                }
                .into()
            }
            Outcome::Rejected(why) => {
                ui.rejected += 1;
                if ui.vendor_pending {
                    match why {
                        Reason::EmptySource => "Not enough coins or items for this trade.",
                        Reason::Conflict => "Talk to the vendor again and choose an offer.",
                        _ => reason(why),
                    }
                    .into()
                } else {
                    reason(why).into()
                }
            }
        };
        ui.reward_pending = false;
        ui.vendor_pending = false;
        ui.document_pending = false;
    }
    if ui.pending || state.inventory_pending {
        ui.pending = true;
        ui.status = if ui.document_pending {
            "Opening document..."
        } else {
            "Saving backpack..."
        }
        .into();
        ui.ignored_pending_clicks += u64::from(
            input.slot.is_some()
                || input.destroy
                || input.equip
                || input.unequip
                || input.claim
                || input.buy.is_some(),
        );
        return;
    }
    if let Some(choice) = input.context.take() {
        context_menu::apply(choice, &config, &view.bag, &mut ui, &mut input);
    }
    if !ui.open && !ui.vendor_open {
        return;
    }
    if input.slot.is_some()
        || input.cancel
        || input.split
        || input.equip
        || input.unequip
        || input.claim
        || input.buy.is_some()
    {
        ui.destroy_confirmation = None;
    }
    if input.destroy {
        if !config.0.catalog().equipment_enabled() {
            return;
        }
        let Some(from) = ui.selected else {
            ui.status = "Select a backpack stack to destroy.".into();
            return;
        };
        let Some(stack) = view.bag.slots()[usize::from(from)].as_ref() else {
            ui.destroy_confirmation = None;
            return;
        };
        let binding = (from, view.bag.revision());
        if ui.destroy_confirmation == Some(binding) {
            ui.destroy_confirmation = None;
            if network.request_destruction(binding.1, from, stack.count()) {
                ui.pending = true;
                ui.submitted += 1;
                ui.status = "Saving item destruction...".into();
            } else {
                ui.status = "Backpack busy; select the stack again.".into();
            }
        } else {
            ui.split = false;
            ui.unequip = false;
            ui.destroy_confirmation = Some(binding);
            let name = config
                .0
                .catalog()
                .definition(stack.item())
                .expect("Checked item")
                .name();
            ui.status = format!(
                "Destroy {name} x{}? Click Confirm destroy, or Cancel to keep it.",
                stack.count()
            );
        }
        return;
    }
    if let Some(command) = input.buy {
        ui.vendor_clicks += 1;
        if network.request_vendor(command) {
            ui.pending = true;
            ui.vendor_pending = true;
            ui.submitted += 1;
            ui.status = "Saving trade...".into();
        } else {
            ui.status = "Items changed or busy; choose the offer again.".into();
        }
        return;
    }
    if input.claim {
        ui.reward_clicks += 1;
        if let Some(reward) = state.rewards.as_ref().and_then(|s| {
            s.available.iter().find(|r| {
                r.corpse.is_none()
                    && config
                        .0
                        .rewards()
                        .and_then(|p| p.catalog().definition(&r.reward))
                        .is_none_or(|d| d.ground().is_none())
            })
        }) && network.request_reward(reward.claim)
        {
            ui.pending = true;
            ui.reward_pending = true;
            ui.selected = None;
            ui.split = false;
            ui.unequip = false;
            ui.submitted += 1;
            ui.status = "Saving reward...".into();
        } else {
            ui.status = "No reward is available to claim.".into();
        }
        return;
    }
    if input.cancel {
        ui.selected = None;
        ui.split = false;
        ui.unequip = false;
        ui.status = "Choose an item.".into();
    }
    if input.split {
        ui.unequip = false;
        if ui.selected.is_some() {
            ui.split = !ui.split;
            ui.status = if ui.split {
                "Choose an empty slot to split one."
            } else {
                "Choose a destination."
            }
            .into();
        } else {
            ui.status = "Select an item first.".into();
        }
    }
    if input.equip {
        ui.equipment_clicks += 1;
        if let Some(from) = ui.selected {
            if network.request_equipment(EquipmentOperation::Equip { from }) {
                ui.pending = true;
                ui.submitted += 1;
                ui.status = "Saving equipment...".into();
            } else {
                ui.status = "Equipment busy or unavailable.".into();
            }
        } else {
            ui.status = "Select a backpack item to equip.".into();
        }
        return;
    }
    if input.unequip {
        ui.equipment_clicks += 1;
        ui.selected = None;
        ui.split = false;
        ui.unequip = !ui.unequip;
        ui.status = if ui.unequip {
            "Choose an empty backpack slot for main hand."
        } else {
            "Choose an item."
        }
        .into();
    }
    if let Some(slot) = input.slot {
        ui.slot_clicks += 1;
        if ui.unequip {
            if network.request_equipment(EquipmentOperation::Unequip { to: slot }) {
                ui.pending = true;
                ui.submitted += 1;
                ui.status = "Saving equipment...".into();
            } else {
                ui.status = "Equipment busy or unavailable.".into();
            }
        } else if let Some(from) = ui.selected {
            if from == slot {
                ui.selected = None;
                ui.split = false;
                ui.status = "Choose an item.".into();
            } else {
                let operation = if ui.split {
                    Operation::Split {
                        from,
                        to: slot,
                        amount: 1,
                    }
                } else {
                    Operation::Move { from, to: slot }
                };
                if network.request_inventory(operation) {
                    ui.pending = true;
                    ui.submitted += 1;
                    ui.status = "Saving backpack...".into();
                } else {
                    ui.status = "Backpack busy; try again.".into();
                }
            }
        } else if view.bag.slots()[usize::from(slot)].is_some() {
            ui.selected = Some(slot);
            ui.status = "Choose a destination.".into();
        } else {
            ui.status = "That slot is empty.".into();
        }
    }
}
type Label<'a> = (
    &'a mut Text,
    Option<&'a SlotLabel>,
    Option<&'a StatusLabel>,
    Option<&'a SplitLabel>,
    Option<&'a DestroyLabel>,
    Option<&'a EquipmentLabel>,
    Option<&'a RewardLabel>,
    Option<&'a OfferLabel>,
);
type ButtonView<'a> = (
    Entity,
    &'a Widget,
    Has<Pressed>,
    Has<crate::ui_theme::Selected>,
    Option<&'a Hovered>,
    &'a mut BackgroundColor,
);
type InventoryPanels<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Has<Pane>,
        Has<VendorPane>,
        Has<EquipmentPane>,
    ),
    Or<(
        With<Pane>,
        With<VendorPane>,
        With<EquipmentPane>,
        With<Feedback>,
    )>,
>;
fn paint(
    config: Res<Config>,
    network: Res<Network>,
    mut ui: ResMut<Ui>,
    mut panels: InventoryPanels<'_, '_>,
    mut labels: Query<Label<'_>>,
) {
    let state = network.state();
    ui.offers.clear();
    if (ui.open || ui.vendor_open)
        && !ui.pending
        && network.connected()
        && let (Some(vendor), Some(bag), Some(dialogue)) =
            (config.0.vendor(), ui.bag.as_ref(), state.dialogue.as_ref())
        && let Some(open) = &dialogue.open
        && state.guide.as_ref().is_some_and(|g| {
            g.find(open.guide).is_some()
                && (g.contacts.is_empty() || g.npc(open.guide) == Some(vendor.catalog().npc()))
        })
    {
        let revision = bag.revision();
        ui.offers = vendor
            .catalog()
            .offers()
            .enumerate()
            .map(|(index, _)| theia_core::vendors::Command {
                expected_revision: revision,
                dialogue_revision: dialogue.revision,
                npc: open.guide,
                offer: index as u8,
            })
            .collect();
    }
    for (mut pane, bag, vendor, equipment) in &mut panels {
        let visible = if bag {
            ui.open
        } else if vendor {
            ui.vendor_open
        } else if equipment {
            ui.equipment_open && !ui.vendor_open
        } else {
            (ui.open || ui.equipment_open || ui.vendor_open)
                && !matches!(ui.status.as_str(), "" | "Loading backpack...")
        };
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if pane.display != display {
            pane.display = display;
        }
    }
    for (mut label, slot, status, split, destroy, equipment, reward, offer) in &mut labels {
        let value = if let Some(slot) = slot {
            if let Some(bag) = &ui.bag {
                bag.slots()[usize::from(slot.0)].as_ref().map_or_else(
                    || "Empty".into(),
                    |stack| {
                        let name = config
                            .0
                            .catalog()
                            .definition(stack.item())
                            .expect("Validated inventory item")
                            .name();
                        format!("{name}\nx{}", stack.count())
                    },
                )
            } else {
                "Loading".into()
            }
        } else if let Some(index) = offer {
            config
                .0
                .vendor()
                .and_then(|v| v.catalog().offers().nth(usize::from(index.0)))
                .map_or(String::new(), |offer| {
                    if ui.vendor_pending {
                        return "Saving trade...".into();
                    }
                    if ui.offers.get(usize::from(index.0)).is_none() {
                        return "Talk to a vendor to trade.".into();
                    }
                    if matches!(offer.cost(), theia_core::vendors::Amount::Coins { .. })
                        || offer.is_sale()
                    {
                        return format!(
                            "{} {}",
                            if offer.is_sale() { "Sell" } else { "Buy" },
                            offer.name()
                        );
                    }
                    format!(
                        "{} {}\n{} → {}",
                        if offer.is_sale() { "Sell" } else { "Buy" },
                        offer.name(),
                        offer.cost().label(config.0.catalog()),
                        offer.grant().label(config.0.catalog())
                    )
                })
        } else if reward.is_some() {
            if ui.reward_pending {
                "Saving reward...".into()
            } else {
                network.state().rewards.as_ref().map_or_else(
                    || "Loading rewards...".into(),
                    |state| {
                        state
                            .available
                            .iter()
                            .find(|r| {
                                r.corpse.is_none()
                                    && config
                                        .0
                                        .rewards()
                                        .and_then(|p| p.catalog().definition(&r.reward))
                                        .is_none_or(|d| d.ground().is_none())
                            })
                            .and_then(|r| config.0.rewards()?.catalog().definition(&r.reward))
                            .map_or_else(
                                || "No rewards available".into(),
                                |r| format!("Claim: {}", r.name()),
                            )
                    },
                )
            }
        } else if equipment.is_some() {
            ui.bag.as_ref().map_or_else(
                || "Main hand".into(),
                |bag| {
                    bag.main_hand()
                        .and_then(|s| config.0.catalog().definition(s.item()))
                        .map_or_else(
                            || "Main hand".into(),
                            |d| format!("Main hand: {}", d.name()),
                        )
                },
            )
        } else if status.is_some() {
            ui.status.clone()
        } else if destroy.is_some() {
            if ui.destroy_confirmation.is_some() {
                "Confirm destroy"
            } else {
                "Destroy selected stack"
            }
            .into()
        } else if split.is_some() {
            if ui.split {
                "[x] Split one"
            } else {
                "Split one"
            }
            .into()
        } else {
            continue;
        };
        if label.0 != value {
            label.0 = value;
        }
    }
}

fn paint_coins_and_offers(ui: Res<Ui>, mut offers: Query<&mut Node, With<BuyButton>>) {
    for mut node in &mut offers {
        node.display = if ui.vendor_open {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn paint_buttons(
    mut commands: Commands,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    ui: Res<Ui>,
    mut buttons: Query<ButtonView<'_>>,
) {
    for (entity, widget, pressed, marked, hovered, mut background) in &mut buttons {
        let selected = matches!(widget,Widget::Slot(slot) if Some(*slot)==ui.selected)
            || matches!(widget, Widget::Unequip) && ui.unequip;
        if skin.is_some() && matches!(widget, Widget::Slot(_)) {
            if selected != marked {
                let mut entity = commands.entity(entity);
                if selected {
                    entity.insert(crate::ui_theme::Selected);
                } else {
                    entity.remove::<crate::ui_theme::Selected>();
                }
            }
            continue;
        }
        let color = if ui.pending {
            Color::srgb(0.08, 0.09, 0.11)
        } else if matches!(widget, Widget::Destroy) && ui.destroy_confirmation.is_some() {
            Color::srgb(0.45, 0.08, 0.06)
        } else if selected {
            Color::srgb(0.12, 0.32, 0.4)
        } else if pressed || hovered.is_some_and(|h| h.0) {
            Color::srgb(0.14, 0.24, 0.3)
        } else {
            Color::srgb(0.08, 0.12, 0.16)
        };
        let color = if skin.is_some() {
            if !ui.pending && matches!(widget, Widget::Destroy) && ui.destroy_confirmation.is_some()
            {
                Color::srgb(0.45, 0.08, 0.06)
            } else if selected && !ui.pending {
                Color::srgb(0.30, 0.22, 0.10)
            } else {
                crate::fantasy_ui::button_color(ui.pending, hovered.is_some_and(|h| h.0), pressed)
            }
        } else {
            color
        };
        if background.0 != color {
            background.0 = color;
        }
    }
}

fn scroll(
    mut events: MessageReader<bevy::input::mouse::MouseWheel>,
    ui: Res<Ui>,
    documents: Option<Res<crate::documents::Ui>>,
    windows: Query<&Window>,
    scale: Res<bevy::ui::UiScale>,
    mut panes: Query<(&Hovered, &mut ScrollPosition, &ComputedNode, &mut Node), With<Pane>>,
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
    let height = windows.single().map_or(720., |w| w.height()) / scale.0;
    for (hovered, mut position, node, mut style) in &mut panes {
        style.max_height = px((height - 212.).max(240.));
        if ui.open && !documents.as_ref().is_some_and(|u| u.open) {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            position.y = (position.y + if hovered.0 { delta } else { 0. }).clamp(0., maximum);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unfocused_inventory_actions_drop_without_cancelling_existing_wait_state() {
        let mut app = App::new();
        app.init_resource::<Pending>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Ui {
                open: true,
                pending: true,
                selected: Some(2),
                ..default()
            })
            .add_observer(activate)
            .add_systems(Update, (collect, close_panel).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                focused: false,
                ..default()
            })
            .id();
        let destroy = app.world_mut().spawn(Widget::Destroy).id();
        app.world_mut().trigger(Activate { entity: destroy });
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyI);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(
            !app.world().resource::<Pending>().destroy && !app.world().resource::<Pending>().toggle
        );
        let ui = app.world().resource::<Ui>();
        assert!(ui.open && ui.pending);
        assert_eq!(ui.selected, Some(2));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.update();
        assert!(
            !app.world().resource::<Pending>().toggle,
            "Held I does not reopen after refocus"
        );
        let equip = app.world_mut().spawn(Widget::Equip).id();
        app.world_mut().trigger(Activate { entity: equip });
        assert!(
            app.world().resource::<Pending>().equip,
            "Fresh focused widgets still collect"
        );
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.update();
        assert!(
            !app.world().resource::<Pending>().equip,
            "Focus loss before consumption drops a queued widget"
        );
        assert!(
            app.world().resource::<Ui>().pending,
            "Existing transaction wait state is not cancelled"
        );
    }
    #[test]
    fn backpack_keyboard_requires_a_fresh_edge_in_one_focused_window() {
        for count in [0, 1, 2] {
            let mut app = App::new();
            app.init_resource::<Pending>()
                .init_resource::<ButtonInput<KeyCode>>()
                .add_systems(Update, collect);
            for _ in 0..count {
                app.world_mut().spawn(Window {
                    focused: true,
                    ..default()
                });
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyI);
            app.update();
            assert_eq!(app.world().resource::<Pending>().toggle, count == 1);
            *app.world_mut().resource_mut::<Pending>() = Pending::default();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            app.update();
            assert!(!app.world().resource::<Pending>().toggle);
        }
    }
    #[test]
    fn header_close_queues_only_the_existing_toggle_while_a_save_is_pending() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.insert_resource(Ui {
            open: true,
            pending: true,
            ..default()
        })
        .init_resource::<Pending>()
        .add_observer(activate);
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn(Node::default()).with_children(|panel| {
                    crate::fantasy_ui::window_header(
                        panel,
                        text("Inventory", 22.),
                        button(Widget::Toggle, "Close backpack", 28., 28.),
                        "Close backpack",
                    );
                });
            })
            .unwrap();
        let world = app.world_mut();
        let close = world
            .query_filtered::<Entity, With<crate::fantasy_ui::HeaderClose>>()
            .single(world)
            .unwrap();
        world.trigger(Activate { entity: close });
        let pending = world.resource::<Pending>();
        assert!(pending.toggle);
        assert!(!pending.destroy && !pending.equip && !pending.claim && pending.slot.is_none());
        assert!(
            world.resource::<Ui>().pending,
            "Closing a panel must not cancel the saving state"
        );
        assert_eq!(world.resource::<Ui>().submitted, 0);
    }

    use super::*;
    #[test]
    fn releasing_over_another_widget_cannot_suppress_the_next_backpack_activation() {
        use bevy::picking::{
            backend::HitData,
            events::{Pointer, PointerPress, PointerRelease},
            pointer::{Location, PointerButton, PointerId},
        };
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<Ui>()
            .init_resource::<Pending>()
            .add_plugins(bevy::ui_widgets::ButtonPlugin)
            .add_observer(activate)
            .add_systems(PostUpdate, crate::ui_buttons::clear_released_presses);
        let window = app.world_mut().spawn_empty().id();
        let button = app
            .world_mut()
            .spawn((Button, ActivateOnPress, Widget::Destroy))
            .id();
        let elsewhere = app.world_mut().spawn_empty().id();
        let pointer = Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                    .normalize(Some(window))
                    .unwrap(),
                position: Vec2::ZERO,
            },
        );
        let press = PointerPress {
            entity: button,
            pointer: pointer.clone(),
            button: PointerButton::Primary,
            hit: HitData::new(window, 0., None, None),
            count: 1,
        };
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut().trigger(press.clone());
        app.update();
        assert!(
            app.world().get::<Pressed>(button).is_some(),
            "A held press must be retained"
        );
        assert!(app.world().resource::<Pending>().destroy);
        app.world_mut().resource_mut::<Pending>().destroy = false;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        // Panel reflow puts another widget under the released pointer. This is
        // the actual Bevy event shape observed during native item destruction.
        app.world_mut().trigger(PointerRelease {
            entity: elsewhere,
            pointer,
            button: PointerButton::Primary,
            hit: HitData::new(window, 0., None, None),
        });
        app.update();
        assert!(app.world().get::<Pressed>(button).is_none());
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut().trigger(press);
        app.update();
        assert!(
            app.world().resource::<Pending>().destroy,
            "The next click must activate immediately"
        );
    }
    #[test]
    fn purchase_activation_keeps_displayed_bag_session_npc_and_offer_binding() {
        use theia_core::identity::{EntityId, EntityKey, WorldEpoch};
        let old = theia_core::vendors::Command {
            expected_revision: 4,
            dialogue_revision: 2,
            npc: EntityKey {
                epoch: WorldEpoch::new(1).unwrap(),
                entity: EntityId::new(9).unwrap(),
            },
            offer: 0,
        };
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.insert_resource(Ui {
            open: true,
            offers: vec![old],
            ..default()
        })
        .init_resource::<Pending>()
        .add_observer(activate);
        let buy = app.world_mut().spawn(Widget::Buy(0)).id();
        app.world_mut().trigger(Activate { entity: buy });
        app.world_mut().resource_mut::<Ui>().offers = vec![theia_core::vendors::Command {
            expected_revision: 5,
            dialogue_revision: 3,
            ..old
        }];
        assert_eq!(app.world().resource::<Pending>().buy, Some(old));
        app.world_mut().resource_mut::<Pending>().buy = None;
        app.world_mut().resource_mut::<Ui>().pending = true;
        app.world_mut().trigger(Activate { entity: buy });
        assert!(app.world().resource::<Pending>().buy.is_none());
        app.world_mut().resource_mut::<Ui>().pending = false;
        app.world_mut().resource_mut::<Ui>().open = false;
        app.world_mut().trigger(Activate { entity: buy });
        assert!(app.world().resource::<Pending>().buy.is_none());
    }
}
