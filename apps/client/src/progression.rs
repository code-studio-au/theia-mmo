//! Committed own-character progression using standard Bevy UI.
pub(crate) mod feedback;
use crate::network::Network;
use bevy::{prelude::*, text::FontSize};
use std::sync::Arc;
#[derive(Resource)]
pub(crate) struct Profile(pub Arc<theia_network::progression_wire::Profile>);
#[derive(Component)]
pub(crate) struct Label;
#[derive(Component)]
struct Fill;
#[derive(Component)]
pub(crate) struct Root;
/// Actual progression widgets with clearly synthetic values; no Network/owner.
pub(crate) fn install_preview(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(
        PostStartup,
        |mut labels: Query<&mut Text, With<Label>>, mut fills: Query<&mut Node, With<Fill>>| {
            for mut label in &mut labels {
                **label = "Level 2 | XP 240 / 600 (fixture)".into();
            }
            for mut fill in &mut fills {
                fill.width = percent(40);
            }
        },
    );
}
pub(crate) fn install(app: &mut App, profile: Arc<theia_network::progression_wire::Profile>) {
    qualification::install(app);
    feedback::install(app);
    app.insert_resource(Profile(profile))
        .add_systems(Startup, setup)
        .add_systems(Update, present.after(crate::network::present));
}
fn setup(mut commands: Commands, skin: Option<Res<crate::fantasy_ui::Skin>>) {
    if skin.is_some() {
        use bevy::picking::{Pickable, hover::Hovered};
        commands
            .spawn((
                Root,
                Hovered::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    margin: UiRect::left(px(-crate::fantasy_ui::ACTION_WIDTH * 0.5)),
                    bottom: px(5),
                    width: px(crate::fantasy_ui::ACTION_WIDTH),
                    height: px(13),
                    ..default()
                },
            ))
            .with_children(|root| {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(5),
                        right: px(5),
                        top: px(3),
                        bottom: px(3),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(crate::fantasy_ui::PANEL),
                    Pickable::IGNORE,
                ))
                .with_child((
                    Fill,
                    crate::ui_theme::Sprite("bar.fill-experience"),
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                ));
                root.spawn((
                    crate::ui_theme::Sprite("bar.frame"),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                ));
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        margin: UiRect::left(px(-6)),
                        top: px(1),
                        width: px(12),
                        height: px(12),
                        ..default()
                    },
                    crate::ui_theme::Sprite("marker.diamond-gold"),
                ));
                root.spawn((
                    Label,
                    Text::new("Experience: waiting for server"),
                    TextLayout {
                        justify: bevy::text::Justify::Center,
                        ..default()
                    },
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    crate::fantasy_ui::hud_shadow(),
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(100),
                        bottom: px(90),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Visibility::Hidden,
                    Pickable::IGNORE,
                ));
            });
        return;
    }
    commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(148),
                width: px(300),
                padding: UiRect::all(px(8)),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.9)),
        ))
        .with_children(|p| {
            p.spawn((
                Label,
                Text::new("Experience: waiting for server"),
                TextFont {
                    font_size: FontSize::Px(14.),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                Node {
                    width: percent(100),
                    height: px(8),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
            ))
            .with_children(|p| {
                p.spawn((
                    Fill,
                    crate::ui_theme::Sprite("bar.fill-experience"),
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.55, 0.3, 0.85)),
                ));
            });
        });
}
fn present(
    network: Res<Network>,
    profile: Res<Profile>,
    mut labels: Query<(&mut Text, &mut Visibility), With<Label>>,
    mut bars: Query<&mut Node, With<Fill>>,
    hover: Query<&bevy::picking::hover::Hovered, With<Root>>,
) {
    let state = network.state();
    let (text, fraction) = if state.error.is_some() {
        ("Experience: disconnected".to_owned(), 0.)
    } else if let Some(view) = &state.progression {
        let progress = profile
            .0
            .profile()
            .progress(&view.progress)
            .expect("Validated authoritative progression");
        match progress.required_in_level {
            Some(required) => (
                format!(
                    "Level {} | XP {} / {}",
                    progress.level, progress.earned_in_level, required
                ),
                f64::from(progress.earned_in_level) / f64::from(required),
            ),
            None => (format!("Level {} | maximum level", progress.level), 1.),
        }
    } else {
        ("Experience: waiting for server".to_owned(), 0.)
    };
    let visible = if hover.single().is_ok_and(|hovered| !hovered.0) {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for (mut label, mut visibility) in &mut labels {
        if *visibility != visible {
            *visibility = visible;
        }
        if label.0 != text {
            label.0.clone_from(&text);
        }
    }
    for mut bar in &mut bars {
        let width = percent((fraction * 100.) as f32);
        if bar.width != width {
            bar.width = width;
        }
    }
}

/// Enabled only by the owned native qualification. Reads ordinary UI and saved
/// observations; grants no XP, chooses no target and emits no gameplay command.
mod qualification {
    use super::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    #[derive(Resource)]
    struct Run {
        output: std::path::PathBuf,
        samples: u64,
        revision: Option<u64>,
        shot: bool,
    }
    pub(super) fn install(app: &mut App) {
        if std::env::var("THEIA_PROGRESSION_QUALIFY").as_deref() != Ok("1") {
            return;
        }
        app.insert_resource(Run {
            output: std::env::var_os("THEIA_GRAYBOX_QUALIFY")
                .expect("XP proof requires output")
                .into(),
            samples: 0,
            revision: None,
            shot: false,
        })
        .add_systems(Last, observe);
    }
    fn observe(
        mut commands: Commands,
        network: Res<Network>,
        profile: Res<Profile>,
        mut run: ResMut<Run>,
        labels: Query<(&Text, &ComputedNode), With<Label>>,
        bars: Query<&ComputedNode, With<Fill>>,
        windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    ) {
        let state = network.state();
        if state.error.is_some() {
            return;
        }
        let Some(view) = &state.progression else {
            return;
        };
        if run.revision != Some(view.progress.revision()) {
            run.samples = 0;
            run.revision = Some(view.progress.revision());
        }
        run.samples += 1;
        if run.samples != 12 || view.progress.total() == 0 {
            return;
        }
        let progress = profile.0.profile().progress(&view.progress).unwrap();
        let Some((label, node)) = labels.iter().next() else {
            return;
        };
        let Some(bar) = bars.iter().next() else {
            return;
        };
        let Some(window) = windows.iter().next() else {
            return;
        };
        let expected = match progress.required_in_level {
            Some(required) => format!(
                "Level {} | XP {} / {}",
                progress.level, progress.earned_in_level, required
            ),
            None => format!("Level {} | maximum level", progress.level),
        };
        let valid = label.0 == expected
            && node.size().x > 0.
            && node.size().x < window.width()
            && node.size().y > 0.
            && node.size().y < 80.
            && bar.size().x.is_finite();
        if valid && !run.shot {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(run.output.join("progression.png")));
            run.shot = true;
        }
        let report = serde_json::json!({"passed":valid&&run.shot&&progress.level>=2,"actor":view.actor.entity.value(),"epoch":view.actor.epoch.value(),"profile_hash":view.profile_hash,"revision":view.progress.revision(),"total":progress.total,"level":progress.level,"text":label.0,"samples":run.samples,"label_size":node.size().to_array(),"bar_width":bar.size().x,"window_size":[window.physical_width(),window.physical_height()],"scope":"Native committed own-character Bevy display; no client award or physical mouse input"});
        std::fs::write(
            run.output.join("progression.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}
