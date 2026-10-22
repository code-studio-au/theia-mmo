//! Shared chunk-world startup in the normal Bevy client. Dynamic publication is
//! qualified separately before replacing the monolithic development map default.
use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState},
    camera::primitives::MeshAabb,
    mesh::morph::{MeshMorphWeights, MorphWeights},
    prelude::*,
    tasks::{IoTaskPool, Task, futures::check_ready},
    world_serialization::WorldAssetRoot,
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};
use theia_core::coordinates::ChunkId;
use theia_network::{chunks::Residency, physics::StaticScene};

#[derive(Resource)]
pub struct Config {
    pub directory: PathBuf,
    pub data: theia_network::chunk_world::Config,
    pub active: Residency,
    recovery_spawns: Vec<theia_core::coordinates::GlobalPosition>,
}
impl Config {
    pub fn parse(args: &mut Vec<String>) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        if args.first().is_none_or(|a| a != "--chunk-world") {
            return Ok(None);
        }
        let directory =
            std::fs::canonicalize(args.get(1).ok_or("--chunk-world needs a directory")?)?;
        let data =
            theia_network::chunk_world::Config::load(&directory).map_err(|e| e.to_string())?;
        let (active, recovery_spawns) = data.prepare_startup().map_err(|e| e.to_string())?;
        active.verify_visuals().map_err(|e| e.to_string())?;
        args.drain(..2);
        Ok(Some(Self {
            directory,
            data,
            active,
            recovery_spawns,
        }))
    }
    pub fn scene(&self) -> &StaticScene {
        self.active
            .scene()
            .expect("Validated initial chunk collision")
    }
}
#[derive(Resource)]
pub struct Ready;
#[derive(Resource)]
pub struct BoundsChecked;
#[derive(Component)]
pub struct Root(pub ChunkId);
#[derive(Resource)]
pub struct Loading {
    handles: BTreeMap<ChunkId, Handle<Gltf>>,
    roots: BTreeMap<ChunkId, Entity>,
    started: Instant,
}
struct Candidate {
    data: Residency,
    handles: BTreeMap<ChunkId, Handle<Gltf>>,
    roots: BTreeMap<ChunkId, Entity>,
    withdrawal: Option<Vec<ChunkId>>,
}
enum Pending {
    Worker(u64, Task<Result<Residency, String>>),
    Assets(u64, Box<Candidate>),
}
#[derive(Resource)]
pub struct Stream {
    pending: Option<Pending>,
    seen: u64,
    permission: Vec<ChunkId>,
    waiting_probe: bool,
    started: Instant,
    delay: Duration,
    pub publications: u64,
    pub rejections: u64,
    pub hidden_remote_frames: u64,
    max_staging_roots: usize,
    records: Vec<serde_json::Value>,
    events: Vec<serde_json::Value>,
    settle_since: Option<Instant>,
    settled_generation: u64,
    waiting_reason: Option<String>,
    pub surface_probe: Option<serde_json::Value>,
}
impl Stream {
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({"server_generation":self.seen,"pending":self.pending.is_some(),"waiting_probe":self.waiting_probe,
            "publications":self.publications,"rejections":self.rejections,"hidden_remote_frames":self.hidden_remote_frames,
            "max_staging_roots":self.max_staging_roots,"records":self.records,"events":self.events,"waiting_reason":self.waiting_reason,"surface_probe":self.surface_probe})
    }
    pub fn diagnostic_ready(&self) -> bool {
        self.pending.is_none()
            && !self.waiting_probe
            && (self.publications > 0 || self.rejections > 0)
            && self.settled_generation == self.seen
    }
}
pub fn load(mut commands: Commands, config: Res<Config>, assets: Res<AssetServer>) {
    let delay = std::env::var("THEIA_CHUNK_PREPARE_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        .min(10_000);
    commands.insert_resource(Stream {
        pending: None,
        seen: 0,
        permission: config.active.loaded(),
        waiting_probe: false,
        started: Instant::now(),
        delay: Duration::from_millis(delay),
        publications: 0,
        rejections: 0,
        hidden_remote_frames: 0,
        max_staging_roots: 0,
        records: Vec::new(),
        events: Vec::new(),
        settle_since: None,
        settled_generation: 0,
        waiting_reason: None,
        surface_probe: None,
    });
    let handles = config
        .active
        .visuals()
        .into_iter()
        .map(|r| {
            let relative = r
                .asset
                .strip_prefix(&config.directory)
                .expect("Derived chunk path")
                .to_str()
                .expect("UTF-8 chunk path")
                .replace('\\', "/");
            (r.chunk, assets.load(relative))
        })
        .collect();
    commands.insert_resource(Loading {
        handles,
        roots: BTreeMap::new(),
        started: Instant::now(),
    });
}
#[derive(bevy::ecs::system::SystemParam)]
pub struct ReadyView<'w, 's> {
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<Camera3d>>,
    frame: Res<'w, crate::floating_origin::Frame>,
}
pub fn report_ready(
    config: Res<Config>,
    ready: Option<Res<Ready>>,
    network: Res<crate::network::Network>,
    backend: Res<crate::physics::Backend>,
    pipelines: Res<crate::capture_ready::CaptureReady>,
    mut stream: ResMut<Stream>,
    view: ReadyView,
) {
    let ReadyView { camera, frame } = view;
    if ready.is_some() && network.connected() && backend.probes > 0 {
        if stream.waiting_probe
            && backend.probed_generation == config.active.generation()
            && pipelines.ready()
        {
            stream.waiting_probe = false;
            stream.permission = config.active.loaded();
        }
        network.publish_chunks(stream.permission.clone());
        if let Some(p) = network.rendered_global()
            && let Ok((camera, transform)) = camera.single()
        {
            let [x, _, z] = p.metres();
            let y = config.data.spawns()[0].metres()[1];
            if let Some(point) = theia_core::coordinates::GlobalPosition::new([x + 2., y, z + 2.])
                && let Some(local) = frame.render.to_local(point)
                && let Ok(viewport) =
                    camera.world_to_viewport(transform, Vec3::from_array(local.metres()))
            {
                let query = config
                    .data
                    .index()
                    .frame()
                    .to_local(point)
                    .unwrap()
                    .metres();
                stream.surface_probe = Some(
                    serde_json::json!({"world":point.metres(),"viewport":viewport.to_array(),"present":config.scene().ground([query[0],query[2]],query[1]+1.).is_some()}),
                );
            }
        }
    }
}

fn cleanup(world: &mut World, candidate: &Candidate) {
    for &entity in candidate.roots.values() {
        world.despawn(entity);
    }
}
fn has_mesh(world: &World, root: Entity) -> bool {
    world.get::<Children>(root).is_some_and(|children| {
        children
            .iter()
            .any(|e| world.get::<Mesh3d>(e).is_some() || has_mesh(world, e))
    })
}
fn observe(world: &mut World, stream: &mut Stream, generation: u64, phase: &str) {
    if stream.events.len() >= 64 {
        return;
    }
    let roots = world
        .query::<(Entity, &Root)>()
        .iter(world)
        .map(|(entity, root)| (entity, root.0))
        .collect::<Vec<_>>();
    let roots = roots
        .into_iter()
        .map(|(entity, id)| {
            let mut pending = vec![entity];
            let mut meshes = 0;
            while let Some(e) = pending.pop() {
                meshes += usize::from(world.get::<Mesh3d>(e).is_some());
                if let Some(children) = world.get::<Children>(e) {
                    pending.extend(children.iter());
                }
            }
        serde_json::json!({"id":[id.x,id.z],"entity":format!("{entity:?}"),"meshes":meshes,"visibility":world.get::<Visibility>(entity).map(|v|format!("{v:?}"))})
        })
        .collect::<Vec<_>>();
    let server = world.resource::<AssetServer>();
    let mut images = world
        .resource::<Assets<Image>>()
        .ids()
        .filter_map(|id| {
            let path = server.get_path(id)?;
            (path.label().is_none() && path.path().starts_with("shared"))
                .then(|| serde_json::json!({"path":path.to_string(),"handle":format!("{id:?}")}))
        })
        .take(1025)
        .collect::<Vec<_>>();
    let truncated = images.len() > 1024;
    images.truncate(1024);
    stream.events.push(serde_json::json!({"generation":generation,"phase":phase,"elapsed_ms":stream.started.elapsed().as_secs_f64()*1000.,
        "roots":roots,"entities":world.entities().len(),"gltfs":world.resource::<Assets<Gltf>>().len(),"meshes":world.resource::<Assets<Mesh>>().len(),
        "images":world.resource::<Assets<Image>>().len(),"shared_images":images,"shared_images_truncated":truncated,"materials":world.resource::<Assets<StandardMaterial>>().len()}));
    info!(
        "Chunk candidate phase: generation={generation} phase={phase} elapsed_ms={:.3}",
        stream.started.elapsed().as_secs_f64() * 1000.
    );
}
/// Validate actual standard-Bevy mesh bounds in package-local coordinates.
/// Ignore the root's floating-origin placement; preserve every child transform.
fn hierarchy_in_bounds(world: &World, root: Entity, bounds: [[f64; 2]; 2]) -> Result<(), String> {
    let meshes = world.resource::<Assets<Mesh>>();
    let mut pending = world
        .get::<Children>(root)
        .into_iter()
        .flat_map(|c| c.iter())
        .map(|e| (e, Mat4::IDENTITY, 0usize))
        .collect::<Vec<_>>();
    let mut visited = 0usize;
    while let Some((entity, parent, depth)) = pending.pop() {
        visited += 1;
        if depth > 128 || visited > 65536 {
            return Err("Chunk visual hierarchy exceeds validation budget".into());
        }
        let matrix = parent
            * world
                .get::<Transform>(entity)
                .map_or(Mat4::IDENTITY, Transform::to_matrix);
        if !matrix.is_finite() {
            return Err("Nonfinite chunk visual transform".into());
        }
        if world.get::<MeshMorphWeights>(entity).is_some()
            || world.get::<MorphWeights>(entity).is_some()
        {
            return Err("Morphing chunk geometry is unsupported".into());
        }
        if let Some(mesh) = world.get::<Mesh3d>(entity) {
            let mesh = meshes.get(&mesh.0).ok_or("Chunk mesh is not loaded")?;
            let aabb = mesh.get_aabb().ok_or("Chunk mesh has no finite bounds")?;
            let low = Vec3::from(aabb.min());
            let high = Vec3::from(aabb.max());
            for x in [low.x, high.x] {
                for y in [low.y, high.y] {
                    for z in [low.z, high.z] {
                        let point = matrix.transform_point3(Vec3::new(x, y, z));
                        if !point.is_finite()
                            || [point.x, point.z]
                                .into_iter()
                                .enumerate()
                                .any(|(axis, value)| {
                                    f64::from(value) < bounds[0][axis]
                                        || f64::from(value) > bounds[1][axis]
                                })
                        {
                            return Err("Visual exceeds indexed contributor bounds".into());
                        }
                    }
                }
            }
        }
        if let Some(children) = world.get::<Children>(entity) {
            pending.extend(children.iter().map(|e| (e, matrix, depth + 1)));
        }
    }
    Ok(())
}
fn validate_roots(
    world: &World,
    residency: &Residency,
    roots: &BTreeMap<ChunkId, Entity>,
    handles: &BTreeMap<ChunkId, Handle<Gltf>>,
) -> Result<(), String> {
    for (&id, &root) in roots {
        let Some(bounds) = residency.index().bounds(id) else {
            continue;
        };
        let gltf = world
            .resource::<Assets<Gltf>>()
            .get(&handles[&id])
            .ok_or("Chunk GLB is not loaded")?;
        if !gltf.skins.is_empty() || !gltf.animations.is_empty() {
            return Err("Animated or skinned chunk geometry is unsupported".into());
        }
        hierarchy_in_bounds(world, root, bounds)?;
    }
    Ok(())
}
pub fn validate_initial(world: &mut World) {
    if world.contains_resource::<BoundsChecked>() {
        return;
    }
    let loading = world.resource::<Loading>();
    if loading.roots.is_empty() || !loading.roots.values().all(|&root| has_mesh(world, root)) {
        return;
    }
    for &root in loading.roots.values() {
        match crate::runtime_materials::root_ready(world, root) {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                error!("Chunk startup material validation failed: {error}");
                world.write_message(AppExit::error());
                return;
            }
        }
    }
    let result = validate_roots(
        world,
        &world.resource::<Config>().active,
        &loading.roots,
        &loading.handles,
    );
    match result {
        Ok(()) => {
            let roots = world
                .resource::<Loading>()
                .roots
                .values()
                .copied()
                .collect::<Vec<_>>();
            for root in roots {
                world.entity_mut(root).insert(Visibility::Inherited);
            }
            world
                .resource_mut::<crate::capture_ready::CaptureReady>()
                .reset();
            world.insert_resource(BoundsChecked);
        }
        Err(error) => {
            error!("Chunk startup contributor validation failed: {error}");
            world.write_message(AppExit::error());
        }
    }
}
fn discard(
    world: &mut World,
    stream: &mut Stream,
    generation: u64,
    candidate: Option<&Candidate>,
    error: &str,
) {
    if let Some(candidate) = candidate {
        observe(world, stream, generation, "discarding");
        cleanup(world, candidate);
    }
    stream.rejections += 1;
    stream.seen = generation;
    stream.permission = world.resource::<Config>().active.loaded();
    stream.settle_since = None;
    stream.waiting_reason = None;
    observe(world, stream, generation, "rejected");
    warn!("Chunk candidate rejected: {error}; active scene retained");
}
pub fn drive(world: &mut World) {
    if !world.contains_resource::<Ready>() {
        return;
    }
    world.resource_scope(|world,mut stream:Mut<Stream>| {
        if stream.waiting_probe {return;}
        if stream.pending.is_none() && (stream.publications>0 || stream.rejections>0) && stream.settled_generation!=stream.seen
            && stream.settle_since.get_or_insert_with(Instant::now).elapsed()>=Duration::from_secs(2) {
            let generation=stream.seen;observe(world,&mut stream,generation,"settled");stream.settled_generation=generation;
        }
        let state=world.resource::<crate::network::Network>().state();
        let Some(offer)=state.chunk_offer.as_ref() else {return;};
        if let Some(pending)=stream.pending.take() {
            match pending {
                Pending::Worker(generation,mut task)=>{
                    match check_ready(&mut task) {
                        None=>{stream.pending=Some(Pending::Worker(generation,task));}
                        Some(_) if generation!=offer.generation=>{},
                        Some(Err(error))=>discard(world,&mut stream,generation,None,&error),
                        Some(Ok(data))=>{
                            observe(world,&mut stream,generation,"prepared");
                            let loading=world.resource::<Loading>();
                            let directory=&world.resource::<Config>().directory;
                            let handles=data.visuals().into_iter().filter(|r|!loading.roots.contains_key(&r.chunk)).map(|r| {
                                let path=r.asset.strip_prefix(directory).unwrap().to_str().unwrap().replace('\\',"/");
                                (r.chunk,world.resource::<AssetServer>().load(path))
                            }).collect();
                            stream.pending=Some(Pending::Assets(generation,Box::new(Candidate {data,handles,roots:BTreeMap::new(),withdrawal:None})));
                        }
                    }
                    return;
                }
                Pending::Assets(generation,mut candidate)=>{
                    if generation!=offer.generation {cleanup(world,&candidate);stream.permission=world.resource::<Config>().active.loaded();observe(world,&mut stream,generation,"superseded");return;}
                    let server=world.resource::<AssetServer>();
                    let failed=candidate.handles.values().any(|h|matches!(server.load_state(h.id()),LoadState::Failed(_))||matches!(server.recursive_dependency_load_state(h.id()),RecursiveDependencyLoadState::Failed(_)));
                    if failed || stream.started.elapsed().as_secs()>120 {discard(world,&mut stream,generation,Some(&candidate),"asset failure or timeout");return;}
                    if !candidate.handles.values().all(|h|server.is_loaded_with_dependencies(h.id())) {stream.pending=Some(Pending::Assets(generation,candidate));return;}
                    for (&id,handle) in &candidate.handles {
                        if candidate.roots.contains_key(&id) {continue;}
                        let gltf=world.resource::<Assets<Gltf>>().get(handle).unwrap();
                        let Some(scene)=gltf.default_scene.clone().or_else(||gltf.scenes.first().cloned()) else {discard(world,&mut stream,generation,Some(&candidate),"GLB has no scene");return;};
                        let local=candidate.data.index().frame().to_local(id.origin(theia_core::coordinates::INITIAL_CHUNK_METRES).unwrap()).unwrap().metres();
                        let position=world.resource::<crate::floating_origin::Frame>().anchored(local.map(f64::from));
                        let root=world.spawn((Root(id),crate::terrain_scene::SceneryRoot,crate::floating_origin::Anchored(local.map(f64::from)),WorldAssetRoot(scene),Transform::from_translation(position),Visibility::Hidden)).id();
                        candidate.roots.insert(id,root);
                    }
                    stream.max_staging_roots=stream.max_staging_roots.max(candidate.roots.len());
                    if !candidate.roots.values().all(|&root|has_mesh(world,root)) {stream.pending=Some(Pending::Assets(generation,candidate));return;}
                    for &root in candidate.roots.values() {
                        match crate::runtime_materials::root_ready(world,root) {
                            Ok(true)=>{},
                            Ok(false)=>{stream.pending=Some(Pending::Assets(generation,candidate));return;},
                            Err(error)=>{discard(world,&mut stream,generation,Some(&candidate),&error);return;}
                        }
                    }
                    if candidate.withdrawal.is_none() {
                        if let Err(error)=validate_roots(world,&candidate.data,&candidate.roots,&candidate.handles) {discard(world,&mut stream,generation,Some(&candidate),&error);return;}
                        observe(world,&mut stream,generation,"staged");
                        let active=world.resource::<Config>().active.loaded();let next=candidate.data.loaded();
                        let intersection=active.into_iter().filter(|id|next.contains(id)).collect::<Vec<_>>();
                        stream.permission=intersection.clone();candidate.withdrawal=Some(intersection);
                    }
                    let mut protected=Vec::new();
                    if let Some(snapshot)=&state.snapshot {protected.extend(snapshot.players.iter().map(|p|snapshot.global_position(p).unwrap()));}
                    if let Some(p)=world.resource::<crate::network::Network>().rendered_global() {protected.push(p);}
                    let mut remotes=world.query::<&crate::network::RemotePawn>();
                    protected.extend(remotes.iter(world).flat_map(|p|p.global_endpoints()));
                    let config=world.resource::<Config>();
                    let valid=if !state.chunk_permission_confirmed(candidate.withdrawal.as_ref().unwrap()) {
                        Err("Waiting for exact retirement acknowledgement/snapshot".to_string())
                    } else {candidate.data.validate_presentation(&config.active,&protected,&config.recovery_spawns).map_err(|e|e.to_string())};
                    if let Err(reason)=valid {
                        if stream.waiting_reason.as_ref()!=Some(&reason) {info!("Chunk publication held: {reason}");stream.waiting_reason=Some(reason);}
                        stream.pending=Some(Pending::Assets(generation,candidate));return;
                    }
                    stream.waiting_reason=None;
                    let next=candidate.data.loaded();
                    let retained=world.resource::<Loading>().roots.iter().filter(|(id,_)|next.contains(id)).map(|(id,entity)|(*id,*entity)).collect::<Vec<_>>();
                    let mut removed=Vec::new();
                    {
                        let mut loading=world.resource_mut::<Loading>();
                        loading.roots.retain(|id,entity| {if next.contains(id) {true} else {removed.push(*entity);false}});
                        loading.handles.retain(|id,_|next.contains(id));
                        loading.roots.append(&mut candidate.roots);loading.handles.append(&mut candidate.handles);
                    }
                    for root in removed {world.despawn(root);}
                    let roots=world.resource::<Loading>().roots.values().copied().collect::<Vec<_>>();
                    for root in roots {world.entity_mut(root).insert(Visibility::Inherited);}
                    crate::physics::replace_scene(world,candidate.data.scene().unwrap(),candidate.data.generation());
                    world.resource_mut::<Config>().active=candidate.data;
                    world.resource_mut::<crate::capture_ready::CaptureReady>().reset();
                    stream.seen=generation;stream.publications+=1;stream.waiting_probe=true;
                    stream.settle_since=None;
                    observe(world,&mut stream,generation,"published");
                    let preserved=retained.iter().all(|(id,entity)|world.resource::<Loading>().roots.get(id)==Some(entity) && has_mesh(world,*entity));
                    assert!(preserved,"Retained chunk root was replaced or lost its hierarchy");
                    if stream.records.len()<4 {stream.records.push(serde_json::json!({"server_generation":generation,"loaded":next.iter().map(|id|[id.x,id.z]).collect::<Vec<_>>(),"ack":state.chunk_ack,"snapshot":state.snapshot,"retained_preserved":preserved}));}
                    info!("Chunk scene published: server_generation={generation}");
                    return;
                }
            }
        }
        if offer.generation<=stream.seen || stream.pending.is_some() {return;}
        let config=world.resource::<Config>();
        let ids=offer.chunks.iter().map(|&[x,z]|ChunkId{x,z}).collect::<Vec<_>>();
        if ids==config.active.loaded() {stream.seen=offer.generation;return;}
        let active=config.active.clone();let delay=stream.delay;
        stream.started=Instant::now();
        stream.settle_since=None;
        stream.waiting_reason=None;
        observe(world,&mut stream,offer.generation,"prepare_started");
        stream.pending=Some(Pending::Worker(offer.generation,IoTaskPool::get().spawn(async move {
            if !delay.is_zero() {std::thread::sleep(delay);}
            let data=active.staged(&ids,&[]).map_err(|e|e.to_string())?;
            data.verify_visuals().map_err(|e|e.to_string())?;
            Ok(data)
        })));
    });
}

