//! Bounded standard Bevy map presentation; never requests terrain or gameplay.
mod projection;
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    mesh::VertexAttributeValues,
    picking::Pickable,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::FontSize,
};
use projection::{DISPLAY, HeightTile, PIXELS, View};
use std::{collections::BTreeMap, time::Instant};
use theia_core::{coordinates::GlobalPosition, quests::lifecycle::Phase};
pub(crate) const VITALS_RIGHT: f32 = DISPLAY + 40.;

#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub visible: bool,
    pub raster_updates: u64,
    pub known_pixels: usize,
    pub resident_tiles_read: usize,
    pub max_raster_us: u64,
    pub max_present_us: u64,
    pub present_samples: u64,
    pub total_present_us: u64,
    pub texture_bytes: usize,
    pub guide_phase: Option<usize>,
    pub player_point: Option<[f32; 2]>,
    pub guide_point: Option<[f32; 2]>,
    pub heading_radians: f32,
    pub centre: Option<[f64; 3]>,
    pub bounds: Option<[f32; 4]>,
    pub arrow_forward: Option<[f32; 2]>,
}
#[derive(Resource)]
struct Map {
    image: Handle<Image>,
    view: Option<View>,
    epoch: Option<u64>,
    updated: f64,
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Arrow;
#[derive(Component)]
struct Guide;

pub(crate) fn install(app: &mut App) {
    app.init_resource::<Metrics>()
        .add_systems(Startup, setup)
        .add_systems(Last, observe)
        .add_systems(
            PostUpdate,
            present
                .after(avian3d::prelude::PhysicsSystems::Last)
                .before(bevy::ui::UiSystems::Layout),
        );
}
fn image(width: usize, pixels: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: width as u32,
            height: width as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}
fn label(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        Pickable::IGNORE,
    )
}
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let handle = images.add(image(PIXELS, vec![0; PIXELS * PIXELS * 4]));
    let mut arrow = vec![0; 16 * 16 * 4];
    for y in 0..16usize {
        for x in 0..16usize {
            let dx = (x as i32 - 7).abs();
            if (y < 10 && dx <= y as i32 / 2) || (y >= 8 && (6..=9).contains(&x)) {
                arrow[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4]
                    .copy_from_slice(&[245, 248, 255, 255]);
            }
        }
    }
    let arrow = images.add(image(16, arrow));
    commands.insert_resource(Map {
        image: handle.clone(),
        view: None,
        epoch: None,
        updated: 0.,
    });
    commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(12),
                width: px(DISPLAY + 16.),
                padding: UiRect::all(px(8)),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.96)),
        ))
        .with_children(|root| {
            root.spawn(label("N  |  Local area", 14.));
            root.spawn((
                Node {
                    width: px(DISPLAY),
                    height: px(DISPLAY),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ImageNode::new(handle),
                Pickable::IGNORE,
            ))
            .with_children(|map| {
                map.spawn((
                    Arrow,
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(18),
                        height: px(18),
                        ..default()
                    },
                    ImageNode::new(arrow),
                    Pickable::IGNORE,
                ));
                map.spawn((
                    Guide,
                    label("G", 18.),
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(18),
                        height: px(22),
                        display: Display::None,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ));
            });
            root.spawn(label("256 m across", 12.));
        });
}
fn raster(world: &mut World, view: View) -> (Vec<u8>, usize, usize) {
    let mut query = world.query::<&crate::graybox::Tile>();
    let meshes = world.resource::<Assets<Mesh>>();
    let tiles: BTreeMap<_, _> = query
        .iter(world)
        .take(theia_network::procedural::MAX_VISUALS)
        .filter_map(|tile| {
            let VertexAttributeValues::Float32x3(positions) = meshes
                .get(&tile.mesh)?
                .attribute(Mesh::ATTRIBUTE_POSITION)?
            else {
                return None;
            };
            Some((
                tile.id,
                HeightTile {
                    id: tile.id,
                    cells: tile.cells,
                    positions,
                },
            ))
        })
        .collect();
    let mut data = vec![0; PIXELS * PIXELS * 4];
    let mut known = 0;
    for z in 0..PIXELS {
        for x in 0..PIXELS {
            let height = view.pixel(x, z).and_then(|p| {
                let id = theia_network::procedural::owner(p)?;
                tiles.get(&id)?.height(p)
            });
            let color = match height {
                Some(h) => {
                    known += 1;
                    let tint = ((h + 40.) / 80.).clamp(0., 1.);
                    let contour = if h.rem_euclid(4.) < 0.35 { 0.8 } else { 1. };
                    [
                        ((42. + tint * 72.) * contour) as u8,
                        ((73. + tint * 68.) * contour) as u8,
                        ((48. + tint * 45.) * contour) as u8,
                        255,
                    ]
                }
                None => [15, 20, 26, 255],
            };
            data[(z * PIXELS + x) * 4..(z * PIXELS + x) * 4 + 4].copy_from_slice(&color);
        }
    }
    (data, known, tiles.len())
}
fn owner_position(
    world: &mut World,
    state: &theia_network::client::State,
) -> Option<GlobalPosition> {
    if state.error.is_some() || state.welcome.as_ref()?.id != state.id? {
        return None;
    }
    let authoritative = state.authority_focus()?;
    let local = world
        .query_filtered::<&Transform, With<crate::movement::LocalPawn>>()
        .iter(world)
        .next()
        .map(|t| t.translation.to_array());
    let predicted = local
        .and_then(theia_core::coordinates::LocalPosition::new)
        .and_then(|p| {
            world
                .resource::<crate::floating_origin::Frame>()
                .render
                .to_global(p)
        });
    // During admission/rebase, an old local transform is not a new map position.
    predicted
        .filter(|p| {
            View {
                centre: authoritative,
            }
            .project(*p)
            .is_some()
        })
        .or(Some(authoritative))
}
fn player_heading(world: &mut World, state: &theia_network::client::State) -> f32 {
    if let Some(controller) = world.get_resource::<crate::classic_controls::Controller>() {
        return controller.yaw();
    }
    if let Some(facing) = state.correction.and_then(|c| c.state.pose.facing) {
        return facing.radians() as f32;
    }
    // Retained non-Classic walking has no facing field. Read its already
    // presented character orientation; never infer a bearing from the camera.
    world
        .query_filtered::<&Transform, With<crate::movement::LocalPawn>>()
        .iter(world)
        .next()
        .map_or(0., |t| t.rotation.to_euler(EulerRot::YXZ).0)
}

