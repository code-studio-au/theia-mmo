//! Presentation bridge. Transport I/O runs on a bounded background worker, never in Update.
use crate::{
    movement::{LocalPawn, Motion, MoveIntent},
    scene::{NetworkHud, PawnAssets},
};
use bevy::prelude::*;
use std::{net::SocketAddr, path::PathBuf, thread::JoinHandle, time::Instant};
use theia_core::coordinates::GlobalPosition;
use theia_network::{
    client::{self, State},
    wire::{Intent, TICK_HZ},
};
use tokio::sync::watch;

#[derive(Resource)]
pub struct Network {
    input: watch::Sender<Intent>,
    chunk_ready: watch::Sender<Option<Vec<theia_core::coordinates::ChunkId>>>,
    state: watch::Receiver<State>,
    stop: watch::Sender<bool>,
    worker: Option<JoinHandle<()>>,
    latest_tick: u64,
    tick_hz: u32,
    pub smoke: Option<Smoke>,
    started: Instant,
    transport: &'static str,
    render_origin: Option<[f64; 3]>,
    local_global: Option<GlobalPosition>,
    local_track: Option<RemotePawn>,
    pub(crate) prediction: Option<theia_network::prediction_driver::Driver>,
    prediction_commands:
        Option<tokio::sync::mpsc::Sender<theia_network::prediction_wire::Commands>>,
    predicted_tick: u64,
}
pub struct Smoke {
    output: PathBuf,
    direction: [i16; 2],
    gaits: bool,
    gait_actors: usize,
    gait_stage: u8,
    jump_demo: bool,
    chunk_route: bool,
    jump_peak: f32,
    remote_jump_peak: f32,
    full_map: bool,
    map_inside: bool,
    map_goals: Option<[[f32; 2]; 6]>,
    map_samples: Vec<theia_network::wire::Player>,
    house: bool,
    camera_blocked: bool,
    camera_restored: bool,
    interior: bool,
    route: bool,
    route_stage: usize,
    stage_started: Option<Instant>,
    entered: Option<theia_network::wire::Player>,
    exited: Option<theia_network::wire::Player>,
    wall: Option<theia_network::wire::Player>,
    inside_camera: bool,
    ready: Option<Instant>,
    ready_announced: bool,
    start_gate: Option<PathBuf>,
    capture_gate: Option<PathBuf>,
    finish_gate: Option<PathBuf>,
    first: Option<theia_network::wire::Snapshot>,
    max_players: usize,
    capture_requested: bool,
}
impl Network {
    pub(crate) fn publish_chunks(&self, ids: Vec<theia_core::coordinates::ChunkId>) {
        self.chunk_ready.send_if_modified(|previous| {
            if previous.as_ref() == Some(&ids) {
                false
            } else {
                *previous = Some(ids);
                true
            }
        });
    }
    pub(crate) fn rendered_global(&self) -> Option<GlobalPosition> {
        self.local_global
    }
    pub(crate) fn state(&self) -> State {
        self.state.borrow().clone()
    }
    pub fn camera_test_blocked(&self) -> bool {
        self.smoke.as_ref().is_some_and(|s| {
            s.house
                && s.ready
                    .is_some_and(|t| (2.8..3.2).contains(&t.elapsed().as_secs_f32()))
        })
    }
    pub fn connected(&self) -> bool {
        let state = self.state.borrow();
        state.snapshot.is_some() && state.error.is_none()
    }
    pub fn from_args(
        args: &[String],
        prediction: bool,
        expected_world: [u8; 32],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let renet = args.first().is_some_and(|arg| arg.starts_with("--renet-"));
        let smoke = args
            .first()
            .is_some_and(|arg| matches!(arg.as_str(), "--network-smoke" | "--renet-smoke"));
        if (!smoke && args.len() != 3) || (smoke && args.len() != 5) {
            return Err("Use --connect <address> <public-cert.der>, --renet-connect <address> <ticket>, or the corresponding --network-smoke/--renet-smoke form with <output> <x|z>".into());
        }
        let address: SocketAddr = args[1].parse()?;
        if !address.ip().is_loopback() {
            return Err("Only loopback connections qualified in this prototype".into());
        }
        let cert = std::fs::read(&args[2])?;
        let smoke = if smoke {
            let house = args[4] == "house";
            let jump_demo = args[4].ends_with("-jump");
            let chunk_route = args[4] == "x-chunks";
            let map_inside = args[4] == "map-inside";
            let interior = matches!(
                args[4].as_str(),
                "inside" | "x-interior" | "map-inside" | "map-outside"
            );
            let route = args[4] == "inside";
            let full_map = args[4].ends_with("-map");
            let gaits = args[4].ends_with("-gaits")
                || matches!(args[4].as_str(), "x-interior" | "map-outside");
            let direction = match args[4].strip_suffix("-gaits").or_else(|| args[4].strip_suffix("-map")).or_else(|| args[4].strip_suffix("-jump")).unwrap_or(&args[4]) {
                "house" => [-1000, 0],
                "inside" => [0, -1000],
                "map-inside" => [0, -1000],
                "map-outside" => [0, 1000],
                "x-interior" => [1000, 0],
                "x"|"x-chunks" => [1000, 0],
                "z" => [0, -1000],
                _ => return Err("Smoke direction must be x, z, x-gaits, z-gaits, x-map, z-map, inside, x-interior or house".into()),
            };
            let output = PathBuf::from(&args[3]);
            std::fs::create_dir_all(&output)?;
            for name in ["network.json", "scene.png", "ready"] {
                let path = output.join(name);
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
            }
            Some(Smoke {
                output,
                direction,
                gaits,
                gait_actors: if args[4] == "map-outside" { 1 } else { 2 },
                gait_stage: 0,
                jump_demo,
                chunk_route,
                jump_peak: 0.,
                remote_jump_peak: 0.,
                full_map,
                map_inside,
                map_goals: None,
                map_samples: Vec::new(),
                house,
                camera_blocked: false,
                camera_restored: false,
                interior,
                route,
                route_stage: 0,
                stage_started: None,
                entered: None,
                exited: None,
                wall: None,
                inside_camera: false,
                ready: None,
                ready_announced: false,
                start_gate: std::env::var_os("THEIA_SMOKE_START_GATE").map(PathBuf::from),
                capture_gate: std::env::var_os("THEIA_SMOKE_CAPTURE_GATE").map(PathBuf::from),
                finish_gate: std::env::var_os("THEIA_SMOKE_FINISH_GATE").map(PathBuf::from),
                first: None,
                max_players: 0,
                capture_requested: false,
            })
        } else {
            None
        };
        let (input, inputs) = watch::channel(Intent::default());
        let (chunk_ready, ready) = watch::channel(None);
        let (prediction_commands, command_queue) = tokio::sync::mpsc::channel(8);
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let worker = std::thread::Builder::new()
            .name("theia-network".into())
            .spawn(move || {
                let result = runtime.block_on(async {
                    if prediction {
                        theia_network::renet_runtime::run_client_predicted(
                            address,
                            &cert,
                            inputs,
                            states.clone(),
                            shutdown,
                            expected_world,
                            ready,
                            command_queue,
                        )
                        .await
                    } else if renet {
                        theia_network::renet_runtime::run_client_with_chunks(
                            address,
                            &cert,
                            inputs,
                            states.clone(),
                            shutdown,
                            expected_world,
                            ready,
                        )
                        .await
                    } else {
                        client::run_in_world(
                            address,
                            &cert,
                            inputs,
                            states.clone(),
                            shutdown,
                            expected_world,
                        )
                        .await
                    }
                });
                if let Err(error) = result {
                    states.send_modify(|state| {
                        state.error = Some(error.to_string());
                        state.snapshot = None;
                    });
                }
            })?;
        Ok(Self {
            input,
            chunk_ready,
            state,
            stop,
            worker: Some(worker),
            latest_tick: 0,
            tick_hz: TICK_HZ,
            smoke,
            started: Instant::now(),
            transport: if renet { "renet" } else { "quinn" },
            render_origin: None,
            local_global: None,
            local_track: None,
            prediction: prediction.then(theia_network::prediction_driver::Driver::default),
            prediction_commands: prediction.then_some(prediction_commands),
            predicted_tick: 0,
        })
    }
    pub fn configure_map_access(
        &mut self,
        directory: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(smoke) = &mut self.smoke
            && smoke.map_inside
        {
            let report: serde_json::Value =
                serde_json::from_slice(&std::fs::read(directory.join("access.json"))?)?;
            let goals: [[f32; 2]; 6] = serde_json::from_value(report["route"].clone())?;
            if goals
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 1023.)
            {
                return Err("Invalid map-access smoke goals".into());
            }
            smoke.map_goals = Some(goals);
        }
        Ok(())
    }
}
impl Drop for Network {
    fn drop(&mut self) {
        let _ = self.stop.send(true);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Component)]
