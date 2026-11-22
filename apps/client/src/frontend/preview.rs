//! Native front-end model uses the same curated parts and animation as gameplay.
use super::*;
use bevy::{
    asset::LoadState,
    camera::{RenderTarget, visibility::RenderLayers},
    world_serialization::WorldAssetRoot,
};
const LAYER: usize = 20;
#[derive(Resource)]
pub(super) struct Preview {
    pub(super) camera: Entity,
    model: Option<Handle<Gltf>>,
    root: Option<Entity>,
    graph: Option<(Handle<AnimationGraph>, AnimationNodeIndex)>,
    pub(super) ready: bool,
    settled: u32,
    appearance: Option<Appearance>,
    pub(super) error: Option<String>,
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
            PreviewCamera,
            Camera3d::default(),
            Camera {
                order: -3,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            RenderTarget::Image(image.into()),
            RenderLayers::layer(LAYER),
            AmbientLight {
                brightness: 220.,
                ..default()
            },
            bevy::core_pipeline::tonemapping::Tonemapping::ReinhardLuminance,
            Transform::from_xyz(0., 1.2, 3.2).looking_at(Vec3::new(0., 0.95, 0.), Vec3::Y),
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
        settled: 0,
        appearance: None,
        error: None,
    });
    camera
}
pub(super) fn tick(world: &mut World) {
    let menu = world.resource::<Menu>();
    let show = menu.active() && menu.page != Page::Login;
    let (appearance, yaw) = (menu.appearance, menu.yaw);
    world.resource_scope(|world, mut preview: Mut<Preview>| {
        world.get_mut::<Camera>(preview.camera).unwrap().is_active = show;
        if !show {
            if let Some(root) = preview.root {
                world.entity_mut(root).insert(Visibility::Hidden);
            }
            return;
        }
        if preview.appearance != Some(appearance) {
            preview.appearance = Some(appearance);
            preview.settled = 0;
        }
        match prepare(world, &mut preview, appearance, yaw) {
            Ok(ready) => {
                preview.settled = if ready
                    && world
                        .resource::<crate::capture_ready::CaptureReady>()
                        .ready()
                {
                    preview.settled.saturating_add(1)
                } else {
                    0
                };
                preview.ready = preview.settled >= 8;
            }
            Err(error) => {
                world.resource_mut::<Menu>().message = error.clone();
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
            .get_or_insert_with(|| server.load("players://character-variants.glb"));
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
    let ready = crate::modular_hero::apply(world, root, appearance, "players")?;
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
