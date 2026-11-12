//! One isolated, on-demand standard Bevy scene for the selected library model.
use super::*;
use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState},
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    ui::widget::ViewportNode,
    world_serialization::WorldInstanceReady,
};

const LAYER: usize = 2; // Zone: 0; embedded Sprinkles: 1.
#[derive(Resource)]
pub(super) struct Preview {
    pub camera: Entity,
    pub root: Option<(usize, Entity)>,
}
#[derive(Component)]
struct Ready;
#[derive(Component)]
struct PreviewImage;
#[derive(Component)]
struct PreviewLabel;
#[derive(Component)]
struct PreviewCamera;

pub(super) fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
                order: -2,
                is_active: false,
                clear_color: ClearColorConfig::Custom(theme::PANEL),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: 2.4,
                    min_height: 2.4,
                },
                ..OrthographicProjection::default_3d()
            }),
            RenderTarget::Image(image.into()),
            RenderLayers::layer(LAYER),
            AmbientLight {
                brightness: 180.,
                ..default()
            },
            bevy::core_pipeline::tonemapping::Tonemapping::ReinhardLuminance,
            Transform::from_xyz(3., 2., 4.).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .id();
    commands.spawn((
        RenderLayers::layer(LAYER),
        DirectionalLight {
            illuminance: 14000.,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(3., 5., 4.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(Preview { camera, root: None });
}

pub(super) fn panel(parent: &mut ChildSpawnerCommands, camera: Entity) {
    parent
        .spawn(Node {
            width: percent(25),
            max_width: px(280),
            min_width: px(100),
            min_height: px(0),
            flex_shrink: 0.,
            flex_direction: FlexDirection::Column,
            border: UiRect::left(px(1)),
            padding: UiRect::all(px(8)),
            ..default()
        })
        .insert(BorderColor::all(theme::BORDER))
        .with_children(|panel| {
            panel.spawn((
                PreviewLabel,
                theme::label("Select a model to preview", 12., theme::MUTED),
            ));
            panel.spawn((
                PreviewImage,
                Pickable::IGNORE,
                ViewportNode::new(camera),
                Node {
                    display: Display::None,
                    width: percent(100),
                    min_height: px(0),
                    flex_grow: 1.,
                    ..default()
                },
            ));
        });
}

// Normalize only this presentation instance. Authored/source transforms stay intact.
fn model_transform(bounds: [[f64; 3]; 2]) -> Transform {
    let min = Vec3::from_array(bounds[0].map(|v| v as f32));
    let max = Vec3::from_array(bounds[1].map(|v| v as f32));
    let scale = 1. / ((max - min).length() / 2.).max(0.001);
    Transform::from_translation(-(min + max) * (0.5 * scale)).with_scale(Vec3::splat(scale))
}

fn isolate(event: On<WorldInstanceReady>, mut commands: Commands, children: Query<&Children>) {
    // The root stays hidden until every imported descendant has its layer assigned.
    for entity in std::iter::once(event.entity).chain(children.iter_descendants(event.entity)) {
        commands
            .entity(entity)
            .insert((RenderLayers::layer(LAYER), Pickable::IGNORE));
    }
    commands
        .entity(event.entity)
        .insert((Ready, Visibility::Inherited));
}

#[allow(clippy::too_many_arguments)]
fn sync(
    mut commands: Commands,
    session: Res<Session>,
    host: Option<Res<bevy_sprinkles_editor::hosting::AuthoringHost>>,
    library: Res<scenery_library::Library>,
    server: Res<AssetServer>,
    mut preview: ResMut<Preview>,
    roots: Query<(&WorldAssetRoot, Has<Ready>)>,
    mut cameras: Query<&mut Camera, With<PreviewCamera>>,
    mut images: Query<&mut Node, With<PreviewImage>>,
    mut labels: Query<&mut Text, With<PreviewLabel>>,
) {
    let selected =
        (session.ui.assets_open && session.play.is_none() && host.is_none_or(|h| !h.active))
            .then_some(session.assets.selected)
            .flatten()
            .filter(|i| library.assets.values().nth(*i).is_some());
    if preview.root.map(|r| r.0) != selected {
        if let Some((_, root)) = preview.root.take() {
            commands.entity(root).despawn();
        }
        if let Some(index) = selected {
            let entry = &library.assets.values().nth(index).unwrap().entry;
            let root = commands
                .spawn((
                    WorldAssetRoot(server.load(format!("scenery://{}#Scene0", entry.file))),
                    model_transform(entry.bounds),
                    Visibility::Hidden,
                    RenderLayers::layer(LAYER),
                    Pickable::IGNORE,
                ))
                .observe(isolate)
                .id();
            preview.root = Some((index, root));
        }
    }
    let root = preview.root.and_then(|(_, e)| roots.get(e).ok());
    let failed = root.is_some_and(|(root, _)| {
        matches!(
            server.get_load_state(root.0.id()),
            Some(LoadState::Failed(_))
        ) || matches!(
            server.get_recursive_dependency_load_state(root.0.id()),
            Some(RecursiveDependencyLoadState::Failed(_))
        )
    });
    let ready = root.is_some_and(|(_, ready)| ready) && !failed;
    if let Ok(mut camera) = cameras.get_mut(preview.camera) {
        camera.is_active = ready;
    }
    for mut node in &mut images {
        node.display = if ready { Display::Flex } else { Display::None };
    }
    for mut label in &mut labels {
        label.0 = if failed {
            "Preview unavailable"
        } else if ready {
            "Model preview"
        } else if selected.is_some() {
            "Loading preview…"
        } else {
            "Select a model to preview"
        }
        .into();
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Report<'w, 's> {
    preview: Res<'w, Preview>,
    cameras: Query<'w, 's, &'static Camera, With<PreviewCamera>>,
    nodes: Query<
        'w,
        's,
        (&'static ComputedNode, &'static bevy::ui::UiGlobalTransform),
        With<PreviewImage>,
    >,
}
impl Report<'_, '_> {
    pub fn snapshot(&self) -> serde_json::Value {
        let node = self.nodes.iter().next().map(|(n, t)| {
            serde_json::json!({
                "centre":(t.affine().translation*n.inverse_scale_factor()).to_array(),
                "size":(n.size()*n.inverse_scale_factor()).to_array()
            })
        });
        serde_json::json!({"selected":self.preview.root.map(|r|r.0),
            "active":self.cameras.get(self.preview.camera).is_ok_and(|c|c.is_active), "node":node})
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        // Must also run while the particle workspace is active to release the preview.
        .add_systems(Update, sync.after(particle_workspace::ZoneAuthoring));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_fit_without_changing_authored_coordinates() {
        for bounds in [
            [[-4., -2., -1.], [8., 30., 2.]],
            [[0., 0., 0.], [0., 0., 0.]],
            [[128., -128., 128.], [128., -128., 128.]],
        ] {
            let transform = model_transform(bounds);
            for corner in 0..8 {
                let point =
                    Vec3::from_array(std::array::from_fn(|i| bounds[(corner >> i) & 1][i] as f32));
                assert!(transform.transform_point(point).length() <= 1.0001);
            }
        }
    }
}
