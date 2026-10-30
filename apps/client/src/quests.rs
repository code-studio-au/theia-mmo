//! Read-only standard Bevy journal for committed own-character objective state.
mod qualification;
use crate::network::Network;
use bevy::{prelude::*, text::FontSize};
use std::sync::Arc;
#[derive(Resource)]
pub(crate) struct Definitions(pub Arc<theia_network::quest_wire::Profile>);
#[derive(Component)]
pub(crate) struct Label;
pub(crate) fn install(app: &mut App, profile: Arc<theia_network::quest_wire::Profile>) {
    qualification::install(app);
    app.insert_resource(Definitions(profile))
        .add_systems(Startup, setup)
        .add_systems(Update, present.after(crate::network::present));
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(206),
                width: px(340),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.9)),
        ))
        .with_child((
            Label,
            Text::new("Objective: waiting for server"),
            TextFont {
                font_size: FontSize::Px(14.),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}
fn present(
    network: Res<Network>,
    definitions: Res<Definitions>,
    mut labels: Query<&mut Text, With<Label>>,
) {
    let state = network.state();
    let text = if state.error.is_some() {
        "Objective: disconnected".to_owned()
    } else if let Some(view) = &state.quest {
        let progress = definitions
            .0
            .profile()
            .progress(&view.progress)
            .expect("Validated committed objective");
        format!(
            "{}\n{}: {} / {}",
            definitions.0.profile().name(),
            if progress.complete {
                "Objective complete"
            } else {
                "Defeat creatures"
            },
            progress.count,
            progress.required
        )
    } else {
        "Objective: waiting for server".to_owned()
    };
    for mut label in &mut labels {
        if label.0 != text {
            label.0.clone_from(&text);
        }
    }
}
