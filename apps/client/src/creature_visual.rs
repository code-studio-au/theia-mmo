//! Opt-in wolf art. Authority remains in the existing creature/combat protocols.
use bevy::{
    asset::{
        AssetApp, LoadState,
        io::{AssetSourceBuilder, file::FileAssetReader},
    },
    camera::primitives::Aabb,
    prelude::*,
};
use std::{path::PathBuf, time::Instant};
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};
use theia_network::simulation::combat::Event;

#[derive(Clone, Copy)]
struct AttackCue {
    serial: u64,
    remaining: f32,
    target: Option<GlobalPosition>,
    remote: bool,
}
/// Public timing takes precedence, including authoritative cancellation. An old
/// owner-private frame cannot restore an attack after the public stream clears it.
fn attack_cue(state: &theia_network::client::State, creature: EntityKey) -> Option<AttackCue> {
    if let Some(public) = &state.public_creature_attack {
        let observation = public.current()?;
        if observation.creature != creature {
            return None;
        }
        let tick = observation
            .tick
            .max(state.snapshot.as_ref().map_or(0, |s| s.tick))
            .max(state.correction.map_or(0, |c| c.state.input.tick));
        let pending = public.active_at(tick)?;
        let own = state.id == Some(pending.target.entity.value())
            && state
                .welcome
                .as_ref()
                .is_some_and(|w| w.epoch == pending.target.epoch.value());
        let target = if own {
            state.authority_focus()
        } else {
            state
                .snapshot
                .as_ref()
                .filter(|s| s.epoch == pending.target.epoch.value())
                .and_then(|s| {
                    s.players
                        .iter()
                        .find(|p| p.id == pending.target.entity.value())
                        .and_then(|p| s.global_position(p))
                })
        };
        return Some(AttackCue {
            serial: pending.completes,
            remaining: (pending.completes - tick) as f32 / public.rate().hz() as f32,
            target,
            remote: !own,
        });
    }
    let private = state.player_life.as_ref()?;
    let observation = private.current()?;
    let pending = observation
        .windup
        .filter(|w| w.creature == creature && w.completes > observation.tick)?;
    Some(AttackCue {
        serial: pending.completes,
        remaining: (pending.completes - observation.tick) as f32 / private.rate().hz() as f32,
        target: state.authority_focus(),
        remote: false,
    })
}

const NAMES: [&str; 6] = ["Idle", "Walk", "Run", "Attack", "Hit", "Death"];
const LIMIT: usize = theia_network::targeting::MAX_TARGETS;
const CORPSE_SECONDS: f64 = 15.;

