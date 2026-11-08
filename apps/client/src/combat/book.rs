//! Configured ability book and local binding sources. No learning or gameplay authority.
use super::Config;
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui_widgets::{Activate, Button},
};
pub(super) const PAGE_SIZE: usize = 3;
#[derive(Resource, Default)]
pub(crate) struct Ui {
    pub open: bool,
    pub closed_this_frame: bool,
    page: usize,
}
impl Ui {
    pub(super) fn page(&self) -> usize {
        self.page
    }
    pub(super) fn ability_index(&self, row: usize) -> Option<usize> {
        (row < PAGE_SIZE).then_some(self.page * PAGE_SIZE + row)
    }
}
pub(crate) fn owns_escape(ui: Option<&Ui>) -> bool {
    ui.is_some_and(|u| u.open || u.closed_this_frame)
}
#[derive(Component)]
struct Pane;
#[derive(Component)]
struct Entry(usize);
#[derive(Component)]
pub(crate) struct BookEntry(pub(crate) usize);
#[derive(Component)]
struct EntryIcon(usize);
#[derive(Component)]
struct PageLabel;
#[derive(Component)]
struct DragHint;
#[derive(Component, Clone, Copy)]
enum Control {
    Previous,
    Next,
    Close,
}
pub(super) fn pages(count: usize) -> usize {
    count.div_ceil(PAGE_SIZE).max(1)
}
fn label(text: &str, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(0.12, 0.08, 0.03)),
        Pickable::IGNORE,
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
                .after(crate::fantasy_ui::settings::keyboard)
                .before(crate::input)
                .before(crate::camera::controls),
        )
        .add_systems(PostUpdate, present.before(bevy::ui::UiSystems::Layout));
}
fn setup(mut commands: Commands, skin: Option<Res<crate::fantasy_ui::Skin>>) {
    if skin.is_none() {
        return;
    }
    commands
        .spawn((
            Pane,
            GlobalZIndex(70),
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(64),
                width: px(420),
                max_width: percent(90),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgb(0.78, 0.65, 0.43)),
            Outline::new(px(1), px(-1), crate::fantasy_ui::GOLD),
        ))
        .with_children(|pane| {
            pane.spawn(label("Abilities [P]", 22.));
            for slot in 0..PAGE_SIZE {
                pane.spawn((
                    Node {
                        column_gap: px(10),
                        min_height: px(112),
                        ..default()
                    },
                    BookEntry(slot),
                ))
                .with_children(|row| {
                    row.spawn((
                        EntryIcon(slot),
                        ImageNode::default(),
                        Node {
                            width: px(32),
                            height: px(32),
                            flex_shrink: 0.,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    row.spawn((
                        Entry(slot),
                        label("", 14.),
                        Node {
                            flex_grow: 1.,
                            min_width: px(0),
                            ..default()
                        },
                    ));
                });
            }
            pane.spawn((PageLabel, label("", 14.)));
            pane.spawn((
                DragHint,
                label(
                    "Drag an ability to a slot; keep this book open to rearrange your bar.",
                    14.,
                ),
            ));
            pane.spawn((
                Node {
                    column_gap: px(8),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                for (action, title) in [
                    (Control::Previous, "Previous"),
                    (Control::Next, "Next"),
                    (Control::Close, "Close [Esc]"),
                ] {
                    row.spawn((
                        action,
                        Button,
                        Hovered::default(),
                        AccessibleLabel::new(title),
                        Node {
                            padding: UiRect::all(px(8)),
                            ..default()
                        },
                        BackgroundColor(crate::fantasy_ui::PANEL),
                    ))
                    .with_child((
                        Text::new(title),
                        TextFont {
                            font_size: FontSize::Px(16.),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                }
            });
        });
}
pub(crate) fn keyboard(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    windows: Query<&Window>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    mut ui: ResMut<Ui>,
    mut settings: Option<ResMut<crate::fantasy_ui::settings::Ui>>,
) {
    ui.closed_this_frame = false;
    let Some(keys) = keys else {
        return;
    };
    if skin.is_none() || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        ui.open = !ui.open;
        ui.closed_this_frame = !ui.open;
        if let Some(settings) = settings.as_mut() {
            settings.open = false;
        }
    } else if keys.just_pressed(KeyCode::Escape)
        && ui.open
        && !crate::fantasy_ui::settings::owns_escape(settings.as_deref())
    {
        ui.open = false;
        ui.closed_this_frame = true;
    }
}
fn activate(
    event: On<Activate>,
    controls: Query<&Control>,
    windows: Query<&Window>,
    config: Res<Config>,
    mut ui: ResMut<Ui>,
) {
    if !ui.open || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    match control {
        Control::Previous => ui.page = ui.page.saturating_sub(1),
        Control::Next => ui.page = (ui.page + 1).min(pages(config.0.abilities().len()) - 1),
        Control::Close => {
            ui.open = false;
            ui.closed_this_frame = true;
        }
    }
}
type BookPresentation<'w> = (
    Res<'w, Ui>,
    Res<'w, Config>,
    Option<Res<'w, crate::fantasy_ui::Skin>>,
    Option<Res<'w, super::action_bar::Drag>>,
);
type EntryFilter = (Without<PageLabel>, Without<DragHint>);
type HintFilter = (With<DragHint>, Without<Entry>, Without<PageLabel>);
fn present(
    (ui, config, skin, drag): BookPresentation<'_>,
    mut icons: Query<(&EntryIcon, &mut ImageNode, &mut Node), Without<Pane>>,
    mut panes: Query<&mut Node, With<Pane>>,
    mut entries: Query<(&Entry, &mut Text), EntryFilter>,
    mut page_labels: Query<&mut Text, (With<PageLabel>, Without<DragHint>)>,
    mut hints: Query<&mut Text, HintFilter>,
) {
    for mut pane in &mut panes {
        pane.display = if ui.open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !ui.open {
        return;
    }
    for mut hint in &mut hints {
        let value = super::action_bar::drag_hint(drag.as_deref()).unwrap_or_else(|| {
            "Drag an ability to a slot; keep this book open to rearrange your bar.".into()
        });
        if hint.0 != value {
            **hint = value;
        }
    }
    let page = ui.page.min(pages(config.0.abilities().len()) - 1);
    for (icon, mut image, mut node) in &mut icons {
        let ability = config.0.abilities().get(page * PAGE_SIZE + icon.0);
        node.display = if ability.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let (Some(ability), Some(skin)) = (ability, skin.as_ref()) {
            image.image = skin.icon(ability.id().as_str());
        }
    }
    for (entry, mut text) in &mut entries {
        let value = config
            .0
            .abilities()
            .get(page * PAGE_SIZE + entry.0)
            .map(|a| super::ui::tooltip::describe(a, "", "Use your action bar to cast."))
            .unwrap_or_default();
        if text.0 != value {
            **text = value;
        }
    }
    for mut text in &mut page_labels {
        let value = format!("Page {} / {}", page + 1, pages(config.0.abilities().len()));
        if text.0 != value {
            **text = value;
        }
    }
}

/// Actual widget geometry for the existing opt-in native-input qualifier.
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let Some(ui) = world.get_resource::<Ui>() else {
        return serde_json::Value::Null;
    };
    let (open, page) = (ui.open, ui.page);
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let rows: Vec<_> = world.query::<(&Entry, &Text, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
        .iter(world).map(|(entry,text,node,transform)| {
            let factor = crate::fantasy_ui::window_factor(node, scale);
            let centre = transform.affine().translation * factor;
            let size = node.size() * factor;
            serde_json::json!({"row":entry.0,"text":text.0,"bounds":[centre.x-size.x/2.,centre.y-size.y/2.,size.x,size.y]})
        }).collect();
    let controls: Vec<_> = world.query::<(&Control,&ComputedNode,&bevy::ui::UiGlobalTransform)>()
        .iter(world).map(|(action,node,transform)| {
            let action=match action {Control::Previous=>"previous",Control::Next=>"next",Control::Close=>"close"};
            serde_json::json!({"action":action,"point":(transform.affine().translation*crate::fantasy_ui::window_factor(node,scale)).to_array()})
        }).collect();
    serde_json::json!({"open":open,"page":page,"rows":rows,"buttons":controls})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app(count: usize) -> App {
        let mut source: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap();
        let template = source["abilities"][0].clone();
        source["abilities"] = (0..count)
            .map(|n| {
                let mut ability = template.clone();
                ability["id"] = format!("theia.test.ability-{n}").into();
                ability
            })
            .collect::<Vec<_>>()
            .into();
        let catalog =
            theia_core::abilities::Catalog::from_json(&serde_json::to_vec(&source).unwrap())
                .unwrap();
        let mut app = App::new();
        app.insert_resource(Config(std::sync::Arc::new(catalog), None, false))
            .insert_resource(crate::fantasy_ui::Skin::fixture())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<crate::fantasy_ui::settings::Ui>();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        install(&mut app);
        app.update();
        app
    }
    fn control(app: &mut App, wanted: Control) -> Entity {
        app.world_mut()
            .query::<(Entity, &Control)>()
            .iter(app.world())
            .find(|(_, c)| std::mem::discriminant(*c) == std::mem::discriminant(&wanted))
            .unwrap()
            .0
    }
    fn key(app: &mut App, key: KeyCode) {
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(key);
        }
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
    }
    #[test]
    fn complete_catalogue_is_paged_without_learning_or_casting_and_bounds_clamp() {
        let mut app = app(theia_core::abilities::MAX_ABILITIES);
        assert!(!app.world().resource::<Ui>().open);
        key(&mut app, KeyCode::KeyP);
        let next = control(&mut app, Control::Next);
        let previous = control(&mut app, Control::Previous);
        let mut seen = std::collections::BTreeSet::new();
        for page in 0..pages(theia_core::abilities::MAX_ABILITIES) {
            assert_eq!(app.world().resource::<Ui>().page, page);
            for (_, text) in app.world_mut().query::<(&Entry, &Text)>().iter(app.world()) {
                if !text.0.is_empty() {
                    seen.insert(text.0.clone());
                    assert!(text.0.contains("30 m range"));
                }
            }
            app.world_mut().trigger(Activate { entity: next });
            app.update();
        }
        assert_eq!(seen.len(), theia_core::abilities::MAX_ABILITIES);
        assert_eq!(app.world().resource::<Ui>().page, pages(seen.len()) - 1);
        for _ in 0..30 {
            app.world_mut().trigger(Activate { entity: previous });
        }
        assert_eq!(app.world().resource::<Ui>().page, 0);
        assert!(!app.world().contains_resource::<crate::network::Network>());
        assert!(!app.world().contains_resource::<super::super::ui::Input>());
    }
    #[test]
    fn unfocused_or_closed_book_refuses_fresh_keyboard_and_widget_changes() {
        let mut app = app(4);
        let next = control(&mut app, Control::Next);
        app.world_mut().trigger(Activate { entity: next });
        assert_eq!(app.world().resource::<Ui>().page, 0);
        app.world_mut()
            .query::<&mut Window>()
            .single_mut(app.world_mut())
            .unwrap()
            .focused = false;
        key(&mut app, KeyCode::KeyP);
        assert!(!app.world().resource::<Ui>().open);
        app.world_mut()
            .query::<&mut Window>()
            .single_mut(app.world_mut())
            .unwrap()
            .focused = true;
        key(&mut app, KeyCode::KeyP);
        assert!(app.world().resource::<Ui>().open);
        app.world_mut()
            .query::<&mut Window>()
            .single_mut(app.world_mut())
            .unwrap()
            .focused = false;
        app.world_mut().trigger(Activate { entity: next });
        assert_eq!(app.world().resource::<Ui>().page, 0);
        app.world_mut()
            .query::<&mut Window>()
            .single_mut(app.world_mut())
            .unwrap()
            .focused = true;
        key(&mut app, KeyCode::Escape);
        assert!(owns_escape(Some(app.world().resource::<Ui>())));
        assert!(!app.world().resource::<Ui>().open);
    }
}
