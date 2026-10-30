//! One opt-in nonblocking patrol, consuming an offline checked route. No path
//! search, file access, wall clock, database or transport I/O in the fixed step.
use super::World;
use crate::{Result, kinematic, navigation, procedural, wire::Intent};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use theia_core::{
    content::ContentId,
    coordinates::{GlobalPosition, LocalPosition},
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
}
#[derive(Clone)]
pub struct Profile {
    id: ContentId,
    route: ContentId,
    speed: f32,
    wait_ms: u32,
    digest: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Creature profile exceeds byte budget".into());
        }
        let raw: RawProfile = serde_json::from_slice(bytes)?;
        if raw.schema != 1
            || !raw.speed_metres_per_second.is_finite()
            || !(0.1..=5.).contains(&raw.speed_metres_per_second)
            || !(100..=60000).contains(&raw.wait_ms)
        {
            return Err("Creature profile exceeds initial patrol bounds".into());
        }
        Ok(Self {
            id: ContentId::new(&raw.id).ok_or("Invalid creature ID")?,
            route: ContentId::new(&raw.route).ok_or("Invalid creature route ID")?,
            speed: raw.speed_metres_per_second,
            wait_ms: raw.wait_ms,
            digest: crate::terrain::sha256(bytes),
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn speed_metres_per_second(&self) -> f32 {
        self.speed
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
        Self::checked(profile, store.identity(), source.identity(), points, patch)
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
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Idle,
    Patrol,
    Dead,
    Blocked,
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
    pub(super) key: EntityKey,
    points: Vec<GlobalPosition>,
    pub(super) patch: Arc<procedural::Patch>,
    coverage: crate::chunks::Coverage,
    body: kinematic::Motion,
    home: GlobalPosition,
    position: GlobalPosition,
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
    pub(super) fn definition(&self) -> &ContentId {
        self.profile.id()
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
        self.body = kinematic::Motion::default();
        self.waypoint = 1;
        self.wait_until = now.checked_add(self.wait_ticks);
        self.mode = Mode::Idle;
        self.respawns += 1;
        if self.wait_until.is_none() {
            self.block();
        }
        self.home
    }
    fn advance(&mut self, world: &mut World) {
        if !world.targets.contains_key(&self.key.entity.value()) {
            self.mode = Mode::Dead;
            self.body = kinematic::Motion::default();
            return;
        }
        if self.mode == Mode::Blocked {
            return;
        }
        let now = world.tick;
        let waiting = self.wait_until.is_some_and(|until| now <= until);
        let goal = self.points[self.waypoint].metres();
        let p = self.position.metres();
        let dx = goal[0] - p[0];
        let dz = goal[2] - p[2];
        let distance = dx.hypot(dz);
        let mut intent = Intent::default();
        if waiting {
            self.mode = Mode::Idle;
            self.idle_ticks += 1;
        } else {
            self.mode = Mode::Patrol;
            self.wait_until = None;
            let travel = f64::from(self.profile.speed) * f64::from(world.rate.seconds());
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
            self.block();
            return;
        }
        if !world.move_patrol_target(self.key, position) {
            self.body = before;
            self.mode = Mode::Dead;
            return;
        }
        self.position = position;
        if !waiting {
            self.moving_ticks += 1;
            self.travelled += horizontal;
            if (goal[0] - next[0]).hypot(goal[2] - next[2]) <= ARRIVAL_METRES {
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
    /// Installation only: consumes already prepared collision/data, no file I/O.
    pub fn enable_patrol(&mut self, prepared: Prepared) -> Result<EntityKey> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.combat.is_some()
            || self.patrol_identity.is_some()
            || self.targets.len() >= crate::targeting::MAX_TARGETS
            || self.next_id == u64::MAX
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
        self.patrol = Some(Patrol {
            profile: prepared.profile,
            key,
            points: prepared.points,
            patch: prepared.patch,
            coverage,
            body: kinematic::Motion::default(),
            home,
            position: home,
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
        if let Some(mut patrol) = self.patrol.take() {
            patrol.advance(self);
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
mod tests {
    use super::*;
    use crate::{
        simulation::combat::{Event, Outcome, Request},
        wire::{ChunkReady, Input, VERSION},
    };
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
    fn fixture(rate: TickRate) -> (World, EntityKey) {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        assert!(world.set_tick_rate(rate));
        let prepared = preparation(&world, [8008., 8000.]);
        let key = world.enable_patrol(prepared).unwrap();
        (world, key)
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
}
