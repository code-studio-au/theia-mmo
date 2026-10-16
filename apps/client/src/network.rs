//! Presentation bridge. QUIC I/O runs on a bounded background worker, never in Update.
use crate::{
    movement::{LocalPawn, Motion, MoveIntent},
    scene::{NetworkHud, PawnAssets},
};
use bevy::prelude::*;
use std::{net::SocketAddr, path::PathBuf, thread::JoinHandle, time::Instant};
use theia_network::{
    client::{self, State},
    wire::{Intent, TICK_HZ},
};
use tokio::sync::watch;

#[derive(Resource)]
pub struct Network {
    input: watch::Sender<Intent>,
    state: watch::Receiver<State>,
    stop: watch::Sender<bool>,
    worker: Option<JoinHandle<()>>,
    latest_tick: u64,
    pub smoke: Option<Smoke>,
    started: Instant,
}
pub struct Smoke {
    output: PathBuf,
    direction: [i16; 2],
    ready: Option<Instant>,
    first: Option<theia_network::wire::Snapshot>,
    max_players: usize,
    capture_requested: bool,
}
impl Network {
    pub fn connected(&self) -> bool {
        let state = self.state.borrow();
        state.snapshot.is_some() && state.error.is_none()
    }
    pub fn from_args(
        args: &[String],
        expected_world: [u8; 32],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let smoke = args.first().is_some_and(|arg| arg == "--network-smoke");
        if (!smoke && args.len() != 3) || (smoke && args.len() != 5) {
            return Err("Use --connect <address> <public-cert.der> or --network-smoke <address> <public-cert.der> <output> <x|z>".into());
        }
        let address: SocketAddr = args[1].parse()?;
        if !address.ip().is_loopback() {
            return Err("Only loopback connections qualified in this prototype".into());
        }
        let cert = std::fs::read(&args[2])?;
        let smoke = if smoke {
            let direction = match args[4].as_str() {
                "x" => [1000, 0],
                "z" => [0, -1000],
                _ => return Err("Smoke direction must be x or z".into()),
            };
            let output = PathBuf::from(&args[3]);
            std::fs::create_dir_all(&output)?;
            for name in ["network.json", "scene.png"] {
                let path = output.join(name);
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
            }
            Some(Smoke {
                output,
                direction,
                ready: None,
                first: None,
                max_players: 0,
                capture_requested: false,
            })
        } else {
            None
        };
        let (input, inputs) = watch::channel(Intent::default());
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let worker = std::thread::Builder::new()
            .name("theia-network".into())
            .spawn(move || {
                if let Err(error) = runtime.block_on(client::run_in_world(
                    address,
                    &cert,
                    inputs,
                    states.clone(),
                    shutdown,
                    expected_world,
                )) {
                    states.send_modify(|state| {
                        state.error = Some(error.to_string());
                        state.snapshot = None;
                    });
                }
            })?;
        Ok(Self {
            input,
            state,
            stop,
            worker: Some(worker),
            latest_tick: 0,
            smoke,
            started: Instant::now(),
        })
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
    from: Vec3,
    to: Vec3,
    elapsed: f32,
}

pub fn send_input(intent: Res<MoveIntent>, network: Res<Network>) {
    let mut value = Intent {
        axes: [
            (intent.direction.x.clamp(-1., 1.) * 1000.) as i16,
            (intent.direction.y.clamp(-1., 1.) * 1000.) as i16,
        ],
        sprint: intent.sprint,
    };
    if let Some(smoke) = &network.smoke {
        value = Intent::default();
        if smoke.ready.is_some_and(|t| t.elapsed().as_secs_f32() < 1.) {
            value.axes = smoke.direction;
        }
    }
    network.input.send_replace(value);
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct SceneState<'w> {
    time: Res<'w, Time>,
    references: Option<Res<'w, crate::reference_scene::Config>>,
    ready: Option<Res<'w, crate::reference_scene::Ready>>,
    terrain: Option<Res<'w, crate::terrain_scene::Config>>,
    terrain_ready: Option<Res<'w, crate::terrain_scene::Ready>>,
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
        time,
        references,
        ready,
        terrain,
        terrain_ready,
    } = scene;
    let state = network.state.borrow().clone();
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
            **text = format!(
                "THEIA | server authority\nPlayer {id} | {} connected | tick {}\nWASD / arrows: move | Shift: run | Esc: quit\nHold RMB: orbit | Wheel: zoom | Home: camera reset",
                snapshot.players.len(),
                snapshot.tick
            );
        }
        if snapshot.tick > network.latest_tick {
            let interval = (snapshot.tick - network.latest_tick) as f32 / TICK_HZ as f32;
            network.latest_tick = snapshot.tick;
            if let Some(smoke) = &mut network.smoke {
                smoke.max_players = smoke.max_players.max(snapshot.players.len());
                if snapshot.players.len() >= 2
                    && smoke.ready.is_none()
                    && (references.is_none() || ready.is_some())
                    && (terrain.is_none() || terrain_ready.is_some())
                {
                    smoke.ready = Some(Instant::now());
                    smoke.first = Some(snapshot.clone());
                }
            }
            for player in &snapshot.players {
                let position = Vec3::new(player.xz[0], player.y, player.xz[1]);
                if player.id == id {
                    if let Ok((mut transform, mut visibility, mut motion)) = local.single_mut() {
                        motion.0 = position.xz().distance(transform.translation.xz()) / interval;
                        *visibility = Visibility::Inherited;
                        face_and_move(&mut transform, position);
                    }
                } else if let Some((_, mut remote, transform, mut motion)) = remotes
                    .iter_mut()
                    .find(|(_, remote, _, _)| remote.id == player.id)
                {
                    motion.0 = position.xz().distance(remote.to.xz()) / interval;
                    remote.from = transform.translation;
                    remote.to = position;
                    remote.elapsed = 0.;
                } else {
                    commands.spawn((
                        RemotePawn {
                            id: player.id,
                            from: position,
                            to: position,
                            elapsed: 0.,
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
    for (_, mut remote, mut transform, _) in &mut remotes {
        remote.elapsed += time.delta_secs();
        let position = remote
            .from
            .lerp(remote.to, (remote.elapsed * TICK_HZ as f32).min(1.));
        face_and_move(&mut transform, position);
    }
    assert!(
        network.smoke.is_none() || network.started.elapsed().as_secs() < 90,
        "Network smoke timed out"
    );
}
fn face_and_move(transform: &mut Transform, position: Vec3) {
    let direction = position - transform.translation;
    if direction.length_squared() > 0.000001 {
        transform.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.z));
    }
    transform.translation = position;
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct ReferenceArt<'w, 's> {
    config: Option<Res<'w, crate::reference_scene::Config>>,
    visuals: Query<'w, 's, &'static crate::reference_scene::Visual>,
    houses: Query<'w, 's, Entity, With<crate::reference_scene::House>>,
    pipelines: Res<'w, crate::capture_ready::CaptureReady>,
    terrain: Option<Res<'w, crate::terrain_scene::Config>>,
}

pub fn capture(
    mut commands: Commands,
    mut network: ResMut<Network>,
    local: Query<&Transform, With<LocalPawn>>,
    remotes: Query<(&RemotePawn, &Transform)>,
    adapter: Res<bevy::render::renderer::RenderAdapterInfo>,
    mut exit: MessageWriter<bevy::app::AppExit>,
    art: ReferenceArt,
) {
    use bevy::render::view::window::screenshot::{Screenshot, ScreenshotCaptured};
    let state = network.state.borrow().clone();
    let Some(smoke) = &mut network.smoke else {
        return;
    };
    if smoke.capture_requested {
        // Keep this peer alive while the other client completes its own capture.
        if smoke.ready.is_some_and(|t| t.elapsed().as_secs_f32() > 9.)
            && smoke.output.join("network.json").exists()
        {
            exit.write(bevy::app::AppExit::Success);
        }
        return;
    }
    if !smoke.ready.is_some_and(|t| t.elapsed().as_secs_f32() > 4.) {
        return;
    }
    if !art.pipelines.ready() {
        return;
    }
    let reference_visuals = if art.config.is_some() {
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
        if humans != 2 || wolves != 1 || art.houses.iter().count() != 1 {
            return;
        }
        Some(
            serde_json::json!({"humans":humans,"wolves":wolves,"houses":art.houses.iter().count()}),
        )
    } else {
        None
    };
    let snapshot = state.snapshot.expect("Connected snapshot");
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
    assert!(
        (final_position[axis] - initial[axis]).abs() > 3.,
        "Server movement must be observed"
    );
    let report = serde_json::json!({"schema":1,"completed":true,"offline":false,"id":id,
        "initial":first,"final":snapshot,"max_players":smoke.max_players,
        "local_render_position":local.single().unwrap().translation.to_array(),
        "remote_render_positions":remotes.iter().map(|(r,t)|serde_json::json!({"id":r.id,"position":t.translation.to_array()})).collect::<Vec<_>>(),
        "gpu":{"name":adapter.name,"backend":format!("{:?}",adapter.backend)},
        "reference_visuals":reference_visuals,
        "terrain_world":art.terrain.as_ref().map(|t|t.terrain.identity),
        "pipelines":{"pending":art.pipelines.pending,"failed":art.pipelines.failed,"compiled":art.pipelines.compiled,"clear_frames":art.pipelines.clear_frames},
        "scope":"actual QUIC authority and two Bevy clients; optional reference art; no prediction, NPC gameplay, combat or MMO capacity claim"});
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