#[derive(Resource)]
struct Loading {
    model: Handle<Gltf>,
    started: Instant,
}
#[derive(Resource)]
struct Catalog {
    scene: Handle<WorldAsset>,
    graph: Handle<AnimationGraph>,
    nodes: Vec<AnimationNodeIndex>,
    durations: Vec<f32>,
}
#[derive(Clone, Copy)]
struct Cue {
    key: EntityKey,
    position: GlobalPosition,
    dead: bool,
    serial: u64,
    until: f64,
}
#[derive(Resource)]
pub(crate) struct Pending {
    started: Instant,
    cues: Vec<Cue>,
    serial: u64,
    pub metrics: Metrics,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub attached: u64,
    pub players: u64,
    pub transitions: [u64; 6],
    pub moving_frames: u64,
    pub dead_frames: u64,
    pub attack_frames: u64,
    pub public_attack_frames: u64,
    pub remote_victim_frames: u64,
    pub hit_frames: u64,
    pub dropped_cues: u64,
    pub max_cues: usize,
    pub max_visuals: usize,
}
impl Default for Pending {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            cues: Vec::new(),
            serial: 0,
            metrics: Metrics::default(),
        }
    }
}
impl Pending {
    fn now(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }
    pub(crate) fn corpse(&self, key: EntityKey) -> Option<GlobalPosition> {
        self.cues
            .iter()
            .find(|c| c.key == key && c.dead && c.until > self.now())
            .map(|c| c.position)
    }
    pub(crate) fn offer(&mut self, event: &Event) {
        let now = self.now();
        self.offer_at(event, now);
    }
    fn offer_at(&mut self, event: &Event, now: f64) {
        let (key, position, dead, seconds) = match event {
            Event::AbilityDamage(fact)
                if fact.result.damage.health_lost > 0 && fact.remaining > 0 =>
            {
                (fact.target, fact.position, false, 0.7)
            }
            Event::WhiteMelee(fact)
                if fact.target_generation.is_none()
                    && fact.result.health_lost > 0
                    && fact.remaining > 0 =>
            {
                (fact.target, fact.position, false, 0.7)
            }
            Event::PhysicalDamage(fact)
                if fact.target_generation.is_none() && fact.remaining > 0 =>
            {
                (fact.target, fact.position, false, 0.7)
            }
            Event::Damage {
                target,
                position,
                remaining,
                ..
            }
            | Event::MeleeDamage {
                target,
                position,
                remaining,
                ..
            } if *remaining > 0 => (*target, *position, false, 0.7),
            Event::Died {
                target, position, ..
            } => (*target, *position, true, CORPSE_SECONDS),
            Event::Respawned { previous, .. } => {
                self.cues.retain(|c| c.key != *previous);
                return;
            }
            _ => return,
        };
        self.cues.retain(|c| c.until > now);
        if self.cues.iter().any(|c| c.key == key && c.dead) {
            return;
        }
        self.cues.retain(|c| c.key != key);
        if self.cues.len() == LIMIT {
            self.metrics.dropped_cues += 1;
            if dead {
                let victim = self.cues.iter().position(|c| !c.dead).unwrap_or(0);
                self.cues.remove(victim);
            } else {
                return;
            }
        }
        self.serial = self.serial.saturating_add(1);
        self.cues.push(Cue {
            key,
            position,
            dead,
            serial: self.serial,
            until: now + seconds,
        });
        self.metrics.max_cues = self.metrics.max_cues.max(self.cues.len());
    }
}
#[derive(Default)]
struct Locomotion {
    last_moved: f64,
    speed: f32,
}
impl Locomotion {
    fn observe(&mut self, measured: f32, now: f64) -> f32 {
        if measured > 0.05 {
            self.last_moved = now;
            self.speed = measured;
        } else if now - self.last_moved > 0.2 {
            self.speed = 0.;
        }
        self.speed
    }
}
#[derive(Component)]
struct Visual {
    owner: Entity,
    previous: GlobalPosition,
    locomotion: Locomotion,
    players: Vec<Entity>,
    active: Option<(usize, u64)>,
}
#[derive(Component)]
struct Attached;
#[derive(Component)]
struct SelectionRing;

pub(crate) fn parse(
    args: &mut Vec<String>,
    enabled: bool,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let Some(i) = args.iter().position(|s| s == "--creature-asset") else {
        return Ok(None);
    };
    args.remove(i);
    if i >= args.len() || !enabled {
        return Err("--creature-asset needs a GLB and predicted creature combat".into());
    }
    let path = PathBuf::from(args.remove(i)).canonicalize()?;
    if path.file_name().and_then(|s| s.to_str()).is_none()
        || path.extension().and_then(|s| s.to_str()) != Some("glb")
        || !path.is_file()
        || path.metadata()?.len() > 8 * 1024 * 1024
    {
        return Err("Creature asset must be a GLB of at most 8 MiB".into());
    }
    Ok(Some(path))
}

