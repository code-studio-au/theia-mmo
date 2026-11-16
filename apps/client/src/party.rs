//! Standard Bevy controls for the admitted, private server-owned party view.
mod invitation;
mod member_menu;
mod members;
pub(crate) mod selection;
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui_widgets::{Activate, Button},
};
use theia_core::party::{Action, Refusal};
use theia_network::{party_client::Intent, party_wire::State};

pub(crate) fn parse(
    args: &mut Vec<String>,
    classic: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Some(index) = args.iter().position(|a| a == "--parties") else {
        return Ok(false);
    };
    args.remove(index);
    if !classic || args.iter().any(|a| a == "--parties") {
        return Err("Choose party mode once with Classic controls".into());
    }
    Ok(true)
}
pub(crate) fn world_identity(source: [u8; 32], enabled: bool) -> [u8; 32] {
    if enabled {
        theia_network::party_wire::profile_identity(source)
    } else {
        source
    }
}
#[derive(Resource, Default)]
pub(crate) struct Ui {
    pub open: bool,
    pub closed_this_frame: bool,
    menu: Option<member_menu::Selection>,
    menu_dragged: bool,
    waiting: Option<u64>,
    rendered: Option<Model>,
    actions: u64,
}
pub(crate) fn owns_escape(ui: Option<&Ui>) -> bool {
    ui.is_some_and(|u| u.open || u.menu.is_some() || u.closed_this_frame)
}
#[derive(Component)]
struct Pane;
#[derive(Component)]
struct ToggleLabel;
#[derive(Component, Clone)]
enum Control {
    Toggle,
    Close,
    Command(Intent),
    Invitation(Intent),
    Member(Intent),
    Manage,
    MenuClose,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Choice {
    label: String,
    intent: Intent,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    label: String,
    choices: Vec<Choice>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Model {
    title: String,
    status: String,
    rows: Vec<Row>,
    leave: Option<Intent>,
    enabled: bool,
}
fn intent(state: &State, action: Action) -> Intent {
    Intent {
        owner: state.view.owner,
        revision: state.view.revision,
        action,
    }
}
fn model(state: &State, enabled: bool, timed_out: bool) -> Model {
    let owner = state.view.owner;
    let membership = &state.view.membership;
    let leader = membership.party.as_ref().is_none_or(|p| p.leader == owner);
    let mut rows = Vec::new();
    for actor in &state.roster {
        let mut label = actor.name.as_str().to_owned();
        let mut choices = Vec::new();
        let mut choice = |label: &str, action| {
            choices.push(Choice {
                label: label.to_owned(),
                intent: intent(state, action),
            })
        };
        if actor.key == owner {
            label.push_str(" (you)");
        }
        if membership
            .party
            .as_ref()
            .is_some_and(|p| p.leader == actor.key)
        {
            label.push_str(" · Leader");
        }
        if membership
            .party
            .as_ref()
            .is_some_and(|p| p.members.contains(&actor.key))
        {
            if actor.key != owner && leader {
                choice("Promote", Action::Promote(actor.key));
                choice("Remove", Action::Kick(actor.key));
            }
        } else if let Some(invitation) = &membership.incoming
            && invitation.inviter == actor.key
        {
            label.push_str(" · Invites you");
            choice("Accept", Action::Accept(invitation.id));
            choice("Decline", Action::Decline(invitation.id));
        } else if let Some(invitation) = membership
            .outgoing
            .iter()
            .find(|i| i.recipient == actor.key)
        {
            label.push_str(" · Invited");
            choice("Cancel", Action::Cancel(invitation.id));
        } else if actor.key != owner && leader {
            let reserved = membership.party.as_ref().map_or(1, |p| p.members.len())
                + membership.outgoing.len();
            if reserved < theia_core::party::MAX_MEMBERS {
                choice("Invite", Action::Invite(actor.key));
            }
        }
        rows.push(Row { label, choices });
    }
    let title = membership.party.as_ref().map_or_else(
        || "Party · Solo".to_owned(),
        |p| format!("Party · {} / 5 members", p.members.len()),
    );
    let status = if !enabled {
        "Waiting for a party response…"
    } else if timed_out {
        "No reply yet. Check the party before trying again."
    } else {
        match state.refusal {
            Some(Refusal::Stale) => "The party changed. Try again.",
            Some(Refusal::RateLimited) => "Please wait a moment before trying again.",
            Some(Refusal::MissingPlayer) => "That player is no longer here.",
            Some(Refusal::SelfTarget) => "Choose another player.",
            Some(Refusal::AlreadyGrouped) => "That player is already in a party.",
            Some(Refusal::InvitationPending) => "That player already has an invitation.",
            Some(Refusal::NotLeader) => "Only the party leader can do that.",
            Some(Refusal::Full) => "The party has no free places.",
            Some(Refusal::MissingInvitation) => "That invitation has ended.",
            Some(Refusal::NotMember) => "That player is no longer in your party.",
            Some(_) => "The party action could not be completed.",
            None if membership.incoming.is_some() => "You have a party invitation.",
            None if state.view.sequence > 0 => "Party updated.",
            None => "Invite another player to form a party.",
        }
    }
    .to_owned();
    Model {
        title,
        status,
        rows,
        leave: membership
            .party
            .as_ref()
            .map(|_| intent(state, Action::Leave)),
        enabled,
    }
}
fn label(value: &str) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(16.),
            ..default()
        },
        TextColor(Color::WHITE),
        Pickable::IGNORE,
    )
}
fn button(control: Control, title: &str, enabled: bool) -> impl Bundle {
    (
        control,
        Button,
        Hovered::default(),
        AccessibleLabel::new(title),
        Node {
            padding: UiRect::all(px(8)),
            min_height: px(34),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(if enabled {
            Color::srgb(0.17, 0.21, 0.28)
        } else {
            Color::srgb(0.08, 0.09, 0.11)
        }),
    )
}
pub(crate) fn install(app: &mut App) {
    invitation::install(app);
    members::install(app);
    member_menu::install(app);
    app.init_resource::<Ui>()
        .init_resource::<selection::Presentation>()
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            keyboard
                .after(bevy::input::InputSystems)
                .after(crate::fantasy_ui::settings::keyboard)
                .before(crate::combat::book::keyboard)
                .before(crate::input),
        )
        .add_systems(
            Update,
            (
                present,
                invitation::present,
                members::present,
                members::paint,
                member_menu::present,
                selection::present,
                selection::paint,
            )
                .chain()
                .after(crate::network::present)
                .after(crate::world_input::Interactions),
        );
}
fn setup(mut commands: Commands) {
    commands
        .spawn(button(Control::Toggle, "Party [O]", true))
        .insert((
            GlobalZIndex(30),
            Node {
                position_type: PositionType::Absolute,
                right: px(312),
                bottom: px(12),
                width: px(140),
                min_height: px(30),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_child((ToggleLabel, label("Party [O]")));
    commands.spawn((
        Pane,
        GlobalZIndex(85),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(110),
            margin: UiRect::left(px(-300)),
            width: px(600),
            max_width: percent(90),
            max_height: percent(80),
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            overflow: Overflow::scroll_y(),
            display: Display::None,
            ..default()
        },
        BackgroundColor(crate::fantasy_ui::PANEL),
        Outline::new(px(1), px(-1), crate::fantasy_ui::GOLD),
    ));
}
pub(crate) fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut ui: ResMut<Ui>,
    interface: Option<Res<crate::fantasy_ui::settings::Ui>>,
) {
    ui.closed_this_frame = false;
    if !windows.single().is_ok_and(|w| w.focused) {
        ui.menu = None;
        return;
    }
    if keys.just_pressed(KeyCode::KeyO) {
        ui.menu = None;
        ui.open = !ui.open;
        ui.closed_this_frame = !ui.open;
    } else if keys.just_pressed(KeyCode::Escape)
        && (ui.open || ui.menu.is_some())
        && !crate::fantasy_ui::settings::owns_escape(interface.as_deref())
    {
        if ui.menu.take().is_none() {
            ui.open = false;
        }
        ui.closed_this_frame = true;
    }
}
fn activate(
    event: On<Activate>,
    controls: Query<&Control>,
    windows: Query<&Window>,
    mut ui: ResMut<Ui>,
    network: Res<crate::network::Network>,
) {
    if !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    match control {
        Control::Toggle => {
            ui.menu = None;
            ui.open = !ui.open;
            ui.closed_this_frame = !ui.open;
        }
        Control::Manage => {
            ui.menu = None;
            ui.open = true;
        }
        Control::MenuClose => {
            ui.menu = None;
            ui.closed_this_frame = true;
        }
        Control::Close if ui.open => {
            ui.open = false;
            ui.closed_this_frame = true;
        }
        Control::Command(intent) | Control::Invitation(intent) | Control::Member(intent)
            if ui.waiting.is_none() =>
        {
            let prompt = matches!(control, Control::Invitation(_));
            let menu = matches!(control, Control::Member(_));
            if !menu && prompt == ui.open {
                return;
            }
            let state = network.state();
            if state.error.is_some() {
                return;
            }
            let Some(view) = state.party else {
                return;
            };
            if view.pending
                || (prompt && !invitation::current(&view.state, intent))
                || (menu
                    && !ui.menu.as_ref().is_some_and(|selected| {
                        member_menu::current(&view.state, selected, intent)
                    }))
            {
                return;
            }
            if network.party_action(intent.clone()) {
                ui.waiting = Some(view.submitted);
                ui.actions += 1;
                if menu {
                    ui.menu = None;
                }
            }
        }
        _ => {}
    }
}
fn present(
    mut commands: Commands,
    network: Res<crate::network::Network>,
    mut ui: ResMut<Ui>,
    mut panes: Query<(Entity, &mut Node), With<Pane>>,
    mut toggles: Query<&mut Text, With<ToggleLabel>>,
) {
    let state = network.state();
    if let Some(view) = &state.party
        && ui
            .waiting
            .is_some_and(|sequence| view.submitted > sequence && !view.pending)
    {
        ui.waiting = None;
    }
    let next = if state.error.is_some() {
        ui.waiting = None;
        Model {
            title: "Party".into(),
            status: "Connection lost.".into(),
            rows: vec![],
            leave: None,
            enabled: false,
        }
    } else if let Some(view) = &state.party {
        model(
            &view.state,
            !view.pending && ui.waiting.is_none(),
            view.timed_out,
        )
    } else {
        Model {
            title: "Party".into(),
            status: "Connecting…".into(),
            rows: vec![],
            leave: None,
            enabled: false,
        }
    };
    let incoming = state
        .party
        .as_ref()
        .is_some_and(|v| v.state.view.membership.incoming.is_some());
    for mut text in &mut toggles {
        text.0 = if incoming {
            "Invite · Party [O]"
        } else {
            "Party [O]"
        }
        .into();
    }
    for (entity, mut node) in &mut panes {
        node.display = if ui.open {
            Display::Flex
        } else {
            Display::None
        };
        if ui.rendered.as_ref() == Some(&next) {
            continue;
        }
        commands
            .entity(entity)
            .despawn_children()
            .with_children(|pane| {
                pane.spawn(label(&next.title))
                    .insert(TextColor(crate::fantasy_ui::GOLD));
                pane.spawn(label(&next.status));
                for row in &next.rows {
                    pane.spawn((
                        Node {
                            width: percent(100),
                            align_items: AlignItems::Center,
                            column_gap: px(8),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|line| {
                        line.spawn((
                            label(&row.label),
                            Node {
                                flex_grow: 1.,
                                ..default()
                            },
                        ));
                        for choice in &row.choices {
                            line.spawn(button(
                                Control::Command(choice.intent.clone()),
                                &format!("{} {}", choice.label, row.label),
                                next.enabled,
                            ))
                            .with_child(label(&choice.label));
                        }
                    });
                }
                if let Some(intent) = &next.leave {
                    pane.spawn(button(
                        Control::Command(intent.clone()),
                        "Leave party",
                        next.enabled,
                    ))
                    .with_child(label("Leave party"));
                }
                pane.spawn(button(Control::Close, "Close party", true))
                    .with_child(label("Close [Esc]"));
            });
    }
    ui.rendered = Some(next);
}

pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let Some(ui) = world.get_resource::<Ui>() else {
        return serde_json::Value::Null;
    };
    let open = ui.open;
    let actions = ui.actions;
    let invitation = invitation::snapshot(world);
    let member_frames = members::snapshot(world);
    let member_menu = member_menu::snapshot(world);
    let selected_member = selection::snapshot(world);
    let menu_visible = member_menu["visible"].as_bool().unwrap_or(false);
    let invitation_visible = invitation["visible"].as_bool().unwrap_or(false);
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let buttons: Vec<_> = world.query::<(&Control, &AccessibleLabel, &ComputedNode, &bevy::ui::UiGlobalTransform)>().iter(world)
        .filter(|(control, _, node, _)| match control {
            Control::Toggle => true,
            Control::Invitation(_) => invitation_visible,
            Control::Member(_) | Control::Manage | Control::MenuClose => menu_visible,
            _ => open,
        } && node.size().min_element() > 0.)
        .map(|(control, label, node, transform)| {
            let (action, target) = match control {
                Control::Toggle => ("toggle", None), Control::Close => ("close", None),
                Control::Manage => ("manage", None), Control::MenuClose => ("menu-close", None),
                Control::Command(intent) | Control::Invitation(intent) | Control::Member(intent) => match intent.action {
                    Action::Invite(k) => ("invite", Some(k.entity.value())), Action::Kick(k) => ("remove", Some(k.entity.value())), Action::Promote(k) => ("promote", Some(k.entity.value())),
                    Action::Accept(id) => ("accept", Some(id)), Action::Decline(id) => ("decline", Some(id)), Action::Cancel(id) => ("cancel", Some(id)), Action::Leave => ("leave", None),
                },
            };
            serde_json::json!({"action":action,"target":target,"source":if matches!(control, Control::Member(_) | Control::Manage | Control::MenuClose) {"member"} else {"party"},"label":label.0,"point":(transform.affine().translation * crate::fantasy_ui::window_factor(node, scale)).to_array()})
        }).collect();
    let state = world
        .get_resource::<crate::network::Network>()
        .map(|n| n.state());
    let party = state.as_ref().and_then(|s| s.party.as_ref());
    serde_json::json!({"open":open,"invitation":invitation,"member_frames":member_frames,"member_menu":member_menu,"selected_member":selected_member,"actions":actions,"buttons":buttons,"state":party.map(|v| serde_json::json!({
        "owner":v.state.view.owner.entity.value(),"revision":v.state.view.revision,"sequence":v.state.view.sequence,
        "pending":v.pending,"timed_out":v.timed_out,"refusal":v.state.refusal.map(|r|format!("{r:?}")),
        "party":v.state.view.membership.party.as_ref().map(|p|serde_json::json!({"id":p.id,"leader":p.leader.entity.value(),"members":p.members.iter().map(|k|k.entity.value()).collect::<Vec<_>>()})),
        "incoming":v.state.view.membership.incoming.as_ref().map(|i|serde_json::json!({"id":i.id,"inviter":i.inviter.entity.value()})),
        "outgoing":v.state.view.membership.outgoing.iter().map(|i|serde_json::json!({"id":i.id,"recipient":i.recipient.entity.value()})).collect::<Vec<_>>(),
        "roster":v.state.roster.iter().map(|a|serde_json::json!({"actor":a.key.entity.value(),"name":a.name.as_str()})).collect::<Vec<_>>()
    }))})
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        identity::{EntityId, EntityKey, WorldEpoch},
        party,
        time::TickRate,
    };
    pub(super) fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    pub(super) fn state(authority: &party::Authority, id: u64) -> State {
        State {
            world: [1; 32],
            tick: 1,
            view: authority.view(key(id)).unwrap(),
            refusal: None,
            member_vitals: authority
                .view(key(id))
                .unwrap()
                .membership
                .party
                .map_or_else(Vec::new, |p| {
                    p.members
                        .into_iter()
                        .map(|key| theia_network::party_wire::MemberVitals { key, vitals: None })
                        .collect()
                }),
            roster: (1..=3)
                .map(|id| theia_network::party_wire::NamedActor {
                    key: key(id),
                    name: theia_core::character_name::CharacterName::parse(
                        ["Arden", "Briar", "Cedar"][(id - 1) as usize],
                    )
                    .unwrap(),
                })
                .collect(),
        }
    }
    #[test]
    fn controls_follow_private_invitation_and_leadership_without_rebinding_painted_clicks() {
        let mut authority = party::Authority::new(key(1).epoch, TickRate::Hz30);
        for id in 1..=3 {
            authority.admit(key(id)).unwrap();
        }
        authority.advance(1).unwrap();
        let solo = model(&state(&authority, 1), true, false);
        let painted = solo.rows[1].choices[0].intent.clone();
        assert_eq!(painted.action, Action::Invite(key(2)));
        assert!(solo.rows[0].choices.is_empty());
        authority
            .request(
                key(1),
                party::Request {
                    sequence: 1,
                    revision: painted.revision,
                    action: painted.action,
                },
            )
            .unwrap();
        let invited = state(&authority, 2);
        let received = model(&invited, true, false);
        assert_eq!(
            received.rows[0]
                .choices
                .iter()
                .map(|c| c.label.as_str())
                .collect::<Vec<_>>(),
            ["Accept", "Decline"]
        );
        let accept = received.rows[0].choices[0].intent.clone();
        authority
            .request(
                key(2),
                party::Request {
                    sequence: 1,
                    revision: accept.revision,
                    action: accept.action,
                },
            )
            .unwrap();
        let leader = model(&state(&authority, 1), true, false);
        assert_eq!(
            leader.rows[1]
                .choices
                .iter()
                .map(|c| c.label.as_str())
                .collect::<Vec<_>>(),
            ["Promote", "Remove"]
        );
        let member = model(&state(&authority, 2), true, false);
        assert!(member.rows.iter().all(|r| r.choices.is_empty()));
        assert!(matches!(member.leave.unwrap().action, Action::Leave));
        assert_ne!(leader.rows[1].choices[0].intent.revision, painted.revision);
        assert_eq!(
            authority.request(
                key(1),
                party::Request {
                    sequence: 2,
                    revision: painted.revision,
                    action: painted.action
                }
            ),
            Err(Refusal::Stale)
        );
        assert!(!model(&state(&authority, 1), false, false).enabled);
        assert!(
            model(&state(&authority, 1), true, true)
                .status
                .contains("No reply")
        );
    }
    #[test]
    fn party_mode_requires_explicit_classic_flag_and_changes_only_its_bound_identity() {
        let mut args = vec!["--parties".to_owned(), "--renet-connect".to_owned()];
        assert!(parse(&mut args, true).unwrap());
        assert_eq!(args, ["--renet-connect"]);
        assert!(!parse(&mut args, true).unwrap());
        assert!(parse(&mut vec!["--parties".into()], false).is_err());
        assert!(parse(&mut vec!["--parties".into(), "--parties".into()], true).is_err());
        assert_eq!(world_identity([1; 32], false), [1; 32]);
        assert_ne!(world_identity([1; 32], true), [1; 32]);
    }
}
