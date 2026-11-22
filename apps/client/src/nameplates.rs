//! Bounded clickable enemy health labels from authoritative observations.
use avian3d::prelude::SpatialQuery;
use bevy::{picking::Pickable, prelude::*, text::FontSize};
use theia_core::identity::EntityKey;
use theia_network::{client::State, combat_wire::Health, targeting::MAX_TARGETS};

const WIDTH: f32 = 140.;
const HEIGHT: f32 = 36.;
/// Cosmetic combat text starts beside the name/health column. Prefer right,
/// then left near the viewport's right edge; crowd/movement stacking is separate.
pub(crate) fn feedback_left(centre: f32, viewport: f32, text_width: f32) -> f32 {
    let right = centre + WIDTH * 0.5 + 8.;
    if right + text_width <= viewport - 4. {
        right
    } else {
        (centre - WIDTH * 0.5 - 8. - text_width).max(4.)
    }
}
#[derive(Component)]
pub(crate) struct Root(usize);
#[derive(Component)]
struct Name(usize);
#[derive(Component)]
struct Fill(usize);
#[derive(Component)]
struct Bar(usize);
struct Plate {
    key: Option<EntityKey>,
    anchor: Vec2,
    name: &'static str,
    fraction: f32,
    selected: bool,
}
#[derive(Resource, Default)]
struct Model(Vec<Plate>);
#[derive(Resource, Default)]
struct Attackers(Vec<(EntityKey, u64)>);
impl Attackers {
    fn observe(&mut self, state: &State) {
        let Some(life) = state.player_life.as_ref().and_then(|s| s.current()) else {
            self.0.clear();
            return;
        };
        if life.health == 0 || life.combat.is_some_and(|c| !c.in_combat()) {
            self.0.clear();
            return;
        }
        let owner = life.key.actor;
        let mut retain = |observation: &theia_network::creature_attack_wire::Observation,
                          hz: u32| {
            if let Some(w) = observation.windup {
                if w.target == owner {
                    // Keep the bar across the ordinary swing cooldown, using
                    // authoritative simulation time rather than render frames.
                    self.0.retain(|(key, _)| *key != observation.creature);
                    self.0
                        .push((observation.creature, w.completes + u64::from(hz) * 3));
                } else {
                    self.0.retain(|(key, _)| *key != observation.creature);
                }
            }
        };
        if let Some(p) = &state.public_population_attack
            && let Some(frame) = p.current()
        {
            for observation in &frame.actors {
                retain(observation, p.rate().hz());
            }
        }
        if let Some(p) = &state.public_creature_attack
            && let Some(observation) = p.current()
        {
            retain(observation, p.rate().hz());
        }
        self.0
            .retain(|(key, deadline)| *deadline >= life.tick && health(state, *key).is_some());
    }
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub visible: usize,
    pub visible_frames: u64,
    pub selected_frames: u64,
    pub laid_out_samples: u64,
}

