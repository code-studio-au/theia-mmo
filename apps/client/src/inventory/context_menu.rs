//! Item actions are bound to the bag revision and slot shown when the menu opened.
use super::*;
use bevy::picking::{
    events::{PointerClick, PointerDrag, PointerPress},
    pointer::{PointerButton, PointerId},
};
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::FocusedInput,
};

#[derive(Clone, Copy, PartialEq)]
enum Source {
    Bag(u8),
    MainHand,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Selection {
    source: Source,
    revision: u64,
    anchor: Vec2,
}
impl Selection {
    fn item(self, bag: &Bag) -> Option<&theia_core::inventory::Stack> {
        if bag.revision() != self.revision {
            return None;
        }
        match self.source {
            Source::Bag(slot) => bag.slots().get(usize::from(slot))?.as_ref(),
            Source::MainHand => bag.main_hand(),
        }
    }

    fn visible(self, ui: &Ui) -> bool {
        match self.source {
            Source::Bag(_) => ui.open,
            Source::MainHand => ui.equipment_open,
        }
    }

    fn confirmation(self, ui: &Ui) -> bool {
        matches!(self.source, Source::Bag(slot) if ui.destroy_confirmation == Some((slot, self.revision)))
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Action {
    Move,
    Equip,
    Unequip,
    Split,
    Read,
    Destroy,
    ConfirmDestroy,
    Cancel,
}
impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::Move => "Move",
            Self::Equip => "Equip",
            Self::Unequip => "Unequip",
            Self::Split => "Split one",
            Self::Read => "Read",
            Self::Destroy => "Destroy stack…",
            Self::ConfirmDestroy => "Confirm destroy",
            Self::Cancel => "Cancel",
        }
    }
}
#[derive(Component, Clone, Copy)]
pub(crate) struct Choice {
    selection: Selection,
    action: Action,
}
impl Choice {
    fn current(self, ui: &Ui, bag: &Bag) -> bool {
        !ui.pending
            && ui.menu == Some(self.selection)
            && self.selection.visible(ui)
            && self.selection.item(bag).is_some()
    }

    pub(crate) fn document_slot(self, ui: &Ui, bag: &Bag) -> Option<u8> {
        match self.selection.source {
            Source::Bag(slot) if self.action == Action::Read && self.current(ui, bag) => Some(slot),
            _ => None,
        }
    }
}
#[derive(Component)]
struct Root;
#[derive(Component)]
pub(super) struct MainHand;

