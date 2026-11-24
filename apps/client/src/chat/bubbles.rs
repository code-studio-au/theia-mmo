//! Bounded Say presentation. Never derives actors or positions from message text.
use avian3d::prelude::SpatialQuery;
use bevy::{picking::Pickable, prelude::*};
use std::{collections::VecDeque, time::Duration};
use theia_core::{
    chat::Channel,
    identity::{EntityId, EntityKey, WorldEpoch},
};
use theia_network::{chat_wire::Delivery, client::State};

const CAPACITY: usize = 8;
const LIFETIME: Duration = Duration::from_secs(6);
const WIDTH: f32 = 260.;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Context {
    owner: EntityKey,
    world: [u8; 32],
}
impl Context {
    fn from_state(state: &State) -> Option<Self> {
        let welcome = state.welcome.as_ref()?;
        let snapshot = state.snapshot.as_ref()?;
        if state.error.is_some() || state.id != Some(welcome.id) || snapshot.epoch != welcome.epoch
        {
            return None;
        }
        Some(Self {
            owner: EntityKey {
                epoch: WorldEpoch::new(welcome.epoch)?,
                entity: EntityId::new(welcome.id)?,
            },
            world: welcome.world,
        })
    }
}
struct Bubble {
    sender: EntityKey,
    serial: u64,
    text: theia_core::chat::Text,
    until: Duration,
}
#[derive(Resource, Default)]
struct Feed {
    context: Option<Context>,
    serial: u64,
    items: VecDeque<Bubble>,
}
impl Feed {
    fn observe<'a>(
        &mut self,
        context: Option<Context>,
        now: Duration,
        history: impl Iterator<Item = &'a Delivery>,
        visible: impl Fn(EntityKey) -> bool,
    ) {
        if self.context != context {
            self.items.clear();
            self.context = context;
            // Attaching/reconnecting must not replay historical speech.
            self.serial = history.map(|d| d.serial).max().unwrap_or(0);
            return;
        }
        let Some(context) = context else {
            self.items.clear();
            return;
        };
        self.items.retain(|b| b.until > now && visible(b.sender));
        for delivery in history {
            if delivery.serial <= self.serial {
                continue;
            }
            self.serial = delivery.serial;
            if delivery.channel != Channel::Say
                || delivery.world != context.world
                || delivery.sender.epoch != context.owner.epoch
                || !visible(delivery.sender)
            {
                continue;
            }
            self.items.retain(|b| b.sender != delivery.sender);
            if self.items.len() == CAPACITY {
                self.items.pop_front();
            }
            self.items.push_back(Bubble {
                sender: delivery.sender,
                serial: delivery.serial,
                text: delivery.text.clone(),
                until: now.saturating_add(LIFETIME),
            });
        }
    }
}
#[derive(Component)]
struct Root(usize);
#[derive(Component)]
struct Label(usize);