pub struct RemotePawn {
    id: u64,
    from: GlobalPosition,
    to: GlobalPosition,
    elapsed: f32,
    duration: f32,
}

impl RemotePawn {
    pub(crate) fn global_endpoints(&self) -> [GlobalPosition; 2] {
        [self.from, self.to]
    }

    fn global_sample(&self) -> GlobalPosition {
        let from = self.from.metres();
        let to = self.to.metres();
        let t = f64::from((self.elapsed / self.duration.max(1. / 120.)).clamp(0., 1.));
        GlobalPosition::new(std::array::from_fn(|i| from[i] + (to[i] - from[i]) * t)).unwrap()
    }
    fn global_in_frame(
        &self,
        frame: &crate::floating_origin::Frame,
        authority: Option<GlobalPosition>,
    ) -> Option<GlobalPosition> {
        let sample = self.global_sample();
        if frame.render.to_local(sample).is_some() {
            Some(sample)
        } else {
            // A teleport snapshot and its independent correction can arrive in
            // either order. Old interpolation must not project into a new frame.
            authority.filter(|p| frame.render.to_local(*p).is_some())
        }
    }
    fn retarget(&mut self, to: GlobalPosition, interval: f32) -> f32 {
        let a = self.to.metres();
        let b = to.metres();
        let distance = (b[0] - a[0]).hypot(b[2] - a[2]);
        let discontinuity = distance > 64. || (b[1] - a[1]).abs() > 32.;
        self.from = if discontinuity {
            to
        } else {
            self.global_sample()
        };
        self.to = to;
        self.elapsed = 0.;
        self.duration = interval.clamp(1. / 120., 0.25);
        if discontinuity {
            0.
        } else {
            distance as f32 / interval.max(1. / 120.)
        }
    }
    fn sample(&self, frame: &crate::floating_origin::Frame) -> Vec3 {
        frame
            .render
            .to_local(self.global_sample())
            .or_else(|| frame.render.to_local(self.to))
            .map(|p| Vec3::from_array(p.metres()))
            .unwrap_or(Vec3::new(0., -1000., 0.))
    }
}

