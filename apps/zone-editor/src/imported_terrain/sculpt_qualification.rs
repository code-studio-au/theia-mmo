//! Explicit app-owned native check; no desktop keyboard/mouse injection.
use super::*;
use bevy::{
    ecs::system::RunSystemOnce,
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
};
#[derive(Resource)]
pub(crate) struct Run {
    output: PathBuf,
    stage: usize,
    frames: usize,
    original: Option<Zone>,
    committed: Option<Zone>,
}
pub(super) fn install(app: &mut App) {
    if let Some(output) = std::env::var_os("THEIA_IMPORTED_SCULPT_QUALIFY") {
        app.insert_resource(Run {
            output: output.into(),
            stage: 0,
            frames: 0,
            original: None,
            committed: None,
        })
        .add_systems(
            PostUpdate,
            drive.after(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
        );
    }
}
fn capture(world: &mut World, run: &Run, name: &str) {
    world
        .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(
            run.output.join(format!("{name}.png")),
        ));
}
fn probe(
    mut rays: MeshRayCast,
    tiles: Query<(), With<Tile>>,
    session: Res<Session>,
    view: Res<View>,
) {
    let zone = session
        .stroke
        .as_ref()
        .map_or(session.editor.zone(), |s| s.draft.zone());
    let global = zone.global(view.target.to_array()).unwrap();
    let local = Vec3::from_array(
        zone.local(zone.surface_position([global[0], global[2]]).unwrap())
            .unwrap(),
    );
    let filter = |e| tiles.contains(e);
    let ray = Ray3d::new(local + Vec3::Y * 100., Dir3::NEG_Y);
    let hits = rays.cast_ray(ray, &MeshRayCastSettings::default().with_filter(&filter));
    assert!(
        (ray.get_point(
            hits.first()
                .expect("Rendered edited terrain pick")
                .1
                .distance
        ) - local)
            .length()
            < 0.002
    );
}
fn stamp(session: &mut Session, tool: terrain::Tool, xz: [f64; 2]) {
    session.brush_tool = tool;
    let p = Vec3::from_array(
        session
            .editor
            .zone()
            .local(session.editor.zone().surface_position(xz).unwrap())
            .unwrap(),
    );
    session.brush_point = Some(p);
    let mut stroke = session.stroke.take().unwrap();
    terrain::stamp_stroke(session, &mut stroke, p);
    session.stroke = Some(stroke);
}
fn drive(world: &mut World) {
    let Some(mut run) = world.remove_resource::<Run>() else {
        return;
    };
    run.frames += 1;
    if run.frames < 45
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
            let session = world.resource::<Session>();
            run.original = Some(session.editor.zone().clone());
            let ground = session
                .editor
                .zone()
                .surface_position([2560., 2560.])
                .unwrap();
            let target = Vec3::from_array(session.editor.zone().local(ground).unwrap());
            let mut view = world.resource_mut::<View>();
            view.target = target;
            view.distance = 100.;
            view.pitch = 1.;
        }
        1 => {
            let mut session = world.resource_mut::<Session>();
            session.set_mode(EditorMode::Sculpt);
            session.brush_strength = 8.;
            session.brush_radius = 16.;
            terrain::begin_stroke(&mut session);
            stamp(&mut session, terrain::Tool::Raise, [2560., 2560.]);
            stamp(&mut session, terrain::Tool::Raise, [2562., 2560.]);
            assert_eq!(session.editor.zone(), run.original.as_ref().unwrap());
            assert_ne!(
                session.stroke.as_ref().unwrap().draft.zone(),
                session.editor.zone()
            );
        }
        2 => {
            world.run_system_once(probe).unwrap();
            capture(world, &run, "preview");
        }
        3 => {
            world.resource_mut::<Session>().stroke = None;
        }
        4 => {
            world.run_system_once(probe).unwrap();
            let mut session = world.resource_mut::<Session>();
            assert_eq!(session.editor.zone(), run.original.as_ref().unwrap());
            terrain::begin_stroke(&mut session);
            stamp(&mut session, terrain::Tool::Raise, [2560., 2560.]);
            stamp(&mut session, terrain::Tool::Lower, [2562., 2560.]);
            stamp(&mut session, terrain::Tool::Smooth, [2560., 2560.]);
            session.flatten_height = session
                .editor
                .zone()
                .surface_position([2560., 2560.])
                .unwrap()[1]
                + 12.;
            session.sculpt_blend = 1.;
            stamp(&mut session, terrain::Tool::Flatten, [2560., 2560.]);
            let stroke = session.stroke.take().unwrap();
            assert_eq!(stroke.stamps(), 4);
            terrain::commit_stroke(&mut session, stroke);
            assert_eq!(session.editor.history(), (1, 0));
            session.set_mode(EditorMode::Paint);
            terrain::begin_stroke(&mut session);
            stamp(
                &mut session,
                terrain::Tool::Paint(theia_zone::painting::Surface::Sand),
                [2560., 2560.],
            );
            let stroke = session.stroke.take().unwrap();
            terrain::commit_stroke(&mut session, stroke);
            assert_eq!(session.editor.history(), (2, 0));
            assert!(session.is_dirty());
            session.brush_point = None;
            run.committed = Some(session.editor.zone().clone());
        }
        5 => {
            world.run_system_once(probe).unwrap();
            let scene = world.resource::<Scene>();
            let meshes = world.resource::<Assets<Mesh>>();
            let mut seam_normal = None;
            let cells = world
                .resource::<Session>()
                .editor
                .zone()
                .imported_terrain
                .as_ref()
                .unwrap()
                .cells;
            for (x, z, index) in [
                (19, 19, (cells + 1).pow(2) - 1),
                (20, 19, cells * (cells + 1)),
                (19, 20, cells),
                (20, 20, 0),
            ] {
                let tile = scene
                    .tiles
                    .iter()
                    .find(|t| t.id == ChunkId { x, z })
                    .unwrap();
                assert_eq!(tile.cells, cells);
                let Some(bevy::mesh::VertexAttributeValues::Float32x3(n)) = meshes
                    .get(&tile.mesh)
                    .unwrap()
                    .attribute(Mesh::ATTRIBUTE_NORMAL)
                else {
                    panic!()
                };
                let n = Vec3::from_array(n[index]);
                if let Some(previous) = seam_normal {
                    assert!(n.distance(previous) < 1e-6);
                }
                seam_normal = Some(n);
            }
            capture(world, &run, "sculpted-painted-seam");
        }
        6 => {
            let mut session = world.resource_mut::<Session>();
            let r = session.editor.revision();
            session.execute(Command::Undo {
                expected_revision: r,
            });
            let r = session.editor.revision();
            session.execute(Command::Undo {
                expected_revision: r,
            });
            assert_eq!(session.editor.zone(), run.original.as_ref().unwrap());
        }
        7 => {
            world.run_system_once(probe).unwrap();
            let mut session = world.resource_mut::<Session>();
            let r = session.editor.revision();
            session.execute(Command::Redo {
                expected_revision: r,
            });
            let r = session.editor.revision();
            session.execute(Command::Redo {
                expected_revision: r,
            });
            assert_eq!(session.editor.zone(), run.committed.as_ref().unwrap());
        }
        8 => {
            world.run_system_once(probe).unwrap();
            let mut view = world.remove_resource::<View>().unwrap();
            let mut session = world.resource_mut::<Session>();
            session.assign_path(run.output.join("edited.zone.json"));
            crate::perform(Action::Save, &mut session, &mut view);
            assert!(!session.is_dirty(), "{}", session.status);
            crate::perform(Action::Load, &mut session, &mut view);
            assert_eq!(session.editor.zone(), run.committed.as_ref().unwrap());
            let g = session
                .editor
                .zone()
                .surface_position([4000., 64.])
                .unwrap();
            view.target = Vec3::from_array(session.editor.zone().local(g).unwrap());
            terrain::begin_stroke(&mut session);
            stamp(&mut session, terrain::Tool::Raise, [4000., 64.]);
            stamp(
                &mut session,
                terrain::Tool::Paint(theia_zone::painting::Surface::Rock),
                [4000., 64.],
            );
            let stroke = session.stroke.take().unwrap();
            terrain::commit_stroke(&mut session, stroke);
            session.brush_point = None;
            world.insert_resource(view);
        }
        9 => {
            world.run_system_once(probe).unwrap();
            capture(world, &run, "far-corner");
        }
        10 => {
            let mut view = world.remove_resource::<View>().unwrap();
            let mut session = world.resource_mut::<Session>();
            crate::perform(Action::Save, &mut session, &mut view);
            assert!(!session.is_dirty());
            let saved = session.editor.zone().clone();
            crate::perform(Action::Load, &mut session, &mut view);
            assert_eq!(session.editor.zone(), &saved);
            let source = saved.imported_terrain.as_ref().unwrap();
            let report = serde_json::json!({"passed":true,"stamps":source.stamp_count(),"preview_cancel":true,"one_undo_per_stroke":true,"undo_redo":true,"save_reload":true,"seam_and_far_rendered_rays":true,"shared_edge_normals":true,"document_bytes":std::fs::metadata(run.output.join("edited.zone.json")).unwrap().len(),"scene":world.resource::<Scene>().summary(),"scope":"Shared stroke lifecycle and native rendered mesh rays; no OS input injection"});
            std::fs::write(
                run.output.join("qualification.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            world.insert_resource(view);
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
