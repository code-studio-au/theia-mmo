//! Standard Bevy quest journal and NPC buttons; all progress comes from committed
//! private authority observations. Clicks retain their painted intent unchanged.
use crate::network::Network;
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use std::sync::Arc;
use theia_core::quests::lifecycle::{Action, Phase};
use theia_network::{
    quest_lifecycle_wire::{Profile, Rejection},
    renet_runtime::quest_lifecycle::Intent,
};
#[derive(Resource)]
pub(crate) struct Definitions(pub Arc<Profile>);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Ui {
    pub pending: bool,
    pub pointer_actions: u64,
    pub submitted: u64,
    pub applied: u64,
    pub rejected: u64,
    pub ignored_pending_clicks: u64,
    pub last_reply: u64,
    #[serde(skip)]
    intent: Option<Intent>,
    #[serde(skip)]
    waiting: bool,
    #[serde(skip)]
    reply_floor: u64,
}
#[derive(Resource, Default)]
struct Pending(Option<Intent>);
#[derive(Component)]
pub(crate) struct Label;
#[derive(Component)]
pub(crate) struct ActionButton;
#[derive(Component)]
struct ActionLabel;
#[derive(Component)]
struct StatusLabel;
#[derive(Component)]
pub(crate) struct RewardLabel;
fn activate(
    event: On<Activate>,
    buttons: Query<(), With<ActionButton>>,
    mut ui: ResMut<Ui>,
    mut pending: ResMut<Pending>,
) {
    if buttons.get(event.entity).is_err() {
        return;
    }
    if ui.pending || pending.0.is_some() {
        ui.ignored_pending_clicks += 1;
        return;
    }
    if let Some(intent) = ui.intent {
        pending.0 = Some(intent);
        ui.pointer_actions += 1;
    }
}
pub(crate) fn install(app: &mut App, profile: Arc<Profile>) {
    crate::quest_marker::install(app);
    app.insert_resource(Definitions(profile))
        .init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate)
        .add_systems(Startup, setup.after(crate::dialogue::GuideUiSetup))
        .add_systems(
            Update,
            (controls, present).chain().after(crate::network::present),
        );
}
fn text(value: &str) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(14.),
            ..default()
        },
        TextColor(Color::WHITE),
    )
}
fn setup(
    mut commands: Commands,
    panes: Query<Entity, With<crate::dialogue::Pane>>,
    definitions: Res<Definitions>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(206),
                width: px(340),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.9)),
            Pickable::IGNORE,
        ))
        .with_child((Label, text("Training: waiting for server")));
    for pane in &panes {
        commands.entity(pane).with_children(|panel| {
            panel
                .spawn((Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    ..default()
                },))
                .with_children(|root| {
                    root.spawn((
                        ActionButton,
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        AccessibleLabel::new("Accept or turn in training quest"),
                        Node {
                            width: percent(100),
                            min_height: px(30),
                            padding: UiRect::all(px(4)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            display: Display::None,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.08, 0.25, 0.3)),
                    ))
                    .with_child((ActionLabel, text("Accept training")));
                    root.spawn((StatusLabel, text("")));
                    if let Some(recipe) = definitions.0.turn_in_xp() {
                        root.spawn((
                            RewardLabel,
                            text(&format!(
                                "Completion reward: {} XP (level cap applies)",
                                recipe.experience()
                            )),
                        ));
                    }
                });
        });
    }
}
fn controls(network: Res<Network>, mut ui: ResMut<Ui>, mut pending: ResMut<Pending>) {
    let state = network.state();
    if let Some(reply) = state.quest_lifecycle_reply
        && reply.sequence > ui.last_reply
    {
        ui.last_reply = reply.sequence;
        if reply.rejection.is_some() {
            ui.rejected += 1;
        } else {
            ui.applied += 1;
        }
    }
    if ui.waiting
        && (state.error.is_some()
            || state
                .quest_lifecycle_reply
                .is_some_and(|r| r.sequence > ui.reply_floor))
    {
        ui.waiting = false;
    }
    if let Some(intent) = pending.0.take()
        && !ui.waiting
        && network.request_quest(intent)
    {
        ui.reply_floor = state.quest_lifecycle_reply.map_or(0, |r| r.sequence);
        ui.waiting = true;
        ui.submitted += 1;
    }
    ui.pending = ui.waiting || state.quest_lifecycle_pending;
}
type TextRole<'a> = (
    &'a mut Text,
    Option<&'a Label>,
    Option<&'a ActionLabel>,
    Option<&'a StatusLabel>,
);
type TextFilter = Or<(With<Label>, With<ActionLabel>, With<StatusLabel>)>;
type ButtonView<'a> = (
    &'a mut Node,
    &'a mut BackgroundColor,
    Option<&'a Hovered>,
    Has<Pressed>,
);
fn present(
    network: Res<Network>,
    definitions: Res<Definitions>,
    mut ui: ResMut<Ui>,
    mut labels: Query<TextRole<'_>, TextFilter>,
    mut buttons: Query<ButtonView<'_>, With<ActionButton>>,
) {
    let state = network.state();
    let view = state
        .quest_lifecycle
        .as_ref()
        .filter(|_| state.error.is_none());
    let progress = view.map(|v| {
        v.progress
            .progress(definitions.0.profile())
            .expect("Validated committed quest lifecycle")
    });
    let context = state
        .dialogue
        .as_ref()
        .filter(|_| state.error.is_none() && !state.dialogue_pending)
        .and_then(|v| v.open.as_ref().map(|open| (v.revision, open.guide)));
    let action = progress.and_then(|p| match p.phase {
        Phase::Available => Some(Action::Accept),
        Phase::Ready => Some(Action::TurnIn),
        _ => None,
    });
    ui.intent = if !ui.pending {
        view.zip(context)
            .zip(action)
            .map(|((view, (dialogue_revision, npc)), action)| Intent {
                expected_revision: view.progress.revision(),
                dialogue_revision,
                npc,
                action,
            })
    } else {
        None
    };
    let journal = if state.error.is_some() {
        "Training: disconnected".to_owned()
    } else if let Some(p) = progress {
        let line = match p.phase {
            Phase::Available => "Talk to the guide to accept training".to_owned(),
            Phase::Active => format!("Defeat creatures: {} / {}", p.count, p.required),
            Phase::Ready => "Ready: return to the guide".to_owned(),
            Phase::TurnedIn => "Training complete".to_owned(),
        };
        format!("{}\n{}", definitions.0.profile().name(), line)
    } else {
        "Training: waiting for server".to_owned()
    };
    let status = if ui.pending {
        "Saving quest..."
    } else if let Some(reason) = state
        .quest_lifecycle_reply
        .filter(|r| view.is_some_and(|v| r.quest_revision == v.progress.revision()))
        .and_then(|r| r.rejection)
    {
        match reason {
            Rejection::NotEntitled => "Talk to the guide again.",
            Rejection::StaleRevision => "Quest updated. Try again.",
            Rejection::NotAvailable => "Check your current training progress.",
            Rejection::NotReady => "Finish your training first.",
            Rejection::AlreadyTurnedIn => "Training complete.",
            Rejection::Busy => "Please try again shortly.",
        }
    } else {
        match progress.map(|p| p.phase) {
            Some(Phase::Available) => "Accept training before defeating creatures.",
            Some(Phase::Active) => "Defeat creatures, then return to the guide.",
            Some(Phase::Ready) => "Your training is ready to turn in.",
            Some(Phase::TurnedIn) => "You have completed this training.",
            None => "Waiting for your quest journal.",
        }
    };
    for (mut text, journal_label, action_label, status_label) in &mut labels {
        let value = if journal_label.is_some() {
            journal.as_str()
        } else if action_label.is_some() {
            if ui.pending {
                "Saving..."
            } else if action == Some(Action::TurnIn) {
                "Turn in training"
            } else {
                "Accept training"
            }
        } else if status_label.is_some() {
            status
        } else {
            continue;
        };
        if text.0 != value {
            text.0 = value.to_owned();
        }
    }
    for (mut node, mut color, hover, pressed) in &mut buttons {
        node.display = if context.is_some() && action.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        color.0 = if ui.pending {
            Color::srgb(0.08, 0.1, 0.12)
        } else if pressed {
            Color::srgb(0.14, 0.4, 0.4)
        } else if hover.is_some_and(|h| h.get()) {
            Color::srgb(0.12, 0.34, 0.38)
        } else {
            Color::srgb(0.08, 0.25, 0.3)
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::identity::{EntityId, EntityKey, WorldEpoch};
    #[test]
    fn activation_retains_painted_quest_and_npc_revisions_and_coalesces_extra_clicks() {
        let npc = EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        let mut app = App::new();
        app.insert_resource(Ui {
            intent: Some(Intent {
                expected_revision: 0,
                dialogue_revision: 1,
                npc,
                action: Action::Accept,
            }),
            ..default()
        })
        .init_resource::<Pending>()
        .add_observer(activate);
        let button = app.world_mut().spawn(ActionButton).id();
        app.world_mut().trigger(Activate { entity: button });
        app.world_mut().resource_mut::<Ui>().intent = Some(Intent {
            expected_revision: 4,
            dialogue_revision: 3,
            npc,
            action: Action::TurnIn,
        });
        app.world_mut().trigger(Activate { entity: button });
        let captured = app.world().resource::<Pending>().0.unwrap();
        assert_eq!(captured.expected_revision, 0);
        assert_eq!(captured.dialogue_revision, 1);
        assert_eq!(captured.npc, npc);
        assert_eq!(captured.action, Action::Accept);
        assert_eq!(app.world().resource::<Ui>().pointer_actions, 1);
        assert_eq!(app.world().resource::<Ui>().ignored_pending_clicks, 1);
    }
}