pub fn remote_visibility(
    config: Res<Config>,
    mut stream: ResMut<Stream>,
    mut remotes: Query<(&crate::network::RemotePawn, &mut Visibility)>,
) {
    let coverage = config
        .data
        .index()
        .coverage(config.active.loaded())
        .unwrap();
    for (remote, mut visibility) in &mut remotes {
        let visible = remote
            .global_endpoints()
            .into_iter()
            .all(|p| coverage.contains(p));
        *visibility = if visible {
            Visibility::Inherited
        } else {
            stream.hidden_remote_frames += 1;
            Visibility::Hidden
        };
    }
}
pub fn tick(
    mut commands: Commands,
    (config, ready, checked): (Res<Config>, Option<Res<Ready>>, Option<Res<BoundsChecked>>),
    mut loading: ResMut<Loading>,
    (server, assets): (Res<AssetServer>, Res<Assets<Gltf>>),
    hierarchy: (Query<&Children>, Query<&Mesh3d>),
    mut pipelines: ResMut<crate::capture_ready::CaptureReady>,
    mut exit: MessageWriter<AppExit>,
) {
    if ready.is_some() {
        return;
    }
    let failed = loading.handles.values().any(|h| {
        matches!(server.load_state(h.id()), LoadState::Failed(_))
            || matches!(
                server.recursive_dependency_load_state(h.id()),
                RecursiveDependencyLoadState::Failed(_)
            )
    });
    if failed || loading.started.elapsed().as_secs() > 120 {
        error!("Chunk startup asset/dependency loading failed or timed out");
        exit.write(AppExit::error());
        return;
    }
    if !loading
        .handles
        .values()
        .all(|h| server.is_loaded_with_dependencies(h.id()))
    {
        return;
    }
    if loading.roots.is_empty() {
        let handles = loading.handles.clone();
        for (id, handle) in handles {
            let gltf = assets.get(&handle).expect("Loaded chunk GLB");
            let Some(scene) = gltf
                .default_scene
                .clone()
                .or_else(|| gltf.scenes.first().cloned())
            else {
                error!("Chunk GLB has no scene");
                exit.write(AppExit::error());
                return;
            };
            let local = config
                .data
                .index()
                .frame()
                .to_local(
                    id.origin(theia_core::coordinates::INITIAL_CHUNK_METRES)
                        .unwrap(),
                )
                .unwrap()
                .metres();
            let entity = commands
                .spawn((
                    Root(id),
                    crate::terrain_scene::SceneryRoot,
                    crate::floating_origin::Anchored(local.map(f64::from)),
                    WorldAssetRoot(scene),
                    Transform::from_translation(Vec3::from_array(local)),
                    Visibility::Hidden,
                ))
                .id();
            loading.roots.insert(id, entity);
        }
        pipelines.reset();
        return;
    }
    let (children, meshes) = hierarchy;
    if checked.is_none()
        || !loading
            .roots
            .values()
            .all(|&root| children.iter_descendants(root).any(|e| meshes.contains(e)))
        || !pipelines.ready()
    {
        return;
    }
    for &root in loading.roots.values() {
        commands.entity(root).insert(Visibility::Inherited);
    }
    commands.insert_resource(Ready);
    info!(
        "Chunk startup ready: {} standard scene roots, matching static collision",
        loading.roots.len()
    );
}

