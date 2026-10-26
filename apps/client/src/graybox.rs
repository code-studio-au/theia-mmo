//! Normal development world: bounded asynchronous procedural terrain and standard
//! Bevy meshes, frustum culling, distance LOD, Avian collision and animated player.
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, mpsc},
    time::Instant,
};
use theia_core::coordinates::{ChunkId, GlobalPosition};
use theia_network::procedural::{self, Geometry, Patch};

#[derive(Component)]
struct Tile {
    id: ChunkId,
    cells: usize,
    mesh: Handle<Mesh>,
}
#[derive(Component)]
struct Body {
    global: [f64; 3],
}
#[derive(Component)]
struct Hud;
struct Built {
    id: ChunkId,
    cells: usize,
    geometry: Geometry,
}
#[derive(Resource)]
pub struct Graybox {
    render_motion: crate::movement_metrics::Metrics,
    saved: Option<Arc<theia_network::saved_terrain::Store>>,
    sender: mpsc::SyncSender<(ChunkId, usize)>,
    receiver: Mutex<mpsc::Receiver<theia_network::Result<Built>>>,
    pending: Option<(ChunkId, usize)>,
    prepared_visuals: std::collections::BTreeSet<ChunkId>,
    gpu_mesh_assets: usize,
    patch_sender: mpsc::SyncSender<ChunkId>,
    patch_receiver: Mutex<mpsc::Receiver<theia_network::Result<Arc<Patch>>>>,
    patch_pending: Option<ChunkId>,
    pub patch: Option<Arc<Patch>>,
    material: Option<Handle<StandardMaterial>>,
    pub loaded: u64,
    pub unloaded: u64,
    pub patches: u64,
    pub probes: u64,
    pub max_ground_error: f32,
    pub max_resident: usize,
    pub last_position: [f64; 3],
    frames: Vec<f64>,
    last_report: Instant,
    pub pose_changes: [usize; 4],
    last_render: Option<GlobalPosition>,
    moving_frames: u64,
    held_frames: u64,
    held_jump_reset_samples: u64,
    held_jump_reset_failures: u64,
    visited: std::collections::BTreeSet<ChunkId>,
    started: Instant,
}
impl Graybox {
    pub fn new(source: procedural::Source) -> Self {
        let saved = match &source {
            procedural::Source::Saved(store) => Some(store.clone()),
            _ => None,
        };
        let visual_source = source.clone();
        let (sender, jobs) = mpsc::sync_channel(1);
        let (results, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("terrain-visuals".into())
            .spawn(move || {
                while let Ok((id, cells)) = jobs.recv() {
                    if results
                        .send(visual_source.geometry(id, cells).map(|geometry| Built {
                            id,
                            cells,
                            geometry,
                        }))
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .expect("Terrain visual worker");
        let (patch_sender, jobs) = mpsc::sync_channel(1);
        let (results, patch_receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("terrain-camera".into())
            .spawn(move || {
                while let Ok(id) = jobs.recv() {
                    if results
                        .send(Patch::load(id, &source).map(Arc::new))
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .expect("Terrain collision worker");
        Self {
            render_motion: Default::default(),
            saved,
            sender,
            receiver: Mutex::new(receiver),
            pending: None,
            prepared_visuals: Default::default(),
            gpu_mesh_assets: 0,
            patch_sender,
            patch_receiver: Mutex::new(patch_receiver),
            patch_pending: None,
            patch: None,
            material: None,
            loaded: 0,
            unloaded: 0,
            patches: 0,
            probes: 0,
            max_ground_error: 0.,
            max_resident: 0,
            last_position: [0.; 3],
            frames: Vec::new(),
            last_report: Instant::now(),
            pose_changes: [0; 4],
            last_render: None,
            moving_frames: 0,
            held_frames: 0,
            held_jump_reset_samples: 0,
            held_jump_reset_failures: 0,
            visited: Default::default(),
            started: Instant::now(),
        }
    }
    pub fn airborne(&self, global: GlobalPosition) -> bool {
        self.patch
            .as_ref()
            .and_then(|p| {
                let feet = p.frame.to_local(global)?.metres();
                let (ground, _) = p.scene.ground([feet[0], feet[2]], feet[1])?;
                Some(feet[1] - theia_network::world::PAWN_Y - ground > 0.16)
            })
            .unwrap_or(false)
    }
    pub fn camera_fraction(
        &self,
        frame: &crate::floating_origin::Frame,
        target: Vec3,
        eye: Vec3,
    ) -> f32 {
        let Some(patch) = &self.patch else {
            return 1.;
        };
        let local = |p: Vec3| {
            frame
                .render
                .to_global(theia_core::coordinates::LocalPosition::new(p.to_array())?)
                .and_then(|g| patch.frame.to_local(g))
                .map(|p| p.metres())
        };
        match (local(target), local(eye)) {
            (Some(a), Some(b)) => patch.scene.camera_fraction(a, b),
            _ => 1.,
        }
    }
}
fn mesh(mut g: Geometry, cells: usize) -> Mesh {
    // Downward skirts hide LOD T-junction gaps; full-resolution collision stays
    // identical to the visible near terrain. Skirts never enter collision.
    for edge in [
        (0..=cells).collect::<Vec<_>>(),
        (0..=cells).map(|x| cells * (cells + 1) + x).collect(),
        (0..=cells).map(|z| z * (cells + 1)).collect(),
        (0..=cells).map(|z| z * (cells + 1) + cells).collect(),
    ] {
        for pair in edge.windows(2) {
            let a = pair[0] as u32;
            let b = pair[1] as u32;
            let c = g.positions.len() as u32;
            for i in [a, b] {
                let mut p = g.positions[i as usize];
                p[1] -= 4.;
                g.positions.push(p);
            }
            g.triangles.extend([[a, b, c], [b, c + 1, c]]);
        }
    }
    let colors: Vec<_> = g
        .positions
        .iter()
        .map(|p| {
            let grid = (p[0] % 16. < 0.1) || (p[2] % 16. < 0.1);
            if grid {
                [0.20, 0.24, 0.28, 1.]
            } else {
                let h = (p[1] + 36.) / 72.;
                [0.35 + h * 0.13, 0.39 + h * 0.13, 0.42 + h * 0.13, 1.]
            }
        })
        .collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, g.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(g.triangles.into_iter().flatten().collect()));
    mesh.compute_normals();
    mesh
}
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gray: ResMut<Graybox>,
) {
    gray.material = Some(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        cull_mode: None,
        ..default()
    }));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 350.,
        ..default()
    });
    commands.spawn((
        Hud,
        Text::new("Preparing graybox terrain..."),
        TextFont {
            font_size: bevy::text::FontSize::Px(16.),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            bottom: px(12),
            ..default()
        },
    ));
}
fn load_failed(world: &mut World, error: Box<dyn std::error::Error + Send + Sync>) {
    eprintln!("Terrain load failed: {error}");
    world
        .resource::<crate::network::Network>()
        .publish_chunks(Default::default());
    world.write_message(bevy::app::AppExit::Error(
        std::num::NonZeroU8::new(1).unwrap(),
    ));
}
pub fn tick(world: &mut World) {
    let state = world.resource::<crate::network::Network>().state();
    let Some(global) = state.authority_focus() else {
        return;
    };
    let owner = procedural::owner(global).unwrap();
    let desired: BTreeMap<_, _> = procedural::neighbours(owner, procedural::VISUAL_RADIUS)
        .into_iter()
        .map(|id| {
            let dist = (id.x - owner.x).abs().max((id.z - owner.z).abs());
            (id, if dist <= 2 { 32 } else { 8 })
        })
        .collect();
    let existing: Vec<_> = world
        .query::<(Entity, &Tile)>()
        .iter(world)
        .map(|(e, t)| (e, t.id, t.cells, t.mesh.clone()))
        .collect();
    for (entity, id, _, handle) in &existing {
        if !desired.contains_key(id) {
            world.despawn(*entity);
            world.resource_mut::<Assets<Mesh>>().remove(handle.id());
            world.resource_mut::<Graybox>().unloaded += 1;
        }
    }
    let built = world
        .resource::<Graybox>()
        .receiver
        .lock()
        .unwrap()
        .try_recv()
        .ok();
    if let Some(result) = built {
        let built = match result {
            Ok(built) => built,
            Err(error) => {
                load_failed(world, error);
                return;
            }
        };
        world.resource_mut::<Graybox>().pending = None;
        if desired.get(&built.id) == Some(&built.cells) {
            if let Some((entity, _, _, handle)) =
                existing.iter().find(|(_, id, _, _)| *id == built.id)
            {
                world.despawn(*entity);
                world.resource_mut::<Assets<Mesh>>().remove(handle.id());
                world.resource_mut::<Graybox>().unloaded += 1;
            }
            let handle = world
                .resource_mut::<Assets<Mesh>>()
                .add(mesh(built.geometry, built.cells));
            let material = world
                .resource::<Graybox>()
                .material
                .as_ref()
                .unwrap()
                .clone();
            let translation = world
                .resource::<crate::floating_origin::Frame>()
                .project(built.id.origin(128).unwrap());
            world.spawn((
                Tile {
                    id: built.id,
                    cells: built.cells,
                    mesh: handle.clone(),
                },
                Mesh3d(handle),
                MeshMaterial3d(material),
                Transform::from_translation(translation),
            ));
            world.resource_mut::<Graybox>().loaded += 1;
        }
    }
    let current: BTreeMap<_, _> = world
        .query::<&Tile>()
        .iter(world)
        .map(|t| (t.id, t.cells))
        .collect();
    if world.resource::<Graybox>().pending.is_none() {
        let next = desired
            .iter()
            .filter(|(id, cells)| current.get(id) != Some(cells))
            .min_by_key(|(id, _)| (id.x - owner.x).pow(2) + (id.z - owner.z).pow(2))
            .map(|(&id, &cells)| (id, cells));
        if let Some(job) = next
            && world.resource::<Graybox>().sender.try_send(job).is_ok()
        {
            world.resource_mut::<Graybox>().pending = Some(job);
        }
    }
    let patch = world
        .resource::<Graybox>()
        .patch_receiver
        .lock()
        .unwrap()
        .try_recv()
        .ok();
    if let Some(result) = patch {
        let patch = match result {
            Ok(patch) => patch,
            Err(error) => {
                load_failed(world, error);
                return;
            }
        };
        world.resource_mut::<Graybox>().patch_pending = None;
        if patch.owner == owner {
            let bodies: Vec<_> = world
                .query_filtered::<Entity, With<Body>>()
                .iter(world)
                .collect();
            for e in bodies {
                world.despawn(e);
            }
            let base = patch.frame.origin().metres();
            for collider in &patch.scene.colliders {
                let p = std::array::from_fn(|i| base[i] + f64::from(collider.center[i]));
                world.spawn((Body { global: p }, collider.bundle()));
            }
            let mut gray = world.resource_mut::<Graybox>();
            gray.patch = Some(patch);
            gray.patches += 1;
        }
    }
    {
        let mut gray = world.resource_mut::<Graybox>();
        if gray.patch.as_ref().is_none_or(|p| p.owner != owner)
            && gray.patch_pending.is_none()
            && gray.patch_sender.try_send(owner).is_ok()
        {
            gray.patch_pending = Some(owner);
        }
        gray.last_position = global.metres();
        gray.visited.insert(owner);
        gray.max_resident = gray.max_resident.max(current.len());
        assert!(current.len() <= procedural::MAX_VISUALS);
    }
    let render = world.resource::<crate::floating_origin::Frame>().render;
    for (tile, mut transform) in world.query::<(&Tile, &mut Transform)>().iter_mut(world) {
        transform.translation = Vec3::from_array(
            render
                .to_local(tile.id.origin(128).unwrap())
                .unwrap()
                .metres(),
        );
    }
    let stale: Vec<_> = world
        .query::<(Entity, &Body)>()
        .iter(world)
        .filter(|(_, b)| {
            render
                .to_local(GlobalPosition::new(b.global).unwrap())
                .is_none()
        })
        .map(|(e, _)| e)
        .collect();
    for e in stale {
        world.despawn(e);
    }
    for (body, mut transform, mut position) in world
        .query::<(&Body, &mut Transform, &mut avian3d::prelude::Position)>()
        .iter_mut(world)
    {
        let p = Vec3::from_array(
            render
                .to_local(GlobalPosition::new(body.global).unwrap())
                .unwrap()
                .metres(),
        );
        transform.translation = p;
        position.0 = p;
    }
    // Report only full-detail visual chunks with matching ready collision. Server
    // gates movement, so CPU generation cannot outrun presentation at a seam.
    let ready = world
        .resource::<Graybox>()
        .patch
        .as_ref()
        .map(|p| {
            p.ids
                .iter()
                .filter(|id| {
                    current.get(id) == Some(&32)
                        && world.resource::<Graybox>().prepared_visuals.contains(id)
                        && world
                            .resource::<crate::capture_ready::CaptureReady>()
                            .ready()
                })
                .copied()
                .collect()
        })
        .unwrap_or_default();
    world
        .resource::<crate::network::Network>()
        .publish_chunks(ready);
}
fn report(
    time: Res<Time>,
    mut gray: ResMut<Graybox>,
    tiles: Query<(&Tile, &ViewVisibility)>,
    (meshes, mut hud): (Res<Assets<Mesh>>, Query<&mut Text, With<Hud>>),
    query: avian3d::prelude::SpatialQuery,
    (frame, network): (
        Res<crate::floating_origin::Frame>,
        Res<crate::network::Network>,
    ),
    pawn: Query<(&Transform, &crate::movement::Motion), With<crate::movement::LocalPawn>>,
) {
    if std::env::var("THEIA_GRAYBOX_HOLD_JUMP_RESET").as_deref() == Ok("1")
        && gray.started.elapsed().as_secs_f64() < 23.
        && let Some(c) = network.state().correction
        && c.state.input.stream_epoch > 1
    {
        gray.held_jump_reset_samples += 1;
        let predicted = network
            .prediction
            .as_ref()
            .and_then(|p| p.pose())
            .map(|p| p.motion);
        if !c.state.pose.motion.grounded
            || !c.state.pose.motion.jump_held
            || predicted.is_some_and(|m| !m.grounded || !m.jump_held)
        {
            gray.held_jump_reset_failures += 1;
        }
    }
    let dt = time.delta_secs_f64() * 1000.;
    if gray.frames.len() < 4096 {
        gray.frames.push(dt);
    }
    if let Ok((pawn, motion)) = pawn.single() {
        let global = frame
            .render
            .to_global(
                theia_core::coordinates::LocalPosition::new(pawn.translation.to_array()).unwrap(),
            )
            .unwrap();
        if motion.0 > 1.
            && let Some(last) = gray.last_render
        {
            let a = global.metres();
            let b = last.metres();
            let distance = (a[0] - b[0]).hypot(a[2] - b[2]);
            if distance < 64. {
                gray.moving_frames += 1;
                if distance < 0.00001 {
                    gray.held_frames += 1;
                }
            }
        }
        gray.last_render = Some(global);
        if let Some(context) =
            network.movement_measurement_context(gray.patch.as_ref().map(|p| p.owner))
        {
            let at = gray.started.elapsed().as_secs_f64();
            gray.render_motion.observe(at, global, context);
        }
    }
    if let (Some(patch), Ok((pawn, _))) = (&gray.patch, pawn.single()) {
        let global = frame
            .render
            .to_global(
                theia_core::coordinates::LocalPosition::new(pawn.translation.to_array()).unwrap(),
            )
            .unwrap();
        if let Some(local) = patch.frame.to_local(global) {
            let p = local.metres();
            let expected = patch.scene.ground([p[0], p[2]], p[1]);
            let actual = query.cast_ray(
                pawn.translation,
                Dir3::NEG_Y,
                200.,
                true,
                &avian3d::prelude::SpatialQueryFilter::from_mask(theia_network::physics::GROUND),
            );
            if let (Some((ground, _)), Some(hit)) = (expected, actual) {
                let error = (hit.distance - (p[1] - ground)).abs();
                gray.max_ground_error = gray.max_ground_error.max(error);
                gray.probes += 1;
                assert!(
                    error < 0.003,
                    "Graybox live physics/render mismatch {error}"
                );
            }
        }
    }
    if gray.last_report.elapsed().as_secs_f32() < 1. {
        return;
    }
    let resident = tiles.iter().count();
    let visible = tiles.iter().filter(|(_, v)| v.get()).count();
    let mesh_bytes: usize = tiles
        .iter()
        .filter_map(|(t, _)| meshes.get(&t.mesh))
        .map(|m| {
            m.attributes()
                .map(|(_, v)| v.get_bytes().len())
                .sum::<usize>()
                + m.indices().map_or(0, |i| i.len() * 4)
        })
        .sum();
    let mean = gray.frames.iter().sum::<f64>() / gray.frames.len().max(1) as f64;
    gray.frames.sort_by(f64::total_cmp);
    let p95 = gray
        .frames
        .get(gray.frames.len() * 95 / 100)
        .copied()
        .unwrap_or(0.);
    let p = gray.last_position;
    if let Ok(mut text) = hud.single_mut() {
        **text = format!(
            "GRAYBOX | 16 × 16 km | 256 km²\nPosition {:.1}, {:.1}, {:.1} m | chunk {}, {}\nTerrain: {resident}/81 resident | {visible} visible | {} loaded / {} retired\n{:.0} FPS | mean {mean:.2} ms | p95 {p95:.2} ms | terrain mesh {:.1} MiB",
            p[0],
            p[1],
            p[2],
            (p[0] / 128.).floor(),
            (p[2] / 128.).floor(),
            gray.loaded,
            gray.unloaded,
            1000. / mean,
            mesh_bytes as f64 / 1048576.
        );
    }
    eprintln!(
        "{}",
        serde_json::json!({"graybox":{"render_motion":gray.render_motion,"saved_terrain": gray.saved.as_ref().map(|s| s.stats()),"seconds":gray.started.elapsed().as_secs_f64(),"position":p,"resident":resident,"visible":visible,"loaded":gray.loaded,"unloaded":gray.unloaded,"patches":gray.patches,"visited":gray.visited.len(),"max_resident":gray.max_resident,"mesh_bytes":mesh_bytes,"mean_ms":mean,"p95_ms":p95,"frame_count":gray.frames.len(),"mesh_assets":meshes.len(),"gpu_mesh_assets":gray.gpu_mesh_assets,"probes":gray.probes,"moving_frames":gray.moving_frames,"held_frames":gray.held_frames,"max_ground_error":gray.max_ground_error,"pose_changes":gray.pose_changes,"rebases":frame.revision}})
    );
    gray.frames.clear();
    gray.last_report = Instant::now();
}
pub fn install(app: &mut App, source: procedural::Source) {
    app.insert_resource(Graybox::new(source))
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, autoplay.after(crate::input))
        .add_systems(Last, qualification.after(report))
        .add_systems(Update, tick.after(crate::network::present))
        .add_systems(Last, report.after(crate::physics::probe));
}

/// Opt-in reproducible native qualification. Ordinary interactive input is unchanged.
fn autoplay(gray: Res<Graybox>, mut intent: ResMut<crate::movement::MoveIntent>) {
    if std::env::var("THEIA_GRAYBOX_AUTOPLAY").as_deref() != Ok("1") {
        return;
    }
    let elapsed = gray.started.elapsed().as_secs_f32();
    let continent = std::env::var("THEIA_GRAYBOX_CONTINENT").as_deref() == Ok("1");
    let t = if continent && elapsed > 16. {
        10.
    } else {
        elapsed % 20.
    };
    intent.direction = if (3. ..16.).contains(&t) {
        Vec2::X
    } else {
        Vec2::ZERO
    };
    if (3. ..7.).contains(&t) {
        let speed = std::env::var("THEIA_GRAYBOX_AUTOPLAY_SPEED")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.)
            .clamp(1., 32.);
        intent.direction /= speed;
    }
    intent.sprint = (7. ..16.).contains(&t);
    intent.jump = (12. ..13.).contains(&t)
        || (std::env::var("THEIA_GRAYBOX_HOLD_JUMP_RESET").as_deref() == Ok("1")
            && (9. ..23.).contains(&elapsed));
}
fn qualification(world: &mut World) {
    let Some(output) = std::env::var_os("THEIA_GRAYBOX_QUALIFY").map(std::path::PathBuf::from)
    else {
        return;
    };
    let gray = world.resource::<Graybox>();
    let duration = std::env::var("THEIA_GRAYBOX_QUALIFY_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(65)
        .clamp(10, 300);
    if gray.started.elapsed().as_secs() < duration {
        return;
    }
    std::fs::create_dir_all(&output).expect("Qualification directory");
    if output.join("requested").exists() {
        return;
    }
    std::fs::write(output.join("requested"), b"requested").unwrap();
    let resident = world.query::<&Tile>().iter(world).count();
    let mesh_assets = world.resource::<Assets<Mesh>>().len();
    let terrain_handles: std::collections::HashSet<_> = world
        .query::<&Tile>()
        .iter(world)
        .map(|t| t.mesh.id())
        .collect();
    let extra_meshes:Vec<_>=world.resource::<Assets<Mesh>>().iter().filter(|(id,_)|!terrain_handles.contains(id)).map(|(id,m)|serde_json::json!({"id":format!("{id:?}"),"vertices":m.count_vertices(),"indices":m.indices().map_or(0,|i|i.len())})).collect();
    let gray = world.resource::<Graybox>();
    let prediction = world
        .resource::<crate::network::Network>()
        .prediction
        .as_ref()
        .map(|p| p.stats.clone());
    let data = serde_json::json!({"render_motion":gray.render_motion,"saved_terrain":gray.saved.as_ref().map(|s| s.stats()),"held_jump_reset_samples":gray.held_jump_reset_samples,"held_jump_reset_failures":gray.held_jump_reset_failures,"prediction":prediction,"resident":resident,"max_resident":gray.max_resident,"mesh_assets":mesh_assets,"extra_meshes":extra_meshes,"gpu_mesh_assets":gray.gpu_mesh_assets,"loaded":gray.loaded,"unloaded":gray.unloaded,"visited":gray.visited.len(),"patches":gray.patches,"probes":gray.probes,"moving_frames":gray.moving_frames,"held_frames":gray.held_frames,"max_ground_error":gray.max_ground_error,"pose_changes":gray.pose_changes,"position":gray.last_position});
    std::fs::write(
        output.join("world.json"),
        serde_json::to_vec_pretty(&data).unwrap(),
    )
    .unwrap();
    world
        .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(
            output.join("scene.png"),
        ))
        .observe(
            |_: On<bevy::render::view::screenshot::ScreenshotCaptured>,
             mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}

pub struct ReadinessPlugin;
impl Plugin for ReadinessPlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(bevy::render::RenderApp)
            .add_systems(bevy::render::ExtractSchedule, render_readiness);
    }
}
fn render_readiness(
    mut main: ResMut<bevy::render::MainWorld>,
    meshes: Res<bevy::render::render_asset::RenderAssets<bevy::render::mesh::RenderMesh>>,
) {
    if !main.contains_resource::<Graybox>() {
        return;
    }
    let prepared = main
        .query::<&Tile>()
        .iter(&main)
        .filter(|tile| meshes.get(tile.mesh.id()).is_some())
        .map(|tile| tile.id)
        .collect();
    let mut gray = main.resource_mut::<Graybox>();
    gray.prepared_visuals = prepared;
    gray.gpu_mesh_assets = meshes.iter().count();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_visual_lods_have_valid_bounded_geometry_and_full_detail_matches_collision() {
        for cells in [8, 32] {
            let id = ChunkId { x: 124, z: 124 };
            let source = procedural::geometry(id, cells);
            let positions = source.positions.clone();
            let mesh = mesh(source, cells);
            assert!(mesh.count_vertices() < 1700);
            let attributes = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            assert_eq!(&attributes[..positions.len()], positions);
            assert!(
                mesh.indices()
                    .unwrap()
                    .iter()
                    .all(|i| i < mesh.count_vertices())
            );
            assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
            assert!(attributes.iter().all(|p| p.iter().all(|v| v.is_finite())));
        }
    }
}
