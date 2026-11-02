//! One clock per authoritative actor. Cooldown survives stop/re-engage/retarget.
//! Eligibility is supplied by authoritative hit validation, never by a client.
use crate::identity::EntityKey;
pub const MAX_PERIOD_TICKS: u64 = 600;
pub const MAX_DELAY_TICKS: u64 = 3600;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Idle,
    Paused,
    Waiting { ready_at: u64 },
    Swing { target: EntityKey, next_at: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    owner: EntityKey,
    target: Option<EntityKey>,
    period: u64,
    ready_at: u64,
    observed_at: u64,
    suspended_at: Option<u64>,
}
impl Clock {
    /// Initial readiness is a Theia prototype policy, not a sourced Classic rule.
    pub fn new(owner: EntityKey, period_ticks: u64, now: u64) -> Result<Self, &'static str> {
        Self::valid_period(period_ticks)?;
        Ok(Self {
            owner,
            target: None,
            period: period_ticks,
            ready_at: now,
            observed_at: now,
            suspended_at: None,
        })
    }
    fn valid_period(period: u64) -> Result<(), &'static str> {
        if !(1..=MAX_PERIOD_TICKS).contains(&period) {
            return Err("Invalid melee period");
        }
        Ok(())
    }
    fn check_time(&self, now: u64) -> Result<(), &'static str> {
        if now < self.observed_at {
            return Err("Melee time moved backwards");
        }
        Ok(())
    }
    pub fn target(&self) -> Option<EntityKey> {
        self.target
    }
    pub fn period_ticks(&self) -> u64 {
        self.period
    }
    pub fn ready_at(&self) -> u64 {
        self.ready_at
    }
    pub fn suspended(&self) -> bool {
        self.suspended_at.is_some()
    }
    /// Repeated engagement and target changes never reset readiness.
    pub fn engage(&mut self, target: EntityKey, now: u64) -> Result<(), &'static str> {
        self.check_time(now)?;
        if target.epoch != self.owner.epoch || target == self.owner {
            return Err("Invalid melee target incarnation");
        }
        self.target = Some(target);
        self.observed_at = now;
        Ok(())
    }
    pub fn stop(&mut self, now: u64) -> Result<(), &'static str> {
        self.check_time(now)?;
        self.target = None;
        self.observed_at = now;
        Ok(())
    }
    pub fn invalidate(&mut self, target: EntityKey, now: u64) -> Result<bool, &'static str> {
        self.check_time(now)?;
        let matched = self.target == Some(target);
        if matched {
            self.target = None;
        }
        self.observed_at = now;
        Ok(matched)
    }
    /// Frozen countdown; nested repeated suspend is idempotent.
    pub fn suspend(&mut self, now: u64) -> Result<(), &'static str> {
        self.check_time(now)?;
        self.suspended_at.get_or_insert(now);
        self.observed_at = now;
        Ok(())
    }
    pub fn resume(&mut self, now: u64) -> Result<(), &'static str> {
        self.check_time(now)?;
        if let Some(at) = self.suspended_at {
            let ready = now
                .checked_add(self.ready_at.saturating_sub(at))
                .ok_or("Melee time exhausted")?;
            self.ready_at = ready;
            self.suspended_at = None;
        }
        self.observed_at = now;
        Ok(())
    }
    /// Delay is an explicit ability rule. The caller validates its ability/cost.
    pub fn delay(&mut self, now: u64, ticks: u64) -> Result<(), &'static str> {
        self.check_time(now)?;
        if ticks > MAX_DELAY_TICKS {
            return Err("Melee delay exceeds budget");
        }
        let anchor = self.suspended_at.unwrap_or(now);
        let ready = self
            .ready_at
            .max(anchor)
            .checked_add(ticks)
            .ok_or("Melee time exhausted")?;
        self.ready_at = ready;
        self.observed_at = now;
        Ok(())
    }
    /// Explicit reset for a committed weapon change or a reset-type ability.
    /// Suspending remains active; its remaining countdown is replaced.
    pub fn reset(&mut self, now: u64, period_ticks: u64) -> Result<(), &'static str> {
        self.check_time(now)?;
        Self::valid_period(period_ticks)?;
        let anchor = self.suspended_at.unwrap_or(now);
        let ready = anchor
            .checked_add(period_ticks)
            .ok_or("Melee time exhausted")?;
        self.period = period_ticks;
        self.ready_at = ready;
        self.observed_at = now;
        Ok(())
    }
    /// At most one swing per call/tick. A long absence stores no catch-up burst.
    /// Pausing eligibility leaves engagement intact and accrues no extra swings.
    /// Errors do not mutate the clock or authorize an attack.
    pub fn step(&mut self, now: u64, eligible: bool) -> Result<Step, &'static str> {
        self.check_time(now)?;
        let result = match self.target {
            None => Step::Idle,
            Some(_) if !eligible || self.suspended_at.is_some() => Step::Paused,
            Some(_) if now < self.ready_at => Step::Waiting {
                ready_at: self.ready_at,
            },
            Some(target) => {
                let next_at = now.checked_add(self.period).ok_or("Melee time exhausted")?;
                self.ready_at = next_at;
                Step::Swing { target, next_at }
            }
        };
        self.observed_at = now;
        Ok(result)
    }
}
