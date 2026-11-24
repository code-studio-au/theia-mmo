//! Optional, read-only performance overlay. Values include presentation/vsync time.
use bevy::{picking::Pickable, prelude::*, text::FontSize};

#[derive(Resource, Default)]
pub(super) struct Enabled(pub bool);
#[derive(Component)]
struct Panel;
#[derive(Resource, Default)]
struct Sample {
    seconds: f64,
    frames: u32,
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<Enabled>()
        .init_resource::<Time<Real>>()
        .init_resource::<Sample>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                present,
                connection_visibility.after(crate::network::present),
            ),
        );
}
fn connection_visibility(
    texts: Query<(&Text, &ChildOf), With<crate::scene::NetworkHud>>,
    mut nodes: Query<&mut Node>,
) {
    for (text, parent) in &texts {
        if let Ok(mut node) = nodes.get_mut(parent.parent()) {
            node.display = if text.0.is_empty() {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Panel,
        Pickable::IGNORE,
        GlobalZIndex(25),
        Node {
            position_type: PositionType::Absolute,
            left: px(super::UNIT_WIDTH + 26.),
            top: px(128),
            padding: UiRect::all(px(10)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(super::PANEL),
        Text::new("Performance"),
        TextFont {
            font_size: FontSize::Px(14.),
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}
fn present(
    enabled: Res<Enabled>,
    time: Res<Time<Real>>,
    mut sample: ResMut<Sample>,
    network: Option<Res<crate::network::Network>>,
    windows: Query<&Window>,
    mut panels: Query<(&mut Node, &mut Text), With<Panel>>,
) {
    sample.seconds += time.delta_secs_f64();
    sample.frames += 1;
    for (mut node, mut text) in &mut panels {
        node.display = if enabled.0 {
            Display::Flex
        } else {
            Display::None
        };
        if !enabled.0 || sample.seconds < 0.25 {
            continue;
        }
        let frame_ms = sample.seconds * 1000. / f64::from(sample.frames);
        let mut value = format!(
            "Performance\nFPS: {:.0}   Frame: {:.2} ms",
            1000. / frame_ms,
            frame_ms
        );
        if let Ok(window) = windows.single() {
            value.push_str(&format!(
                "\nWindow: {:.0} × {:.0}",
                window.width(),
                window.height()
            ));
        }
        if let Some(network) = &network {
            let state = network.state();
            value.push_str(if network.connected() {
                "\nConnected"
            } else {
                "\nDisconnected / loading"
            });
            if let Some(snapshot) = &state.snapshot {
                value.push_str(&format!(
                    " · {} Hz · tick {}\nPlayers: {}",
                    snapshot.tick_hz,
                    snapshot.tick,
                    snapshot.players.len()
                ));
            }
            value.push_str(&format!(
                "\nLoaded chunks: {}",
                state.chunk_report.as_ref().map_or(0, |r| r.chunks.len())
            ));
            if let Some(correction) = state.correction {
                let p = correction.state.pose.feet.metres();
                value.push_str(&format!(
                    "\nPosition: {:.1}, {:.1}, {:.1} m",
                    p[0], p[1], p[2]
                ));
            }
        }
        text.0 = value;
    }
    if sample.seconds >= 0.25 {
        sample.seconds = 0.;
        sample.frames = 0;
    }
}
