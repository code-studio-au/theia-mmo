use crate::movement::{LocalPawn, START};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    text::FontSize,
};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let grass = materials.add(Color::srgb(0.16, 0.35, 0.17));
    let road = materials.add(Color::srgb(0.48, 0.40, 0.29));
    let stone = materials.add(Color::srgb(0.45, 0.49, 0.54));
    let wood = materials.add(Color::srgb(0.42, 0.22, 0.10));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40., 40.))),
        MeshMaterial3d(grass),
    ));
    let unit_box = meshes.add(Cuboid::new(1., 1., 1.));
    for (position, size, material) in [
        (
            Vec3::new(0., 0.015, 0.),
            Vec3::new(3., 0.03, 40.),
            road.clone(),
        ),
        (Vec3::new(0., 0.016, -5.), Vec3::new(40., 0.03, 3.), road),
        (
            Vec3::new(-7., 1.5, -7.),
            Vec3::new(4., 3., 4.),
            stone.clone(),
        ),
        (
            Vec3::new(-7., 3.2, -7.),
            Vec3::new(4.6, 0.4, 4.6),
            wood.clone(),
        ),
        (Vec3::new(-7., 0.7, -4.95), Vec3::new(0.8, 1.4, 0.12), wood),
        (
            Vec3::new(8., 0.6, 4.),
            Vec3::new(1.8, 1.2, 1.3),
            stone.clone(),
        ),
        (Vec3::new(10., 0.4, 6.), Vec3::new(1.2, 0.8, 1.8), stone),
    ] {
        commands.spawn((
            Mesh3d(unit_box.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(position).with_scale(size),
        ));
    }
    commands
        .spawn((
            LocalPawn,
            Mesh3d(meshes.add(Capsule3d::new(0.35, 1.1))),
            MeshMaterial3d(materials.add(Color::srgb(0.12, 0.42, 0.95))),
            Transform::from_translation(START),
        ))
        .with_child((
            Mesh3d(unit_box),
            MeshMaterial3d(materials.add(Color::srgb(1., 0.65, 0.12))),
            Transform::from_xyz(0., 0.25, -0.35).with_scale(Vec3::new(0.3, 0.18, 0.18)),
        ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(8., 12., 6.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        DebandDither::Disabled,
        Transform::from_xyz(0., 7.9, 12.).looking_at(START, Vec3::Y),
    ));
    commands.spawn((
        Node { position_type: PositionType::Absolute, left: px(12), top: px(12),
            padding: UiRect::all(px(12)), ..default() },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
    )).with_child((
        Text::new("THEIA\nLocal movement sandbox\nWASD / arrows: move | Shift: run\nR: reset | Esc: quit"),
        TextFont { font_size: FontSize::Px(18.), ..default() },
        TextColor(Color::WHITE),
    ));
}
