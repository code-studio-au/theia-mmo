//! Bounded authoritative chunk ownership/activation. All package loading is in
//! LoadPlan::load, outside World::step; publication revalidates live actor state.
use crate::{
    Result,
    chunks::{Index, Residency},
    kinematic::{self, Motion},
    wire::{Intent, MAX_PLAYERS},
    world_bounds::{LocalBounds, WorldBounds},
};
use bevy::prelude::Vec3;
use std::cell::Cell;
use theia_core::{
    coordinates::{ChunkId, GlobalPosition, INITIAL_CHUNK_METRES, LocalFrame, LocalPosition},
    time::TickRate,
};

struct MovementBoundary {
    authority: crate::chunks::Coverage,
    client: Option<crate::chunks::Coverage>,
    world: Option<LocalBounds>,
    blocked: Cell<bool>,
    unready: Cell<bool>,
}
impl kinematic::Boundary for MovementBoundary {
    fn sweep(&self, feet: Vec3, delta: Vec3) -> Option<crate::physics::SweepHit> {
        let authority = self.authority.sweep(feet, delta);
        let client = self.client.as_ref().and_then(|c| c.sweep(feet, delta));
        let coverage = match (authority, client) {
            (Some(a), Some(b)) => Some(if a.distance < b.distance { a } else { b }),
            (a, b) => a.or(b),
        };
        let world = self.world.as_ref().and_then(|b| b.sweep(feet, delta));
        match (coverage, world) {
            (Some(a), Some(b)) => Some(if a.distance < b.distance { a } else { b }),
            (a, b) => a.or(b),
        }
    }
    fn on_hit(&self, feet: Vec3, delta: Vec3) {
        self.blocked.set(true);
        if let Some(client) = self.client.as_ref().and_then(|c| c.sweep(feet, delta))
            && self
                .authority
                .sweep(feet, delta)
                .is_none_or(|a| client.distance <= a.distance)
        {
            self.unready.set(true);
        }
    }
}

