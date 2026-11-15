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
    pub expanded: bool,
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
struct JournalToggle;
#[derive(Component)]
struct JournalToggleLabel;
#[derive(Component)]
struct Scroll;
#[derive(Component)]
struct Offers;
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
            (journal_keys, controls, present, scroll)
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
    skin: Option<Res<crate::fantasy_ui::Skin>>,
) {
    commands
        .spawn((
            Scroll,
            GlobalZIndex(10),
            crate::fantasy_ui::Panel::QuestTracker,
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
            root.spawn((
                JournalToggle,
                Button,
                Hovered::default(),
                AccessibleLabel::new("Expand or collapse quest journal"),
                Node {
                    width: percent(100),
                    padding: UiRect::axes(px(6), px(5)),
                    ..default()
                },
                BackgroundColor(crate::fantasy_ui::PANEL),
            ))
            .with_child((
                JournalToggleLabel,
                Text::new("Objectives   ·   Journal [J]"),
                TextFont {
                    font_size: FontSize::Px(18.),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.80, 0.50)),
                Pickable::IGNORE,
            ));
            root.spawn((StatusLabel, text("")));
            root.spawn((Tracker, text("Waiting for your journal...")))
                .insert(TextFont {
                    font_size: FontSize::Px(16.),
                    ..default()
                });
        });
    for pane in &panes {
        commands.entity(pane).with_children(|panel| {
            panel
                .spawn((
                    Offers,
                    Hovered::default(),
                    ScrollPosition::default(),
                    Node {
                        width: percent(100),
                        min_height: px(0),
                        max_height: percent(45),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
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
                                flex_direction: if skin.is_some() {
                                    FlexDirection::Column
                                } else {
                                    FlexDirection::Row
                                },
                                row_gap: px(6),
                                flex_shrink: 0.,
                                padding: UiRect::all(px(5)),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.08, 0.25, 0.3)),
                        ))
                        .with_children(|label| {
                            label.spawn((
                                ActionLabel(quest.id().clone()),
                                text(quest.title()),
                                Node {
                                    max_width: percent(100),
                                    flex_shrink: 0.,
                                    ..default()
                                },
                            ));
                            if let Some(skin) = &skin
                                && let Some(grants) = definitions
                                    .0
                                    .item_rewards()
                                    .and_then(|rewards| rewards.grants(quest.id()))
                            {
                                crate::fantasy_ui::reward_icons(label, skin, grants);
                            }
                        });
                    }
                });
        });
    }
}
fn journal_keys(keys: Res<ButtonInput<KeyCode>>, windows: Query<&Window>, mut ui: ResMut<Ui>) {
    if keys.just_pressed(KeyCode::KeyJ) && windows.single().is_ok_and(|window| window.focused) {
        ui.expanded = !ui.expanded;
    }
}
fn activate(
    event: On<Activate>,
    buttons: Query<&ActionButton>,
    toggles: Query<(), With<JournalToggle>>,
    mut ui: ResMut<Ui>,
    mut pending: ResMut<Pending>,
) {
    if toggles.contains(event.entity) {
        ui.expanded = !ui.expanded;
        return;
    }
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
    let guide = presence.find(open.guide)?;
    if !theia_network::dialogue::in_range(state.authority_focus()?, guide.position) {
        return None;
    }
    let progress = view.journal.progress(quest.id());
    let action = match progress.map(|p| p.status()) {
        None if eligible(profile, state, quest) => Action::Accept,
        Some(Status::Ready) => Action::TurnIn,
        _ => return None,
    };
    let contact = match action {
        Action::Accept => quest.giver(),
        Action::TurnIn => quest.turn_in(),
    };
    if !presence.contacts.is_empty() && presence.npc(open.guide) != Some(contact) {
        return None;
    }
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
    journal_content(profile, state, true)
}
fn journal_content(profile: &Profile, state: &State, expanded: bool) -> String {
    let Some(view) = state
        .quest_journal
        .as_ref()
        .filter(|j| j.profile_hash == profile.hash() && state.error.is_none())
    else {
        return "Waiting for your journal...".into();
    };
    let mut lines = Vec::new();
    let mut entries: Vec<_> = profile
        .catalog()
        .quests()
        .filter_map(|quest| {
            view.journal
                .progress(quest.id())
                .map(|progress| (quest, progress))
        })
        .collect();
    // Keep the current objective visible above completed history in the bounded
    // tracker. Preserve catalogue order within each group for stable scrolling.
    entries.sort_by_key(|(_, progress)| progress.status() == Status::Completed);
    for (quest, progress) in entries {
        if !expanded && progress.status() == Status::Completed {
            continue;
        }
        let status = match progress.status() {
            Status::Active => "In progress",
            Status::Ready => quest.ready_label().unwrap_or("Return to your contact"),
            Status::Completed => "Completed",
        };
        lines.push(format!("{}\n{status}", quest.title()));
        if expanded {
            lines.push(quest.description().to_owned());
        }
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
        if expanded && progress.status() != Status::Completed {
            lines.push(format!("Completion reward: {} XP", quest.experience()));
            if let Some(items) = reward_items(profile, quest) {
                lines.push(format!("Items: {items}"));
            }
        }
        lines.push(String::new());
    }
    if lines.is_empty() {
        if !expanded
            && profile
                .catalog()
                .quests()
                .any(|q| view.journal.progress(q.id()).is_some())
        {
            "No active quests. Open Journal to see completed quests.".into()
        } else {
            "No accepted quests. Speak to a quest giver.".into()
        }
    } else {
        lines.join("\n")
    }
}
fn reward_items(profile: &Profile, quest: &Definition) -> Option<String> {
    let grants = profile.item_rewards()?.grants(quest.id())?;
    let inventory = profile.inventory_catalog()?;
    Some(
        grants
            .iter()
            .map(|grant| {
                // Profile construction validates every grant against this catalogue.
                let item = inventory
                    .definition(grant.item())
                    .expect("validated reward item");
                format!("{} x {}", grant.count(), item.name())
            })
            .collect::<Vec<_>>()
            .join(", "),
    )
}
fn action_text(profile: &Profile, quest: &Definition, action: Action) -> String {
    let verb = match action {
        Action::Accept => "Accept",
        Action::TurnIn => "Complete",
    };
    let mut label = format!("{verb}: {} ({} XP)", quest.title(), quest.experience());
    if action == Action::Accept {
        label.push_str(&format!("\n{}", quest.description()));
        for objective in quest.stages().flat_map(|s| s.objectives()) {
            label.push_str(&format!(
                "\n- {} ({})",
                objective.label(),
                objective.required()
            ));
        }
    }
    if let Some(items) = reward_items(profile, quest) {
        label.push_str(&format!("\nItems: {items}"));
    }
    label
}
fn status_text(state: &State, pending: bool) -> &'static str {
    use theia_network::quest_journal_wire::Rejection;
    if state.error.is_some() {
        "Connection lost. Rejoin to refresh your journal."
    } else if pending {
        "Waiting for confirmation..."
    } else {
        match state.quest_journal_reply.as_ref().and_then(|r| r.rejection) {
            Some(Rejection::InventoryFull) => {
                "Not enough backpack space for the quest reward. Free some space, then try completing the quest again."
            }
            Some(_) => "Quest unavailable or updated. Speak to your contact and try again.",
            None => "",
        }
    }
}
type LabelFilter = Or<(
    With<ActionLabel>,
    With<Tracker>,
    With<StatusLabel>,
    With<JournalToggleLabel>,
)>;
type LabelView<'a> = (
    &'a mut Text,
    Option<&'a ActionLabel>,
    Has<Tracker>,
    Has<StatusLabel>,
    Has<JournalToggleLabel>,
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
    skin: Option<Res<crate::fantasy_ui::Skin>>,
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
    for (mut label, quest, tracker, status, toggle) in &mut labels {
        let next = if tracker {
            if ui.expanded {
                journal_text(profile, &state)
            } else {
                journal_content(profile, &state, false)
            }
        } else if toggle {
            if ui.expanded {
                "Quest journal   ·   Close [J]"
            } else {
                "Objectives   ·   Journal [J]"
            }
            .into()
        } else if status {
            status_text(&state, ui.pending).into()
        } else if let Some(label) = quest {
            let definition = profile.catalog().quest(&label.0).unwrap();
            match ui.intents.get(&label.0).map(|i| i.command.action) {
                Some(action) => action_text(profile, definition, action),
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
        color.0 = if skin.is_some() {
            crate::fantasy_ui::button_color(false, hovered.is_some_and(|h| h.0), pressed)
        } else if pressed || hovered.is_some_and(|h| h.0) {
            Color::srgb(0.12, 0.38, 0.42)
        } else {
            Color::srgb(0.08, 0.25, 0.3)
        };
    }
}
type ScrollFilter = Or<(With<Scroll>, With<Offers>)>;
type ScrollView<'a> = (
    &'a Hovered,
    &'a ComputedNode,
    &'a mut ScrollPosition,
    &'a mut Node,
    Option<&'a mut GlobalZIndex>,
    Has<Offers>,
);
fn scroll(
    mut events: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    ui: Res<Ui>,
    scale: Res<bevy::ui::UiScale>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    mut roots: Query<ScrollView<'_>, ScrollFilter>,
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
    let height = windows.single().map_or(720., |w| w.height()) / scale.0;
    for (hovered, node, mut position, mut style, layer, offers) in &mut roots {
        if !offers {
            style.display = Display::Flex;
            let top = if ui.expanded {
                80.
            } else if skin.is_some() {
                252.
            } else {
                206.
            };
            style.max_height = px((height - top - 110.).max(80.));
            style.top = px(top);
            let width = px(if ui.expanded { 520. } else { 300. });
            if style.width != width {
                position.y = 0.;
            }
            style.width = width;
            style.left = if ui.expanded {
                percent(50)
            } else if skin.is_some() {
                Val::Auto
            } else {
                px(12)
            };
            style.margin.left = px(if ui.expanded { -260. } else { 0. });
            style.right = if ui.expanded || skin.is_none() {
                Val::Auto
            } else {
                px(12.)
            };
            if let Some(mut layer) = layer {
                layer.0 = if ui.expanded { 30 } else { 10 };
            }
        }
        let max = ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
        // Removing an accepted/completed offer must not leave the remaining
        // rows outside the viewport. Preserve offset during transient waiting.
        if offers && !ui.pending {
            position.y = position.y.clamp(0., max);
        }
        if hovered.0 && delta != 0. {
            position.y = (position.y + delta).clamp(0., max);
        }
    }
}

