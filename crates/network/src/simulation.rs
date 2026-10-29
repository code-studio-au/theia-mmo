//! Server-only fixed-step authority. Inputs never choose timestep, speed or position.
pub mod combat;
use crate::wire::{Input, Intent, MAX_PLAYERS, Player, Snapshot, TICK_HZ, VERSION};
use std::collections::BTreeMap;
use std::time::Duration;
use theia_core::{
    coordinates::{GlobalPosition, LocalFrame, LocalPosition},
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};

pub const INPUT_TIMEOUT_TICKS: u64 = 10;
pub struct World {
    epoch: WorldEpoch,
    frame: LocalFrame,
    replication: LocalFrame,
    replication_span: f32,
    rate: TickRate,
    tick: u64,
    next_id: u64,
    movement_recoveries: u64,
    players: BTreeMap<u64, Actor>,
    targets: BTreeMap<u64, crate::targeting::Target>,
    targeting_enabled: bool,
    combat: Option<combat::Authority>,
    terrain: Option<crate::terrain::Terrain>,
    chunks: Option<crate::chunk_world::ChunkWorld>,
    client_chunks_required: bool,
    procedural: Option<crate::procedural::Runtime>,
    terrain_teleport: Option<(u64, [f64; 2])>,
    pub dev_tour: bool,
    tour_stage: usize,
    tour_started: Option<std::time::Instant>,
    predicted: bool,
}
struct Actor {
    id: EntityId,
    position: GlobalPosition,
    applied_sequence: u64,
    intent: Intent,
    input_tick: u64,
    body: crate::kinematic::Motion,
    spawn: LocalPosition,
    ready: Option<std::collections::BTreeSet<theia_core::coordinates::ChunkId>>,
    ready_sequence: u64,
    commands: Option<crate::commands::Timeline>,
    selected_target: Option<EntityKey>,
}
impl Default for World {
    fn default() -> Self {
        use ring::rand::{SecureRandom, SystemRandom};
        let epoch = loop {
            let mut bytes = [0u8; 8];
            SystemRandom::new()
                .fill(&mut bytes)
                .expect("OS randomness for world incarnation");
            if let Some(epoch) = WorldEpoch::new(u64::from_le_bytes(bytes)) {
                break epoch;
            }
        };
        let frame = LocalFrame::new(GlobalPosition::new([0.; 3]).unwrap());
        Self {
            epoch,
            frame,
            replication: frame,
            replication_span: 128.,
            rate: TickRate::default(),
            tick: 0,
            next_id: 0,
            movement_recoveries: 0,
            players: BTreeMap::new(),
            targets: BTreeMap::new(),
            targeting_enabled: false,
            combat: None,
            terrain: None,
            chunks: None,
            client_chunks_required: false,
            procedural: None,
            terrain_teleport: None,
            dev_tour: false,
            tour_stage: 0,
            tour_started: None,
            predicted: false,
        }
    }
}
impl World {
    pub fn enable_prediction(&mut self) -> crate::Result<()> {
        if self.procedural.is_none() || !self.players.is_empty() {
            return Err("Prediction profile requires an inactive procedural world".into());
        }
        self.predicted = true;
        Ok(())
    }
    pub fn prediction_enabled(&self) -> bool {
        self.predicted
    }
    pub fn command(&mut self, id: u64, packet: &crate::prediction_wire::Commands) -> usize {
        if packet.epoch != self.epoch() {
            return 0;
        }
        let Some(actor) = self.players.get_mut(&id) else {
            return 0;
        };
        if packet.ready_sequence > actor.ready_sequence {
            return 0;
        }
        let Some(timeline) = &mut actor.commands else {
            return 0;
        };
        packet
            .commands
            .iter()
            .filter(|&&command| {
                matches!(
                    timeline.offer(packet.stream_epoch, command),
                    Ok(crate::commands::Receipt::Queued)
                )
            })
            .count()
    }
    pub fn correction(&self, id: u64) -> Option<crate::prediction_wire::Corrected> {
        let actor = self.players.get(&id)?;
        let mut feet = actor.position.metres();
        feet[1] -= f64::from(crate::world::PAWN_Y);
        Some(crate::prediction_wire::Corrected {
            speed: self.procedural.as_ref()?.speed as u8,
            state: crate::prediction::Correction {
                world: self.identity(),
                actor: self.entity_key(id)?,
                rate: self.rate,
                input: actor.commands.as_ref()?.state(),
                pose: crate::prediction::Pose {
                    feet: GlobalPosition::new(feet)?,
                    motion: actor.body.snapshot(),
                },
                ready_sequence: actor.ready_sequence,
            },
        })
    }
    pub fn with_procedural(spawn: [f64; 2], speed: f32) -> crate::Result<Self> {
        Self::with_terrain_source(spawn, speed, crate::procedural::Source::Procedural)
    }
    pub fn with_terrain_source(
        spawn: [f64; 2],
        speed: f32,
        source: crate::procedural::Source,
    ) -> crate::Result<Self> {
        Ok(Self {
            procedural: Some(crate::procedural::Runtime::with_source(
                spawn, speed, source,
            )?),
            client_chunks_required: true,
            ..Self::default()
        })
    }
    pub fn procedural(&self) -> Option<&crate::procedural::Runtime> {
        self.procedural.as_ref()
    }
    /// Trusted owner-side developer operation. No wire message can set coordinates.
    pub fn teleport_development(&mut self, id: u64, xz: [f64; 2]) -> bool {
        if self.procedural.is_none()
            || !self.players.contains_key(&id)
            || self.terrain_teleport.is_some()
            || xz
                .iter()
                .any(|v| !v.is_finite() || !(1. ..=crate::procedural::SIDE - 1.).contains(v))
        {
            return false;
        }
        self.terrain_teleport = Some((id, xz));
        true
    }
    pub fn poll_terrain(&mut self) {
        self.poll_terrain_inner(&[]);
    }
    /// Trusted owner-side preparation, outside the fixed step. Pending restored
    /// admissions retain bounded collision demand until their peer joins.
    pub fn poll_terrain_keeping(&mut self, additional: &[GlobalPosition]) -> crate::Result<()> {
        if self.procedural.is_none() || additional.len() > MAX_PLAYERS {
            return Err("Restored terrain demand exceeds supported backend/capacity".into());
        }
        let r = f64::from(crate::kinematic::FOOTPRINT_RADIUS);
        if additional.iter().any(|p| {
            let [x, y, z] = p.metres();
            [-r, r].into_iter().any(|dx| {
                [-r, r].into_iter().any(|dz| {
                    GlobalPosition::new([x + dx, y, z + dz])
                        .and_then(crate::procedural::owner)
                        .is_none()
                })
            })
        }) {
            return Err("Restored admission footprint outside finite terrain".into());
        }
        self.poll_terrain_inner(additional);
        if let Some(error) = self.procedural.as_ref().and_then(|r| r.error.as_ref()) {
            return Err(error.clone().into());
        }
        Ok(())
    }
    fn poll_terrain_inner(&mut self, additional: &[GlobalPosition]) {
        if self.dev_tour
            && let Some(&id) = self.players.keys().next()
        {
            let started = self
                .tour_started
                .get_or_insert_with(std::time::Instant::now);
            let destinations = [
                [100., 100.],
                [15850., 100.],
                [15850., 15850.],
                [100., 15850.],
                [8000., 8000.],
            ];
            if self.tour_stage < destinations.len()
                && started.elapsed().as_secs() >= 10 * (self.tour_stage as u64 + 1)
                && self.terrain_teleport.is_none()
            {
                let xz = destinations[self.tour_stage];
                self.tour_stage += 1;
                self.teleport_development(id, xz);
            }
        }
        if let Some(runtime) = &mut self.procedural {
            let mut positions: Vec<_> = self.players.values().map(|a| a.position).collect();
            if let Some((_, [x, z])) = self.terrain_teleport {
                positions.push(GlobalPosition::new([x, 0., z]).unwrap());
            }
            positions.extend_from_slice(additional);
            runtime.poll(&positions);
            if let Some((id, [x, z])) = self.terrain_teleport {
                let p = GlobalPosition::new([x, 0., z]).unwrap();
                if let Some(patch) = runtime.patches.get(&crate::procedural::owner(p).unwrap()) {
                    if let Some(actor) = self.players.get_mut(&id) {
                        let Some(feet) = patch.try_spawn(x, z) else {
                            runtime.error =
                                Some("Saved terrain teleport has no supported spawn".into());
                            return;
                        };
                        let mut p = feet.metres();
                        p[1] += f64::from(crate::world::PAWN_Y);
                        actor.position = GlobalPosition::new(p).unwrap();
                        actor.body.reset(actor.intent.jump);
                        actor.ready = Some(Default::default());
                        if let Some(commands) = &actor.commands {
                            let stream = commands
                                .state()
                                .stream_epoch
                                .checked_add(1)
                                .expect("Movement stream exhausted");
                            actor.commands = Some(
                                crate::commands::Timeline::new(stream, self.rate, self.tick)
                                    .unwrap(),
                            );
                            actor.intent = Intent::default();
                        }
                    }
                    self.terrain_teleport = None;
                    if let Some(combat) = &mut self.combat {
                        combat.interrupt(id);
                    }
                    eprintln!("terrain_teleport actor={id} x={x} z={z}");
                }
            }
        }
    }
    pub fn snapshot_for(&self, id: u64) -> Snapshot {
        let focus = self
            .players
            .get(&id)
            .map_or(self.frame.origin(), |a| a.position);
        let [x, _, z] = focus.metres();
        let frame = LocalFrame::new(
            GlobalPosition::new([(x / 128.).floor() * 128., 0., (z / 128.).floor() * 128.])
                .unwrap(),
        );
        Snapshot {
            version: VERSION,
            epoch: self.epoch(),
            origin: frame.origin().metres(),
            tick: self.tick,
            tick_hz: self.rate.hz(),
            players: self
                .players
                .values()
                .filter_map(|a| {
                    if !crate::targeting::relevant(focus, a.position) {
                        return None;
                    }
                    let p = frame.to_local(a.position)?.metres();
                    Some(Player {
                        id: a.id.value(),
                        xz: [p[0], p[2]],
                        y: p[1],
                        applied_sequence: a.applied_sequence,
                    })
                })
                .collect(),
        }
    }
    fn step_procedural(&mut self, timeout: u64) -> Snapshot {
        let terrain = self.procedural.as_ref().unwrap();
        for actor in self.players.values_mut() {
            if let Some(commands) = &mut actor.commands {
                let applied = commands.advance().expect("Simulation tick exhausted");
                debug_assert_eq!(applied.tick, self.tick);
                actor.intent = applied.effective_intent(actor.body.snapshot());
                actor.input_tick = applied.fresh_tick;
                actor.applied_sequence = applied.applied_sequence;
            }
            if actor.commands.is_none() && self.tick - actor.input_tick > timeout {
                actor.intent = Intent::default();
            }
            let Some(patch) = terrain.patch(actor.position) else {
                continue;
            };
            let centre = patch.frame.to_local(actor.position).unwrap().metres();
            let start =
                LocalPosition::new([centre[0], centre[1] - crate::world::PAWN_Y, centre[2]])
                    .unwrap();
            // Shared swept coverage holds XZ at unknown geometry/permission,
            // including gravity slides, while vertical falling/jumping continues.
            let coverage = crate::chunks::Coverage::procedural_cells(
                patch.frame,
                patch
                    .ids
                    .iter()
                    .copied()
                    .filter(|id| actor.ready.as_ref().is_none_or(|ready| ready.contains(id))),
            )
            .expect("Bounded procedural patch coverage");
            let before = actor.body;
            match crate::kinematic::advance_development(
                &patch.scene,
                &mut actor.body,
                start,
                actor.intent,
                self.rate,
                Some(&coverage),
                terrain.speed,
            ) {
                Ok(feet) => {
                    let p = feet.metres();
                    let feet_global = patch.frame.to_global(feet).unwrap();
                    if patch.covers(feet_global, None) {
                        actor.position = patch
                            .frame
                            .to_global(
                                LocalPosition::new([p[0], p[1] + crate::world::PAWN_Y, p[2]])
                                    .unwrap(),
                            )
                            .unwrap();
                    } else {
                        actor.body = before;
                    }
                }
                Err(_) => {
                    actor.body = before;
                    self.movement_recoveries += 1;
                    if let Some(commands) = &actor.commands {
                        let stream = commands
                            .state()
                            .stream_epoch
                            .checked_add(1)
                            .expect("Movement stream exhausted");
                        actor.commands = Some(
                            crate::commands::Timeline::new(stream, self.rate, self.tick).unwrap(),
                        );
                        actor.intent = Intent::default();
                    }
                }
            }
        }
        self.snapshot_for(self.players.keys().next().copied().unwrap_or(0))
    }
    pub fn movement_recoveries(&self) -> u64 {
        self.movement_recoveries
    }
    pub fn epoch(&self) -> u64 {
        self.epoch.value()
    }
    pub fn completed_tick(&self) -> u64 {
        self.tick
    }
    pub fn origin(&self) -> [f64; 3] {
        self.frame.origin().metres()
    }
    pub fn entity_key(&self, id: u64) -> Option<EntityKey> {
        self.players.get(&id).map(|actor| EntityKey {
            epoch: self.epoch,
            entity: actor.id,
        })
    }
    pub fn global_position(&self, id: u64) -> Option<GlobalPosition> {
        self.players.get(&id).map(|actor| actor.position)
    }
    pub fn set_origin(&mut self, origin: GlobalPosition) -> bool {
        if self.tick != 0
            || !self.players.is_empty()
            || self.chunks.is_some()
            || self.procedural.is_some()
        {
            return false;
        }
        let frame = LocalFrame::new(origin);
        if [-4096., 4096.].into_iter().any(|v| {
            frame
                .to_global(LocalPosition::new([v; 3]).unwrap())
                .is_none()
        }) {
            return false;
        }
        self.frame = frame;
        self.replication = frame;
        true
    }
    pub fn set_replication_span(&mut self, span: f32) -> bool {
        if self.tick != 0
            || !self.players.is_empty()
            || !span.is_finite()
            || !(1. ..=1024.).contains(&span)
        {
            return false;
        }
        self.replication_span = span;
        true
    }
    pub fn at_rate(rate: TickRate) -> Self {
        Self {
            rate,
            ..Self::default()
        }
    }
    pub fn set_tick_rate(&mut self, rate: TickRate) -> bool {
        // Rate changes are configuration, never a live change to gameplay time.
        if self.tick != 0 || !self.players.is_empty() || self.combat.is_some() {
            return false;
        }
        self.rate = rate;
        true
    }
    pub fn tick_rate(&self) -> TickRate {
        self.rate
    }
    pub fn with_terrain(terrain: crate::terrain::Terrain) -> Self {
        Self {
            terrain: Some(terrain.into_world()),
            ..Self::default()
        }
    }
    /// Prepare initial collision before starting the runtime. Future loads use
    /// plan_chunks on the owner and LoadPlan::load on a bounded worker.
    pub fn with_residency(
        residency: crate::chunks::Residency,
        spawns: &[GlobalPosition],
        fall_plane: f64,
    ) -> crate::Result<Self> {
        let chunks = crate::chunk_world::ChunkWorld::new(residency, spawns, fall_plane)?;
        let frame = chunks.frame();
        Ok(Self {
            frame,
            replication: frame,
            chunks: Some(chunks),
            ..Self::default()
        })
    }
    pub fn with_chunk_config(config: &crate::chunk_world::Config) -> crate::Result<Self> {
        let chunks = crate::chunk_world::ChunkWorld::with_bounds(
            config.prepare()?,
            config.spawns(),
            config.fall_plane(),
            config.world_bounds(),
        )?;
        let frame = chunks.frame();
        Ok(Self {
            frame,
            replication: frame,
            chunks: Some(chunks),
            ..Self::default()
        })
    }
    /// Runtime admission policy; headless controller callers may remain trusted.
    pub fn require_client_chunks(&mut self) -> crate::Result<()> {
        if self.chunks.is_some() {
            if self.tick != 0 || !self.players.is_empty() {
                return Err("Chunk readiness policy must precede admission".into());
            }
            self.client_chunks_required = true;
        }
        Ok(())
    }
    pub fn client_ready(&mut self, id: u64, message: &crate::wire::ChunkReady) -> bool {
        if !self.client_chunks_required || !message.valid() || message.epoch != self.epoch() {
            return false;
        }
        let ids: std::collections::BTreeSet<_> = message
            .chunks
            .iter()
            .map(|&[x, z]| theia_core::coordinates::ChunkId { x, z })
            .collect();
        if ids.iter().any(|&id| {
            self.chunks.as_ref().map_or_else(
                || !crate::procedural::contains(id),
                |chunks| !chunks.residency().index().contains(id),
            )
        }) {
            return false;
        }
        let Some(actor) = self.players.get_mut(&id) else {
            return false;
        };
        if message.sequence <= actor.ready_sequence {
            return false;
        }
        actor.ready_sequence = message.sequence;
        actor.ready = Some(ids);
        true
    }
    pub fn chunks(&self) -> Option<&crate::chunk_world::ChunkWorld> {
        self.chunks.as_ref()
    }
    pub fn chunk_demand(&self) -> Option<Vec<theia_core::coordinates::ChunkId>> {
        let actors: Vec<_> = self
            .players
            .values()
            .map(|a| (a.position, a.intent, a.body.vertical_speed() < 0.))
            .collect();
        self.chunks.as_ref()?.demand(&actors)
    }
    pub fn plan_chunks(
        &self,
        required: &[theia_core::coordinates::ChunkId],
    ) -> crate::Result<crate::chunk_world::LoadPlan> {
        self.chunks
            .as_ref()
            .ok_or("World has no chunk backend")?
            .plan(self.epoch(), required)
    }
    /// Publish against current actors, never the positions captured when loading
    /// began. This operation performs no file reads and does not advance time.
    pub fn activate_chunks(&mut self, prepared: crate::chunk_world::Prepared) -> crate::Result<()> {
        let actors: Vec<_> = self
            .players
            .values()
            .map(|actor| {
                let p = actor.position.metres();
                crate::chunk_world::Occupant {
                    feet: GlobalPosition::new([p[0], p[1] - f64::from(crate::world::PAWN_Y), p[2]])
                        .expect("Validated actor feet"),
                    grounded: actor.body.grounded(),
                }
            })
            .collect();
        let epoch = self.epoch();
        self.chunks
            .as_mut()
            .ok_or("World has no chunk backend")?
            .activate(epoch, prepared, &actors)
    }
    pub fn identity(&self) -> [u8; 32] {
        if let Some(terrain) = &self.procedural {
            let identity = terrain.source.identity();
            let identity = if self.targeting_enabled {
                crate::targeting::profile_identity(identity)
            } else {
                identity
            };
            return self
                .combat
                .as_ref()
                .map_or(identity, |c| c.identity(identity));
        }
        self.chunks.as_ref().map_or_else(
            || {
                self.terrain
                    .as_ref()
                    .map_or(crate::terrain::GRAYBOX_ID, |t| t.identity)
            },
            |chunks| chunks.identity(),
        )
    }
    pub fn terrain_prepared_for(&self, position: GlobalPosition) -> bool {
        self.procedural
            .as_ref()
            .and_then(|r| r.patch(position))
            .is_some_and(|p| p.covers(position, None))
    }
    /// Body-centre placement from the shared static controller, for a trusted
    /// development fixture slot. No account or transport identity is created.
    pub fn development_player_spawn(&self, slot: u8) -> Option<GlobalPosition> {
        if !(1..=MAX_PLAYERS as u8).contains(&slot) {
            return None;
        }
        let runtime = self.procedural.as_ref()?;
        let offset = f64::from((slot - 1) % 4) * 2.;
        let x = if runtime.spawn[0] + offset <= 15999. {
            runtime.spawn[0] + offset
        } else {
            runtime.spawn[0] - offset
        };
        let z = runtime.spawn[1];
        let requested = GlobalPosition::new([x, 0., z])?;
        let mut p = runtime.patch(requested)?.try_spawn(x, z)?.metres();
        p[1] += f64::from(crate::world::PAWN_Y);
        GlobalPosition::new(p)
    }
    /// Stored body centres are never trusted without prepared world collision.
    /// Controller velocity, held input, target and casts are deliberately fresh.
    pub fn validate_restored_player(&self, position: GlobalPosition) -> crate::Result<()> {
        let runtime = self
            .procedural
            .as_ref()
            .ok_or("Restored admission requires streamed terrain")?;
        let patch = runtime
            .patch(position)
            .ok_or("Restored admission collision is not prepared")?;
        if !patch.covers(position, None) {
            return Err("Restored footprint lacks finite terrain coverage".into());
        }
        let mut feet = position.metres();
        feet[1] -= f64::from(crate::world::PAWN_Y);
        let local = patch
            .frame
            .to_local(GlobalPosition::new(feet).ok_or("Invalid restored feet")?)
            .ok_or("Restored pose outside bounded collision frame")?;
        crate::kinematic::validate_pose(&patch.scene, local, false)?;
        if crate::kinematic::support_below(&patch.scene, local).is_none() {
            return Err("Restored pose has no bounded landing support".into());
        }
        Ok(())
    }
    pub fn join_restored(&mut self, position: GlobalPosition) -> Option<u64> {
        self.validate_restored_player(position).ok()?;
        self.join_inner(Some(position))
    }
    pub fn join(&mut self) -> Option<u64> {
        self.join_inner(None)
    }
    fn join_inner(&mut self, restored: Option<GlobalPosition>) -> Option<u64> {
        if self.players.len() >= MAX_PLAYERS {
            return None;
        }
        self.next_id = self.next_id.checked_add(1)?;
        let id = self.next_id;
        let x = ((id - 1) % 4) as f32 * 2.;
        let spawn = if let Some(runtime) = &self.procedural {
            let [x, _, z] = restored.map_or(
                [runtime.spawn[0] + f64::from(x), 0., runtime.spawn[1]],
                GlobalPosition::metres,
            );
            let patch = runtime.patch(GlobalPosition::new([x, 0., z]).unwrap())?;
            let p = patch.try_spawn(x, z)?.metres();
            // Spawn storage is unused for this backend; global truth is assigned below.
            [0., (p[1] + f64::from(crate::world::PAWN_Y)) as f32, 0.]
        } else if let Some(chunks) = &self.chunks {
            let [x, y, z] = chunks.spawn((id - 1) as usize).metres();
            [x, y + crate::world::PAWN_Y, z]
        } else if let Some(terrain) = &self.terrain {
            let feet = terrain.spawn((id - 1) as usize)?;
            let [x, y, z] = feet.metres();
            [x, y + crate::world::PAWN_Y, z]
        } else {
            let feet = crate::kinematic::spawn(
                crate::world::physics_scene(),
                LocalPosition::new([x, 0., 0.]).unwrap(),
            )
            .ok()?
            .metres();
            [feet[0], feet[1] + crate::world::PAWN_Y, feet[2]]
        };
        self.players.insert(
            id,
            Actor {
                id: EntityId::new(id).expect("Monotonic nonzero actor ID"),
                position: self
                    .frame
                    .to_global(LocalPosition::new(spawn).unwrap())
                    .expect("Validated world anchor"),
                applied_sequence: 0,
                selected_target: None,
                commands: self
                    .predicted
                    .then(|| crate::commands::Timeline::new(1, self.rate, self.tick).unwrap()),
                ready: self
                    .client_chunks_required
                    .then(std::collections::BTreeSet::new),
                ready_sequence: 0,
                intent: Intent::default(),
                input_tick: self.tick,
                body: crate::kinematic::Motion::default(),
                spawn: LocalPosition::new([spawn[0], spawn[1] - crate::world::PAWN_Y, spawn[2]])
                    .expect("Validated capsule spawn"),
            },
        );
        if let Some(position) = restored {
            self.players.get_mut(&id).unwrap().position = position;
        } else if let Some(runtime) = &self.procedural {
            let x = runtime.spawn[0] + f64::from(x);
            let z = runtime.spawn[1];
            let mut p = runtime
                .patch(GlobalPosition::new([x, 0., z]).unwrap())?
                .try_spawn(x, z)?
                .metres();
            p[1] += f64::from(crate::world::PAWN_Y);
            self.players.get_mut(&id).unwrap().position = GlobalPosition::new(p)?;
        }
        Some(id)
    }
    /// Trusted developer placement on prepared graybox geometry, before admission.
    /// No collision object is constructed for a targetable entity.
    pub fn spawn_development_target(&mut self, xz: [f64; 2]) -> crate::Result<EntityKey> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.combat.is_some()
            || self.targets.len() >= crate::targeting::MAX_TARGETS
        {
            return Err(
                "Targets require inactive graybox world and available target budget".into(),
            );
        }
        let runtime = self
            .procedural
            .as_ref()
            .ok_or("Development targets require graybox terrain")?;
        let focus = GlobalPosition::new([xz[0], 0., xz[1]]).ok_or("Invalid target position")?;
        if crate::procedural::owner(focus).is_none() {
            return Err("Target outside graybox world".into());
        }
        let patch = runtime
            .patch(focus)
            .ok_or("Target collision is not prepared")?;
        let mut position = patch
            .try_spawn(xz[0], xz[1])
            .ok_or("Target has no supported placement")?
            .metres();
        position[1] += f64::from(crate::world::PAWN_Y);
        let position = GlobalPosition::new(position).ok_or("Invalid supported target position")?;
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or("Entity identity exhausted")?;
        let key = EntityKey {
            epoch: self.epoch,
            entity: EntityId::new(next_id).unwrap(),
        };
        self.next_id = next_id;
        self.targeting_enabled = true;
        self.targets
            .insert(next_id, crate::targeting::Target { key, position });
        Ok(key)
    }
    pub fn targeting_enabled(&self) -> bool {
        self.targeting_enabled
    }
    /// The caller binds actor_id from its admitted session, never from payload data.
    pub fn select_target(&mut self, actor_id: u64, target: Option<EntityKey>) -> bool {
        let Some(actor) = self.players.get_mut(&actor_id) else {
            return false;
        };
        if let Some(key) = target {
            let Some(entity) = self.targets.get(&key.entity.value()) else {
                return false;
            };
            if key != entity.key || !crate::targeting::relevant(actor.position, entity.position) {
                return false;
            }
        }
        actor.selected_target = target;
        true
    }
    pub fn remove_development_target(&mut self, key: EntityKey) -> bool {
        let removed = self.retire_target(key);
        let forgotten = self.combat.as_mut().is_some_and(|c| c.forget(key));
        removed || forgotten
    }
    fn retire_target(&mut self, key: EntityKey) -> bool {
        if self
            .targets
            .get(&key.entity.value())
            .is_none_or(|target| target.key != key)
        {
            return false;
        }
        self.targets.remove(&key.entity.value());
        for actor in self.players.values_mut() {
            if actor.selected_target == Some(key) {
                actor.selected_target = None;
            }
        }
        true
    }
    /// Server-owner view only. Network/client adoption is a separate profile.
    pub fn targeting_snapshot(&self, actor_id: u64) -> Option<crate::targeting::Snapshot> {
        let actor = self.players.get(&actor_id)?;
        Some(crate::targeting::Snapshot {
            selected: actor.selected_target,
            targets: self
                .targets
                .values()
                .filter(|target| crate::targeting::relevant(actor.position, target.position))
                .copied()
                .collect(),
        })
    }
    pub fn leave(&mut self, id: u64) {
        self.players.remove(&id);
        if let Some(combat) = &mut self.combat {
            combat.leave(id);
        }
    }
    pub fn input(&mut self, id: u64, input: Input) -> bool {
        if self.predicted {
            return false;
        }
        let Some(actor) = self.players.get_mut(&id) else {
            return false;
        };
        if input.version != VERSION
            || input.epoch != self.epoch.value()
            || !input.intent.valid()
            || input.sequence <= actor.applied_sequence
        {
            return false;
        }
        actor.applied_sequence = input.sequence;
        actor.intent = input.intent;
        actor.input_tick = self.tick;
        true
    }
    pub fn step(&mut self) -> Snapshot {
        let before = self.combat.as_ref().map(|_| {
            self.players
                .iter()
                .map(|(&id, a)| (id, a.position))
                .collect()
        });
        let snapshot = self.step_movement();
        if let Some(before) = before {
            self.advance_combat(before);
        }
        if !self.targets.is_empty() {
            for actor in self.players.values_mut() {
                if actor.selected_target.is_some_and(|key| {
                    self.targets.get(&key.entity.value()).is_none_or(|target| {
                        target.key != key
                            || !crate::targeting::relevant(actor.position, target.position)
                    })
                }) {
                    actor.selected_target = None;
                }
            }
        }
        snapshot
    }
    fn step_movement(&mut self) -> Snapshot {
        self.tick += 1;
        // Preserve the old 10/30 second timeout at every configured rate.
        let timeout = self
            .rate
            .ticks_for(Duration::from_secs_f64(
                INPUT_TIMEOUT_TICKS as f64 / f64::from(TICK_HZ),
            ))
            .expect("Bounded input timeout");
        if self.procedural.is_some() {
            return self.step_procedural(timeout);
        }
        for actor in self.players.values_mut() {
            if self.tick - actor.input_tick > timeout {
                actor.intent = Intent::default();
            }
            let local = self
                .frame
                .to_local(actor.position)
                .expect("Actor within content frame")
                .metres();
            let start = LocalPosition::new([local[0], local[1] - crate::world::PAWN_Y, local[2]])
                .expect("Actor within content frame");
            let result = if let Some(chunks) = &mut self.chunks {
                chunks
                    .advance_ready(
                        &mut actor.body,
                        start,
                        actor.intent,
                        self.rate,
                        actor.ready.as_ref(),
                    )
                    .map_err(|_| "Rejected chunk movement or unsupported fall")
            } else if let Some(terrain) = &self.terrain {
                crate::kinematic::advance(
                    &terrain.physics,
                    &mut actor.body,
                    start,
                    actor.intent,
                    self.rate,
                )
                .and_then(|feet| {
                    if terrain.below_world(feet) {
                        Err("Unsupported fall below world recovery plane")
                    } else {
                        Ok(feet)
                    }
                })
            } else {
                crate::kinematic::advance(
                    crate::world::physics_scene(),
                    &mut actor.body,
                    start,
                    actor.intent,
                    self.rate,
                )
            };
            let feet = match result {
                Ok(feet) => feet,
                Err(reason) => {
                    self.movement_recoveries += 1;
                    eprintln!(
                        "movement_recovery actor={} reason={reason}",
                        actor.id.value()
                    );
                    actor.body.reset(actor.intent.jump);
                    actor.spawn
                }
            }
            .metres();
            let xz = if self.terrain.is_some() || self.chunks.is_some() {
                [feet[0], feet[2]]
            } else {
                [
                    feet[0].clamp(-crate::wire::WORLD_LIMIT, crate::wire::WORLD_LIMIT),
                    feet[2].clamp(-crate::wire::WORLD_LIMIT, crate::wire::WORLD_LIMIT),
                ]
            };
            let y = feet[1] + crate::world::PAWN_Y;
            actor.position = self
                .frame
                .to_global(LocalPosition::new([xz[0], y, xz[1]]).unwrap())
                .expect("Validated world anchor");
        }
        // Every actor fits the canonical content frame, but a frame following
        // one actor can exclude a distant newly admitted peer.
        if self
            .players
            .values()
            .any(|a| self.replication.to_local(a.position).is_none())
        {
            self.replication = self.frame;
        }
        if let Some(first) = self.players.values().next() {
            let relative = self
                .replication
                .to_local(first.position)
                .expect("Bounded fixture extent")
                .metres();
            if relative[0].abs() > self.replication_span
                || relative[2].abs() > self.replication_span
            {
                let p = self.frame.to_local(first.position).unwrap().metres();
                let span = self.replication_span;
                let origin = self
                    .frame
                    .to_global(
                        LocalPosition::new([
                            (p[0] / span).floor() * span,
                            0.,
                            (p[2] / span).floor() * span,
                        ])
                        .unwrap(),
                    )
                    .unwrap();
                let candidate = LocalFrame::new(origin);
                if self
                    .players
                    .values()
                    .all(|a| candidate.to_local(a.position).is_some())
                {
                    self.replication = candidate;
                }
            }
        }
        Snapshot {
            version: VERSION,
            epoch: self.epoch.value(),
            origin: self.replication.origin().metres(),
            tick: self.tick,
            tick_hz: self.rate.hz(),
            players: self
                .players
                .values()
                .map(|actor| {
                    let p = self
                        .replication
                        .to_local(actor.position)
                        .expect("Bounded snapshot frame")
                        .metres();
                    Player {
                        id: actor.id.value(),
                        xz: [p[0], p[2]],
                        y: p[1],
                        applied_sequence: actor.applied_sequence,
                    }
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::WORLD_LIMIT;
    #[test]
    fn rates_preserve_speed_timeout_and_forbid_live_clock_changes() {
        for hz in [20, 30, 60] {
            let rate = TickRate::from_hz(hz).unwrap();
            let mut world = World::at_rate(rate);
            let id = world.join().unwrap();
            assert!(!world.set_tick_rate(TickRate::Hz60));
            for seq in 1..=hz {
                world.input(id, moving(&world, u64::from(seq), [1000, 0]));
                let snapshot = world.step();
                assert_eq!(snapshot.tick_hz, hz);
                assert_eq!(snapshot.players[0].applied_sequence, u64::from(seq));
            }
            assert!((world.step().players[0].xz[0] - (5. + 5. / hz as f32)).abs() < 0.00001);
            for _ in 0..hz {
                world.step();
            }
            let stopped = world.step().players[0].xz;
            for _ in 0..hz {
                world.step();
            }
            assert_eq!(world.step().players[0].xz, stopped);
            assert!(stopped[0] > 6. && stopped[0] < 7.);
        }
    }
    fn moving(world: &World, sequence: u64, axes: [i16; 2]) -> Input {
        Input {
            version: VERSION,
            epoch: world.epoch(),
            sequence,
            intent: Intent {
                jump: false,
                axes,
                sprint: false,
            },
        }
    }
    #[test]
    fn invalid_actor_state_recovers_independently_and_keeps_sequence_progress() {
        let mut world = World::default();
        let a = world.join().unwrap();
        let b = world.join().unwrap();
        world.players.get_mut(&a).unwrap().position = world
            .frame
            .to_global(LocalPosition::new([-8., 1., -8.]).unwrap())
            .unwrap();
        assert!(world.input(a, moving(&world, 7, [0, 0])));
        assert!(world.input(b, moving(&world, 8, [1000, 0])));
        let snapshot = world.step();
        let first = snapshot.players.iter().find(|p| p.id == a).unwrap();
        let second = snapshot.players.iter().find(|p| p.id == b).unwrap();
        assert!(first.xz[0].abs() < 0.001 && first.xz[1].abs() < 0.001);
        assert_eq!(first.applied_sequence, 7);
        assert!(second.xz[0] > 2.16);
        assert_eq!(second.applied_sequence, 8);
        assert_eq!(world.movement_recoveries(), 1);
        world.step();
        assert_eq!(world.movement_recoveries(), 1);
    }
    #[test]
    fn terrain_players_pass_through_and_snapshots_follow_surface_height() {
        let a = [-24., -7.2, -24.];
        let b = [24., 7.2, -24.];
        let c = [24., 7.2, 24.];
        let d = [-24., -7.2, 24.];
        let bytes=serde_json::to_vec(&serde_json::json!({"schema":1,"source_origin":[0,0,0],"visual_sha256":vec![0;32],"triangles":[[a,c,b],[a,d,c]]})).unwrap();
        let mut world = World::with_terrain(crate::terrain::Terrain::decode(&bytes).unwrap());
        let a = world.join().unwrap();
        let b = world.join().unwrap();
        let mut crossed = false;
        for sequence in 1..=30 {
            world.input(b, moving(&world, sequence, [-1000, 0]));
            let s = world.step();
            for p in &s.players {
                // Independent perpendicular capsule-plane clearance on slope 0.3.
                let clearance = (p.y - 0.9 - p.xz[0] * 0.3 + 0.35) / 1.09_f32.sqrt() - 0.35;
                assert!((0.006..0.010).contains(&clearance), "{p:?}; {clearance}");
            }
            let first = s.players.iter().find(|p| p.id == a).unwrap();
            let second = s.players.iter().find(|p| p.id == b).unwrap();
            crossed |= (second.xz[0] - first.xz[0]).abs() < 0.1;
            if sequence == 30 {
                assert!(second.xz[0] < first.xz[0] - 2.9);
            }
        }
        assert!(crossed, "Players share terrain without blocking each other");
    }
    #[test]
    fn speed_diagonal_stale_timeout_and_bounds_are_server_owned() {
        let mut world = World::default();
        let a = world.join().unwrap();
        let b = world.join().unwrap();
        for sequence in 1..=30 {
            assert!(world.input(a, moving(&world, sequence, [1000, 1000])));
            assert!(world.input(b, moving(&world, sequence, [1000, 0])));
            // Even arbitrarily many accepted/replayed packets cannot run an extra step.
            assert!(!world.input(a, moving(&world, sequence, [-1000, 0])));
            world.step();
        }
        let snapshot = world.step();
        let diagonal = snapshot.players[0].xz;
        assert!((diagonal[0].hypot(diagonal[1]) - 5. * 31. / 30.).abs() < 0.001);
        let before = snapshot.players[1].xz;
        for _ in 0..30 {
            world.step();
        }
        let stopped = world.step().players[1].xz;
        for _ in 0..30 {
            world.step();
        }
        assert_eq!(stopped, world.step().players[1].xz);
        assert!(stopped[0] - before[0] < 2.);
        for seq in 31..1000 {
            world.input(a, moving(&world, seq, [1000, 1000]));
            world.step();
        }
        assert_eq!(world.step().players[0].xz, [WORLD_LIMIT; 2]);
    }
    #[test]
    fn capacity_disconnect_non_reused_ids_and_invalid_inputs() {
        let mut world = World::default();
        let first = world.join().unwrap();
        assert!(!world.input(first, moving(&world, 1, [1001, 0])));
        assert!(!world.input(999, moving(&world, 1, [0, 0])));
        for _ in 1..MAX_PLAYERS {
            world.join().unwrap();
        }
        assert!(world.join().is_none());
        world.leave(first);
        assert!(world.join().unwrap() > MAX_PLAYERS as u64);
        assert!(world.step().players.iter().all(|p| p.id != first));
    }

    #[test]
    fn players_can_overlap_and_pass_through_each_other() {
        let mut world = World::default();
        let a = world.join().unwrap();
        let b = world.join().unwrap();
        for sequence in 1..=12 {
            world.input(a, moving(&world, sequence, [1000, 0]));
            world.input(b, moving(&world, sequence, [-1000, 0]));
            let snapshot = world.step();
            if sequence == 6 {
                assert!((snapshot.players[0].xz[0] - snapshot.players[1].xz[0]).abs() < 0.00001);
            }
            if sequence == 12 {
                assert!((snapshot.players[0].xz[0] - 2.).abs() < 0.00001);
                assert!(snapshot.players[1].xz[0].abs() < 0.00001);
            }
        }
    }
    #[test]
    fn large_signed_origins_rebase_wire_frames_without_changing_authority_motion() {
        for sign in [-1., 1.] {
            let origin = GlobalPosition::new([
                sign * 1.0e9 + 0.01,
                sign * 1.0e9 + 0.02,
                sign * 1.0e9 - 0.03,
            ])
            .unwrap();
            let mut distant = World::default();
            let mut reference = World::default();
            assert!(distant.set_origin(origin));
            assert!(distant.set_replication_span(2.));
            assert!(reference.set_replication_span(2.));
            let id = distant.join().unwrap();
            let other = reference.join().unwrap();
            assert!(!distant.set_origin(origin));
            assert!(!distant.set_replication_span(4.));
            let anchor = LocalFrame::new(origin);
            let mut changes = 0;
            let mut last = origin.metres();
            for sequence in 1..=90 {
                distant.input(id, moving(&distant, sequence, [0, 1000]));
                reference.input(other, moving(&reference, sequence, [0, 1000]));
                let snapshot = distant.step();
                let expected = reference.step();
                if snapshot.origin != last {
                    changes += 1;
                    last = snapshot.origin;
                }
                let decoded =
                    crate::wire::Snapshot::decode(&crate::wire::encode(&snapshot).unwrap())
                        .unwrap();
                let local = decoded.projected(anchor).unwrap();
                let expected = expected.projected(reference.frame).unwrap();
                assert!((local.players[0].xz[1] - expected.players[0].xz[1]).abs() < 0.00002);
                let canon = distant.global_position(id).unwrap().metres();
                let received = decoded
                    .global_position(&decoded.players[0])
                    .unwrap()
                    .metres();
                for axis in 0..3 {
                    assert!((canon[axis] - received[axis]).abs() < 0.000002);
                }
                assert!(
                    anchor
                        .to_local(distant.global_position(id).unwrap())
                        .is_some()
                );
            }
            assert!(changes >= 5, "Actual replication frames must change");
            let key = distant.entity_key(id).unwrap();
            distant.leave(id);
            assert!(distant.entity_key(id).is_none());
            let replacement = distant.join().unwrap();
            assert_ne!(distant.entity_key(replacement).unwrap(), key);
        }
    }

    #[test]
    fn restart_epoch_rejects_stale_commands_even_when_entity_numbers_match() {
        let mut old = World::default();
        let old_id = old.join().unwrap();
        let stale = moving(&old, u64::MAX, [1000, 0]);
        let old_snapshot = old.step();
        let mut new = World::default();
        let new_id = new.join().unwrap();
        assert_eq!(new_id, old_id);
        assert_ne!(old.epoch(), new.epoch());
        assert!(!new.input(new_id, stale));
        assert!(new.input(new_id, moving(&new, 1, [0, 0])));
        assert_eq!(new.step().players[0].applied_sequence, 1);
        let welcome = crate::wire::Welcome {
            terrain_stream: false,
            id: new_id,
            epoch: new.epoch(),
            world: new.identity(),
            origin: new.origin(),
        };
        assert!(!welcome.accepts(&old_snapshot));
    }
}