pub(crate) fn install(app: &mut App) {
    widgets(app);
    app.add_systems(
        PostUpdate,
        (present, paint)
            .chain()
            .after(crate::camera::sync_camera_global)
            .after(avian3d::prelude::PhysicsSystems::StepSimulation)
            .before(bevy::ui::UiSystems::Layout),
    );
}
pub(crate) fn install_preview(app: &mut App, anchor: Vec2) {
    widgets(app);
    app.insert_resource(Model(vec![Plate {
        key: None,
        anchor,
        name: "Wolf (fixture)",
        fraction: 0.57,
        selected: true,
    }]))
    .add_systems(PostUpdate, paint.before(bevy::ui::UiSystems::Layout));
}
fn widgets(app: &mut App) {
    app.init_resource::<Model>()
        .init_resource::<Attackers>()
        .init_resource::<Metrics>()
        .add_observer(select)
        .add_systems(Startup, setup)
        .add_systems(Last, observe);
}
fn setup(mut commands: Commands) {
    for slot in 0..MAX_TARGETS {
        commands
            .spawn((
                Root(slot),
                PlateTarget(None),
                bevy::ui_widgets::Button,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(WIDTH),
                    height: px(HEIGHT),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    display: Display::None,
                    ..default()
                },
                GlobalZIndex(2),
            ))
            .with_children(|root| {
                root.spawn((
                    Name(slot),
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    TextLayout::justify(Justify::Center),
                    TextShadow {
                        offset: Vec2::ONE,
                        color: Color::BLACK,
                    },
                    Pickable::IGNORE,
                ));
                root.spawn((
                    Bar(slot),
                    Node {
                        width: percent(100),
                        height: px(10),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(Color::BLACK),
                    BackgroundColor(Color::srgb(0.08, 0.03, 0.03)),
                    Pickable::IGNORE,
                ))
                .with_child((
                    Fill(slot),
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.78, 0.16, 0.12)),
                    Pickable::IGNORE,
                ));
            });
    }
}
fn health(state: &State, key: EntityKey) -> Option<&Health> {
    if state.error.is_some() || state.welcome.as_ref()?.epoch != key.epoch.value() {
        return None;
    }
    let targets = state.targeting.as_ref()?;
    let combat = state.combat.as_ref()?;
    let report = state.chunk_report.as_ref()?;
    if targets.epoch != key.epoch.value()
        || combat.epoch != targets.epoch
        || report.epoch != targets.epoch
        || !report.valid()
    {
        return None;
    }
    let target = targets.targets.iter().find(|target| target.key == key)?;
    let chunk = target.position.chunk(128)?;
    if !report.chunks.contains(&[chunk.x, chunk.z]) {
        return None;
    }
    combat.targets.iter().find(|h| {
        h.entity == key.entity
            && h.current > 0
            && h.maximum > 0
            && h.current <= h.maximum
            && h.respawn_at.is_none()
    })
}
#[allow(clippy::too_many_arguments)]
fn present(
    network: Res<crate::network::Network>,
    scale: Res<bevy::ui::UiScale>,
    cameras: Query<(&Camera, &GlobalTransform), crate::frontend::WorldCamera>,
    actors: Query<(&crate::targeting::TargetVisual, &Transform, &Visibility)>,
    spatial: SpatialQuery,
    creatures: Option<Res<crate::creature_visual::Pending>>,
    mut model: ResMut<Model>,
    mut attackers: ResMut<Attackers>,
) {
    model.0.clear();
    if !network.connected() {
        return;
    }
    let state = network.state();
    attackers.observe(&state);
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Some(viewport) = camera
        .logical_viewport_size()
        .and_then(|p| crate::fantasy_ui::projected_ui_point(p, scale.0))
    else {
        return;
    };
    for (actor, transform, visibility) in &actors {
        if model.0.len() == MAX_TARGETS {
            break;
        }
        if *visibility == Visibility::Hidden {
            continue;
        }
        let selected = state
            .targeting
            .as_ref()
            .is_some_and(|v| v.selected == Some(actor.key.entity));
        if !selected && !attackers.0.iter().any(|(key, _)| *key == actor.key) {
            continue;
        }
        let Some(health) = health(&state, actor.key) else {
            continue;
        };
        // The current wolf/training proxy root is its body centre. This is an
        // art-presentation offset in metres, not a hitbox or Classic range rule.
        let point = transform.translation + Vec3::Y * 0.9;
        let Ok(screen) = camera.world_to_viewport(camera_transform, point) else {
            continue;
        };
        let Some(anchor) = crate::fantasy_ui::projected_ui_point(screen, scale.0) else {
            continue;
        };
        if anchor.x < WIDTH * 0.5
            || anchor.x > viewport.x - WIDTH * 0.5
            || anchor.y < HEIGHT
            || anchor.y > viewport.y
            || crate::quest_marker::occluded(&spatial, camera_transform.translation(), point)
        {
            continue;
        }
        model.0.push(Plate {
            key: Some(actor.key),
            anchor,
            name: if creatures.is_some() {
                "Wolf"
            } else {
                "Training target"
            },
            fraction: health.current as f32 / health.maximum as f32,
            selected: state
                .targeting
                .as_ref()
                .is_some_and(|v| v.selected == Some(actor.key.entity)),
        });
    }
    model.0.sort_by_key(|p| p.key.map(|k| k.entity.value()));
    spread(&mut model.0, viewport);
}

