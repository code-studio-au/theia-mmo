use crate::movement::{LocalPawn, START};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    text::FontSize,
};

#[derive(Resource)]
pub struct PawnAssets {
    pub mesh: Handle<Mesh>,
    pub remote: Handle<StandardMaterial>,
}
#[derive(Component)]
pub struct NetworkHud;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    network: Option<Res<crate::network::Network>>,
    references: Option<Res<crate::reference_scene::Config>>,
    terrain: Option<Res<crate::terrain_scene::Config>>,
) {
    let grass = materials.add(Color::srgb(0.16, 0.35, 0.17));
    let road = materials.add(Color::srgb(0.48, 0.40, 0.29));
    let stone = materials.add(Color::srgb(0.45, 0.49, 0.54));
    let wood = materials.add(Color::srgb(0.42, 0.22, 0.10));
    if terrain.is_none() {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(40., 40.))),
            MeshMaterial3d(grass),
        ));
    }
    let unit_box = meshes.add(Cuboid::new(1., 1., 1.));
    for (index, prop) in theia_network::world::PROPS.into_iter().enumerate() {
        if terrain.is_some() || (references.is_some() && (2..=4).contains(&index)) {
            continue;
        }
        let material = match prop.surface {
            theia_network::world::Surface::Road => road.clone(),
            theia_network::world::Surface::Stone => stone.clone(),
            theia_network::world::Surface::Wood => wood.clone(),
        };
        commands.spawn((
            Mesh3d(unit_box.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::from_array(prop.center))
                .with_scale(Vec3::from_array(prop.size)),
        ));
    }
    let capsule = meshes.add(Capsule3d::new(
        theia_network::world::PAWN_RADIUS,
        theia_network::world::PAWN_HEIGHT - 2. * theia_network::world::PAWN_RADIUS,
    ));
    commands.insert_resource(PawnAssets {
        mesh: capsule.clone(),
        remote: materials.add(Color::srgb(0.95, 0.25, 0.12)),
    });
    let mut pawn = commands.spawn((
        LocalPawn,
        crate::movement::Motion::default(),
        Transform::from_translation(START),
        if network.is_some() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        },
    ));
    if references.is_none() {
        pawn.insert((
            Mesh3d(capsule),
            MeshMaterial3d(materials.add(Color::srgb(0.12, 0.42, 0.95))),
        ));
        pawn.with_child((
            Mesh3d(unit_box),
            MeshMaterial3d(materials.add(Color::srgb(1., 0.65, 0.12))),
            Transform::from_xyz(0., 0.25, -0.35).with_scale(Vec3::new(0.3, 0.18, 0.18)),
        ));
    }
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
        if references.is_some() || terrain.is_some() {
            Tonemapping::Reinhard
        } else {
            Tonemapping::None
        },
        DebandDither::Disabled,
        Transform::from_xyz(0., 7.9, 12.).looking_at(START, Vec3::Y),
    ));
    commands.spawn((
        Node { position_type: PositionType::Absolute, left: px(12), top: px(12),
            padding: UiRect::all(px(12)), ..default() },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
    )).with_child((
        NetworkHud,
        Text::new(if network.is_some() { "THEIA\nConnecting to local server...\nEsc: quit" } else { "THEIA\nLocal movement sandbox\nWASD / arrows: move | Shift: run\nR: reset | Esc: quit\nHold RMB: orbit | Wheel: zoom | Home: camera reset" }),
        TextFont { font_size: FontSize::Px(18.), ..default() },
        TextColor(Color::WHITE),
    ));
}