pub fn send_input(
    intent: Res<MoveIntent>,
    mut network: ResMut<Network>,
    terrain: Option<Res<crate::terrain_scene::Config>>,
    visuals: Query<&crate::reference_scene::Visual>,
) {
    let mut value = Intent {
        jump: intent.jump,
        axes: [
            (intent.direction.x.clamp(-1., 1.) * 1000.) as i16,
            (intent.direction.y.clamp(-1., 1.) * 1000.) as i16,
        ],
        sprint: intent.sprint,
    };
    let state = network
        .state
        .borrow()
        .local_view()
        .expect("Validated admitted frame");
    let player = state
        .snapshot
        .as_ref()
        .and_then(|s| s.players.iter().find(|p| Some(p.id) == state.id))
        .cloned();
    if let Some(smoke) = &mut network.smoke {
        value = Intent::default();
        if let Some(ready) = smoke.ready {
            let elapsed = ready.elapsed().as_secs_f32();
            value.jump = smoke.jump_demo && (0.3..1.5).contains(&elapsed);
            if smoke.map_inside {
                if let Some(player) = player {
                    value = smoke.map_intent(player);
                }
            } else if smoke.route {
                if let Some(player) = player {
                    value = smoke.interior_intent(player);
                }
            } else if smoke.full_map {
                if elapsed < 7. {
                    value.axes = smoke.direction;
                    value.sprint = elapsed >= 1.6;
                }
            } else if smoke.house {
                value.axes = if elapsed < 1.6 {
                    [-1000, 0]
                } else if elapsed < 3.8 {
                    [0, -1000]
                } else {
                    [0, 0]
                };
            } else if !smoke.gaits {
                // Stop this short diagnostic at an observed 4 m displacement.
                // A wall-clock window varies with render cadence and transport
                // delivery, and can put the other pawn outside the screenshot.
                let initial = smoke
                    .first
                    .as_ref()
                    .and_then(|s| s.players.iter().find(|p| Some(p.id) == state.id));
                if let (Some(initial), Some(player)) = (initial, player.as_ref())
                    && Vec2::from_array(initial.xz).distance(Vec2::from_array(player.xz))
                        < if smoke.chunk_route { 6. } else { 4. }
                {
                    value.axes = smoke.direction;
                }
            } else if let (Some(first), Some(player)) = (
                smoke
                    .first
                    .as_ref()
                    .and_then(|s| s.players.iter().find(|p| Some(p.id) == state.id)),
                player.as_ref(),
            ) {
                // Exercise walk/run/stand using observed authority displacement.
                // Keep both moving phases forward: a late sampled stop may
                // overshoot, but cannot undo already demonstrated travel.
                let direction =
                    Vec2::new(f32::from(smoke.direction[0]), f32::from(smoke.direction[1])) / 1000.;
                let forward =
                    (Vec2::from_array(player.xz) - Vec2::from_array(first.xz)).dot(direction);
                // Participating skeletons must sample the phase before leaving
                // it; a heavy render frame can otherwise skip a short run.
                let sampled = |gait: usize| {
                    let mut actors = 0;
                    let mut changed = 0;
                    for visual in visuals
                        .iter()
                        .filter(|v| v.kind == 0 && !v.players.is_empty())
                    {
                        actors += 1;
                        changed += usize::from(visual.pose_changes[gait] > 0);
                    }
                    actors == 0 || changed >= smoke.gait_actors
                };
                match smoke.gait_stage {
                    0 if forward >= 4. && sampled(1) => smoke.gait_stage = 1,
                    1 if forward >= 7. && sampled(2) => smoke.gait_stage = 2,
                    _ => {}
                }
                if smoke.gait_stage < 2 {
                    value.sprint = smoke.gait_stage == 1;
                    value.axes = smoke.direction;
                }
            }
        }
        if smoke.jump_demo && terrain.is_some() {
            value.axes = [0, 0];
        }
    }
    network.input.send_replace(value);
}