fn present(world: &mut World) {
    let started = Instant::now();
    present_inner(world);
    let micros = started.elapsed().as_micros() as u64;
    let mut metrics = world.resource_mut::<Metrics>();
    metrics.max_present_us = metrics.max_present_us.max(micros);
    metrics.total_present_us += micros;
    metrics.present_samples += 1;
}
fn present_inner(world: &mut World) {
    let state = world.resource::<crate::network::Network>().state();
    let hidden = world
        .get_resource::<crate::inventory::Ui>()
        .is_some_and(|ui| ui.open);
    let position = (!hidden).then(|| owner_position(world, &state)).flatten();
    let visible = position.is_some();
    for mut node in world
        .query_filtered::<&mut Node, With<Root>>()
        .iter_mut(world)
    {
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    world.resource_mut::<Metrics>().visible = visible;
    let Some(position) = position else {
        world.resource_mut::<Metrics>().player_point = None;
        world.resource_mut::<Metrics>().guide_phase = None;
        world.resource_mut::<Metrics>().guide_point = None;
        return;
    };
    let now = world.resource::<Time>().elapsed_secs_f64();
    let epoch = state.welcome.as_ref().unwrap().epoch;
    let mut map = world.remove_resource::<Map>().unwrap();
    let update = map.epoch != Some(epoch) || now - map.updated >= 0.25;
    if update {
        let started = Instant::now();
        let view = View { centre: position };
        let (data, known, tiles) = raster(world, view);
        world
            .resource_mut::<Assets<Image>>()
            .get_mut(&map.image)
            .unwrap()
            .data = Some(data);
        map.view = Some(view);
        map.epoch = Some(epoch);
        map.updated = now;
        let mut metrics = world.resource_mut::<Metrics>();
        metrics.raster_updates += 1;
        metrics.known_pixels = known;
        metrics.resident_tiles_read = tiles;
        metrics.texture_bytes = (PIXELS * PIXELS + 16 * 16) * 4;
        metrics.max_raster_us = metrics
            .max_raster_us
            .max(started.elapsed().as_micros() as u64);
        metrics.centre = Some(position.metres());
    }
    let view = map.view.unwrap();
    world.insert_resource(map);
    let heading = player_heading(world, &state);
    let Some(player_point) = view.project(position) else {
        for mut node in world
            .query_filtered::<&mut Node, With<Root>>()
            .iter_mut(world)
        {
            if node.display != Display::None {
                node.display = Display::None;
            }
        }
        let mut metrics = world.resource_mut::<Metrics>();
        metrics.visible = false;
        metrics.player_point = None;
        metrics.guide_point = None;
        metrics.guide_phase = None;
        return;
    };
    for (mut node, mut transform) in world
        .query_filtered::<(&mut Node, &mut UiTransform), With<Arrow>>()
        .iter_mut(world)
    {
        let (left, top) = (px(player_point[0] - 9.), px(player_point[1] - 9.));
        if node.left != left || node.top != top {
            node.left = left;
            node.top = top;
        }
        let rotation = Rot2::radians(-heading);
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
    let guide = crate::quest_marker::guide_observation(
        &state,
        world
            .get_resource::<crate::quest_lifecycle::Definitions>()
            .map(|d| d.0.as_ref()),
        world
            .get_resource::<crate::quest_journal::Definitions>()
            .map(|d| d.0.as_ref()),
    )
    .and_then(|(phase, position, _)| Some((phase, view.project(position)?)));
    for (mut node, mut text, mut color) in world
        .query_filtered::<(&mut Node, &mut Text, &mut TextColor), With<Guide>>()
        .iter_mut(world)
    {
        let display = if guide.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if let Some((phase, point)) = guide {
            let glyph = match phase {
                Phase::Available => "!",
                Phase::Ready => "?",
                _ => "G",
            };
            if text.0 != glyph {
                text.0 = glyph.into();
            }
            let tint = if matches!(phase, Phase::Available | Phase::Ready) {
                Color::srgb(1., 0.83, 0.15)
            } else {
                Color::srgb(0.5, 0.9, 1.)
            };
            if color.0 != tint {
                color.0 = tint;
            }
            let (left, top) = (px(point[0] - 9.), px(point[1] - 11.));
            if node.left != left || node.top != top {
                node.left = left;
                node.top = top;
            }
        }
    }
    let mut metrics = world.resource_mut::<Metrics>();
    metrics.player_point = Some(player_point);
    metrics.heading_radians = heading;
    metrics.guide_phase = guide.map(|(p, _)| match p {
        Phase::Available => 0,
        Phase::Active => 1,
        Phase::Ready => 2,
        Phase::TurnedIn => 3,
    });
    metrics.guide_point = guide.map(|(_, p)| p);
}

fn observe(
    roots: Query<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Root>>,
    arrows: Query<&bevy::ui::UiGlobalTransform, With<Arrow>>,
    mut metrics: ResMut<Metrics>,
) {
    metrics.arrow_forward = metrics
        .visible
        .then(|| {
            let transform = arrows.single().ok()?;
            (transform.affine().matrix2 * Vec2::NEG_Y)
                .try_normalize()
                .map(|v| v.to_array())
        })
        .flatten();
    metrics.bounds = metrics
        .visible
        .then(|| {
            let (node, transform) = roots.single().ok()?;
            let centre = transform.affine().translation * node.inverse_scale_factor();
            let size = node.size() * node.inverse_scale_factor();
            Some([
                centre.x - size.x * 0.5,
                centre.y - size.y * 0.5,
                size.x,
                size.y,
            ])
        })
        .flatten();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_walking_arrow_follows_actual_character_rotation_without_camera_authority() {
        let mut world = World::new();
        world.insert_resource(crate::camera::Orbit::default());
        let pawn = world
            .spawn((crate::movement::LocalPawn, Transform::default()))
            .id();
        for radians in [0.9, std::f32::consts::FRAC_PI_2, std::f32::consts::PI, 4.7] {
            let rotation = Quat::from_rotation_y(radians);
            world.get_mut::<Transform>(pawn).unwrap().rotation = rotation;
            let yaw = player_heading(&mut world, &theia_network::client::State::default());
            let actual = rotation * Vec3::NEG_Z;
            assert!(Vec2::new(-yaw.sin(), -yaw.cos()).distance(actual.xz()) < 0.00001);
        }
    }
}
