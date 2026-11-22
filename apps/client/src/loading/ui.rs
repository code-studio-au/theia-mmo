//! Standard Bevy widgets over an original embedded illustration.
use super::{Artwork, Phase, State};
use bevy::{
    prelude::*,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
const GOLD: Color = Color::srgb(0.72, 0.56, 0.32);
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Art;
#[derive(Component)]
pub(super) struct Status;
#[derive(Component)]
pub(super) struct Progress;
#[derive(Component)]
pub(super) struct Cancel;
#[derive(Component)]
pub(super) struct CancelText;
fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(0.94, 0.88, 0.74)),
        Pickable::IGNORE,
    )
}
pub(super) fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    artwork: Res<Artwork>,
) {
    let camera = commands
        .spawn((
            super::presentation::LoadingCamera,
            Camera2d,
            Camera {
                order: 1000,
                clear_color: bevy::camera::ClearColorConfig::Custom(Color::srgb(
                    0.035, 0.028, 0.023,
                )),
                ..default()
            },
            bevy::camera::visibility::RenderLayers::none(),
        ))
        .id();
    let image = Image::from_buffer(
        artwork.bytes(),
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::default(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .expect("Embedded Theia loading illustration");
    let handle = images.add(image);
    commands.spawn((Root, Node {
        position_type: PositionType::Absolute, width: percent(100), height: percent(100),
        flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween, padding: UiRect::all(px(16)), ..default()
    }, UiTargetCamera(camera), BackgroundColor(Color::srgb(0.035, 0.028, 0.023)), GlobalZIndex(1000)))
    .with_children(|root| {
        if *artwork == Artwork::Valley {
            root.spawn((text("T H E I A", 36.), Node { height: px(56), flex_shrink: 0., ..default() }));
        }
        root.spawn((Art, ImageNode::new(handle), Node { width: px(800), height: px(450), flex_shrink: 0., ..default() }, Outline::new(px(2), px(0), GOLD), Pickable::IGNORE));
        root.spawn((Node { width: percent(90), max_width: px(1000), height: px(190), flex_shrink: 0., flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::SpaceEvenly, row_gap: px(8), ..default() }, Pickable::IGNORE)).with_children(|panel| {
            panel.spawn((Status, text("Connecting to the world...", 22.)));
            panel.spawn((Node { width: percent(90), height: px(14), overflow: Overflow::clip(), ..default() }, BackgroundColor(Color::srgb(0.09, 0.07, 0.05)), Outline::new(px(1), px(0), GOLD), Pickable::IGNORE)).with_children(|bar| {
                bar.spawn((Progress, Node { width: percent(0), height: percent(100), ..default() }, BackgroundColor(Color::srgb(0.25, 0.53, 0.61)), Pickable::IGNORE));
            });
            panel.spawn(text("Tip: Select a target, then choose an action. Your character faces where you turn.", 16.));
            panel.spawn((Cancel, Button, ActivateOnPress, Node { width: px(180), height: px(34), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }, BackgroundColor(Color::srgb(0.31, 0.085, 0.035)), Outline::new(px(1), px(0), GOLD)))
                .with_children(|button| { button.spawn((CancelText, text("Cancel  [Esc]", 16.))); });
        });
    });
}
type Nodes<'a> = (&'a mut Node, Has<Root>, Has<Art>, Has<Progress>);
#[allow(clippy::too_many_arguments)]
pub(super) fn present(
    state: Res<State>,
    artwork: Res<Artwork>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    mut nodes: Query<Nodes<'_>>,
    mut labels: Query<(&mut Text, Has<Status>, Has<CancelText>)>,
) {
    let size = windows
        .single()
        .map(|w| Vec2::new(w.width(), w.height()) / scale.0)
        .unwrap_or(Vec2::new(1720., 700.));
    let reserved = if *artwork == Artwork::Millhaven {
        230.
    } else {
        290.
    };
    let width = (size.x - 64.)
        .min((size.y - reserved).max(1.) * artwork.aspect())
        .max(1.);
    for (mut node, root, art, progress) in &mut nodes {
        if root {
            node.display = if state.phase == Phase::Playing {
                Display::None
            } else {
                Display::Flex
            };
        }
        if art {
            node.width = px(width);
            node.height = px(width / artwork.aspect());
        }
        if progress {
            node.width = percent(state.progress_percent() as f32);
        }
    }
    for (mut label, status, cancel) in &mut labels {
        if status {
            label.0 = state.label().into();
        }
        if cancel {
            label.0 = if state.phase == Phase::Failed {
                "Close  [Esc]"
            } else {
                "Cancel  [Esc]"
            }
            .into();
        }
    }
}
pub(super) fn cancel(
    event: On<Activate>,
    buttons: Query<(), With<Cancel>>,
    state: Res<State>,
    windows: Query<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.blocks_input()
        && buttons.contains(event.entity)
        && windows.single().is_ok_and(|w| w.focused)
    {
        exit.write(if state.phase == Phase::Failed {
            AppExit::error()
        } else {
            AppExit::Success
        });
    }
}

pub(super) fn geometry(world: &mut World) -> serde_json::Value {
    let scale = world.resource::<UiScale>().0;
    world
        .query::<(
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Has<Root>,
            Has<Art>,
            Has<Status>,
            Has<Cancel>,
        )>()
        .iter(world)
        .filter_map(|(node, transform, root, art, status, cancel)| {
            let name = if root {
                "root"
            } else if art {
                "art"
            } else if status {
                "status"
            } else if cancel {
                "cancel"
            } else {
                return None;
            };
            let factor = crate::fantasy_ui::window_factor(node, scale);
            let centre = transform.affine().translation * factor;
            let size = node.size() * factor;
            Some((
                name.to_string(),
                serde_json::json!([
                    centre.x - size.x / 2.,
                    centre.y - size.y / 2.,
                    size.x,
                    size.y
                ]),
            ))
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}
