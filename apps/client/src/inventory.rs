//! Standard Bevy backpack widgets. Contents and outcomes come from the authority;
//! selecting slots never changes the displayed bag optimistically.
mod qualification;
mod vendor_qualification;
mod zone_qualification;
use crate::network::Network;
use bevy::{
    picking::hover::Hovered,
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
    Slot(u8),
    Split,
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
struct Pane;
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
    slot: Option<u8>,
    split: bool,
    cancel: bool,
    equip: bool,
    unequip: bool,
    claim: bool,
    buy: Option<theia_core::vendors::Command>,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Ui {
    pub open: bool,
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
    pub vendor_open: bool,
    pub vendor_clicks: u64,
    #[serde(skip)]
    offers: Vec<theia_core::vendors::Command>,
    #[serde(skip)]
    pub bag: Option<Bag>,
}
fn activate(
    event: On<Activate>,
    widgets: Query<&Widget>,
    mut pending: ResMut<Pending>,
    ui: Res<Ui>,
) {
    if let Ok(widget) = widgets.get(event.entity) {
        match widget {
            Widget::Toggle => pending.toggle = true,
            Widget::Slot(slot) => pending.slot = Some(*slot),
            Widget::Split => pending.split = true,
            Widget::Cancel => pending.cancel = true,
            Widget::Equip => pending.equip = true,
            Widget::Unequip => pending.unequip = true,
            Widget::Claim => pending.claim = true,
            Widget::Buy(index) => {
                if ui.open && !ui.pending {
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
    app.insert_resource(Config(profile))
        .init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (present, paint, scroll)
                .chain()
                .after(crate::network::present),
        );
    qualification::install(app);
    vendor_qualification::install(app);
    zone_qualification::install(app);
}
fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
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
fn setup(mut commands: Commands, config: Res<Config>) {
    commands
        .spawn(button(Widget::Toggle, "Open backpack", 340., 30.))
        .insert(Node {
            position_type: PositionType::Absolute,
            right: px(12),
            top: px(124),
            width: px(340),
            min_height: px(30),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_child(text("Backpack [I]", 16.));
    commands
        .spawn((
            Pane,
            ScrollPosition::default(),
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(162),
                width: px(340),
                max_height: if config.0.vendor().is_some() {
                    percent(75)
                } else {
                    Val::Auto
                },
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
            if config.0.catalog().equipment_enabled() {
                panel.spawn((EquipmentLabel, text("Main hand: loading...", 14.)));
            }
            panel
                .spawn((Node {
                    width: percent(100),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(4),
                    row_gap: px(4),
                    ..default()
                },))
                .with_children(|grid| {
                    for slot in 0..16 {
                        grid.spawn((
                            SlotButton(slot),
                            button(
                                Widget::Slot(slot),
                                &format!("Backpack slot {}", slot + 1),
                                76.,
                                if config.0.vendor().is_some() {
                                    40.
                                } else if config.0.rewards().is_some() {
                                    52.
                                } else if config.0.catalog().equipment_enabled() {
                                    64.
                                } else {
                                    72.
                                },
                            ),
                        ))
                        .with_child((SlotLabel(slot), text("Loading", 12.)));
                    }
                });
            if config.0.catalog().equipment_enabled() {
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
                    row.spawn(button(Widget::Cancel, "Cancel item selection", 150., 30.))
                        .with_child(text("Cancel", 14.));
                });
            panel.spawn((
                StatusLabel,
                text("Loading backpack...", 14.),
                Node {
                    width: percent(100),
                    min_height: px(40),
                    ..default()
                },
            ));
            if let Some(vendor) = config.0.vendor() {
                for (index, offer) in vendor.catalog().offers().enumerate() {
                    panel
                        .spawn((
                            BuyButton(index as u8),
                            button(
                                Widget::Buy(index as u8),
                                &format!("Buy {}", offer.name()),
                                316.,
                                58.,
                            ),
                        ))
                        .with_child((
                            OfferLabel(index as u8),
                            text("Talk to the guide to trade.", 13.),
                        ));
                }
            }
            if config.0.rewards().is_some() {
                panel
                    .spawn((
                        ClaimButton,
                        button(Widget::Claim, "Claim available reward", 316., 30.),
                    ))
                    .with_child((RewardLabel, text("Loading rewards...", 14.)));
            }
        });
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
fn present(
    network: Res<Network>,
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<Pending>,
    mut ui: ResMut<Ui>,
    config: Res<Config>,
) {
    let input = std::mem::take(&mut *pending);
    if input.toggle || keys.just_pressed(KeyCode::KeyI) {
        ui.open = !ui.open;
    }
    let state = network.state();
    if !network.connected() {
        ui.status = "Disconnected / loading".into();
        return;
    }
    let trading =
        config.0.vendor().is_some() && state.dialogue.as_ref().is_some_and(|v| v.open.is_some());
    if trading && !ui.vendor_open {
        ui.open = true;
    }
    ui.vendor_open = trading;
    let Some(view) = &state.inventory else {
        ui.status = "Loading backpack...".into();
        return;
    };
    if ui.bag.as_ref() != Some(&view.bag) {
        ui.bag = Some(view.bag.clone());
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
                if ui.vendor_pending {
                    "Purchase saved."
                } else if ui.reward_pending {
                    "Reward claimed."
                } else {
                    "Backpack saved."
                }
                .into()
            }
            Outcome::Replayed => {
                ui.replayed += 1;
                if ui.vendor_pending {
                    "Purchase saved."
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
                        Reason::EmptySource => "Not enough trade items.",
                        Reason::Conflict => "Talk to the guide again and choose an offer.",
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
    }
    if ui.pending || state.inventory_pending {
        ui.pending = true;
        ui.status = "Saving backpack...".into();
        ui.ignored_pending_clicks += u64::from(
            input.slot.is_some()
                || input.equip
                || input.unequip
                || input.claim
                || input.buy.is_some(),
        );
        return;
    }
    if !ui.open {
        return;
    }
    if let Some(command) = input.buy {
        ui.vendor_clicks += 1;
        if network.request_vendor(command) {
            ui.pending = true;
            ui.vendor_pending = true;
            ui.submitted += 1;
            ui.status = "Saving purchase...".into();
        } else {
            ui.status = "Items changed or busy; choose the offer again.".into();
        }
        return;
    }
    if input.claim {
        ui.reward_clicks += 1;
        if let Some(reward) = state.rewards.as_ref().and_then(|s| s.available.first())
            && network.request_reward(reward.claim)
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
            ui.status = "Choose a destination, or Split one.".into();
        } else {
            ui.status = "That slot is empty.".into();
        }
    } else if ui.status.is_empty() {
        ui.status = "Choose an item.".into();
    }
}
type Label<'a> = (
    &'a mut Text,
    Option<&'a SlotLabel>,
    Option<&'a StatusLabel>,
    Option<&'a SplitLabel>,
    Option<&'a EquipmentLabel>,
    Option<&'a RewardLabel>,
    Option<&'a OfferLabel>,
);
type ButtonView<'a> = (
    &'a Widget,
    Has<Pressed>,
    Option<&'a Hovered>,
    &'a mut BackgroundColor,
);
fn paint(
    config: Res<Config>,
    network: Res<Network>,
    mut ui: ResMut<Ui>,
    mut panels: Query<&mut Node, With<Pane>>,
    mut labels: Query<Label<'_>>,
    mut buttons: Query<ButtonView<'_>>,
) {
    let state = network.state();
    ui.offers.clear();
    if ui.open
        && !ui.pending
        && network.connected()
        && let (Some(vendor), Some(bag), Some(dialogue)) =
            (config.0.vendor(), ui.bag.as_ref(), state.dialogue.as_ref())
        && let Some(open) = &dialogue.open
        && state
            .guide
            .as_ref()
            .and_then(|g| g.guide)
            .is_some_and(|g| g.key == open.guide)
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
    for mut pane in &mut panels {
        let display = if ui.open {
            Display::Flex
        } else {
            Display::None
        };
        if pane.display != display {
            pane.display = display;
        }
    }
    for (mut label, slot, status, split, equipment, reward, offer) in &mut labels {
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
                        return "Saving purchase...".into();
                    }
                    if ui.offers.get(usize::from(index.0)).is_none() {
                        return "Talk to the guide to trade.".into();
                    }
                    let name = |id| {
                        config
                            .0
                            .catalog()
                            .definition(id)
                            .expect("Checked offer item")
                            .name()
                    };
                    format!(
                        "{}
{} {} for {} {}",
                        offer.name(),
                        offer.cost().count(),
                        name(offer.cost().item()),
                        offer.grant().count(),
                        name(offer.grant().item())
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
                            .first()
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
                || "Main hand: loading...".into(),
                |bag| {
                    let name = bag
                        .main_hand()
                        .and_then(|s| config.0.catalog().definition(s.item()))
                        .map_or("Empty", |d| d.name());
                    format!("Main hand: {name}")
                },
            )
        } else if status.is_some() {
            ui.status.clone()
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
    for (widget, pressed, hovered, mut background) in &mut buttons {
        let selected = matches!(widget,Widget::Slot(slot) if Some(*slot)==ui.selected)
            || matches!(widget, Widget::Unequip) && ui.unequip;
        let color = if ui.pending {
            Color::srgb(0.08, 0.09, 0.11)
        } else if selected {
            Color::srgb(0.12, 0.32, 0.4)
        } else if pressed || hovered.is_some_and(|h| h.0) {
            Color::srgb(0.14, 0.24, 0.3)
        } else {
            Color::srgb(0.08, 0.12, 0.16)
        };
        if background.0 != color {
            background.0 = color;
        }
    }
}

fn scroll(
    mut events: MessageReader<bevy::input::mouse::MouseWheel>,
    ui: Res<Ui>,
    mut panes: Query<(&mut ScrollPosition, &ComputedNode), With<Pane>>,
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
        for (mut position, node) in &mut panes {
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
