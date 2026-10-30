//! Standard Bevy informational guide/panel. Every displayed node is observed;
//! widget choice and movement never advance dialogue optimistically.
mod qualification;
use crate::{floating_origin::Frame, network::Network, scene::PawnAssets};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::{FontSize, LineBreak},
    ui::Pressed,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use std::sync::Arc;
use theia_network::dialogue::{Action, Profile, Rejection};
#[derive(Resource)]
struct Config(Arc<Profile>);
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
}
#[derive(Resource, Default)]
struct Pending {
    choice: Option<(u64, theia_core::content::ContentId)>,
    close: Option<u64>,
}
#[derive(Component)]
pub(super) struct GuideVisual;
#[derive(Component)]
struct Panel;
#[derive(Component)]
struct Pane;
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
pub(crate) fn install(app: &mut App, profile: Arc<Profile>) {
    app.insert_resource(Config(profile))
        .init_resource::<Ui>()
        .init_resource::<Pending>()
        .add_observer(activate)
        .add_systems(Startup, setup.after(crate::scene::setup))
        .add_systems(
            Update,
            (controls, present, scroll)
                .chain()
                .after(crate::network::present),
        );
    qualification::install(app);
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
        ActivateOnPress,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        GuideVisual,
        Mesh3d(pawns.mesh.clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.65, 0.72))),
        Transform::default(),
        Visibility::Hidden,
    ));
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
    commands
        .spawn((
            Panel,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: percent(20),
                height: percent(50),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Pane,
                Node {
                    width: px(480),
                    max_width: percent(90),
                    height: percent(100),
                    padding: UiRect::all(px(14)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.05, 0.08, 0.98)),
            ))
            .with_children(|panel| {
                panel.spawn((Title, text(config.0.profile().name(), 20.)));
                panel
                    .spawn((
                        Body,
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
fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    network: Res<Network>,
    mut pending: ResMut<Pending>,
    mut ui: ResMut<Ui>,
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
    let escape = keys.just_pressed(KeyCode::Escape);
    if escape && state.dialogue_pending {
        ui.close_when_open = true;
    }
    if state.dialogue_pending {
        return;
    }
    let action = if view.open.is_some() {
        if pointer_close.is_some() || escape || ui.close_when_open {
            let revision = if ui.close_when_open {
                Some(view.revision)
            } else {
                pointer_close.or(ui.displayed_revision)
            };
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
        if keys.just_pressed(KeyCode::KeyE) {
            ui.keyboard_actions += 1;
            state
                .guide
                .as_ref()
                .and_then(|p| p.guide)
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
    network: Res<Network>,
    config: Res<Config>,
    frame: Res<Frame>,
    mut ui: ResMut<Ui>,
    mut visuals: Query<Visual<'_>, VisualFilter>,
    mut labels: Query<Label<'_>>,
    mut buttons: Query<ButtonView<'_>>,
    mut bodies: Query<&mut ScrollPosition, With<Body>>,
) {
    let state = network.state();
    let view = state.dialogue.as_ref().filter(|_| state.error.is_none());
    let open = view.and_then(|view| view.open.as_ref());
    let dialogue_node = open.and_then(|open| config.0.profile().node(&open.node));
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
    for (mut visibility, is_guide, is_panel, transform) in &mut visuals {
        if is_panel.is_some() {
            *visibility = if ui.open {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if is_guide.is_some() {
            if let Some(local) = guide.and_then(|guide| frame.render.to_local(guide.position)) {
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
            config.0.profile().name().to_owned()
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
                    format!("{} [E]", config.0.profile().name())
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
        color.0 = if ui.pending {
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
    ui: Res<Ui>,
    mut bodies: Query<(&mut ScrollPosition, &ComputedNode), With<Body>>,
) {
    let delta: f32 = events
        .read()
        .map(|event| {
            -event.y
                * if event.unit == MouseScrollUnit::Line {
                    24.
                } else {
                    1.
                }
        })
        .sum();
    if ui.open && delta != 0. {
        for (mut position, node) in &mut bodies {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            position.y = (position.y + delta).clamp(0., maximum);
        }
    }
}

#[cfg(test)]
mod tests {
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
