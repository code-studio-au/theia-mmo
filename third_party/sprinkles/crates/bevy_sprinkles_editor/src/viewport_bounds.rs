//! Bevy 0.20 adapter: standard line meshes avoid native gizmo upload hazards.
use crate::hosting::AuthoringAppExt;
use crate::{
    state::{EditorState, Inspectable},
    ui::tokens::PRIMARY_COLOR,
};
use bevy::{
    asset::RenderAssetUsages, camera::primitives::Aabb, mesh::PrimitiveTopology, prelude::*,
};
use bevy_sprinkles::prelude::*;

#[derive(Component)]
pub(crate) struct ShowBounds;

#[derive(Component)]
struct Overlay(Entity);

#[derive(Resource)]
struct Geometry {
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    bounds: Handle<StandardMaterial>,
    enabled: Handle<StandardMaterial>,
    disabled: Handle<StandardMaterial>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_authoring_systems(sync.after(crate::viewport::sync_inspected_emitter_aabb));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let corners: Vec<Vec3> = (0..8)
        .map(|i| {
            Vec3::new(
                if i & 1 == 0 { -0.5 } else { 0.5 },
                if i & 2 == 0 { -0.5 } else { 0.5 },
                if i & 4 == 0 { -0.5 } else { 0.5 },
            )
        })
        .collect();
    let mut cube = Vec::new();
    for i in 0..8 {
        for bit in [1, 2, 4] {
            if i & bit == 0 {
                cube.extend([corners[i], corners[i | bit]]);
            }
        }
    }
    let mut sphere = Vec::new();
    for axis in 0..3 {
        for i in 0..32 {
            for step in [i, i + 1] {
                let angle = step as f32 * std::f32::consts::TAU / 32.;
                let mut point = Vec3::ZERO;
                point[(axis + 1) % 3] = angle.cos();
                point[(axis + 2) % 3] = angle.sin();
                sphere.push(point);
            }
        }
    }
    let line_mesh = |points: Vec<Vec3>| {
        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    };
    let material = |color: Color| StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };
    commands.insert_resource(Geometry {
        cube: meshes.add(line_mesh(cube)),
        sphere: meshes.add(line_mesh(sphere)),
        bounds: materials.add(material(Color::WHITE.with_alpha(0.1))),
        enabled: materials.add(material(PRIMARY_COLOR.into())),
        disabled: materials.add(material(bevy::color::palettes::tailwind::ZINC_200.into())),
    });
}

fn sync(
    mut commands: Commands,
    geometry: Res<Geometry>,
    state: Res<EditorState>,
    bounds: Query<(Entity, &Aabb, &GlobalTransform), With<ShowBounds>>,
    colliders: Query<(Entity, &ParticlesCollider3D, &ColliderEntity, &Transform), Without<Overlay>>,
    mut overlays: Query<(
        Entity,
        &Overlay,
        &mut Transform,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let mut wanted = Vec::new();
    for (owner, bounds, global) in &bounds {
        let local = Transform::from_translation(bounds.center.into())
            .with_scale(Vec3::from(bounds.half_extents) * 2.);
        wanted.push((
            owner,
            global.mul_transform(local).compute_transform(),
            geometry.cube.clone(),
            geometry.bounds.clone(),
        ));
    }
    let selected = state
        .inspecting
        .as_ref()
        .filter(|i| i.kind == Inspectable::Collider)
        .map(|i| i.index as usize);
    for (owner, collider, entity, transform) in &colliders {
        if selected != Some(entity.collider_index) {
            continue;
        }
        let (pose, mesh) = match collider.shape {
            ParticlesColliderShape3D::Box { size } => (
                Transform {
                    scale: transform.scale * size,
                    ..*transform
                },
                geometry.cube.clone(),
            ),
            ParticlesColliderShape3D::Sphere { radius } => (
                Transform::from_translation(transform.translation)
                    .with_scale(Vec3::splat(radius * transform.scale.max_element())),
                geometry.sphere.clone(),
            ),
        };
        wanted.push((
            owner,
            pose,
            mesh,
            if collider.enabled {
                geometry.enabled.clone()
            } else {
                geometry.disabled.clone()
            },
        ));
    }
    for (entity, overlay, mut pose, mut mesh, mut material) in &mut overlays {
        if let Some(index) = wanted.iter().position(|(owner, ..)| *owner == overlay.0) {
            let (_, next_pose, next_mesh, next_material) = wanted.swap_remove(index);
            pose.set_if_neq(next_pose);
            mesh.set_if_neq(Mesh3d(next_mesh));
            material.set_if_neq(MeshMaterial3d(next_material));
        } else {
            commands.entity(entity).despawn();
        }
    }
    for (owner, pose, mesh, material) in wanted {
        commands.spawn((
            Overlay(owner),
            pose,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Inspecting;

    #[test]
    fn overlays_follow_bounds_and_selection_without_growing_assets_or_outliving_sources() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<EditorState>()
            .add_systems(Startup, setup)
            .add_authoring_systems(sync);
        let source = app
            .world_mut()
            .spawn((
                ShowBounds,
                Aabb {
                    center: Vec3::new(1., 2., 3.).into(),
                    half_extents: Vec3::new(2., 3., 4.).into(),
                },
                GlobalTransform::from(Transform::from_xyz(10., 20., 30.)),
            ))
            .id();
        for step in 0..40 {
            app.world_mut().get_mut::<Aabb>(source).unwrap().center.x = 1. + step as f32;
            app.update();
            let mut query = app.world_mut().query::<(&Overlay, &Transform)>();
            let (overlay, transform) = query.single(app.world()).unwrap();
            assert_eq!(overlay.0, source);
            assert_eq!(
                transform.translation,
                Vec3::new(11. + step as f32, 22., 33.)
            );
            assert_eq!(transform.scale, Vec3::new(4., 6., 8.));
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 2);
            assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 3);
        }
        app.world_mut().entity_mut(source).remove::<ShowBounds>();
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&Overlay>()
                .iter(app.world())
                .count(),
            0
        );
        let collider = app
            .world_mut()
            .spawn((
                ParticlesCollider3D {
                    enabled: true,
                    shape: ParticlesColliderShape3D::Sphere { radius: 2. },
                },
                ColliderEntity {
                    parent_system: source,
                    collider_index: 0,
                },
                Transform::from_xyz(4., 5., 6.).with_scale(Vec3::new(1., 3., 2.)),
            ))
            .id();
        app.world_mut().resource_mut::<EditorState>().inspecting = Some(Inspecting {
            kind: Inspectable::Collider,
            index: 0,
        });
        app.update();
        let mut query = app.world_mut().query::<(
            &Transform,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &Overlay,
        )>();
        let (pose, mesh, material, overlay) = query.single(app.world()).unwrap();
        let geometry = app.world().resource::<Geometry>();
        assert_eq!(overlay.0, collider);
        assert_eq!(pose.translation, Vec3::new(4., 5., 6.));
        assert_eq!(pose.scale, Vec3::splat(6.));
        assert_eq!(mesh.0, geometry.sphere);
        assert_eq!(material.0, geometry.enabled);
        app.world_mut()
            .get_mut::<ParticlesCollider3D>(collider)
            .unwrap()
            .enabled = false;
        app.update();
        let (_, _, material, _) = query.single(app.world()).unwrap();
        assert_eq!(material.0, app.world().resource::<Geometry>().disabled);
        app.world_mut().entity_mut(collider).despawn();
        app.update();
        assert_eq!(query.iter(app.world()).count(), 0);
    }
}
