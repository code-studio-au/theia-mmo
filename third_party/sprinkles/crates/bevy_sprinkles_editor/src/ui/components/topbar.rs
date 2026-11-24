use crate::hosting::AuthoringAppExt;
use bevy::prelude::*;

use crate::project::SaveProjectEvent;
use crate::ui::components::playback_controls::playback_controls;
use crate::ui::components::project_selector::project_selector;
use crate::ui::components::seekbar::seekbar;
use crate::ui::tokens::{BACKGROUND_COLOR, BORDER_COLOR};
use crate::ui::widgets::button::{ButtonClickEvent, ButtonProps, ButtonVariant, button};
use crate::ui::widgets::separator::EditorSeparator;

pub fn plugin(app: &mut App) {
    app.add_authoring_systems(setup_save_button_observer);
}

#[derive(Component)]
pub struct SaveButton;

fn setup_save_button_observer(buttons: Query<Entity, Added<SaveButton>>, mut commands: Commands) {
    for entity in &buttons {
        commands.entity(entity).observe(on_save_button_click);
    }
}

fn on_save_button_click(_event: On<ButtonClickEvent>, mut commands: Commands) {
    commands.trigger(SaveProjectEvent);
}

#[derive(Component, Default, Clone)]
pub struct EditorTopbar;

pub fn topbar() -> impl Scene {
    bsn! {
        EditorTopbar
        Node {
            width: percent(100),
            height: px(52),
            padding: { UiRect::all(px(12)) },
            border: { UiRect::bottom(px(1)) },
            justify_content: { JustifyContent::SpaceBetween },
            align_items: { AlignItems::Center },
        }
        BackgroundColor(BACKGROUND_COLOR)
        @template_value(BorderColor::all(BORDER_COLOR))
    }
}

fn topbar_right() -> impl Scene {
    bsn! {
        Node {
            column_gap: px(12),
            align_items: { AlignItems::Center },
        }
    }
}

pub fn spawn_topbar(commands: &mut Commands, parent: Entity, embedded: bool) {
    let bar = commands.spawn_scene(topbar()).insert(ChildOf(parent)).id();

    let selector = commands.spawn_scene(project_selector()).id();
    if embedded {
        let left = commands
            .spawn((
                Node {
                    column_gap: px(12),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ChildOf(bar),
            ))
            .id();
        commands
            .spawn_scene(button(
                ButtonProps::new("Zone").with_variant(ButtonVariant::Ghost),
            ))
            .insert((crate::hosting::ReturnToHostButton, ChildOf(left)));
        commands.entity(left).add_child(selector);
    } else {
        commands.entity(bar).add_child(selector);
    }

    let right = commands
        .spawn_scene(topbar_right())
        .insert(ChildOf(bar))
        .id();
    commands.spawn_scene(seekbar()).insert(ChildOf(right));
    commands
        .spawn_scene(playback_controls())
        .insert(ChildOf(right));
    commands
        .spawn_scene(EditorSeparator::vertical())
        .insert(ChildOf(right));
    commands
        .spawn_scene(button(
            ButtonProps::new("Save").with_variant(ButtonVariant::Primary),
        ))
        .insert(SaveButton)
        .insert(ChildOf(right));
}
