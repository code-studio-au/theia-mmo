//! Exercise the retained particle confirmation widget without a window or input device.
use std::time::Duration;

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    camera::RenderTarget,
    prelude::*,
    render::render_resource::TextureFormat,
    window::ExitCondition,
    winit::WinitPlugin,
};
use bevy_sprinkles_editor::{
    assets,
    hosting::AuthoringHostPlugin,
    ui::widgets::dialog::{self, DialogVariant, EditorDialog, OpenDialogEvent},
};

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_target_texture(
        512,
        384,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands.spawn((Camera2d, RenderTarget::Image(image.into())));
    let mut dialog = OpenDialogEvent::new("Unsaved particle project", "Discard changes")
        .with_variant(DialogVariant::Destructive)
        .with_close_button(false)
        .with_close_on_click_outside(false);
    dialog.description = Some("Your current particle project has unsaved changes. Save it first or discard them to continue.".into());
    commands.trigger(dialog);
}

fn target_dialog(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera2d>>,
    dialogs: Query<Entity, (With<EditorDialog>, Without<UiTargetCamera>)>,
) {
    if let Ok(camera) = cameras.single() {
        for entity in &dialogs {
            commands.entity(entity).insert(UiTargetCamera(camera));
        }
    }
}

fn verify(
    mut frame: Local<usize>,
    dialogs: Query<&ComputedNode, With<EditorDialog>>,
    mut exits: MessageWriter<AppExit>,
) {
    *frame += 1;
    if *frame == 120 {
        if dialogs
            .iter()
            .any(|node| node.size.x > 0.0 && node.size.y > 0.0)
        {
            println!("headless particle confirmation layout passed");
            exits.write(AppExit::Success);
        } else {
            eprintln!(
                "headless particle confirmation layout missing: {:?}",
                dialogs.iter().map(|node| node.size).collect::<Vec<_>>()
            );
            exits.write(AppExit::error());
        }
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins((AuthoringHostPlugin, assets::plugin, dialog::plugin))
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(16)))
        .add_systems(Startup, setup)
        .add_systems(Update, target_dialog)
        .add_systems(PostUpdate, verify)
        .run()
}