fn spread(plates: &mut [Plate], viewport: Vec2) {
    for i in 0..plates.len() {
        let origin = plates[i].anchor;
        let mut best = None;
        for column in -4i32..=4 {
            for row in -16i32..=16 {
                let candidate =
                    origin + Vec2::new(column as f32 * (WIDTH + 8.), row as f32 * (HEIGHT + 5.));
                if candidate.x < WIDTH * 0.5
                    || candidate.x > viewport.x - WIDTH * 0.5
                    || candidate.y < HEIGHT
                    || candidate.y > viewport.y
                {
                    continue;
                }
                if plates[..i].iter().any(|p| {
                    (p.anchor.x - candidate.x).abs() < WIDTH + 4.
                        && (p.anchor.y - candidate.y).abs() < HEIGHT + 4.
                }) {
                    continue;
                }
                let cost = candidate.distance_squared(origin);
                if best.is_none_or(|(_, previous)| cost < previous) {
                    best = Some((candidate, cost));
                }
            }
        }
        if let Some((anchor, _)) = best {
            plates[i].anchor = anchor;
        }
    }
}
#[derive(Component)]
struct PlateTarget(Option<EntityKey>);
fn select(
    event: On<bevy::ui_widgets::Activate>,
    plates: Query<&PlateTarget>,
    network: Res<crate::network::Network>,
    windows: Query<&Window>,
    mut selection: ResMut<crate::world_input::Selection>,
) {
    if !windows.single().is_ok_and(|w| w.focused) || !network.can_act() {
        return;
    }
    let Ok(PlateTarget(Some(key))) = plates.get(event.entity) else {
        return;
    };
    if health(&network.state(), *key).is_some() {
        network.request_target(Some(key.entity));
        selection.friendly = None;
        selection.enemy_clicks += 1;
    }
}
fn paint(
    model: Res<Model>,
    mut roots: Query<(&Root, &mut Node, &mut PlateTarget), Without<Fill>>,
    mut fills: Query<(&Fill, &mut Node), Without<Root>>,
    mut names: Query<(&Name, &mut Text)>,
    mut bars: Query<(&Bar, &mut BorderColor)>,
    mut metrics: ResMut<Metrics>,
) {
    metrics.visible = model.0.len();
    metrics.visible_frames += u64::from(!model.0.is_empty());
    metrics.selected_frames += u64::from(model.0.iter().any(|p| p.selected));
    for (slot, mut node, mut target) in &mut roots {
        target.0 = model.0.get(slot.0).and_then(|p| p.key);
        let Some(plate) = model.0.get(slot.0) else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let (left, top) = (
            px(plate.anchor.x - WIDTH * 0.5),
            px(plate.anchor.y - HEIGHT),
        );
        if node.display != Display::Flex || node.left != left || node.top != top {
            node.display = Display::Flex;
            node.left = left;
            node.top = top;
        }
    }
    for (slot, mut node) in &mut fills {
        if let Some(plate) = model.0.get(slot.0) {
            let width = percent(plate.fraction * 100.);
            if node.width != width {
                node.width = width;
            }
        }
    }
    for (slot, mut text) in &mut names {
        if let Some(plate) = model.0.get(slot.0)
            && text.0 != plate.name
        {
            text.0 = plate.name.into();
        }
    }
    for (slot, mut border) in &mut bars {
        if let Some(plate) = model.0.get(slot.0) {
            let color = BorderColor::all(if plate.selected {
                crate::fantasy_ui::GOLD
            } else {
                Color::BLACK
            });
            if *border != color {
                *border = color;
            }
        }
    }
}
fn observe(nodes: Query<(&Node, &ComputedNode), With<Root>>, mut metrics: ResMut<Metrics>) {
    metrics.laid_out_samples += nodes
        .iter()
        .filter(|(node, computed)| {
            node.display != Display::None && computed.size().min_element() > 0.
        })
        .count() as u64;
}