pub(super) fn install(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, paint.after(Phase::Present))
        .add_observer(pressed)
        .add_observer(dragged)
        .add_observer(clicked)
        .add_observer(keyboard)
        .add_observer(activate_choice);
}
fn setup(mut commands: Commands, skin: Option<Res<crate::fantasy_ui::Skin>>) {
    if skin.is_some() {
        commands.spawn((
            Root,
            GlobalZIndex(88),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                display: Display::None,
                ..default()
            },
        ));
    }
}
fn pressed(event: On<PointerPress>, mut ui: ResMut<Ui>) {
    if event.pointer.id == PointerId::Mouse
        && matches!(
            event.button,
            PointerButton::Primary | PointerButton::Secondary
        )
    {
        ui.menu_dragged = false;
    }
}
fn dragged(event: On<PointerDrag>, mut ui: ResMut<Ui>) {
    if event.pointer.id == PointerId::Mouse
        && matches!(
            event.button,
            PointerButton::Primary | PointerButton::Secondary
        )
        && event.distance.length() > 6.
    {
        ui.menu_dragged = true;
    }
}
type SlotAnchor<'a> = (
    Option<&'a SlotButton>,
    Has<MainHand>,
    &'a ComputedNode,
    &'a bevy::ui::UiGlobalTransform,
);
type MenuTargets<'w, 's> = (
    Query<'w, 's, (), With<Root>>,
    Query<'w, 's, SlotAnchor<'static>>,
    Query<'w, 's, (), With<Choice>>,
);
fn clicked(
    mut event: On<PointerClick>,
    (roots, slots, choices): MenuTargets<'_, '_>,
    windows: Query<&Window>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    mut ui: ResMut<Ui>,
    mut commands: Commands,
) {
    if skin.is_none()
        || event.pointer.id != PointerId::Mouse
        || !windows.single().is_ok_and(|window| window.focused)
    {
        return;
    }
    let target = event.original_event_target();
    if roots.contains(target) {
        ui.menu = None;
        ui.destroy_confirmation = None;
        event.propagate(false);
        return;
    }
    if ui.menu_dragged || ui.pending {
        return;
    }
    if event.button == PointerButton::Primary {
        if choices.contains(target)
            || (ui.open
                && slots
                    .get(target)
                    .is_ok_and(|(slot, _, _, _)| slot.is_some()))
        {
            event.propagate(false);
            commands.trigger(Activate { entity: target });
        }
        return;
    }
    if event.button != PointerButton::Secondary {
        return;
    }
    let Ok((bag_slot, main_hand, node, transform)) = slots.get(target) else {
        return;
    };
    let source = if let Some(slot) = bag_slot {
        Source::Bag(slot.0)
    } else if main_hand {
        Source::MainHand
    } else {
        return;
    };
    event.propagate(false);
    let anchor = (transform.affine().translation + Vec2::new(node.size().x, -node.size().y) * 0.5)
        * node.inverse_scale_factor()
        + Vec2::new(6., 0.);
    ui.destroy_confirmation = None;
    ui.menu = ui.bag.as_ref().and_then(|bag| {
        let selection = Selection {
            source,
            revision: bag.revision(),
            anchor,
        };
        (anchor.is_finite() && selection.visible(&ui) && selection.item(bag).is_some())
            .then_some(selection)
    });
}
type ItemControls = Or<(With<SlotButton>, With<Choice>)>;
fn keyboard(
    mut event: On<FocusedInput<KeyboardInput>>,
    controls: Query<(), ItemControls>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    mut commands: Commands,
) {
    let key = &event.input;
    if skin.is_some()
        && controls.contains(event.focused_entity)
        && !key.repeat
        && key.state == ButtonState::Pressed
        && matches!(key.key_code, KeyCode::Enter | KeyCode::Space)
    {
        event.propagate(false);
        commands.trigger(Activate {
            entity: event.focused_entity,
        });
    }
}
fn activate_choice(
    event: On<Activate>,
    choices: Query<&Choice>,
    ui: Res<Ui>,
    mut pending: ResMut<Pending>,
) {
    if let Ok(choice) = choices.get(event.entity)
        && ui.bag.as_ref().is_some_and(|bag| choice.current(&ui, bag))
    {
        pending.context = Some(*choice);
    }
}
fn options(config: &Config, selection: Selection, ui: &Ui, bag: &Bag) -> Vec<Action> {
    let Some(stack) = selection.item(bag) else {
        return vec![];
    };
    if selection.confirmation(ui) {
        return vec![Action::ConfirmDestroy, Action::Cancel];
    }
    let mut actions = Vec::new();
    match selection.source {
        Source::MainHand => actions.push(Action::Unequip),
        Source::Bag(_) => {
            actions.push(Action::Move);
            if config
                .0
                .catalog()
                .definition(stack.item())
                .is_some_and(|item| item.main_hand_appearance().is_some())
            {
                actions.push(Action::Equip);
            }
            if stack.count() > 1 && bag.slots().iter().any(Option::is_none) {
                actions.push(Action::Split);
            }
            if config
                .0
                .documents()
                .is_some_and(|docs| docs.catalog().for_item(stack.item()).is_some())
            {
                actions.push(Action::Read);
            }
            if config.0.catalog().equipment_enabled() {
                actions.push(Action::Destroy);
            }
        }
    }
    actions.push(Action::Cancel);
    actions
}
pub(super) fn apply(choice: Choice, config: &Config, bag: &Bag, ui: &mut Ui, input: &mut Pending) {
    if !choice.current(ui, bag)
        || !options(config, choice.selection, ui, bag).contains(&choice.action)
    {
        ui.menu = None;
        ui.destroy_confirmation = None;
        return;
    }
    if choice.action != Action::Destroy {
        ui.menu = None;
    }
    if matches!(choice.action, Action::Read | Action::Cancel) {
        input.cancel = choice.action == Action::Cancel;
        return;
    }
    ui.open = true;
    ui.split = false;
    ui.unequip = false;
    ui.selected = match choice.selection.source {
        Source::Bag(slot) => Some(slot),
        Source::MainHand => None,
    };
    match choice.action {
        Action::Move => ui.status = "Choose a destination.".into(),
        Action::Equip => input.equip = true,
        Action::Unequip => input.unequip = true,
        Action::Split => input.split = true,
        Action::Destroy | Action::ConfirmDestroy => input.destroy = true,
        Action::Read | Action::Cancel => {}
    }
}