/// Shared immutable startup metadata. Loading this descriptor reads only bounded
/// JSON; collision preparation is explicit and must precede runtime/GPU startup.
#[derive(Clone)]
pub struct Config {
    index: Index,
    budget: usize,
    initial: Vec<ChunkId>,
    spawns: Vec<GlobalPosition>,
    fall_plane: f64,
    world_bounds: Option<WorldBounds>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StartDocument {
    schema: u32,
    resident_budget: usize,
    initial_chunks: Vec<[i64; 2]>,
    spawns: Vec<[f64; 3]>,
    fall_plane: f64,
    world_bounds: Option<[[f64; 2]; 2]>,
}
impl Config {
    pub fn load(directory: &std::path::Path) -> Result<Self> {
        let raw: StartDocument = serde_json::from_slice(&crate::content::read_bounded(
            &directory.join("world.session.json"),
            8192,
        )?)?;
        if !matches!((raw.schema, raw.world_bounds), (1, None) | (2, Some(_)))
            || !(1..=crate::chunks::MAX_RESIDENT).contains(&raw.resident_budget)
            || raw.initial_chunks.is_empty()
            || raw.initial_chunks.len() > raw.resident_budget
            || raw.spawns.is_empty()
            || raw.spawns.len() > MAX_PLAYERS
        {
            return Err("Unsupported or over-budget chunk startup metadata".into());
        }
        let index = Index::load(directory)?;
        let world_bounds = raw
            .world_bounds
            .map(|b| WorldBounds::new(b).ok_or("Invalid finite world bounds"))
            .transpose()?;
        if let Some(bounds) = world_bounds {
            bounds.in_frame(index.frame())?;
        }
        let mut initial = std::collections::BTreeSet::new();
        for [x, z] in raw.initial_chunks {
            let id = ChunkId { x, z };
            if !initial.insert(id)
                || id.origin(INITIAL_CHUNK_METRES).and_then(|p| index.owner(p)) != Some(id)
            {
                return Err("Duplicate or unindexed initial chunk".into());
            }
        }
        let mut spawns = Vec::new();
        for p in raw.spawns {
            let global = GlobalPosition::new(p).ok_or("Invalid global spawn")?;
            if world_bounds.is_some_and(|b| !b.contains(global))
                || index.frame().to_local(global).is_none()
                || index.owner(global).is_none_or(|id| {
                    !index
                        .contributors(id)
                        .unwrap()
                        .iter()
                        .all(|needed| initial.contains(needed))
                })
            {
                return Err("Spawn outside initial residency/query frame".into());
            }
            spawns.push(global);
        }
        let fall = raw.fall_plane - index.frame().origin().metres()[1];
        if !fall.is_finite() || !(-4000.0..=4000.0).contains(&fall) {
            return Err("Fall plane outside bounded chunk frame".into());
        }
        Ok(Self {
            index,
            budget: raw.resident_budget,
            initial: initial.into_iter().collect(),
            spawns,
            fall_plane: raw.fall_plane,
            world_bounds,
        })
    }
    pub fn index(&self) -> &Index {
        &self.index
    }
    pub fn budget(&self) -> usize {
        self.budget
    }
    pub fn initial(&self) -> &[ChunkId] {
        &self.initial
    }
    pub fn spawns(&self) -> &[GlobalPosition] {
        &self.spawns
    }
    pub fn fall_plane(&self) -> f64 {
        self.fall_plane
    }
    pub fn world_bounds(&self) -> Option<WorldBounds> {
        self.world_bounds
    }
    pub fn identity(&self) -> [u8; 32] {
        world_identity(
            &self.index,
            &self.spawns,
            self.fall_plane,
            &self.initial,
            self.budget,
            self.world_bounds,
        )
    }
    /// Blocking initial preparation, including trusted spawn body validation.
    /// Runtime changes use LoadPlan, never this method inside a fixed step.
    pub fn prepare(&self) -> Result<Residency> {
        self.prepare_startup().map(|(residency, _)| residency)
    }
    /// The metadata contains trusted guesses. Share the same resolved recovery
    /// feet with presentation that authority uses after support/clearance checks.
    pub fn prepare_startup(&self) -> Result<(Residency, Vec<GlobalPosition>)> {
        let mut residency = Residency::new(self.index.clone(), self.budget)?;
        residency.replace(&self.initial, &[])?;
        let world = ChunkWorld::with_bounds(
            residency.clone(),
            &self.spawns,
            self.fall_plane,
            self.world_bounds,
        )?;
        let spawns = (0..self.spawns.len())
            .map(|slot| world.frame().to_global(world.spawn(slot)).unwrap())
            .collect();
        Ok((residency, spawns))
    }
}
fn world_identity(
    index: &Index,
    spawns: &[GlobalPosition],
    fall_plane: f64,
    initial: &[ChunkId],
    budget: usize,
    world_bounds: Option<WorldBounds>,
) -> [u8; 32] {
    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    hash.update(if world_bounds.is_some() {
        b"theia-authoritative-chunk-world-v4\0"
    } else {
        b"theia-authoritative-chunk-world-v3\0"
    });
    hash.update(&index.identity());
    hash.update(&(budget as u16).to_le_bytes());
    if let Some(bounds) = world_bounds {
        for value in bounds.metres().into_iter().flatten() {
            hash.update(&value.to_le_bytes());
        }
    }
    hash.update(&(spawns.len() as u16).to_le_bytes());
    for p in spawns {
        for v in p.metres() {
            hash.update(&v.to_le_bytes());
        }
    }
    hash.update(&fall_plane.to_le_bytes());
    hash.update(&(initial.len() as u16).to_le_bytes());
    for id in initial {
        hash.update(&id.x.to_le_bytes());
        hash.update(&id.z.to_le_bytes());
    }
    hash.finish().as_ref().try_into().unwrap()
}

pub struct ChunkWorld {
    residency: Residency,
    spawns: Vec<LocalPosition>,
    fall_limit: f32,
    identity: [u8; 32],
    rejected: u64,
    blocked: u64,
    unready_steps: u64,
    world_bounds: Option<WorldBounds>,
}
pub struct LoadPlan {
    epoch: u64,
    source: Residency,
    required: Vec<ChunkId>,
}
pub struct Prepared {
    epoch: u64,
    base_generation: u64,
    candidate: Residency,
}
#[derive(Clone, Copy)]
pub struct Occupant {
    pub feet: GlobalPosition,
    pub grounded: bool,
}
impl LoadPlan {
    /// Blocking I/O/query preparation: call on a bounded worker, never a tick.
    pub fn load(self) -> Result<Prepared> {
        let base_generation = self.source.generation();
        let candidate = self.source.staged(&self.required, &[])?;
        Ok(Prepared {
            epoch: self.epoch,
            base_generation,
            candidate,
        })
    }
}
impl Prepared {
    pub fn residency(&self) -> &Residency {
        &self.candidate
    }
}
impl ChunkWorld {
    pub fn new(residency: Residency, spawns: &[GlobalPosition], fall_plane: f64) -> Result<Self> {
        Self::with_bounds(residency, spawns, fall_plane, None)
    }
    pub fn with_bounds(
        residency: Residency,
        spawns: &[GlobalPosition],
        fall_plane: f64,
        world_bounds: Option<WorldBounds>,
    ) -> Result<Self> {
        if spawns.is_empty() || spawns.len() > MAX_PLAYERS || !fall_plane.is_finite() {
            return Err("Chunk world requires bounded trusted spawns/fall plane".into());
        }
        let frame = residency.index().frame();
        if let Some(bounds) = world_bounds {
            bounds.in_frame(frame)?;
        }
        let extent = theia_core::coordinates::MAX_LOCAL_METRES as f32;
        if [-extent, extent].into_iter().any(|v| {
            frame
                .to_global(LocalPosition::new([v; 3]).unwrap())
                .is_none()
        }) {
            return Err("Chunk query frame lacks a checked global margin".into());
        }
        let fall_limit = (fall_plane - frame.origin().metres()[1]) as f32;
        if !fall_limit.is_finite() || !(-4000.0..=4000.0).contains(&fall_limit) {
            return Err("Fall plane outside bounded chunk frame".into());
        }
        let scene = residency
            .scene()
            .ok_or("Spawn collision must be prepared before admission")?;
        let mut points = Vec::new();
        for &spawn in spawns {
            let guess = frame.to_local(spawn).ok_or("Spawn outside query frame")?;
            let feet = kinematic::spawn(scene, guess)?;
            if world_bounds.is_some_and(|b| !b.contains(frame.to_global(feet).unwrap()))
                || !Self::covered(&residency, frame.to_global(feet).unwrap())
                || feet.metres()[1] <= fall_limit
            {
                return Err("Spawn capsule is not fully resident/above fall plane".into());
            }
            points.push(feet);
        }
        let identity = world_identity(
            residency.index(),
            spawns,
            fall_plane,
            &residency.loaded(),
            residency.budget(),
            world_bounds,
        );
        Ok(Self {
            residency,
            spawns: points,
            fall_limit,
            identity,
            rejected: 0,
            blocked: 0,
            unready_steps: 0,
            world_bounds,
        })
    }
    /// Full XZ capsule footprint, including skin, must have known active owners.
    fn covered(residency: &Residency, feet: GlobalPosition) -> bool {
        Self::footprint(residency.index(), feet, |id| {
            residency.colliders(id).is_some()
        })
    }
    fn footprint(index: &Index, feet: GlobalPosition, contains: impl Fn(ChunkId) -> bool) -> bool {
        let [x, y, z] = feet.metres();
        let radius = f64::from(kinematic::FOOTPRINT_RADIUS);
        for dx in [-radius, 0., radius] {
            for dz in [-radius, 0., radius] {
                let Some(p) = GlobalPosition::new([x + dx, y, z + dz]) else {
                    return false;
                };
                if index.owner(p).is_none_or(|id| {
                    !index
                        .contributors(id)
                        .unwrap()
                        .iter()
                        .copied()
                        .all(&contains)
                }) {
                    return false;
                }
            }
        }
        true
    }
    pub fn residency(&self) -> &Residency {
        &self.residency
    }
    pub fn frame(&self) -> LocalFrame {
        self.residency.index().frame()
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn rejected(&self) -> u64 {
        self.rejected
    }
    pub fn blocked(&self) -> u64 {
        self.blocked
    }
    pub fn unready_steps(&self) -> u64 {
        self.unready_steps
    }
    pub fn spawn(&self, slot: usize) -> LocalPosition {
        self.spawns[slot % self.spawns.len()]
    }
    pub fn demand(&self, actors: &[(GlobalPosition, Intent, bool)]) -> Option<Vec<ChunkId>> {
        use std::collections::BTreeSet;
        if actors.is_empty()
            || actors
                .iter()
                .all(|(_, i, falling)| i.axes == [0, 0] && !falling)
        {
            return None;
        }
        let mut ids = BTreeSet::new();
        let add = |ids: &mut BTreeSet<ChunkId>, p: GlobalPosition| {
            let p = self.world_bounds.map_or(p, |bounds| bounds.clamp(p));
            let [x, y, z] = p.metres();
            let r = f64::from(kinematic::FOOTPRINT_RADIUS);
            for dx in [-r, 0., r] {
                for dz in [-r, 0., r] {
                    if let Some(id) = GlobalPosition::new([x + dx, y, z + dz])
                        .and_then(|p| self.residency.index().owner(p))
                    {
                        ids.extend(self.residency.index().contributors(id).unwrap());
                    }
                }
            }
        };
        for &p in &self.spawns {
            add(&mut ids, self.frame().to_global(p)?);
        }
        for &(p, _, _) in actors {
            add(&mut ids, p);
        }
        if ids.len() > self.residency.budget() {
            return None;
        }
        for &(p, intent, falling) in actors {
            let norm = f64::from(intent.axes[0])
                .hypot(f64::from(intent.axes[1]))
                .max(1000.);
            let [x, y, z] = p.metres();
            let ahead = GlobalPosition::new([
                x + f64::from(intent.axes[0]) / norm * 4.,
                y,
                z + f64::from(intent.axes[1]) / norm * 4.,
            ])?;
            let mut next = ids.clone();
            add(&mut next, ahead);
            if next.len() <= self.residency.budget() {
                ids = next;
            }
            if falling {
                for [dx, dz] in [
                    [4., 0.],
                    [-4., 0.],
                    [0., 4.],
                    [0., -4.],
                    [4., 4.],
                    [4., -4.],
                    [-4., 4.],
                    [-4., -4.],
                ] {
                    let mut next = ids.clone();
                    add(&mut next, GlobalPosition::new([x + dx, y, z + dz])?);
                    if next.len() <= self.residency.budget() {
                        ids = next;
                    }
                }
            }
        }
        let active = self.residency.loaded();
        // Keep spare residency until a real new load needs the budget. An
        // unload-only offer can otherwise supersede a delayed peer's prepare
        // as soon as a second peer starts moving toward the next cell.
        if ids.iter().all(|id| active.contains(id)) {
            return Some(active);
        }
        Some(ids.into_iter().collect())
    }
    pub fn plan(&self, epoch: u64, required: &[ChunkId]) -> Result<LoadPlan> {
        if required.len() > self.residency.budget() {
            return Err("Chunk plan exceeds residency budget".into());
        }
        Ok(LoadPlan {
            epoch,
            source: self.residency.clone(),
            required: required.to_vec(),
        })
    }
    pub fn activate(&mut self, epoch: u64, prepared: Prepared, actors: &[Occupant]) -> Result<()> {
        let result = self.validate(epoch, &prepared, actors);
        if let Err(reason) = result {
            self.rejected += 1;
            return Err(reason);
        }
        self.residency = prepared.candidate;
        Ok(())
    }
    fn validate(&self, epoch: u64, prepared: &Prepared, actors: &[Occupant]) -> Result<()> {
        let next = &prepared.candidate;
        if prepared.epoch != epoch
            || prepared.base_generation != self.residency.generation()
            || next.index().identity() != self.residency.index().identity()
            || next.index().frame().origin() != self.frame().origin()
            || next.budget() != self.residency.budget()
            || actors.len() > MAX_PLAYERS
        {
            return Err("Stale or foreign chunk candidate".into());
        }
        let scene = next
            .scene()
            .ok_or("Cannot publish empty collision with world spawns")?;
        // Initial world spawns remain available for recovery and fresh admission.
        for &spawn in &self.spawns {
            if !Self::covered(next, self.frame().to_global(spawn).unwrap()) {
                return Err("Candidate unloads a world spawn footprint".into());
            }
            kinematic::validate_pose(scene, spawn, true)?;
        }
        for actor in actors {
            if self.world_bounds.is_some_and(|b| !b.contains(actor.feet)) {
                return Err("Actor outside finite world bounds".into());
            }
            let local = self
                .frame()
                .to_local(actor.feet)
                .ok_or("Actor outside chunk query frame")?;
            if !Self::covered(next, actor.feet) {
                return Err("Candidate unloads actor capsule footprint".into());
            }
            kinematic::validate_pose(scene, local, actor.grounded)?;
            if !actor.grounded
                && let Some(before) =
                    kinematic::support_below(self.residency.scene().unwrap(), local)
                && kinematic::support_below(scene, local)
                    .is_none_or(|after| (after - before).abs() > 0.02)
            {
                return Err("Candidate changes airborne actor landing support".into());
            }
        }
        Ok(())
    }
    pub fn advance(
        &mut self,
        body: &mut Motion,
        start: LocalPosition,
        intent: Intent,
        rate: TickRate,
    ) -> Result<LocalPosition> {
        self.advance_ready(body, start, intent, rate, None)
    }
    pub fn advance_ready(
        &mut self,
        body: &mut Motion,
        start: LocalPosition,
        intent: Intent,
        rate: TickRate,
        ready: Option<&std::collections::BTreeSet<ChunkId>>,
    ) -> Result<LocalPosition> {
        let boundary = MovementBoundary {
            authority: self.residency.index().coverage(self.residency.loaded())?,
            client: ready
                .map(|ids| self.residency.index().coverage(ids.iter().copied()))
                .transpose()?,
            world: self
                .world_bounds
                .map(|b| b.in_frame(self.frame()))
                .transpose()?,
            blocked: Cell::new(false),
            unready: Cell::new(false),
        };
        let mut unready = false;
        let norm = (f32::from(intent.axes[0]).hypot(f32::from(intent.axes[1])) / 1000.).max(1.);
        let normalized = intent.axes.map(|v| (f32::from(v) / norm) as i16);
        let attempts = [intent.axes, [0, normalized[1]], [normalized[0], 0], [0, 0]];
        for (attempt, axes) in attempts.into_iter().enumerate() {
            let mut state = *body;
            let feet = kinematic::advance_bounded(
                self.residency.scene().unwrap(),
                &mut state,
                start,
                Intent { axes, ..intent },
                rate,
                Some(&boundary),
            )?;
            if self
                .world_bounds
                .is_none_or(|b| b.contains(self.frame().to_global(feet).unwrap()))
                && Self::covered(
                    &self.residency,
                    self.frame()
                        .to_global(feet)
                        .ok_or("Actor outside global frame")?,
                )
            {
                if axes != [0, 0]
                    && ready.is_some_and(|ids| {
                        !Self::footprint(
                            self.residency.index(),
                            self.frame().to_global(feet).unwrap(),
                            |id| ids.contains(&id),
                        )
                    })
                {
                    unready = true;
                    continue;
                }
                if feet.metres()[1] < self.fall_limit {
                    return Err("Unsupported chunk-world fall".into());
                }
                if attempt > 0 || boundary.blocked.get() {
                    self.blocked += 1;
                }
                if unready || boundary.unready.get() {
                    self.unready_steps += 1;
                }
                *body = state;
                return Ok(feet);
            }
        }
        Err("Actor capsule outside resident collision".into())
    }
}
