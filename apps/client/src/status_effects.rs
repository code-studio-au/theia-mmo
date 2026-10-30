//! Selected-target status observations from the authority. No local damage ticking.
use crate::network::Network;
use bevy::{prelude::*, text::FontSize};
use std::sync::Arc;
use theia_core::statuses::Catalog;
#[derive(Resource)]
struct Definitions(Arc<Catalog>);
#[derive(Component)]
struct Label;
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub active_frames: u64,
    pub empty_frames: u64,
    pub own_frames: u64,
    pub other_frames: u64,
    pub max_rows: usize,
}
pub(crate) fn install(app: &mut App, catalog: Arc<Catalog>) {
    app.insert_resource(Definitions(catalog))
        .init_resource::<Metrics>()
        .add_systems(Startup, setup)
        .add_systems(Update, present.after(crate::network::present));
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                bottom: px(270),
                width: px(348),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.9)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Label,
                Text::new("Statuses: waiting for server"),
                TextFont {
                    font_size: FontSize::Px(14.),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
fn present(
    network: Res<Network>,
    definitions: Res<Definitions>,
    mut metrics: ResMut<Metrics>,
    mut labels: Query<&mut Text, With<Label>>,
) {
    let state = network.state();
    let selected = state.targeting.as_ref().and_then(|v| v.selected);
    let text = if state.error.is_some() {
        "Statuses: disconnected".to_owned()
    } else if let Some(view) = &state.statuses {
        let now = state
            .snapshot
            .as_ref()
            .map_or(view.tick, |s| s.tick.max(view.tick));
        let rows: Vec<_> = view
            .statuses
            .iter()
            .filter(|s| Some(s.target.entity) == selected)
            .collect();
        metrics.max_rows = metrics.max_rows.max(rows.len());
        if rows.is_empty() {
            metrics.empty_frames += 1;
            if selected.is_some() {
                "Target statuses: none"
            } else {
                "Select a target to inspect statuses"
            }
            .to_owned()
        } else {
            metrics.active_frames += 1;
            let mut text = String::from("Target statuses");
            for status in rows {
                let own = state.id == Some(status.source.entity.value());
                if own {
                    metrics.own_frames += 1;
                } else {
                    metrics.other_frames += 1;
                }
                let definition = definitions
                    .0
                    .definition(&status.status)
                    .expect("Validated status catalog");
                let seconds =
                    status.expires_at.saturating_sub(now) as f64 / f64::from(view.rate.hz());
                text.push_str(&format!(
                    "\n{}{} | {:.1}s",
                    definition.name(),
                    if own { " (you)" } else { " (other)" },
                    seconds
                ));
            }
            text
        }
    } else {
        "Statuses: waiting for server".to_owned()
    };
    for mut label in &mut labels {
        if label.0 != text {
            label.0.clone_from(&text);
        }
    }
}
