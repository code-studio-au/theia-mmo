//! Standard Bevy journal and guide actions, painted from private committed state.
use crate::network::Network;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, Button},
};
use std::{collections::BTreeMap, num::NonZeroU128, sync::Arc};
use theia_core::{
    content::ContentId,
    quests::{
        catalog::Definition,
        journal::{Action, Command, Status},
    },
};
use theia_network::{client::State, quest_journal::Profile, renet_runtime::quest_journal::Intent};

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
    intents: BTreeMap<ContentId, Intent>,
    #[serde(skip)]
    waiting: Option<NonZeroU128>,
}
#[derive(Resource, Default)]
struct Pending(Option<Intent>);
#[derive(Component)]
struct Tracker;
#[derive(Component)]
struct Scroll;
#[derive(Component)]
struct StatusLabel;
#[derive(Component)]
struct ActionButton(ContentId);
#[derive(Component)]
struct ActionLabel(ContentId);

pub(crate) fn install(app: &mut App, profile: Arc<Profile>) {
    crate::quest_marker::install(app);
    app.insert_resource(Definitions(profile))
        .init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate)
        .add_systems(Startup, setup.after(crate::dialogue::GuideUiSetup))
        .add_systems(
            Update,
            (controls, present, scroll)
                .chain()
                .after(crate::network::present),
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
        Pickable::IGNORE,
    )
}
fn setup(
    mut commands: Commands,
    panes: Query<Entity, With<crate::dialogue::Pane>>,
    definitions: Res<Definitions>,
) {
    commands
        .spawn((
            Scroll,
            Hovered::default(),
            ScrollPosition::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(206),
                width: px(340),
                max_height: percent(38),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(10)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.9)),
        ))
        .with_children(|root| {
            root.spawn(text("Quests"));
            root.spawn((Tracker, text("Waiting for your journal...")));
            root.spawn((StatusLabel, text("")));
        });
    for pane in &panes {
        commands.entity(pane).with_children(|panel| {
            panel
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    ..default()
                })
                .with_children(|root| {
                    for quest in definitions.0.catalog().quests() {
                        root.spawn((
                            ActionButton(quest.id().clone()),
                            Button,
                            Hovered::default(),
                            AccessibleLabel::new(format!("Quest: {}", quest.title())),
                            Node {
                                width: percent(100),
                                min_height: px(34),
                                padding: UiRect::all(px(5)),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.08, 0.25, 0.3)),
                        ))
                        .with_child((ActionLabel(quest.id().clone()), text(quest.title())));
                    }
                });
        });
    }
}
fn activate(
    event: On<Activate>,
    buttons: Query<&ActionButton>,
    mut ui: ResMut<Ui>,
    mut pending: ResMut<Pending>,
) {
    let Ok(button) = buttons.get(event.entity) else {
        return;
    };
    if ui.pending || pending.0.is_some() {
        ui.ignored_pending_clicks += 1;
        return;
    }
    if let Some(intent) = ui.intents.get(&button.0).cloned() {
        pending.0 = Some(intent);
        ui.pointer_actions += 1;
    }
}
fn controls(network: Res<Network>, mut ui: ResMut<Ui>, mut pending: ResMut<Pending>) {
    let state = network.state();
    if let Some(reply) = &state.quest_journal_reply
        && reply.sequence > ui.last_reply
    {
        ui.last_reply = reply.sequence;
        if reply.rejection.is_some() {
            ui.rejected += 1;
        } else {
            ui.applied += 1;
        }
        if ui.waiting == Some(reply.request_id) {
            ui.waiting = None;
        }
    }
    if state.error.is_some() {
        ui.waiting = None;
    }
    if let Some(intent) = pending.0.take()
        && ui.waiting.is_none()
        && network.request_quest_journal(intent.clone())
    {
        ui.waiting = Some(intent.command.request_id);
        ui.submitted += 1;
    }
    ui.pending = ui.waiting.is_some() || state.quest_journal_pending;
}
fn eligible(profile: &Profile, state: &State, quest: &Definition) -> bool {
    let Some(view) = state
        .quest_journal
        .as_ref()
        .filter(|j| j.profile_hash == profile.hash() && state.error.is_none())
    else {
        return false;
    };
    quest.prerequisites().iter().all(|q| {
        view.journal
            .progress(q)
            .is_some_and(|p| p.status() == Status::Completed)
    })
}
fn action(profile: &Profile, state: &State, quest: &Definition, nonce: u64) -> Option<Intent> {
    let view = state.quest_journal.as_ref().filter(|j| {
        j.profile_hash == profile.hash() && state.error.is_none() && !state.quest_journal_pending
    })?;
    let dialogue = state
        .dialogue
        .as_ref()
        .filter(|d| d.actor == view.actor && !state.dialogue_pending)?;
    let open = dialogue.open.as_ref()?;
    let presence = state
        .guide
        .as_ref()
        .filter(|g| g.actor == view.actor && g.profile_hash == dialogue.profile_hash)?;
    let guide = presence.guide.filter(|g| g.key == open.guide)?;
    if !theia_network::dialogue::in_range(state.authority_focus()?, guide.position) {
        return None;
    }
    let progress = view.journal.progress(quest.id());
    let action = match progress.map(|p| p.status()) {
        None if eligible(profile, state, quest) => Action::Accept,
        Some(Status::Ready) => Action::TurnIn,
        _ => return None,
    };
    // Session actor incarnation plus monotonic UI intent number keeps identities
    // distinct. It is a receipt key, never permission or a reward amount.
    let mut bytes = b"theia-journal-ui-intent-v1\0".to_vec();
    bytes.extend(view.actor.epoch.value().to_le_bytes());
    bytes.extend(view.actor.entity.value().to_le_bytes());
    bytes.extend(nonce.to_le_bytes());
    bytes.extend(quest.id().as_str().as_bytes());
    let hash = theia_network::terrain::sha256(&bytes);
    Some(Intent {
        npc: open.guide,
        dialogue_revision: dialogue.revision,
        command: Command {
            request_id: NonZeroU128::new(u128::from_le_bytes(hash[..16].try_into().ok()?).max(1))?,
            quest: quest.id().clone(),
            npc: match action {
                Action::Accept => quest.giver().clone(),
                Action::TurnIn => quest.turn_in().clone(),
            },
            expected_revision: progress.map_or(0, |p| p.revision()),
            action,
        },
    })
}
fn journal_text(profile: &Profile, state: &State) -> String {
    let Some(view) = state
        .quest_journal
        .as_ref()
        .filter(|j| j.profile_hash == profile.hash() && state.error.is_none())
    else {
        return "Waiting for your journal...".into();
    };
    let mut lines = Vec::new();
    for quest in profile.catalog().quests() {
        let Some(progress) = view.journal.progress(quest.id()) else {
            continue;
        };
        let status = match progress.status() {
            Status::Active => "In progress",
            Status::Ready => "Return to your contact",
            Status::Completed => "Completed",
        };
        lines.push(format!("{} - {status}", quest.title()));
        if progress.status() == Status::Active
            && let Some(stage) = quest.stages().find(|s| s.id() == progress.stage())
        {
            for objective in stage.objectives() {
                lines.push(format!(
                    "  {}: {}/{}",
                    objective.label(),
                    progress.count(objective.id()).unwrap_or(0),
                    objective.required()
                ));
            }
        }
        if progress.status() != Status::Completed {
            lines.push(format!(
                "Completion reward: {} XP (level cap applies)",
                quest.experience()
            ));
        }
        lines.push(String::new());
    }
    if lines.is_empty() {
        "No accepted quests. Speak to a quest giver.".into()
    } else {
        lines.join("\n")
    }
}
type LabelFilter = Or<(With<ActionLabel>, With<Tracker>, With<StatusLabel>)>;
type LabelView<'a> = (
    &'a mut Text,
    Option<&'a ActionLabel>,
    Has<Tracker>,
    Has<StatusLabel>,
);
type ButtonView<'a> = (
    &'a ActionButton,
    &'a mut Node,
    &'a mut BackgroundColor,
    Option<&'a Hovered>,
    Has<Pressed>,
);
fn present(
    network: Res<Network>,
    definitions: Res<Definitions>,
    mut ui: ResMut<Ui>,
    mut labels: Query<LabelView<'_>, LabelFilter>,
    mut buttons: Query<ButtonView<'_>>,
) {
    let state = network.state();
    let profile = &definitions.0;
    ui.intents.clear();
    if !ui.pending && network.can_act() {
        for quest in profile.catalog().quests() {
            if let Some(intent) = action(profile, &state, quest, ui.submitted.saturating_add(1)) {
                ui.intents.insert(quest.id().clone(), intent);
            }
        }
    }
    for (mut label, quest, tracker, status) in &mut labels {
        let next = if tracker {
            journal_text(profile, &state)
        } else if status {
            if state.error.is_some() {
                "Connection lost. Rejoin to refresh your journal.".into()
            } else if ui.pending {
                "Waiting for confirmation...".into()
            } else if state
                .quest_journal_reply
                .as_ref()
                .is_some_and(|r| r.rejection.is_some())
            {
                "Quest unavailable or updated. Speak to your contact and try again.".into()
            } else {
                String::new()
            }
        } else if let Some(label) = quest {
            let definition = profile.catalog().quest(&label.0).unwrap();
            match ui.intents.get(&label.0).map(|i| i.command.action) {
                Some(Action::Accept) => format!(
                    "Accept: {} ({} XP)",
                    definition.title(),
                    definition.experience()
                ),
                Some(Action::TurnIn) => format!(
                    "Complete: {} ({} XP)",
                    definition.title(),
                    definition.experience()
                ),
                None => String::new(),
            }
        } else {
            String::new()
        };
        if label.0 != next {
            label.0 = next;
        }
    }
    for (button, mut node, mut color, hovered, pressed) in &mut buttons {
        node.display = if ui.intents.contains_key(&button.0) {
            Display::Flex
        } else {
            Display::None
        };
        color.0 = if pressed || hovered.is_some_and(|h| h.0) {
            Color::srgb(0.12, 0.38, 0.42)
        } else {
            Color::srgb(0.08, 0.25, 0.3)
        };
    }
}
fn scroll(
    mut events: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut roots: Query<(&Hovered, &ComputedNode, &mut ScrollPosition, &mut Node), With<Scroll>>,
) {
    let delta: f32 = events
        .read()
        .map(|e| {
            -e.y * if e.unit == MouseScrollUnit::Line {
                24.
            } else {
                1.
            }
        })
        .sum();
    let height = windows.single().map_or(720., |w| w.height());
    // Keep the tracker above the existing lower-left target/status panels.
    for (hovered, node, mut position, mut style) in &mut roots {
        style.max_height = px((height - 526.).max(80.).min(height * 0.38));
        if hovered.0 && delta != 0. {
            let max =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            position.y = (position.y + delta).clamp(0., max);
        }
    }
}

