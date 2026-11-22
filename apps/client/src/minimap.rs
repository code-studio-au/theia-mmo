//! Bounded standard Bevy map presentation; never requests terrain or gameplay.
mod authored;
mod guidance;
pub(crate) mod hints;
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
pub(crate) use projection::heading_rotation;
use projection::{DISPLAY, HeightTile, PIXELS, View};
use std::{collections::BTreeMap, time::Instant};
use theia_core::{coordinates::GlobalPosition, quests::lifecycle::Phase};
pub(crate) const VITALS_RIGHT: f32 = DISPLAY + 40.;

#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub visible: bool,
    pub hover_text: Option<String>,
    pub raster_updates: u64,
    pub known_pixels: usize,
    pub objective_regions: usize,
    pub objective_pixels: usize,
    pub resident_tiles_read: usize,
    pub authored_cached_samples: usize,
    pub authored_height_queries: u64,
    pub max_authored_warm_us: u64,
    pub max_raster_us: u64,
    pub max_present_us: u64,
    pub present_samples: u64,
    pub total_present_us: u64,
    pub texture_bytes: usize,
    pub guide_phase: Option<usize>,
    pub player_point: Option<[f32; 2]>,
    pub guide_point: Option<[f32; 2]>,
    pub guide_bounds: Option<[f32; 4]>,
    pub tooltip_bounds: Option<[f32; 4]>,
    pub heading_radians: f32,
    pub centre: Option<[f64; 3]>,
    pub bounds: Option<[f32; 4]>,
    pub surface_bounds: Option<[f32; 4]>,
    pub arrow_forward: Option<[f32; 2]>,
}
#[derive(Resource)]
struct Map {
    image: Handle<Image>,
    view: Option<View>,
    epoch: Option<u64>,
    updated: f64,
    authored: authored::Cache,
}
#[derive(Component)]
pub(crate) struct Root;
#[derive(Component)]
pub(crate) struct Surface;
#[derive(Component)]
struct Arrow;
#[derive(Component)]
struct Guide(usize);
#[derive(Component)]
enum MapCaption {
    Heading,
    Extent,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<Metrics>()
        .add_systems(Startup, (setup, hints::setup))
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
fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
) {
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
        authored: default(),
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
            BackgroundColor(if skin.is_some() {
                Color::NONE
            } else {
                Color::srgba(0.02, 0.03, 0.05, 0.96)
            }),
        ))
        .with_children(|root| {
            root.spawn((MapCaption::Heading, label("N  |  Local area", 14.)));
            root.spawn((
                Surface,
                bevy::ui::RelativeCursorPosition::default(),
                Node {
                    width: px(DISPLAY),
                    height: px(DISPLAY),
                    overflow: Overflow::clip(),
                    border_radius: if skin.is_some() {
                        BorderRadius::MAX
                    } else {
                        BorderRadius::ZERO
                    },
                    ..default()
                },
                ImageNode::new(handle),
                Pickable::IGNORE,
                Outline::new(
                    px(if skin.is_some() { 2. } else { 0. }),
                    px(-1),
                    crate::fantasy_ui::GOLD,
                ),
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
                for index in 0..theia_core::dialogue::catalog::MAX_CONTACTS {
                    map.spawn((
                        Guide(index),
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
                }
            });
            root.spawn((MapCaption::Extent, label("256 m across", 12.)));
        });
}
/// Same widgets as the live map, with an explicitly synthetic raster. No World,
/// terrain, network or map-present system is installed by the developer preview.
pub(crate) fn install_preview(app: &mut App) {
    app.init_resource::<Metrics>()
        .add_systems(Startup, (setup, hints::setup))
        .add_systems(PostStartup, (seed_preview, seed_hover_preview).chain());
}
type PreviewNode<'a> = (&'a mut Node, Has<Root>, Has<Arrow>, Option<&'a Guide>);
fn seed_preview(
    map: Res<Map>,
    mut metrics: ResMut<Metrics>,
    mut images: ResMut<Assets<Image>>,
    mut nodes: Query<PreviewNode<'_>>,
    mut captions: Query<(&mut Text, Option<&MapCaption>, Has<Guide>)>,
) {
    let mut pixels = vec![0; PIXELS * PIXELS * 4];
    for y in 0..PIXELS {
        for x in 0..PIXELS {
            let shade = if (x / 8 + y / 8) % 2 == 0 { 0 } else { 12 };
            pixels[(y * PIXELS + x) * 4..(y * PIXELS + x) * 4 + 4].copy_from_slice(&[
                44 + shade,
                62 + shade,
                42 + shade,
                255,
            ]);
        }
    }
    // Explicit metadata-only search-area sample; no quest/player observation.
    let areas = theia_core::quests::areas::Catalog::from_json(br#"{
        "schema":1,"id":"theia.test.map-preview-areas","revision":1,"areas":[
            {"id":"theia.test.map-preview-area","name":"Fixture search area","min":[-90,-10,-45],"max":[-15,10,45]}
        ]}"#).unwrap();
    let regions: Vec<_> = areas.areas().collect();
    metrics.objective_regions = regions.len();
    metrics.objective_pixels = guidance::shade(
        &mut pixels,
        View {
            north_positive_z: false,
            centre: GlobalPosition::new([0., 0., 0.]).unwrap(),
        },
        &regions,
        true,
    );
    images.get_mut(&map.image).unwrap().data = Some(pixels);
    for (mut node, root, arrow, guide) in &mut nodes {
        if root {
            node.display = Display::Flex;
        } else if arrow {
            node.left = px(DISPLAY * 0.5 - 9.);
            node.top = px(DISPLAY * 0.5 - 9.);
        } else if let Some(guide) = guide {
            node.display = if guide.0 == 0 {
                Display::Flex
            } else {
                Display::None
            };
            node.left = px(DISPLAY * 0.75 - 9.);
            node.top = px(DISPLAY * 0.25 - 11.);
        }
    }
    for (mut text, caption, guide) in &mut captions {
        if let Some(caption) = caption {
            **text = match caption {
                MapCaption::Heading => "N | Fixture map",
                MapCaption::Extent => "256 m fixture raster",
            }
            .into();
        } else if guide {
            **text = "!".into();
        }
    }
}
fn seed_hover_preview(world: &mut World) {
    hints::set(
        world,
        Some("Quest contact (fixture)\nAvailable quests\nMeet the guide".into()),
    );
}
fn raster(world: &mut World, view: View, cache: &authored::Cache) -> (Vec<u8>, usize, usize) {
    let mut query = world.query::<&crate::graybox::Tile>();
    let meshes = world.resource::<Assets<Mesh>>();
    let tiles: BTreeMap<_, _> = query
        .iter(world)
        // The Graphics setting may retain hundreds of distant tiles. Select
        // the minimap's 256 m footprint before bounding mesh reads.
        .filter(|tile| {
            view.centre.chunk(128).is_some_and(|centre| {
                (tile.id.x - centre.x).abs() <= 1 && (tile.id.z - centre.z).abs() <= 1
            })
        })
        .take(9)
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
    let authored = world.get_resource::<crate::authored_scene::Scene>();
    let mut data = vec![0; PIXELS * PIXELS * 4];
    let mut known = 0;
    for z in 0..PIXELS {
        for x in 0..PIXELS {
            let height = view.pixel(x, z).and_then(|p| {
                if let Some(scene) = authored.filter(|s| !s.has_streamed_ground()) {
                    scene.map_contains(p).then(|| cache.height(p)).flatten()
                } else {
                    let id = theia_network::procedural::owner(p)?;
                    tiles.get(&id)?.height(p)
                }
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
                None => view
                    .pixel(x, z)
                    .and_then(|p| crate::zone_map::cartography_pixel(world, p))
                    .unwrap_or([15, 20, 26, 255]),
            };
            data[(z * PIXELS + x) * 4..(z * PIXELS + x) * 4 + 4].copy_from_slice(&color);
        }
    }
    (data, known, tiles.len())
}
pub(crate) fn owner_position(
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
                north_positive_z: false,
                centre: authoritative,
            }
            .project(*p)
            .is_some()
        })
        .or(Some(authoritative))
}
pub(crate) fn player_heading(world: &mut World, state: &theia_network::client::State) -> f32 {
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
    hints::present(world);
    let micros = started.elapsed().as_micros() as u64;
    let mut metrics = world.resource_mut::<Metrics>();
    metrics.max_present_us = metrics.max_present_us.max(micros);
    metrics.total_present_us += micros;
    metrics.present_samples += 1;
}
fn present_inner(world: &mut World) {
    let state = world.resource::<crate::network::Network>().state();
    let position = owner_position(world, &state);
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
        world.resource_mut::<Metrics>().objective_regions = 0;
        world.resource_mut::<Metrics>().objective_pixels = 0;
        world.resource_mut::<Metrics>().player_point = None;
        world.resource_mut::<Metrics>().guide_phase = None;
        world.resource_mut::<Metrics>().guide_point = None;
        return;
    };
    let now = world.resource::<Time>().elapsed_secs_f64();
    let epoch = state.welcome.as_ref().unwrap().epoch;
    let mut map = world.remove_resource::<Map>().unwrap();
    if map.epoch != Some(epoch) {
        map.authored = default();
    }
    if let Some(scene) = world
        .get_resource::<crate::authored_scene::Scene>()
        .filter(|s| s.ready() && !s.has_streamed_ground())
    {
        let started = Instant::now();
        let queries = map.authored.warm(
            View {
                north_positive_z: false,
                centre: position,
            },
            |p| scene.map_height(p),
        );
        let elapsed = started.elapsed().as_micros() as u64;
        let mut metrics = world.resource_mut::<Metrics>();
        metrics.authored_cached_samples = map.authored.len();
        metrics.authored_height_queries += queries as u64;
        metrics.max_authored_warm_us = metrics.max_authored_warm_us.max(elapsed);
    }
    let update = map.epoch != Some(epoch) || now - map.updated >= 0.25;
    if update {
        let started = Instant::now();
        let view = View {
            north_positive_z: crate::zone_map::north_positive_z(world),
            centre: position,
        };
        let (mut data, known, tiles) = raster(world, view, &map.authored);
        let (objective_regions, objective_pixels) = world
            .get_resource::<crate::quest_journal::Definitions>()
            .map_or((0, 0), |definitions| {
                let regions = guidance::regions(&state, &definitions.0);
                let pixels = guidance::shade(
                    &mut data,
                    view,
                    &regions,
                    world.contains_resource::<crate::fantasy_ui::Skin>(),
                );
                (regions.len(), pixels)
            });
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
        metrics.objective_regions = objective_regions;
        metrics.objective_pixels = objective_pixels;
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
        let rotation = heading_rotation(heading, view.north_positive_z);
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
    let guides: Vec<_> = crate::quest_marker::guide_observations(
        &state,
        world
            .get_resource::<crate::quest_lifecycle::Definitions>()
            .map(|d| d.0.as_ref()),
        world
            .get_resource::<crate::quest_journal::Definitions>()
            .map(|d| d.0.as_ref()),
    )
    .into_iter()
    .filter_map(|(phase, position, _)| Some((phase, view.project(position)?)))
    .filter(|(_, point)| {
        !world.contains_resource::<crate::fantasy_ui::Skin>()
            || projection::circle_contains(*point, 15.)
    })
    .collect();
    for (index, mut node, mut text, mut color) in world
        .query::<(&Guide, &mut Node, &mut Text, &mut TextColor)>()
        .iter_mut(world)
    {
        let guide = guides.get(index.0).copied();
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
    let guide = guides.first().copied();
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
    surfaces: Query<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Surface>>,
    guides: Query<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Guide>>,
    tooltips: Query<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<hints::Tooltip>>,
    ui_scale: Option<Res<bevy::ui::UiScale>>,
    mut metrics: ResMut<Metrics>,
) {
    let bounds = |node: &ComputedNode, transform: &bevy::ui::UiGlobalTransform| {
        let factor =
            crate::fantasy_ui::window_factor(node, ui_scale.as_deref().map_or(1., |s| s.0));
        let centre = transform.affine().translation * factor;
        let size = node.size() * factor;
        [
            centre.x - size.x * 0.5,
            centre.y - size.y * 0.5,
            size.x,
            size.y,
        ]
    };
    metrics.surface_bounds = metrics
        .visible
        .then(|| {
            let (node, transform) = surfaces.single().ok()?;
            Some(bounds(node, transform))
        })
        .flatten();
    metrics.guide_bounds = (metrics.visible && metrics.guide_point.is_some())
        .then(|| {
            let (node, transform) = guides.iter().next()?;
            Some(bounds(node, transform))
        })
        .flatten();
    metrics.tooltip_bounds = metrics
        .hover_text
        .is_some()
        .then(|| {
            let (node, transform) = tooltips.single().ok()?;
            Some(bounds(node, transform))
        })
        .flatten();
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
            Some(bounds(node, transform))
        })
        .flatten();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maximum_view_distant_tiles_cannot_displace_nearby_minimap_samples() {
        let mut world = World::new();
        let mut assets = Assets::<Mesh>::default();
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0., 25., 0.]; 81]);
        let handle = assets.add(mesh);
        world.insert_resource(assets);
        for x in 40..140 {
            world.spawn(crate::graybox::Tile {
                id: theia_core::coordinates::ChunkId { x, z: 40 },
                cells: 8,
                mesh: handle.clone(),
            });
        }
        for z in 9..=11 {
            for x in 9..=11 {
                world.spawn(crate::graybox::Tile {
                    id: theia_core::coordinates::ChunkId { x, z },
                    cells: 8,
                    mesh: handle.clone(),
                });
            }
        }
        let view = View {
            north_positive_z: false,
            centre: GlobalPosition::new([1344., 0., 1344.]).unwrap(),
        };
        let (pixels, known, tiles) = raster(&mut world, view, &authored::Cache::default());
        assert_eq!(tiles, 9);
        assert_eq!(known, PIXELS * PIXELS);
        assert!(
            pixels
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p != [15, 20, 26, 255])
        );
    }
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