fn paint(
    mut commands: Commands,
    config: Res<Config>,
    mut ui: ResMut<Ui>,
    windows: Query<&Window>,
    scale: Res<bevy::ui::UiScale>,
    mut roots: Query<(Entity, &mut Node), With<Root>>,
    mut painted: Local<Option<(Selection, bool, Vec2)>>,
) {
    let window = windows.single().ok().filter(|w| w.focused);
    let next = ui.menu.and_then(|selection| {
        let window = window?;
        let bag = ui.bag.as_ref()?;
        (selection.visible(&ui) && !ui.pending && selection.item(bag).is_some()).then_some((
            selection,
            selection.confirmation(&ui),
            Vec2::new(window.width(), window.height()) / scale.0,
        ))
    });
    if ui.menu.is_some() && next.is_none() {
        ui.menu = None;
        ui.destroy_confirmation = None;
    }
    let Ok((root, mut node)) = roots.single_mut() else {
        return;
    };
    if *painted == next {
        return;
    }
    *painted = next;
    commands.entity(root).despawn_children();
    let Some((selection, confirmation, viewport)) = next else {
        node.display = Display::None;
        return;
    };
    let bag = ui.bag.as_ref().unwrap();
    let stack = selection.item(bag).unwrap();
    let name = config.0.catalog().definition(stack.item()).unwrap().name();
    let title = if confirmation {
        format!("Destroy {name} ×{}?", stack.count())
    } else {
        name.to_owned()
    };
    let actions = options(&config, selection, &ui, bag);
    let size = Vec2::new(188., 72. + actions.len() as f32 * 33.);
    let at = selection
        .anchor
        .clamp(Vec2::splat(6.), (viewport - size).max(Vec2::splat(6.)));
    node.display = Display::Flex;
    commands.entity(root).with_children(|screen| {
        screen
            .spawn((
                crate::ui_theme::Panel,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(at.x),
                    top: px(at.y),
                    width: px(size.x),
                    padding: UiRect::all(px(9)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    ..default()
                },
                BackgroundColor(crate::ui_theme::PANEL),
            ))
            .with_children(|menu| {
                menu.spawn(text(&title, 15.));
                for action in actions {
                    menu.spawn((
                        Choice { selection, action },
                        Button,
                        Hovered::default(),
                        AccessibleLabel::new(action.label()),
                        crate::ui_theme::Style::Slot,
                        crate::ui_theme::slot::frame(Node {
                            width: percent(100),
                            min_height: px(30),
                            padding: UiRect::horizontal(px(9)),
                            align_items: AlignItems::Center,
                            ..default()
                        }),
                    ))
                    // Keep Button's required accessibility node. Item menus
                    // dispatch pointer events by their specific mouse button.
                    .remove::<Button>()
                    .with_child(text(action.label(), 14.));
                }
            });
    });
}

pub(crate) fn evidence(world: &mut World) -> serde_json::Value {
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let point = |node: &ComputedNode, transform: &bevy::ui::UiGlobalTransform| {
        (node.size().min_element() > 0.).then(|| {
            (transform.affine().translation * crate::fantasy_ui::window_factor(node, scale))
                .to_array()
        })
    };
    let actions: serde_json::Map<_, _> = world
        .query::<(&Choice, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
        .iter(world)
        .filter_map(|(choice, node, transform)| {
            Some((
                choice.action.label().to_owned(),
                point(node, transform)?.into(),
            ))
        })
        .collect();
    let main_hand = world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<MainHand>>()
        .iter(world)
        .find_map(|(node, transform)| point(node, transform));
    let character = world
        .query::<(
            &AccessibleLabel,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
        )>()
        .iter(world)
        .find_map(|(label, node, transform)| {
            (label.0 == "Character")
                .then(|| point(node, transform))
                .flatten()
        });
    let selected = world.get_resource::<Ui>().and_then(|ui| ui.selected);
    serde_json::json!({"actions":actions,"main_hand":main_hand,"character":character,"selected":selected})
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{content::ContentId, inventory::Stack};

    #[test]
    fn menu_actions_cannot_use_an_item_after_the_painted_revision_changes() {
        let config = Config(
            Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-document-items.json"
            ))
            .unwrap(),
        );
        let catalog = config.0.catalog();
        let sword = ContentId::new("theia.dev.training-sword").unwrap();
        let bag = |revision| {
            Bag::from_slots(
                catalog,
                revision,
                std::array::from_fn(|slot| {
                    (slot == 0).then(|| Stack::new(catalog, sword.clone(), 1).unwrap())
                }),
            )
            .unwrap()
        };
        let selection = Selection {
            source: Source::Bag(0),
            revision: 4,
            anchor: Vec2::ZERO,
        };
        let mut ui = Ui {
            open: true,
            menu: Some(selection),
            ..default()
        };
        let equip = Choice {
            selection,
            action: Action::Equip,
        };
        let mut pending = Pending::default();
        apply(equip, &config, &bag(5), &mut ui, &mut pending);
        assert!(ui.menu.is_none());
        assert!(!pending.equip);

        ui.menu = Some(selection);
        let destroy = Choice {
            selection,
            action: Action::Destroy,
        };
        apply(destroy, &config, &bag(4), &mut ui, &mut pending);
        assert!(pending.destroy && ui.menu.is_some());
        assert_eq!(ui.selected, Some(0));
        assert!(!options(&config, selection, &ui, &bag(4)).contains(&Action::ConfirmDestroy));
        ui.destroy_confirmation = Some((0, 4));
        let mut confirmed = Pending::default();
        apply(
            Choice {
                selection,
                action: Action::ConfirmDestroy,
            },
            &config,
            &bag(5),
            &mut ui,
            &mut confirmed,
        );
        assert!(!confirmed.destroy);
        assert!(ui.destroy_confirmation.is_none());
    }
}
