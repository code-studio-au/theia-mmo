//! Friendly target presentation consumes only the current private party view.
use bevy::{picking::Pickable, prelude::*, text::FontSize};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    resources::Vitals,
};
use theia_network::client::State;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Member {
    pub key: EntityKey,
    pub name: String,
    pub vitals: Option<Vitals>,
}
pub(crate) fn member(state: &State, key: EntityKey) -> Option<Member> {
    let welcome = state.welcome.as_ref()?;
    let targets = state.targeting.as_ref()?;
    let party = &state.party.as_ref()?.state;
    let owner = EntityKey {
        epoch: WorldEpoch::new(welcome.epoch)?,
        entity: EntityId::new(welcome.id)?,
    };
    if state.error.is_some()
        || state.id != Some(welcome.id)
        || targets.epoch != welcome.epoch
        || key.epoch != owner.epoch
        || key == owner
        || party.world != welcome.world
        || party.view.owner != owner
        || !party.view.membership.party.as_ref()?.members.contains(&key)
    {
        return None;
    }
    let name = party
        .roster
        .iter()
        .find(|p| p.key == key)?
        .name
        .as_str()
        .to_owned();
    let vitals = party.member_vitals.iter().find(|p| p.key == key)?.vitals;
    Some(Member { key, name, vitals })
}
pub(crate) fn selected(state: &State) -> Option<Member> {
    let targets = state.targeting.as_ref()?;
    if targets.selected.is_some() {
        return None;
    }
    member(
        state,
        EntityKey {
            epoch: WorldEpoch::new(targets.epoch)?,
            entity: targets.selected_member?,
        },
    )
}
#[derive(Clone, Copy)]
enum Pool {
    Health,
    Mana,
}
impl Pool {
    fn values(self, v: Vitals) -> (u32, u32) {
        match self {
            Self::Health => (v.health(), v.maximum_health()),
            Self::Mana => (v.mana(), v.maximum_mana()),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Health => "Health",
            Self::Mana => "Mana",
        }
    }
    fn label(self, v: Option<Vitals>) -> String {
        v.map_or_else(
            || format!("{} unavailable", self.name()),
            |v| {
                let (a, b) = self.values(v);
                format!("{} {a} / {b}", self.name())
            },
        )
    }
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Fill(Pool);
#[derive(Component)]
pub(super) struct Value(Pool);
pub(crate) fn widgets(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Root,
            Node {
                display: Display::None,
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            for pool in [Pool::Health, Pool::Mana] {
                root.spawn((
                    Value(pool),
                    Text::new(pool.label(None)),
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
                root.spawn((
                    Node {
                        width: percent(100),
                        height: px(match pool {
                            Pool::Health => 12.,
                            Pool::Mana => 8.,
                        }),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.08, 0.12, 0.18)),
                    Pickable::IGNORE,
                ))
                .with_child((
                    Fill(pool),
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(match pool {
                        Pool::Health => Color::srgb(0.2, 0.68, 0.37),
                        Pool::Mana => Color::srgb(0.18, 0.42, 0.92),
                    }),
                    Pickable::IGNORE,
                ));
            }
        });
}
#[derive(Resource, Default)]
pub(super) struct Presentation(Option<Member>);
pub(super) fn present(
    network: Res<crate::network::Network>,
    selection: Option<Res<crate::world_input::Selection>>,
    mut view: ResMut<Presentation>,
) {
    let state = network.state();
    let next = selection
        .as_ref()
        .is_none_or(|s| s.friendly.is_none())
        .then(|| selected(&state))
        .flatten();
    if view.0 != next {
        view.0 = next;
    }
}
pub(super) fn paint(
    view: Res<Presentation>,
    mut roots: Query<&mut Node, With<Root>>,
    mut fills: Query<(&Fill, &mut Node), Without<Root>>,
    mut labels: Query<(&Value, &mut Text)>,
    mut frames: Query<(&super::members::Frame, &mut BorderColor)>,
) {
    let member = &view.0;
    for mut node in &mut roots {
        let display = if member.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (fill, mut node) in &mut fills {
        let ratio = member.as_ref().and_then(|m| m.vitals).map_or(0., |v| {
            let (a, b) = fill.0.values(v);
            a as f32 / b as f32
        });
        let width = percent(ratio * 100.);
        if node.width != width {
            node.width = width;
        }
    }
    for (value, mut text) in &mut labels {
        let label = value.0.label(member.as_ref().and_then(|m| m.vitals));
        if text.0 != label {
            **text = label;
        }
    }
    for (frame, mut border) in &mut frames {
        let color = if member.as_ref().is_some_and(|m| m.key == frame.0) {
            Color::WHITE
        } else {
            crate::fantasy_ui::GOLD
        };
        let value = BorderColor::all(color);
        if *border != value {
            *border = value;
        }
    }
}
pub(super) fn snapshot(world: &mut World) -> serde_json::Value {
    let member = world.resource::<Presentation>().0.clone();
    let bounds = world
        .query::<(
            &crate::fantasy_ui::UnitFrame,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
        )>()
        .iter(world)
        .find(|(frame, node, _)| !frame.0 && member.is_some() && node.size().min_element() > 0.)
        .map(|(_, node, t)| {
            let factor =
                crate::fantasy_ui::window_factor(node, world.resource::<bevy::ui::UiScale>().0);
            let centre = t.affine().translation * factor;
            let size = node.size() * factor;
            [
                centre.x - size.x / 2.,
                centre.y - size.y / 2.,
                size.x,
                size.y,
            ]
        });
    let bars:Vec<_>=world.query::<(&Fill,&ComputedNode,&ChildOf)>().iter(world).filter_map(|(fill,node,parent)|{
        let parent=world.get::<ComputedNode>(parent.parent())?;
        (member.is_some()&&parent.size().x>0.).then(||serde_json::json!({"pool":fill.0.name().to_lowercase(),"width":node.size().x,"maximum_width":parent.size().x}))
    }).collect();
    let labels:Vec<_>=world.query::<(&Value,&Text,&ComputedNode,Option<&bevy::text::TextLayoutInfo>)>().iter(world).filter(|_|member.is_some()).map(|(_,text,node,glyphs)|serde_json::json!({"text":text.0,"rendered":node.size().min_element()>0.&&glyphs.is_some_and(|g|!g.glyphs.is_empty())})).collect();
    let headings:Vec<_>=world.query_filtered::<(&Text,&ComputedNode,Option<&bevy::text::TextLayoutInfo>),With<crate::targeting::TargetHud>>().iter(world).filter(|_|member.is_some()).map(|(text,node,glyphs)|serde_json::json!({"text":text.0,"rendered":node.size().min_element()>0.&&glyphs.is_some_and(|g|!g.glyphs.is_empty())})).collect();
    let highlighted: Vec<_> = world
        .query::<(&super::members::Frame, &BorderColor)>()
        .iter(world)
        .filter(|(_, b)| b.left == Color::WHITE)
        .map(|(f, _)| f.0.entity.value())
        .collect();
    serde_json::json!({"actor":member.as_ref().map(|m|m.key.entity.value()),"name":member.as_ref().map(|m|&m.name),"health":member.as_ref().and_then(|m|m.vitals).map(|v|[v.health(),v.maximum_health()]),"mana":member.as_ref().and_then(|m|m.vitals).map(|v|[v.mana(),v.maximum_mana()]),"bounds":bounds,"bars":bars,"labels":labels,"headings":headings,"highlighted":highlighted})
}

#[cfg(test)]
mod tests {
    use super::super::tests::{key, state as party_state};
    use super::*;
    use theia_core::{
        party::{self, Action, Request},
        time::TickRate,
    };
    fn fixture() -> State {
        let mut a = party::Authority::new(key(1).epoch, TickRate::Hz30);
        for id in 1..=3 {
            a.admit(key(id)).unwrap();
        }
        a.advance(1).unwrap();
        let v = a.view(key(1)).unwrap();
        a.request(
            key(1),
            Request {
                sequence: v.sequence + 1,
                revision: v.revision,
                action: Action::Invite(key(2)),
            },
        )
        .unwrap();
        let v = a.view(key(2)).unwrap();
        a.request(
            key(2),
            Request {
                sequence: v.sequence + 1,
                revision: v.revision,
                action: Action::Accept(v.membership.incoming.unwrap().id),
            },
        )
        .unwrap();
        State {
            id: Some(1),
            welcome: Some(theia_network::wire::Welcome {
                id: 1,
                epoch: 1,
                world: [1; 32],
                origin: [0.; 3],
                terrain_stream: true,
            }),
            party: Some(theia_network::party_client::View {
                state: std::sync::Arc::new(party_state(&a, 1)),
                submitted: 1,
                pending: false,
                timed_out: false,
            }),
            targeting: Some(theia_network::targeting_wire::State {
                epoch: 1,
                tick: 1,
                request_sequence: 1,
                accepted: true,
                selected: None,
                selected_member: Some(key(2).entity),
                targets: vec![],
            }),
            ..Default::default()
        }
    }
    #[test]
    fn member_display_requires_current_admission_membership_and_selected_identity() {
        let original = fixture();
        assert_eq!(selected(&original).unwrap().name, "Briar");
        assert!(selected(&original).unwrap().vitals.is_none());
        for case in 0..11 {
            let mut s = original.clone();
            match case {
                0 => s.error = Some("Disconnected".into()),
                1 => s.id = Some(9),
                2 => s.welcome.as_mut().unwrap().epoch += 1,
                3 => s.targeting.as_mut().unwrap().epoch += 1,
                4 => s.targeting.as_mut().unwrap().selected_member = Some(key(3).entity),
                5 => s.targeting.as_mut().unwrap().selected_member = Some(key(1).entity),
                6 => s.targeting.as_mut().unwrap().selected = Some(key(3).entity),
                7 => std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state).world = [2; 32],
                8 => {
                    std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state)
                        .view
                        .owner = key(2)
                }
                9 => {
                    std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state)
                        .view
                        .membership
                        .party = None
                }
                10 => std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state)
                    .roster
                    .clear(),
                _ => unreachable!(),
            }
            assert_eq!(selected(&s), None, "case {case}");
        }
    }
    #[test]
    fn member_widgets_repaint_partial_zero_unavailable_and_retire_without_rebuilding() {
        let mut m = selected(&fixture()).unwrap();
        let mut app = App::new();
        app.insert_resource(Presentation(Some(m.clone())))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Node::default()).with_children(widgets);
                commands.spawn((
                    super::super::members::Frame(key(2)),
                    BorderColor::all(crate::fantasy_ui::GOLD),
                ));
            })
            .add_systems(Update, paint);
        app.update();
        let entities: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<Value>>()
            .iter(app.world())
            .collect();
        for vitals in [
            Vitals::new(37, 100, 24, 100),
            Vitals::new(0, 100, 0, 100),
            None,
        ] {
            m.vitals = vitals;
            app.world_mut().resource_mut::<Presentation>().0 = Some(m.clone());
            app.update();
            for (fill, node) in app.world_mut().query::<(&Fill, &Node)>().iter(app.world()) {
                let ratio = vitals.map_or(0., |v| {
                    let (a, b) = fill.0.values(v);
                    a as f32 / b as f32
                });
                assert_eq!(node.width, percent(ratio * 100.));
            }
            for (value, text) in app.world_mut().query::<(&Value, &Text)>().iter(app.world()) {
                assert_eq!(text.0, value.0.label(vitals));
            }
            assert_eq!(
                app.world_mut()
                    .query_filtered::<Entity, With<Value>>()
                    .iter(app.world())
                    .collect::<Vec<_>>(),
                entities
            );
        }
        app.world_mut().resource_mut::<Presentation>().0 = None;
        app.update();
        assert!(
            app.world_mut()
                .query_filtered::<&Node, With<Root>>()
                .iter(app.world())
                .all(|n| n.display == Display::None)
        );
        assert!(
            app.world_mut()
                .query_filtered::<&BorderColor, With<super::super::members::Frame>>()
                .iter(app.world())
                .all(|b| b.left == crate::fantasy_ui::GOLD)
        );
    }
}
