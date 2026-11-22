//! Isolated standard Bevy character preview; cosmetic choices never touch the zone.
use super::*;
use bevy::{
    asset::{
        AssetApp, LoadState,
        io::{AssetSourceBuilder, file::FileAssetReader},
    },
    camera::{RenderTarget, visibility::RenderLayers},
    ui::widget::ViewportNode,
    world_serialization::WorldAssetRoot,
};
const LAYER: usize = 19;
#[derive(Resource)]
struct Available;
#[derive(Resource)]
pub(super) struct Preview {
    camera: Entity,
    model: Option<Handle<Gltf>>,
    root: Option<Entity>,
    graph: Option<(Handle<AnimationGraph>, AnimationNodeIndex)>,
    pub(super) ready: bool,
    pub(super) evidence: serde_json::Value,
    pub(super) error: Option<String>,
}
pub(crate) fn install(app: &mut App, directory: &Path) -> Result<(), String> {
    if !directory.join("modular-hero.json").is_file() {
        return Ok(());
    }
    let directory = directory.canonicalize().map_err(|e| e.to_string())?;
    app.register_asset_source(
        "hero-preview",
        AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(directory.clone()))),
    );
    app.insert_resource(Available);
    Ok(())
}
pub(super) fn setup(commands: &mut Commands, images: &mut Assets<Image>) -> Entity {
    let image = images.add(Image::new_target_texture(
        1,
        1,
        bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
        None,
    ));
    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: -3,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.035, 0.045, 0.06)),
                ..default()
            },
            RenderTarget::Image(image.into()),
            RenderLayers::layer(LAYER),
            AmbientLight {
                brightness: 220.,
                ..default()
            },
            bevy::core_pipeline::tonemapping::Tonemapping::ReinhardLuminance,
            Transform::from_xyz(0., 1.2, 3.6).looking_at(Vec3::new(0., 0.95, 0.), Vec3::Y),
        ))
        .id();
    commands.spawn((
        RenderLayers::layer(LAYER),
        DirectionalLight {
            illuminance: 16000.,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(3., 4., 5.).looking_at(Vec3::Y, Vec3::Y),
    ));
    commands.insert_resource(Preview {
        camera,
        model: None,
        root: None,
        graph: None,
        ready: false,
        evidence: serde_json::Value::Null,
        error: None,
    });
    camera
}
pub(super) fn panel(parent: &mut ChildSpawnerCommands, camera: Entity) {
    parent
        .spawn(Node {
            width: percent(42),
            flex_shrink: 0.,
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        })
        .with_children(|panel| {
            panel.spawn((
                ViewportNode::new(camera),
                Pickable::IGNORE,
                Node {
                    width: percent(100),
                    height: px(350),
                    ..default()
                },
            ));
            panel
                .spawn(Node {
                    column_gap: px(8),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|row| {
                    button(row, Choice::Rotate(false), "Turn left");
                    button(row, Choice::Rotate(true), "Turn right");
                });
            panel.spawn(theme::label(
                "Choose your look. Use the arrows to turn.",
                12.,
                theme::MUTED,
            ));
        });
}
pub(super) fn tick(world: &mut World) {
    let appearance = world
        .resource::<Picker>()
        .draft
        .as_ref()
        .map(|d| (d.appearance, d.yaw));
    let available = world.contains_resource::<Available>();
    world.resource_scope(|world, mut preview: Mut<Preview>| {
        world.get_mut::<Camera>(preview.camera).unwrap().is_active =
            appearance.is_some() && available;
        let Some((appearance, yaw)) = appearance.filter(|_| available) else {
            if let Some(root) = preview.root {
                world.entity_mut(root).insert(Visibility::Hidden);
            }
            return;
        };
        if preview.error.is_some() {
            return;
        }
        let result = prepare(world, &mut preview, appearance, yaw);
        match result {
            Ok(ready) => {
                preview.ready = ready;
                if ready {
                    preview.evidence = crate::modular_hero::qualification(world);
                }
            }
            Err(error) => {
                if let Some(draft) = &mut world.resource_mut::<Picker>().draft {
                    draft.message = error.clone();
                }
                preview.error = Some(error);
            }
        }
    });
}
fn prepare(
    world: &mut World,
    preview: &mut Preview,
    appearance: Appearance,
    yaw: f32,
) -> Result<bool, String> {
    if preview.root.is_none() {
        let server = world.resource::<AssetServer>();
        let model = preview
            .model
            .get_or_insert_with(|| server.load("hero-preview://character-variants.glb"));
        if matches!(server.load_state(model.id()), LoadState::Failed(_)) {
            return Err("Character preview failed to load".into());
        }
        if !server.is_loaded_with_dependencies(model.id()) {
            return Ok(false);
        }
        let gltf = world
            .resource::<Assets<Gltf>>()
            .get(model)
            .ok_or("Missing hero model")?;
        let scene = gltf
            .default_scene
            .clone()
            .ok_or("Hero has no default scene")?;
        let idle = gltf
            .named_animations
            .get("Idle")
            .ok_or("Hero has no idle animation")?
            .clone();
        let (graph, node) = AnimationGraph::from_clip(idle);
        preview.graph = Some((
            world.resource_mut::<Assets<AnimationGraph>>().add(graph),
            node,
        ));
        preview.root = Some(
            world
                .spawn((
                    WorldAssetRoot(scene),
                    Transform::default(),
                    Visibility::Hidden,
                    RenderLayers::layer(LAYER),
                ))
                .id(),
        );
    }
    let root = preview.root.unwrap();
    let mut entities = vec![root];
    let mut index = 0;
    while index < entities.len() {
        if let Some(children) = world.get::<Children>(entities[index]) {
            entities.extend(children.iter());
        }
        index += 1;
    }
    let (graph, node) = preview.graph.as_ref().unwrap();
    for entity in entities {
        world
            .entity_mut(entity)
            .insert((RenderLayers::layer(LAYER), Pickable::IGNORE));
        if world.get::<AnimationPlayer>(entity).is_some()
            && world.get::<AnimationGraphHandle>(entity).is_none()
        {
            world
                .get_mut::<AnimationPlayer>(entity)
                .unwrap()
                .start(*node)
                .repeat();
            world
                .entity_mut(entity)
                .insert(AnimationGraphHandle(graph.clone()));
        }
    }
    let ready = crate::modular_hero::apply(world, root, appearance, "hero-preview")?;
    world.entity_mut(root).insert((
        Transform::from_rotation(Quat::from_rotation_y(yaw)),
        if ready {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        },
    ));
    Ok(ready)
}
