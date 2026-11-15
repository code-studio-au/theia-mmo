//! Private ground pickups, visible only while the authority offers a claim.
use super::*;
#[derive(Component)]
pub(crate) struct Pickup(pub theia_core::rewards::ClaimId);
#[derive(Component)]
struct GroundItem(theia_core::content::ContentId);
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let camera = world
        .query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>()
        .iter(world)
        .next()
        .map(|(c, t)| (c.clone(), *t));
    let values: Vec<_> = world.query::<(&GroundItem, &Transform, &Visibility, Option<&Pickup>)>()
        .iter(world).map(|(item, transform, visibility, pickup)| serde_json::json!({
            "id":item.0.as_str(), "available":pickup.is_some(), "visible":*visibility != Visibility::Hidden,
            "screen":camera.as_ref().and_then(|(c,t)| c.world_to_viewport(t, transform.translation).ok()).map(|p|p.to_array())
        })).collect();
    serde_json::json!(values)
}
pub(super) fn install(app: &mut App) {
    app.add_systems(Startup, setup.after(super::setup))
        .add_systems(Update, present.after(crate::network::present));
}
fn setup(
    mut commands: Commands,
    config: Res<Config>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(rewards) = config.0.rewards() else {
        return;
    };
    for definition in rewards
        .catalog()
        .definitions()
        .filter(|d| d.ground().is_some())
    {
        commands.spawn((
            GroundItem(definition.id().clone()),
            Mesh3d(meshes.add(Cuboid::new(0.55, 0.35, 0.4))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.65, 0.42, 0.12),
                metallic: 0.5,
                perceptual_roughness: 0.35,
                ..default()
            })),
            Transform::default(),
            Visibility::Hidden,
            AccessibleLabel::new(format!("Collect {}", definition.name())),
        ));
    }
}
fn present(
    mut commands: Commands,
    config: Res<Config>,
    network: Res<Network>,
    frame: Res<crate::floating_origin::Frame>,
    mut items: Query<(
        Entity,
        &GroundItem,
        Option<&Pickup>,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    let state = network.state();
    for (entity, item, previous, mut transform, mut visibility) in &mut items {
        let available = (|| {
            if !network.connected() {
                return None;
            }
            let rewards = config.0.rewards()?;
            let ground = rewards.catalog().definition(&item.0)?.ground()?;
            let chunk = ground.position.chunk(128)?;
            if !state
                .chunk_report
                .as_ref()?
                .chunks
                .contains(&[chunk.x, chunk.z])
            {
                return None;
            }
            let view = state
                .rewards
                .as_ref()
                .filter(|v| v.profile_hash == rewards.hash())?;
            let claim = view.available.iter().find(|r| r.reward == item.0)?.claim;
            let position =
                Vec3::from_array(frame.render.to_local(ground.position)?.metres()) + Vec3::Y * 0.2;
            Some((claim, position))
        })();
        *visibility = if available.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some((claim, position)) = available {
            transform.translation = position;
            if previous.is_none_or(|p| p.0 != claim) {
                commands.entity(entity).insert(Pickup(claim));
            }
        } else if previous.is_some() {
            commands.entity(entity).remove::<Pickup>();
        }
    }
}