/// Read-only presentation telemetry for the existing explicit owned-X11 check.
/// This writes no intent, quest state, counter or permission.
pub(crate) fn qualification(world: &mut World, state: &State) -> Option<serde_json::Value> {
    let ui_scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |scale| scale.0);
    let definitions = world.get_resource::<Definitions>()?.0.clone();
    let journal = state.quest_journal.as_ref()?;
    let ui = serde_json::to_value(world.resource::<Ui>()).ok()?;
    let buttons:Vec<_>=world.query::<(&ActionButton,&ComputedNode,&bevy::ui::UiGlobalTransform,Has<Pressed>)>().iter(world)
        .filter(|(_,node,_,_)|node.size().min_element()>0.)
        .map(|(button,node,t,pressed)|serde_json::json!({"pressed":pressed,"quest":button.0.as_str(),"point":(t.affine().translation*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),"size":(node.size()*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),"content_box_size":(node.content_box().size()*crate::fantasy_ui::window_factor(node, ui_scale)).to_array()})).collect();
    let quests:Vec<_>=definitions.catalog().quests().filter_map(|q|journal.journal.progress(q.id()).map(|p|serde_json::json!({"id":q.id().as_str(),"revision":p.revision(),"status":format!("{:?}",p.status()),"counts":q.stages().flat_map(|s|s.objectives()).map(|o|(o.id().as_str(),p.count(o.id()).unwrap_or(0))).collect::<BTreeMap<_,_>>()}))).collect();
    let tracker=world.query_filtered::<(&Text,&ComputedNode,&bevy::ui::UiGlobalTransform,Option<&bevy::text::TextLayoutInfo>),With<Tracker>>().iter(world).next().map(|(text,node,t,glyphs)|serde_json::json!({"text":text.0,"point":(t.affine().translation*crate::fantasy_ui::window_factor(node,ui_scale)).to_array(),"size":(node.size()*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),"glyphs":glyphs.map_or(0,|g|g.glyphs.len())}));
    let action_labels: Vec<_> = world
        .query::<(
            &ActionLabel,
            &Text,
            &ComputedNode,
            Option<&bevy::text::TextLayoutInfo>,
        )>()
        .iter(world)
        .filter(|(_, _, node, _)| node.size().min_element() > 0.)
        .map(|(label, text, node, glyphs)| {
            serde_json::json!({
                "quest":label.0.as_str(),"text":text.0,
                "size":(node.size()*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
                "layout_size":glyphs.map_or(Vec2::ZERO,|g|g.size*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
                "glyphs":glyphs.map_or(0,|g|g.glyphs.len())
            })
        })
        .collect();
    let status = world.query_filtered::<(&Text, Option<&bevy::text::TextLayoutInfo>), With<StatusLabel>>()
        .iter(world).next().map(|(text, glyphs)|serde_json::json!({"text":text.0,"glyphs":glyphs.map_or(0,|g|g.glyphs.len())}));
    let choices = crate::dialogue::qualification_choices(world);
    let dialogue_body = crate::dialogue::qualification_body(world);
    let offer_scroll=world.query_filtered::<(&ComputedNode,&bevy::ui::UiGlobalTransform,&ScrollPosition),With<Offers>>()
        .iter(world).find(|(node,_,_)|node.size().min_element()>0.)
        .map(|(node,t,scroll)|serde_json::json!({
            "point":(t.affine().translation*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
            "size":(node.size()*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
            "content":(node.content_size()*crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
            "scroll":[scroll.x,scroll.y]
        }));
    Some(
        serde_json::json!({"revision":journal.revision,"quests":quests,"ui":ui,"buttons":buttons,"action_labels":action_labels,"status":status,"choices":choices,"dialogue_body":dialogue_body,"offer_scroll":offer_scroll,"tracker":tracker,"xp":state.progression.as_ref().map(|s|s.progress.total())}),
    )
}

#[cfg(test)]
pub(crate) mod tests;
