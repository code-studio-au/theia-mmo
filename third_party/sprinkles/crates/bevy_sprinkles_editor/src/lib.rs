#![allow(deprecated)]
// Retained upstream BSN formatting; modern scene composition is ported.
// Bevy 0.20 scene macros generate unit bindings for nested scenes.
#![allow(clippy::let_unit_value)]
pub mod assets;
pub mod io;
pub mod plugin;
pub mod project;
pub mod state;
pub mod ui;
mod utils;
pub mod viewport;
mod viewport_bounds;

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

use plugin::SprinklesEditorPlugin;
use ui::EditorUiPlugin;

/// Builds the retained Sprinkles editor with Theia's Bevy version and data namespace.
pub fn standalone_app() -> App {
    let editor_data = io::load_editor_data();
    let present_mode = if editor_data.settings.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Theia Particle Editor — Sprinkles".into(),
                    resolution: WindowResolution::new(1366, 768),
                    present_mode,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            }),
    )
    .add_plugins(bevy_easings::EasingsPlugin::default())
    .add_plugins(SprinklesEditorPlugin)
    .add_plugins(EditorUiPlugin);
    app
}
