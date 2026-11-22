//! Interface preferences and opt-in local developer controls.
use super::{PreferredScale, button_color, graphics::ViewDistance};
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
    pub graphics_open: bool,
    pub developer_open: bool,
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
#[derive(Component)]
struct GraphicsSummary;
#[derive(Component)]
enum Section {
    Interface,
    Graphics,
    Developer,
}
#[derive(Component, Clone, Copy)]
enum Control {
    Toggle,
    Close,
    Scale(u8),
    Exit,
    Abilities,
    Debug,
    Graphics,
    Back,
    FrameCap(u16),
    Vsync(bool),
    ViewDistance(ViewDistance),
    Developer,
    DeveloperSpeed(u8),
    DeveloperReset,
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
    super::developer::install(app);
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
            graphics_open: false,
            developer_open: false,
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
    let persistence = world
        .get_resource::<super::preferences::Store>()
        .map(|store| store.hint());
    let present_mode = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map(|window| format!("{:?}", window.present_mode));
    let buttons:Vec<_>=world.query::<(&Control,&ComputedNode,&bevy::ui::UiGlobalTransform)>().iter(world)
        .filter(|(action,node,_)|node.size().min_element()>0. && (open || matches!(action,Control::Toggle)))
        .map(|(action,node,transform)|{
            let name=match action {Control::Toggle=>"toggle".to_owned(),Control::Close=>"close".to_owned(),Control::Scale(value)=>format!("scale-{value}"),Control::Exit=>"exit".to_owned(),Control::Abilities=>"abilities".to_owned(),Control::Debug=>"debug".to_owned(),Control::Graphics=>"graphics".to_owned(),Control::Developer=>"developer".to_owned(),Control::DeveloperReset=>"reset-position".to_owned(),Control::DeveloperSpeed(value)=>format!("speed-{value}"),Control::Back=>"back".to_owned(),Control::FrameCap(value)=>format!("fps-{value}"),Control::ViewDistance(value)=>format!("view-{}",value.label().to_lowercase()),Control::Vsync(value)=>format!("vsync-{}",if *value {"on"} else {"off"})};
            serde_json::json!({"action":name,"point":(transform.affine().translation*super::window_factor(node,scale)).to_array()})
        }).collect();
    serde_json::json!({"open":open,"graphics_open":world.resource::<Ui>().graphics_open,"developer_open":world.resource::<Ui>().developer_open,"developer_speed":world.get_resource::<super::developer::Speed>().map(|s|serde_json::json!({"requested":s.requested,"applied":s.applied,"reset_nonce":s.reset_nonce,"reset_status":s.reset_status,"stream":s.stream})),"view_distance":world.get_resource::<ViewDistance>(),"fps_cap":world.get_resource::<super::graphics::FrameCap>().map(|cap|cap.0),"vsync":world.get_resource::<super::graphics::Vsync>().is_none_or(|choice|choice.0),"present_mode":present_mode,"preferred":preferred,"displayed":scale,"buttons":buttons,"persistence":persistence,"debug_stats":world.get_resource::<super::debug::Enabled>().is_some_and(|v|v.0)})
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
    commands.spawn((Pane,GlobalZIndex(90),Node{position_type:PositionType::Absolute,left:percent(50),top:px(80),margin:UiRect::left(px(-230)),width:px(460),max_width:percent(90),padding:UiRect::all(px(14)),flex_direction:FlexDirection::Column,row_gap:px(6),display:Display::None,..default()},BackgroundColor(super::PANEL),Outline::new(px(1),px(-1),super::GOLD)))
        .with_children(|pane| {
            pane.spawn((Section::Interface, Node { flex_direction: FlexDirection::Column, row_gap: px(6), ..default() }, Pickable::IGNORE)).with_children(|section| {
            section.spawn((Text::new("Interface"),TextFont{font_size:FontSize::Px(22.),..default()},TextColor(super::GOLD),Pickable::IGNORE));
            section.spawn((Summary,label("")));
            section.spawn((Node{flex_wrap:FlexWrap::Wrap,column_gap:px(8),row_gap:px(8),..default()},Pickable::IGNORE)).with_children(|row| {
                for value in [75,100,125,150] {row.spawn(button(Control::Scale(value),&format!("UI size {value}%"))).with_child(label(&format!("{value}%")));}
            });
            section.spawn(label("The interface fits smaller windows and restores your preferred size when space returns."));
            section.spawn(button(Control::Abilities,"Open ability book [P]")).with_child(label("Abilities [P]"));
            section.spawn(button(Control::Debug,"Toggle debug statistics")).with_child(label("Show / hide debug statistics"));
            section.spawn(button(Control::Graphics,"Open graphics settings")).with_child(label("Graphics"));
            section.spawn(button(Control::Developer,"Open developer settings")).with_child(label("Developer"));
            });
            pane.spawn((Section::Graphics, Node { display: Display::None, flex_direction: FlexDirection::Column, row_gap: px(6), ..default() }, Pickable::IGNORE)).with_children(|section| {
                section.spawn((Text::new("Graphics"), TextFont { font_size: FontSize::Px(22.), ..default() }, TextColor(super::GOLD), Pickable::IGNORE));
                section.spawn((GraphicsSummary, label("FPS cap: 60 FPS")));
                section.spawn((Node { flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(8), ..default() }, Pickable::IGNORE)).with_children(|row| {
                    for value in super::graphics::CAPS {
                        let name = super::graphics::FrameCap(value).label();
                        row.spawn(button(Control::FrameCap(value), &format!("FPS cap: {name}"))).with_child(label(&name));
                    }
                });
                section.spawn(label("Limits rendering to reduce GPU use."));
                section.spawn((Node { column_gap: px(8), ..default() }, Pickable::IGNORE)).with_children(|row| {
                    for (value, name) in [(true, "VSync On"), (false, "VSync Off")] {
                        row.spawn(button(Control::Vsync(value), name)).with_child(label(name));
                    }
                });
                section.spawn(label("VSync prevents tearing; FPS may follow display refresh."));
                section.spawn(label("Terrain view distance"));
                section.spawn((Node { flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(8), ..default() }, Pickable::IGNORE)).with_children(|row| {
                    for value in ViewDistance::ALL {
                        row.spawn(button(Control::ViewDistance(value), &format!("View distance: {}", value.label()))).with_child(label(value.label()));
                    }
                });
                section.spawn(label("Loads distant terrain progressively. Maximum shows the whole zone where supported and uses more memory."));
                section.spawn(button(Control::Back, "Back to interface settings")).with_child(label("Back [Esc]"));
            });
            pane.spawn((Section::Developer, Node {display:Display::None,flex_direction:FlexDirection::Column,row_gap:px(8),..default()},Pickable::IGNORE)).with_children(|section| {
                section.spawn((Text::new("Developer"),TextFont{font_size:FontSize::Px(22.),..default()},TextColor(super::GOLD),Pickable::IGNORE));
                section.spawn((super::developer::Summary,label("Running speed")));
                section.spawn((Node{flex_wrap:FlexWrap::Wrap,column_gap:px(8),row_gap:px(8),..default()},Pickable::IGNORE)).with_children(|row| {
                    for value in theia_network::development::SPEEDS {
                        row.spawn(button(Control::DeveloperSpeed(value),&format!("Running speed: {value}x"))).with_child(label(&format!("{value}×")));
                    }
                });
                section.spawn(label("Changes running speed for this local walkthrough. Walking and jumping stay unchanged. Resets to 1× for a new session."));
                section.spawn(button(Control::DeveloperReset,"Reset position to starting point")).with_child(label("Reset position"));
                section.spawn((super::developer::ResetSummary,label("Returns to your starting point and clears falling movement.")));
                section.spawn(button(Control::Back,"Back from developer settings")).with_child(label("Back [Esc]"));
            });
            pane.spawn((PersistenceHint, label("Applies to this session. The world continues running.")));
            pane.spawn(button(Control::Close,"Close interface settings")).with_child(label("Close"));
            pane.spawn(button(Control::Exit,"Exit game")).with_child(label("Exit game"));
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
        if ui.graphics_open || ui.developer_open {
            ui.graphics_open = false;
            ui.developer_open = false;
        } else {
            ui.open = false;
        }
        ui.closed_this_frame = true;
    } else if keys.just_pressed(KeyCode::F10) {
        ui.open = !ui.open;
        ui.graphics_open = false;
        ui.developer_open = false;
    }
}
#[allow(clippy::too_many_arguments)]
fn activate(
    event: On<Activate>,
    controls: Query<&Control>,
    mut ui: ResMut<Ui>,
    mut preferred: ResMut<PreferredScale>,
    debug: Option<ResMut<super::debug::Enabled>>,
    cap: Option<ResMut<super::graphics::FrameCap>>,
    vsync: Option<ResMut<super::graphics::Vsync>>,
    view: Option<ResMut<ViewDistance>>,
    speed: Option<ResMut<super::developer::Speed>>,
    windows: Query<&Window>,
    (store, book): (
        Option<ResMut<super::preferences::Store>>,
        Option<ResMut<crate::combat::book::Ui>>,
    ),
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    if !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    match *control {
        Control::Developer if ui.open => {
            ui.developer_open = true;
            ui.graphics_open = false;
        }
        Control::DeveloperReset if ui.open && ui.developer_open => {
            if let Some(mut speed) = speed {
                speed.reset_position();
            }
        }
        Control::DeveloperSpeed(value) if ui.open && ui.developer_open => {
            if let Some(mut speed) = speed {
                speed.request(value);
            }
        }

        Control::ViewDistance(value) if ui.open && ui.graphics_open => {
            if let Some(mut choice) = view {
                *choice = value;
            }
            if let Some(mut store) = store {
                store.retry_failed();
            }
        }

        Control::Vsync(value) if ui.open && ui.graphics_open => {
            if let Some(mut choice) = vsync {
                choice.0 = value;
            }
            if let Some(mut store) = store {
                store.retry_failed();
            }
        }
        Control::Graphics if ui.open => {
            ui.graphics_open = true;
            ui.developer_open = false;
        }
        Control::Back if ui.open => {
            ui.graphics_open = false;
            ui.developer_open = false;
        }
        Control::FrameCap(value)
            if ui.open && ui.graphics_open && super::graphics::FrameCap(value).valid() =>
        {
            if let Some(mut cap) = cap {
                cap.0 = value;
            }
            if let Some(mut store) = store {
                store.retry_failed();
            }
        }

        Control::Debug if ui.open => {
            if let Some(mut debug) = debug {
                debug.0 = !debug.0;
            }
        }
        Control::Toggle => {
            ui.open = !ui.open;
            ui.graphics_open = false;
            ui.developer_open = false;
        }
        Control::Close if ui.open => {
            ui.open = false;
        }
        Control::Scale(value) if ui.open && matches!(value, 75 | 100 | 125 | 150) => {
            if let Some(mut store) = store {
                store.retry_failed();
            }
            preferred.0 = f32::from(value) / 100.
        }
        Control::Abilities if ui.open => {
            if let Some(mut book) = book {
                book.open = true;
                ui.open = false;
                ui.closed_this_frame = true;
            }
        }
        Control::Exit if ui.open => {
            ui.open = false;
            ui.closed_this_frame = true;
            exit.write(bevy::app::AppExit::Success);
        }
        _ => {}
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn present(
    ui: Res<Ui>,
    preferred: Res<PreferredScale>,
    scale: Res<bevy::ui::UiScale>,
    mut panes: Query<&mut Node, (With<Pane>, Without<Section>)>,
    mut sections: Query<(&Section, &mut Node), Without<Pane>>,
    cap: Option<Res<super::graphics::FrameCap>>,
    vsync: Option<Res<super::graphics::Vsync>>,
    view: Option<Res<ViewDistance>>,
    speed: Option<Res<super::developer::Speed>>,
    mut graphics_summary: Query<
        &mut Text,
        (
            With<GraphicsSummary>,
            Without<Summary>,
            Without<PersistenceHint>,
        ),
    >,
    mut summaries: Query<&mut Text, (With<Summary>, Without<GraphicsSummary>)>,
    store: Option<Res<super::preferences::Store>>,
    mut hints: Query<
        &mut Text,
        (
            With<PersistenceHint>,
            Without<Summary>,
            Without<GraphicsSummary>,
        ),
    >,
    mut buttons: Query<(&Control, &Hovered, Has<Pressed>, &mut BackgroundColor)>,
) {
    for (section, mut node) in &mut sections {
        let visible = match section {
            Section::Interface => !ui.graphics_open && !ui.developer_open,
            Section::Graphics => ui.graphics_open,
            Section::Developer => ui.developer_open,
        };
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let cap = cap.as_deref().copied().unwrap_or_default();
    let vsync = vsync.as_ref().is_none_or(|choice| choice.0);
    let view = view.as_deref().copied().unwrap_or_default();
    let graphics_text = format!(
        "FPS cap: {} · VSync: {}\nView distance: {}",
        cap.label(),
        if vsync { "On" } else { "Off" },
        view.label()
    );
    for mut text in &mut graphics_summary {
        if text.0 != graphics_text {
            text.0 = graphics_text.clone();
        }
    }
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
            || matches!(control, Control::FrameCap(v) if *v == cap.0)
            || matches!(control, Control::Vsync(v) if *v == vsync)
            || matches!(control, Control::ViewDistance(v) if *v == view)
            || matches!(control, Control::DeveloperSpeed(v) if speed.as_ref().is_some_and(|s|s.applied==Some(*v)))
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
    fn developer_speed_requires_focused_open_page_and_keeps_applied_authoritative() {
        let mut app = App::new();
        app.init_resource::<Ui>()
            .insert_resource(PreferredScale(1.))
            .insert_resource(super::super::developer::Speed::new(
                "/tmp/unused-developer-test".into(),
            ))
            .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let speed = app.world_mut().spawn(Control::DeveloperSpeed(8)).id();
        for (open, developer, focused) in [
            (false, false, true),
            (true, false, true),
            (true, true, false),
        ] {
            let mut ui = app.world_mut().resource_mut::<Ui>();
            ui.open = open;
            ui.developer_open = developer;
            app.world_mut().get_mut::<Window>(window).unwrap().focused = focused;
            app.world_mut().trigger(Activate { entity: speed });
            assert_eq!(
                app.world()
                    .resource::<super::super::developer::Speed>()
                    .requested,
                1
            );
        }
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().trigger(Activate { entity: speed });
        let choice = app.world().resource::<super::super::developer::Speed>();
        assert_eq!(choice.requested, 8);
        assert_eq!(choice.applied, None);
        let invalid = app.world_mut().spawn(Control::DeveloperSpeed(255)).id();
        app.world_mut().trigger(Activate { entity: invalid });
        assert_eq!(
            app.world()
                .resource::<super::super::developer::Speed>()
                .requested,
            8
        );
        let back = app.world_mut().spawn(Control::Back).id();
        app.world_mut().trigger(Activate { entity: back });
        assert!(!app.world().resource::<Ui>().developer_open);
        assert!(app.world().resource::<Ui>().open);
    }
    #[test]
    fn reset_button_requires_focus_open_developer_page_and_connected_stream() {
        let mut app = App::new();
        let mut speed = super::super::developer::Speed::new("/tmp/unused-reset-test".into());
        speed.stream = Some(1);
        app.init_resource::<Ui>()
            .insert_resource(PreferredScale(1.))
            .insert_resource(speed)
            .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let reset = app.world_mut().spawn(Control::DeveloperReset).id();
        for (open, developer, focused) in [
            (false, false, true),
            (true, false, true),
            (true, true, false),
        ] {
            let mut ui = app.world_mut().resource_mut::<Ui>();
            ui.open = open;
            ui.developer_open = developer;
            app.world_mut().get_mut::<Window>(window).unwrap().focused = focused;
            app.world_mut().trigger(Activate { entity: reset });
            assert_eq!(
                app.world()
                    .resource::<super::super::developer::Speed>()
                    .reset_nonce,
                0
            );
        }
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().trigger(Activate { entity: reset });
        app.world_mut().trigger(Activate { entity: reset });
        assert_eq!(
            app.world()
                .resource::<super::super::developer::Speed>()
                .reset_nonce,
            1
        );
        assert_eq!(
            app.world()
                .resource::<super::super::developer::Speed>()
                .stream,
            Some(1)
        );
    }
    #[test]
    fn vsync_buttons_require_the_open_focused_graphics_page_and_preserve_fps_cap() {
        use super::super::graphics::{FrameCap, Vsync};
        let mut app = App::new();
        app.init_resource::<Ui>()
            .init_resource::<Vsync>()
            .insert_resource(FrameCap(144))
            .insert_resource(PreferredScale(1.))
            .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let off = app.world_mut().spawn(Control::Vsync(false)).id();
        let on = app.world_mut().spawn(Control::Vsync(true)).id();
        for (open, graphics, focused) in [
            (false, false, true),
            (true, false, true),
            (true, true, false),
        ] {
            app.world_mut().resource_mut::<Ui>().open = open;
            app.world_mut().resource_mut::<Ui>().graphics_open = graphics;
            app.world_mut().get_mut::<Window>(window).unwrap().focused = focused;
            app.world_mut().trigger(Activate { entity: off });
            assert!(app.world().resource::<Vsync>().0);
        }
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().trigger(Activate { entity: off });
        assert!(!app.world().resource::<Vsync>().0);
        app.world_mut().trigger(Activate { entity: on });
        assert!(app.world().resource::<Vsync>().0);
        assert_eq!(app.world().resource::<FrameCap>().0, 144);
    }
    #[test]
    fn graphics_cap_requires_visible_focused_page_and_escape_returns_to_interface() {
        use super::super::graphics::FrameCap;
        let mut app = App::new();
        app.init_resource::<Ui>()
            .init_resource::<FrameCap>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(PreferredScale(1.))
            .add_observer(activate)
            .add_systems(Update, keyboard);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let graphics = app.world_mut().spawn(Control::Graphics).id();
        let cap = app.world_mut().spawn(Control::FrameCap(120)).id();
        let invalid = app.world_mut().spawn(Control::FrameCap(1)).id();
        app.world_mut().trigger(Activate { entity: cap });
        assert_eq!(app.world().resource::<FrameCap>().0, 60);
        app.world_mut().resource_mut::<Ui>().open = true;
        app.world_mut().trigger(Activate { entity: cap });
        assert_eq!(app.world().resource::<FrameCap>().0, 60);
        app.world_mut().trigger(Activate { entity: graphics });
        app.world_mut().trigger(Activate { entity: cap });
        assert_eq!(app.world().resource::<FrameCap>().0, 120);
        app.world_mut().trigger(Activate { entity: invalid });
        assert_eq!(app.world().resource::<FrameCap>().0, 120);
        let unlimited = app.world_mut().spawn(Control::FrameCap(0)).id();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut().trigger(Activate { entity: unlimited });
        assert_eq!(app.world().resource::<FrameCap>().0, 120);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().trigger(Activate { entity: unlimited });
        assert_eq!(app.world().resource::<FrameCap>().0, 0);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        let ui = app.world().resource::<Ui>();
        assert!(ui.open && !ui.graphics_open && ui.closed_this_frame);
    }
    #[test]
    fn debug_stats_toggle_requires_the_open_focused_interface() {
        let mut app = App::new();
        app.init_resource::<Ui>()
            .init_resource::<super::super::debug::Enabled>()
            .insert_resource(PreferredScale(1.))
            .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let button = app.world_mut().spawn(Control::Debug).id();
        app.world_mut().trigger(Activate { entity: button });
        assert!(!app.world().resource::<super::super::debug::Enabled>().0);
        app.world_mut().resource_mut::<Ui>().open = true;
        app.world_mut().trigger(Activate { entity: button });
        assert!(app.world().resource::<super::super::debug::Enabled>().0);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut().trigger(Activate { entity: button });
        assert!(app.world().resource::<super::super::debug::Enabled>().0);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().trigger(Activate { entity: button });
        assert!(!app.world().resource::<super::super::debug::Enabled>().0);
    }
    #[test]
    fn local_presets_require_an_open_panel_and_leave_preferences_in_bounds() {
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
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
    fn interface_widgets_require_one_focused_window_before_editing_preferences_or_panels() {
        for count in [0, 1, 2] {
            let mut app = App::new();
            app.insert_resource(Ui {
                open: true,
                closed_this_frame: false,
                graphics_open: false,
                developer_open: false,
            })
            .insert_resource(PreferredScale(1.))
            .add_observer(activate);
            for _ in 0..count {
                app.world_mut().spawn(Window {
                    focused: true,
                    ..default()
                });
            }
            let preset = app.world_mut().spawn(Control::Scale(125)).id();
            let close = app.world_mut().spawn(Control::Close).id();
            app.world_mut().trigger(Activate { entity: preset });
            app.world_mut().trigger(Activate { entity: close });
            assert_eq!(
                app.world().resource::<PreferredScale>().0,
                if count == 1 { 1.25 } else { 1. }
            );
            assert_eq!(app.world().resource::<Ui>().open, count != 1);
        }
        let mut app = App::new();
        app.insert_resource(Ui {
            open: true,
            closed_this_frame: false,
            graphics_open: false,
            developer_open: false,
        })
        .insert_resource(PreferredScale(1.))
        .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: false,
                ..default()
            })
            .id();
        let preset = app.world_mut().spawn(Control::Scale(150)).id();
        app.world_mut().trigger(Activate { entity: preset });
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().trigger(Activate { entity: preset });
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.5);
    }
    #[test]
    fn exit_requires_an_open_pane_and_one_focused_window_and_is_not_repeated() {
        for (count, focused, open) in [
            (0, true, true),
            (2, true, true),
            (1, false, true),
            (1, true, false),
            (1, true, true),
        ] {
            let mut app = App::new();
            app.insert_resource(Ui {
                open,
                closed_this_frame: false,
                graphics_open: false,
                developer_open: false,
            })
            .insert_resource(PreferredScale(1.))
            .add_observer(activate);
            for _ in 0..count {
                app.world_mut().spawn(Window {
                    focused,
                    ..default()
                });
            }
            let button = app.world_mut().spawn(Control::Exit).id();
            for _ in 0..2 {
                app.world_mut().trigger(Activate { entity: button });
            }
            let accepted = count == 1 && focused && open;
            assert_eq!(
                app.world().resource::<Messages<bevy::app::AppExit>>().len(),
                usize::from(accepted)
            );
            assert_eq!(app.world().resource::<Ui>().open, open && !accepted);
            assert_eq!(app.world().resource::<Ui>().closed_this_frame, accepted);
            assert_eq!(app.world().resource::<PreferredScale>().0, 1.);
        }
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
