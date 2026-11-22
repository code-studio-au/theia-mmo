//! App-owned native fixture. Mutates only its explicit disposable output document.
use super::*;
use bevy::{
    ecs::system::RunSystemOnce,
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
};
use std::path::PathBuf;
#[derive(Resource)]
struct Run {
    output: PathBuf,
    stage: usize,
    frames: usize,
    reports: Vec<serde_json::Value>,
}
pub(super) fn install(app: &mut App) {
    if let Some(output) = std::env::var_os("THEIA_IMPORTED_EDITOR_QUALIFY") {
        app.insert_resource(Run {
            output: output.into(),
            stage: 0,
            frames: 0,
            reports: vec![],
        })
        .add_systems(
            PostUpdate,
            drive.after(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
        );
    }
}
fn capture(world: &mut World, output: &std::path::Path, name: &str) {
    world
        .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(
            output.join(format!("{name}.png")),
        ));
}
fn probe(
    mut raycast: MeshRayCast,
    tiles: Query<(), With<Tile>>,
    session: Res<Session>,
    view: Res<View>,
) -> [f64; 3] {
    let zone = session.editor.zone();
    let global = zone.global(view.target.to_array()).unwrap();
    let expected = zone.surface_position([global[0], global[2]]).unwrap();
    let local = Vec3::from_array(zone.local(expected).unwrap());
    let ray = Ray3d::new(local + Vec3::Y * 100., Dir3::NEG_Y);
    let filter = |e| tiles.contains(e);
    let settings = MeshRayCastSettings::default().with_filter(&filter);
    let hits = raycast.cast_ray(ray, &settings);
    let (_, hit) = hits
        .first()
        .expect("Imported rendered terrain must be pickable");
    assert!(
        (ray.get_point(hit.distance) - local).length() < 0.002,
        "Rendered pick must match 1 m surface"
    );
    expected
}
fn drive(world: &mut World) {
    let Some(mut run) = world.remove_resource::<Run>() else {
        return;
    };
    run.frames += 1;
    let count = world
        .resource::<Session>()
        .editor
        .zone()
        .imported_terrain
        .as_ref()
        .unwrap()
        .tile_count();
    if world.resource::<Scene>().tiles.len() != count || run.frames < 60 {
        world.insert_resource(run);
        return;
    }
    std::fs::create_dir_all(&run.output).unwrap();
    match run.stage {
        0 => {
            assert!(world.resource::<Session>().editor.zone().objects.is_empty());
            capture(world, &run.output, "overview");
        }
        1 | 4 | 7 => {
            let xz = match run.stage {
                1 => [2049.25, 1811.625],
                4 => [2561.75, 2540.5],
                _ => [4000.25, 64.5],
            };
            let zone = world.resource::<Session>().editor.zone();
            let ground = zone.surface_position(xz).unwrap();
            let local = Vec3::from_array(zone.local(ground).unwrap());
            let mut view = world.resource_mut::<View>();
            view.target = local;
            view.distance = 90.;
            view.pitch = 0.8;
            view.yaw = std::f32::consts::PI;
        }
        2 | 5 | 8 => {
            let ground = world.run_system_once(probe).unwrap();
            let mut session = world.resource_mut::<Session>();
            let before = session.editor.zone().objects.len();
            session.place_primitive(Primitive::Box, ground);
            assert_eq!(
                session.editor.zone().objects.len(),
                before + 1,
                "{}",
                session.status
            );
            let object = session.editor.zone().objects.last().unwrap();
            let height = session
                .editor
                .zone()
                .surface_position([object.pose.position[0], object.pose.position[2]])
                .unwrap()[1];
            assert!((object.pose.position[1] - object.pose.size[1] * 0.5 - height).abs() < 0.0001);
            run.reports
                .push(serde_json::json!({"ray_ground":ground,"object":object}));
        }
        3 => capture(world, &run.output, "village"),
        6 => capture(world, &run.output, "caves"),
        9 => capture(world, &run.output, "far-corner"),
        10 => {
            let mut view = world.remove_resource::<View>().unwrap();
            let mut session = world.resource_mut::<Session>();
            let before = session.editor.zone().clone();
            crate::perform(Action::Undo, &mut session, &mut view);
            assert_eq!(session.editor.zone().objects.len(), 2);
            crate::perform(Action::Redo, &mut session, &mut view);
            assert_eq!(session.editor.zone(), &before);
            // Exercise an existing object edit across the former 512 m limit.
            let id = session.editor.zone().objects[0].id;
            let mut pose = session.editor.zone().objects[0].pose.clone();
            pose.position = session
                .editor
                .zone()
                .surface_position([128., 2900.])
                .unwrap();
            pose.position[1] += pose.size[1] * 0.5;
            session.edit(vec![Edit::Transform { id, pose }]);
            assert_eq!(session.editor.zone().objects[0].pose.position[2], 2900.);
            session.assign_path(run.output.join("placements.zone.json"));
            crate::perform(Action::Save, &mut session, &mut view);
            assert!(!session.is_dirty(), "{}", session.status);
            let saved = session.editor.zone().clone();
            crate::perform(Action::Load, &mut session, &mut view);
            assert_eq!(session.editor.zone(), &saved);
            run.reports.push(serde_json::json!({"save_reload":true,"undo_redo":true,"far_transform":true,"document_bytes":std::fs::metadata(run.output.join("placements.zone.json")).unwrap().len()}));
            world.insert_resource(view);
            let data = serde_json::json!({"passed":true,"checks":run.reports,"scene":world.resource::<Scene>().summary(),"source_samples_bytes":world.resource::<Session>().editor.zone().imported_terrain.as_ref().unwrap().sample_bytes(),"scope":"Native rendered terrain picks, placement/edit/save/reload and undo/redo through app-owned actions; no OS input injection"});
            std::fs::write(
                run.output.join("qualification.json"),
                serde_json::to_vec_pretty(&data).unwrap(),
            )
            .unwrap();
        }
        _ => {
            world.write_message(bevy::app::AppExit::Success);
            world.insert_resource(run);
            return;
        }
    }
    run.stage += 1;
    run.frames = 0;
    world.insert_resource(run);
}
