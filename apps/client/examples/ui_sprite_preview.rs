//! Offline atlas/nine-slice qualification. Does not connect to an account or server.
use bevy::{
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    sprite::{BorderRect, TextureSlicer},
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Deserialize)]
struct Manifest {
    assets: BTreeMap<String, Sprite>,
    atlases: Vec<Atlas>,
}
#[derive(Deserialize)]
struct Atlas {
    file: String,
}
#[derive(Deserialize)]
struct Sprite {
    size: [f32; 2],
    rect: [f32; 4],
    atlas: usize,
    nine_slice: Option<[f32; 4]>,
    #[serde(default = "one")]
    pixels_per_ui_unit: f32,
}
fn one() -> f32 {
    1.0
}

#[derive(Resource)]
struct Run {
    output: PathBuf,
    frame: usize,
    pages: usize,
    captured: usize,
}
#[derive(Component)]
struct Page(usize);

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("build/ui-sprites-production/native"));
    std::fs::create_dir_all(&output).unwrap();
    App::new()
        .insert_resource(Run {
            output,
            frame: 0,
            pages: 0,
            captured: 0,
        })
        .insert_resource(ClearColor(Color::srgb(0.04, 0.07, 0.1)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Theia sprite qualification".into(),
                resolution: (1440, 1000).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}

fn sprite_node(sprite: &Sprite, images: &[Handle<Image>]) -> ImageNode {
    let [x0, y0, x1, y1] = sprite.rect;
    ImageNode {
        image: images[sprite.atlas].clone(),
        rect: Some(Rect::new(x0, y0, x1, y1)),
        image_mode: sprite.nine_slice.map_or(
            NodeImageMode::Stretch,
            |[left, top, right, bottom]| {
                NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect {
                        min_inset: Vec2::new(left, top),
                        max_inset: Vec2::new(right, bottom),
                    },
                    max_corner_scale: 1.0 / sprite.pixels_per_ui_unit,
                    ..default()
                })
            },
        ),
        ..default()
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut run: ResMut<Run>) {
    commands.spawn(Camera2d);
    let root = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content/ui/sprites-v1")
        });
    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let textures: Vec<_> = manifest
        .atlases
        .iter()
        .map(|a| {
            images.add(
                Image::from_buffer(
                    &std::fs::read(root.join(&a.file)).unwrap(),
                    bevy::image::ImageType::Extension("png"),
                    bevy::image::CompressedImageFormats::NONE,
                    true,
                    bevy::image::ImageSampler::linear(),
                    bevy::asset::RenderAssetUsages::default(),
                )
                .unwrap(),
            )
        })
        .collect();
    let names: Vec<_> = manifest.assets.keys().collect();
    run.pages = 1 + names.len().div_ceil(48);
    for page in 0..run.pages {
        commands
            .spawn((
                Page(page),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|parent| {
                let entries: Vec<(String, f32, f32, f32, f32)> = if page == 0 {
                    [
                        ("panel.navy", 20., 20., 690., 420.),
                        ("panel.parchment", 730., 20., 690., 220.),
                        ("panel.compact-danger", 730., 260., 690., 180.),
                        ("button.primary.default", 20., 470., 690., 70.),
                        ("button.primary.selected", 730., 470., 690., 70.),
                        ("title.ornate", 20., 570., 1400., 75.),
                        ("notice.warning", 20., 680., 1400., 65.),
                        ("notice.success", 20., 780., 1400., 65.),
                    ]
                    .into_iter()
                    .map(|(n, x, y, w, h)| {
                        let sprite = &manifest.assets[n];
                        let h = if n.starts_with("notice.") {
                            sprite.size[1] / sprite.pixels_per_ui_unit
                        } else {
                            h
                        };
                        (n.to_owned(), x, y, w, h)
                    })
                    .collect()
                } else {
                    names
                        .iter()
                        .skip((page - 1) * 48)
                        .take(48)
                        .enumerate()
                        .map(|(i, n)| {
                            let a = &manifest.assets[*n];
                            let scale = (214.0 / a.size[0])
                                .min(87.0 / a.size[1])
                                .min(1.0 / a.pixels_per_ui_unit);
                            (
                                n.to_string(),
                                (i % 6) as f32 * 240. + 10.,
                                (i / 6) as f32 * 123. + 5.,
                                a.size[0] * scale,
                                a.size[1] * scale,
                            )
                        })
                        .collect()
                };
                for (i, (name, x, y, w, h)) in entries.into_iter().enumerate() {
                    if page > 0 {
                        parent.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(x),
                                top: px(y),
                                width: px(220),
                                height: px(90),
                                ..default()
                            },
                            BackgroundColor(if i % 2 == 0 {
                                Color::srgb(0.88, 0.86, 0.82)
                            } else {
                                Color::srgb(0.02, 0.03, 0.04)
                            }),
                        ));
                    }
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(x),
                            top: px(y),
                            width: px(w),
                            height: px(h),
                            ..default()
                        },
                        sprite_node(&manifest.assets[&name], &textures),
                    ));
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(x + 8.),
                            top: px(y + if page == 0 { 25. } else { 94. }),
                            ..default()
                        },
                        Text::new(name),
                        TextFont {
                            font_size: 13.0.into(),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
    }
}

fn capture(
    mut commands: Commands,
    mut run: ResMut<Run>,
    mut pages: Query<(&Page, &mut Node)>,
    adapter: Res<RenderAdapterInfo>,
) {
    let page = run.captured;
    if run.frame == 0 {
        for (id, mut node) in &mut pages {
            node.display = if id.0 == page {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
    run.frame += 1;
    if run.frame == 40 {
        let output = run.output.clone();
        let device = serde_json::json!({"name":adapter.name,"backend":format!("{:?}",adapter.backend),"driver":adapter.driver,"driver_info":adapter.driver_info});
        commands.spawn(Screenshot::primary_window()).observe(move |event: On<ScreenshotCaptured>, mut run: ResMut<Run>, mut exit: MessageWriter<AppExit>| {
            event.image.clone().try_into_dynamic().unwrap().save(output.join(format!("page-{page:02}.png"))).unwrap();
            run.captured += 1;
            run.frame = 0;
            if run.captured == run.pages {
                std::fs::write(output.join("report.json"),serde_json::to_vec_pretty(&serde_json::json!({"completed":true,"pages":run.pages,"gpu":device,"scope":"Offline standard Bevy UI: all sprite atlas rectangles plus resized nine-slice samples. No gameplay integration or Windows qualification."})).unwrap()).unwrap();
                exit.write(AppExit::Success);
            }
        });
    }
    assert!(run.frame < 600, "Screenshot capture timed out");
}