pub(super) fn install(app: &mut App) {
    app.init_resource::<Feed>()
        .add_systems(Startup, setup)
        .add_systems(
            PostUpdate,
            present
                .after(crate::camera::sync_camera_global)
                .after(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate)
                .before(bevy::ui::UiSystems::Layout),
        );
}
fn setup(mut commands: Commands) {
    for slot in 0..CAPACITY {
        commands
            .spawn((
                Root(slot),
                Pickable::IGNORE,
                GlobalZIndex(3),
                Node {
                    position_type: PositionType::Absolute,
                    width: px(WIDTH),
                    padding: UiRect::all(px(12)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                    display: Display::None,
                    ..default()
                },
                UiTransform::from_translation(Val2 {
                    x: percent(-50),
                    y: percent(-100),
                }),
                BorderColor::all(crate::ui_theme::GOLD),
                BackgroundColor(crate::ui_theme::PANEL),
            ))
            .with_children(|root| {
                root.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        bottom: px(-4),
                        width: px(8),
                        height: px(8),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    UiTransform::from_rotation(Rot2::degrees(45.)),
                    BorderColor::all(crate::ui_theme::GOLD),
                    BackgroundColor(crate::ui_theme::PANEL),
                ));
                root.spawn((
                    Label(slot),
                    Text::new(""),
                    TextFont::from_font_size(14.),
                    TextColor(Color::WHITE),
                    TextLayout::new(Justify::Center, bevy::text::LineBreak::WordOrCharacter),
                    Node {
                        width: percent(100),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            });
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn present(
    network: Option<Res<crate::network::Network>>,
    time: Res<Time<Real>>,
    scale: Res<UiScale>,
    cameras: Query<(&Camera, &GlobalTransform), crate::frontend::WorldCamera>,
    pawns: Query<
        (
            Entity,
            &GlobalTransform,
            &Visibility,
            Option<&crate::movement::LocalPawn>,
            Option<&crate::network::RemotePawn>,
            Has<Mesh3d>,
        ),
        Or<(
            With<crate::movement::LocalPawn>,
            With<crate::network::RemotePawn>,
        )>,
    >,
    visuals: Query<(
        &crate::reference_scene::Visual,
        &Visibility,
        &InheritedVisibility,
    )>,
    spatial: SpatialQuery,
    mut feed: ResMut<Feed>,
    mut roots: Query<(&Root, &mut Node, &ComputedNode)>,
    mut labels: Query<(&Label, &mut Text)>,
) {
    let state = network
        .as_deref()
        .filter(|n| !n.loading_blocked && n.connected())
        .map(|n| n.state());
    let context = state
        .as_ref()
        .filter(|s| s.chat.is_some())
        .and_then(Context::from_state);
    let camera = cameras.single().ok().filter(|(c, _)| c.is_active);
    let viewport = camera
        .and_then(|(c, _)| c.logical_viewport_size())
        .map(|v| v / scale.0);
    let mut anchors = Vec::new();
    if let (Some(context), Some(state), Some((camera, eye)), Some(viewport)) =
        (context, state.as_ref(), camera, viewport)
    {
        for (entity, transform, visibility, local, remote, mesh) in &pawns {
            let Some(id) = local
                .map(|_| context.owner.entity.value())
                .or_else(|| remote.map(|r| r.actor_id()))
            else {
                continue;
            };
            let rendered = mesh
                || visuals.iter().any(|(v, visibility, inherited)| {
                    v.owner == entity
                        && !v.players.is_empty()
                        && *visibility != Visibility::Hidden
                        && inherited.get()
                });
            if *visibility == Visibility::Hidden
                || !rendered
                || !state
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.players.iter().any(|p| p.id == id))
            {
                continue;
            }
            let point = transform.translation() + Vec3::Y * 1.25;
            if crate::quest_marker::occluded(&spatial, eye.translation(), point) {
                continue;
            }
            let Some(screen) = camera
                .world_to_viewport(eye, point)
                .ok()
                .and_then(|p| crate::fantasy_ui::projected_ui_point(p, scale.0))
            else {
                continue;
            };
            if screen.x < 0. || screen.y < 0. || screen.x > viewport.x || screen.y > viewport.y {
                continue;
            }
            if let Some(entity) = EntityId::new(id) {
                anchors.push((
                    EntityKey {
                        epoch: context.owner.epoch,
                        entity,
                    },
                    screen,
                ));
            }
        }
    }
    let history = state
        .as_ref()
        .and_then(|s| s.chat.as_ref())
        .into_iter()
        .flat_map(|view| view.inbox.history());
    feed.observe(context, time.elapsed(), history, |key| {
        anchors.iter().any(|(actor, _)| *actor == key)
    });
    for (root, mut node, computed) in &mut roots {
        let anchor = feed
            .items
            .get(root.0)
            .and_then(|b| anchors.iter().find(|(key, _)| *key == b.sender))
            .map(|(_, p)| *p);
        node.display = Display::None;
        if let (Some(anchor), Some(viewport)) = (anchor, viewport) {
            let height = computed.size().y * computed.inverse_scale_factor();
            if viewport.x >= WIDTH + 8. && viewport.y > height + 8. {
                node.display = Display::Flex;
                node.left = px(anchor
                    .x
                    .clamp(WIDTH * 0.5 + 4., viewport.x - WIDTH * 0.5 - 4.));
                node.top = px((anchor.y - 12.).clamp(height + 4., viewport.y - 4.));
            }
        }
    }
    for (label, mut text) in &mut labels {
        let value = feed.items.get(label.0).map_or("", |b| b.text.as_str());
        if text.0 != value {
            **text = value.to_owned();
        }
    }
}

pub(super) fn snapshot(world: &mut World) -> serde_json::Value {
    let Some(feed) = world.get_resource::<Feed>() else {
        return serde_json::Value::Null;
    };
    let entries: Vec<_> = feed.items.iter().map(|b| serde_json::json!({"actor":b.sender.entity.value(),"serial":b.serial,"bytes":b.text.as_str().len()})).collect();
    let visible = world
        .query_filtered::<&Node, With<Root>>()
        .iter(world)
        .filter(|n| n.display != Display::None)
        .count();
    serde_json::json!({"entries":entries,"visible":visible,"capacity":CAPACITY,"lifetime_seconds":LIFETIME.as_secs()})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> Context {
        Context {
            owner: key(1),
            world: [1; 32],
        }
    }
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(3).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn delivery(serial: u64, actor: u64, channel: Channel) -> Delivery {
        Delivery {
            serial,
            sender: key(actor),
            world: [1; 32],
            sequence: serial,
            tick: serial,
            channel,
            name: theia_core::character_name::CharacterName::parse("Arden").unwrap(),
            text: theia_core::chat::Text::parse("Hello <nearby> 🌿").unwrap(),
        }
    }
    #[test]
    fn bubbles_expire_without_duplicate_refresh_and_do_not_replay_history() {
        let mut feed = Feed::default();
        let mut history = vec![delivery(1, 1, Channel::Say)];
        feed.observe(Some(context()), Duration::ZERO, history.iter(), |_| true);
        assert!(feed.items.is_empty());
        history.push(delivery(2, 1, Channel::Say));
        feed.observe(
            Some(context()),
            Duration::from_secs(1),
            history.iter(),
            |_| true,
        );
        assert_eq!(feed.items.len(), 1);
        assert_eq!(feed.items[0].text.as_str(), "Hello <nearby> 🌿");
        feed.observe(
            Some(context()),
            Duration::from_secs(6),
            history.iter(),
            |_| true,
        );
        assert_eq!(feed.items.len(), 1);
        feed.observe(
            Some(context()),
            Duration::from_secs(7),
            history.iter(),
            |_| true,
        );
        assert!(feed.items.is_empty());
        feed.observe(None, Duration::from_secs(8), history.iter(), |_| true);
        feed.observe(
            Some(context()),
            Duration::from_secs(9),
            history.iter(),
            |_| true,
        );
        assert!(feed.items.is_empty());
    }
    #[test]
    fn area_unknown_hidden_and_foreign_speakers_never_get_bubbles() {
        let mut feed = Feed::default();
        feed.observe(Some(context()), Duration::ZERO, [].iter(), |_| true);
        let mut history = vec![delivery(1, 1, Channel::Area), delivery(2, 2, Channel::Say)];
        let mut foreign = delivery(3, 1, Channel::Say);
        foreign.sender.epoch = WorldEpoch::new(4).unwrap();
        history.push(foreign);
        let mut foreign = delivery(4, 1, Channel::Say);
        foreign.world = [2; 32];
        history.push(foreign);
        feed.observe(Some(context()), Duration::ZERO, history.iter(), |k| {
            k == key(1)
        });
        assert!(feed.items.is_empty());
        feed.observe(Some(context()), Duration::ZERO, history.iter(), |_| true);
        assert!(feed.items.is_empty());
        history.push(delivery(5, 1, Channel::Say));
        feed.observe(Some(context()), Duration::ZERO, history.iter(), |_| true);
        assert_eq!(feed.items.len(), 1);
        feed.observe(Some(context()), Duration::ZERO, history.iter(), |_| false);
        assert!(feed.items.is_empty());
    }
    #[test]
    fn bubbles_replace_per_speaker_and_evict_oldest_at_fixed_capacity() {
        let mut feed = Feed::default();
        feed.observe(Some(context()), Duration::ZERO, [].iter(), |_| true);
        let mut history: Vec<_> = (1..=9).map(|i| delivery(i, i, Channel::Say)).collect();
        feed.observe(Some(context()), Duration::ZERO, history.iter(), |_| true);
        assert_eq!(feed.items.len(), CAPACITY);
        assert_eq!(feed.items.front().unwrap().sender, key(2));
        history.push(delivery(10, 9, Channel::Say));
        feed.observe(Some(context()), Duration::ZERO, history.iter(), |_| true);
        assert_eq!(feed.items.len(), CAPACITY);
        assert_eq!(feed.items.back().unwrap().serial, 10);
        let mut next = context();
        next.owner.epoch = WorldEpoch::new(4).unwrap();
        feed.observe(Some(next), Duration::ZERO, history.iter(), |_| true);
        assert!(feed.items.is_empty());
    }
}
