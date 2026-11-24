//! Shared symbols consume only private authoritative party marks.
mod icons;
use super::{Control, Ui, button, intent, label};
use bevy::{picking::Pickable, prelude::*};
use theia_core::{
    identity::{EntityKey, WorldEpoch},
    party::{Action, Party, marks::Symbol},
};
use theia_network::{client::State, party_client::Intent};

fn party(state: &State) -> Option<&Party> {
    let welcome = state.welcome.as_ref()?;
    let view = &state.party.as_ref()?.state;
    (state.error.is_none()
        && state.id == Some(welcome.id)
        && view.world == welcome.world
        && view.view.owner.epoch.value() == welcome.epoch
        && view.view.owner.entity.value() == welcome.id)
        .then_some(())?;
    view.view.membership.party.as_ref()
}
fn npc(state: &State, key: EntityKey) -> bool {
    state
        .guide
        .as_ref()
        .filter(|p| p.actor == state.party.as_ref().unwrap().state.view.owner)
        .is_some_and(|p| {
            p.guide.is_some_and(|g| g.key == key) || p.contacts.iter().any(|c| c.guide.key == key)
        })
}
fn selected(state: &State, friendly: Option<EntityKey>) -> Option<EntityKey> {
    let group = party(state)?;
    let epoch = WorldEpoch::new(state.welcome.as_ref()?.epoch)?;
    if let Some(key) = friendly {
        return (key.epoch == epoch && npc(state, key)).then_some(key);
    }
    let targets = state
        .targeting
        .as_ref()
        .filter(|v| v.epoch == epoch.value())?;
    match (targets.selected, targets.selected_member) {
        (Some(entity), None) => targets
            .targets
            .iter()
            .find(|t| t.key.entity == entity && t.key.epoch == epoch)
            .map(|t| t.key),
        (None, Some(entity)) => group
            .members
            .iter()
            .find(|k| k.entity == entity && k.epoch == epoch)
            .copied(),
        _ => None,
    }
}
fn current_selection(
    state: &State,
    friendly: Option<EntityKey>,
    network: &crate::network::Network,
) -> Option<EntityKey> {
    selected(state, friendly)
        .filter(|k| friendly == Some(*k) || network.target_intent_matches(k.entity))
}
fn allowed(state: &State, selected: Option<EntityKey>, painted: &Intent) -> bool {
    let Some(group) = party(state) else {
        return false;
    };
    let view = &state.party.as_ref().unwrap().state.view;
    painted.owner == view.owner
        && painted.revision == view.revision
        && group.leader == view.owner
        && match painted.action {
            Action::Mark { target, symbol } => {
                selected == Some(target)
                    && (symbol.is_some() || group.marks.iter().any(|m| m.target == target))
            }
            Action::ClearMarks => !group.marks.is_empty(),
            _ => false,
        }
}
pub(super) fn current(
    state: &State,
    friendly: Option<EntityKey>,
    network: &crate::network::Network,
    painted: &Intent,
) -> bool {
    allowed(state, current_selection(state, friendly, network), painted)
}
#[derive(Resource)]
pub(super) struct Icons([Handle<Image>; 8]);
impl Icons {
    fn get(&self, s: Symbol) -> Handle<Image> {
        self.0[s as usize - 1].clone()
    }
}
#[derive(Component)]
pub(super) struct Palette;
#[derive(Component)]
struct Overhead(Symbol);
#[derive(Component)]
pub(super) struct TargetIcon;
#[derive(Resource, Default)]
pub(super) struct Presentation {
    model: Option<Model>,
    target: Option<Symbol>,
}
#[derive(Clone, PartialEq, Eq)]
struct Model {
    owner: EntityKey,
    revision: u64,
    target: Option<EntityKey>,
    caption: String,
    status: String,
    enabled: bool,
    marked: Option<Symbol>,
    any: bool,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Presentation>()
        .add_systems(Startup, setup)
        .add_systems(
            PostUpdate,
            project
                .after(crate::camera::sync_camera_global)
                .after(avian3d::prelude::PhysicsSystems::StepSimulation)
                .before(bevy::ui::UiSystems::Layout),
        );
}
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icons = Icons(Symbol::ALL.map(|s| images.add(icons::image(s))));
    for s in Symbol::ALL {
        commands.spawn((
            Overhead(s),
            ImageNode::new(icons.get(s)),
            AccessibleLabel::new(format!("{} party mark", s.name())),
            Node {
                position_type: PositionType::Absolute,
                width: px(32),
                height: px(32),
                display: Display::None,
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(2),
        ));
    }
    commands.insert_resource(icons);
    commands.spawn((
        Palette,
        Node {
            position_type: PositionType::Absolute,
            left: px(292),
            top: px(96),
            width: px(244),
            max_height: percent(65),
            padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            overflow: Overflow::scroll_y(),
            display: Display::None,
            ..default()
        },
        BackgroundColor(crate::fantasy_ui::PANEL),
        Outline::new(px(1), px(-1), crate::fantasy_ui::GOLD),
        GlobalZIndex(84),
    ));
}
pub(crate) fn target_widget(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        TargetIcon,
        ImageNode::default(),
        Node {
            position_type: PositionType::Absolute,
            right: px(-40),
            top: px(0),
            width: px(32),
            height: px(32),
            display: Display::None,
            ..default()
        },
        Pickable::IGNORE,
    ));
}
type TargetFilter = (With<TargetIcon>, Without<Palette>);
#[allow(clippy::too_many_arguments)]
pub(super) fn present(
    mut commands: Commands,
    network: Res<crate::network::Network>,
    selection: Option<Res<crate::world_input::Selection>>,
    mut ui: ResMut<Ui>,
    icons: Res<Icons>,
    config: Option<Res<crate::dialogue::Config>>,
    mut presentation: ResMut<Presentation>,
    mut roots: Query<(Entity, &mut Node), With<Palette>>,
    mut target_icons: Query<(&mut Node, &mut ImageNode), TargetFilter>,
) {
    let state = network.state();
    let group = party(&state);
    let target = current_selection(
        &state,
        selection.as_ref().and_then(|s| s.friendly),
        &network,
    );
    let symbol = group
        .and_then(|p| p.marks.iter().find(|m| Some(m.target) == target))
        .map(|m| m.symbol);
    for (mut node, mut image) in &mut target_icons {
        node.display = if symbol.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(symbol) = symbol {
            image.image = icons.get(symbol);
        }
    }
    presentation.target = symbol;
    if group.is_none() {
        ui.marks_open = false;
    }
    let model = group.map(|g| {
        let view = state.party.as_ref().unwrap();
        let caption = target.map_or_else(
            || "Select a unit to mark".to_owned(),
            |k| {
                view.state.roster.iter().find(|p| p.key == k).map_or_else(
                    || {
                        if npc(&state, k) {
                            state
                                .guide
                                .as_ref()
                                .and_then(|p| p.npc(k))
                                .and_then(|id| {
                                    config
                                        .as_ref()?
                                        .0
                                        .contacts()
                                        .find(|c| c.npc() == id)
                                        .map(|c| c.name().to_owned())
                                })
                                .unwrap_or_else(|| "Selected NPC".to_owned())
                        } else {
                            "Selected enemy".to_owned()
                        }
                    },
                    |p| p.name.as_str().to_owned(),
                )
            },
        );
        let leader = g.leader == view.state.view.owner;
        let status = if !leader {
            "Only the leader can change marks.".to_owned()
        } else {
            super::model(
                &view.state,
                !view.pending && ui.waiting.is_none(),
                view.timed_out,
            )
            .status
        };
        Model {
            owner: view.state.view.owner,
            revision: view.state.view.revision,
            target,
            caption,
            status,
            enabled: leader && !view.pending && ui.waiting.is_none(),
            marked: symbol,
            any: !g.marks.is_empty(),
        }
    });
    for (entity, mut node) in &mut roots {
        node.display = if ui.marks_open {
            Display::Flex
        } else {
            Display::None
        };
        if model == presentation.model {
            continue;
        }
        commands
            .entity(entity)
            .despawn_children()
            .with_children(|pane| {
                let Some(m) = &model else {
                    return;
                };
                pane.spawn(label("Unit marks"))
                    .insert(TextColor(crate::fantasy_ui::GOLD));
                pane.spawn(label(&m.caption));
                pane.spawn(label(&m.status)).insert(TextFont {
                    font_size: bevy::text::FontSize::Px(12.),
                    ..default()
                });
                for row in Symbol::ALL.chunks(4) {
                    pane.spawn((
                        Node {
                            column_gap: px(4),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|line| {
                        for &s in row {
                            let enabled = m.enabled && m.target.is_some();
                            let control = m.target.map(|target| {
                                Control::Mark(Intent {
                                    owner: m.owner,
                                    revision: m.revision,
                                    action: Action::Mark {
                                        target,
                                        symbol: Some(s),
                                    },
                                })
                            });
                            let mut cell = line.spawn((
                                Node {
                                    width: px(51),
                                    height: px(62),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::all(px(1)),
                                    ..default()
                                },
                                BorderColor::all(if m.marked == Some(s) {
                                    Color::WHITE
                                } else {
                                    crate::fantasy_ui::GOLD
                                }),
                                BackgroundColor(if enabled {
                                    Color::srgb(0.15, 0.19, 0.25)
                                } else {
                                    Color::srgb(0.06, 0.07, 0.09)
                                }),
                            ));
                            if let Some(control) = control {
                                cell.insert((
                                    control,
                                    bevy::ui_widgets::Button,
                                    bevy::picking::hover::Hovered::default(),
                                    AccessibleLabel::new(format!("Mark {}", s.name())),
                                ));
                            }
                            cell.with_child((
                                ImageNode {
                                    color: Color::srgba(
                                        1.,
                                        1.,
                                        1.,
                                        if enabled { 1. } else { 0.45 },
                                    ),
                                    ..ImageNode::new(icons.get(s))
                                },
                                Node {
                                    width: px(30),
                                    height: px(30),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                            cell.with_children(|cell| {
                                cell.spawn(label(s.name())).insert(TextFont {
                                    font_size: bevy::text::FontSize::Px(11.),
                                    ..default()
                                });
                            });
                        }
                    });
                }
                if let Some(target) = m.target {
                    pane.spawn(button(
                        Control::Mark(Intent {
                            owner: m.owner,
                            revision: m.revision,
                            action: Action::Mark {
                                target,
                                symbol: None,
                            },
                        }),
                        "Clear unit mark",
                        m.enabled && m.marked.is_some(),
                    ))
                    .with_child(label("Clear unit"));
                }
                let view = &state.party.as_ref().unwrap().state;
                pane.spawn(button(
                    Control::Mark(intent(view, Action::ClearMarks)),
                    "Clear all marks",
                    m.enabled && m.any,
                ))
                .with_child(label("Clear all"));
                pane.spawn(button(Control::MarksToggle, "Close unit marks", true))
                    .with_child(label("Close [Esc]"));
            });
    }
    presentation.model = model;
}

type Visual<'a> = (
    &'a Transform,
    &'a Visibility,
    Option<&'a crate::targeting::TargetVisual>,
    Option<&'a crate::dialogue::ContactKey>,
    Option<&'a crate::network::RemotePawn>,
    Has<crate::movement::LocalPawn>,
);
fn observed(state: &State, key: EntityKey) -> bool {
    let Some(group) = party(state) else {
        return false;
    };
    if key.epoch.value() != state.welcome.as_ref().unwrap().epoch {
        return false;
    }
    let position = if group.members.contains(&key) {
        state
            .snapshot
            .as_ref()
            .filter(|s| s.epoch == key.epoch.value())
            .and_then(|s| {
                s.players
                    .iter()
                    .find(|p| p.id == key.entity.value())
                    .and_then(|p| s.global_position(p))
            })
    } else if npc(state, key) {
        state
            .guide
            .as_ref()
            .and_then(|p| {
                p.guide.filter(|g| g.key == key).or_else(|| {
                    p.contacts
                        .iter()
                        .find(|c| c.guide.key == key)
                        .map(|c| c.guide)
                })
            })
            .map(|g| g.position)
    } else {
        state
            .targeting
            .as_ref()
            .filter(|v| v.epoch == key.epoch.value())
            .and_then(|v| v.targets.iter().find(|t| t.key == key))
            .map(|t| t.position)
    };
    position.and_then(|p| p.chunk(128)).is_some_and(|c| {
        state.chunk_report.as_ref().is_some_and(|r| {
            r.epoch == key.epoch.value() && r.valid() && r.chunks.contains(&[c.x, c.z])
        })
    })
}
#[allow(clippy::too_many_arguments)]
fn project(
    network: Res<crate::network::Network>,
    cameras: Query<(&Camera, &GlobalTransform), crate::frontend::WorldCamera>,
    scale: Res<bevy::ui::UiScale>,
    spatial: avian3d::prelude::SpatialQuery,
    visuals: Query<Visual<'_>>,
    mut markers: Query<(&Overhead, &mut Node)>,
) {
    let state = network.state();
    for (marker, mut node) in &mut markers {
        node.display = Display::None;
        let Some(mark) = party(&state).and_then(|p| p.marks.iter().find(|m| m.symbol == marker.0))
        else {
            continue;
        };
        if !observed(&state, mark.target) {
            continue;
        }
        let owner = state.party.as_ref().unwrap().state.view.owner;
        let Some((transform, visibility, _, contact, _, _)) =
            visuals
                .iter()
                .find(|(_, _, target, contact, remote, local)| {
                    target.is_some_and(|t| t.key == mark.target)
                        || contact.is_some_and(|c| c.0 == Some(mark.target))
                        || remote.is_some_and(|p| p.actor_id() == mark.target.entity.value())
                        || (*local && mark.target == owner)
                })
        else {
            continue;
        };
        if *visibility == Visibility::Hidden {
            continue;
        }
        let Ok((camera, eye)) = cameras.single() else {
            continue;
        };
        let point = transform.translation + Vec3::Y * if contact.is_some() { 1.9 } else { 1.4 };
        if point.distance(eye.translation()) > 60.
            || crate::quest_marker::occluded(&spatial, eye.translation(), point)
        {
            continue;
        }
        let Some(screen) = camera
            .world_to_viewport(eye, point)
            .ok()
            .and_then(|p| crate::fantasy_ui::projected_ui_point(p, scale.0))
        else {
            continue;
        };
        let Some(viewport) = camera
            .logical_viewport_size()
            .and_then(|p| crate::fantasy_ui::projected_ui_point(p, scale.0))
        else {
            continue;
        };
        let top = screen.y - if contact.is_some() { 104. } else { 40. };
        if screen.x < 16. || screen.x > viewport.x - 16. || top < 0. || top + 32. > viewport.y {
            continue;
        }
        node.left = px(screen.x - 16.);
        node.top = px(top);
        node.display = Display::Flex;
    }
}
pub(super) fn snapshot(world: &mut World) -> serde_json::Value {
    let ui = world.resource::<Ui>();
    let open = ui.marks_open;
    let target = world.resource::<Presentation>().target.map(Symbol::name);
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let bounds = world
        .query_filtered::<(&Node, &ComputedNode, &bevy::ui::UiGlobalTransform), With<Palette>>()
        .iter(world)
        .find(|(n, c, _)| n.display != Display::None && c.size().min_element() > 0.)
        .map(|(_, c, t)| {
            let factor = crate::fantasy_ui::window_factor(c, scale);
            let centre = t.affine().translation * factor;
            let size = c.size() * factor;
            [
                centre.x - size.x / 2.,
                centre.y - size.y / 2.,
                size.x,
                size.y,
            ]
        });
    let player_bounds = world
        .query::<(
            &crate::fantasy_ui::UnitFrame,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
        )>()
        .iter(world)
        .find(|(f, n, _)| f.0 && n.size().min_element() > 0.)
        .map(|(_, n, t)| {
            let factor = crate::fantasy_ui::window_factor(n, scale);
            let centre = t.affine().translation * factor;
            let size = n.size() * factor;
            [
                centre.x - size.x / 2.,
                centre.y - size.y / 2.,
                size.x,
                size.y,
            ]
        });
    let overhead:Vec<_>=world.query::<(&Overhead,&Node,&ComputedNode,&bevy::ui::UiGlobalTransform)>().iter(world).filter(|(_,n,c,_)|n.display!=Display::None && c.size().min_element()>0.).map(|(m,_,c,t)|serde_json::json!({"symbol":m.0.name(),"point":(t.affine().translation*crate::fantasy_ui::window_factor(c,scale)).to_array()})).collect();
    let target_rendered = world
        .query_filtered::<(&Node, &ComputedNode), With<TargetIcon>>()
        .iter(world)
        .any(|(n, c)| n.display != Display::None && c.size().min_element() > 0.);
    serde_json::json!({"open":open,"bounds":bounds,"player_bounds":player_bounds,"target_symbol":target,"target_rendered":target_rendered,"overhead":overhead})
}

#[cfg(test)]
mod tests {
    use super::super::tests::{key, state as party_state};
    use super::*;
    use theia_core::{
        coordinates::GlobalPosition,
        party::{Authority, Request},
        time::TickRate,
    };
    fn fixture() -> State {
        let mut authority = Authority::new(key(1).epoch, TickRate::Hz30);
        for id in 1..=3 {
            authority.admit(key(id)).unwrap();
        }
        for (id, action) in [(1, Action::Invite(key(2))), (2, Action::Accept(1))] {
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
                state: std::sync::Arc::new(party_state(&authority, 1)),
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
            guide: Some(theia_network::dialogue_wire::Presence {
                actor: key(1),
                profile_hash: [1; 32],
                contacts: vec![],
                guide: Some(theia_network::dialogue::Guide {
                    key: key(9),
                    position: GlobalPosition::new([5., 0.9, 0.]).unwrap(),
                }),
            }),
            chunk_report: Some(theia_network::wire::ChunkReady {
                version: theia_network::wire::VERSION,
                epoch: 1,
                sequence: 1,
                chunks: vec![[0, 0]],
            }),
            ..default()
        }
    }
    #[test]
    fn painted_marker_commands_reject_selection_revision_leadership_and_epoch_changes() {
        let original = fixture();
        let painted = intent(
            &original.party.as_ref().unwrap().state,
            Action::Mark {
                target: key(2),
                symbol: Some(Symbol::Skull),
            },
        );
        assert!(allowed(&original, selected(&original, None), &painted));
        assert!(!allowed(
            &original,
            selected(&original, Some(key(9))),
            &painted
        ));
        for case in 0..7 {
            let mut s = original.clone();
            match case {
                0 => s.targeting.as_mut().unwrap().selected_member = None,
                1 => {
                    std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state)
                        .view
                        .revision += 1
                }
                2 => {
                    std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state)
                        .view
                        .membership
                        .party
                        .as_mut()
                        .unwrap()
                        .leader = key(2)
                }
                3 => s.welcome.as_mut().unwrap().epoch = 2,
                4 => s.error = Some("Lost".into()),
                5 => std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state).world = [2; 32],
                6 => s.targeting.as_mut().unwrap().selected_member = Some(key(3).entity),
                _ => unreachable!(),
            }
            assert!(!allowed(&s, selected(&s, None), &painted), "case {case}");
        }
        let mut s = original;
        s.targeting.as_mut().unwrap().selected = Some(key(9).entity);
        assert_eq!(selected(&s, None), None, "Ambiguous selected identities");
    }
    #[test]
    fn overhead_requires_current_observed_loaded_identity_and_private_party() {
        let original = fixture();
        assert!(observed(&original, key(9)));
        assert!(
            !observed(&original, key(2)),
            "A distant member has no position"
        );
        for case in 0..5 {
            let mut s = original.clone();
            match case {
                0 => s.guide.as_mut().unwrap().guide = None,
                1 => s.chunk_report.as_mut().unwrap().chunks.clear(),
                2 => s.chunk_report.as_mut().unwrap().epoch = 2,
                3 => {
                    std::sync::Arc::make_mut(&mut s.party.as_mut().unwrap().state)
                        .view
                        .membership
                        .party = None
                }
                4 => s.guide.as_mut().unwrap().actor = key(2),
                _ => unreachable!(),
            }
            assert!(!observed(&s, key(9)), "case {case}");
        }
    }
    #[test]
    fn symbol_images_are_distinct_visible_rgba_and_transparent_at_edges() {
        let images = Symbol::ALL.map(icons::image);
        for (i, image) in images.iter().enumerate() {
            let data = image.data.as_ref().unwrap();
            assert_eq!(data.len(), 64 * 64 * 4);
            assert!(data.as_chunks::<4>().0.iter().filter(|p| p[3] > 0).count() > 300);
            assert_eq!(data[3], 0);
            for other in &images[..i] {
                assert_ne!(image.data, other.data);
            }
        }
    }
}
