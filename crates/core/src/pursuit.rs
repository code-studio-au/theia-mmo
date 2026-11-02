//! Pure single-target perception/chase/leash policy. No paths, damage or I/O.
use crate::{
    coordinates::GlobalPosition, identity::EntityKey, player_life::LifeKey, time::TickRate,
};
use serde::Deserialize;
use std::time::Duration;

pub const MAX_CANDIDATES: usize = 256;
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub perception_metres: f64,
    pub leash_metres: f64,
    pub chase_speed: f32,
    pub return_speed: f32,
    pub los_grace_ms: u32,
    /// Soft pursuit allowance, refreshed by authoritative visible melee contact.
    /// This never extends the captured-anchor distance boundary.
    pub engagement_ms: u32,
    pub reacquire_ms: u32,
}
impl Settings {
    pub fn valid(self) -> bool {
        self.perception_metres.is_finite()
            && (1. ..=30.).contains(&self.perception_metres)
            && self.leash_metres.is_finite()
            && (self.perception_metres..=128.).contains(&self.leash_metres)
            && self.chase_speed.is_finite()
            && (0.1..=5.).contains(&self.chase_speed)
            && self.return_speed.is_finite()
            && (0.1..=5.).contains(&self.return_speed)
            && (100..=5000).contains(&self.los_grace_ms)
            && (1000..=60000).contains(&self.engagement_ms)
            && self.los_grace_ms < self.engagement_ms
            && (500..=10000).contains(&self.reacquire_ms)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub key: LifeKey,
    pub position: GlobalPosition,
    pub alive: bool,
    pub visible: bool,
    /// Current server-side melee reach and static LOS result.
    pub in_melee: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Patrol,
    Chase,
    Attack,
    Return,
    Blocked,
    Dead,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidSettings,
    CandidateBudget,
    StaleTick,
    TimeExhausted,
    InvalidTransfer,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Directive {
    pub mode: Mode,
    pub target: Option<LifeKey>,
    pub anchor: Option<GlobalPosition>,
    pub goal: Option<GlobalPosition>,
    pub speed: f32,
    pub can_attack: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Observation<'a> {
    pub position: GlobalPosition,
    pub alive: bool,
    pub candidates: &'a [Target],
    /// Exact current life observation for the previously selected target.
    pub target: Option<Target>,
    /// Current first attacker, supplied by authoritative damage attribution.
    pub provoked: Option<Target>,
    /// A confirmed unusable chase route, not merely a queued query.
    pub blocked: bool,
}
fn distance(a: GlobalPosition, b: GlobalPosition) -> f64 {
    a.metres()
        .into_iter()
        .zip(b.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
fn valid_target(creature: EntityKey, target: Target) -> bool {
    target.alive && target.key.actor.epoch == creature.epoch && target.key.actor != creature
}
pub fn select(
    creature: EntityKey,
    position: GlobalPosition,
    settings: Settings,
    candidates: &[Target],
) -> Result<Option<Target>, Error> {
    if !settings.valid() {
        return Err(Error::InvalidSettings);
    }
    if candidates.len() > MAX_CANDIDATES {
        return Err(Error::CandidateBudget);
    }
    Ok(candidates
        .iter()
        .copied()
        .filter(|t| {
            valid_target(creature, *t)
                && t.visible
                && distance(position, t.position) <= settings.perception_metres
        })
        .min_by(|a, b| {
            distance(position, a.position)
                .total_cmp(&distance(position, b.position))
                .then_with(|| a.key.actor.entity.cmp(&b.key.actor.entity))
        }))
}
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    settings: Settings,
    creature: EntityKey,
    rate: TickRate,
    advanced: Option<u64>,
    mode: Mode,
    target: Option<LifeKey>,
    anchor: Option<GlobalPosition>,
    last_seen: Option<GlobalPosition>,
    expires: Option<u64>,
    lost_since: Option<u64>,
    next_acquire: Option<u64>,
}
impl State {
    pub fn new(settings: Settings, creature: EntityKey, rate: TickRate) -> Result<Self, Error> {
        if !settings.valid() {
            return Err(Error::InvalidSettings);
        }
        Ok(Self {
            settings,
            creature,
            rate,
            advanced: None,
            mode: Mode::Patrol,
            target: None,
            anchor: None,
            last_seen: None,
            expires: None,
            lost_since: None,
            next_acquire: None,
        })
    }
    pub fn directive(&self) -> Directive {
        Directive {
            mode: self.mode,
            target: self.target,
            anchor: self.anchor,
            goal: match self.mode {
                Mode::Chase => self.last_seen,
                Mode::Return => self.anchor,
                _ => None,
            },
            speed: match self.mode {
                Mode::Chase => self.settings.chase_speed,
                Mode::Return => self.settings.return_speed,
                _ => 0.,
            },
            can_attack: self.mode == Mode::Attack,
        }
    }
    /// Prepared authority-only victim transfer, before this tick's evaluation.
    /// Preserve the original patrol anchor and soft allowance; changing victims
    /// never creates a new leash origin. The initial transfer requires a live,
    /// visible same-world target within the existing captured boundary.
    pub fn transfer(
        &mut self,
        now: u64,
        position: GlobalPosition,
        target: Target,
    ) -> Result<Directive, Error> {
        if self.advanced.is_some_and(|last| now <= last) {
            return Err(Error::StaleTick);
        }
        let anchor = self.anchor.ok_or(Error::InvalidTransfer)?;
        if !matches!(self.mode, Mode::Chase | Mode::Attack)
            || !valid_target(self.creature, target)
            || !target.visible
            || distance(anchor, position) > self.settings.leash_metres
            || distance(anchor, target.position) > self.settings.leash_metres
        {
            return Err(Error::InvalidTransfer);
        }
        if self.target == Some(target.key) {
            return Ok(self.directive());
        }
        self.target = Some(target.key);
        self.last_seen = Some(target.position);
        self.lost_since = None;
        self.mode = if target.in_melee {
            Mode::Attack
        } else {
            Mode::Chase
        };
        Ok(self.directive())
    }
    fn deadline(&self, now: u64, ms: u32) -> Result<u64, Error> {
        now.checked_add(
            self.rate
                .ticks_for(Duration::from_millis(u64::from(ms)))
                .unwrap(),
        )
        .ok_or(Error::TimeExhausted)
    }
    fn evade(&mut self, now: u64) -> Result<(), Error> {
        self.mode = Mode::Return;
        self.target = None;
        self.last_seen = None;
        self.lost_since = None;
        self.expires = Some(self.deadline(now, self.settings.engagement_ms)?);
        Ok(())
    }
    /// Atomic on refusal; an evaluated tick returns its existing directive.
    pub fn advance(&mut self, now: u64, observation: Observation<'_>) -> Result<Directive, Error> {
        if self.advanced.is_some_and(|tick| now < tick) {
            return Err(Error::StaleTick);
        }
        if self.advanced == Some(now) {
            return Ok(self.directive());
        }
        if observation.candidates.len() > MAX_CANDIDATES {
            return Err(Error::CandidateBudget);
        }
        let mut next = self.clone();
        next.update(now, observation)?;
        next.advanced = Some(now);
        *self = next;
        Ok(self.directive())
    }
    fn update(&mut self, now: u64, observation: Observation<'_>) -> Result<(), Error> {
        if !observation.alive {
            self.mode = Mode::Dead;
            self.target = None;
            self.anchor = None;
            self.last_seen = None;
            self.expires = None;
            self.lost_since = None;
            self.next_acquire = None;
            return Ok(());
        }
        match self.mode {
            Mode::Dead | Mode::Blocked => return Ok(()),
            Mode::Return => {
                if self
                    .anchor
                    .is_some_and(|anchor| distance(anchor, observation.position) <= 0.1)
                {
                    self.next_acquire = Some(self.deadline(now, self.settings.reacquire_ms)?);
                    self.mode = Mode::Patrol;
                    self.anchor = None;
                    self.expires = None;
                } else if self.expires.is_some_and(|until| now >= until) {
                    self.mode = Mode::Blocked;
                }
                return Ok(());
            }
            Mode::Patrol => {
                if self.next_acquire.is_some_and(|until| now < until) {
                    return Ok(());
                }
                let provoked = observation.provoked.filter(|target| {
                    valid_target(self.creature, *target)
                        && distance(observation.position, target.position)
                            <= self.settings.leash_metres
                });
                let candidate = match provoked {
                    Some(target) => Some(target),
                    None => select(
                        self.creature,
                        observation.position,
                        self.settings,
                        observation.candidates,
                    )?,
                };
                if let Some(target) = candidate {
                    self.anchor = Some(observation.position);
                    self.target = Some(target.key);
                    self.last_seen = Some(target.position);
                    self.expires = Some(self.deadline(now, self.settings.engagement_ms)?);
                    self.mode = if target.in_melee {
                        Mode::Attack
                    } else {
                        Mode::Chase
                    };
                }
                return Ok(());
            }
            Mode::Chase | Mode::Attack => {}
        }
        let Some(target) = observation
            .target
            .filter(|t| Some(t.key) == self.target && valid_target(self.creature, *t))
        else {
            return self.evade(now);
        };
        let anchor = self.anchor.unwrap();
        if observation.blocked
            || distance(anchor, observation.position) > self.settings.leash_metres
            || distance(anchor, target.position) > self.settings.leash_metres
        {
            return self.evade(now);
        }
        // Ordinary melee combat must not end merely because acquisition was
        // thirty seconds ago. Reach includes server-side static LOS; misses and
        // avoidance do not break engagement. This is a Theia contact policy,
        // not a claim about Classic's exact damage/threat refresh algorithm.
        if target.visible && target.in_melee {
            self.expires = Some(self.deadline(now, self.settings.engagement_ms)?);
        } else if self.expires.is_some_and(|until| now >= until) {
            return self.evade(now);
        }
        if target.visible {
            self.lost_since = None;
            self.last_seen = Some(target.position);
        } else {
            let since = *self.lost_since.get_or_insert(now);
            if now >= self.deadline(since, self.settings.los_grace_ms)? {
                return self.evade(now);
            }
        }
        self.mode = if target.visible && target.in_melee {
            Mode::Attack
        } else {
            Mode::Chase
        };
        Ok(())
    }
}
