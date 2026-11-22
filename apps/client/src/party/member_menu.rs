//! Context actions bind to the painted party revision and exact member life.
use super::{Control, Ui, button, intent, label};
use bevy::{
    picking::{
        events::{PointerClick, PointerDrag, PointerPress},
        pointer::{PointerButton, PointerId},
    },
    prelude::*,
};
use theia_core::{identity::EntityKey, party::Action};
use theia_network::{party_client::Intent, party_wire::State};

#[derive(Clone, Debug)]
pub(super) struct Selection {
    owner: EntityKey,
    revision: u64,
    target: EntityKey,
    anchor: Vec2,
}
impl Selection {
    fn new(state: &State, target: EntityKey, anchor: Vec2) -> Option<Self> {
        let selected = Self {
            owner: state.view.owner,
            revision: state.view.revision,
            target,
            anchor,
        };
        selected.valid(state).then_some(selected)
    }
    fn valid(&self, state: &State) -> bool {
        self.anchor.is_finite()
            && self.owner == state.view.owner
            && self.revision == state.view.revision
            && self.target != self.owner
            && state
                .view
                .membership
                .party
                .as_ref()
                .is_some_and(|p| p.members.contains(&self.target))
    }
}
pub(super) fn current(state: &State, selected: &Selection, painted: &Intent) -> bool {
    selected.valid(state)
        && painted.owner == selected.owner
        && painted.revision == selected.revision
        && state
            .view
            .membership
            .party
            .as_ref()
            .is_some_and(|p| p.leader == selected.owner)
        && matches!(painted.action, Action::Promote(key) | Action::Kick(key) if key == selected.target)
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Model {
    name: String,
    leader: bool,
    choices: Vec<(String, Intent)>,
    enabled: bool,
}
fn model(state: &State, selected: &Selection, enabled: bool) -> Option<Model> {
    if !selected.valid(state) {
        return None;
    }
    let group = state.view.membership.party.as_ref()?;
    let name = state
        .roster
        .iter()
        .find(|a| a.key == selected.target)?
        .name
        .as_str()
        .to_owned();
    let choices = if group.leader == state.view.owner {
        vec![
            (
                "Promote to leader".into(),
                intent(state, Action::Promote(selected.target)),
            ),
            (
                "Remove from party".into(),
                intent(state, Action::Kick(selected.target)),
            ),
        ]
    } else {
        vec![]
    };
    Some(Model {
        name,
        leader: group.leader == selected.target,
        choices,
        enabled,
    })
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Panel;
#[derive(Resource, Default)]
pub(super) struct Presentation {
    model: Option<Model>,
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<Presentation>()
        .add_systems(Startup, setup)
        .add_observer(pressed)
        .add_observer(dragged)
        .add_observer(clicked);
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Root,
        GlobalZIndex(87),
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            display: Display::None,
            ..default()
        },
    ));
}
fn pressed(event: On<PointerPress>, mut ui: ResMut<Ui>) {
    if event.pointer.id == PointerId::Mouse {
        ui.menu_dragged = false;
    }
}
fn dragged(event: On<PointerDrag>, mut ui: ResMut<Ui>) {
    if event.pointer.id == PointerId::Mouse && event.distance.length() > 6. {
        ui.menu_dragged = true;
    }
}
fn clicked(
    mut event: On<PointerClick>,
    frames: Query<(
        &super::members::Frame,
        &ComputedNode,
        &bevy::ui::UiGlobalTransform,
    )>,
    roots: Query<(), With<Root>>,
    windows: Query<&Window>,
    network: Res<crate::network::Network>,
    mut ui: ResMut<Ui>,
    mut selection: Option<ResMut<crate::world_input::Selection>>,
) {
    if event.pointer.id != PointerId::Mouse || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let target = event.original_event_target();
    if roots.contains(target) {
        // The backdrop consumes dismissal; it cannot become an attack or drag.
        ui.menu = None;
        event.propagate(false);
        return;
    }
    if !matches!(
        event.button,
        PointerButton::Primary | PointerButton::Secondary
    ) || ui.menu_dragged
    {
        return;
    }
    let Ok((frame, node, transform)) = frames.get(target) else {
        return;
    };
    event.propagate(false);
    let state = network.state();
    if state.error.is_some() {
        ui.menu = None;
        return;
    }
    if event.button == PointerButton::Primary {
        if super::selection::member(&state, frame.0).is_some()
            && let Some(selection) = selection.as_mut()
        {
            network.request_target(Some(frame.0.entity));
            selection.friendly = None;
            ui.menu = None;
        }
        return;
    }
    let anchor = (transform.affine().translation
        + Vec2::new(node.size().x / 2., -node.size().y / 2.))
        * node.inverse_scale_factor()
        + Vec2::new(8., 0.);
    ui.menu = state
        .party
        .as_ref()
        .and_then(|view| Selection::new(&view.state, frame.0, anchor));
}
fn position(anchor: Vec2, viewport: Vec2) -> Vec2 {
    anchor.clamp(
        Vec2::splat(12.),
        (viewport - Vec2::new(248., 292.)).max(Vec2::splat(12.)),
    )
}
#[allow(clippy::too_many_arguments)]
pub(super) fn present(
    mut commands: Commands,
    network: Res<crate::network::Network>,
    mut ui: ResMut<Ui>,
    interface: Option<Res<crate::fantasy_ui::settings::Ui>>,
    windows: Query<&Window>,
    scale: Res<bevy::ui::UiScale>,
    mut presentation: ResMut<Presentation>,
    mut roots: Query<(Entity, &mut Node), With<Root>>,
    mut panels: Query<&mut Node, (With<Panel>, Without<Root>)>,
) {
    let state = network.state();
    let window = windows.single().ok();
    if state.error.is_some()
        || !window.is_some_and(|w| w.focused)
        || interface.as_ref().is_some_and(|u| u.open)
    {
        ui.menu = None;
    }
    let next = ui.menu.as_ref().and_then(|selected| {
        state
            .party
            .as_ref()
            .and_then(|view| model(&view.state, selected, !view.pending && ui.waiting.is_none()))
    });
    if next.is_none() {
        ui.menu = None;
    }
    let location =
        ui.menu.as_ref().zip(window).map(|(selected, w)| {
            position(selected.anchor, Vec2::new(w.width(), w.height()) / scale.0)
        });
    for (entity, mut node) in &mut roots {
        let display = if next.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if presentation.model == next {
            continue;
        }
        commands
            .entity(entity)
            .despawn_children()
            .with_children(|root| {
                if let Some(menu) = &next {
                    let at = location.unwrap();
                    root.spawn((
                        Panel,
                        crate::ui_theme::Panel,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(at.x),
                            top: px(at.y),
                            width: px(236),
                            padding: UiRect::all(px(12)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            ..default()
                        },
                        BackgroundColor(crate::fantasy_ui::PANEL),
                        Outline::new(px(1), px(-1), crate::fantasy_ui::GOLD),
                    ))
                    .with_children(|panel| {
                        panel
                            .spawn(label(&menu.name))
                            .insert(TextColor(crate::fantasy_ui::GOLD));
                        panel.spawn(label(if menu.leader {
                            "Party leader"
                        } else {
                            "Party member"
                        }));
                        for title in ["Whisper", "Inspect", "Set Role"] {
                            crate::fantasy_ui::kit::disabled(panel, title);
                        }
                        panel
                            .spawn(button(Control::MarksToggle, "Set Marker", true))
                            .with_child(label("Set Marker  ▸"));
                        for (title, intent) in &menu.choices {
                            panel
                                .spawn(button(Control::Member(intent.clone()), title, menu.enabled))
                                .with_child(label(title));
                        }
                        panel
                            .spawn(button(Control::Manage, "Manage party", true))
                            .with_child(label("Manage party"));
                        panel
                            .spawn(button(Control::MenuClose, "Close member menu", true))
                            .with_child(label("Close [Esc]"));
                    });
                }
            });
    }
    if let Some(at) = location {
        for mut node in &mut panels {
            if node.left != px(at.x) {
                node.left = px(at.x);
            }
            if node.top != px(at.y) {
                node.top = px(at.y);
            }
        }
    }
    presentation.model = next;
}
pub(super) fn snapshot(world: &mut World) -> serde_json::Value {
    let menu = world.resource::<Presentation>().model.clone();
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let bounds = world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Panel>>()
        .iter(world)
        .find(|(node, _)| menu.is_some() && node.size().min_element() > 0.)
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
    serde_json::json!({"visible":menu.is_some(),"name":menu.as_ref().map(|m| &m.name),"bounds":bounds})
}

