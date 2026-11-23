//! Standard Bevy informational guide/panel. Every displayed node is observed;
//! widget choice and movement never advance dialogue optimistically.
mod motion;
pub(crate) mod names;
mod qualification;
use crate::{floating_origin::Frame, network::Network, scene::PawnAssets};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::{FontSize, LineBreak},
    ui::Pressed,
    ui_widgets::{Activate, Button},
};
use std::sync::Arc;
use theia_network::dialogue::{Action, Profile, Rejection};
#[derive(Resource)]
pub(crate) struct Config(pub(crate) Arc<Profile>);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Ui {
    pub open: bool,
    pub pending: bool,
    pub close_when_open: bool,
    pub node: Option<String>,
    pub displayed_revision: Option<u64>,
    #[serde(skip)]
    choices: Vec<(u64, theia_core::content::ContentId)>,
    pub pointer_actions: u64,
    pub keyboard_actions: u64,
    pub submitted: u64,
    pub visible_guide_frames: u64,
    pub open_frames: u64,
    pub scroll_events: u64,
    pub scroll_delta: f32,
    pub scroll_maximum: f32,
}
#[derive(Resource, Default)]
struct Pending {
    choice: Option<(u64, theia_core::content::ContentId)>,
    close: Option<u64>,
}
#[derive(Component)]
pub(super) struct GuideVisual;
#[derive(Component)]
pub(super) struct ContactDefinition(theia_core::content::ContentId);
#[derive(Component, Default)]
pub(super) struct ContactKey(pub Option<theia_core::identity::EntityKey>);
#[derive(Component)]
struct Panel;
#[derive(Component)]
pub(crate) struct Pane;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GuideUiSetup;
#[derive(Component)]
struct Body;
#[derive(Component)]
struct Title;
#[derive(Component)]
struct DialogueText;
#[derive(Component)]
struct Status;
#[derive(Component)]
struct Prompt;
#[derive(Component)]
struct ChoiceLabel(usize);
#[derive(Component, Clone, Copy)]
pub(super) enum Widget {
    Choice(usize),
    Close,
}
fn activate(
    event: On<Activate>,
    buttons: Query<&Widget>,
    mut pending: ResMut<Pending>,
    mut ui: ResMut<Ui>,
) {
    if !ui.open || ui.pending {
        return;
    }
    if let Ok(widget) = buttons.get(event.entity) {
        match widget {
            Widget::Choice(index) => pending.choice = ui.choices.get(*index).cloned(),
            Widget::Close => pending.close = ui.displayed_revision,
        }
        ui.pointer_actions += 1;
    }
}
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct InputControls;
pub(crate) fn install(app: &mut App, profile: Arc<Profile>) {
    app.insert_resource(Config(profile))
        .init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate)
        .add_systems(
            Startup,
            setup.after(crate::scene::setup).in_set(GuideUiSetup),
        )
        .add_systems(
            Update,
            (controls.in_set(InputControls), present, scroll)
                .chain()
                .after(crate::network::present),
        );
    qualification::install(app);
    names::install(app);
}
fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::linebreak(LineBreak::WordOrCharacter),
        Node {
            width: percent(100),
            flex_shrink: 0.,
            ..default()
        },
    )
}
fn button(widget: Widget) -> impl Bundle {
    (
        widget,
        Button,
        Hovered::default(),
        Node {
            width: percent(100),
            min_height: px(40),
            padding: UiRect::all(px(8)),
            align_items: AlignItems::Center,
            flex_shrink: 0.,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.14, 0.19)),
    )
}
fn setup(
    mut commands: Commands,
    pawns: Res<PawnAssets>,
    config: Res<Config>,
    journal: Option<Res<crate::quest_journal::Definitions>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
) {
    for contact in config.0.contacts() {
        commands.spawn((
            GuideVisual,
            ContactDefinition(contact.npc().clone()),
            ContactKey::default(),
            motion::Motion::default(),
            Mesh3d(pawns.mesh.clone()),
            MeshMaterial3d(materials.add(Color::srgb(0.1, 0.65, 0.72))),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    // The expanded journal supplies this context beside its committed progress;
    // retain the legacy fixed-position prompt only in the legacy presentation.
    if journal.is_none() {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(12),
                    top: px(268),
                    width: px(340),
                    padding: UiRect::all(px(10)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.9)),
                Pickable::IGNORE,
            ))
            .with_child((Prompt, text("Guide: waiting for server", 14.)));
    }
    commands
        .spawn((
            Panel,
            Node {
                position_type: PositionType::Absolute,
                left: px(if skin.is_some() { 12. } else { 0. }),
                right: if skin.is_some() { Val::Auto } else { px(0) },
                width: if skin.is_some() { px(420) } else { Val::Auto },
                max_width: if skin.is_some() {
                    percent(45)
                } else {
                    Val::Auto
                },
                top: if skin.is_some() { px(64) } else { percent(20) },
                height: percent(if skin.is_some() { 70. } else { 50. }),
                max_height: if skin.is_some() {
                    px(crate::fantasy_ui::PAPER_MAX_HEIGHT)
                } else {
                    Val::Auto
                },
                justify_content: if skin.is_some() {
                    JustifyContent::FlexStart
                } else {
                    JustifyContent::Center
                },
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Pane,
                crate::fantasy_ui::Panel::Paper,
                Node {
                    width: px(if skin.is_some() { 420. } else { 480. }),
                    max_width: percent(if skin.is_some() { 100. } else { 90. }),
                    height: percent(100),
                    padding: UiRect::all(px(14)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.05, 0.08, 0.98)),
            ))
            .with_children(|panel| {
                if skin.is_some() {
                    crate::fantasy_ui::window_header(
                        panel,
                        (Title, text(config.0.profile().name(), 20.)),
                        button(Widget::Close),
                        "Close dialogue",
                    );
                } else {
                    panel.spawn((Title, text(config.0.profile().name(), 20.)));
                }
                panel
                    .spawn((
                        Body,
                        Hovered::default(),
                        ScrollPosition::default(),
                        Node {
                            width: percent(100),
                            min_height: px(0),
                            flex_grow: 1.,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        body.spawn((DialogueText, text("Waiting for server...", 16.)));
                        for index in 0..theia_core::dialogue::MAX_CHOICES {
                            body.spawn(button(Widget::Choice(index)))
                                .with_child((ChoiceLabel(index), text("", 16.)));
                        }
                    });
                panel.spawn((Status, text("", 14.)));
                panel
                    .spawn(button(Widget::Close))
                    .with_child(text("Close [Esc]", 16.));
            });
        });
}
#[allow(clippy::too_many_arguments)]
fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    network: Res<Network>,
    mut pending: ResMut<Pending>,
    mut ui: ResMut<Ui>,
    classic: Option<Res<crate::classic_controls::Controller>>,
    selection: Option<Res<crate::world_input::Selection>>,
    loot: Option<Res<crate::loot::Ui>>,
    documents: Option<Res<crate::documents::Ui>>,
    (interface, book): (
        Option<Res<crate::fantasy_ui::settings::Ui>>,
        Option<Res<crate::combat::book::Ui>>,
    ),
    party: Option<Res<crate::party::Ui>>,
) {
    let state = network.state();
    let pointer_choice = pending.choice.take();
    let pointer_close = pending.close.take();
    if state.error.is_some() {
        ui.close_when_open = false;
        return;
    }
    let Some(view) = &state.dialogue else {
        return;
    };
    let loot_owns = crate::party::owns_escape(party.as_deref())
        || crate::combat::book::owns_escape(book.as_deref())
        || crate::fantasy_ui::settings::owns_escape(interface.as_deref())
        || loot.as_ref().is_some_and(|u| u.open || u.closed_this_frame)
        || documents
            .as_ref()
            .is_some_and(|u| u.open || u.closed_this_frame);
    let escape = keys.just_pressed(KeyCode::Escape) && !loot_owns;
    if escape && state.dialogue_pending {
        ui.close_when_open = true;
    }
    if state.dialogue_pending {
        return;
    }
    let action = if view.open.is_some() {
        if pointer_close.is_some() || escape || ui.close_when_open {
            let revision = close_revision(view.revision, escape, ui.close_when_open, pointer_close);
            ui.close_when_open = false;
            if escape {
                ui.keyboard_actions += 1;
            }
            revision.map(|revision| (revision, Action::Close))
        } else {
            let keyboard = [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4]
                .iter()
                .position(|key| keys.just_pressed(*key));
            if keyboard.is_some() {
                ui.keyboard_actions += 1;
            }
            pointer_choice
                .or_else(|| keyboard.and_then(|index| ui.choices.get(index).cloned()))
                .map(|(revision, choice)| (revision, Action::Choose(choice)))
        }
    } else {
        ui.close_when_open = false;
        if !loot_owns
            && (keys.just_pressed(KeyCode::KeyF)
                || (classic.is_none() && keys.just_pressed(KeyCode::KeyE)))
        {
            ui.keyboard_actions += 1;
            state
                .guide
                .as_ref()
                .and_then(|p| {
                    // Explicit friendly selection wins; never substitute another NPC.
                    if let Some(key) = selection.as_ref().and_then(|s| s.friendly) {
                        return p.find(key);
                    }
                    let focus = state.authority_focus()?.metres();
                    p.guides().min_by(|a, b| {
                        let d = |g: &theia_network::dialogue::Guide| {
                            g.position
                                .metres()
                                .into_iter()
                                .zip(focus)
                                .map(|(a, b)| (a - b).powi(2))
                                .sum::<f64>()
                        };
                        d(a).total_cmp(&d(b))
                    })
                })
                .filter(|guide| {
                    state
                        .authority_focus()
                        .is_some_and(|p| theia_network::dialogue::in_range(p, guide.position))
                })
                .map(|guide| (view.revision, Action::Open(guide.key)))
        } else {
            None
        }
    };
    if let Some((revision, action)) = action
        && network.request_dialogue(revision, action)
    {
        ui.submitted += 1;
    }
}

