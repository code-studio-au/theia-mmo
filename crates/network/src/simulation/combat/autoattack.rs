//! Opt-in authoritative basic melee. Normal application presentation is separate.
use super::{Authority, Event, Reason};
use crate::{Result, simulation::World};
use theia_core::abilities::{Ability, classic::Swing};
use theia_core::{
    content::ContentId,
    identity::EntityKey,
    inventory,
    melee::{
        self,
        timer::{Clock, Step},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pause {
    NoWeapon,
    InvalidTarget,
    DeadPlayer,
    NotReady,
    OutOfRange,
    NotFacing,
    Obstructed,
    Casting,
    EventBudget,
    TimeExhausted,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    Engaged,
    Attacking,
    Paused(Pause),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ignored,
    Rejected(Pause),
    Engaged,
    Stopped,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub sequence: u64,
    pub outcome: Outcome,
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub stream_epoch: u64,
    pub epoch: u64,
    pub sequence: u64,
    pub target: Option<EntityKey>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub actor: EntityKey,
    pub stream_epoch: u64,
    pub tick: u64,
    pub target: Option<EntityKey>,
    pub phase: Phase,
    pub weapon: Option<ContentId>,
    pub ready_at: Option<u64>,
    pub swing_sequence: u64,
    pub reply: Option<Reply>,
}
#[derive(Default)]
pub(super) struct Player {
    sequence: u64,
    reply: Option<Reply>,
    clock: Option<Clock>,
    weapon: Option<ContentId>,
    phase: Phase,
    swings: u64,
}
pub(super) struct Profile {
    catalog: melee::Catalog,
    items: [u8; 32],
    digest: [u8; 32],
}
impl Profile {
    fn parse(weapons: &[u8], items: &[u8]) -> Result<Self> {
        let catalog = melee::Catalog::from_json(weapons)?;
        catalog.validate_items(&inventory::Catalog::from_json(items)?)?;
        let items = crate::terrain::sha256(items);
        let mut data = b"theia-melee-config-v1\0".to_vec();
        data.extend(crate::terrain::sha256(weapons));
        data.extend(items);
        Ok(Self {
            catalog,
            items,
            digest: crate::terrain::sha256(&data),
        })
    }
    pub(super) fn catalog(&self) -> &melee::Catalog {
        &self.catalog
    }
    pub(super) fn items_digest(&self) -> [u8; 32] {
        self.items
    }
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut data = b"theia-melee-world-v1\0".to_vec();
        data.extend(base);
        data.extend(self.digest);
        crate::terrain::sha256(&data)
    }
}
/// Apply after existing ability/player-life identity and before later World roles.
pub fn profile_identity(base: [u8; 32], weapons: &[u8], items: &[u8]) -> Result<[u8; 32]> {
    Ok(Profile::parse(weapons, items)?.identity(base))
}
pub const MAX_READY_AHEAD_TICKS: u64 = 3600;
pub(super) struct SwingPlan {
    clock: Clock,
    weapon: Option<ContentId>,
}
impl Authority {
    pub(super) fn plan_ability_swing(
        &self,
        world: &World,
        id: u64,
        ability: &Ability,
    ) -> std::result::Result<Option<SwingPlan>, Reason> {
        let Some(rules) = ability.combat_rules() else {
            return Ok(None);
        };
        let Some(player) = self.actors.get(&id).map(|a| &a.autoattack) else {
            return Ok(None);
        };
        let Some(mut clock) = player.clock else {
            return Ok(None);
        };
        clock
            .resume(world.tick)
            .map_err(|_| Reason::TimeExhausted)?;
        let mut item = player.weapon.clone();
        if let Ok(weapon) = self.melee_weapon(id)
            && item.as_ref() != Some(weapon.item())
        {
            clock
                .reset(world.tick, weapon.period_ticks(world.rate))
                .map_err(|_| Reason::TimeExhausted)?;
            item = Some(weapon.item().clone());
        }
        match rules.autoattack() {
            Swing::Continue => {}
            Swing::Suspend => clock
                .suspend(world.tick)
                .map_err(|_| Reason::TimeExhausted)?,
            Swing::Delay { .. } => clock
                .delay(
                    world.tick,
                    rules
                        .autoattack()
                        .delay_ticks(world.rate)
                        .ok_or(Reason::TimeExhausted)?,
                )
                .map_err(|_| Reason::TimeExhausted)?,
            Swing::Reset => clock
                .reset(world.tick, clock.period_ticks())
                .map_err(|_| Reason::TimeExhausted)?,
        }
        if clock.ready_at().saturating_sub(world.tick) > MAX_READY_AHEAD_TICKS {
            return Err(Reason::TimeExhausted);
        }
        Ok(Some(SwingPlan {
            clock,
            weapon: item,
        }))
    }
    pub(super) fn commit_ability_swing(&mut self, id: u64, plan: Option<SwingPlan>) {
        if let Some(plan) = plan {
            let player = &mut self.actor(id).autoattack;
            player.clock = Some(plan.clock);
            player.weapon = plan.weapon;
        }
    }
    pub(super) fn observe_gear(
        &mut self,
        id: u64,
        committed: Option<([u8; 32], u64, Option<ContentId>)>,
        now: u64,
        rate: theia_core::time::TickRate,
    ) {
        let previous = self
            .actors
            .get(&id)
            .and_then(|a| a.equipment.committed.clone());
        self.actor(id).equipment.observe(committed);
        if self.catalog.schema() < 7 {
            return;
        }
        let Some(profile) = &self.autoattack else {
            return;
        };
        let actor = self.actors.get_mut(&id).unwrap();
        let Some((hash, _, item)) = &actor.equipment.committed else {
            return;
        };
        if !actor.equipment.ready
            || *hash != profile.items
            || previous
                .as_ref()
                .is_none_or(|(old_hash, _, old)| old_hash != hash || old == item)
        {
            return;
        }
        let Some(clock) = &mut actor.autoattack.clock else {
            return;
        };
        let weapon = item.as_ref().and_then(|item| profile.catalog.weapon(item));
        let period = weapon.map_or(clock.period_ticks(), |w| w.period_ticks(rate));
        if clock.reset(now, period).is_err() {
            let _ = clock.stop(now);
            actor.autoattack.phase = Phase::Paused(Pause::TimeExhausted);
        } else if let Some(weapon) = weapon {
            actor.autoattack.weapon = Some(weapon.item().clone());
        }
    }
    pub(super) fn melee_weapon(&self, id: u64) -> std::result::Result<melee::Weapon, Pause> {
        let profile = self.autoattack.as_ref().ok_or(Pause::NoWeapon)?;
        let gear = &self.actors.get(&id).ok_or(Pause::NoWeapon)?.equipment;
        if !gear.ready {
            return Err(Pause::NoWeapon);
        }
        let (hash, _, item) = gear.committed.as_ref().ok_or(Pause::NoWeapon)?;
        if *hash != profile.items {
            return Err(Pause::NoWeapon);
        }
        profile
            .catalog
            .weapon(item.as_ref().ok_or(Pause::NoWeapon)?)
            .cloned()
            .ok_or(Pause::NoWeapon)
    }
    pub(in crate::simulation) fn stop_autoattack(&mut self, id: u64, now: u64) {
        if let Some(actor) = self.actors.get_mut(&id) {
            if let Some(clock) = &mut actor.autoattack.clock {
                let _ = clock.stop(now);
            }
            actor.autoattack.phase = Phase::Idle;
        }
    }
    pub(super) fn invalidate_autoattack(&mut self, target: EntityKey, now: u64) {
        for actor in self.actors.values_mut() {
            if let Some(clock) = &mut actor.autoattack.clock
                && clock.target() == Some(target)
            {
                let _ = clock.invalidate(target, now);
                actor.autoattack.phase = Phase::Idle;
            }
        }
    }
    fn request_autoattack(&mut self, world: &mut World, id: u64, request: Request) -> Reply {
        let ignored = Reply {
            sequence: request.sequence,
            outcome: Outcome::Ignored,
        };
        if self.autoattack.is_none()
            || world.entity_key(id).is_none()
            || world
                .players
                .get(&id)
                .and_then(|a| a.commands.as_ref())
                .is_none_or(|commands| commands.state().stream_epoch != request.stream_epoch)
            || request.epoch != world.epoch()
            || request.sequence == 0
            || request.sequence <= self.actors.get(&id).map_or(0, |a| a.autoattack.sequence)
        {
            return ignored;
        }
        let result = (|| -> std::result::Result<Outcome, Pause> {
            let Some(target) = request.target else {
                self.stop_autoattack(id, world.tick);
                return Ok(Outcome::Stopped);
            };
            if !self.player_alive(id) {
                return Err(Pause::DeadPlayer);
            }
            if !self
                .creatures
                .iter()
                .any(|c| c.key == target && c.current > 0)
                || !world.select_target(id, Some(target))
            {
                return Err(Pause::InvalidTarget);
            }
            if self
                .actors
                .get(&id)
                .and_then(|a| a.autoattack.clock)
                .is_some_and(|c| c.target() != Some(target))
            {
                self.stop_autoattack(id, world.tick);
            }
            let weapon = self.melee_weapon(id)?;
            let owner = world.entity_key(id).ok_or(Pause::InvalidTarget)?;
            let player = &mut self.actor(id).autoattack;
            if player.clock.is_none() {
                player.clock = Some(
                    Clock::new(owner, weapon.period_ticks(world.rate), world.tick)
                        .map_err(|_| Pause::TimeExhausted)?,
                );
            }
            if player
                .weapon
                .as_ref()
                .is_some_and(|item| item != weapon.item())
            {
                player
                    .clock
                    .as_mut()
                    .unwrap()
                    .reset(world.tick, weapon.period_ticks(world.rate))
                    .map_err(|_| Pause::TimeExhausted)?;
            }
            player.weapon = Some(weapon.item().clone());
            player
                .clock
                .as_mut()
                .unwrap()
                .engage(target, world.tick)
                .map_err(|_| Pause::InvalidTarget)?;
            player.phase = Phase::Engaged;
            Ok(Outcome::Engaged)
        })();
        let reply = Reply {
            sequence: request.sequence,
            outcome: result.unwrap_or_else(Outcome::Rejected),
        };
        let player = &mut self.actor(id).autoattack;
        player.sequence = request.sequence;
        player.reply = Some(reply);
        reply
    }
    fn melee_eligibility(
        &self,
        world: &World,
        id: u64,
        target: EntityKey,
        weapon: &melee::Weapon,
    ) -> std::result::Result<(), Pause> {
        if !self.player_alive(id) {
            return Err(Pause::DeadPlayer);
        }
        if self.white_melee.is_some() && !self.ordinary_target_ready() {
            return Err(Pause::NotReady);
        }
        if self.white_melee.as_ref().is_some_and(|p| p.error.is_some())
            || self.physical.as_ref().is_some_and(|p| p.error.is_some())
        {
            return Err(Pause::NotReady);
        }
        if self.physical.is_some() && world.patrol.as_ref().is_none_or(|p| p.key != target) {
            return Err(Pause::InvalidTarget);
        }
        let actor = world.players.get(&id).ok_or(Pause::DeadPlayer)?;
        let target = self
            .creatures
            .iter()
            .find(|c| c.key == target && c.current > 0)
            .ok_or(Pause::InvalidTarget)?;
        if !Self::ready(world, id) {
            return Err(Pause::NotReady);
        }
        if self
            .actors
            .get(&id)
            .and_then(|a| a.cast.as_ref())
            .is_some_and(|cast| {
                self.catalog
                    .ability(cast.ability.as_str())
                    .and_then(|a| a.combat_rules())
                    .is_none_or(|rules| rules.autoattack() != Swing::Continue)
            })
        {
            return Err(Pause::Casting);
        }
        let from = actor.position.metres();
        let to = target.position.metres();
        if from
            .into_iter()
            .zip(to)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            > weapon.reach_metres().powi(2)
        {
            return Err(Pause::OutOfRange);
        }
        let facing = actor.facing.ok_or(Pause::NotFacing)?;
        if !weapon
            .arc()
            .contains(facing, actor.position, target.position)
        {
            return Err(Pause::NotFacing);
        }
        let patch = world
            .procedural
            .as_ref()
            .and_then(|r| r.patch(actor.position))
            .ok_or(Pause::NotReady)?;
        if !patch.covers(target.position, actor.ready.as_ref()) {
            return Err(Pause::NotReady);
        }
        let from = patch
            .frame
            .to_local(actor.position)
            .ok_or(Pause::NotReady)?;
        let to = patch
            .frame
            .to_local(target.position)
            .ok_or(Pause::NotReady)?;
        if !patch.scene.line_clear(
            bevy::prelude::Vec3::from_array(from.metres()),
            bevy::prelude::Vec3::from_array(to.metres()),
        ) {
            return Err(Pause::Obstructed);
        }
        Ok(())
    }
    pub(super) fn advance_autoattack(&mut self, world: &mut World) {
        if self.autoattack.is_none() {
            return;
        }
        // End suspension even after selection was cleared while a spell ran.
        for actor in self.actors.values_mut() {
            if actor.cast.is_none()
                && let Some(clock) = &mut actor.autoattack.clock
                && clock.suspended()
                && clock.resume(world.tick).is_err()
            {
                let _ = clock.stop(world.tick);
                actor.autoattack.phase = Phase::Paused(Pause::TimeExhausted);
            }
        }
        let ids: Vec<_> = self
            .actors
            .iter()
            .filter(|(_, a)| a.autoattack.clock.is_some_and(|c| c.target().is_some()))
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let target = self.actors[&id].autoattack.clock.unwrap().target().unwrap();
            let weapon = self.melee_weapon(id);
            let eligibility = weapon
                .as_ref()
                .map_err(|e| *e)
                .and_then(|w| self.melee_eligibility(world, id, target, w));
            let slots = self.physical_melee_event_slots(target);
            let eligibility = if self.events.len() + slots > self.step_event_budget() {
                Err(Pause::EventBudget)
            } else {
                eligibility
            };
            let next = self.actors[&id].autoattack.swings.checked_add(1);
            let eligibility = if next.is_none() {
                Err(Pause::TimeExhausted)
            } else {
                eligibility
            };
            let player = &mut self.actors.get_mut(&id).unwrap().autoattack;
            if let Ok(weapon) = &weapon
                && player.weapon.as_ref() != Some(weapon.item())
            {
                if player
                    .clock
                    .as_mut()
                    .unwrap()
                    .reset(world.tick, weapon.period_ticks(world.rate))
                    .is_err()
                {
                    player.phase = Phase::Paused(Pause::TimeExhausted);
                    continue;
                }
                player.weapon = Some(weapon.item().clone());
            }
            let step = player
                .clock
                .as_mut()
                .unwrap()
                .step(world.tick, eligibility.is_ok());
            match step {
                Ok(Step::Swing { .. }) => {
                    let sequence = next.unwrap();
                    player.swings = sequence;
                    player.phase = Phase::Attacking;
                    let weapon = weapon.unwrap();
                    if self.white_melee.is_some() {
                        if let Err(error) =
                            self.apply_white_melee(world, id, target, &weapon, sequence)
                        {
                            self.white_melee.as_mut().unwrap().error = Some(error.to_string());
                            let player = &mut self.actor(id).autoattack;
                            let _ = player.clock.as_mut().unwrap().stop(world.tick);
                            player.phase = Phase::Paused(Pause::TimeExhausted);
                        }
                        continue;
                    }
                    if self.physical.is_some() {
                        if let Err(error) =
                            self.apply_physical_melee(world, id, target, &weapon, sequence)
                        {
                            self.physical.as_mut().unwrap().error = Some(error.to_string());
                            let player = &mut self.actor(id).autoattack;
                            let _ = player.clock.as_mut().unwrap().stop(world.tick);
                            player.phase = Phase::Paused(Pause::TimeExhausted);
                        }
                        continue;
                    }
                    let actor = world.entity_key(id).unwrap();
                    self.apply_damage(
                        world,
                        actor,
                        target,
                        weapon.damage(),
                        |amount, remaining, position| Event::MeleeDamage {
                            weapon: weapon.item().clone(),
                            actor,
                            sequence,
                            target,
                            position,
                            amount,
                            remaining,
                        },
                    );
                }
                Ok(Step::Paused) => {
                    player.phase = Phase::Paused(eligibility.err().unwrap_or(Pause::Casting))
                }
                Ok(Step::Waiting { .. }) => player.phase = Phase::Engaged,
                Ok(Step::Idle) => player.phase = Phase::Idle,
                Err(_) => {
                    let _ = player.clock.as_mut().unwrap().stop(world.tick);
                    player.phase = Phase::Paused(Pause::TimeExhausted);
                }
            }
        }
    }
    pub(super) fn retire_invalid_autoattacks(&mut self, world: &World) {
        let ids: Vec<_> = self
            .actors
            .iter()
            .filter_map(|(id, a)| {
                a.autoattack
                    .clock
                    .and_then(|c| c.target())
                    .filter(|target| {
                        !self.player_alive(*id)
                            || !self
                                .creatures
                                .iter()
                                .any(|c| c.key == *target && c.current > 0)
                    })
                    .map(|_| *id)
            })
            .collect();
        for id in ids {
            self.stop_autoattack(id, world.tick);
        }
    }
}
impl World {
    pub fn enable_autoattack(&mut self, weapons: &[u8], items: &[u8]) -> Result<()> {
        if !self.classic
            || self.tick != 0
            || !self.players.is_empty()
            || self.combat.as_ref().is_none_or(|c| c.autoattack.is_some())
        {
            return Err("Basic melee requires inactive classic combat".into());
        }
        let profile = Profile::parse(weapons, items)?;
        self.combat.as_mut().unwrap().autoattack = Some(profile);
        Ok(())
    }
    pub fn autoattack_catalog(&self) -> Option<&melee::Catalog> {
        self.combat
            .as_ref()?
            .autoattack
            .as_ref()
            .map(|p| &p.catalog)
    }
    pub fn autoattack_enabled(&self) -> bool {
        self.combat.as_ref().is_some_and(|c| c.autoattack.is_some())
    }
    /// Session owner supplies actor identity; payloads never address another actor.
    pub fn request_autoattack(&mut self, id: u64, request: Request) -> Reply {
        let Some(mut authority) = self.combat.take() else {
            return Reply {
                sequence: request.sequence,
                outcome: Outcome::Ignored,
            };
        };
        let reply = authority.request_autoattack(self, id, request);
        self.combat = Some(authority);
        reply
    }
    pub fn autoattack_snapshot(&self, id: u64) -> Option<Snapshot> {
        let authority = self.combat.as_ref().filter(|a| a.autoattack.is_some())?;
        self.players.get(&id)?;
        let player = authority.actors.get(&id).map(|a| &a.autoattack);
        Some(Snapshot {
            actor: self.entity_key(id)?,
            stream_epoch: self
                .players
                .get(&id)?
                .commands
                .as_ref()?
                .state()
                .stream_epoch,
            tick: self.tick,
            target: player.and_then(|p| p.clock.and_then(|c| c.target())),
            phase: player.map_or(Phase::Idle, |p| p.phase),
            weapon: player.and_then(|p| p.weapon.clone()),
            ready_at: player.and_then(|p| p.clock.map(|c| c.ready_at())),
            swing_sequence: player.map_or(0, |p| p.swings),
            reply: player.and_then(|p| p.reply),
        })
    }
}

#[cfg(test)]
mod tests;
