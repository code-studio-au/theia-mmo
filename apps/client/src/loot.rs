//! Private corpse contents are committed observations. Viewing grants nothing.
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    text::FontSize,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
#[derive(Component, Clone, Copy)]
pub(crate) struct CorpseVisual(pub theia_core::rewards::CorpseSource);
#[derive(Component)]
pub(crate) struct CorpsePick;
use theia_core::{
    progression::KillId,
    rewards::{ClaimId, MAX_VISIBLE_LIFE_CLAIMS},
};
use theia_network::{
    inventory_wire::{Outcome, Reason},
    renet_runtime::inventory::Profile,
};
#[derive(Resource)]
struct Config(Profile);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Ui {
    pub open: bool,
    pub closed_this_frame: bool,
    #[serde(skip)]
    pub life: Option<KillId>,
    #[serde(skip)]
    waiting: Option<(ClaimId, u64)>,
    #[serde(skip)]
    await_removal: Option<ClaimId>,
    #[serde(skip)]
    painted: Vec<ClaimId>,
    all: bool,
    pub status: String,
    pub opened: u64,
    pub submitted: u64,
    pub applied: u64,
    pub rejected: u64,
    pub visible_frames: u64,
}
impl Ui {
    pub(crate) fn show(&mut self, life: KillId) {
        self.life = Some(life);
        self.open = true;
        self.all = false;
        self.status = "Choose an item to pick up.".into();
        self.opened += 1;
    }
    pub(crate) fn close(&mut self) {
        self.closed_this_frame |= self.open;
        self.open = false;
        self.all = false;
    }
}
#[derive(Component, Clone, Copy)]
enum Action {
    Take(usize),
    All,
    Close,
}
#[derive(Component)]
struct Pane;
#[derive(Component)]
struct Row(usize);
#[derive(Component)]
struct Status;
#[derive(Resource, Default)]
struct Pending {
    take: Option<ClaimId>,
    all: bool,
    close: bool,
}
impl Pending {
    fn take_focused(&mut self, focused: bool) -> Self {
        let input = std::mem::take(self);
        if focused { input } else { Self::default() }
    }
}
fn activate(
    event: On<Activate>,
    actions: Query<&Action>,
    ui: Res<Ui>,
    mut input: ResMut<Pending>,
    windows: Query<&Window>,
) {
    if !ui.open {
        return;
    }
    if let Ok(action) = actions.get(event.entity) {
        if !windows.single().is_ok_and(|window| window.focused) {
            *input = Pending::default();
            return;
        }
        match action {
            Action::Take(n) => input.take = ui.painted.get(*n).copied(),
            Action::All => input.all = true,
            Action::Close => input.close = true,
        }
    }
}
pub(crate) fn install(app: &mut App, profile: Profile) {
    if !profile.rewards().is_some_and(|r| {
        r.catalog()
            .definitions()
            .any(|d| d.scope() == theia_core::rewards::Scope::PerCreatureLife)
    }) {
        return;
    }
    app.insert_resource(Config(profile))
        .init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            close_panel
                .before(update)
                .before(crate::dialogue::InputControls),
        )
        .add_systems(
            Update,
            update
                .after(crate::network::present)
                .after(crate::world_input::Interactions)
                .before(crate::inventory::close_panel),
        );
}
fn text(value: impl Into<String>) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(15.),
            ..default()
        },
        TextColor(Color::WHITE),
    )
}
fn button(action: Action, label: &str) -> impl Bundle {
    (
        Button,
        ActivateOnPress,
        Hovered::default(),
        action,
        AccessibleLabel::new(label),
        Node {
            width: percent(100),
            min_height: px(34),
            padding: UiRect::all(px(6)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.12, 0.16)),
    )
}
fn setup(mut commands: Commands, skin: Option<Res<crate::fantasy_ui::Skin>>) {
    commands
        .spawn((
            Pane,
            crate::fantasy_ui::Panel::Standard,
            GlobalZIndex(25),
            bevy::ui_widgets::ScrollArea,
            Node {
                position_type: PositionType::Absolute,
                left: px(360),
                top: px(160),
                width: px(320),
                max_height: percent(75),
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(px(12)),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.97)),
        ))
        .with_children(|panel| {
            if skin.is_some() {
                crate::fantasy_ui::window_header(
                    panel,
                    text("Wolf loot"),
                    button(Action::Close, "Close loot"),
                    "Close loot",
                );
            } else {
                panel.spawn(text("Wolf loot"));
            }
            for n in 0..MAX_VISIBLE_LIFE_CLAIMS {
                panel
                    .spawn(button(Action::Take(n), "Take loot item"))
                    .with_child((Row(n), text("")));
            }
            panel
                .spawn(button(Action::All, "Loot all"))
                .with_child(text("Loot all"));
            panel
                .spawn(button(Action::Close, "Close loot"))
                .with_child(text("Close"));
            panel.spawn((Status, text("")));
        });
}
pub(crate) fn close_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui: ResMut<Ui>,
    documents: Option<Res<crate::documents::Ui>>,
    (interface, book): (
        Option<Res<crate::fantasy_ui::settings::Ui>>,
        Option<Res<crate::combat::book::Ui>>,
    ),
    party: Option<Res<crate::party::Ui>>,
    windows: Query<&Window>,
) {
    ui.closed_this_frame = false;
    if windows.single().is_ok_and(|window| window.focused)
        && ui.open
        && keys.just_pressed(KeyCode::Escape)
        && !crate::party::owns_escape(party.as_deref())
        && !crate::combat::book::owns_escape(book.as_deref())
        && !crate::fantasy_ui::settings::owns_escape(interface.as_deref())
        && !documents
            .as_ref()
            .is_some_and(|u| u.open || u.closed_this_frame)
    {
        ui.close();
    }
}
#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    network: Res<crate::network::Network>,
    config: Res<Config>,
    mut ui: ResMut<Ui>,
    mut pending: ResMut<Pending>,
    mut panels: Query<&mut Node, With<Pane>>,
    mut buttons: Query<(Entity, &Action, &mut Node, Has<bevy::ui::Pressed>), Without<Pane>>,
    mut labels: Query<(&Row, &mut Text), Without<Status>>,
    mut status: Query<&mut Text, With<Status>>,
    windows: Query<&Window>,
) {
    let input = pending.take_focused(windows.single().is_ok_and(|window| window.focused));
    let state = network.state();
    if !network.connected()
        || state.error.is_some()
        || state
            .player_life
            .as_ref()
            .and_then(|p| p.current())
            .is_some_and(|p| p.health == 0)
    {
        ui.close();
    }
    if ui.open && input.close {
        ui.close();
    }
    let available = state
        .rewards
        .as_ref()
        .map_or(&[][..], |r| r.available.as_slice());
    if let Some((_, previous)) = ui.waiting
        && !state.inventory_pending
        && let Some(reply) = state
            .inventory
            .as_ref()
            .and_then(|i| i.reply)
            .filter(|r| r.sequence > previous)
    {
        let (claim, _) = ui.waiting.take().unwrap();
        match reply.outcome {
            Outcome::Applied | Outcome::Replayed => {
                ui.applied += 1;
                ui.await_removal = Some(claim);
                ui.status = "Item collected.".into();
            }
            Outcome::Rejected(reason) => {
                ui.rejected += 1;
                ui.all = false;
                ui.status = match reason {
                    Reason::Full => "Not enough backpack space. Loot remains on the corpse.",
                    Reason::Conflict => {
                        "Loot is unavailable. Check your quest and move closer to try again."
                    }
                    Reason::Stale => "Backpack changed. Try again.",
                    _ => "Pickup refused. Loot remains available.",
                }
                .into();
            }
        }
    }
    if ui
        .await_removal
        .is_some_and(|claim| !available.iter().any(|r| r.claim == claim))
    {
        ui.await_removal = None;
    }
    let rows: Vec<_> = available
        .iter()
        .filter(|r| r.corpse.is_some_and(|c| Some(c.life) == ui.life))
        .collect();
    if ui.open && rows.is_empty() && ui.waiting.is_none() && ui.await_removal.is_none() {
        ui.close();
    }
    ui.painted = rows.iter().map(|r| r.claim).collect();
    if ui.open {
        ui.visible_frames += 1;
        if input.all {
            ui.all = true;
        }
        let take = input
            .take
            .or_else(|| ui.all.then(|| rows.first().map(|r| r.claim)).flatten());
        if ui.waiting.is_none()
            && ui.await_removal.is_none()
            && !state.inventory_pending
            && let Some(claim) = take.filter(|c| rows.iter().any(|r| r.claim == *c))
            && let (Some(life), Some(bag)) = (ui.life, state.inventory.as_ref())
        {
            let previous = bag.reply.map_or(0, |r| r.sequence);
            if network.request_pickup(claim, life) {
                ui.waiting = Some((claim, previous));
                ui.submitted += 1;
                ui.status = "Picking up...".into();
            } else {
                ui.all = false;
                ui.status = "Backpack busy. Try again.".into();
            }
        }
    }
    for mut panel in &mut panels {
        panel.display = if ui.open {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (entity, action, mut node, pressed) in &mut buttons {
        // ActivateOnPress can hide the panel before pointer release reaches its
        // button. A hidden button must not retain Pressed across the next open.
        if !ui.open && pressed {
            commands.entity(entity).remove::<bevy::ui::Pressed>();
        }
        if let Action::Take(n) = action {
            node.display = if *n < rows.len() {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
    for (Row(n), mut label) in &mut labels {
        **label = rows
            .get(*n)
            .and_then(|r| config.0.rewards()?.catalog().definition(&r.reward))
            .map_or_else(String::new, |recipe| {
                let names: Vec<_> = recipe
                    .grants()
                    .iter()
                    .map(|g| {
                        format!(
                            "{} x{}",
                            config.0.catalog().definition(g.item()).unwrap().name(),
                            g.count()
                        )
                    })
                    .collect();
                format!("Take {}", names.join(", "))
            });
    }
    for mut label in &mut status {
        **label = ui.status.clone();
    }
}

/// Read-only native qualification telemetry; never accepts input or grants loot.
pub(crate) fn qualification(world: &mut World) -> serde_json::Value {
    let ui_scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |scale| scale.0);
    let Some(ui) = world.get_resource::<Ui>() else {
        return serde_json::Value::Null;
    };
    let value = serde_json::to_value(ui).unwrap();
    let open = ui.open;
    let count = ui.painted.len();
    let selected_life = ui.life.map(|life| life.bytes());
    let mut take = vec![None; count];
    let mut all = None;
    let mut close = None;
    if open {
        for (action, node, transform) in world
            .query::<(&Action, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
            .iter(world)
        {
            if node.size().min_element() <= 0. {
                continue;
            }
            let point = (transform.affine().translation
                * crate::fantasy_ui::window_factor(node, ui_scale))
            .to_array();
            match action {
                Action::Take(n) if *n < count => take[*n] = Some(point),
                Action::All => all = Some(point),
                Action::Close => close = Some(point),
                _ => {}
            }
        }
    }
    let sources: Vec<_> = world
        .query::<&CorpseVisual>()
        .iter(world)
        .map(|c| c.0)
        .collect();
    let frame = world.resource::<crate::floating_origin::Frame>().render;
    let corpses:Vec<_>=world.query_filtered::<(&Camera,&GlobalTransform),crate::frontend::WorldCamera>().single(world).ok().map_or_else(Vec::new,|(camera,transform)|sources.into_iter().map(|source| {
        let point=frame.to_local(source.position).map(|p|Vec3::from_array(p.metres())+Vec3::new(0.,-theia_network::world::PAWN_Y+0.2,0.));
        let screen=point.and_then(|p|camera.world_to_viewport(transform,p).ok());
        serde_json::json!({"life":source.life.bytes(),"position":source.position.metres(),"screen":screen.map(|s|s.to_array())})
    }).collect());
    serde_json::json!({"ui":value,"selected_life":selected_life,"layout":{"take":take,"all":all,"close":close},"corpses":corpses,"particle_count":crate::creature_visual::loot_particle_count(world)})
}

#[cfg(test)]
mod focus_tests {
    use super::*;
    #[test]
    fn unfocused_loot_widgets_drop_new_choices_and_keep_existing_pickup_intent() {
        let claim = ClaimId::new([1; 16]).unwrap();
        let mut app = App::new();
        app.init_resource::<Pending>()
            .insert_resource(Ui {
                open: true,
                painted: vec![claim],
                waiting: Some((claim, 4)),
                all: true,
                ..default()
            })
            .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let all = app.world_mut().spawn(Action::All).id();
        let take = app.world_mut().spawn(Action::Take(0)).id();
        app.world_mut().trigger(Activate { entity: all });
        assert!(app.world().resource::<Pending>().all);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut().trigger(Activate { entity: take });
        assert!(
            !app.world().resource::<Pending>().all
                && app.world().resource::<Pending>().take.is_none()
        );
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        let prior = app.world_mut().resource_mut::<Pending>().take_focused(true);
        assert!(!prior.all && prior.take.is_none());
        app.world_mut().trigger(Activate { entity: take });
        assert_eq!(app.world().resource::<Pending>().take, Some(claim));
        let discarded = app
            .world_mut()
            .resource_mut::<Pending>()
            .take_focused(false);
        assert!(discarded.take.is_none());
        let ui = app.world().resource::<Ui>();
        assert!(ui.open && ui.all);
        assert_eq!(
            ui.waiting,
            Some((claim, 4)),
            "Existing pickup/wait/All intent is not cancelled"
        );
    }
    #[test]
    fn loot_escape_requires_one_focused_window_and_preserves_close_priority_state() {
        for count in [0, 1, 2] {
            let claim = ClaimId::new([1; 16]).unwrap();
            let mut app = App::new();
            app.insert_resource(Ui {
                open: true,
                all: true,
                waiting: Some((claim, 4)),
                ..default()
            })
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, close_panel);
            for _ in 0..count {
                app.world_mut().spawn(Window {
                    focused: true,
                    ..default()
                });
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Escape);
            app.update();
            let ui = app.world().resource::<Ui>();
            assert_eq!(ui.open, count != 1);
            assert_eq!(ui.closed_this_frame, count == 1);
            assert_eq!(ui.all, count != 1);
            assert_eq!(ui.waiting, Some((claim, 4)));
        }
    }
}