/// Read-only painted targets for the owned native input qualification.
pub(crate) fn qualification(world: &mut World) -> serde_json::Value {
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let plates: Vec<_> = world.query::<(&PlateTarget, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
        .iter(world)
        .filter_map(|(target, node, transform)| {
            let key = target.0?;
            (node.size().min_element() > 0.).then(|| serde_json::json!({
                "entity": key.entity.value(),
                "point": (transform.affine().translation * crate::fantasy_ui::window_factor(node, scale)).to_array(),
                "size": (node.size() * crate::fantasy_ui::window_factor(node, scale)).to_array()
            }))
        }).collect();
    serde_json::json!(plates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        coordinates::GlobalPosition,
        identity::{EntityId, WorldEpoch},
    };
    use theia_network::{combat_wire, targeting, targeting_wire, wire};
    #[test]
    fn crowded_nameplates_remain_separate_inside_small_and_ultrawide_views() {
        for viewport in [Vec2::new(1280., 720.), Vec2::new(3440., 1440.)] {
            let mut plates: Vec<_> = (0..MAX_TARGETS)
                .map(|_| Plate {
                    key: None,
                    anchor: viewport * 0.5,
                    name: "Wolf",
                    fraction: 1.,
                    selected: false,
                })
                .collect();
            spread(&mut plates, viewport);
            for (i, a) in plates.iter().enumerate() {
                assert!(a.anchor.x >= WIDTH * 0.5 && a.anchor.x <= viewport.x - WIDTH * 0.5);
                assert!(a.anchor.y >= HEIGHT && a.anchor.y <= viewport.y);
                for b in &plates[..i] {
                    assert!(
                        (a.anchor.x - b.anchor.x).abs() >= WIDTH
                            || (a.anchor.y - b.anchor.y).abs() >= HEIGHT
                    );
                }
            }
        }
    }
    #[test]
    fn damage_text_lanes_stay_clear_of_the_nameplate_and_viewport_edges() {
        for centre in [70., 640., 1210.] {
            let left = feedback_left(centre, 1280., 160.);
            assert!(left >= 0. && left + 160. <= 1280.);
            assert!(left >= centre + WIDTH * 0.5 || left + 160. <= centre - WIDTH * 0.5);
        }
    }
    fn fixture() -> (State, EntityKey) {
        let key = EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(2).unwrap(),
        };
        (
            State {
                welcome: Some(wire::Welcome {
                    terrain_stream: true,
                    id: 1,
                    epoch: 7,
                    world: [1; 32],
                    origin: [0.; 3],
                }),
                targeting: Some(targeting_wire::State {
                    epoch: 7,
                    tick: 1,
                    request_sequence: 0,
                    accepted: true,
                    selected: Some(key.entity),
                    selected_member: None,
                    targets: vec![targeting::Target {
                        key,
                        position: GlobalPosition::new([4., 0.9, 4.]).unwrap(),
                    }],
                }),
                combat: Some(combat_wire::State {
                    vitals: None,
                    epoch: 7,
                    tick: 1,
                    reply: None,
                    gcd_until: 0,
                    cast: None,
                    queued: None,
                    cooldowns: vec![],
                    targets: vec![Health {
                        entity: key.entity,
                        current: 57,
                        maximum: 100,
                        respawn_at: None,
                    }],
                }),
                chunk_report: Some(wire::ChunkReady {
                    version: wire::VERSION,
                    epoch: 7,
                    sequence: 1,
                    chunks: vec![[0, 0]],
                }),
                ..default()
            },
            key,
        )
    }
    #[test]
    fn stale_dead_unknown_and_unloaded_observations_do_not_become_nameplates() {
        let (state, key) = fixture();
        assert_eq!(health(&state, key).unwrap().current, 57);
        let invalid: [fn(&mut State); 11] = [
            |s| s.error = Some("Disconnected".into()),
            |s| s.welcome.as_mut().unwrap().epoch = 8,
            |s| s.targeting.as_mut().unwrap().epoch = 8,
            |s| s.combat.as_mut().unwrap().epoch = 8,
            |s| s.chunk_report.as_mut().unwrap().epoch = 8,
            |s| s.chunk_report.as_mut().unwrap().chunks.clear(),
            |s| s.targeting.as_mut().unwrap().targets.clear(),
            |s| s.combat.as_mut().unwrap().targets[0].current = 0,
            |s| s.combat.as_mut().unwrap().targets[0].maximum = 0,
            |s| s.combat.as_mut().unwrap().targets[0].current = 101,
            |s| s.combat.as_mut().unwrap().targets[0].respawn_at = Some(100),
        ];
        for invalidate in invalid {
            let mut changed = state.clone();
            invalidate(&mut changed);
            assert!(health(&changed, key).is_none());
        }
        assert!(
            health(
                &state,
                EntityKey {
                    entity: EntityId::new(9).unwrap(),
                    ..key
                }
            )
            .is_none()
        );
        assert!(health(&State::default(), key).is_none());
    }
    #[test]
    fn bounded_widgets_show_observed_health_then_clear_without_stale_labels() {
        let mut app = App::new();
        widgets(&mut app);
        app.insert_resource(Model(vec![Plate {
            key: None,
            anchor: Vec2::new(200., 200.),
            name: "Wolf",
            fraction: 0.57,
            selected: true,
        }]))
        .add_systems(Update, paint);
        app.update();
        let world = app.world_mut();
        let mut roots = world.query::<(&Root, &Node)>();
        assert_eq!(roots.iter(world).count(), MAX_TARGETS);
        assert_eq!(
            roots
                .iter(world)
                .filter(|(_, n)| n.display != Display::None)
                .count(),
            1
        );
        let mut fills = world.query::<(&Fill, &Node)>();
        let fill = fills.iter(world).find(|(slot, _)| slot.0 == 0).unwrap().1;
        assert!(matches!(fill.width,Val::Percent(v) if (v-57.).abs()<0.001));
        let mut bars = world.query::<(&Bar, &BorderColor)>();
        assert_eq!(
            *bars.iter(world).find(|(slot, _)| slot.0 == 0).unwrap().1,
            BorderColor::all(crate::fantasy_ui::GOLD)
        );
        world.resource_mut::<Model>().0.clear();
        app.update();
        let world = app.world_mut();
        let mut roots = world.query_filtered::<&Node, With<Root>>();
        assert!(roots.iter(world).all(|n| n.display == Display::None));
    }
}