#[cfg(test)]
mod tests {
    use super::super::tests::{key, state};
    use super::*;
    use theia_core::{
        party::{self, Request},
        time::TickRate,
    };
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
    fn grouped() -> party::Authority {
        let mut authority = party::Authority::new(key(1).epoch, TickRate::Hz30);
        for id in 1..=3 {
            authority.admit(key(id)).unwrap();
        }
        authority.advance(1).unwrap();
        send(&mut authority, 1, Action::Invite(key(2)));
        let invitation = authority
            .view(key(2))
            .unwrap()
            .membership
            .incoming
            .unwrap()
            .id;
        send(&mut authority, 2, Action::Accept(invitation));
        authority
    }
    #[test]
    fn member_actions_require_painted_owner_revision_membership_and_leadership() {
        let mut authority = grouped();
        let original = state(&authority, 1);
        let selected = Selection::new(&original, key(2), Vec2::new(288., 127.)).unwrap();
        let menu = model(&original, &selected, true).unwrap();
        assert_eq!(menu.name, "Briar");
        assert_eq!(menu.choices.len(), 2);
        for (_, action) in &menu.choices {
            assert!(current(&original, &selected, action));
        }
        assert!(Selection::new(&original, key(1), Vec2::ZERO).is_none());
        assert!(Selection::new(&original, key(3), Vec2::ZERO).is_none());
        assert!(Selection::new(&original, key(2), Vec2::NAN).is_none());
        let guest = state(&authority, 2);
        let guest_selected = Selection::new(&guest, key(1), Vec2::ZERO).unwrap();
        let guest_menu = model(&guest, &guest_selected, true).unwrap();
        assert!(guest_menu.leader && guest_menu.choices.is_empty());
        assert!(!current(
            &guest,
            &guest_selected,
            &intent(&guest, Action::Kick(key(1)))
        ));
        assert!(!current(
            &original,
            &selected,
            &intent(&original, Action::Kick(key(3)))
        ));
        assert!(!current(
            &original,
            &selected,
            &intent(&original, Action::Leave)
        ));
        assert!(model(&guest, &selected, true).is_none());
        let mut updated = original.clone();
        updated.tick += 1;
        updated.member_vitals[1].vitals = theia_core::resources::Vitals::new(18, 100, 24, 100);
        assert_eq!(model(&updated, &selected, true), Some(menu.clone()));
        assert!(!model(&updated, &selected, false).unwrap().enabled);
        send(&mut authority, 1, Action::Promote(key(2)));
        let promoted = state(&authority, 1);
        assert!(model(&promoted, &selected, true).is_none());
        assert!(!current(&promoted, &selected, &menu.choices[1].1));
        let new_menu = Selection::new(&promoted, key(2), Vec2::ZERO).unwrap();
        assert!(
            model(&promoted, &new_menu, true)
                .unwrap()
                .choices
                .is_empty()
        );
        send(&mut authority, 2, Action::Kick(key(1)));
        assert!(model(&state(&authority, 1), &new_menu, true).is_none());
    }
    #[test]
    fn escape_closes_context_before_party_and_focus_loss_retires_it() {
        let mut app = App::new();
        let selected = Selection::new(&state(&grouped(), 1), key(2), Vec2::ZERO).unwrap();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Ui {
                open: true,
                menu: Some(selected.clone()),
                ..default()
            })
            .add_systems(Update, super::super::keyboard);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        let ui = app.world().resource::<Ui>();
        assert!(ui.open && ui.menu.is_none() && ui.closed_this_frame);
        assert!(super::super::owns_escape(Some(ui)));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(!app.world().resource::<Ui>().open);
        app.world_mut().resource_mut::<Ui>().menu = Some(selected);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.update();
        assert!(app.world().resource::<Ui>().menu.is_none());
    }
    #[test]
    fn popup_position_preserves_margins_at_both_tile_heights_and_edges() {
        for height in [700., 1400.] {
            for anchor in [
                Vec2::new(288., 127.),
                Vec2::new(1700., height - 10.),
                Vec2::new(-100., -100.),
            ] {
                let at = position(anchor, Vec2::new(1720., height));
                assert!(
                    at.x >= 12.
                        && at.y >= 12.
                        && at.x + 236. <= 1708.
                        && at.y + 280. <= height - 12.
                );
            }
        }
    }
}