#[cfg(test)]
mod contributor_tests {
    use super::*;

    #[test]
    fn bounds_follow_actual_nested_mesh_transforms_and_ignore_render_root_placement() {
        let mut world = World::new();
        world.insert_resource(Assets::<Mesh>::default());
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(2., 2., 6.));
        let root = world.spawn(Transform::from_xyz(-1000., 25., 700.)).id();
        let node = world
            .spawn((Transform::from_xyz(132., 1., 132.), ChildOf(root)))
            .id();
        let part = world
            .spawn((
                Mesh3d(mesh.clone()),
                Transform::from_xyz(2., 0., 0.),
                ChildOf(node),
            ))
            .id();
        let bounds = [[133., 129.], [135., 135.]];
        assert!(hierarchy_in_bounds(&world, root, bounds).is_ok());
        assert!(
            hierarchy_in_bounds(&world, root, [[133., 129.], [134.99, 135.]])
                .unwrap_err()
                .contains("Visual exceeds")
        );
        // The normal render extraction retains cached bounds after CPU vertex
        // data leaves the main world. Validation must still inspect that mesh.
        let _uploaded = world
            .resource_mut::<Assets<Mesh>>()
            .get_mut(&mesh)
            .unwrap()
            .take_gpu_data()
            .unwrap();
        assert!(hierarchy_in_bounds(&world, root, bounds).is_ok());
        world
            .entity_mut(part)
            .insert(MeshMorphWeights::Reference(node));
        assert!(
            hierarchy_in_bounds(&world, root, bounds)
                .unwrap_err()
                .contains("Morphing")
        );
        world.entity_mut(part).remove::<MeshMorphWeights>();
        world.get_mut::<Transform>(part).unwrap().translation.z += 1.;
        assert!(hierarchy_in_bounds(&world, root, bounds).is_err());
    }

    #[test]
    fn contributor_extent_checks_reject_nonfinite_or_excessive_hierarchies() {
        let mut world = World::new();
        world.insert_resource(Assets::<Mesh>::default());
        let root = world.spawn_empty().id();
        let bad = world
            .spawn((Transform::from_xyz(f32::NAN, 0., 0.), ChildOf(root)))
            .id();
        assert!(
            hierarchy_in_bounds(&world, root, [[0., 0.], [128., 128.]])
                .unwrap_err()
                .contains("Nonfinite")
        );
        world.despawn(bad);
        let mut parent = root;
        for _ in 0..130 {
            parent = world.spawn((Transform::default(), ChildOf(parent))).id();
        }
        assert!(
            hierarchy_in_bounds(&world, root, [[0., 0.], [128., 128.]])
                .unwrap_err()
                .contains("budget")
        );
    }
}
