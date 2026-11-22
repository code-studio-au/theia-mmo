//! Opt-in, input-free native rendering check on a disposable document copy.
use super::*;
use bevy::ecs::system::RunSystemOnce;
#[derive(Resource)]
struct Run {
    output: PathBuf,
    stage: usize,
    frames: usize,
    source: Option<Zone>,
}
pub(super) fn install(app: &mut App) {
    if let Some(output) = std::env::var_os("THEIA_AREA_BRUSH_QUALIFY") {
        app.insert_resource(Run {
            output: output.into(),
            stage: 0,
            frames: 0,
            source: None,
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
fn probe_area(
    mut rays: bevy::picking::mesh_picking::ray_cast::MeshRayCast,
    tiles: Query<(), With<Tile>>,
    session: Res<Session>,
    view: Res<View>,
) {
    let zone = session
        .stroke
        .as_ref()
        .map_or(session.editor.zone(), |s| s.draft.zone());
    let centre = zone.global(view.target.to_array()).unwrap();
    let filter = |e| tiles.contains(e);
    let settings =
        bevy::picking::mesh_picking::ray_cast::MeshRayCastSettings::default().with_filter(&filter);
    for dx in [-128., 0., 128.] {
        for dz in [-128., 0., 128.] {
            let p = Vec3::from_array(
                zone.local(
                    zone.surface_position([centre[0] + dx, centre[2] + dz])
                        .unwrap(),
                )
                .unwrap(),
            );
            let ray = Ray3d::new(p + Vec3::Y * 100., Dir3::NEG_Y);
            let hits = rays.cast_ray(ray, &settings);
            assert!(
                (ray.get_point(hits.first().expect("Rendered area sample").1.distance) - p)
                    .length()
                    < 0.002
            );
        }
    }
}
fn drive(world: &mut World) {
    let Some(mut run) = world.remove_resource::<Run>() else {
        return;
    };
    run.frames += 1;
    if run.stage == 0 && run.frames == 1 {
        let mut query = world.query::<&mut Window>();
        for mut window in query.iter_mut(world) {
            window.mode = bevy::window::WindowMode::Windowed;
            window.resolution.set(1600., 1000.);
        }
    }
    if run.frames < 60
        || world.resource::<Scene>().tiles.len()
            != world
                .resource::<Session>()
                .editor
                .zone()
                .imported_terrain
                .as_ref()
                .unwrap()
                .tile_count()
    {
        world.insert_resource(run);
        return;
    }
    std::fs::create_dir_all(&run.output).unwrap();
    match run.stage {
        0 => {
            let mut s = world.resource_mut::<Session>();
            run.source = Some(s.editor.zone().clone());
            s.set_mode(EditorMode::Sculpt);
            s.ui.explorer_collapsed = true;
            s.brush_tool = terrain::Tool::FlattenArea;
            s.brush_radius = 160.;
            s.flatten_edge = 40.;
            let z = s.editor.zone();
            let xz = [z.spawn[0], z.spawn[2]];
            let p = Vec3::from_array(z.local(z.surface_position(xz).unwrap()).unwrap());
            terrain::begin_stroke(&mut s);
            let mut stroke = s.stroke.take().unwrap();
            terrain::stamp_stroke(&mut s, &mut stroke, p);
            assert_eq!(stroke.stamps(), 1);
            s.stroke = Some(stroke);
            s.brush_point = Some(p);
            let mut v = world.resource_mut::<View>();
            v.target = p;
            v.distance = 800.;
            v.pitch = 1.1;
        }
        1 => {
            world.run_system_once(probe_area).unwrap();
            capture(world, &run.output, "flat-preview");
        }
        2 => {
            let mut s = world.resource_mut::<Session>();
            let stroke = s.stroke.take().unwrap();
            terrain::commit_stroke(&mut s, stroke);
            assert_eq!(s.editor.history(), (1, 0));
            s.editor
                .execute(Command::Undo {
                    expected_revision: 1,
                })
                .unwrap();
            assert_eq!(s.editor.zone(), run.source.as_ref().unwrap());
            s.editor
                .execute(Command::Redo {
                    expected_revision: 2,
                })
                .unwrap();
            s.flatten_noise = true;
            s.noise_height = 3.;
            s.noise_width = 24.;
            let z = s.editor.zone();
            let xz = [z.spawn[0], z.spawn[2]];
            let p = Vec3::from_array(z.local(z.surface_position(xz).unwrap()).unwrap());
            terrain::begin_stroke(&mut s);
            let mut stroke = s.stroke.take().unwrap();
            terrain::stamp_stroke(&mut s, &mut stroke, p);
            assert_eq!(stroke.stamps(), 1);
            terrain::commit_stroke(&mut s, stroke);
            let saved = run.output.join("edited.zone.json");
            s.editor.zone().save(&saved).unwrap();
            let loaded = Zone::load(&saved).unwrap();
            assert_eq!(&loaded, s.editor.zone());
            for ((_, a), (_, b)) in loaded
                .imported_terrain
                .as_ref()
                .unwrap()
                .tiles()
                .zip(s.editor.zone().imported_terrain.as_ref().unwrap().tiles())
            {
                assert_eq!(a, b);
            }
            s.brush_point = Some(p);
        }
        3 => {
            assert_eq!(
                world
                    .resource::<Scene>()
                    .tiles
                    .iter()
                    .filter(|t| t.cells == 64)
                    .count(),
                25
            );
            world.run_system_once(probe_area).unwrap();
            capture(world, &run.output, "noise-controls");
        }
        4 => {
            let mut s = world.resource_mut::<Session>();
            s.brush_tool = terrain::Tool::Smooth;
            s.sculpt_blend = 1.;
            let z = s.editor.zone();
            let p = Vec3::from_array(
                z.local(z.surface_position([z.spawn[0], z.spawn[2]]).unwrap())
                    .unwrap(),
            );
            terrain::begin_stroke(&mut s);
            let mut stroke = s.stroke.take().unwrap();
            terrain::stamp_stroke(&mut s, &mut stroke, p);
            assert_eq!(stroke.stamps(), 1);
            terrain::commit_stroke(&mut s, stroke);
            s.brush_point = Some(p);
        }
        5 => {
            world.run_system_once(probe_area).unwrap();
            capture(world, &run.output, "large-smooth");
            let report = serde_json::json!({"passed":true,"radius_m":160,"edge_m":40,"noise_height_m":3,"noise_width_m":24,"preview_commit_undo_redo":true,"save_reload_sample_equality":true,"rendered_rays":true,"scope":"App-owned native rendering, no OS input injection"});
            std::fs::write(
                run.output.join("qualification.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
        }
        _ => {
            world.write_message(bevy::app::AppExit::Success);
        }
    }
    run.stage += 1;
    run.frames = 0;
    world.insert_resource(run);
}
pub(crate) fn active() -> bool {
    std::env::var_os("THEIA_AREA_BRUSH_QUALIFY").is_some()
}
