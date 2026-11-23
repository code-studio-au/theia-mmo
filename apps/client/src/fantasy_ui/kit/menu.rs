//! The kit's main menu and HUD shortcuts route to existing native windows.
use crate::ui_theme::{self, Style};
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{Activate, Button},
};
#[derive(Resource, Default)]
pub(crate) struct State {
    open: bool,
    closed_this_frame: bool,
}
pub(crate) fn owns_escape(state: Option<&State>) -> bool {
    state.is_some_and(|s| s.open || s.closed_this_frame)
}
#[derive(Component, Clone, Copy)]
enum Action {
    Menu,
    Character,
    Inventory,
    Skills,
    Quests,
    Map,
    Party,
    Settings,
    Quit,
}
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct Pane;
pub(super) fn install(app: &mut App) {
    app.init_resource::<State>()
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(Update, present);
}
fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    action: Option<Action>,
    title: &str,
    index: usize,
    compact: bool,
) {
    let mut e = parent.spawn((
        Button,
        Hovered::default(),
        Style::Secondary,
        AccessibleLabel::new(title),
        Node {
            width: if compact { px(42) } else { percent(100) },
            min_height: px(40),
            padding: UiRect::horizontal(px(if compact { 4. } else { 14. })),
            column_gap: px(10),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    if let Some(action) = action {
        e.insert(action);
    } else {
        e.insert(InteractionDisabled);
    }
    e.with_children(|row| {
        super::icon(row, index, action.is_none());
        if !compact {
            row.spawn(super::text(title, 17.));
        }
    });
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Hud,
            GlobalZIndex(45),
            Node {
                position_type: PositionType::Absolute,
                right: px(14),
                bottom: px(14),
                column_gap: px(5),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|hud| {
            for (action, title, index) in [
                (Some(Action::Character), "Character", 0),
                (Some(Action::Inventory), "Inventory", 1),
                (Some(Action::Skills), "Skills", 5),
                (Some(Action::Quests), "Quests", 3),
                (Some(Action::Map), "Map", 4),
                (Some(Action::Party), "Party", 15),
                (None, "Collections", 6),
                (Some(Action::Menu), "Main Menu", 7),
            ] {
                spawn_button(hud, action, title, index, true);
            }
        });
    commands
        .spawn((
            Pane,
            ui_theme::Panel,
            GlobalZIndex(95),
            Node {
                position_type: PositionType::Absolute,
                right: px(14),
                bottom: px(72),
                width: px(250),
                max_height: percent(85),
                padding: UiRect::all(px(16)),
                display: Display::None,
                ..super::column()
            },
            BackgroundColor(ui_theme::PANEL),
        ))
        .with_children(|menu| {
            ui_theme::heading(menu, "Main Menu");
            for (action, title, index) in [
                (Some(Action::Character), "Character", 0),
                (Some(Action::Inventory), "Inventory", 1),
                (Some(Action::Skills), "Skills", 2),
                (Some(Action::Quests), "Quests", 3),
                (Some(Action::Map), "Map", 4),
                (None, "Journal", 5),
                (None, "Collections", 6),
                (Some(Action::Settings), "Settings", 7),
                (Some(Action::Quit), "Quit", 8),
            ] {
                spawn_button(menu, action, title, index, false);
            }
            spawn_button(menu, Some(Action::Menu), "Close", 21, false);
        });
}
#[allow(clippy::too_many_arguments)]
fn activate(
    event: On<Activate>,
    actions: Query<&Action, Without<InteractionDisabled>>,
    mut state: ResMut<State>,
    mut inventory: Option<ResMut<crate::inventory::Ui>>,
    mut skills: Option<ResMut<crate::combat::book::Ui>>,
    mut quests: Option<ResMut<crate::quest_journal::Ui>>,
    mut map: Option<ResMut<crate::zone_map::Ui>>,
    mut party: Option<ResMut<crate::party::Ui>>,
    mut settings: Option<ResMut<super::super::settings::Ui>>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    match action {
        Action::Menu => {
            state.open = !state.open;
            return;
        }
        Action::Character => {
            if let Some(ref mut ui) = inventory
                && ui.equipment_enabled
                && !ui.vendor_open
            {
                ui.equipment_open = !ui.equipment_open;
            }
        }
        Action::Inventory => {
            if let Some(ref mut ui) = inventory {
                ui.open = !ui.open;
            }
        }
        Action::Skills => {
            if let Some(ref mut ui) = skills {
                ui.open = !ui.open;
            }
        }
        Action::Quests => {
            if let Some(ref mut ui) = quests {
                ui.expanded = !ui.expanded;
            }
        }
        Action::Map => {
            if let Some(ref mut ui) = map {
                ui.open = !ui.open;
            }
        }
        Action::Party => {
            if let Some(ref mut ui) = party {
                ui.open = !ui.open;
            }
        }
        Action::Settings => {
            if let Some(ref mut ui) = settings {
                ui.open = true;
                ui.graphics_open = true;
                ui.developer_open = false;
            }
        }
        Action::Quit => {
            exit.write(bevy::app::AppExit::Success);
        }
    }
    state.open = false;
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn present(
    mut commands: Commands,
    mut state: ResMut<State>,
    network: Option<Res<crate::network::Network>>,
    frontend: Option<Res<crate::frontend::Menu>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut nodes: Query<(&mut Node, Has<Pane>), Or<(With<Hud>, With<Pane>)>>,
    buttons: Query<(Entity, &Action)>,
    inventory: Option<Res<crate::inventory::Ui>>,
    skills: Option<Res<crate::combat::book::Ui>>,
    quests: Option<Res<crate::quest_journal::Ui>>,
    map: Option<Res<crate::zone_map::Ui>>,
    settings: Option<Res<super::super::settings::Ui>>,
) {
    let active = frontend.as_ref().is_none_or(|m| !m.active())
        && network.as_ref().is_some_and(|n| n.state().id.is_some());
    state.closed_this_frame = state.open && keys.just_pressed(KeyCode::Escape);
    if !active || keys.just_pressed(KeyCode::Escape) {
        state.open = false;
    }
    for (mut node, pane) in &mut nodes {
        let display = if active && (!pane || state.open) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (e, action) in &buttons {
        let enabled = match action {
            Action::Character => inventory
                .as_ref()
                .is_some_and(|ui| ui.equipment_enabled && !ui.vendor_open),
            Action::Inventory => inventory.is_some(),
            Action::Skills => skills.is_some(),
            Action::Quests => quests.is_some(),
            Action::Map => map.is_some(),
            Action::Settings => settings.is_some(),
            Action::Party => network.as_ref().is_some_and(|n| n.state().party.is_some()),
            _ => true,
        };
        if enabled {
            commands.entity(e).remove::<InteractionDisabled>();
        } else {
            commands.entity(e).insert(InteractionDisabled);
        }
    }
}
