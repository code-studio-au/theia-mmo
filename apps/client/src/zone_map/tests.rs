use super::*;

#[test]
fn whole_zone_and_local_map_project_facing_and_right_side_consistently() {
    let d: Data = serde_json::from_slice(MAP_JSON).unwrap();
    let centre = [2048., 1810.];
    let size = Vec2::new(1024., 768.);
    for yaw in [
        0.,
        0.4,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        4.7,
    ] {
        let rotation = Quat::from_rotation_y(yaw);
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;
        let project = |v: Vec3| {
            ((d.project([
                centre[0] + f64::from(v.x) * 20.,
                centre[1] + f64::from(v.z) * 20.,
            ])
            .unwrap()
                - d.project(centre).unwrap())
                * size)
                .normalize()
        };
        let arrow = crate::minimap::heading_rotation(yaw, d.north_positive_z) * Vec2::NEG_Y;
        assert!(project(forward).distance(arrow) < 0.00002);
        assert!(
            project(right).distance(Rot2::radians(std::f32::consts::FRAC_PI_2) * arrow) < 0.00002
        );
    }
    // Geographic farm labels follow the game compass, without moving source sites.
    assert!(d.landmarks[4].name.starts_with("Southeast"));
    assert!(d.project(d.landmarks[4].xz).unwrap().x > 0.5);
    assert!(d.landmarks[5].name.starts_with("Southwest"));
    assert!(d.project(d.landmarks[5].xz).unwrap().x < 0.5);
}

#[test]
fn overview_releases_mouse_capture_and_prevents_orbit_and_zoom() {
    use bevy::{
        input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
        window::{CursorGrabMode, CursorOptions},
    };
    let mut app = App::new();
    app.init_resource::<Ui>()
        .init_resource::<crate::camera::Orbit>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseMotion>()
        .init_resource::<AccumulatedMouseScroll>()
        .add_systems(Update, crate::camera::controls);
    let window = app
        .world_mut()
        .spawn((
            Window {
                focused: true,
                ..default()
            },
            CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            },
        ))
        .id();
    app.world_mut().resource_mut::<Ui>().open = true;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.world_mut()
        .resource_mut::<AccumulatedMouseMotion>()
        .delta = Vec2::splat(30.);
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::new(0., 4.);
    let before = app.world().resource::<crate::camera::Orbit>().angles();
    let distance = app.world().resource::<crate::camera::Orbit>().distance();
    app.update();
    assert_eq!(
        before,
        app.world().resource::<crate::camera::Orbit>().angles()
    );
    assert_eq!(
        distance,
        app.world().resource::<crate::camera::Orbit>().distance()
    );
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert!(cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    app.world_mut().resource_mut::<Ui>().open = false;
    app.update();
    assert_ne!(
        before,
        app.world().resource::<crate::camera::Orbit>().angles()
    );
}

#[test]
fn landmark_controls_require_open_focused_map_and_validate_the_index() {
    let mut app = App::new();
    app.init_resource::<Ui>()
        .insert_resource(serde_json::from_slice::<Data>(MAP_JSON).unwrap())
        .add_observer(activate);
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            ..default()
        })
        .id();
    let button = app.world_mut().spawn(Control::Landmark(6)).id();
    app.world_mut().trigger(Activate { entity: button });
    assert_eq!(app.world().resource::<Ui>().selected, 0);
    app.world_mut().resource_mut::<Ui>().open = true;
    app.world_mut().trigger(Activate { entity: button });
    assert_eq!(app.world().resource::<Ui>().selected, 6);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.world_mut()
        .entity_mut(button)
        .insert(Control::Landmark(0));
    app.world_mut().trigger(Activate { entity: button });
    assert_eq!(app.world().resource::<Ui>().selected, 6);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    app.world_mut()
        .entity_mut(button)
        .insert(Control::Landmark(99));
    app.world_mut().trigger(Activate { entity: button });
    assert_eq!(app.world().resource::<Ui>().selected, 6);
}

#[test]
fn cartography_contract_and_positions_match_the_zone_and_source_north() {
    let mut d: Data = serde_json::from_slice(MAP_JSON).unwrap();
    assert!(d.valid());
    assert_eq!(d.project([4096., 3072.]), Some(Vec2::ZERO));
    assert_eq!(d.project([0., 0.]), Some(Vec2::ONE));
    assert_eq!(d.project([2048., 1536.]), Some(Vec2::splat(0.5)));
    assert!(d.project([f64::NAN, 0.]).is_none());
    assert!(d.project([4096.1, 0.]).is_none());
    let village = d.project(d.landmarks[0].xz).unwrap();
    let caves = d.project(d.landmarks[6].xz).unwrap();
    assert!(caves.y < village.y && caves.x < village.x);
    let offset = 1e11;
    for bound in &mut d.bounds {
        for v in bound {
            *v += offset;
        }
    }
    assert_eq!(
        d.project([offset + 2048., offset + 1536.]),
        Some(Vec2::splat(0.5))
    );
    let image = Image::from_buffer(
        MAP_PNG,
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::default(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .unwrap();
    assert_eq!((image.width(), image.height()), (1024, 768));
}

#[test]
fn map_fits_small_and_wide_windows_without_stretching_terrain() {
    for size in [
        Vec2::new(640., 480.),
        Vec2::new(1280., 720.),
        Vec2::new(1720., 700.),
        Vec2::new(3440., 1400.),
    ] {
        let (map, panel) = layout(size, 4. / 3.);
        assert!(panel.cmple(size).all(), "{size:?}: {panel:?}");
        assert!((map.x / map.y - 4. / 3.).abs() < 0.0001);
    }
}

#[test]
fn focused_map_keys_toggle_and_consume_only_the_owned_escape() {
    let mut app = App::new();
    app.init_resource::<Ui>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, keyboard);
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            ..default()
        })
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyM);
    app.update();
    assert!(app.world().resource::<Ui>().open);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert!(!app.world().resource::<Ui>().open);
    assert!(app.world().resource::<Ui>().closed_this_frame);
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Escape)
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyM);
    app.update();
    assert!(!app.world().resource::<Ui>().open);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Escape)
    );
}

#[test]
fn regional_portrait_map_fits_and_preserves_aspect() {
    for size in [
        Vec2::new(640., 480.),
        Vec2::new(1280., 720.),
        Vec2::new(3440., 1440.),
    ] {
        let aspect = 7296. / 8192.;
        let (map, panel) = layout(size, aspect);
        assert!(panel.cmple(size).all());
        assert!((map.x / map.y - aspect).abs() < 0.0001);
    }
}
#[test]
fn invalid_external_map_is_rejected_and_cartography_samples_known_orientation() {
    assert!(decode_map(b"{}", MAP_PNG).is_err());
    assert!(decode_map(MAP_JSON, b"not a png").is_err());
    let (data, image) = decode_map(MAP_JSON, MAP_PNG).unwrap();
    let mut world = World::new();
    world.insert_resource(data);
    world.insert_resource(MapImage(image));
    let position = theia_core::coordinates::GlobalPosition::new([2048., 0., 1536.]).unwrap();
    assert!(cartography_pixel(&world, position).is_some());
    assert!(
        cartography_pixel(
            &world,
            theia_core::coordinates::GlobalPosition::new([5000., 0., 0.]).unwrap()
        )
        .is_none()
    );
}
