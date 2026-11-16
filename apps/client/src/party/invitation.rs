//! A nonmodal invitation prompt backed by the same private party state.
use super::{Control, Ui, button, intent, label};
use bevy::{picking::Pickable, prelude::*};
use theia_core::party::Action;
use theia_network::{party_client::Intent, party_wire::State};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Model {
    inviter: String,
    accept: Intent,
    decline: Intent,
    enabled: bool,
}
fn model(state: &State, enabled: bool) -> Option<Model> {
    let incoming = state.view.membership.incoming.as_ref()?;
    let inviter = state.roster.iter().find(|a| a.key == incoming.inviter)?;
    Some(Model {
        inviter: inviter.name.as_str().to_owned(),
        accept: intent(state, Action::Accept(incoming.id)),
        decline: intent(state, Action::Decline(incoming.id)),
        enabled,
    })
}
pub(super) fn current(state: &State, painted: &Intent) -> bool {
    painted.owner == state.view.owner
        && painted.revision == state.view.revision
        && state.view.membership.incoming.as_ref().is_some_and(
            |i| matches!(painted.action, Action::Accept(id) | Action::Decline(id) if id == i.id),
        )
}
#[derive(Resource, Default)]
pub(super) struct Presentation {
    model: Option<Model>,
    visible: bool,
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
struct Panel;

pub(super) fn install(app: &mut App) {
    app.init_resource::<Presentation>()
        .add_systems(Startup, setup);
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Root,
        GlobalZIndex(84),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(72),
            display: Display::None,
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));
}
pub(super) fn present(
    mut commands: Commands,
    network: Res<crate::network::Network>,
    ui: Res<Ui>,
    mut presentation: ResMut<Presentation>,
    mut roots: Query<(Entity, &mut Node), With<Root>>,
) {
    let state = network.state();
    let next = state
        .party
        .as_ref()
        .filter(|_| state.error.is_none())
        .and_then(|view| model(&view.state, !view.pending && ui.waiting.is_none()));
    presentation.visible = !ui.open && next.is_some();
    for (entity, mut node) in &mut roots {
        node.display = if presentation.visible {
            Display::Flex
        } else {
            Display::None
        };
        if presentation.model == next {
            continue;
        }
        let mut root = commands.entity(entity);
        root.despawn_children();
        if let Some(invitation) = &next {
            root.with_children(|root| {
                root.spawn((
                    Panel,
                    Node {
                        width: px(460),
                        max_width: percent(90),
                        padding: UiRect::all(px(14)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(10),
                        ..default()
                    },
                    BackgroundColor(crate::fantasy_ui::PANEL),
                    Outline::new(px(1), px(-1), crate::fantasy_ui::GOLD),
                ))
                .with_children(|panel| {
                    panel.spawn(label(&format!(
                        "{} invites you to a party.",
                        invitation.inviter
                    )));
                    panel
                        .spawn((
                            Node {
                                column_gap: px(14),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|row| {
                            for (title, command) in [
                                ("Accept", &invitation.accept),
                                ("Decline", &invitation.decline),
                            ] {
                                row.spawn(button(
                                    Control::Invitation(command.clone()),
                                    title,
                                    invitation.enabled,
                                ))
                                .insert(Node {
                                    min_width: px(140),
                                    min_height: px(34),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                })
                                .with_child(label(title));
                            }
                        });
                    if !invitation.enabled {
                        panel.spawn(label("Waiting for a party response…"));
                    }
                });
            });
        }
    }
    presentation.model = next;
}
pub(super) fn snapshot(world: &mut World) -> serde_json::Value {
    let visible = world.resource::<Presentation>().visible;
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let bounds = world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Panel>>()
        .iter(world)
        .find(|(node, _)| visible && node.size().min_element() > 0.)
        .map(|(node, transform)| {
            let factor = crate::fantasy_ui::window_factor(node, scale);
            let centre = transform.affine().translation * factor;
            let size = node.size() * factor;
            [
                centre.x - size.x / 2.,
                centre.y - size.y / 2.,
                size.x,
                size.y,
            ]
        });
    serde_json::json!({"visible":visible,"bounds":bounds})
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        identity::{EntityId, EntityKey, WorldEpoch},
        party,
        time::TickRate,
    };
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn state(authority: &party::Authority, id: u64) -> State {
        State {
            world: [1; 32],
            tick: 1,
            view: authority.view(key(id)).unwrap(),
            refusal: None,
            roster: vec![theia_network::party_wire::NamedActor {
                key: key(1),
                name: theia_core::character_name::CharacterName::parse("Arden").unwrap(),
            }],
        }
    }
    #[test]
    fn cancelled_replaced_and_expired_invites_cannot_reuse_painted_commands() {
        let mut authority = party::Authority::new(key(1).epoch, TickRate::Hz30);
        for id in [1, 2] {
            authority.admit(key(id)).unwrap();
        }
        authority.advance(1).unwrap();
        authority
            .request(
                key(1),
                party::Request {
                    sequence: 1,
                    revision: state(&authority, 1).view.revision,
                    action: Action::Invite(key(2)),
                },
            )
            .unwrap();
        let received = state(&authority, 2);
        let prompt = model(&received, true).unwrap();
        assert_eq!(prompt.inviter, "Arden");
        assert!(current(&received, &prompt.accept));
        assert!(current(&received, &prompt.decline));
        assert!(!model(&received, false).unwrap().enabled);
        let mut foreign = prompt.accept.clone();
        foreign.owner = key(1);
        assert!(!current(&received, &foreign));
        let mut unnamed = received.clone();
        unnamed.roster.clear();
        assert!(model(&unnamed, true).is_none());
        let id = received.view.membership.incoming.unwrap().id;
        authority
            .request(
                key(1),
                party::Request {
                    sequence: 2,
                    revision: state(&authority, 1).view.revision,
                    action: Action::Cancel(id),
                },
            )
            .unwrap();
        let cancelled = state(&authority, 2);
        assert!(model(&cancelled, true).is_none());
        assert!(!current(&cancelled, &prompt.accept));
        authority.advance(100).unwrap();
        authority
            .request(
                key(1),
                party::Request {
                    sequence: 3,
                    revision: state(&authority, 1).view.revision,
                    action: Action::Invite(key(2)),
                },
            )
            .unwrap();
        let replacement = state(&authority, 2);
        let next = model(&replacement, true).unwrap();
        assert!(!current(&replacement, &prompt.accept));
        assert!(current(&replacement, &next.accept));
        authority.advance(2000).unwrap();
        let expired = state(&authority, 2);
        assert!(model(&expired, true).is_none());
        assert!(!current(&expired, &next.accept));
    }
}