fn close_revision(
    current: u64,
    escape: bool,
    deferred_escape: bool,
    pointer: Option<u64>,
) -> Option<u64> {
    // Escape closes the current conversation, including a response that arrived
    // before presentation refreshed. Pointer actions retain the painted identity.
    if escape || deferred_escape {
        Some(current)
    } else {
        pointer
    }
}

type Label<'a> = (
    &'a mut Text,
    Option<&'a Title>,
    Option<&'a DialogueText>,
    Option<&'a Status>,
    Option<&'a Prompt>,
    Option<&'a ChoiceLabel>,
);
type Visual<'a> = (
    &'a mut Visibility,
    Option<&'a GuideVisual>,
    Option<&'a Panel>,
    Option<&'a mut Transform>,
    Option<&'a ContactDefinition>,
    Option<&'a mut ContactKey>,
    Option<&'a mut motion::Motion>,
);
type VisualFilter = Or<(With<GuideVisual>, With<Panel>)>;
type ButtonView<'a> = (
    &'a Widget,
    &'a mut Node,
    &'a mut BackgroundColor,
    Option<&'a Hovered>,
    Has<Pressed>,
);
#[allow(clippy::too_many_arguments)]
fn present(
    time: Res<Time>,
    network: Res<Network>,
    config: Res<Config>,
    frame: Res<Frame>,
    mut ui: ResMut<Ui>,
    mut visuals: Query<Visual<'_>, VisualFilter>,
    mut labels: Query<Label<'_>>,
    mut buttons: Query<ButtonView<'_>>,
    mut bodies: Query<&mut ScrollPosition, With<Body>>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
) {
    let state = network.state();
    let view = state.dialogue.as_ref().filter(|_| state.error.is_none());
    let open = view.and_then(|view| view.open.as_ref());
    let dialogue_node = open.and_then(|open| config.0.node(&open.node));
    let node_id = dialogue_node.map(|node| node.id().as_str().to_owned());
    ui.displayed_revision = view.map(|view| view.revision);
    ui.choices = dialogue_node.map_or_else(Vec::new, |node| {
        node.choices()
            .iter()
            .map(|choice| (view.unwrap().revision, choice.id().clone()))
            .collect()
    });
    if ui.node != node_id {
        for mut scroll in &mut bodies {
            *scroll = ScrollPosition::default();
        }
        ui.node = node_id;
    }
    ui.open = open.is_some();
    ui.pending = state.error.is_none() && state.dialogue_pending;
    if ui.open {
        ui.open_frames += 1;
    }
    let guide = state
        .guide
        .as_ref()
        .and_then(|p| p.guide)
        .filter(|_| state.error.is_none());
    let guide = guide.filter(|guide| {
        guide.position.chunk(128).is_some_and(|id| {
            state
                .chunk_report
                .as_ref()
                .is_some_and(|report| report.chunks.contains(&[id.x, id.z]))
        })
    });
    for (mut visibility, is_guide, is_panel, transform, definition, key, motion) in &mut visuals {
        if is_panel.is_some() {
            *visibility = if ui.open {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if is_guide.is_some() {
            let contact = state
                .guide
                .as_ref()
                .and_then(|p| {
                    if config.0.catalog().is_none() {
                        return p.guide;
                    }
                    p.contacts
                        .iter()
                        .find(|c| definition.is_some_and(|d| d.0 == c.npc))
                        .map(|c| c.guide)
                })
                .filter(|g| {
                    state.error.is_none()
                        && g.position.chunk(128).is_some_and(|id| {
                            state
                                .chunk_report
                                .as_ref()
                                .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
                        })
                });
            if let Some(mut key) = key {
                key.0 = contact.map(|c| c.key);
            }
            let tick_seconds = state
                .snapshot
                .as_ref()
                .map_or(1. / 30., |s| 1. / s.tick_hz as f32);
            let position = motion
                .and_then(|mut motion| motion.sample(contact, time.delta_secs(), tick_seconds));
            if let Some(local) = position.and_then(|position| frame.render.to_local(position)) {
                transform.unwrap().translation = Vec3::from_array(local.metres());
                *visibility = Visibility::Inherited;
                ui.visible_guide_frames += 1;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
    let status = if ui.pending {
        "Waiting for response..."
    } else {
        match state.dialogue_reply.as_ref().and_then(|r| r.rejection) {
            Some(Rejection::OutOfRange) => "Move closer to talk.",
            Some(_) => "That choice is unavailable. Please try again.",
            None => "",
        }
    };
    for (mut label, title, body, is_status, prompt, choice) in &mut labels {
        let value = if title.is_some() {
            open.and_then(|o| config.0.node_contact(&o.node))
                .map_or("Conversation", |p| p.name())
                .to_owned()
        } else if body.is_some() {
            dialogue_node.map_or("", |node| node.text()).to_owned()
        } else if is_status.is_some() {
            status.to_owned()
        } else if prompt.is_some() {
            if state.error.is_some() {
                "Guide: disconnected".to_owned()
            } else if let Some(guide) = guide {
                if state
                    .authority_focus()
                    .is_some_and(|p| theia_network::dialogue::in_range(p, guide.position))
                {
                    format!("{} [F / RMB]", config.0.profile().name())
                } else {
                    format!("{} — move closer to talk", config.0.profile().name())
                }
            } else if state.guide.is_none() {
                "Guide: waiting for server".to_owned()
            } else {
                String::new()
            }
        } else if let Some(choice) = choice {
            dialogue_node
                .and_then(|node| node.choices().get(choice.0))
                .map_or(String::new(), |offered| {
                    format!("[F{}] {}", choice.0 + 1, offered.label())
                })
        } else {
            continue;
        };
        if label.0 != value {
            label.0 = value;
        }
    }
    for (widget, mut button_node, mut color, hovered, pressed) in &mut buttons {
        button_node.display = if matches!(widget,Widget::Choice(index) if dialogue_node.is_none_or(|node|*index>=node.choices().len()))
        {
            Display::None
        } else {
            Display::Flex
        };
        color.0 = if skin.is_some() {
            crate::fantasy_ui::button_color(ui.pending, hovered.is_some_and(|h| h.0), pressed)
        } else if ui.pending {
            Color::srgb(0.07, 0.08, 0.1)
        } else if pressed || hovered.is_some_and(|h| h.0) {
            Color::srgb(0.12, 0.3, 0.38)
        } else {
            Color::srgb(0.08, 0.14, 0.19)
        };
    }
}
fn scroll(
    mut events: MessageReader<MouseWheel>,
    mut ui: ResMut<Ui>,
    mut bodies: Query<(&Hovered, &mut ScrollPosition, &ComputedNode), With<Body>>,
) {
    let delta: f32 = events
        .read()
        .map(|event| {
            ui.scroll_events += 1;
            -event.y
                * if event.unit == MouseScrollUnit::Line {
                    24.
                } else {
                    1.
                }
        })
        .sum();
    if ui.open && delta != 0. {
        ui.scroll_delta = delta;
        for (hovered, mut position, node) in &mut bodies {
            if !hovered.0 {
                continue;
            }
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            ui.scroll_maximum = maximum;
            position.y = (position.y + delta).clamp(0., maximum);
        }
    }
}

/// Read-only bounds and scroll state for diagnosing clipped dialogue choices.
pub(crate) fn qualification_body(world: &mut World) -> Option<serde_json::Value> {
    let ui_scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |scale| scale.0);
    world.query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform, &ScrollPosition), With<Body>>()
        .iter(world).next().map(|(node, transform, scroll)| serde_json::json!({
            "point": (transform.affine().translation * crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
            "size": (node.size() * crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
            "content": (node.content_size() * crate::fantasy_ui::window_factor(node, ui_scale)).to_array(),
            "scroll": [scroll.x, scroll.y]
        }))
}

/// Current painted choice locations for explicit read-only OS qualification.
pub(crate) fn qualification_choices(world: &mut World) -> Vec<serde_json::Value> {
    let ui_scale = world
        .get_resource::<bevy::ui::UiScale>()
        .map_or(1., |scale| scale.0);
    let Some(ui) = world.get_resource::<Ui>() else {
        return Vec::new();
    };
    if !ui.open {
        return Vec::new();
    }
    let choices = ui.choices.clone();
    world.query::<(&Widget,&ComputedNode,&bevy::ui::UiGlobalTransform)>().iter(world)
        .filter_map(|(widget,node,t)| {
            let Widget::Choice(index)=widget else {return None};
            if node.size().min_element()<=0. {return None;}
            let (_,id)=choices.get(*index)?;
            Some(serde_json::json!({"choice":id.as_str(),"point":(t.affine().translation*crate::fantasy_ui::window_factor(node, ui_scale)).to_array()}))
        }).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn escape_closes_newly_received_view_but_pointer_keeps_painted_revision() {
        assert_eq!(close_revision(10, true, false, Some(9)), Some(10));
        assert_eq!(close_revision(10, true, false, None), Some(10));
        assert_eq!(close_revision(10, false, true, None), Some(10));
        assert_eq!(close_revision(10, false, false, Some(9)), Some(9));
        assert_eq!(close_revision(10, false, false, None), None);
    }

    #[test]
    fn exported_dialogue_bounds_use_window_pixels_without_rescaling_scroll_offsets() {
        let mut app = App::new();
        app.insert_resource(bevy::ui::UiScale(1.25));
        app.world_mut().spawn((
            Body,
            ComputedNode {
                size: Vec2::new(400., 200.),
                content_size: Vec2::new(600., 1200.),
                inverse_scale_factor: 0.4,
                ..default()
            },
            bevy::ui::UiGlobalTransform::from_xy(1200., 900.),
            ScrollPosition(Vec2::new(0., 17.)),
        ));
        let value = qualification_body(app.world_mut()).unwrap();
        assert_eq!(value["point"], serde_json::json!([600., 450.]));
        assert_eq!(value["size"], serde_json::json!([200., 100.]));
        assert_eq!(value["content"], serde_json::json!([300., 600.]));
        assert_eq!(value["scroll"], serde_json::json!([0., 17.]));
        assert_eq!(app.world().resource::<bevy::ui::UiScale>().0, 1.25);
    }

    #[test]
    fn header_close_retains_revision_and_pending_guards_of_the_existing_action() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = App::new();
        app.insert_resource(Ui {
            open: true,
            displayed_revision: Some(7),
            ..default()
        })
        .init_resource::<Pending>()
        .add_observer(activate);
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn(Node::default()).with_children(|panel| {
                    crate::fantasy_ui::window_header(
                        panel,
                        (Title, text("Dialogue", 20.)),
                        button(Widget::Close),
                        "Close dialogue",
                    );
                });
            })
            .unwrap();
        let world = app.world_mut();
        let close = world
            .query_filtered::<Entity, With<crate::fantasy_ui::HeaderClose>>()
            .single(world)
            .unwrap();
        world.trigger(Activate { entity: close });
        assert_eq!(world.resource::<Pending>().close, Some(7));
        assert!(world.resource::<Pending>().choice.is_none());
        world.resource_mut::<Pending>().close = None;
        world.resource_mut::<Ui>().pending = true;
        world.trigger(Activate { entity: close });
        assert!(world.resource::<Pending>().close.is_none());
        assert_eq!(world.resource::<Ui>().pointer_actions, 1);
    }

    use super::*;
    #[test]
    fn widget_activation_keeps_the_displayed_branch_identity_and_revision() {
        let old = theia_core::content::ContentId::new("ask-movement").unwrap();
        let new = theia_core::content::ContentId::new("finish-movement").unwrap();
        let mut app = App::new();
        app.insert_resource(Ui {
            open: true,
            displayed_revision: Some(1),
            choices: vec![(1, old.clone())],
            ..default()
        })
        .init_resource::<Pending>()
        .add_observer(activate);
        let choice = app.world_mut().spawn(Widget::Choice(0)).id();
        let close = app.world_mut().spawn(Widget::Close).id();
        app.world_mut().trigger(Activate { entity: choice });
        {
            let mut ui = app.world_mut().resource_mut::<Ui>();
            ui.displayed_revision = Some(2);
            ui.choices = vec![(2, new)];
        }
        assert_eq!(
            app.world().resource::<Pending>().choice,
            Some((1, old)),
            "A newer view must not reinterpret the activated old label"
        );
        app.world_mut().resource_mut::<Pending>().choice = None;
        app.world_mut().resource_mut::<Ui>().pending = true;
        app.world_mut().trigger(Activate { entity: choice });
        assert!(app.world().resource::<Pending>().choice.is_none());
        app.world_mut().resource_mut::<Ui>().pending = false;
        app.world_mut().trigger(Activate { entity: close });
        app.world_mut().resource_mut::<Ui>().displayed_revision = Some(3);
        assert_eq!(app.world().resource::<Pending>().close, Some(2));
        app.world_mut().resource_mut::<Pending>().close = None;
        app.world_mut().resource_mut::<Ui>().open = false;
        app.world_mut().trigger(Activate { entity: close });
        assert!(app.world().resource::<Pending>().close.is_none());
    }
}
