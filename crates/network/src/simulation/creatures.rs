//! One opt-in nonblocking patrol, consuming an offline checked route. No path
//! search, file access, wall clock, database or transport I/O in the fixed step.
use super::World;
use crate::{Result, kinematic, navigation, procedural, wire::Intent};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use theia_core::{
    content::ContentId,
    coordinates::{GlobalPosition, LocalPosition},
    facing::Facing,
    identity::EntityKey,
};
pub const MAX_PROFILE_BYTES: usize = 4096;
const ARRIVAL_METRES: f64 = 0.01;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    schema: u32,
    id: String,
    route: String,
    speed_metres_per_second: f32,
    wait_ms: u32,
    aggro: Option<theia_core::pursuit::Settings>,
    #[serde(default, deserialize_with = "deflect_eligibility")]
    can_deflect: Option<bool>,
}
fn deflect_eligibility<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<bool>, D::Error> {
    bool::deserialize(d).map(Some)
}
#[derive(Clone)]
pub struct Profile {
    id: ContentId,
    route: ContentId,
    speed: f32,
    wait_ms: u32,
    digest: [u8; 32],
    aggro: Option<theia_core::pursuit::Settings>,
    can_deflect: bool,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Creature profile exceeds byte budget".into());
        }
        let raw: RawProfile = serde_json::from_slice(bytes)?;
        let can_deflect = match (raw.schema, raw.can_deflect) {
            (1, None) => false,
            (2, Some(value)) => value,
            _ => return Err("Creature deflect eligibility requires explicit schema 2".into()),
        };
        if !matches!(raw.schema, 1..=2)
            || !raw.speed_metres_per_second.is_finite()
            || !(0.1..=5.).contains(&raw.speed_metres_per_second)
            || !(100..=60000).contains(&raw.wait_ms)
            || raw.aggro.is_some_and(|settings| !settings.valid())
        {
            return Err("Creature profile exceeds initial patrol bounds".into());
        }
        Ok(Self {
            id: ContentId::new(&raw.id).ok_or("Invalid creature ID")?,
            route: ContentId::new(&raw.route).ok_or("Invalid creature route ID")?,
            speed: raw.speed_metres_per_second,
            wait_ms: raw.wait_ms,
            digest: crate::terrain::sha256(bytes),
            aggro: raw.aggro,
            can_deflect,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn speed_metres_per_second(&self) -> f32 {
        self.speed
    }
    pub fn aggro(&self) -> Option<theia_core::pursuit::Settings> {
        self.aggro
    }
    pub fn can_deflect(&self) -> bool {
        self.can_deflect
    }
}
pub fn profile_identity(base: [u8; 32], profile: &Profile, navigation: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-patrol-profile-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(profile.digest);
    bytes.extend(navigation);
    crate::terrain::sha256(&bytes)
}
/// Startup/loader work only. Validates files and prepares one bounded static
/// collision patch; the fixed step consumes only the resulting immutable data.
pub struct Prepared {
    profile: Profile,
    navigation: [u8; 32],
    source: [u8; 32],
    points: Vec<GlobalPosition>,
    patch: Arc<procedural::Patch>,
    worker: Option<navigation::worker::Worker>,
}
impl Prepared {
    pub fn load(
        profile: Profile,
        store: &navigation::Store,
        source: &procedural::Source,
    ) -> Result<Self> {
        if store.manifest().source != source.identity() {
            return Err("Patrol navigation belongs to another terrain".into());
        }
        store.verify()?;
        let route = store
            .manifest()
            .routes
            .iter()
            .find(|r| r.id == profile.route.as_str())
            .ok_or("Patrol route is absent")?;
        let points = route
            .points
            .iter()
            .map(|p| GlobalPosition::new(*p).ok_or_else(|| "Invalid patrol point".into()))
            .collect::<Result<Vec<_>>>()?;
        let owner = procedural::owner(points[0]).ok_or("Patrol home outside world")?;
        let patch = Arc::new(procedural::Patch::load(owner, source)?);
        let enabled = profile.aggro.is_some();
        let mut prepared =
            Self::checked(profile, store.identity(), source.identity(), points, patch)?;
        if enabled {
            prepared.worker = Some(navigation::worker::Worker::load(store)?);
        }
        Ok(prepared)
    }
    fn checked(
        profile: Profile,
        navigation: [u8; 32],
        source: [u8; 32],
        points: Vec<GlobalPosition>,
        patch: Arc<procedural::Patch>,
    ) -> Result<Self> {
        if !(3..=navigation::MAX_ROUTE_POINTS).contains(&points.len()) {
            return Err("Patrol requires a bounded closed route".into());
        }
        let mut length = 0_f64;
        for (i, p) in points.iter().enumerate() {
            let p = p.metres();
            let supported = patch
                .try_spawn(p[0], p[2])
                .ok_or("Unsupported patrol waypoint")?;
            if !patch.covers(supported, None) || (supported.metres()[1] - p[1]).abs() > 0.25 {
                return Err("Patrol waypoint differs from prepared collision support".into());
            }
            if i > 0 {
                let a = points[i - 1].metres();
                let distance = ((a[0] - p[0]).powi(2) + (a[2] - p[2]).powi(2)).sqrt();
                if distance < 0.1 {
                    return Err("Patrol has a stationary route segment".into());
                }
                length += distance;
            }
        }
        let a = points[0].metres();
        let b = points.last().unwrap().metres();
        if (a[0] - b[0]).abs() > 0.001 || (a[2] - b[2]).abs() > 0.001 || length > 1024. {
            return Err("Patrol route must close without an implicit direct-line leg".into());
        }
        Ok(Self {
            profile,
            navigation,
            source,
            points,
            patch,
            worker: None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Idle,
    Patrol,
    Chase,
    Attack,
    Return,
    Dead,
    Blocked,
}
pub(super) struct PursuitMotion {
    pub mode: theia_core::pursuit::Mode,
    pub goal: Option<GlobalPosition>,
    pub speed: f32,
    pub look_at: Option<GlobalPosition>,
}
struct SavedPatrol {
    waypoint: usize,
    remaining_wait: u64,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Snapshot {
    pub tick: u64,
    pub actor: u64,
    pub mode: Mode,
    pub waypoint: usize,
    pub wait_until: Option<u64>,
    pub position: [f64; 3],
    pub home: [f64; 3],
    pub moving_ticks: u64,
    pub idle_ticks: u64,
    pub blocked: u64,
    pub arrivals: u64,
    pub loops: u64,
    pub respawns: u64,
    pub travelled_metres: f64,
}
pub(super) struct Patrol {
    profile: Profile,
    pub(super) worker: Option<navigation::worker::Worker>,
    saved: Option<SavedPatrol>,
    pub(super) motion_blocked: bool,
    pub(super) key: EntityKey,
    points: Vec<GlobalPosition>,
    pub(super) patch: Arc<procedural::Patch>,
    coverage: crate::chunks::Coverage,
    body: kinematic::Motion,
    home: GlobalPosition,
    position: GlobalPosition,
    facing: Facing,
    waypoint: usize,
    wait_ticks: u64,
    wait_until: Option<u64>,
    mode: Mode,
    moving_ticks: u64,
    idle_ticks: u64,
    blocked: u64,
    arrivals: u64,
    loops: u64,
    respawns: u64,
    travelled: f64,
}
impl Patrol {
    pub(super) fn profile(&self) -> &Profile {
        &self.profile
    }
    pub(super) fn profile_hash(&self) -> [u8; 32] {
        self.profile.digest
    }
    pub(super) fn definition(&self) -> &ContentId {
        self.profile.id()
    }
    pub(super) fn aggro(&self) -> Option<theia_core::pursuit::Settings> {
        self.profile.aggro
    }
    pub(super) fn home(&self) -> GlobalPosition {
        self.home
    }
    fn block(&mut self) {
        if self.mode != Mode::Blocked {
            self.blocked += 1;
        }
        self.mode = Mode::Blocked;
    }
    pub(super) fn respawn(&mut self, key: EntityKey, now: u64) -> GlobalPosition {
        self.key = key;
        self.position = self.home;
        self.facing = Facing::towards(self.home, self.points[1]).expect("Checked patrol segment");
        self.body = kinematic::Motion::default();
        self.waypoint = 1;
        self.saved = None;
        self.motion_blocked = false;
        self.wait_until = now.checked_add(self.wait_ticks);
        self.mode = Mode::Idle;
        self.respawns += 1;
        if self.wait_until.is_none() {
            self.block();
        }
        self.home
    }
    fn advance(&mut self, world: &mut World, combat_hold: bool, pursuit: Option<PursuitMotion>) {
        if !world.targets.contains_key(&self.key.entity.value()) {
            self.mode = Mode::Dead;
            self.body = kinematic::Motion::default();
            return;
        }
        if self.mode == Mode::Blocked {
            return;
        }
        let now = world.tick;
        self.motion_blocked = false;
        let pursuing = pursuit
            .as_ref()
            .is_some_and(|p| p.mode != theia_core::pursuit::Mode::Patrol);
        if pursuing && self.saved.is_none() {
            self.saved = Some(SavedPatrol {
                waypoint: self.waypoint,
                remaining_wait: self.wait_until.map_or(0, |until| until.saturating_sub(now)),
            });
        } else if !pursuing && let Some(saved) = self.saved.take() {
            self.waypoint = saved.waypoint;
            self.wait_until = now.checked_add(saved.remaining_wait);
            if self.wait_until.is_none() {
                self.block();
                return;
            }
        }
        let phase = pursuit.as_ref().filter(|_| pursuing);
        let phase_mode = phase.map(|p| match p.mode {
            theia_core::pursuit::Mode::Chase => Mode::Chase,
            theia_core::pursuit::Mode::Attack => Mode::Attack,
            theia_core::pursuit::Mode::Return => Mode::Return,
            theia_core::pursuit::Mode::Dead => Mode::Dead,
            _ => Mode::Blocked,
        });
        if phase_mode == Some(Mode::Blocked) {
            self.block();
            return;
        }
        let waiting = phase.map_or_else(
            || combat_hold || self.wait_until.is_some_and(|until| now <= until),
            |p| p.goal.is_none(),
        );
        let goal = phase
            .map_or(self.points[self.waypoint], |p| {
                p.goal.unwrap_or(self.position)
            })
            .metres();
        let p = self.position.metres();
        let dx = goal[0] - p[0];
        let dz = goal[2] - p[2];
        let distance = dx.hypot(dz);
        let mut intent = Intent::default();
        if waiting {
            self.mode = phase_mode.unwrap_or(Mode::Idle);
            self.idle_ticks += 1;
        } else {
            self.mode = phase_mode.unwrap_or(Mode::Patrol);
            if !pursuing {
                self.wait_until = None;
            }
            let speed = phase.map_or(self.profile.speed, |p| p.speed);
            let travel = f64::from(speed) * f64::from(world.rate.seconds());
            let scale = distance.min(travel) / (5. * f64::from(world.rate.seconds()));
            if distance > 0.000001 {
                intent.axes = [
                    (dx / distance * scale * 1000.).round() as i16,
                    (dz / distance * scale * 1000.).round() as i16,
                ];
            }
        }
        let Some(local) = self.patch.frame.to_local(self.position) else {
            self.block();
            return;
        };
        let mut feet = local.metres();
        feet[1] -= crate::world::PAWN_Y;
        let Some(feet) = LocalPosition::new(feet) else {
            self.block();
            return;
        };
        let before = self.body;
        let moved = kinematic::advance_development(
            &self.patch.scene,
            &mut self.body,
            feet,
            intent,
            world.rate,
            Some(&self.coverage),
            1.,
        );
        let Ok(feet) = moved else {
            self.body = before;
            self.block();
            return;
        };
        let mut centre = feet.metres();
        centre[1] += crate::world::PAWN_Y;
        let Some(position) = LocalPosition::new(centre).and_then(|p| self.patch.frame.to_global(p))
        else {
            self.body = before;
            self.block();
            return;
        };
        let next = position.metres();
        let horizontal = (next[0] - p[0]).hypot(next[2] - p[2]);
        if !waiting && distance > ARRIVAL_METRES && horizontal < 0.00001 {
            self.body = before;
            if pursuing {
                self.motion_blocked = true;
                self.blocked += 1;
            } else {
                self.block();
            }
            return;
        }
        if !world.move_patrol_target(self.key, position) {
            self.body = before;
            self.mode = Mode::Dead;
            return;
        }
        let facing = phase
            .and_then(|p| p.look_at)
            .and_then(|target| Facing::towards(position, target))
            .or_else(|| {
                (!waiting)
                    .then(|| Facing::towards(self.position, position))
                    .flatten()
            });
        if let Some(facing) = facing {
            self.facing = facing;
        }
        self.position = position;
        if !waiting {
            self.moving_ticks += 1;
            self.travelled += horizontal;
            if !pursuing && (goal[0] - next[0]).hypot(goal[2] - next[2]) <= ARRIVAL_METRES {
                self.arrivals += 1;
                self.waypoint += 1;
                if self.waypoint == self.points.len() {
                    self.waypoint = 1;
                    self.loops += 1;
                }
                self.wait_until = now.checked_add(self.wait_ticks);
                self.mode = Mode::Idle;
                if self.wait_until.is_none() {
                    self.block();
                }
            }
        }
    }
}
impl World {
    pub(super) fn creature_can_deflect(&self, target: EntityKey) -> bool {
        self.patrol
            .as_ref()
            .is_some_and(|p| p.key == target && p.profile.can_deflect())
    }
    /// Installation only: consumes already prepared collision/data, no file I/O.
    pub fn enable_patrol(&mut self, prepared: Prepared) -> Result<EntityKey> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.combat.is_some()
            || self.patrol_identity.is_some()
            || self.targets.len() >= crate::targeting::MAX_TARGETS
            || self.next_id == u64::MAX
            || (prepared.profile.aggro.is_some() && prepared.worker.is_none())
        {
            return Err("Patrol requires inactive world before combat/admission".into());
        }
        let wait_ticks = self
            .rate
            .ticks_for(Duration::from_millis(u64::from(prepared.profile.wait_ms)))
            .ok_or("Patrol wait overflows clock")?;
        let coverage = crate::chunks::Coverage::procedural_cells(
            prepared.patch.frame,
            prepared.patch.ids.iter().copied(),
        )?;
        let runtime = self
            .procedural
            .as_mut()
            .ok_or("Patrol requires graybox terrain")?;
        if runtime.source.identity() != prepared.source {
            return Err("Patrol prepared for another terrain".into());
        }
        let replaced = runtime
            .patches
            .insert(prepared.patch.owner, prepared.patch.clone());
        runtime.generated = runtime.generated.saturating_add(1);
        if replaced.is_some() {
            runtime.retired = runtime.retired.saturating_add(1);
        }
        let p = prepared.points[0].metres();
        let key = self.spawn_development_target([p[0], p[2]])?;
        let home = self.targets[&key.entity.value()].position;
        self.patrol_identity = Some(profile_identity(
            crate::targeting::profile_identity(prepared.source),
            &prepared.profile,
            prepared.navigation,
        ));
        let facing = Facing::towards(home, prepared.points[1]).ok_or("Missing patrol bearing")?;
        self.patrol = Some(Patrol {
            profile: prepared.profile,
            worker: prepared.worker,
            saved: None,
            motion_blocked: false,
            key,
            points: prepared.points,
            patch: prepared.patch,
            coverage,
            body: kinematic::Motion::default(),
            home,
            position: home,
            facing,
            waypoint: 1,
            wait_ticks,
            wait_until: Some(wait_ticks),
            mode: Mode::Idle,
            moving_ticks: 0,
            idle_ticks: 0,
            blocked: 0,
            arrivals: 0,
            loops: 0,
            respawns: 0,
            travelled: 0.,
        });
        Ok(key)
    }
    fn move_patrol_target(&mut self, key: EntityKey, position: GlobalPosition) -> bool {
        let Some(target) = self
            .targets
            .get_mut(&key.entity.value())
            .filter(|t| t.key == key)
        else {
            return false;
        };
        if self
            .combat
            .as_mut()
            .is_some_and(|c| !c.move_target(key, position))
        {
            return false;
        }
        target.position = position;
        true
    }
    /// Authoritative orientation for the exact current patrol incarnation.
    /// This does not publish an appearance heading or enable defensive outcomes.
    pub fn patrol_facing(&self, key: EntityKey) -> Option<Facing> {
        let patrol = self.patrol.as_ref().filter(|p| p.key == key)?;
        self.targets
            .get(&key.entity.value())
            .filter(|target| target.key == key)?;
        Some(patrol.facing)
    }
    pub fn patrol_snapshot(&self) -> Option<Snapshot> {
        let p = self.patrol.as_ref()?;
        Some(Snapshot {
            tick: self.tick,
            actor: p.key.entity.value(),
            mode: p.mode,
            waypoint: p.waypoint,
            wait_until: p.wait_until,
            position: p.position.metres(),
            home: p.home.metres(),
            moving_ticks: p.moving_ticks,
            idle_ticks: p.idle_ticks,
            blocked: p.blocked,
            arrivals: p.arrivals,
            loops: p.loops,
            respawns: p.respawns,
            travelled_metres: p.travelled,
        })
    }
    pub(super) fn advance_patrol(&mut self) {
        let mut combat = self.combat.take();
        let pursuit = combat.as_mut().and_then(|c| c.advance_pursuit(self));
        let hold = combat.as_ref().is_some_and(|c| c.holds_patrol(self));
        self.combat = combat;
        if let Some(mut patrol) = self.patrol.take() {
            patrol.advance(self, hold, pursuit);
            self.patrol = Some(patrol);
        }
    }
    pub(super) fn patrol_respawn(
        &mut self,
        previous: EntityKey,
        key: EntityKey,
    ) -> Option<GlobalPosition> {
        let now = self.tick;
        self.patrol
            .as_mut()
            .filter(|p| p.key == previous)
            .map(|p| p.respawn(key, now))
    }
    pub(super) fn sync_patrol_life(&mut self) {
        if let Some(p) = &mut self.patrol
            && !self.targets.contains_key(&p.key.entity.value())
        {
            p.mode = Mode::Dead;
            p.body = kinematic::Motion::default();
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        simulation::combat::{Event, Outcome, Request},
        wire::{ChunkReady, Input, VERSION},
    };
    #[test]
    fn deflect_eligibility_requires_explicit_schema_two_and_preserves_legacy_identity() {
        let baseline = Profile::from_json(PROFILE).unwrap();
        assert!(!baseline.can_deflect());
        let mut raw: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
        raw["can_deflect"] = true.into();
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
        raw["schema"] = 2.into();
        let enabled = Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(enabled.can_deflect());
        assert_ne!(
            profile_identity([7; 32], &baseline, [8; 32]),
            profile_identity([7; 32], &enabled, [8; 32])
        );
        raw["can_deflect"] = false.into();
        assert!(
            !Profile::from_json(&serde_json::to_vec(&raw).unwrap())
                .unwrap()
                .can_deflect()
        );
        for invalid in [
            serde_json::Value::Null,
            serde_json::json!(0),
            serde_json::json!("true"),
        ] {
            raw["can_deflect"] = invalid;
            assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
        }
        raw.as_object_mut().unwrap().remove("can_deflect");
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
    #[test]
    fn optional_aggro_is_strict_bounded_and_changes_profile_identity() {
        let base = Profile::from_json(PROFILE).unwrap();
        assert!(base.aggro().is_none());
        let mut raw: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
        raw["aggro"] = serde_json::json!({
            "perception_metres": 12., "leash_metres": 35.,
            "chase_speed": 5., "return_speed": 5.,
            "los_grace_ms": 2000, "engagement_ms": 30000, "reacquire_ms": 2000
        });
        let enabled = Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(enabled.aggro().unwrap().leash_metres, 35.);
        assert_ne!(
            profile_identity([7; 32], &base, [8; 32]),
            profile_identity([7; 32], &enabled, [8; 32])
        );
        raw["aggro"]["leash_metres"] = 2.into();
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
        raw["aggro"]["leash_metres"] = 35.into();
        raw["aggro"]["client_can_aggro"] = true.into();
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
    use theia_core::time::TickRate;
    const PROFILE: &[u8] =
        include_bytes!("../../../../content/gameplay/development-creature-patrol.json");
    fn preparation(world: &World, home: [f64; 2]) -> Prepared {
        let focus = GlobalPosition::new([home[0], 0., home[1]]).unwrap();
        let source = &world.procedural.as_ref().unwrap().source;
        let patch =
            Arc::new(procedural::Patch::load(procedural::owner(focus).unwrap(), source).unwrap());
        let points = [[0., 0.], [10., 0.], [10., 10.], [0., 0.]]
            .map(|[x, z]| patch.try_spawn(home[0] + x, home[1] + z).unwrap())
            .to_vec();
        let nav = crate::terrain::sha256(
            &serde_json::to_vec(&points.iter().map(|p| p.metres()).collect::<Vec<_>>()).unwrap(),
        );
        Prepared::checked(
            Profile::from_json(PROFILE).unwrap(),
            nav,
            source.identity(),
            points,
            patch,
        )
        .unwrap()
    }
    pub(crate) fn fixture(rate: TickRate) -> (World, EntityKey) {
        fixture_at(rate, [8000., 8000.])
    }
    pub(crate) fn fixture_at(rate: TickRate, spawn: [f64; 2]) -> (World, EntityKey) {
        fixture_profile_at(rate, spawn, PROFILE)
    }
    pub(crate) fn fixture_profile_at(
        rate: TickRate,
        spawn: [f64; 2],
        profile: &[u8],
    ) -> (World, EntityKey) {
        let mut world = World::with_procedural(spawn, 1.).unwrap();
        assert!(world.set_tick_rate(rate));
        let mut prepared = preparation(&world, [8008., 8000.]);
        let profile = Profile::from_json(profile).unwrap();
        assert_eq!(profile.route, prepared.profile.route);
        prepared.profile = profile;
        if prepared.profile.aggro.is_some() {
            // Consumer fixture only: exact gentle-terrain triangles, without
            // claiming Recast clearance/obstacle baking. Actual bake/worker
            // obstacle and seam integration is tested by the nav-bake package.
            prepared.worker = Some(
                navigation::worker::Worker::spawn(
                    query_fixture(&world, &prepared.patch, 128.),
                    prepared.navigation,
                )
                .unwrap(),
            );
        }
        let key = world.enable_patrol(prepared).unwrap();
        (world, key)
    }
    pub(crate) fn query_fixture(
        world: &World,
        patch: &procedural::Patch,
        max_x: f32,
    ) -> navigation::query::Query {
        let geometry = world
            .procedural
            .as_ref()
            .unwrap()
            .source
            .geometry(patch.owner, 32)
            .unwrap();
        let triangles: Vec<_> = geometry
            .triangles
            .iter()
            .filter(|t| {
                t.iter()
                    .all(|i| geometry.positions[*i as usize][0] <= max_x)
            })
            .collect();
        let mut detail = navigation::Detail {
            vertices: Vec::new(),
            triangles: Vec::new(),
            polygons: Vec::new(),
        };
        for triangle in &triangles {
            let base = detail.vertices.len() as u32;
            let first = detail.triangles.len() as u32;
            detail
                .vertices
                .extend(triangle.iter().map(|i| geometry.positions[*i as usize]));
            detail.triangles.push([0, 1, 2]);
            detail.polygons.push([base, 3, first, 1]);
        }
        let polygons = triangles
            .iter()
            .map(|t| t.iter().map(|i| *i as u16).collect())
            .collect();
        let tile = navigation::Tile {
            schema: 1,
            id: [patch.owner.x, patch.owner.z],
            source: world.procedural.as_ref().unwrap().source.identity(),
            agent: navigation::Agent::development(),
            vertices: geometry.positions,
            polygons,
            detail,
        };
        navigation::query::Query::new(&[tile]).unwrap()
    }
    fn actor(world: &mut World) -> u64 {
        let actor = world.join().unwrap();
        let p = world.global_position(actor).unwrap();
        let chunks = world
            .procedural
            .as_ref()
            .unwrap()
            .patch(p)
            .unwrap()
            .ids
            .iter()
            .map(|c| [c.x, c.z])
            .collect();
        assert!(world.client_ready(
            actor,
            &ChunkReady {
                version: VERSION,
                epoch: world.epoch(),
                sequence: 1,
                chunks
            }
        ));
        actor
    }
    #[test]
    fn explicit_quest_mode_requires_matching_inactive_guide_world_and_excludes_auto_start() {
        let data = include_bytes!("../../../../content/gameplay/development-quest.json");
        let explicit = Arc::new(crate::quest_lifecycle_wire::Profile::from_json(data).unwrap());
        let automatic = Arc::new(crate::quest_wire::Profile::from_json(data).unwrap());
        let (mut world, _) = fixture(TickRate::Hz30);
        assert!(world.enable_quest_lifecycle(explicit.clone()).is_err());
        world
            .enable_combat(
                include_bytes!("../../../../content/gameplay/development-abilities.json"),
                100,
                1000,
            )
            .unwrap();
        world
            .enable_progression(Arc::new(
                crate::progression_wire::Profile::from_json(include_bytes!(
                    "../../../../content/gameplay/development-progression.json"
                ))
                .unwrap(),
            ))
            .unwrap();
        assert!(world.enable_quest_lifecycle(explicit.clone()).is_err());
        let guide = Arc::new(
            crate::dialogue::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-dialogue.json"
            ))
            .unwrap(),
        );
        world
            .spawn_development_guide(guide.clone(), [8002., 8000.])
            .unwrap();
        let mut bad: serde_json::Value = serde_json::from_slice(data).unwrap();
        bad["objective"]["creature"] = "theia.dev.other-creature".into();
        assert!(
            world
                .enable_quest_lifecycle(Arc::new(
                    crate::quest_lifecycle_wire::Profile::from_json(
                        &serde_json::to_vec(&bad).unwrap(),
                    )
                    .unwrap()
                ))
                .is_err()
        );
        let baseline = world.identity();
        let recipe =
            include_bytes!("../../../../content/gameplay/development-quest-turn-in-xp.json");
        let xp = include_bytes!("../../../../content/gameplay/development-progression.json");
        let different =
            crate::progression_wire::Profile::from_json(&[xp.as_slice(), b" "].concat()).unwrap();
        let incompatible = Arc::new(
            crate::quest_lifecycle_wire::Profile::from_json_with_turn_in_xp(
                data, recipe, &different,
            )
            .unwrap(),
        );
        assert!(world.enable_quest_lifecycle(incompatible).is_err());
        world.enable_quest_lifecycle(explicit.clone()).unwrap();
        assert_ne!(world.identity(), baseline);
        assert_eq!(
            world.quest_lifecycle_profile().unwrap().hash(),
            explicit.hash()
        );
        assert!(world.quest_profile().is_none());
        assert!(world.enable_quest_lifecycle(explicit.clone()).is_err());
        assert!(world.enable_quest(automatic.clone()).is_err());

        let (mut old, _) = fixture(TickRate::Hz30);
        old.enable_combat(
            include_bytes!("../../../../content/gameplay/development-abilities.json"),
            100,
            1000,
        )
        .unwrap();
        old.enable_progression(Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        ))
        .unwrap();
        old.spawn_development_guide(guide, [8002., 8000.]).unwrap();
        old.enable_quest(automatic).unwrap();
        assert_ne!(old.identity(), world.identity());
        assert!(old.enable_quest_lifecycle(explicit.clone()).is_err());
        old.join().unwrap();
        assert!(old.enable_quest_lifecycle(explicit).is_err());
    }

    #[test]
    fn explicit_quest_entitlement_requires_current_own_dialogue_and_three_dimensional_range() {
        use crate::dialogue::{Action as Talk, Request as Interaction};
        let (mut world, _) = fixture(TickRate::Hz30);
        world
            .enable_combat(
                include_bytes!("../../../../content/gameplay/development-abilities.json"),
                100,
                1000,
            )
            .unwrap();
        world
            .enable_progression(Arc::new(
                crate::progression_wire::Profile::from_json(include_bytes!(
                    "../../../../content/gameplay/development-progression.json"
                ))
                .unwrap(),
            ))
            .unwrap();
        let guide = world
            .spawn_development_guide(
                Arc::new(
                    crate::dialogue::Profile::from_json(include_bytes!(
                        "../../../../content/gameplay/development-dialogue.json"
                    ))
                    .unwrap(),
                ),
                [8002., 8000.],
            )
            .unwrap();
        let rewarded = Arc::new(
            crate::quest_lifecycle_wire::Profile::from_json_with_turn_in_xp(
                include_bytes!("../../../../content/gameplay/development-quest.json"),
                include_bytes!("../../../../content/gameplay/development-quest-turn-in-xp.json"),
                world.progression_profile().unwrap(),
            )
            .unwrap(),
        );
        world.enable_quest_lifecycle(rewarded).unwrap();
        let a = world.join().unwrap();
        world.players.get_mut(&a).unwrap().position = guide.position;
        let mut command = crate::quest_lifecycle_wire::Request {
            actor: world.entity_key(a).unwrap(),
            sequence: 1,
            command: theia_core::quests::lifecycle::Command {
                expected_revision: 0,
                action: theia_core::quests::lifecycle::Action::Accept,
            },
            dialogue_revision: 1,
            npc: guide.key,
        };
        assert!(!world.quest_lifecycle_eligible(a, command));
        world
            .interact(
                a,
                &Interaction {
                    sequence: 1,
                    expected_revision: 0,
                    action: Talk::Open(guide.key),
                },
            )
            .unwrap();
        assert!(world.quest_lifecycle_eligible(a, command));
        let b = world.join().unwrap();
        world.players.get_mut(&b).unwrap().position = guide.position;
        assert!(!world.quest_lifecycle_eligible(b, command));
        assert!(!world.quest_lifecycle_eligible(
            a,
            crate::quest_lifecycle_wire::Request {
                actor: world.entity_key(b).unwrap(),
                ..command
            }
        ));
        assert!(!world.quest_lifecycle_eligible(
            a,
            crate::quest_lifecycle_wire::Request {
                dialogue_revision: 2,
                ..command
            }
        ));
        assert!(!world.quest_lifecycle_eligible(
            a,
            crate::quest_lifecycle_wire::Request {
                npc: world.entity_key(a).unwrap(),
                ..command
            }
        ));
        let p = guide.position.metres();
        world.players.get_mut(&a).unwrap().position =
            GlobalPosition::new([p[0], p[1] + 3.0001, p[2]]).unwrap();
        assert!(
            !world.quest_lifecycle_eligible(a, command),
            "Recheck current 3D range before a prune step"
        );
        world.players.get_mut(&a).unwrap().position = guide.position;
        world
            .interact(
                a,
                &Interaction {
                    sequence: 2,
                    expected_revision: 1,
                    action: Talk::Close,
                },
            )
            .unwrap();
        assert!(!world.quest_lifecycle_eligible(a, command));
        world
            .interact(
                a,
                &Interaction {
                    sequence: 3,
                    expected_revision: 2,
                    action: Talk::Open(guide.key),
                },
            )
            .unwrap();
        assert!(!world.quest_lifecycle_eligible(a, command));
        command.dialogue_revision = 3;
        assert!(world.quest_lifecycle_eligible(a, command));
        world.retire_development_guide(guide.key);
        assert!(!world.quest_lifecycle_eligible(a, command));
    }

    #[test]
    fn objectives_bind_the_progression_profile_and_reuse_current_live_kill_facts() {
        let (mut world, target) = fixture(TickRate::Hz30);
        let profile = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        let objective = Arc::new(
            crate::quest_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-quest.json"
            ))
            .unwrap(),
        );
        assert!(world.enable_quest(objective.clone()).is_err());
        assert!(world.enable_progression(profile.clone()).is_err());
        let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-abilities.json"
        ))
        .unwrap();
        data["abilities"][0]["cast_ms"] = 0.into();
        data["abilities"][0]["effect"]["amount"] = 100.into();
        world
            .enable_combat(&serde_json::to_vec(&data).unwrap(), 100, 1000)
            .unwrap();
        let base = world.identity();
        world.enable_progression(profile.clone()).unwrap();
        assert_eq!(
            world.identity(),
            crate::progression_wire::profile_identity(base, profile.hash())
        );
        assert!(world.enable_progression(profile).is_err());
        let mut bad: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-quest.json"
        ))
        .unwrap();
        bad["objective"]["creature"] = "theia.dev.other-creature".into();
        assert!(
            world
                .enable_quest(Arc::new(
                    crate::quest_wire::Profile::from_json(&serde_json::to_vec(&bad).unwrap())
                        .unwrap()
                ))
                .is_err()
        );
        let base = world.identity();
        world.enable_quest(objective.clone()).unwrap();
        assert_eq!(
            world.identity(),
            crate::quest_wire::profile_identity(base, objective.hash())
        );
        assert_eq!(world.quest_profile().unwrap().hash(), objective.hash());
        assert!(world.enable_quest(objective).is_err());

        let player = actor(&mut world);
        let key = world.entity_key(player).unwrap();
        let request = Request {
            epoch: world.epoch(),
            sequence: 1,
            target,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
        };
        world.cast(player, request);
        world.step();
        let events = world.combat_events().to_vec();
        let credit = world.experience_opportunities(&events).unwrap();
        assert_eq!(credit.len(), 1);
        assert_eq!(credit[0].actor, key);
        assert_eq!(
            credit[0].event.creature.as_str(),
            "theia.dev.training-creature"
        );
        assert_eq!(credit[0].event.id.victim(), target);
        let fatal = events
            .iter()
            .find(|e| matches!(e, Event::Damage { remaining: 0, .. }))
            .unwrap()
            .clone();
        assert_eq!(
            world
                .experience_opportunities(&[fatal.clone(), fatal.clone()])
                .unwrap(),
            credit
        );
        let mut nonfatal = fatal.clone();
        if let Event::Damage { remaining, .. } = &mut nonfatal {
            *remaining = 1;
        }
        assert!(
            world
                .experience_opportunities(&[nonfatal])
                .unwrap()
                .is_empty()
        );
        let mut foreign = fatal.clone();
        if let Event::Damage { actor, .. } = &mut foreign {
            actor.epoch = theia_core::identity::WorldEpoch::new(world.epoch() + 1).unwrap();
        }
        assert!(
            world
                .experience_opportunities(&[foreign])
                .unwrap()
                .is_empty()
        );
        for _ in 0..31 {
            world.step();
        }
        assert_ne!(world.patrol.as_ref().unwrap().key, target);
        assert!(world.experience_opportunities(&[fatal]).unwrap().is_empty());
    }
    #[test]
    fn experience_credit_comes_from_a_fatal_current_creature_life_and_live_player_only() {
        let (mut world, target) = fixture(TickRate::Hz30);
        let profile = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        assert!(world.enable_progression(profile.clone()).is_err());
        let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-abilities.json"
        ))
        .unwrap();
        data["abilities"][0]["cast_ms"] = 0.into();
        data["abilities"][0]["effect"]["amount"] = 100.into();
        world
            .enable_combat(&serde_json::to_vec(&data).unwrap(), 100, 1000)
            .unwrap();
        let base = world.identity();
        world.enable_progression(profile.clone()).unwrap();
        assert_eq!(
            world.identity(),
            crate::progression_wire::profile_identity(base, profile.hash())
        );
        assert!(world.enable_progression(profile).is_err());
        let player = actor(&mut world);
        let key = world.entity_key(player).unwrap();
        let request = Request {
            epoch: world.epoch(),
            sequence: 1,
            target,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
        };
        world.cast(player, request);
        world.step();
        let events = world.combat_events().to_vec();
        let credit = world.experience_opportunities(&events).unwrap();
        assert_eq!(credit.len(), 1);
        assert_eq!(credit[0].actor, key);
        assert_eq!(
            credit[0].event.creature.as_str(),
            "theia.dev.training-creature"
        );
        assert_eq!(credit[0].event.id.victim(), target);
        let fatal = events
            .iter()
            .find(|e| matches!(e, Event::Damage { remaining: 0, .. }))
            .unwrap()
            .clone();
        assert_eq!(
            world
                .experience_opportunities(&[fatal.clone(), fatal.clone()])
                .unwrap(),
            credit
        );
        let mut nonfatal = fatal.clone();
        if let Event::Damage { remaining, .. } = &mut nonfatal {
            *remaining = 1;
        }
        assert!(
            world
                .experience_opportunities(&[nonfatal])
                .unwrap()
                .is_empty()
        );
        let mut foreign = fatal.clone();
        if let Event::Damage { actor, .. } = &mut foreign {
            actor.epoch = theia_core::identity::WorldEpoch::new(world.epoch() + 1).unwrap();
        }
        assert!(
            world
                .experience_opportunities(&[foreign])
                .unwrap()
                .is_empty()
        );
        for _ in 0..31 {
            world.step();
        }
        assert_ne!(world.patrol.as_ref().unwrap().key, target);
        assert!(world.experience_opportunities(&[fatal]).unwrap().is_empty());
    }
    #[test]
    fn reward_credit_comes_from_a_fatal_current_creature_life_and_live_player_only() {
        let (mut world, target) = fixture(TickRate::Hz30);
        let items = crate::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let profile = Arc::new(
            crate::reward_wire::Profile::from_json(
                include_bytes!("../../../../content/gameplay/development-rewards.json"),
                &items,
            )
            .unwrap(),
        );
        assert!(world.enable_rewards(profile.clone()).is_err());
        let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-abilities.json"
        ))
        .unwrap();
        data["abilities"][0]["cast_ms"] = 0.into();
        data["abilities"][0]["effect"]["amount"] = 100.into();
        world
            .enable_combat(&serde_json::to_vec(&data).unwrap(), 100, 1000)
            .unwrap();
        let base = world.identity();
        world.enable_rewards(profile.clone()).unwrap();
        assert_eq!(
            world.identity(),
            super::super::rewards::profile_identity(base, profile.hash())
        );
        assert!(world.enable_rewards(profile).is_err());
        let player = actor(&mut world);
        let key = world.entity_key(player).unwrap();
        let request = Request {
            epoch: world.epoch(),
            sequence: 1,
            target,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
        };
        world.cast(player, request);
        world.step();
        let events = world.combat_events().to_vec();
        let credit = world.reward_opportunities(&events).unwrap();
        assert_eq!(credit.len(), 1);
        assert_eq!(credit[0].actor, key);
        assert_eq!(credit[0].reward.as_str(), "theia.dev.first-training-kill");
        let fatal = events
            .iter()
            .find(|e| matches!(e, Event::Damage { remaining: 0, .. }))
            .unwrap()
            .clone();
        assert_eq!(
            world
                .reward_opportunities(&[fatal.clone(), fatal.clone()])
                .unwrap(),
            credit
        );
        let mut nonfatal = fatal.clone();
        if let Event::Damage { remaining, .. } = &mut nonfatal {
            *remaining = 1;
        }
        assert!(world.reward_opportunities(&[nonfatal]).unwrap().is_empty());
        let mut foreign = fatal.clone();
        if let Event::Damage { actor, .. } = &mut foreign {
            actor.epoch = theia_core::identity::WorldEpoch::new(world.epoch() + 1).unwrap();
        }
        assert!(world.reward_opportunities(&[foreign]).unwrap().is_empty());
        for _ in 0..31 {
            world.step();
        }
        assert_ne!(world.patrol.as_ref().unwrap().key, target);
        assert!(world.reward_opportunities(&[fatal]).unwrap().is_empty());
    }
    #[test]
    fn patrol_wait_speed_arrivals_and_loop_are_fixed_tick_authority_at_every_rate() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (mut world, key) = fixture(rate);
            let home = world.patrol_snapshot().unwrap().position;
            assert!(!world.set_tick_rate(rate));
            for _ in 0..rate.hz() {
                world.step();
                let view = world.patrol_snapshot().unwrap();
                assert_eq!(view.mode, Mode::Idle);
                assert_eq!(view.moving_ticks, 0);
                assert!(
                    (view.position[0] - home[0]).abs() < 0.00001
                        && (view.position[2] - home[2]).abs() < 0.00001
                );
            }
            world.step();
            let first = world.patrol_snapshot().unwrap();
            assert_eq!(first.mode, Mode::Patrol);
            assert!((first.position[0] - home[0] - 2. / f64::from(rate.hz())).abs() < 0.00002);
            for _ in 0..rate.hz() * 25 {
                world.step();
                let view = world.patrol_snapshot().unwrap();
                assert_eq!(
                    world.targets[&key.entity.value()].position.metres(),
                    view.position
                );
                assert_eq!(view.blocked, 0);
                assert_ne!(view.mode, Mode::Dead);
            }
            let view = world.patrol_snapshot().unwrap();
            assert!(view.loops >= 1 && view.arrivals >= 3 && view.travelled_metres > 34.);
        }
    }
    #[test]
    fn moving_health_damage_death_and_fresh_home_respawn_remain_coherent() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (mut world, key) = fixture(rate);
            let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
                "../../../../content/gameplay/development-abilities.json"
            ))
            .unwrap();
            data["abilities"][0]["cast_ms"] = 0.into();
            data["abilities"][0]["effect"]["amount"] = 100.into();
            world
                .enable_combat(&serde_json::to_vec(&data).unwrap(), 100, 1000)
                .unwrap();
            let player = actor(&mut world);
            for _ in 0..rate.hz() * 2 {
                world.step();
            }
            let moving = world.patrol_snapshot().unwrap();
            assert!(moving.position[0] > moving.home[0] + 1.9);
            assert_eq!(
                world.combat_snapshot(player).unwrap().targets[0]
                    .position
                    .metres(),
                moving.position
            );
            let reply = world.cast(
                player,
                Request {
                    epoch: world.epoch(),
                    sequence: 1,
                    target: key,
                    ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
                },
            );
            assert!(matches!(reply.reply.outcome, Outcome::Started { .. }));
            world.step();
            let death = world.patrol_snapshot().unwrap();
            assert_eq!(death.mode, Mode::Dead);
            assert!(world.patrol_facing(key).is_none());
            assert!(
                world
                    .combat_events()
                    .iter()
                    .any(|e| matches!(e,Event::Died {target,..} if *target==key))
            );
            for _ in 0..rate.hz() - 1 {
                world.step();
                assert_eq!(world.patrol_snapshot().unwrap().position, death.position);
            }
            world.step();
            let respawn = world.patrol_snapshot().unwrap();
            assert_eq!(respawn.respawns, 1);
            assert_eq!(respawn.mode, Mode::Idle);
            assert_ne!(respawn.actor, key.entity.value());
            assert_eq!(respawn.position, respawn.home);
            let fresh = world.patrol.as_ref().unwrap().key;
            assert_ne!(fresh, key);
            assert!(world.patrol_facing(key).is_none());
            assert_eq!(world.patrol_facing(fresh).unwrap().units(), 49152);
            assert!(world.combat_events().iter().any(|e|matches!(e,Event::Respawned {previous,target,position} if *previous==key && target.entity.value()==respawn.actor && position.metres()==respawn.home)));
            for _ in 0..rate.hz() {
                world.step();
                assert_eq!(world.patrol_snapshot().unwrap().mode, Mode::Idle);
            }
            world.step();
            assert_eq!(world.patrol_snapshot().unwrap().mode, Mode::Patrol);
        }
    }
    #[test]
    fn static_collision_blocks_a_stale_route_and_never_creates_actor_blockers() {
        use crate::physics::{PlacedCollider, SOLID, StaticScene};
        use avian3d::prelude::Collider;
        use bevy::prelude::Vec3;
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        let mut prepared = preparation(&world, [8008., 8000.]);
        let old = &prepared.patch;
        let mut colliders = old.scene.colliders.clone();
        let foot = old.try_spawn(8011., 8000.).unwrap();
        let p = old.frame.to_local(foot).unwrap().metres();
        colliders.push(PlacedCollider {
            shape: Collider::cuboid(0.2, 4., 12.),
            center: Vec3::new(p[0], p[1] + 2., p[2]),
            layer: SOLID,
        });
        prepared.patch = Arc::new(procedural::Patch {
            owner: old.owner,
            frame: old.frame,
            ids: old.ids.clone(),
            scene: StaticScene::new(colliders),
        });
        world.enable_patrol(prepared).unwrap();
        for _ in 0..300 {
            world.step();
        }
        let view = world.patrol_snapshot().unwrap();
        assert_eq!(view.mode, Mode::Blocked);
        assert_eq!(view.blocked, 1);
        assert!(view.position[0] < 8010.7);
        let blocked_key = world.patrol.as_ref().unwrap().key;
        let blocked_facing = world.patrol_facing(blocked_key).unwrap();
        for _ in 0..30 {
            world.step();
            assert_eq!(world.patrol_facing(blocked_key), Some(blocked_facing));
        }
        let (mut world, key) = fixture(TickRate::Hz30);
        let before = world.patrol.as_ref().unwrap().patch.scene.colliders.len();
        let player = actor(&mut world);
        let mut overlap = false;
        assert!(!world.input(
            key.entity.value(),
            Input {
                version: VERSION,
                epoch: world.epoch(),
                sequence: 1,
                intent: Intent {
                    axes: [1000, 0],
                    ..Default::default()
                }
            }
        ));
        for sequence in 1..=150 {
            assert!(world.input(
                player,
                Input {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence,
                    intent: Intent {
                        axes: [1000, 0],
                        ..Default::default()
                    }
                }
            ));
            world.step();
            let a = world.global_position(player).unwrap().metres();
            let b = world.patrol_snapshot().unwrap().position;
            overlap |= (a[0] - b[0]).hypot(a[2] - b[2]) < 0.1;
        }
        assert!(overlap);
        assert!(world.global_position(player).unwrap().metres()[0] > 8024.9);
        assert_eq!(
            world.patrol.as_ref().unwrap().patch.scene.colliders.len(),
            before
        );
    }
    #[test]
    fn source_profile_route_bounds_retirement_and_collision_residency_fail_closed() {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        let mut prepared = preparation(&world, [15048., 15040.]);
        let owner = prepared.patch.owner;
        let baseline = world.identity();
        prepared.source = [8; 32];
        assert!(world.enable_patrol(prepared).is_err());
        assert_eq!(world.identity(), baseline);
        let prepared = preparation(&world, [15048., 15040.]);
        let key = world.enable_patrol(prepared).unwrap();
        assert_ne!(world.identity(), baseline);
        assert_eq!(world.procedural.as_ref().unwrap().generated, 2);
        for _ in 0..10 {
            world.poll_terrain();
            assert!(
                world
                    .procedural
                    .as_ref()
                    .unwrap()
                    .patches
                    .contains_key(&owner)
            );
            world.step();
        }
        let admitted_identity = world.identity();
        assert!(world.remove_development_target(key));
        assert_eq!(world.identity(), admitted_identity);
        assert!(world.patrol_snapshot().is_none());
        for _ in 0..100 {
            world.step();
        }
        assert!(world.targets.is_empty());
        world.poll_terrain();
        assert!(
            !world
                .procedural
                .as_ref()
                .unwrap()
                .patches
                .contains_key(&owner)
        );
        let mut raw: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
        raw["speed_metres_per_second"] = 6.into();
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
        assert!(Profile::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1]).is_err());
        let mut prepared = preparation(&world, [8008., 8000.]);
        prepared.points.pop();
        assert!(
            Prepared::checked(
                prepared.profile,
                prepared.navigation,
                prepared.source,
                prepared.points,
                prepared.patch
            )
            .is_err()
        );
    }
    #[test]
    fn authoritative_patrol_facing_follows_successful_motion_and_holds_through_waits() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (mut world, key) = fixture(rate);
            let initial = world.patrol_facing(key).unwrap();
            assert_eq!(initial.units(), 49152);
            let mut headings = std::collections::BTreeSet::new();
            for _ in 0..rate.hz() * 22 {
                let before = world.patrol_snapshot().unwrap();
                let old_facing = world.patrol_facing(key).unwrap();
                world.step();
                let after = world.patrol_snapshot().unwrap();
                let a = GlobalPosition::new(before.position).unwrap();
                let b = GlobalPosition::new(after.position).unwrap();
                let facing = world.patrol_facing(key).unwrap();
                if let Some(expected) = Facing::towards(a, b) {
                    assert!(facing.difference(expected).abs() < 0.0001);
                } else {
                    assert_eq!(facing, old_facing);
                }
                headings.insert(facing.units());
            }
            assert!(headings.len() >= 3, "Expected route turns: {headings:?}");
            assert!(
                world
                    .patrol_facing(EntityKey {
                        epoch: theia_core::identity::WorldEpoch::new(world.epoch() + 1).unwrap(),
                        ..key
                    })
                    .is_none()
            );
        }
    }
}
