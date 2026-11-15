//! Bevy mesh picking feeds existing authoritative commands; selection is cosmetic.
pub(crate) mod qualification;
use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    picking::{
        PickingSystems,
        events::{PointerClick, PointerDrag, PointerPress},
        mesh_picking::{MeshPickingCamera, MeshPickingPlugin, MeshPickingSettings},
        pointer::{PointerButton, PointerId},
    },
    prelude::*,
};
use theia_core::{content::ContentId, identity::EntityKey};

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Interactions;
#[derive(Resource)]
struct Bindings {
    basic_attack: ContentId,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Selection {
    #[serde(skip)]
    pub friendly: Option<EntityKey>,
    pub enemy_clicks: u64,
    pub friendly_clicks: u64,
    pub basic_requests: u64,
    pub interaction_requests: u64,
    pub drag_suppressed: u64,
    pub blocked_clicks: u64,
}
#[derive(Clone, Copy)]
enum Actor {
    Enemy(EntityKey),
    Guide(Option<EntityKey>),
    Corpse(theia_core::progression::KillId),
    Ground(theia_core::rewards::ClaimId),
}
#[derive(Resource, Default)]
struct Pending(Option<(Actor, PointerButton, Vec3)>);
impl Pending {
    fn take_focused(&mut self, focused: bool) -> Option<(Actor, PointerButton, Vec3)> {
        self.0.take().filter(|_| focused)
    }
}
#[derive(Resource, Default)]
pub(crate) struct Gesture {
    travel: f32,
    ui_origin: bool,
}
impl Gesture {
    pub(crate) fn world_origin(&self) -> bool {
        !self.ui_origin
    }
    pub(crate) fn camera_dragging(&self) -> bool {
        self.travel > 6. && !self.ui_origin
    }
}
fn track(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut gesture: ResMut<Gesture>,
) {
    if buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right) {
        gesture.travel = 0.;
        gesture.ui_origin = false;
    }
    if buttons.pressed(MouseButton::Left)
        || buttons.pressed(MouseButton::Right)
        || buttons.just_released(MouseButton::Left)
        || buttons.just_released(MouseButton::Right)
    {
        gesture.travel += motion.delta.length();
    }
}
type ActorNode<'a> = (
    Option<&'a crate::targeting::TargetVisual>,
    Has<crate::dialogue::GuideVisual>,
    Option<&'a ChildOf>,
    Has<Node>,
    Option<&'a crate::loot::CorpseVisual>,
    Option<&'a crate::dialogue::ContactKey>,
    Option<&'a crate::inventory::ground::Pickup>,
);
fn actor(mut entity: Entity, nodes: &Query<ActorNode<'_>>) -> Option<Actor> {
    for _ in 0..32 {
        let (target, guide, parent, ui, corpse, contact, ground) = nodes.get(entity).ok()?;
        if ui {
            return None;
        }
        if let Some(ground) = ground {
            return Some(Actor::Ground(ground.0));
        }
        if let Some(corpse) = corpse {
            return Some(Actor::Corpse(corpse.0.life));
        }
        if let Some(target) = target {
            return Some(Actor::Enemy(target.key));
        }
        if guide {
            return Some(Actor::Guide(contact.and_then(|k| k.0)));
        }
        entity = parent?.parent();
    }
    None
}
fn pressed(event: On<PointerPress>, nodes: Query<(), With<Node>>, mut gesture: ResMut<Gesture>) {
    if event.pointer.id == PointerId::Mouse {
        gesture.travel = 0.;
        gesture.ui_origin = nodes.get(event.original_event_target()).is_ok();
    }
}
fn dragged(event: On<PointerDrag>, mut gesture: ResMut<Gesture>) {
    if event.pointer.id == PointerId::Mouse {
        gesture.travel = gesture.travel.max(event.distance.length());
    }
}
fn clicked(
    mut event: On<PointerClick>,
    nodes: Query<ActorNode<'_>>,
    gesture: Res<Gesture>,
    mut pending: ResMut<Pending>,
    mut selection: ResMut<Selection>,
    mode: Option<Res<crate::camera::mode::Mode>>,
    windows: Query<&Window>,
) {
    if !windows.single().is_ok_and(|window| window.focused) {
        pending.0 = None;
        event.propagate(false);
        return;
    }
    if mode.as_ref().is_some_and(|mode| mode.active)
        || event.pointer.id != PointerId::Mouse
        || !matches!(
            event.button,
            PointerButton::Primary | PointerButton::Secondary
        )
    {
        return;
    }
    let Some(actor) = actor(event.original_event_target(), &nodes) else {
        return;
    };
    event.propagate(false);
    if gesture.travel > 6. {
        selection.drag_suppressed += 1;
        return;
    }
    if let Some(point) = event.hit.position {
        pending.0 = Some((actor, event.button, point));
    }
}
type MeshForPicking = (Entity, Has<crate::loot::CorpsePick>);
fn mark_meshes(
    mut commands: Commands,
    nodes: Query<ActorNode<'_>>,
    meshes: Query<MeshForPicking, (With<Mesh3d>, Without<bevy::picking::Pickable>)>,
) {
    for (entity, surface) in &meshes {
        if let Some(picked) = actor(entity, &nodes) {
            commands
                .entity(entity)
                .insert(if matches!(picked, Actor::Corpse(_)) && !surface {
                    bevy::picking::Pickable::IGNORE
                } else {
                    bevy::picking::Pickable::default()
                });
        }
    }
}
fn cameras(mut commands: Commands, cameras: Query<Entity, With<Camera3d>>) {
    for camera in &cameras {
        commands.entity(camera).insert(MeshPickingCamera);
    }
}
fn reticle_click(
    mode: Res<crate::camera::mode::Mode>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    nodes: Query<ActorNode<'_>>,
    mut ray_cast: bevy::picking::mesh_picking::ray_cast::MeshRayCast,
    mut pending: ResMut<Pending>,
) {
    if !mode.active || mode.transitioned {
        return;
    }
    let button = if buttons.just_pressed(MouseButton::Left) {
        // Feed the same checked basic-attack path used by classic right-click.
        PointerButton::Secondary
    } else if buttons.just_pressed(MouseButton::Right) {
        PointerButton::Primary
    } else {
        return;
    };
    let Ok((camera, transform)) = camera.single() else {
        return;
    };
    let Some(size) = camera.logical_viewport_size() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, size * 0.5) else {
        return;
    };
    let filter = |entity| actor(entity, &nodes).is_some();
    let settings =
        bevy::picking::mesh_picking::ray_cast::MeshRayCastSettings::default().with_filter(&filter);
    if let Some((entity, hit)) = ray_cast.cast_ray(ray, &settings).first()
        && let Some(picked) = actor(*entity, &nodes)
    {
        // Friendly reticle clicks select only. F remains the interaction key.
        let button = if matches!(picked, Actor::Guide(_)) {
            PointerButton::Primary
        } else {
            button
        };
        pending.0 = Some((picked, button, ray.origin + *ray.direction * hit.distance));
    }
}
// A missing interest/presentation observation is not an entity removal.
fn retain_friendly(selected: EntityKey, epoch: Option<u64>, observed: Option<EntityKey>) -> bool {
    epoch.is_none_or(|e| e == selected.epoch.value()) && observed.is_none_or(|key| key == selected)
}
#[allow(clippy::too_many_arguments)]
fn consume(
    network: Res<crate::network::Network>,
    bindings: Res<Bindings>,
    config: Option<Res<crate::combat::Config>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<Pending>,
    mut selection: ResMut<Selection>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    spatial: SpatialQuery,
    classic: Option<Res<crate::classic_controls::Controller>>,
    mut loot: Option<ResMut<crate::loot::Ui>>,
    windows: Query<&Window>,
) {
    let focused = windows.single().is_ok_and(|window| window.focused);
    let state = network.state();
    if state.error.is_some() || !network.connected() {
        selection.friendly = None;
        pending.0 = None;
        return;
    }
    if focused
        && ((classic.is_none() && keys.just_pressed(KeyCode::Tab))
            || keys.just_pressed(KeyCode::Backspace)
            || (keys.just_pressed(KeyCode::Escape)
                && classic.as_ref().is_some_and(|c| !c.escape_owned_by_panel)))
    {
        selection.friendly = None;
    }
    let resident = |p: theia_core::coordinates::GlobalPosition| {
        p.chunk(128).is_some_and(|id| {
            state
                .chunk_report
                .as_ref()
                .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
        })
    };
    let guide = state
        .guide
        .as_ref()
        .and_then(|p| selection.friendly.and_then(|key| p.find(key)).or(p.guide))
        .filter(|g| resident(g.position));
    if selection.friendly.is_some_and(|key| {
        if classic.is_some() {
            !retain_friendly(
                key,
                state
                    .guide
                    .as_ref()
                    .map(|v| v.actor.epoch.value())
                    .or_else(|| state.targeting.as_ref().map(|v| v.epoch)),
                state
                    .guide
                    .as_ref()
                    .and_then(|v| v.find(key))
                    .map(|g| g.key),
            )
        } else {
            guide.is_none_or(|g| g.key != key)
        }
    }) {
        selection.friendly = None;
    }
    let Some((picked, button, point)) = pending.take_focused(focused) else {
        return;
    };
    let Ok(camera) = cameras.single() else {
        return;
    };
    let delta = point - camera.translation();
    let Ok(direction) = Dir3::new(delta) else {
        return;
    };
    if spatial
        .cast_ray(
            camera.translation(),
            direction,
            (delta.length() - 0.03).max(0.),
            true,
            &SpatialQueryFilter::from_mask(
                theia_network::physics::SOLID | theia_network::physics::GROUND,
            ),
        )
        .is_some()
    {
        selection.blocked_clicks += 1;
        return;
    }
    match picked {
        Actor::Ground(claim) => {
            if button == PointerButton::Secondary
                && state
                    .rewards
                    .as_ref()
                    .is_some_and(|r| r.available.iter().any(|r| r.claim == claim))
            {
                network.request_reward(claim);
            }
        }
        Actor::Corpse(life) => {
            if state.rewards.as_ref().is_some_and(|r| {
                r.available.iter().any(|r| {
                    r.corpse
                        .is_some_and(|c| c.life == life && resident(c.position))
                })
            }) {
                selection.friendly = None;
                if button == PointerButton::Secondary
                    && let Some(ui) = loot.as_mut()
                {
                    ui.show(life);
                }
            }
        }
        Actor::Enemy(key) => {
            if !state.targeting.as_ref().is_some_and(|v| {
                v.targets
                    .iter()
                    .any(|t| t.key == key && resident(t.position))
            }) {
                return;
            }
            selection.friendly = None;
            network.request_target(Some(key.entity));
            selection.enemy_clicks += 1;
            if button == PointerButton::Secondary {
                let requested = if network.autoattack_enabled() {
                    network.request_autoattack(Some(key))
                } else {
                    config.as_ref().is_some_and(|config| {
                        config
                            .0
                            .ability(bindings.basic_attack.as_str())
                            .is_some_and(|ability| {
                                network.request_cast(ability.id().clone(), key.entity)
                            })
                    })
                };
                selection.basic_requests += u64::from(requested);
            }
        }
        Actor::Guide(key) => {
            if let Some(ui) = loot.as_mut() {
                ui.close();
            }
            let Some(guide) = state
                .guide
                .as_ref()
                .and_then(|p| match key {
                    Some(key) => p.find(key),
                    None if p.contacts.is_empty() => p.guide,
                    None => None,
                })
                .filter(|g| resident(g.position))
            else {
                return;
            };
            selection.friendly = Some(guide.key);
            network.request_target(None);
            selection.friendly_clicks += 1;
            if button == PointerButton::Secondary
                && let Some(view) = state.dialogue.as_ref()
                && !state.dialogue_pending
                && network.request_dialogue(
                    view.revision,
                    theia_network::dialogue::Action::Open(guide.key),
                )
            {
                selection.interaction_requests += 1;
            }
        }
    }
}
pub(crate) fn install(app: &mut App) {
    crate::camera::mode::install(app);
    qualification::install(app);
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Raw {
        schema: u32,
        basic_attack: String,
    }
    let raw: Raw = serde_json::from_slice(include_bytes!(
        "../../../content/input/development-controls.json"
    ))
    .expect("Built-in control bindings");
    assert_eq!(raw.schema, 1);
    app.insert_resource(Bindings {
        basic_attack: ContentId::new(&raw.basic_attack).expect("Basic attack content ID"),
    })
    .init_resource::<Selection>()
    .init_resource::<Pending>()
    .init_resource::<Gesture>()
    .insert_resource(MeshPickingSettings {
        require_markers: true,
        ..default()
    })
    .add_plugins(MeshPickingPlugin)
    .add_observer(clicked)
    .add_observer(pressed)
    .add_observer(dragged)
    .add_systems(Startup, cameras.after(crate::scene::setup))
    .add_systems(
        PreUpdate,
        track
            .after(bevy::input::InputSystems)
            .before(PickingSystems::ProcessInput),
    )
    .add_systems(Update, mark_meshes)
    .add_systems(
        Update,
        reticle_click.after(crate::network::present).before(consume),
    )
    .add_systems(
        Update,
        consume
            .in_set(Interactions)
            .after(crate::targeting::controls)
            .after(crate::network::present)
            .before(crate::targeting::Presentation)
            .before(crate::combat::Presentation),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::picking::{backend::HitData, events::Pointer, pointer::Location};
    use std::time::Duration;
    use theia_core::identity::{EntityId, WorldEpoch};
    #[test]
    fn corpse_art_ignores_bind_pose_picking_and_uses_only_nonblocking_pick_surface() {
        let mut app = App::new();
        app.add_systems(Update, mark_meshes);
        let key = EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(9).unwrap(),
        };
        let owner = app
            .world_mut()
            .spawn(crate::loot::CorpseVisual(
                theia_core::rewards::CorpseSource {
                    life: theia_core::progression::KillId::from_victim(key),
                    position: theia_core::coordinates::GlobalPosition::new([0.; 3]).unwrap(),
                },
            ))
            .id();
        let skin = app
            .world_mut()
            .spawn((Mesh3d::default(), ChildOf(owner)))
            .id();
        let surface = app
            .world_mut()
            .spawn((Mesh3d::default(), ChildOf(owner), crate::loot::CorpsePick))
            .id();
        let live = app
            .world_mut()
            .spawn((Mesh3d::default(), crate::targeting::TargetVisual { key }))
            .id();
        app.update();
        assert!(
            !app.world()
                .get::<bevy::picking::Pickable>(skin)
                .unwrap()
                .is_hoverable
        );
        assert!(
            app.world()
                .get::<bevy::picking::Pickable>(surface)
                .unwrap()
                .is_hoverable
        );
        assert!(
            app.world()
                .get::<bevy::picking::Pickable>(live)
                .unwrap()
                .is_hoverable
        );
        assert!(
            app.world()
                .get::<avian3d::prelude::Collider>(surface)
                .is_none()
        );
    }
    #[test]
    fn friendly_survives_missing_interest_but_not_a_known_replacement_or_world_change() {
        let key = EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        assert!(retain_friendly(key, Some(7), None));
        assert!(retain_friendly(key, None, None));
        assert!(retain_friendly(key, Some(7), Some(key)));
        assert!(!retain_friendly(key, Some(8), None));
        assert!(!retain_friendly(
            key,
            Some(7),
            Some(EntityKey {
                entity: EntityId::new(2).unwrap(),
                ..key
            })
        ));
    }
    #[test]
    fn camera_waits_for_drag_and_ui_press_retains_pointer_ownership() {
        use bevy::{
            input::mouse::AccumulatedMouseScroll,
            window::{CursorGrabMode, CursorOptions},
        };
        let mut app = app();
        app.init_resource::<crate::camera::Orbit>()
            .init_resource::<crate::inventory::Ui>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .add_systems(Update, crate::camera::controls);
        app.world_mut().resource_mut::<crate::inventory::Ui>().open = true;
        let world = app.world_mut();
        let window = world
            .query_filtered::<Entity, With<Window>>()
            .single(world)
            .unwrap();
        world.entity_mut(window).insert(CursorOptions::default());
        let original = app.world().resource::<crate::camera::Orbit>().angles();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(4., 0.);
        app.update();
        assert_eq!(
            app.world().resource::<crate::camera::Orbit>().angles(),
            original
        );
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::None
        );
        app.world_mut().resource_mut::<Gesture>().travel = 10.;
        app.update();
        assert_ne!(
            app.world().resource::<crate::camera::Orbit>().angles(),
            original
        );
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::Locked
        );
        let after = app.world().resource::<crate::camera::Orbit>().angles();
        let ui = app.world_mut().spawn(Node::default()).id();
        app.world_mut().trigger(PointerPress {
            entity: ui,
            pointer: pointer(window),
            button: PointerButton::Secondary,
            hit: HitData::new(ui, 0., None, None),
            count: 1,
        });
        app.world_mut().resource_mut::<Gesture>().travel = 50.;
        app.update();
        assert_eq!(
            app.world().resource::<crate::camera::Orbit>().angles(),
            after
        );
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::None
        );
    }
    fn pointer(window: Entity) -> Pointer {
        Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                    .normalize(Some(window))
                    .unwrap(),
                position: Vec2::new(100., 100.),
            },
        )
    }
    fn click(world: &mut World, entity: Entity, button: PointerButton) {
        let window = world
            .query_filtered::<Entity, With<Window>>()
            .iter(world)
            .next()
            .unwrap_or(entity);
        world.trigger(PointerClick {
            entity,
            pointer: pointer(window),
            button,
            hit: HitData::new(entity, 3., Some(Vec3::new(0., 1., 3.)), None),
            duration: Duration::from_millis(100),
            count: 1,
        });
    }
    fn app() -> App {
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.init_resource::<Pending>()
            .init_resource::<Gesture>()
            .init_resource::<Selection>()
            .add_observer(clicked)
            .add_observer(pressed)
            .add_observer(dragged);
        app
    }
    #[test]
    fn unfocused_or_missing_window_clicks_discard_pending_actions_but_keep_selection() {
        for mode in 0..3 {
            let mut app = app();
            let guide = app.world_mut().spawn(crate::dialogue::GuideVisual).id();
            let key = EntityKey {
                entity: EntityId::new(2).unwrap(),
                epoch: WorldEpoch::new(7).unwrap(),
            };
            app.world_mut().resource_mut::<Selection>().friendly = Some(key);
            click(app.world_mut(), guide, PointerButton::Secondary);
            assert!(app.world().resource::<Pending>().0.is_some());
            let world = app.world_mut();
            let window = world
                .query_filtered::<Entity, With<Window>>()
                .single(world)
                .unwrap();
            match mode {
                0 => world.get_mut::<Window>(window).unwrap().focused = false,
                1 => {
                    world.despawn(window);
                }
                _ => {
                    world.spawn(Window {
                        focused: true,
                        ..default()
                    });
                }
            }
            click(app.world_mut(), guide, PointerButton::Secondary);
            assert!(app.world().resource::<Pending>().0.is_none());
            assert_eq!(app.world().resource::<Selection>().friendly, Some(key));
            assert_eq!(app.world().resource::<Selection>().interaction_requests, 0);
        }
    }
    #[test]
    fn focus_loss_between_click_and_consumption_does_not_replay_after_refocus() {
        let mut app = app();
        let guide = app.world_mut().spawn(crate::dialogue::GuideVisual).id();
        click(app.world_mut(), guide, PointerButton::Secondary);
        assert!(app.world().resource::<Pending>().0.is_some());
        assert!(
            app.world_mut()
                .resource_mut::<Pending>()
                .take_focused(false)
                .is_none()
        );
        assert!(
            app.world_mut()
                .resource_mut::<Pending>()
                .take_focused(true)
                .is_none()
        );
        click(app.world_mut(), guide, PointerButton::Primary);
        let choice = app
            .world_mut()
            .resource_mut::<Pending>()
            .take_focused(true)
            .unwrap();
        assert!(matches!(choice.0, Actor::Guide(_)));
        assert_eq!(choice.1, PointerButton::Primary);
    }
    #[test]
    fn child_mesh_click_keeps_exact_enemy_key_and_coalesces_pending_commands() {
        let mut app = app();
        let key = EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(2).unwrap(),
        };
        let root = app
            .world_mut()
            .spawn(crate::targeting::TargetVisual { key })
            .id();
        let mesh = app.world_mut().spawn(ChildOf(root)).id();
        for _ in 0..100 {
            click(app.world_mut(), mesh, PointerButton::Primary);
        }
        let pending = app.world().resource::<Pending>().0.unwrap();
        assert!(matches!(pending.0,Actor::Enemy(k) if k==key));
        assert_eq!(pending.1, PointerButton::Primary);
        click(app.world_mut(), mesh, PointerButton::Secondary);
        assert_eq!(
            app.world().resource::<Pending>().0.unwrap().1,
            PointerButton::Secondary
        );
    }
    #[test]
    fn friendly_click_is_distinct_and_ui_and_unknown_meshes_do_not_select() {
        let mut app = app();
        let guide = app.world_mut().spawn(crate::dialogue::GuideVisual).id();
        click(app.world_mut(), guide, PointerButton::Primary);
        assert!(matches!(
            app.world().resource::<Pending>().0.unwrap().0,
            Actor::Guide(_)
        ));
        app.world_mut().resource_mut::<Pending>().0 = None;
        let ui = app
            .world_mut()
            .spawn((Node::default(), ChildOf(guide)))
            .id();
        click(app.world_mut(), ui, PointerButton::Primary);
        let scenery = app.world_mut().spawn_empty().id();
        click(app.world_mut(), scenery, PointerButton::Secondary);
        assert!(app.world().resource::<Pending>().0.is_none());
    }
    #[test]
    fn right_drag_suppresses_attack_and_next_press_starts_a_fresh_click() {
        let mut app = app();
        let world = app.world_mut();
        let window = world
            .query_filtered::<Entity, With<Window>>()
            .single(world)
            .unwrap();
        let guide = app.world_mut().spawn(crate::dialogue::GuideVisual).id();
        app.world_mut().trigger(PointerDrag {
            entity: guide,
            pointer: pointer(window),
            button: PointerButton::Secondary,
            distance: Vec2::new(8., 0.),
            delta: Vec2::new(8., 0.),
        });
        click(app.world_mut(), guide, PointerButton::Secondary);
        assert!(app.world().resource::<Pending>().0.is_none());
        assert_eq!(app.world().resource::<Selection>().drag_suppressed, 1);
        app.world_mut().trigger(PointerPress {
            entity: guide,
            pointer: pointer(window),
            button: PointerButton::Secondary,
            hit: HitData::new(guide, 3., Some(Vec3::ZERO), None),
            count: 1,
        });
        click(app.world_mut(), guide, PointerButton::Secondary);
        assert!(app.world().resource::<Pending>().0.is_some());
    }
}
