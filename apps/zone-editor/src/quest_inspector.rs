//! Read-only NPC quest links from a validated gameplay snapshot, never player progress.
use crate::{EditorMode, Session, particle_workspace::ZoneAppExt, theme, workspace};
use bevy::prelude::*;
use bevy::{
    picking::hover::Hovered,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::{
    content::ContentId,
    inventory,
    quests::{
        catalog::{Catalog, Completion},
        item_rewards,
    },
};
use theia_network::gameplay_bundle;
use theia_zone::{
    Zone,
    markers::{Marker, MarkerKind},
};

pub(super) struct Context {
    pub path: PathBuf,
    generation: u64,
    catalog: Catalog,
    rewards: Option<item_rewards::Catalog>,
    pickups: Vec<(ContentId, String)>,
    creature: theia_network::simulation::creatures::Definition,
}
impl Context {
    pub fn load(path: &Path) -> Result<Self, String> {
        let snapshot = gameplay_bundle::snapshot(path).map_err(|e| e.to_string())?;
        let catalog = Catalog::from_json(
            snapshot
                .profile("quest")
                .ok_or("Template has no quest catalogue")?,
        )
        .map_err(|e| format!("NPC quest inspection needs a quest catalogue: {e}"))?;
        let rewards = snapshot
            .profile("quest_item_rewards")
            .map(|raw| {
                let items = inventory::Catalog::from_json(
                    snapshot.profile("items").ok_or("Missing item catalogue")?,
                )?;
                item_rewards::Catalog::from_json(raw, &catalog, &items)
            })
            .transpose()?;
        let pickups = if let Some(raw) = snapshot.profile("rewards") {
            let items = inventory::Catalog::from_json(
                snapshot.profile("items").ok_or("Missing item catalogue")?,
            )?;
            let rewards = theia_core::rewards::Catalog::from_json(raw, &items)?;
            rewards.definitions().filter_map(|reward| {
                let ground = reward.ground()?;
                let names = reward.grants().iter().map(|grant| {
                    format!("{} × {}", grant.count(), items.definition(grant.item()).expect("Validated item").name())
                }).collect::<Vec<_>>().join(", ");
                let [x,y,z] = ground.position.metres();
                Some((ground.quest.clone(), format!("{names}\nArea: {}\nGround position: {x:.3}, {y:.3}, {z:.3} m\nAvailable after acceptance; collected through normal world interaction.", ground.area)))
            }).collect()
        } else {
            Vec::new()
        };
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let creature = theia_network::simulation::creatures::Profile::from_json(
            snapshot
                .profile("creature")
                .ok_or("Template has no creature definition")?,
        )
        .map_err(|e| e.to_string())?
        .definition()
        .clone();
        Ok(Self {
            path: path.canonicalize().map_err(|e| e.to_string())?,
            generation: NEXT.fetch_add(1, Ordering::Relaxed),
            catalog,
            rewards,
            pickups,
            creature,
        })
    }
    pub fn creature(&self) -> &theia_network::simulation::creatures::Definition {
        &self.creature
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    fn cards(&self, npc: &str) -> Vec<Card> {
        self.catalog
            .quests()
            .filter(|q| q.giver().as_str() == npc || q.turn_in().as_str() == npc)
            .map(|q| {
                let role = match (q.giver().as_str() == npc, q.turn_in().as_str() == npc) {
                    (true, true) => "Offers and accepts turn-in",
                    (true, false) => "Offers quest",
                    _ => "Accepts turn-in",
                }
                .to_owned();
                let requirements = if q.prerequisites().is_empty() {
                    "No prerequisite quests".into()
                } else {
                    format!(
                        "Complete all: {}",
                        q.prerequisites()
                            .iter()
                            .map(|id| { self.catalog.quest(id).map_or(id.as_str(), |q| q.title()) })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                let mut stages = Vec::new();
                let mut next = Some(q.start_stage());
                while let Some(id) = next {
                    // The validated catalogue guarantees a bounded, acyclic stage chain.
                    let stage = q.stage(id).expect("Validated stage");
                    let rule = match stage.completion() {
                        Completion::All => "Complete all".into(),
                        Completion::Any => "Complete any one".into(),
                        Completion::AtLeast { count } => format!("Complete at least {count}"),
                    };
                    let objectives = stage
                        .objectives()
                        .iter()
                        .map(|o| {
                            let count = if o.required() > 1 {
                                format!(" ({})", o.required())
                            } else {
                                String::new()
                            };
                            let consume = if o.consume_on_turn_in() {
                                " — consumed at turn-in"
                            } else {
                                ""
                            };
                            format!("• {}{count}{consume}", o.label())
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    stages.push(format!("Stage {} — {rule}\n{objectives}", stages.len() + 1));
                    next = stage.next();
                }
                let mut rewards = format!("{} XP", q.experience());
                if let Some(items) = &self.rewards {
                    for grant in items.grants(q.id()).unwrap_or_default() {
                        let name = items
                            .item_catalog()
                            .definition(grant.item())
                            .expect("Validated item")
                            .name();
                        rewards.push_str(&format!("\n{} × {name}", grant.count()));
                    }
                }
                let mut card = Card {
                    quest: q.id().clone(),
                    title: q.title().into(),
                    role,
                    destination: None,
                    details: vec![
                        ("Description".into(), q.description().into()),
                        ("Requirements".into(), requirements),
                        ("Objectives".into(), stages.join("\n\n")),
                        ("Rewards".into(), rewards),
                        (
                            "References".into(),
                            format!(
                                "{}\nGiver: {}\nTurn-in: {}",
                                q.id().as_str(),
                                q.giver().as_str(),
                                q.turn_in().as_str()
                            ),
                        ),
                    ],
                };
                if let Some(label) = q.ready_label() {
                    card.details
                        .insert(3, ("Ready to turn in".into(), label.into()));
                }
                let pickups = self
                    .pickups
                    .iter()
                    .filter(|(quest, _)| quest == q.id())
                    .map(|(_, text)| text.as_str())
                    .collect::<Vec<_>>();
                if !pickups.is_empty() {
                    card.details
                        .insert(3, ("Ground collection".into(), pickups.join("\n\n")));
                }
                card
            })
            .collect()
    }

    fn destination_markers(&self, npc: &str, zone: &Zone) -> Vec<Marker> {
        let ids: Vec<_> = self
            .catalog
            .quests()
            .filter(|q| q.giver().as_str() == npc || q.turn_in().as_str() == npc)
            .flat_map(|q| q.stages().flat_map(|s| s.objectives()))
            .filter_map(|o| o.follow().map(|p| p.destination))
            .collect();
        zone.markers
            .iter()
            .filter(|m| ids.contains(&m.id))
            .cloned()
            .collect()
    }

    fn cards_in(&self, npc: &str, zone: &Zone) -> Vec<Card> {
        let mut cards = self.cards(npc);
        for card in &mut cards {
            let quest = self.catalog.quest(&card.quest).expect("Validated quest");
            let Some(policy) = quest
                .stages()
                .flat_map(|s| s.objectives())
                .find_map(|o| o.follow())
            else {
                continue;
            };
            let destination = zone
                .markers
                .iter()
                .find(|m| m.id == policy.destination && m.kind == MarkerKind::Quest);
            let endpoint = if let Some(marker) = destination {
                let [x, y, z] = marker.position;
                card.destination = Some(marker.id);
                format!("{} (#{}): {x:.3}, {y:.3}, {z:.3} m", marker.name, marker.id)
            } else {
                format!(
                    "Missing Quest marker #{} — export will be refused.",
                    policy.destination
                )
            };
            card.details.insert(3, ("Escort travel".into(), format!(
                "Destination: {endpoint}\nSpeed: {:.1} m/s\nFollow range: {} m; waits outside range.\nNavmesh derives the route at export/startup; no intermediate guide points are required.\nArrival makes the quest ready; speak to the NPC to complete it.",
                f64::from(policy.speed_millimetres_per_second)/1000., policy.max_distance_metres)));
        }
        cards
    }
}
#[derive(Debug, PartialEq)]
struct Card {
    quest: ContentId,
    title: String,
    role: String,
    details: Vec<(String, String)>,
    destination: Option<u64>,
}
#[derive(Component, Default)]
struct Root(Option<Key>);
#[derive(Clone, PartialEq)]
struct Key {
    document: u64,
    marker: u64,
    npc: Option<String>,
    generation: u64,
    destinations: Vec<Marker>,
}
#[derive(Component)]
struct TravelLink {
    key: Key,
    destination: u64,
}
#[derive(Component)]
struct Details;
#[derive(Component)]
struct Chevron;
#[derive(Component)]
struct Toggle(Entity, String);
#[derive(Component)]
struct Choose;

pub(super) fn spawn(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Root::default(),
        Node {
            display: Display::None,
            width: percent(100),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.,
            row_gap: px(6),
            margin: UiRect::top(px(8)),
            ..default()
        },
    ));
}
pub(super) fn plugin(app: &mut App) {
    app.add_zone_systems(sync.after(crate::present))
        .add_observer(toggle)
        .add_observer(choose)
        .add_observer(select_destination);
}
fn selected(session: &Session) -> Option<&Marker> {
    if session.play.is_some() || session.palette.is_some() || session.path_draft.is_some() {
        return None;
    }
    session
        .editor
        .zone()
        .markers
        .iter()
        .find(|m| Some(m.id) == session.marker_selected && m.kind == MarkerKind::Npc)
}
fn current_key(session: &Session) -> Option<Key> {
    selected(session).map(|m| Key {
        document: session.document_id,
        marker: m.id,
        npc: m.definition.clone(),
        generation: session.quest_context.as_ref().map_or(0, |c| c.generation),
        destinations: session
            .quest_context
            .as_ref()
            .zip(m.definition.as_deref())
            .map_or_else(Vec::new, |(c, npc)| {
                c.destination_markers(npc, session.editor.zone())
            }),
    })
}
fn sync(
    mut commands: Commands,
    session: Res<Session>,
    mut roots: Query<(Entity, &mut Root, &mut Node)>,
) {
    for (entity, mut root, mut node) in &mut roots {
        let marker = selected(&session);
        node.display = if marker.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        let key = current_key(&session);
        if root.0 == key {
            continue;
        }
        root.0 = key.clone();
        commands.entity(entity).despawn_children();
        let Some(marker) = marker else {
            continue;
        };
        let context = session.quest_context.as_ref();
        commands.entity(entity).with_children(|root| {
            label(root, "Quests", 12., theme::DISPLAY);
            root.spawn((
                Choose,
                Button,
                ActivateOnPress,
                bevy::input_focus::tab_navigation::TabIndex(0),
                Hovered::default(),
                theme::button_node(),
                BackgroundColor(theme::Variant::Default.background(false)),
            ))
            .with_child((
                theme::label(
                    if context.is_some() {
                        "Change / reload gameplay…"
                    } else {
                        "Choose gameplay file…"
                    },
                    12.,
                    theme::BODY,
                ),
                Pickable::IGNORE,
            ));
            let Some(context) = context else {
                label(
                    root,
                    "Load a gameplay manifest to inspect this NPC's quests.",
                    11.,
                    theme::MUTED,
                );
                return;
            };
            label(
                root,
                &format!(
                    "Catalogue: {}\nLoaded from {}",
                    context.catalog.id().as_str(),
                    context
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                ),
                10.,
                theme::MUTED,
            );
            let Some(npc) = &marker.definition else {
                label(
                    root,
                    "Set this NPC's Catalog ID to link its quests.",
                    11.,
                    theme::MUTED,
                );
                return;
            };
            let cards = context.cards_in(npc, session.editor.zone());
            if cards.is_empty() {
                label(
                    root,
                    "No quests in this catalogue use this NPC as giver or turn-in contact.",
                    11.,
                    theme::MUTED,
                );
            }
            for card in cards {
                root.spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.,
                    border: UiRect::top(px(1)),
                    padding: UiRect::vertical(px(6)),
                    row_gap: px(4),
                    ..default()
                })
                .insert(BorderColor::all(theme::BORDER))
                .with_children(|entry| {
                    let button = entry
                        .spawn((
                            Button,
                            theme::Variant::Ghost,
                            ActivateOnPress,
                            bevy::input_focus::tab_navigation::TabIndex(0),
                            Hovered::default(),
                            Node {
                                width: percent(100),
                                height: Val::Auto,
                                column_gap: px(10),
                                padding: UiRect::all(px(6)),
                                ..theme::button_node()
                            },
                            BackgroundColor(theme::Variant::Default.background(false)),
                        ))
                        .with_children(|header| {
                            header.spawn((
                                Chevron,
                                Pickable::IGNORE,
                                Node {
                                    width: px(6),
                                    height: px(6),
                                    flex_shrink: 0.,
                                    border: UiRect {
                                        right: px(1),
                                        bottom: px(1),
                                        ..default()
                                    },
                                    ..default()
                                },
                                BorderColor::all(theme::BODY),
                                UiTransform::from_rotation(Rot2::degrees(-45.)),
                            ));
                            header
                                .spawn(Node {
                                    flex_grow: 1.,
                                    min_width: px(0),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                })
                                .with_children(|text| {
                                    label(text, &card.title, 12., theme::BODY);
                                    label(text, &card.role, 10., theme::MUTED);
                                });
                        })
                        .id();
                    let details = entry
                        .spawn((
                            Details,
                            Node {
                                display: Display::None,
                                width: percent(100),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(4),
                                padding: UiRect::horizontal(px(6)),
                                flex_shrink: 0.,
                                ..default()
                            },
                        ))
                        .with_children(|body| {
                            if let Some(destination) = card.destination {
                                body.spawn((
                                    TravelLink {
                                        key: key.as_ref().expect("Selected NPC").clone(),
                                        destination,
                                    },
                                    Button,
                                    ActivateOnPress,
                                    bevy::input_focus::tab_navigation::TabIndex(0),
                                    Hovered::default(),
                                    theme::button_node(),
                                    BackgroundColor(theme::Variant::Default.background(false)),
                                ))
                                .with_child((
                                    theme::label("Select destination marker", 12., theme::BODY),
                                    Pickable::IGNORE,
                                ));
                            }
                            for (title, text) in &card.details {
                                label(body, title, 11., theme::DISPLAY);
                                label(body, text, 11., theme::BODY);
                            }
                        })
                        .id();
                    entry
                        .commands()
                        .entity(button)
                        .insert(Toggle(details, card.title));
                });
            }
            label(
                root,
                "Content definitions only; player quest progress is stored by the server.",
                10.,
                theme::MUTED,
            );
        });
    }
}
fn label(parent: &mut ChildSpawnerCommands, text: &str, size: f32, color: Color) {
    let mut label = parent.spawn((
        theme::label(text, size, color),
        Pickable::IGNORE,
        TextLayout::linebreak(LineBreak::WordOrCharacter),
        Node {
            width: percent(100),
            flex_shrink: 0.,
            ..default()
        },
    ));
    if color == theme::MUTED {
        label.insert(theme::Secondary);
    }
}
fn toggle(
    event: On<Activate>,
    buttons: Query<&Toggle>,
    mut details: Query<&mut Node, With<Details>>,
    children: Query<&Children>,
    mut chevrons: Query<&mut UiTransform, With<Chevron>>,
    session: Res<Session>,
) {
    if session.has_modal() {
        return;
    }
    if let Ok(toggle) = buttons.get(event.entity)
        && let Ok(mut node) = details.get_mut(toggle.0)
    {
        let open = node.display == Display::None;
        node.display = if open { Display::Flex } else { Display::None };
        if let Ok(children) = children.get(event.entity)
            && let Some(first) = children.first()
            && let Ok(mut transform) = chevrons.get_mut(*first)
        {
            *transform = UiTransform::from_rotation(Rot2::degrees(if open { 45. } else { -45. }));
        }
    }
}
fn choose(
    event: On<Activate>,
    buttons: Query<(), With<Choose>>,
    session: Res<Session>,
    mut requests: ResMut<workspace::Requests>,
) {
    if buttons.contains(event.entity) && !session.has_modal() {
        requests.gameplay_open = true;
    }
}

fn select_destination(
    event: On<Activate>,
    buttons: Query<&TravelLink>,
    mut session: ResMut<Session>,
) {
    let Ok(link) = buttons.get(event.entity) else {
        return;
    };
    if workspace::busy(&session) || current_key(&session).as_ref() != Some(&link.key) {
        return;
    }
    let Some(marker) = session
        .editor
        .zone()
        .markers
        .iter()
        .find(|m| m.id == link.destination && m.kind == MarkerKind::Quest)
    else {
        return;
    };
    let name = marker.name.clone();
    session.set_mode(EditorMode::Markers);
    session.marker_selected = Some(link.destination);
    session.status = format!("Selected quest destination: {name}");
}

#[derive(bevy::ecs::system::SystemParam)]
#[allow(clippy::type_complexity)]
pub(super) struct Report<'w, 's> {
    buttons: Query<
        'w,
        's,
        (
            Option<&'static Toggle>,
            Has<Choose>,
            Option<&'static TravelLink>,
            &'static ComputedNode,
            &'static bevy::ui::UiGlobalTransform,
        ),
        Or<(With<Toggle>, With<Choose>, With<TravelLink>)>,
    >,
    details: Query<'w, 's, &'static Node, With<Details>>,
}
impl Report<'_, '_> {
    pub fn snapshot(&self, session: &Session, scale: f32) -> serde_json::Value {
        let Some(marker) = selected(session) else {
            return serde_json::Value::Null;
        };
        let controls: Vec<_> = self.buttons.iter().map(|(toggle, choose, travel, node, transform)| {
            let factor = node.inverse_scale_factor() * scale;
            serde_json::json!({"action": if choose { "Choose gameplay" } else if travel.is_some() { "Select destination" } else { "Toggle quest" },
                "destination": travel.map(|t| t.destination),
                "title": toggle.map(|t| &t.1),
                "expanded": toggle.is_some_and(|t| self.details.get(t.0).is_ok_and(|n| n.display != Display::None)),
                "centre":(transform.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()})
        }).collect();
        serde_json::json!({"marker": marker.id, "npc":marker.definition,
            "template": session.quest_context.as_ref().map(|c| &c.path), "controls":controls,
            "cards":session.quest_context.as_ref().zip(marker.definition.as_deref()).map(|(c,npc)| c.cards_in(npc, session.editor.zone()).iter().map(|card| serde_json::json!({"title":card.title,"details":card.details,"destination":card.destination})).collect::<Vec<_>>())})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor, Zone};
    fn template(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content")
            .join(name)
    }
    fn travel_zone() -> Zone {
        Zone {
            schema: 4,
            markers: vec![
                Marker {
                    id: 1004,
                    name: "Elowen".into(),
                    kind: MarkerKind::Npc,
                    position: [2., 0.03, 0.],
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: Some("theia.dev.ranger".into()),
                },
                Marker {
                    id: 1202,
                    name: "Ancient tree".into(),
                    kind: MarkerKind::Quest,
                    position: [20., 0.03, 10.],
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: None,
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn travel_cards_resolve_destination_speed_collection_and_delivery_without_mutation() {
        let context =
            Context::load(&template("development-gameplay-starter-economy.json")).unwrap();
        let mut zone = travel_zone();
        let original = zone.clone();
        let cards = context.cards_in("theia.dev.ranger", &zone);
        let follow = cards.iter().find(|c| c.destination == Some(1202)).unwrap();
        let travel = &follow
            .details
            .iter()
            .find(|(k, _)| k == "Escort travel")
            .unwrap()
            .1;
        assert!(
            travel.contains("Ancient tree")
                && travel.contains("7.0 m/s")
                && travel.contains("10 m")
        );
        assert!(
            follow
                .details
                .iter()
                .any(|(k, v)| k == "Ready to turn in" && v.contains("Ranger Elowen"))
        );
        let cards = context.cards_in("theia.dev.villager", &zone);
        assert!(
            cards
                .iter()
                .any(|c| c.details.iter().any(|(k, v)| k == "Ground collection"
                    && v.contains("Lost Compass")
                    && v.contains("Northern Meadow")))
        );
        assert!(cards.iter().any(|c| {
            c.details
                .iter()
                .any(|(k, v)| k == "Ready to turn in" && v == "Deliver Lost Compass to Rowan")
        }));
        assert_eq!(zone, original);
        zone.markers.pop();
        let cards = context.cards_in("theia.dev.ranger", &zone);
        let follow = cards
            .iter()
            .find(|c| c.quest.as_str() == "theia.dev.quest-follow-ranger")
            .unwrap();
        assert!(follow.destination.is_none());
        assert!(
            follow
                .details
                .iter()
                .any(|(_, v)| v.contains("Missing Quest marker #1202"))
        );
    }

    #[test]
    fn destination_selection_guards_stale_context_and_preserves_zone_history() {
        let mut app = fixture();
        let zone = travel_zone();
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            "unused.zone.json".into(),
            Some(zone.clone()),
        );
        session.marker_selected = Some(1004);
        session.quest_context =
            Some(Context::load(&template("development-gameplay-starter-economy.json")).unwrap());
        app.insert_resource(session);
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<TravelLink>>()
            .single(app.world())
            .unwrap();
        // Changing the loaded catalogue invalidates old links even before sync.
        app.world_mut().resource_mut::<Session>().quest_context =
            Some(Context::load(&template("development-gameplay-starter-economy.json")).unwrap());
        app.world_mut().trigger(Activate { entity: button });
        assert_eq!(
            app.world().resource::<Session>().marker_selected,
            Some(1004)
        );
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<TravelLink>>()
            .single(app.world())
            .unwrap();
        app.world_mut().trigger(Activate { entity: button });
        let session = app.world().resource::<Session>();
        assert_eq!(session.marker_selected, Some(1202));
        assert!(session.marker_mode);
        assert_eq!(session.editor.zone(), &zone);
        assert_eq!(session.editor.history(), (0, 0));
        assert_eq!(session.editor.revision(), 0);
        // A destination edited while inspecting its NPC must invalidate the card,
        // without requiring a selection change or catalogue reload.
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            session.marker_selected = Some(1004);
        }
        app.update();
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            let mut marker = session.editor.zone().markers[1].clone();
            marker.name = "Moved destination".into();
            marker.position[0] = 24.;
            session
                .editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::PutMarker { marker }],
                })
                .unwrap();
        }
        app.update();
        assert!(
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .any(|t| t.0.contains("Moved destination (#1202): 24.000"))
        );
    }
    #[test]
    fn validated_catalogue_resolves_roles_requirements_stages_and_fixed_rewards() {
        let context =
            Context::load(&template("development-gameplay-classic-quest-areas.json")).unwrap();
        let cards = context.cards("theia.dev.guide");
        assert_eq!(cards.len(), 3);
        assert!(cards.iter().all(|c| c.role == "Offers and accepts turn-in"));
        let intro = cards.iter().find(|c| c.title == "Meet the guide").unwrap();
        assert!(intro.details.contains(&("Rewards".into(), "25 XP".into())));
        assert!(cards.iter().any(|c| {
            c.details
                .iter()
                .any(|(k, v)| k == "Requirements" && v == "Complete all: Meet the guide")
        }));
        assert!(context.cards("theia.unknown").is_empty());
        let mut split: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/quests/development-journal-areas.json"
        ))
        .unwrap();
        split["quests"][0]["turn_in"] = serde_json::json!("theia.other.npc");
        let split = Context {
            creature: context.creature.clone(),
            catalog: Catalog::from_json(&serde_json::to_vec(&split).unwrap()).unwrap(),
            rewards: None,
            pickups: Vec::new(),
            path: PathBuf::new(),
            generation: 1,
        };
        assert_eq!(split.cards("theia.other.npc")[0].role, "Accepts turn-in");
        assert_eq!(
            split
                .cards("theia.dev.guide")
                .iter()
                .find(|c| c.title == "Meet the guide")
                .unwrap()
                .role,
            "Offers quest"
        );

        let context =
            Context::load(&template("development-gameplay-classic-item-rewards.json")).unwrap();
        assert!(context.cards("theia.dev.guide").iter().any(|c| {
            c.details
                .iter()
                .any(|(k, v)| k == "Rewards" && v.contains("3 ×"))
        }));
        assert!(Context::load(&template("zones/quest-areas.zone.json")).is_err());
    }
    fn fixture() -> App {
        let zone = Zone {
            schema: 4,
            markers: vec![Marker {
                id: 1,
                name: "Guide".into(),
                kind: MarkerKind::Npc,
                position: [2., 0.03, 0.],
                yaw_degrees: 0.,
                radius_metres: 1.,
                definition: Some("theia.dev.guide".into()),
            }],
            ..default()
        };
        let mut session = Session::new(
            Editor::new(zone.clone()).unwrap(),
            "unused.zone.json".into(),
            Some(zone),
        );
        session.marker_selected = Some(1);
        session.quest_context = Some(
            Context::load(&template("development-gameplay-classic-quest-areas.json")).unwrap(),
        );
        let mut app = App::new();
        app.insert_resource(session)
            .init_resource::<workspace::Requests>()
            .add_plugins(plugin);
        app.world_mut()
            .commands()
            .spawn(Node::default())
            .with_children(spawn);
        app.world_mut().flush();
        app.update();
        app
    }
    #[test]
    fn changing_npc_binding_rebuilds_cards_but_transform_edits_preserve_expansion() {
        let mut app = fixture();
        let details: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<Details>>()
            .iter(app.world())
            .collect();
        assert_eq!(details.len(), 3);
        let button = app
            .world_mut()
            .query::<(Entity, &Toggle)>()
            .iter(app.world())
            .find(|(_, t)| t.0 == details[0])
            .unwrap()
            .0;
        app.world_mut().trigger(Activate { entity: button });
        assert_eq!(
            app.world().get::<Node>(details[0]).unwrap().display,
            Display::Flex
        );
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            session
                .editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::Translate {
                        id: 1,
                        offset: [0.125, 0., 0.],
                    }],
                })
                .unwrap();
        }
        app.update();
        assert_eq!(
            app.world().get::<Node>(details[0]).unwrap().display,
            Display::Flex
        );
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            let mut marker = session.editor.zone().markers[0].clone();
            marker.definition = None;
            session
                .editor
                .execute(Command::Apply {
                    expected_revision: 1,
                    edits: vec![Edit::PutMarker { marker }],
                })
                .unwrap();
        }
        app.update();
        assert!(app.world().get::<Node>(details[0]).is_none());
        assert_eq!(
            app.world_mut()
                .query::<&Details>()
                .iter(app.world())
                .count(),
            0
        );
        assert!(
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .any(|t| t.0.contains("Set this NPC's Catalog ID"))
        );
        app.world_mut().resource_mut::<Session>().marker_selected = None;
        app.update();
        assert!(
            app.world_mut()
                .query_filtered::<&Node, With<Root>>()
                .iter(app.world())
                .all(|n| n.display == Display::None)
        );
    }
}
