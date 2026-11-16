//! Compact party frames. Identity/layout changes rebuild; resource updates paint
//! the existing widgets. No inferred portraits, levels or client-owned stats.
use super::{Control, button};
use bevy::{picking::Pickable, prelude::*, text::FontSize};
use theia_core::{identity::EntityKey, resources::Vitals};
use theia_network::party_wire::{MemberVitals, State};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    key: EntityKey,
    name: String,
    leader: bool,
}
fn members(state: &State) -> Vec<Member> {
    state
        .view
        .membership
        .party
        .as_ref()
        .map_or_else(Vec::new, |party| {
            party
                .members
                .iter()
                .filter(|key| **key != state.view.owner)
                .filter_map(|key| {
                    let actor = state.roster.iter().find(|actor| actor.key == *key)?;
                    Some(Member {
                        key: *key,
                        name: actor.name.as_str().to_owned(),
                        leader: party.leader == *key,
                    })
                })
                .collect()
        })
}
#[derive(Resource, Default)]
pub(super) struct Presentation {
    members: Vec<Member>,
    vitals: Vec<MemberVitals>,
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Frame(pub(super) EntityKey);
#[derive(Component)]
struct IdentityLabel(EntityKey);
#[derive(Clone, Copy)]
enum Pool {
    Health,
    Mana,
}
impl Pool {
    fn value(self, vitals: Vitals) -> (u32, u32) {
        match self {
            Self::Health => (vitals.health(), vitals.maximum_health()),
            Self::Mana => (vitals.mana(), vitals.maximum_mana()),
        }
    }
    fn label(self, vitals: Option<Vitals>) -> String {
        let name = match self {
            Self::Health => "Health",
            Self::Mana => "Mana",
        };
        vitals.map_or_else(
            || format!("{name} unavailable"),
            |v| {
                let (current, maximum) = self.value(v);
                format!("{name} {current} / {maximum}")
            },
        )
    }
}
#[derive(Component)]
pub(super) struct Fill(EntityKey, Pool);
#[derive(Component)]
pub(super) struct ValueLabel(EntityKey, Pool);
fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    )
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<Presentation>()
        .add_systems(Startup, setup);
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Root,
        GlobalZIndex(32),
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(96),
            width: px(268),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            ..default()
        },
        Pickable::IGNORE,
    ));
}
fn children(root: &mut ChildSpawnerCommands, members: &[Member]) {
    root.spawn(button(Control::Toggle, "Manage party", true))
        .insert(Node {
            width: percent(100),
            height: px(26),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(8)),
            ..default()
        })
        .with_children(|header| {
            header.spawn(text(
                format!("Party · {} / 5", members.len() + 1),
                14.,
                crate::fantasy_ui::GOLD,
            ));
            header.spawn(text("Manage [O]", 12., Color::WHITE));
        });
    for member in members {
        root.spawn((
            Frame(member.key),
            AccessibleLabel::new(format!("Party member {}", member.name)),
            Node {
                width: percent(100),
                padding: UiRect::all(px(7)),
                column_gap: px(8),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(crate::fantasy_ui::PANEL),
            BorderColor::all(crate::fantasy_ui::GOLD),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    width: px(32),
                    height: px(32),
                    flex_shrink: 0.,
                    border_radius: BorderRadius::MAX,
                    border: UiRect::all(px(1)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.16, 0.21)),
                BorderColor::all(crate::fantasy_ui::GOLD),
                Pickable::IGNORE,
            ))
            .with_child(text(
                member.name.chars().next().unwrap_or('?').to_string(),
                20.,
                crate::fantasy_ui::GOLD,
            ));
            row.spawn((
                Node {
                    width: px(204),
                    flex_shrink: 0.,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                body.spawn((
                    Node {
                        width: percent(100),
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: px(4),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|heading| {
                    heading.spawn((
                        IdentityLabel(member.key),
                        text(&member.name, 14., Color::WHITE),
                        Node {
                            width: px(if member.leader { 160. } else { 204. }),
                            flex_shrink: 0.,
                            ..default()
                        },
                        TextLayout::linebreak(bevy::text::LineBreak::AnyCharacter),
                    ));
                    if member.leader {
                        heading.spawn((
                            IdentityLabel(member.key),
                            text("Leader", 11., crate::fantasy_ui::GOLD),
                        ));
                    }
                });
                for (pool, color) in [
                    (Pool::Health, Color::srgb(0.16, 0.49, 0.25)),
                    (Pool::Mana, Color::srgb(0.13, 0.32, 0.65)),
                ] {
                    body.spawn((
                        Node {
                            width: percent(100),
                            height: px(17),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.035, 0.045, 0.06)),
                        Pickable::IGNORE,
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Fill(member.key, pool),
                            Node {
                                width: percent(0),
                                height: percent(100),
                                ..default()
                            },
                            BackgroundColor(color),
                            Pickable::IGNORE,
                        ));
                        bar.spawn((
                            ValueLabel(member.key, pool),
                            text(pool.label(None), 12., Color::WHITE),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(4),
                                top: px(0),
                                ..default()
                            },
                        ));
                    });
                }
            });
        });
    }
}
pub(super) fn present(
    mut commands: Commands,
    network: Res<crate::network::Network>,
    dialogue: Option<Res<crate::dialogue::Ui>>,
    mut presentation: ResMut<Presentation>,
    mut roots: Query<(Entity, &mut Node), With<Root>>,
) {
    let state = network.state();
    let party = state.party.as_ref().filter(|_| state.error.is_none());
    let next = party.map_or_else(Vec::new, |view| members(&view.state));
    presentation.vitals = party.map_or_else(Vec::new, |view| view.state.member_vitals.clone());
    for (entity, mut node) in &mut roots {
        let display = if next.is_empty() {
            Display::None
        } else {
            Display::Flex
        };
        let left = px(if dialogue.as_ref().is_some_and(|d| d.open) {
            444.
        } else {
            12.
        });
        if node.display != display {
            node.display = display;
        }
        if node.left != left {
            node.left = left;
        }
        if presentation.members != next {
            commands
                .entity(entity)
                .despawn_children()
                .with_children(|root| children(root, &next));
        }
    }
    presentation.members = next;
}
pub(super) fn paint(
    presentation: Res<Presentation>,
    mut fills: Query<(&Fill, &mut Node)>,
    mut labels: Query<(&ValueLabel, &mut Text)>,
) {
    let value = |key| {
        presentation
            .vitals
            .iter()
            .find(|v| v.key == key)
            .and_then(|v| v.vitals)
    };
    for (fill, mut node) in &mut fills {
        let width = percent(value(fill.0).map_or(0., |v| {
            let (current, maximum) = fill.1.value(v);
            current as f32 / maximum as f32 * 100.
        }));
        if node.width != width {
            node.width = width;
        }
    }
    for (label, mut text) in &mut labels {
        let next = label.1.label(value(label.0));
        if text.0 != next {
            text.0 = next;
        }
    }
}
pub(super) fn snapshot(world: &mut World) -> serde_json::Value {
    let presentation = world.resource::<Presentation>();
    let members = presentation.members.clone();
    let vitals = presentation.vitals.clone();
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let frames: Vec<_> = world.query::<(&Frame, &ComputedNode, &bevy::ui::UiGlobalTransform)>().iter(world)
        .filter(|(frame, node, _)| node.size().min_element() > 0. && members.iter().any(|m| m.key == frame.0))
        .map(|(frame, node, transform)| {
            let factor = crate::fantasy_ui::window_factor(node, scale);
            let centre = transform.affine().translation * factor;
            let size = node.size() * factor;
            let member = members.iter().find(|m| m.key == frame.0).unwrap();
            let value = vitals.iter().find(|v| v.key == frame.0).and_then(|v| v.vitals);
            serde_json::json!({"actor":frame.0.entity.value(),"name":member.name,"leader":member.leader,
                "health":value.map(|v| [v.health(),v.maximum_health()]),"mana":value.map(|v| [v.mana(),v.maximum_mana()]),
                "bounds":[centre.x-size.x/2.,centre.y-size.y/2.,size.x,size.y]})
        }).collect();
    let bars: Vec<_> = world.query::<(&Fill, &ComputedNode, &ChildOf)>().iter(world)
        .filter_map(|(fill, node, parent)| {
            let parent = world.get::<ComputedNode>(parent.parent())?;
            (parent.size().x > 0. && members.iter().any(|m| m.key == fill.0)).then(|| serde_json::json!({
                "actor":fill.0.entity.value(), "pool":match fill.1 { Pool::Health=>"health", Pool::Mana=>"mana" },
                "width":node.size().x,"maximum_width":parent.size().x
            }))
        }).collect();
    let labels: Vec<_> = world.query::<(&ValueLabel, &Text, &ComputedNode, Option<&bevy::text::TextLayoutInfo>)>().iter(world)
        .filter(|(label, _, _, _)| members.iter().any(|m| m.key == label.0))
        .map(|(label, text, node, glyphs)| serde_json::json!({
            "actor":label.0.entity.value(),"text":text.0,
            "rendered":node.size().min_element()>0. && glyphs.is_some_and(|g|!g.glyphs.is_empty())
        })).collect();
    let identities: Vec<_> = world.query::<(&IdentityLabel, &Text, &ComputedNode, Option<&bevy::text::TextLayoutInfo>)>().iter(world)
        .filter(|(label, _, _, _)| members.iter().any(|m| m.key == label.0))
        .map(|(label, text, node, glyphs)| serde_json::json!({
            "actor":label.0.entity.value(),"text":text.0,
            "rendered":node.size().min_element()>0. && glyphs.is_some_and(|g|!g.glyphs.is_empty())
        })).collect();
    serde_json::json!({"visible":!members.is_empty(),"frames":frames,"bars":bars,"labels":labels,"identities":identities})
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        character_name::CharacterName,
        identity::{EntityId, WorldEpoch},
        party::{self, Action, Request},
        time::TickRate,
    };
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn send(authority: &mut party::Authority, id: u64, action: Action) {
        let view = authority.view(key(id)).unwrap();
        authority
            .request(
                key(id),
                Request {
                    sequence: view.sequence + 1,
                    revision: view.revision,
                    action,
                },
            )
            .unwrap();
    }
    fn state(authority: &party::Authority) -> State {
        let view = authority.view(key(1)).unwrap();
        let member_vitals = view.membership.party.as_ref().map_or_else(Vec::new, |p| {
            p.members
                .iter()
                .map(|key| MemberVitals {
                    key: *key,
                    vitals: Vitals::new(37, 100, 24, 100),
                })
                .collect()
        });
        State {
            world: [1; 32],
            tick: 1,
            view,
            refusal: None,
            member_vitals,
            roster: (1..=5)
                .map(|id| theia_network::party_wire::NamedActor {
                    key: key(id),
                    name: CharacterName::parse(
                        ["Arden", "Briar", "Cedar", "Dawn", "Abcdefghijklmnopqrst"]
                            [(id - 1) as usize],
                    )
                    .unwrap(),
                })
                .collect(),
        }
    }
    fn grouped() -> party::Authority {
        let mut authority = party::Authority::new(key(1).epoch, TickRate::Hz30);
        for id in 1..=5 {
            authority.admit(key(id)).unwrap();
        }
        authority.advance(1).unwrap();
        for id in 2..=5 {
            send(&mut authority, 1, Action::Invite(key(id)));
            let invitation = authority
                .view(key(id))
                .unwrap()
                .membership
                .incoming
                .unwrap()
                .id;
            send(&mut authority, id, Action::Accept(invitation));
        }
        authority.advance(100).unwrap();
        authority
    }
    #[test]
    fn frames_exclude_self_and_follow_leader_removal_and_dissolution() {
        let mut authority = grouped();
        let before = members(&state(&authority));
        assert_eq!(
            before.iter().map(|m| m.key).collect::<Vec<_>>(),
            (2..=5).map(key).collect::<Vec<_>>()
        );
        assert!(before.iter().all(|m| !m.leader));
        send(&mut authority, 1, Action::Promote(key(2)));
        let promoted = members(&state(&authority));
        assert!(promoted[0].leader);
        assert_eq!(promoted[3].name, "Abcdefghijklmnopqrst");
        send(&mut authority, 2, Action::Kick(key(3)));
        assert!(!members(&state(&authority)).iter().any(|m| m.key == key(3)));
        send(&mut authority, 1, Action::Leave);
        assert!(members(&state(&authority)).is_empty());
    }
    #[test]
    fn four_frames_paint_live_zero_and_unavailable_pools_without_rebuilding_widgets() {
        let state = state(&grouped());
        let mut app = App::new();
        app.insert_resource(Presentation {
            members: members(&state),
            vitals: state.member_vitals,
        })
        .add_systems(
            Startup,
            |mut commands: Commands, presentation: Res<Presentation>| {
                commands
                    .spawn(Node::default())
                    .with_children(|root| children(root, &presentation.members));
            },
        )
        .add_systems(Update, paint);
        app.update();
        let frames = app
            .world_mut()
            .query_filtered::<Entity, With<Frame>>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert_eq!(frames.len(), 4);
        assert_eq!(
            app.world_mut().query::<&Fill>().iter(app.world()).count(),
            8
        );
        for (fill, node) in app.world_mut().query::<(&Fill, &Node)>().iter(app.world()) {
            assert_eq!(
                node.width,
                percent(match fill.1 {
                    Pool::Health => 37.,
                    Pool::Mana => 24.,
                })
            );
        }
        {
            let mut presentation = app.world_mut().resource_mut::<Presentation>();
            presentation.vitals[1].vitals = Vitals::new(0, 100, 0, 100);
            presentation.vitals[2].vitals = None;
        }
        app.update();
        assert_eq!(
            frames,
            app.world_mut()
                .query_filtered::<Entity, With<Frame>>()
                .iter(app.world())
                .collect::<Vec<_>>()
        );
        for (label, text) in app
            .world_mut()
            .query::<(&ValueLabel, &Text)>()
            .iter(app.world())
        {
            if label.0 == key(2) {
                assert!(text.0.ends_with("0 / 100"));
            }
            if label.0 == key(3) {
                assert!(text.0.ends_with("unavailable"));
            }
        }
        for (fill, node) in app.world_mut().query::<(&Fill, &Node)>().iter(app.world()) {
            if fill.0 == key(2) || fill.0 == key(3) {
                assert_eq!(node.width, percent(0));
            }
        }
    }
}
