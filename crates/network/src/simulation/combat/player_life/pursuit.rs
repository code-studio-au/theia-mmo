//! Opt-in single-creature pursuit. Searches/polling stay on the owner pump and
//! dedicated worker; fixed ticks consume only bounded immutable replies.
use crate::{
    Result,
    navigation::worker::{Reply, Request, Tag, Worker},
    simulation::creatures::PursuitMotion,
};
use std::time::Duration;
use theia_core::{
    coordinates::GlobalPosition, identity::EntityKey, pursuit as policy, time::TickRate,
};

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub acquisitions: u64,
    pub chase_ticks: u64,
    pub attack_ticks: u64,
    pub returns: u64,
    pub arrivals: u64,
    pub blocked: u64,
    pub requests: u64,
    pub published: u64,
    pub rejected: u64,
    pub failed: u64,
    pub last_anchor: Option<[f64; 3]>,
}
struct Path {
    points: Vec<GlobalPosition>,
    index: usize,
    expires: u64,
    mode: policy::Mode,
    target: Option<theia_core::player_life::LifeKey>,
}
pub(super) struct Runtime {
    settings: policy::Settings,
    state: policy::State,
    rate: TickRate,
    worker: Worker,
    reply: Option<Reply>,
    path: Option<Path>,
    tag: Option<Tag>,
    revision: u64,
    next_request: u64,
    requested_goal: Option<GlobalPosition>,
    failed_route: bool,
    next_scan: u64,
    metrics: Metrics,
}
fn distance(a: GlobalPosition, b: GlobalPosition) -> f64 {
    a.metres()
        .into_iter()
        .zip(b.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
// Match prepared route support tolerance. Collision owns actual grounding;
// a baked detail-height approximation must not stall a reached XZ waypoint.
fn waypoint_reached(goal: GlobalPosition, position: GlobalPosition) -> bool {
    let a = goal.metres();
    let b = position.metres();
    (a[0] - b[0]).hypot(a[2] - b[2]) <= 0.1 && (a[1] - b[1]).abs() <= 0.25
}
fn feet(p: GlobalPosition) -> Result<GlobalPosition> {
    let mut p = p.metres();
    p[1] -= f64::from(crate::world::PAWN_Y);
    GlobalPosition::new(p).ok_or_else(|| "Invalid pursuit feet".into())
}
fn centre(p: GlobalPosition) -> Result<GlobalPosition> {
    let mut p = p.metres();
    p[1] += f64::from(crate::world::PAWN_Y);
    GlobalPosition::new(p).ok_or_else(|| "Invalid pursuit centre".into())
}
impl Runtime {
    pub(super) fn new(
        settings: policy::Settings,
        npc: EntityKey,
        rate: TickRate,
        worker: Worker,
    ) -> Result<Self> {
        let state =
            policy::State::new(settings, npc, rate).map_err(|_| "Invalid pursuit settings")?;
        Ok(Self {
            settings,
            state,
            rate,
            worker,
            reply: None,
            path: None,
            tag: None,
            revision: 1,
            next_request: 0,
            requested_goal: None,
            failed_route: false,
            next_scan: 0,
            metrics: Metrics::default(),
        })
    }
    pub(super) fn reset(&mut self, npc: EntityKey) -> Result<()> {
        self.state = policy::State::new(self.settings, npc, self.rate)
            .map_err(|_| "Invalid pursuit reset")?;
        self.invalidate()?;
        Ok(())
    }
    fn invalidate(&mut self) -> Result<()> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or("Pursuit revision exhausted")?;
        self.path = None;
        self.reply = None;
        self.tag = None;
        self.requested_goal = None;
        self.failed_route = false;
        self.next_scan = 0;
        Ok(())
    }
    pub(super) fn settings(&self) -> policy::Settings {
        self.settings
    }
    pub(super) fn directive(&self) -> policy::Directive {
        self.state.directive()
    }
    pub(super) fn metrics(&self) -> Metrics {
        self.metrics
    }
    pub(super) fn scan_due(&mut self, now: u64) -> Result<bool> {
        if now < self.next_scan {
            return Ok(false);
        }
        self.next_scan = now
            .checked_add(self.rate.ticks_for(Duration::from_millis(100)).unwrap())
            .ok_or("Perception clock exhausted")?;
        Ok(true)
    }
    /// Owner pump only. Never starts a search or waits for one.
    pub(super) fn poll(&mut self) -> Result<()> {
        if let Some(reply) = self
            .worker
            .poll()
            .map_err(|_| "Pursuit worker closed or mismatched")?
        {
            self.reply = Some(reply);
        }
        Ok(())
    }
    pub(super) fn advance(
        &mut self,
        now: u64,
        npc: EntityKey,
        mut observation: policy::Observation<'_>,
    ) -> Result<PursuitMotion> {
        observation.blocked |= self.failed_route;
        self.failed_route = false;
        let previous = self.state.directive().mode;
        let directive = self
            .state
            .advance(now, observation)
            .map_err(|e| format!("Pursuit policy failed: {e:?}"))?;
        if previous != directive.mode {
            // Chase/Attack share a target, but a phase change must not retain
            // movement from the preceding phase or an old return request.
            self.invalidate()?;
            if previous == policy::Mode::Patrol
                && matches!(directive.mode, policy::Mode::Chase | policy::Mode::Attack)
            {
                self.metrics.acquisitions += 1;
                self.metrics.last_anchor = directive.anchor.map(|p| p.metres());
            }
            if directive.mode == policy::Mode::Return {
                self.metrics.returns += 1;
            }
            if previous == policy::Mode::Return && directive.mode == policy::Mode::Patrol {
                self.metrics.arrivals += 1;
            }
            if directive.mode == policy::Mode::Blocked {
                self.metrics.blocked += 1;
            }
        }
        self.metrics.chase_ticks += u64::from(directive.mode == policy::Mode::Chase);
        self.metrics.attack_ticks += u64::from(directive.mode == policy::Mode::Attack);
        let look_at = (directive.mode == policy::Mode::Attack)
            .then(|| {
                observation
                    .target
                    .into_iter()
                    .chain(observation.provoked)
                    .chain(observation.candidates.iter().copied())
                    .find(|target| {
                        Some(target.key) == directive.target && target.alive && target.visible
                    })
                    .map(|target| target.position)
            })
            .flatten();
        let Some(goal) = directive.goal else {
            return Ok(PursuitMotion {
                mode: directive.mode,
                goal: None,
                speed: directive.speed,
                look_at,
            });
        };
        if distance(goal, observation.position) <= 0.1 {
            self.path = None;
            return Ok(PursuitMotion {
                mode: directive.mode,
                goal: None,
                speed: directive.speed,
                look_at,
            });
        }
        if observation.blocked {
            self.path = None;
            self.requested_goal = None;
        }
        if let Some(reply) = self.reply.take() {
            let points = self
                .tag
                .and_then(|tag| reply.current(self.worker.navigation(), tag, now));
            if let Some(points) = points.filter(|_| {
                centre(reply.request.start).is_ok_and(|p| distance(p, observation.position) <= 1.)
                    && centre(reply.request.end).is_ok_and(|p| distance(p, goal) <= 2.)
            }) {
                let points = points
                    .iter()
                    .copied()
                    .map(centre)
                    .collect::<Result<Vec<_>>>()?;
                self.path = Some(Path {
                    points,
                    index: 1,
                    expires: reply.request.expires,
                    mode: directive.mode,
                    target: directive.target,
                });
                self.metrics.published += 1;
            } else {
                self.metrics.rejected += 1;
                let current = self.tag.is_some_and(|tag| reply.request.tag == tag)
                    && reply.navigation == self.worker.navigation()
                    && (reply.request.submitted..=reply.request.expires).contains(&now);
                // A missing/disconnected route never produces movement. The
                // next tick feeds confirmed failure back to the policy.
                if current && reply.route.is_err() {
                    self.metrics.failed += 1;
                    self.failed_route = true;
                    self.path = None;
                    self.requested_goal = None;
                }
            }
        }
        if self.path.as_ref().is_some_and(|p| {
            now > p.expires || p.mode != directive.mode || p.target != directive.target
        }) {
            self.path = None;
        }
        if let Some(path) = &mut self.path {
            while path.index < path.points.len()
                && waypoint_reached(path.points[path.index], observation.position)
            {
                path.index += 1;
            }
        }
        let refresh = self.requested_goal.is_none_or(|p| distance(p, goal) > 0.5)
            || self.path.as_ref().is_none_or(|p| {
                p.index >= p.points.len()
                    || p.expires.saturating_sub(now) <= u64::from(self.rate.hz()) / 4
            });
        if refresh && now >= self.next_request && self.worker.pending().is_none() {
            let revision = self
                .revision
                .checked_add(1)
                .ok_or("Pursuit revision exhausted")?;
            let tag = Tag {
                creature: npc,
                target: directive.target,
                revision,
            };
            let expires = now
                .checked_add(u64::from(self.rate.hz()))
                .ok_or("Pursuit clock exhausted")?;
            let request = Request {
                tag,
                start: feet(observation.position)?,
                end: feet(goal)?,
                submitted: now,
                expires,
            };
            self.worker
                .submit(request)
                .map_err(|e| format!("Pursuit request refused: {e:?}"))?;
            self.revision = revision;
            self.tag = Some(tag);
            self.requested_goal = Some(goal);
            self.next_request = now
                .checked_add(self.rate.ticks_for(Duration::from_millis(250)).unwrap())
                .ok_or("Pursuit clock exhausted")?;
            self.metrics.requests += 1;
        }
        let waypoint = self
            .path
            .as_ref()
            .and_then(|p| p.points.get(p.index))
            .copied();
        Ok(PursuitMotion {
            mode: directive.mode,
            goal: waypoint,
            speed: directive.speed,
            look_at,
        })
    }
}

impl super::Combat {
    pub(in crate::simulation) fn poll_pursuit(&mut self) {
        if let Some(pve) = &mut self.pve
            && let Some(runtime) = &mut pve.pursuit
            && let Err(error) = runtime.poll()
        {
            pve.error = Some(error.to_string());
        }
    }
    fn pursuit_target(
        &self,
        world: &crate::simulation::World,
        npc: EntityKey,
        id: u64,
        vision: f64,
        reach: f64,
    ) -> Option<policy::Target> {
        let player = self.actors.get(&id)?.life.as_ref()?;
        let key = player.state.key();
        if world.entity_key(id) != Some(key.actor) {
            return None;
        }
        let position = world.global_position(id)?;
        let creature = self.creatures.iter().find(|c| c.key == npc)?;
        let visible = self.reachable(world, npc, key, vision);
        Some(policy::Target {
            key,
            position,
            alive: player.state.alive(),
            visible,
            in_melee: visible && distance(position, creature.position) <= reach,
        })
    }
    pub(in crate::simulation) fn advance_pursuit(
        &mut self,
        world: &crate::simulation::World,
    ) -> Option<PursuitMotion> {
        let mut pve = self.pve.take()?;
        let Some(mut runtime) = pve.pursuit.take() else {
            self.pve = Some(pve);
            return None;
        };
        let result = (|| -> Result<PursuitMotion> {
            if pve.error.is_some() {
                return Err("Pursuit is halted after an authority error".into());
            }
            let (position, alive) = self
                .creatures
                .iter()
                .find(|c| c.key == pve.npc)
                .map(|c| (c.position, c.current > 0))
                // Explicit despawn ends engagement too. Position is unused by
                // the dead policy; it must not turn a valid retirement fatal.
                .unwrap_or((world.frame.origin(), false));
            let settings = runtime.settings();
            let directive = runtime.directive();
            let reach = pve.profile.retaliation().reach_metres();
            let target = directive.target.and_then(|key| {
                self.pursuit_target(
                    world,
                    pve.npc,
                    key.actor.entity.value(),
                    settings.leash_metres,
                    reach,
                )
            });
            let provoked = pve.policy.target().and_then(|key| {
                self.pursuit_target(
                    world,
                    pve.npc,
                    key.actor.entity.value(),
                    settings.leash_metres,
                    reach,
                )
                .filter(|t| t.key == key)
            });
            let candidates = if directive.mode == policy::Mode::Patrol
                && runtime.scan_due(world.tick)?
            {
                world
                    .players
                    .keys()
                    .filter_map(|id| {
                        self.pursuit_target(world, pve.npc, *id, settings.perception_metres, reach)
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let blocked = world.patrol.as_ref().is_some_and(|p| p.motion_blocked);
            runtime.advance(
                world.tick,
                pve.npc,
                policy::Observation {
                    position,
                    alive,
                    candidates: &candidates,
                    target,
                    provoked,
                    blocked,
                },
            )
        })();
        let motion = match result {
            Ok(motion) => motion,
            Err(error) => {
                pve.error = Some(error.to_string());
                PursuitMotion {
                    mode: policy::Mode::Blocked,
                    goal: None,
                    speed: 0.,
                    look_at: None,
                }
            }
        };
        if matches!(motion.mode, policy::Mode::Chase | policy::Mode::Attack) && pve.error.is_none()
        {
            if let Some(key) = runtime.directive().target
                && let Err(error) = pve.policy.record_hit(world.tick, key)
            {
                pve.error = Some(format!("Pursuit engagement failed: {error:?}"));
            }
        } else {
            pve.metrics.cancelled = pve
                .metrics
                .cancelled
                .saturating_add(u64::from(pve.policy.windup_until().is_some()));
            pve.policy =
                theia_core::retaliation::State::new(pve.profile.retaliation().clone(), world.rate);
        }
        pve.pursuit = Some(runtime);
        self.pve = Some(pve);
        Some(motion)
    }
}
impl crate::simulation::World {
    pub fn creature_pursuit_metrics(&self) -> Option<Metrics> {
        Some(
            self.combat
                .as_ref()?
                .pve
                .as_ref()?
                .pursuit
                .as_ref()?
                .metrics(),
        )
    }
    pub fn creature_pursuit_directive(&self) -> Option<policy::Directive> {
        Some(
            self.combat
                .as_ref()?
                .pve
                .as_ref()?
                .pursuit
                .as_ref()?
                .directive(),
        )
    }
}

#[cfg(test)]
mod tests;
