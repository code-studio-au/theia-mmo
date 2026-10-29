//! Opt-in bounded publication qualifier using real standard GLB scenes and Avian.
//! No network authority, normal-tile partitioning or general controller claim.
use crate::capture_ready::{CaptureReady, CaptureReadyPlugin};
use avian3d::prelude::*;
use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState},
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::window::screenshot::{Screenshot, ScreenshotCaptured},
    },
    tasks::{IoTaskPool, Task, futures::check_ready},
    window::WindowResolution,
    world_serialization::WorldAssetRoot,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::Instant,
};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalPosition};
use theia_network::{
    chunks::{Index, Residency},
    physics::{GROUND, SOLID},
};

#[derive(Component)]
struct Owned {
    id: ChunkId,
    visual: bool,
}
struct Root {
    entity: Entity,
    handle: Handle<Gltf>,
}
struct Candidate {
    data: Residency,
    handles: BTreeMap<ChunkId, Handle<Gltf>>,
    roots: BTreeMap<ChunkId, Root>,
}
enum Pending {
    Worker(Task<Result<Residency, String>>),
    Assets(Box<Candidate>),
}
#[derive(Resource)]
struct Session {
    directory: PathBuf,
    output: PathBuf,
    active: Residency,
    roots: BTreeMap<ChunkId, Root>,
    pending: Option<Pending>,
    stage: usize,
    stable: u32,
    capture: bool,
    expect_failure: bool,
    retained: Option<Entity>,
    rejected: bool,
    records: Vec<Value>,
    started: Instant,
    frame_checks: u64,
    max_staging_roots: usize,
}
#[derive(Resource, Default)]
struct Captured(bool);

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !(args.len() == 3 || args.len() == 4 && args[3] == "--expect-failure") {
        return Err(
            "Expected --chunk-smoke <fixture-directory> <output> [--expect-failure]".into(),
        );
    }
    let directory = std::fs::canonicalize(&args[1])?;
    let output = PathBuf::from(&args[2]);
    std::fs::create_dir_all(&output)?;
    let output = std::fs::canonicalize(output)?;
    for name in ["report.json", "initial.png", "replacement.png", "empty.png"] {
        let _ = std::fs::remove_file(output.join(name));
    }
    let index = Index::load(&directory).map_err(|e| e.to_string())?;
    for id in index.ids() {
        let manifest = theia_network::content::Manifest::load_optional(
            &index
                .directory(id)
                .join(theia_network::content::MANIFEST_FILE),
        )
        .map_err(|e| e.to_string())?;
        if manifest.is_some_and(|m| m.requires_material_profiles()) {
            return Err("Bound projection packages require the normal Renet material/readiness path; --chunk-smoke supports standard material packages".into());
        }
    }
    let active = Residency::new(index, 2).map_err(|e| e.to_string())?;
    App::new()
        .insert_resource(Session {
            directory: directory.clone(),
            output,
            active,
            roots: BTreeMap::new(),
            pending: None,
            stage: 0,
            stable: 0,
            capture: false,
            expect_failure: args.len() == 4,
            retained: None,
            rejected: false,
            records: Vec::new(),
            started: Instant::now(),
            frame_checks: 0,
            max_staging_roots: 0,
        })
        .insert_resource(ClearColor(Color::srgb(0.03, 0.04, 0.07)))
        .init_resource::<Captured>()
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::audio::AudioPlugin>()
                .set(AssetPlugin {
                    file_path: directory
                        .to_str()
                        .ok_or("Fixture path must be UTF-8")?
                        .to_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Theia — bounded chunk publication".into(),
                        resolution: WindowResolution::new(960, 640).with_scale_factor_override(1.),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(PhysicsPlugins::new(PostUpdate))
        .add_plugins(CaptureReadyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, drive)
        .add_systems(Last, probe)
        .run();
    if !std::path::Path::new(&args[2]).join("report.json").is_file() {
        return Err("Chunk qualification did not finish".into());
    }
    Ok(())
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Tonemapping::Reinhard,
        Transform::from_xyz(64., 210., 290.).looking_at(Vec3::new(64., 0., 64.), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.4, 0.)),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 300.,
        ..default()
    });
}
fn point(data: &Residency, x: f32) -> GlobalPosition {
    data.index()
        .frame()
        .to_global(LocalPosition::new([x, 0., 32.]).unwrap())
        .unwrap()
}
fn request(session: &mut Session, ids: Vec<ChunkId>, protected: Vec<GlobalPosition>) {
    assert!(session.pending.is_none(), "At most one pending candidate");
    let active = session.active.clone();
    session.pending = Some(Pending::Worker(IoTaskPool::get().spawn(async move {
        let candidate = active.staged(&ids, &protected).map_err(|e| e.to_string())?;
        candidate.verify_visuals().map_err(|e| e.to_string())?;
        Ok(candidate)
    })));
    session.stable = 0;
}
fn drive(world: &mut World) {
    world.resource_scope(|world, mut s: Mut<Session>| {
        assert!(
            s.started.elapsed().as_secs() < 120,
            "Chunk publication qualification timed out"
        );
        if s.stage == 0 {
            let left = s.active.index().owner(point(&s.active, -0.25)).unwrap();
            let middle = s.active.index().owner(point(&s.active, 0.)).unwrap();
            request(&mut s, vec![left, middle], vec![]);
            s.stage = 1;
        }
        if let Some(pending) = s.pending.take() {
            let result = match pending {
                Pending::Worker(mut task) => match check_ready(&mut task) {
                    None => {
                        s.pending = Some(Pending::Worker(task));
                        return;
                    }
                    Some(Err(error)) => Err(error),
                    Some(Ok(data)) => {
                        let handles = data
                            .visuals()
                            .into_iter()
                            .filter(|r| !s.roots.contains_key(&r.chunk))
                            .map(|r| {
                                let relative = r
                                    .asset
                                    .strip_prefix(&s.directory)
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .replace('\\', "/");
                                (
                                    r.chunk,
                                    world.resource::<AssetServer>().load::<Gltf>(relative),
                                )
                            })
                            .collect();
                        s.pending = Some(Pending::Assets(Box::new(Candidate {
                            data,
                            handles,
                            roots: BTreeMap::new(),
                        })));
                        return;
                    }
                },
                Pending::Assets(mut c) => {
                    if c.handles.values().any(|h| {
                        matches!(
                            world.resource::<AssetServer>().load_state(h.id()),
                            LoadState::Failed(_)
                        ) || matches!(
                            world
                                .resource::<AssetServer>()
                                .recursive_dependency_load_state(h.id()),
                            RecursiveDependencyLoadState::Failed(_)
                        )
                    }) {
                        for root in c.roots.values() {
                            world.despawn(root.entity);
                        }
                        Err("Standard GLB asset/dependency loading failed".into())
                    } else if !c.handles.values().all(|h| {
                        world
                            .resource::<AssetServer>()
                            .is_loaded_with_dependencies(h.id())
                    }) {
                        s.pending = Some(Pending::Assets(c));
                        return;
                    } else {
                        for (&id, handle) in &c.handles {
                            if c.roots.contains_key(&id) {
                                continue;
                            }
                            let gltf = world.resource::<Assets<Gltf>>().get(handle).unwrap();
                            let scene = gltf
                                .default_scene
                                .clone()
                                .or_else(|| gltf.scenes.first().cloned())
                                .expect("Fixture GLB scene");
                            let position = c
                                .data
                                .index()
                                .frame()
                                .to_local(id.origin(128).unwrap())
                                .unwrap()
                                .metres();
                            let entity = world
                                .spawn((
                                    Owned { id, visual: true },
                                    WorldAssetRoot(scene),
                                    Transform::from_translation(Vec3::from_array(position)),
                                    Visibility::Hidden,
                                ))
                                .id();
                            c.roots.insert(
                                id,
                                Root {
                                    entity,
                                    handle: handle.clone(),
                                },
                            );
                        }
                        s.max_staging_roots = s.max_staging_roots.max(c.roots.len());
                        let ready = c
                            .roots
                            .values()
                            .all(|r| hierarchy_has_mesh(world, r.entity));
                        if !ready {
                            s.pending = Some(Pending::Assets(c));
                            return;
                        }
                        publish(world, &mut s, *c);
                        Ok(())
                    }
                }
            };
            if let Err(error) = result {
                assert!(
                    s.expect_failure && s.stage == 2,
                    "Unexpected chunk rejection: {error}"
                );
                s.rejected = true;
                s.stable = 0;
            }
        }
    });
}
fn hierarchy_has_mesh(world: &World, root: Entity) -> bool {
    let Some(children) = world.get::<Children>(root) else {
        return false;
    };
    children
        .iter()
        .any(|child| world.get::<Mesh3d>(child).is_some() || hierarchy_has_mesh(world, child))
}
fn publish(world: &mut World, s: &mut Session, mut c: Candidate) {
    let ids: BTreeSet<_> = c.data.loaded().into_iter().collect();
    let mut remove = world.query::<(Entity, &Owned)>();
    let remove: Vec<_> = remove
        .iter(world)
        .filter(|(_, owner)| !ids.contains(&owner.id))
        .map(|(e, _)| e)
        .collect();
    for entity in remove {
        world.despawn(entity);
    }
    s.roots.retain(|id, _| ids.contains(id));
    for root in c.roots.values() {
        world.entity_mut(root.entity).insert(Visibility::Inherited);
    }
    for (&id, root) in &c.roots {
        // Flat physics entities get explicit frame-relative positions; a root
        // Transform alone does not relocate Avian's body positions.
        let offset = Vec3::from_array(
            c.data
                .index()
                .frame()
                .to_local(id.origin(128).unwrap())
                .unwrap()
                .metres(),
        );
        for collider in c.data.colliders(id).unwrap() {
            world
                .spawn((Owned { id, visual: false }, collider.bundle()))
                .insert(Transform::from_translation(collider.center + offset));
        }
        assert!(root.handle.id() != Handle::<Gltf>::default().id());
    }
    s.roots.append(&mut c.roots);
    s.active = c.data;
    s.stable = 0;
    world.resource_mut::<CaptureReady>().reset();
}
fn probe(world: &mut World) {
    world.resource_scope(|world,mut s:Mut<Session>| {
        // Query actual client Avian after PostUpdate, including during staging.
        let loaded=s.active.loaded();
        {
        let mut state=bevy::ecs::system::SystemState::<SpatialQuery>::new(world);
        let query=state.get(world).unwrap();
        for x in [-64.,64.,192.] {
            let expected=s.active.scene().and_then(|scene|scene.ground([x,32.],5.));
            let actual=query.cast_ray(Vec3::new(x,5.,32.),Dir3::NEG_Y,10.,true,&SpatialQueryFilter::from_mask(GROUND));
            assert_eq!(actual.is_some(),expected.is_some(),"Published collision disagrees at x={x}");
            if let (Some(hit),Some((ground,_)))=(actual,expected) {assert!((5.-hit.distance-ground).abs()<0.002);}
        }
        let hit=query.cast_shape(&theia_network::physics::character_shape(0.),Vec3::new(63.,0.925,64.),Quat::IDENTITY,Dir3::X,&ShapeCastConfig {max_distance:2.,target_distance:0.0005,ignore_origin_penetration:true,..default()},&SpatialQueryFilter::from_mask(SOLID));
        assert_eq!(hit.is_some(),!loaded.is_empty());
        if let Some(hit)=hit {assert!((hit.distance-0.5495).abs()<0.002);}
        }
        let mut owned=world.query::<(&Owned,Option<&Visibility>)>();
        let visible: BTreeSet<_>=owned.iter(world).filter(|(o,v)|o.visual && v.is_some_and(|v|*v!=Visibility::Hidden)).map(|(o,_)|o.id).collect();
        assert_eq!(visible,loaded.iter().copied().collect(),"Visual/collision publication mismatch");
        assert!(s.roots.len()<=2 && s.max_staging_roots<=2);
        s.frame_checks+=1;
        if s.pending.is_some() || s.capture || s.stage==0 {return;}
        if s.stage==2 && s.expect_failure && !s.rejected {return;}
        if !world.resource::<CaptureReady>().ready() && !loaded.is_empty() {return;}
        s.stable+=1;
        if s.stable<8 {return;}
        if let Some(entity)=s.retained && s.stage==2 {
            assert_eq!(s.roots.get(&s.active.index().owner(point(&s.active,64.)).unwrap()).unwrap().entity,entity,"Retained root was respawned");
        }
        let record=json!({"stage":s.stage,"loaded":loaded.iter().map(|id|[id.x,id.z]).collect::<Vec<_>>(),"roots":s.roots.len(),"rejected":s.rejected,"frame_checks":s.frame_checks,"max_staging_roots":s.max_staging_roots});
        s.records.push(record);
        let name=match s.stage {1=>"initial.png",2=>"replacement.png",3=>"empty.png",_=>unreachable!()};
        let path=s.output.join(name);s.capture=true;world.resource_mut::<Captured>().0=false;
        world.spawn(Screenshot::primary_window()).observe(move |event:On<ScreenshotCaptured>,mut captured:ResMut<Captured>| {
            event.image.clone().try_into_dynamic().unwrap().save(&path).unwrap();captured.0=true;
        });
    });
    if !world.resource::<Captured>().0 {
        return;
    }
    world.resource_scope(|world,mut s:Mut<Session>| {
        if !s.capture {return;}
        s.capture=false;world.resource_mut::<Captured>().0=false;
        if s.stage==1 {
            let middle=s.active.index().owner(point(&s.active,64.)).unwrap();
            let right=s.active.index().owner(point(&s.active,128.)).unwrap();
            s.retained=Some(s.roots[&middle].entity);
            let protected=point(&s.active,64.);request(&mut s,vec![middle,right],vec![protected]);s.stage=2;
        } else if s.stage==2 {request(&mut s,vec![],vec![]);s.stage=3;}
        else {
            let adapter=world.resource::<RenderAdapterInfo>();
            let report=json!({"schema":1,"passed":true,"expected_failure":s.expect_failure,"records":s.records,"gpu":{"name":adapter.name,"backend":format!("{:?}",adapter.backend)},"frame_checks":s.frame_checks,"query_origin":s.active.index().frame().origin().metres(),"scope":"real bounded GLB/Avian publication; no networking, normal-tile partition or capacity claim"});
            std::fs::write(s.output.join("report.json"),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            world.write_message(AppExit::Success);
        }
    });
}