impl Smoke {
    fn map_intent(&mut self, player: theia_network::wire::Player) -> Intent {
        let mut intent = Intent::default();
        let goals = self.map_goals.expect("Map access goals configured");
        let goal = goals[self.route_stage.min(5)];
        let delta = (Vec2::from_array(goal) - Vec2::from_array(player.xz)) / 0.5;
        if self.route_stage >= 6 {
            // Keep the final diagnostic waypoint settled through ordinary input.
            // A stop sent after the first arrival can leave a previous intent
            // in flight, especially while the imported scene is rendering.
            if delta.length() < 0.06 {
                return intent;
            }
        } else if self.route_stage == 2 {
            if self
                .stage_started
                .get_or_insert_with(Instant::now)
                .elapsed()
                .as_secs_f32()
                > 2.5
            {
                self.map_samples.push(player);
                self.route_stage += 1;
                return intent;
            }
        } else if delta.length() < 0.24 {
            self.map_samples.push(player);
            self.route_stage += 1;
            self.stage_started = None;
            return intent;
        }
        let direction = if self.route_stage == 2 {
            // Press into the wall while correcting sideways error against the
            // approach line. A distant point beyond the wall dilutes that
            // correction and makes contact samples depend on arrival timing.
            let inside = Vec2::from_array(goals[1]);
            let normal = (Vec2::from_array(goals[2]) - inside).normalize_or_zero();
            let tangent = Vec2::new(-normal.y, normal.x);
            let sideways = (inside - Vec2::from_array(player.xz)).dot(tangent);
            (normal + tangent * (sideways / 0.5).clamp(-1., 1.)).clamp_length_max(1.)
        } else {
            delta.clamp_length_max(1.)
        };
        intent.axes = [(direction.x * 1000.) as i16, (direction.y * 1000.) as i16];
        intent
    }
    fn interior_intent(&mut self, player: theia_network::wire::Player) -> Intent {
        let mut value = Intent::default();
        if self.route_stage >= 7 {
            return value;
        }
        if self.route_stage == 3 {
            let started = self.stage_started.get_or_insert_with(Instant::now);
            if started.elapsed().as_secs_f32() < 1.8 {
                value.axes = [1000, 0];
            } else {
                self.wall = Some(player);
                self.route_stage = 4;
            }
            return value;
        }
        let goal = match self.route_stage {
            0 => [0., 3.],
            1 | 5 => [-8.913, 3.],
            2 | 4 | 6 => [-8.913, -7.],
            _ => unreachable!(),
        };
        let delta = Vec2::new(goal[0] - player.xz[0], goal[1] - player.xz[1]);
        if delta.length() < 0.12 {
            if self.route_stage == 2 {
                self.entered = Some(player.clone());
                self.stage_started = Some(Instant::now());
            }
            if self.route_stage == 5 {
                self.exited = Some(player);
            }
            self.route_stage += 1;
        } else {
            // Intent only; the server still decides every position and height.
            // Ease into diagnostic waypoints so a final queued 30 Hz input
            // cannot overshoot a goal after the client has requested a stop.
            let direction = (delta / 0.5).clamp_length_max(1.);
            value.axes = [(direction.x * 1000.) as i16, (direction.y * 1000.) as i16];
        }
        value
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct SceneState<'w> {
    graybox: Option<Res<'w, crate::graybox::Graybox>>,
    frame: Res<'w, crate::floating_origin::Frame>,
    pipelines: Res<'w, crate::capture_ready::CaptureReady>,
    time: Res<'w, Time>,
    references: Option<Res<'w, crate::reference_scene::Config>>,
    ready: Option<Res<'w, crate::reference_scene::Ready>>,
    terrain: Option<Res<'w, crate::terrain_scene::Config>>,
    terrain_ready: Option<Res<'w, crate::terrain_scene::Ready>>,
    chunks: Option<Res<'w, crate::chunk_world::Config>>,
    chunks_ready: Option<Res<'w, crate::chunk_world::Ready>>,
}

pub fn present(
    mut commands: Commands,
    mut network: ResMut<Network>,
    assets: Res<PawnAssets>,
    mut local: Query<(&mut Transform, &mut Visibility, &mut Motion), With<LocalPawn>>,
    mut remotes: Query<(Entity, &mut RemotePawn, &mut Transform, &mut Motion), Without<LocalPawn>>,
    mut hud: Query<&mut Text, With<NetworkHud>>,
    scene: SceneState,
) {
    let SceneState {
        graybox,
        frame,
        pipelines,
        time,
        references,
        ready,
        terrain,
        terrain_ready,
        chunks,
        chunks_ready,
    } = scene;
    let raw = network.state();
    let state = raw.local_view().expect("Validated admitted frame");
    if network.render_origin != Some(frame.render.origin().metres()) {
        if let Some(global) = network.local_global
            && let Ok((mut transform, _, _)) = local.single_mut()
            && let Some(p) = frame.render.to_local(global)
        {
            transform.translation = Vec3::from_array(p.metres());
        }
        for (_, remote, mut transform, _) in &mut remotes {
            transform.translation = remote.sample(&frame);
        }
        network.render_origin = Some(frame.render.origin().metres());
    }
    if let Some(error) = &state.error {
        if let Ok(mut text) = hud.single_mut() {
            **text =
                format!("THEIA\nDisconnected: {error}\nEsc: quit; restart client to reconnect");
        }
        for (entity, _, _, _) in &remotes {
            commands.entity(entity).despawn();
        }
        if let Ok((_, mut visibility, _)) = local.single_mut() {
            *visibility = Visibility::Hidden;
        }
        if let Some(smoke) = &network.smoke {
            if smoke.capture_requested && smoke.output.join("network.json").exists() {
                // The independently scheduled peer may finish its verified run first.
                return;
            } else {
                panic!("Network smoke failed: {error}");
            }
        }
        return;
    }
    if let Some(snapshot) = &state.snapshot {
        let id = state.id.expect("Snapshot requires welcome");
        if let Ok(mut text) = hud.single_mut() {
            **text = if (terrain.is_some() && terrain_ready.is_none())
                || (chunks.is_some() && chunks_ready.is_none())
                || (references.is_some() && ready.is_none())
            {
                format!(
                    "THEIA\nPlayer {id} | connected\nLoading world and characters...\nEsc: quit"
                )
            } else {
                format!(
                    "THEIA | server authority\nPlayer {id} | {} connected | tick {}\nWASD / arrows: move | Shift: run | Space: jump | Esc: quit\nHold RMB: orbit | Wheel: zoom | Home: camera reset",
                    snapshot.players.len(),
                    snapshot.tick
                )
            };
        }
        if snapshot.tick > network.latest_tick {
            network.tick_hz = snapshot.tick_hz;
            let interval = (snapshot.tick - network.latest_tick) as f32 / snapshot.tick_hz as f32;
            network.latest_tick = snapshot.tick;
            if let Some(smoke) = &mut network.smoke {
                smoke.max_players = smoke.max_players.max(snapshot.players.len());
                if snapshot.players.len() >= 2
                    && smoke.ready.is_none()
                    && pipelines.ready()
                    && (references.is_none() || ready.is_some())
                    && (terrain.is_none() || terrain_ready.is_some())
                    && (chunks.is_none() || chunks_ready.is_some())
                {
                    if !smoke.ready_announced {
                        std::fs::write(smoke.output.join("ready"), b"ready\n")
                            .expect("Smoke readiness marker");
                        smoke.ready_announced = true;
                    }
                    if smoke.start_gate.as_ref().is_none_or(|p| p.is_file()) {
                        smoke.ready = Some(Instant::now());
                        smoke.first = Some(snapshot.clone());
                    }
                }
            }
            for (player, source) in snapshot
                .players
                .iter()
                .zip(&raw.snapshot.as_ref().unwrap().players)
            {
                let global = raw
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .global_position(source)
                    .unwrap();
                let position = frame
                    .render
                    .to_local(global)
                    .map(|p| Vec3::from_array(p.metres()))
                    .unwrap_or(Vec3::new(0., -1000., 0.));
                if player.id == id {
                    if network.prediction.is_some() {
                        if let Ok((_, mut visibility, _)) = local.single_mut() {
                            *visibility = Visibility::Inherited;
                        }
                        continue;
                    }
                    let speed = if let Some(track) = &mut network.local_track {
                        track.retarget(global, interval)
                    } else {
                        network.local_track = Some(RemotePawn {
                            id,
                            from: global,
                            to: global,
                            elapsed: 0.,
                            duration: interval,
                        });
                        0.
                    };
                    if let Ok((_, mut visibility, mut motion)) = local.single_mut() {
                        motion.0 = speed;
                        *visibility = Visibility::Inherited;
                    }
                } else if let Some((_, mut remote, _, mut motion)) = remotes
                    .iter_mut()
                    .find(|(_, remote, _, _)| remote.id == player.id)
                {
                    motion.0 = remote.retarget(global, interval);
                } else {
                    commands.spawn((
                        RemotePawn {
                            id: player.id,
                            from: global,
                            to: global,
                            elapsed: 0.,
                            duration: interval,
                        },
                        Mesh3d(assets.mesh.clone()),
                        MeshMaterial3d(assets.remote.clone()),
                        Transform::from_translation(position),
                        Motion::default(),
                    ));
                }
            }
            for (entity, remote, _, _) in &remotes {
                if !snapshot.players.iter().any(|p| p.id == remote.id) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
    let current_intent = *network.input.borrow();
    let driven = network.prediction.as_mut().and_then(|driver| {
        match driver.tick(
            &raw,
            current_intent,
            graybox.as_ref().and_then(|gray| gray.patch.as_deref()),
            Instant::now(),
        ) {
            Ok(value) => value,
            Err(error) => {
                warn!("Prediction paused: {error}");
                None
            }
        }
    });
    if let Some(driven) = driven {
        if let Some(packet) = driven.packet
            && network
                .prediction_commands
                .as_ref()
                .unwrap()
                .try_send(packet)
                .is_err()
        {
            network.prediction.as_mut().unwrap().stats.queue_drops += 1;
        }
        let mut p = driven.pose.feet.metres();
        p[1] += f64::from(theia_network::world::PAWN_Y);
        let global = GlobalPosition::new(p).unwrap();
        if network.predicted_tick != driven.tick || driven.reset || network.local_track.is_none() {
            network.predicted_tick = driven.tick;
            let interval = 1. / network.tick_hz as f32;
            let speed = if let Some(track) = &mut network.local_track {
                track.retarget(global, interval)
            } else {
                network.local_track = Some(RemotePawn {
                    id: state.id.unwrap_or(0),
                    from: global,
                    to: global,
                    elapsed: 0.,
                    duration: interval,
                });
                0.
            };
            if let Ok((_, _, mut motion)) = local.single_mut() {
                motion.0 = speed;
            }
        }
    }
    if let Some(track) = &mut network.local_track {
        track.elapsed += time.delta_secs();
        if let Some(global) = track.global_in_frame(&frame, state.authority_focus()) {
            if let Ok((mut transform, _, _)) = local.single_mut() {
                face_and_move(&mut transform, frame.project(global));
            }
            network.local_global = Some(global);
        }
    }
    for (_, mut remote, mut transform, _) in &mut remotes {
        remote.elapsed += time.delta_secs();
        let position = remote.sample(&frame);
        face_and_move(&mut transform, position);
    }
    assert!(
        network.smoke.is_none() || network.started.elapsed().as_secs() < 180,
        "Network smoke timed out"
    );
}
fn face_and_move(transform: &mut Transform, position: Vec3) {
    let direction = position - transform.translation;
    if direction.xz().length_squared() > 0.000001 {
        transform.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.z));
    }
    transform.translation = position;
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct ReferenceArt<'w, 's> {
    physics: Res<'w, crate::physics::Backend>,
    frame: Res<'w, crate::floating_origin::Frame>,
    config: Option<Res<'w, crate::reference_scene::Config>>,
    visuals: Query<'w, 's, &'static crate::reference_scene::Visual>,
    houses: Query<'w, 's, Entity, With<crate::reference_scene::House>>,
    pipelines: Res<'w, crate::capture_ready::CaptureReady>,
    terrain: Option<Res<'w, crate::terrain_scene::Config>>,
    chunks: Option<Res<'w, crate::chunk_world::Config>>,
    chunk_roots: Query<'w, 's, &'static crate::chunk_world::Root>,
    chunk_stream: Option<Res<'w, crate::chunk_world::Stream>>,
    shared_images: crate::shared_images::Bindings<'w, 's>,
    material_profiles: crate::runtime_materials::Evidence<'w, 's>,
    camera: Query<'w, 's, &'static Transform, With<Camera3d>>,
    camera_projection:
        Query<'w, 's, (&'static Projection, &'static GlobalTransform), With<Camera3d>>,
    scenery_stats: Res<'w, crate::camera::SceneryCameraStats>,
}

pub fn capture(
    mut commands: Commands,
    mut network: ResMut<Network>,
    local: Query<(&Transform, &Visibility), With<LocalPawn>>,
    remotes: Query<(&RemotePawn, &Transform)>,
    adapter: Res<bevy::render::renderer::RenderAdapterInfo>,
    mut exit: MessageWriter<bevy::app::AppExit>,
    art: ReferenceArt,
) {
    use bevy::render::view::window::screenshot::{Screenshot, ScreenshotCaptured};
    let raw = network.state.borrow().clone();
    let state = raw.local_view().expect("Validated admitted frame");
    let transport = network.transport;
    let raw_snapshot = raw.snapshot;
    let Some(smoke) = &mut network.smoke else {
        return;
    };
    if smoke.jump_demo
        && let Some(initial) = &smoke.first
    {
        if let Ok((pose, _)) = local.single()
            && let Some(base) = initial.players.iter().find(|p| Some(p.id) == state.id)
        {
            smoke.jump_peak = smoke
                .jump_peak
                .max(art.frame.content_point(pose.translation).y - base.y);
        }
        for (actor, pose) in &remotes {
            if let Some(base) = initial.players.iter().find(|p| p.id == actor.id) {
                smoke.remote_jump_peak = smoke
                    .remote_jump_peak
                    .max(art.frame.content_point(pose.translation).y - base.y);
            }
        }
    }
    if smoke.house
        && let Some(ready) = smoke.ready
    {
        let elapsed = ready.elapsed().as_secs_f32();
        if let (Ok((local, visibility)), Ok(camera)) = (local.single(), art.camera.single()) {
            let distance = camera.translation.distance(local.translation);
            if (2.85..3.15).contains(&elapsed) && distance < 1. && *visibility == Visibility::Hidden
            {
                smoke.camera_blocked = true;
            }
            if elapsed > 3.4 && distance > 2. && *visibility == Visibility::Inherited {
                smoke.camera_restored = true;
            }
        }
    }
    if smoke.capture_requested {
        // Keep this peer alive while the other client completes its own capture.
        if smoke.finish_gate.as_ref().map_or_else(
            || {
                smoke.ready.is_some_and(|t| {
                    t.elapsed().as_secs_f32()
                        > if smoke.interior {
                            30.
                        } else if smoke.full_map {
                            20.
                        } else {
                            9.
                        }
                })
            },
            |p| p.is_file(),
        ) && smoke.output.join("network.json").exists()
        {
            exit.write(bevy::app::AppExit::Success);
        }
        return;
    }
    if smoke.route
        && smoke.route_stage >= 3
        && let (Ok((local, _)), Ok(camera)) = (local.single(), art.camera.single())
    {
        let distance = camera.translation.distance(local.translation);
        if art.frame.content_point(local.translation).z < -5.
            && camera.translation.y < local.translation.y + 2.
            && distance > 1.
            && distance < 10.
        {
            smoke.inside_camera = true;
        }
    }
    if !smoke.ready.is_some_and(|t| {
        t.elapsed().as_secs_f32()
            > if smoke.interior {
                25.
            } else if smoke.full_map {
                13.
            } else {
                4.
            }
    }) {
        return;
    }
    if !art.pipelines.ready() {
        return;
    }
    if smoke.gaits && smoke.gait_stage < 2 {
        return;
    }
    if smoke.chunk_route
        && (art
            .chunk_stream
            .as_ref()
            .is_none_or(|s| !s.diagnostic_ready())
            || art
                .chunks
                .as_ref()
                .is_none_or(|c| !state.chunk_permission_confirmed(&c.active.loaded())))
    {
        return;
    }
    let reference_visuals = if let Some(config) = &art.config {
        let humans = art
            .visuals
            .iter()
            .filter(|v| v.kind == 0 && !v.players.is_empty())
            .count();
        let wolves = art
            .visuals
            .iter()
            .filter(|v| v.kind == 1 && !v.players.is_empty())
            .count();
        let environment_count = usize::from(!config.players_only);
        if humans != 2
            || wolves != environment_count
            || art.houses.iter().count() != environment_count
        {
            return;
        }
        Some(
            serde_json::json!({"humans":humans,"wolves":wolves,"houses":art.houses.iter().count()}),
        )
    } else {
        None
    };
    let snapshot = state.snapshot.as_ref().expect("Connected snapshot");
    assert_eq!(
        snapshot.players.len(),
        2,
        "Both clients must remain connected"
    );
    assert_eq!(remotes.iter().count(), 1, "One rendered remote required");
    let id = state.id.unwrap();
    let first = smoke.first.as_ref().unwrap();
    let initial = first.players.iter().find(|p| p.id == id).unwrap().xz;
    let final_position = snapshot.players.iter().find(|p| p.id == id).unwrap().xz;
    let axis = usize::from(smoke.direction[1] != 0);
    if smoke.jump_demo && art.terrain.is_some() {
        assert!(
            smoke.jump_peak > 1. && smoke.remote_jump_peak > 1.,
            "Stationary map jump must be observed in both actual render poses"
        );
        assert!(
            Vec2::from_array(final_position).distance(Vec2::from_array(initial)) < 0.01,
            "Stationary jump diagnostic must not walk"
        );
    } else {
        assert!(
            (final_position[axis] - initial[axis]).abs() > 3.,
            "Server movement must be observed: initial={initial:?}, final={final_position:?}"
        );
    }
    if let Some(gate) = &smoke.capture_gate {
        let marker = smoke.output.join("capture-ready");
        if !marker.is_file() {
            std::fs::write(marker, b"ready\n").expect("Smoke capture readiness marker");
        }
        if !gate.is_file() {
            return;
        }
    }
    let report = serde_json::json!({"schema":1,"completed":true,"offline":false,"id":id,
        "initial":first,"final":snapshot,"max_players":smoke.max_players,
        "local_render_position":local.single().unwrap().0.translation.to_array(),
        "local_visible":*local.single().unwrap().1 != Visibility::Hidden,
        "gait_diagnostic":smoke.gaits.then_some(serde_json::json!({"stage":smoke.gait_stage,"walk_goal_metres":4.,"run_goal_metres":7.,"pose_gated":true,"actors":smoke.gait_actors})),
        "remote_render_positions":remotes.iter().map(|(r,t)|serde_json::json!({"id":r.id,"position":t.translation.to_array()})).collect::<Vec<_>>(),
        "gpu":{"name":adapter.name,"backend":format!("{:?}",adapter.backend)},
        "reference_visuals":reference_visuals,
        "interior": smoke.route.then_some(serde_json::json!({"stage":smoke.route_stage,"entered":smoke.entered,"exited":smoke.exited,"wall":smoke.wall,"camera_inside":smoke.inside_camera})),
        "static_collision": smoke.house.then_some(serde_json::json!({"camera_blocked":smoke.camera_blocked,"camera_restored":smoke.camera_restored})),
        "human_animation":art.visuals.iter().filter(|v|v.kind==0).map(|v|serde_json::json!({"owner":format!("{:?}",v.owner),"pose_changes":v.pose_changes})).collect::<Vec<_>>(),
        "map_access":smoke.map_inside.then_some(serde_json::json!({"stage":smoke.route_stage,"samples":smoke.map_samples})),
        "scenery_camera":{"checks":art.scenery_stats.checks,"retractions":art.scenery_stats.retractions,"distance":art.camera.single().unwrap().translation.distance(local.single().unwrap().0.translation)},
        "terrain_world":art.terrain.as_ref().map(|t|t.terrain.identity),
        "clip_from_world":art.camera_projection.single().ok().map(|(p,t)|(p.get_clip_from_view()*t.to_matrix().inverse()).to_cols_array()),
        "chunk_world":art.chunks.as_ref().map(|c|serde_json::json!({"identity":c.data.identity(),"loaded":c.active.loaded().iter().map(|id|[id.x,id.z]).collect::<Vec<_>>(),"roots":art.chunk_roots.iter().map(|r|[r.0.x,r.0.z]).collect::<Vec<_>>(),"report":state.chunk_report,"ack":state.chunk_ack,"offer":state.chunk_offer,"generation":c.active.generation(),"stream":art.chunk_stream.as_ref().map(|s|s.report()),"authority_confirmed":state.chunk_permission_confirmed(&c.active.loaded()),"shared_images":art.shared_images.report(),"material_profiles":art.material_profiles.report()})),
        "jump":smoke.jump_demo.then_some(serde_json::json!({"local_peak":smoke.jump_peak,"remote_peak":smoke.remote_jump_peak})),
        "content_package":art.terrain.as_ref().and_then(|t|t.terrain.package.as_ref()).map(|p|serde_json::json!({"content_id":p.content_id().as_str(),"revision":p.revision()})),
        "pipelines":{"pending":art.pipelines.pending,"failed":art.pipelines.failed,"compiled":art.pipelines.compiled,"clear_frames":art.pipelines.clear_frames},
        "physics_backend":{"engine":"avian","revision":theia_network::physics::AVIAN_REVISION,
            "static_colliders":art.physics.static_colliders,
            "frame_revision":art.physics.frame_revision,"probes":art.physics.probes,"max_ground_error":art.physics.max_ground_error,"generation":art.physics.generation,"probed_generation":art.physics.probed_generation,"residency_probes":art.physics.residency_probes},
        "transport":transport,
        "transport_metrics":state.transport,
        "coordinates":{
            "epoch":state.welcome.as_ref().unwrap().epoch,
            "content_origin":state.welcome.as_ref().unwrap().origin,
            "query_origin":art.frame.content.origin().metres(),
            "render_origin":art.frame.render.origin().metres(),"render_rebases":art.frame.revision,
            "wire_origin":raw_snapshot.as_ref().unwrap().origin,
            "wire_rebases":state.wire_rebases,
            "global_players":raw_snapshot.as_ref().unwrap().players.iter().map(|p| {
                serde_json::json!({"id":p.id,"position":raw_snapshot.as_ref().unwrap().global_position(p).unwrap().metres()})
            }).collect::<Vec<_>>()
        },
        "scope":"actual encrypted authority and two Bevy clients; Avian world queries; no prediction, NPC gameplay, combat or MMO capacity claim"});
    let output = smoke.output.clone();
    smoke.capture_requested = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(move |event: On<ScreenshotCaptured>| {
            event
                .image
                .clone()
                .try_into_dynamic()
                .unwrap()
                .save(output.join("scene.png"))
                .unwrap();
            std::fs::write(
                output.join("network.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
        });
}

#[cfg(test)]
mod rebase_tests {
    use super::*;
    #[test]
    fn interpolation_preserves_global_path_and_camera_delta_across_rebases() {
        use crate::floating_origin::Frame;
        use theia_core::coordinates::LocalPosition;
        let mut frame = Frame::default();
        frame.span = 2.;
        frame.admit(&theia_network::wire::Welcome {
            id: 1,
            epoch: 1,
            world: [0; 32],
            origin: [1e9 + 0.01, 1e9 + 0.02, -1e9 - 0.03],
        });
        let from = frame
            .content
            .to_global(LocalPosition::new([3., 0.9, -4.]).unwrap())
            .unwrap();
        let to = frame
            .content
            .to_global(LocalPosition::new([3.5, 0.9, -4.25]).unwrap())
            .unwrap();
        let remote = RemotePawn {
            id: 2,
            from,
            to,
            elapsed: 1. / 60.,
            duration: 1. / 30.,
        };
        let before = remote.sample(&frame);
        let global_before = frame
            .render
            .to_global(LocalPosition::new(before.to_array()).unwrap())
            .unwrap();
        frame.focus(to);
        assert!(frame.revision > 0);
        let after = remote.sample(&frame);
        let global_after = frame
            .render
            .to_global(LocalPosition::new(after.to_array()).unwrap())
            .unwrap();
        for (a, b) in global_before
            .metres()
            .into_iter()
            .zip(global_after.metres())
        {
            assert!((a - b).abs() < 0.000002);
        }
        let camera_before = crate::camera::Orbit::default().unobstructed_eye(before) - before;
        let camera_after = crate::camera::Orbit::default().unobstructed_eye(after) - after;
        assert!(camera_before.distance(camera_after) < 0.000002);
    }
}

#[cfg(test)]
mod smoothing_tests {
    use super::*;
    fn p(x: f64) -> GlobalPosition {
        GlobalPosition::new([x, 1., 8000.]).unwrap()
    }
    #[test]
    fn thirty_hz_authority_is_presented_at_every_120_hz_render_frame() {
        let mut track = RemotePawn {
            id: 1,
            from: p(8000.),
            to: p(8000.),
            elapsed: 0.,
            duration: 1. / 30.,
        };
        let mut previous = 8000.;
        for frame in 0..120 {
            if frame % 4 == 0 {
                track.retarget(p(8000. + f64::from(frame / 4 + 1) * 8. / 30.), 1. / 30.);
            }
            track.elapsed += 1. / 120.;
            let current = track.global_sample().metres()[0];
            assert!(
                (current - previous - 8. / 120.).abs() < 0.000001,
                "Held or stepped render pose at frame {frame}"
            );
            previous = current;
        }
    }
    #[test]
    fn skipped_snapshot_intervals_and_teleports_use_explicit_durations_and_snap() {
        let mut track = RemotePawn {
            id: 1,
            from: p(8000.),
            to: p(8000.),
            elapsed: 0.,
            duration: 1. / 30.,
        };
        track.retarget(p(8001.), 2. / 30.);
        track.elapsed = 1. / 30.;
        assert!((track.global_sample().metres()[0] - 8000.5).abs() < 1e-6);
        assert_eq!(track.retarget(p(100.), 1. / 30.), 0.);
        assert_eq!(track.global_sample(), p(100.));
        let mut transform =
            Transform::from_xyz(1., 1., 1.).with_rotation(Quat::from_rotation_y(0.7));
        let rotation = transform.rotation;
        face_and_move(&mut transform, Vec3::new(1., 2., 1.));
        assert_eq!(
            transform.rotation, rotation,
            "Vertical jump must not change yaw"
        );
    }
    #[test]
    fn teleport_snapshot_before_correction_does_not_project_old_interpolation_into_new_frame() {
        let old = GlobalPosition::new([8000., 1., 8000.]).unwrap();
        let new = GlobalPosition::new([15800., 25., 180.]).unwrap();
        let mut frame = crate::floating_origin::Frame::default();
        frame.focus_large(new);
        let mut track = RemotePawn {
            id: 1,
            from: old,
            to: old,
            elapsed: 0.,
            duration: 1. / 30.,
        };
        assert!(frame.render.to_local(old).is_none());
        assert_eq!(track.global_in_frame(&frame, Some(new)), Some(new));
        assert!(track.global_in_frame(&frame, None).is_none());
        track.retarget(new, 1. / 30.);
        assert_eq!(track.global_in_frame(&frame, Some(old)), Some(new));
    }
}