/// Read-only presentation telemetry for the existing explicit owned-X11 check.
/// This writes no intent, quest state, counter or permission.
pub(crate) fn qualification(world: &mut World, state: &State) -> Option<serde_json::Value> {
    let definitions = world.get_resource::<Definitions>()?.0.clone();
    let journal = state.quest_journal.as_ref()?;
    let ui = serde_json::to_value(world.resource::<Ui>()).ok()?;
    let buttons:Vec<_>=world.query::<(&ActionButton,&ComputedNode,&bevy::ui::UiGlobalTransform,Has<Pressed>)>().iter(world)
        .filter(|(_,node,_,_)|node.size().min_element()>0.)
        .map(|(button,node,t,pressed)|serde_json::json!({"pressed":pressed,"quest":button.0.as_str(),"point":(t.affine().translation*node.inverse_scale_factor()).to_array(),"size":(node.size()*node.inverse_scale_factor()).to_array()})).collect();
    let quests:Vec<_>=definitions.catalog().quests().filter_map(|q|journal.journal.progress(q.id()).map(|p|serde_json::json!({"id":q.id().as_str(),"revision":p.revision(),"status":format!("{:?}",p.status()),"counts":q.stages().flat_map(|s|s.objectives()).map(|o|(o.id().as_str(),p.count(o.id()).unwrap_or(0))).collect::<BTreeMap<_,_>>()}))).collect();
    let tracker=world.query_filtered::<(&Text,&ComputedNode,Option<&bevy::text::TextLayoutInfo>),With<Tracker>>().iter(world).next().map(|(text,node,glyphs)|serde_json::json!({"text":text.0,"size":(node.size()*node.inverse_scale_factor()).to_array(),"glyphs":glyphs.map_or(0,|g|g.glyphs.len())}));
    let choices = crate::dialogue::qualification_choices(world);
    Some(
        serde_json::json!({"revision":journal.revision,"quests":quests,"ui":ui,"buttons":buttons,"choices":choices,"tracker":tracker,"xp":state.progression.as_ref().map(|s|s.progress.total())}),
    )
}

#[cfg(test)]
pub(crate) mod tests;
