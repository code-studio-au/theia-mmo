//! Local interface preferences. Never sends gameplay commands.
use super::{PreferredScale, button_color};
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, Button},
};
#[derive(Resource, Default)]
pub(crate) struct Ui {
    pub open: bool,
    pub closed_this_frame: bool,
}
pub(crate) fn owns_escape(ui: Option<&Ui>) -> bool {
    ui.is_some_and(|u| u.open || u.closed_this_frame)
}
#[derive(Component)]
pub(crate) struct Pane;
#[derive(Component)]
struct Summary;
#[derive(Component)]
struct PersistenceHint;
#[derive(Component, Clone, Copy)]
enum Control {
    Toggle,
    Close,
    Scale(u8),
}
fn label(value: &str) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(16.),
            ..default()
        },
        TextColor(Color::WHITE),
        Pickable::IGNORE,
    )
}
fn button(action: Control, title: &str) -> impl Bundle {
    (
        action,
        Button,
        Hovered::default(),
        AccessibleLabel::new(title),
        Node {
            padding: UiRect::all(px(8)),
            min_height: px(34),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(button_color(false, false, false)),
    )
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<Ui>()
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            keyboard
                .after(bevy::input::InputSystems)
                .before(crate::camera::controls)
                .before(crate::input),
        )
        .add_systems(PostUpdate, present.before(bevy::ui::UiSystems::Layout));
}
pub(super) fn install_preview(app: &mut App, preferred: f32) {
    app.insert_resource(PreferredScale(preferred))
        .insert_resource(Ui {
            open: true,
            closed_this_frame: false,
        })
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, present.before(bevy::ui::UiSystems::Layout));
}
/// Read-only native pointer telemetry; it supplies no actions or preference edits.
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let Some(ui) = world.get_resource::<Ui>() else {
        return serde_json::Value::Null;
    };
    let open = ui.open;
    let preferred = world.resource::<PreferredScale>().0;
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let buttons:Vec<_>=world.query::<(&Control,&ComputedNode,&bevy::ui::UiGlobalTransform)>().iter(world)
        .filter(|(action,_,_)|open || matches!(action,Control::Toggle))
        .map(|(action,node,transform)|{
            let name=match action {Control::Toggle=>"toggle".to_owned(),Control::Close=>"close".to_owned(),Control::Scale(value)=>format!("scale-{value}")};
            serde_json::json!({"action":name,"point":(transform.affine().translation*super::window_factor(node,scale)).to_array()})
        }).collect();
    serde_json::json!({"open":open,"preferred":preferred,"displayed":scale,"buttons":buttons})
}
fn setup(mut commands: Commands) {
    commands
        .spawn(button(Control::Toggle, "Interface settings [F10]"))
        .insert((
            GlobalZIndex(30),
            Node {
                position_type: PositionType::Absolute,
                right: px(160),
                bottom: px(12),
                width: px(140),
                min_height: px(30),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_child(label("Interface [F10]"));
    commands.spawn((Pane,GlobalZIndex(90),Node{position_type:PositionType::Absolute,left:percent(50),top:px(80),margin:UiRect::left(px(-230)),width:px(460),max_width:percent(90),padding:UiRect::all(px(14)),flex_direction:FlexDirection::Column,row_gap:px(12),display:Display::None,..default()},BackgroundColor(super::PANEL),Outline::new(px(1),px(-1),super::GOLD)))
        .with_children(|pane| {
            pane.spawn((Text::new("Interface"),TextFont{font_size:FontSize::Px(22.),..default()},TextColor(super::GOLD),Pickable::IGNORE));
            pane.spawn((Summary,label("")));
            pane.spawn((Node{flex_wrap:FlexWrap::Wrap,column_gap:px(8),row_gap:px(8),..default()},Pickable::IGNORE)).with_children(|row| {
                for value in [75,100,125,150] {row.spawn(button(Control::Scale(value),&format!("UI size {value}%"))).with_child(label(&format!("{value}%")));}
            });
            pane.spawn(label("The interface fits smaller windows and restores your preferred size when space returns."));
            pane.spawn((PersistenceHint, label("Applies to this session. The world continues running.")));
            pane.spawn(button(Control::Close,"Close interface settings")).with_child(label("Close [Esc]"));
        });
}
pub(crate) fn keyboard(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    windows: Query<&Window>,
    mut ui: ResMut<Ui>,
) {
    ui.closed_this_frame = false;
    let Some(keys) = keys else {
        return;
    };
    if !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && ui.open {
        ui.open = false;
        ui.closed_this_frame = true;
    } else if keys.just_pressed(KeyCode::F10) {
        ui.open = !ui.open;
    }
}
fn activate(
    event: On<Activate>,
    controls: Query<&Control>,
    mut ui: ResMut<Ui>,
    mut preferred: ResMut<PreferredScale>,
) {
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    match *control {
        Control::Toggle => ui.open = !ui.open,
        Control::Close if ui.open => {
            ui.open = false;
        }
        Control::Scale(value) if ui.open && matches!(value, 75 | 100 | 125 | 150) => {
            preferred.0 = f32::from(value) / 100.
        }
        _ => {}
    }
}
#[allow(clippy::too_many_arguments)]
fn present(
    ui: Res<Ui>,
    preferred: Res<PreferredScale>,
    scale: Res<bevy::ui::UiScale>,
    mut panes: Query<&mut Node, With<Pane>>,
    mut summaries: Query<&mut Text, With<Summary>>,
    store: Option<Res<super::preferences::Store>>,
    mut hints: Query<&mut Text, (With<PersistenceHint>, Without<Summary>)>,
    mut buttons: Query<(&Control, &Hovered, Has<Pressed>, &mut BackgroundColor)>,
) {
    for mut node in &mut panes {
        let display = if ui.open {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let text = format!(
        "UI size: {:.0}% preferred · {:.0}% displayed",
        preferred.0 * 100.,
        scale.0 * 100.
    );
    for mut label in &mut summaries {
        if label.0 != text {
            label.0 = text.clone();
        }
    }
    let hint = store.as_ref().map_or(
        "Applies to this session. The world continues running.",
        |store| store.hint(),
    );
    for mut text in &mut hints {
        if text.0 != hint {
            text.0 = hint.to_owned();
        }
    }
    for (control, hovered, pressed, mut color) in &mut buttons {
        let value = if matches!(control,Control::Scale(v) if (f32::from(*v)/100.-preferred.0).abs()<0.001)
        {
            Color::srgb(0.38, 0.25, 0.08)
        } else {
            button_color(false, hovered.0, pressed)
        };
        if color.0 != value {
            color.0 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_presets_require_an_open_panel_and_leave_preferences_in_bounds() {
        let mut app = App::new();
        app.init_resource::<Ui>()
            .insert_resource(PreferredScale(1.));
        app.add_observer(activate);
        let toggle = app.world_mut().spawn(Control::Toggle).id();
        let preset = app.world_mut().spawn(Control::Scale(125)).id();
        let invalid = app.world_mut().spawn(Control::Scale(255)).id();
        app.world_mut().trigger(Activate { entity: preset });
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.);
        app.world_mut().trigger(Activate { entity: toggle });
        app.world_mut().trigger(Activate { entity: preset });
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.25);
        app.world_mut().trigger(Activate { entity: invalid });
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.25);
        // A small viewport fits down without losing the chosen preference.
        assert_eq!(
            super::super::fit_scale(
                app.world().resource::<PreferredScale>().0,
                Vec2::new(1280., 720.)
            ),
            1.
        );
        assert_eq!(
            super::super::fit_scale(
                app.world().resource::<PreferredScale>().0,
                Vec2::new(1920., 1200.)
            ),
            1.25
        );
    }
    #[test]
    fn keyboard_does_not_open_settings_when_the_window_lacks_focus() {
        let mut app = App::new();
        app.init_resource::<Ui>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, keyboard);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: false,
                ..default()
            })
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F10);
        app.update();
        assert!(!app.world().resource::<Ui>().open);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.update();
        assert!(app.world().resource::<Ui>().open);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::Escape);
        }
        app.update();
        assert!(
            !app.world().resource::<Ui>().open && app.world().resource::<Ui>().closed_this_frame
        );
    }
}