/// Called before DefaultPlugins so the explicitly selected local asset source exists.
pub(crate) fn install(app: &mut App, path: PathBuf) {
    let directory = path.parent().unwrap().to_owned();
    let filename = path.file_name().unwrap().to_str().unwrap().to_owned();
    app.register_asset_source(
        "creature",
        AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(directory.clone()))),
    );
    app.init_resource::<Pending>()
        .add_systems(
            Startup,
            move |mut commands: Commands, server: Res<AssetServer>| {
                commands.insert_resource(Loading {
                    model: server.load(format!("creature://{filename}")),
                    started: Instant::now(),
                });
            },
        )
        .add_systems(
            Update,
            prepare
                .after(crate::combat::Presentation)
                .before(crate::targeting::Presentation),
        )
        .add_systems(Update, update.after(crate::targeting::Presentation));
}
fn prepare(mut pending: ResMut<Pending>, network: Res<crate::network::Network>) {
    let state = network.state();
    let epoch = state.targeting.as_ref().map(|s| s.epoch);
    let now = pending.now();
    if !network.connected() || state.error.is_some() {
        pending.cues.clear();
    } else {
        pending
            .cues
            .retain(|c| c.until > now && Some(c.key.epoch.value()) == epoch);
    }
}
fn load(world: &mut World) -> Result<bool, String> {
    if world.contains_resource::<Catalog>() {
        return Ok(true);
    }
    let loading = world.resource::<Loading>();
    let server = world.resource::<AssetServer>();
    if matches!(server.load_state(loading.model.id()), LoadState::Failed(_))
        || loading.started.elapsed().as_secs() > 60
    {
        return Err("Wolf asset failed to load within 60 seconds".into());
    }
    if !server.is_loaded_with_dependencies(loading.model.id()) {
        return Ok(false);
    }
    let gltf = world
        .resource::<Assets<Gltf>>()
        .get(&loading.model)
        .ok_or("Missing loaded wolf")?;
    if gltf.skins.len() != 1
        || gltf.meshes.len() != 1
        || gltf.materials.len() != 1
        || gltf.named_animations.len() != 6
    {
        return Err("Unexpected wolf skin/mesh/material/animation contract".into());
    }
    let scene = gltf.default_scene.clone().ok_or("Missing wolf scene")?;
    let mut clips = Vec::new();
    let mut durations = Vec::new();
    for name in NAMES {
        let handle = gltf
            .named_animations
            .get(name)
            .ok_or_else(|| format!("Missing wolf clip {name}"))?
            .clone();
        let duration = world
            .resource::<Assets<AnimationClip>>()
            .get(&handle)
            .ok_or("Missing wolf clip data")?
            .duration();
        if !duration.is_finite() || duration <= 0. || duration > 6. {
            return Err("Invalid wolf clip duration".into());
        }
        clips.push(handle);
        durations.push(duration);
    }
    let (graph, nodes) = AnimationGraph::from_clips(clips);
    let graph = world.resource_mut::<Assets<AnimationGraph>>().add(graph);
    world.insert_resource(Catalog {
        scene,
        graph,
        nodes,
        durations,
    });
    Ok(true)
}
fn update(world: &mut World) {
    match load(world) {
        Ok(false) => return,
        Err(e) => {
            error!("Creature presentation: {e}");
            world.write_message(AppExit::error());
            return;
        }
        Ok(true) => (),
    }
    let state = world.resource::<crate::network::Network>().state();
    let targets = state
        .targeting
        .as_ref()
        .map(|t| t.targets.as_slice())
        .unwrap_or_default();
    let owners: Vec<_> = world
        .query_filtered::<(Entity, &crate::targeting::TargetVisual), Without<Attached>>()
        .iter(world)
        .map(|(e, t)| (e, t.key))
        .collect();
    for (owner, key) in owners {
        let position = targets
            .iter()
            .find(|t| t.key == key)
            .map(|t| t.position)
            .or_else(|| world.resource::<Pending>().corpse(key));
        let Some(position) = position else {
            continue;
        };
        let scene = world.resource::<Catalog>().scene.clone();
        // The source faces +Z. Our characters face -Z; feet remain at authority ground.
        world.entity_mut(owner).remove::<Mesh3d>().insert(Attached);
        world.spawn((
            WorldAssetRoot(scene),
            ChildOf(owner),
            Transform::from_xyz(0., -theia_network::world::PAWN_Y, 0.)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            Visual {
                owner,
                previous: position,
                locomotion: Locomotion::default(),
                players: Vec::new(),
                active: None,
            },
        ));
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Annulus::new(0.72, 0.78));
        let material = world
            .get::<MeshMaterial3d<StandardMaterial>>(owner)
            .unwrap()
            .clone();
        world.spawn((
            SelectionRing,
            ChildOf(owner),
            Mesh3d(mesh),
            material,
            Transform::from_xyz(0., -theia_network::world::PAWN_Y + 0.015, 0.)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            Visibility::Hidden,
        ));
        world.resource_mut::<Pending>().metrics.attached += 1;
    }
    let rings: Vec<_> = world
        .query_filtered::<(Entity, &ChildOf), With<SelectionRing>>()
        .iter(world)
        .map(|(e, p)| (e, p.parent()))
        .collect();
    for (ring, owner) in rings {
        let key = world
            .get::<crate::targeting::TargetVisual>(owner)
            .unwrap()
            .key;
        let selected = state
            .targeting
            .as_ref()
            .is_some_and(|s| s.selected == Some(key.entity));
        let alive = world.resource::<Pending>().corpse(key).is_none();
        let material = world
            .get::<MeshMaterial3d<StandardMaterial>>(owner)
            .unwrap()
            .clone();
        world.entity_mut(ring).insert((
            material,
            if selected && alive {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ));
    }
    let roots: Vec<_> = world
        .query_filtered::<Entity, With<Visual>>()
        .iter(world)
        .collect();
    let count = roots.len();
    {
        let mut pending = world.resource_mut::<Pending>();
        pending.metrics.max_visuals = pending.metrics.max_visuals.max(count);
    }
    for root in roots {
        let owner = world.get::<Visual>(root).unwrap().owner;
        let Some(target) = world.get::<crate::targeting::TargetVisual>(owner) else {
            continue;
        };
        let key = target.key;
        if world.get::<Visual>(root).unwrap().players.is_empty() {
            let descendants = crate::reference_scene::descendants(world, root);
            let players: Vec<_> = descendants
                .iter()
                .copied()
                .filter(|e| world.get::<AnimationPlayer>(*e).is_some())
                .collect();
            if players.is_empty() {
                continue;
            }
            if players.len() != 1 {
                error!("Wolf has multiple animation players");
                world.write_message(AppExit::error());
                return;
            }
            let graph = world.resource::<Catalog>().graph.clone();
            for e in &players {
                world
                    .entity_mut(*e)
                    .insert(AnimationGraphHandle(graph.clone()));
            }
            for e in descendants {
                if world.get::<Mesh3d>(e).is_some() {
                    // Conservative union of independently verified six-clip bounds.
                    world.entity_mut(e).insert(Aabb::from_min_max(
                        Vec3::new(-0.6, -0.1, -1.2),
                        Vec3::new(0.4, 1.1, 1.4),
                    ));
                }
            }
            world.resource_mut::<Pending>().metrics.players += players.len() as u64;
            world.get_mut::<Visual>(root).unwrap().players = players;
        }
        let pending = world.resource::<Pending>();
        let cue = pending.cues.iter().find(|c| c.key == key).copied();
        let dead = cue.is_some_and(|c| c.dead);
        let position = targets
            .iter()
            .find(|t| t.key == key)
            .map(|t| t.position)
            .or_else(|| cue.map(|c| c.position));
        let Some(position) = position else {
            continue;
        };
        let previous = world.get::<Visual>(root).unwrap().previous.metres();
        let p = position.metres();
        let seconds = world.resource::<Time>().delta_secs().max(1e-6);
        // Prefer the interpolated target's actual position; global deltas avoid origin-rebase spikes.
        let presented = world
            .get::<crate::targeting::motion::TargetMotion>(owner)
            .map_or(position, |m| m.global());
        let q = presented.metres();
        let movement = Vec2::new((q[0] - previous[0]) as f32, (q[2] - previous[2]) as f32);
        let now = world.resource::<Pending>().now();
        let speed = world
            .get_mut::<Visual>(root)
            .unwrap()
            .locomotion
            .observe(movement.length() / seconds, now);
        let windup = attack_cue(&state, key);
        let (clip, serial, rate) = if dead {
            (5, cue.unwrap().serial, 1.)
        } else if let Some(attack) = windup {
            (
                3,
                attack.serial,
                world.resource::<Catalog>().durations[3] / attack.remaining,
            )
        } else if let Some(cue) = cue {
            (4, cue.serial, 1.)
        } else if speed > 3. {
            (2, 0, (speed / 4.).clamp(0.5, 2.))
        } else if speed > 0.05 {
            (1, 0, (speed / 1.5).clamp(0.5, 2.))
        } else {
            (0, 0, 1.)
        };
        let facing = if windup.is_some() && !dead {
            windup
                .and_then(|attack| attack.target)
                .map(|focus| {
                    let f = focus.metres();
                    Vec2::new((f[0] - p[0]) as f32, (f[2] - p[2]) as f32)
                })
                .unwrap_or(Vec2::ZERO)
        } else {
            movement
        };
        if facing.length_squared() > 1e-8 && !dead {
            world.get_mut::<Transform>(owner).unwrap().rotation =
                Quat::from_rotation_y((-facing.x).atan2(-facing.y));
        }
        if world.get::<Visual>(root).unwrap().active != Some((clip, serial)) {
            let node = world.resource::<Catalog>().nodes[clip];
            let players = world.get::<Visual>(root).unwrap().players.clone();
            for e in players {
                let mut player = world.get_mut::<AnimationPlayer>(e).unwrap();
                let active = player.stop_all().play(node);
                active.set_speed(rate.clamp(0.1, 12.));
                if clip < 3 {
                    active.repeat();
                }
            }
            world.get_mut::<Visual>(root).unwrap().active = Some((clip, serial));
            world.resource_mut::<Pending>().metrics.transitions[clip] += 1;
        }
        world.get_mut::<Visual>(root).unwrap().previous = presented;
        let mut pending = world.resource_mut::<Pending>();
        pending.metrics.moving_frames += u64::from(clip == 1 || clip == 2);
        pending.metrics.dead_frames += u64::from(clip == 5);
        pending.metrics.attack_frames += u64::from(clip == 3);
        pending.metrics.public_attack_frames +=
            u64::from(clip == 3 && state.public_creature_attack.is_some());
        pending.metrics.remote_victim_frames +=
            u64::from(clip == 3 && windup.is_some_and(|w| w.remote));
        pending.metrics.hit_frames += u64::from(clip == 4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        content::ContentId,
        identity::{EntityId, WorldEpoch},
    };
    #[test]
    fn public_windup_faces_actual_remote_victim_and_expires_without_private_life_all_rates() {
        use std::sync::Arc;
        use theia_core::time::TickRate;
        use theia_network::{
            client::State,
            creature_attack_wire::{self as wire, ClientState, Observation, Profile, Windup},
            player_life_wire,
            wire::{Player, Snapshot, VERSION, Welcome},
        };
        let profile = Arc::new(Profile::new(Arc::new(
            player_life_wire::Profile::from_json(
                include_bytes!("../../../content/gameplay/development-character-stats.json"),
                include_bytes!(
                    "../../../content/gameplay/development-creature-deflect-pursuit.json"
                ),
                include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
                include_bytes!("../../../content/gameplay/development-player-respawn.json"),
            )
            .unwrap(),
        )));
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let mut observation = Observation {
                profile: profile.hash(),
                creature: key(1),
                tick: 100,
                windup: Some(Windup {
                    target: key(2),
                    starts: 100,
                    completes: 100 + profile.windup_ticks(rate).unwrap(),
                }),
            };
            let mut public =
                ClientState::new(profile.clone(), key(1).epoch, rate, [7; 32]).unwrap();
            public
                .apply(&wire::encode(&profile, rate, &observation).unwrap())
                .unwrap();
            let mut state = State {
                id: Some(3),
                welcome: Some(Welcome {
                    id: 3,
                    epoch: 1,
                    world: [7; 32],
                    origin: [8000., 0., 8000.],
                    terrain_stream: true,
                }),
                public_creature_attack: Some(public),
                snapshot: Some(Snapshot {
                    version: VERSION,
                    epoch: 1,
                    origin: [8000., 0., 8000.],
                    tick: 100,
                    tick_hz: rate.hz(),
                    players: vec![
                        Player {
                            id: 2,
                            xz: [5., 9.],
                            y: 1.,
                            applied_sequence: 0,
                            facing: None,
                        },
                        Player {
                            id: 3,
                            xz: [9., 5.],
                            y: 1.,
                            applied_sequence: 0,
                            facing: None,
                        },
                    ],
                }),
                ..Default::default()
            };
            assert!(state.player_life.is_none());
            let attack = attack_cue(&state, key(1)).unwrap();
            assert!(attack.remote);
            assert_eq!(attack.target.unwrap().metres(), [8005., 1., 8009.]);
            assert_eq!(attack.serial, observation.windup.unwrap().completes);
            assert!(attack.remaining > 0.);
            assert!(attack_cue(&state, key(99)).is_none());
            state.snapshot.as_mut().unwrap().players.remove(0);
            assert!(
                attack_cue(&state, key(1)).unwrap().target.is_none(),
                "Missing remote victim must not face the local observer"
            );
            state.snapshot.as_mut().unwrap().tick = attack.serial;
            assert!(
                attack_cue(&state, key(1)).is_none(),
                "Late public frames must not restart expired attacks"
            );
            observation.tick = attack.serial;
            observation.windup = None;
            state
                .public_creature_attack
                .as_mut()
                .unwrap()
                .apply(&wire::encode(&profile, rate, &observation).unwrap())
                .unwrap();
            assert!(attack_cue(&state, key(1)).is_none());
        }
    }
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn position() -> GlobalPosition {
        GlobalPosition::new([8000., 1., 8000.]).unwrap()
    }
    fn hit(id: u64) -> Event {
        Event::Damage {
            ability: ContentId::new("theia.test.hit").unwrap(),
            actor: key(100),
            sequence: 1,
            target: key(id),
            position: position(),
            amount: 1,
            remaining: 10,
        }
    }
    fn death(id: u64) -> Event {
        Event::Died {
            target: key(id),
            position: position(),
            respawn_at: 100,
        }
    }
    #[test]
    fn short_replication_gaps_hold_the_walk_phase_but_a_real_stop_returns_idle() {
        let mut gait = Locomotion::default();
        assert_eq!(gait.observe(1., 0.1), 1.);
        assert_eq!(gait.observe(0., 0.15), 1.);
        assert_eq!(gait.observe(0., 0.29), 1.);
        assert_eq!(gait.observe(1., 0.30), 1.);
        assert_eq!(gait.observe(0., 0.49), 1.);
        assert_eq!(gait.observe(0., 0.51), 0.);
        assert_eq!(Locomotion::default().observe(0., 5.), 0.);
    }
    #[test]
    fn death_survives_cosmetic_hit_pressure_and_late_hits_cannot_revive_it() {
        let mut pending = Pending::default();
        for id in 1..=LIMIT as u64 {
            pending.offer_at(&hit(id), 0.);
        }
        pending.offer_at(&death(50), 0.1);
        pending.offer_at(&hit(50), 0.2);
        assert_eq!(pending.cues.len(), LIMIT);
        assert!(pending.cues.iter().any(|c| c.key == key(50) && c.dead));
        assert_eq!(pending.metrics.dropped_cues, 1);
    }
    #[test]
    fn respawn_retires_the_old_corpse_without_transferring_its_cue() {
        let mut pending = Pending::default();
        pending.offer_at(&death(1), 0.);
        pending.offer_at(
            &Event::Respawned {
                previous: key(1),
                target: key(2),
                position: position(),
            },
            0.1,
        );
        assert!(pending.cues.is_empty());
        pending.offer_at(&hit(2), 0.2);
        assert_eq!(pending.cues.len(), 1);
        assert!(!pending.cues[0].dead);
        assert_eq!(pending.cues[0].key, key(2));
    }
    #[test]
    fn expired_corpse_does_not_consume_capacity_during_later_lives() {
        let mut pending = Pending::default();
        for id in 1..=LIMIT as u64 {
            pending.offer_at(&death(id), 0.);
        }
        pending.offer_at(&death(100), CORPSE_SECONDS + 1.);
        assert_eq!(pending.cues.len(), 1);
        assert_eq!(pending.cues[0].key, key(100));
    }
    #[test]
    fn typed_ability_flinches_require_positive_damage_and_a_surviving_target() {
        use theia_core::damage::{melee_special::Outcome as S, spell::Outcome as M};
        use theia_network::simulation::combat::ability_damage::Outcome;
        let mut pending = Pending::default();
        for outcome in [
            Outcome::MeleeSpecial(S::Miss),
            Outcome::MeleeSpecial(S::Dodge),
            Outcome::MeleeSpecial(S::Parry),
            Outcome::Arcane(M::Miss),
        ] {
            pending.offer_at(
                &Event::AbilityDamage(crate::combat::test_facts::ability_fact(outcome, 100)),
                0.,
            );
        }
        assert!(pending.cues.is_empty());
        pending.offer_at(
            &Event::AbilityDamage(crate::combat::test_facts::ability_fact(
                Outcome::Arcane(M::Hit),
                1,
            )),
            0.,
        );
        assert!(pending.cues.is_empty());
        pending.offer_at(
            &Event::AbilityDamage(crate::combat::test_facts::ability_fact(
                Outcome::MeleeSpecial(S::Critical),
                100,
            )),
            0.,
        );
        assert_eq!(pending.cues.len(), 1);
        assert!(!pending.cues[0].dead);
    }
}
