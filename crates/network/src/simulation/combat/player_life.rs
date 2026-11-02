//! Opt-in fixed-step development life/retaliation. Private transport/presentation
//! adoption remains separate. Collision loading stays in the existing owner pump.
use super::{Authority as Combat, Reason};
use crate::{Result, player_life_wire::Profile, simulation::World, wire::Intent};
use std::sync::Arc;
use theia_core::{
    coordinates::GlobalPosition,
    identity::EntityKey,
    player_life::{self as life, LifeKey},
    resources::Pool,
    retaliation,
};
mod pursuit;
pub use pursuit::Metrics as PursuitMetrics;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Windup {
    pub creature: EntityKey,
    pub target: LifeKey,
    pub completes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub profile: [u8; 32],
    pub key: LifeKey,
    pub tick: u64,
    pub movement_stream: u64,
    pub health: u32,
    pub maximum_health: u32,
    pub respawn_at: Option<u64>,
    pub windup: Option<Windup>,
    pub combat: Option<theia_core::combat_membership::Summary>,
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub windups: u64,
    pub cancelled: u64,
    pub strikes: u64,
    pub damage: u64,
    pub deaths: u64,
    pub respawns: u64,
    pub held_respawns: u64,
}
pub(super) struct Player {
    pub(super) state: life::State,
    home: GlobalPosition,
}
pub(super) struct Authority {
    pub(super) profile: Arc<Profile>,
    npc: EntityKey,
    policy: retaliation::State,
    error: Option<String>,
    metrics: Metrics,
    pursuit: Option<pursuit::Runtime>,
}
impl Authority {
    pub(super) fn retired_player(&mut self, id: u64, rate: theia_core::time::TickRate) {
        if self
            .policy
            .target()
            .is_some_and(|key| key.actor.entity.value() == id)
        {
            self.metrics.cancelled = self
                .metrics
                .cancelled
                .saturating_add(u64::from(self.policy.windup_until().is_some()));
            self.policy = retaliation::State::new(self.profile.retaliation().clone(), rate);
        }
    }
    pub(super) fn respawned_creature(
        &mut self,
        previous: EntityKey,
        next: EntityKey,
        rate: theia_core::time::TickRate,
    ) {
        if self.npc == previous {
            self.npc = next;
            self.policy = retaliation::State::new(self.profile.retaliation().clone(), rate);
            if let Some(runtime) = &mut self.pursuit
                && let Err(error) = runtime.reset(next)
            {
                self.error = Some(error.to_string());
            }
        }
    }
}
impl Combat {
    pub(in crate::simulation::combat) fn membership_encounter(
        &self,
    ) -> Option<(EntityKey, theia_core::pursuit::Mode, Option<LifeKey>)> {
        let pve = self.pve.as_ref()?;
        let directive = pve.pursuit.as_ref()?.directive();
        let mode = if pve.error.is_some() {
            theia_core::pursuit::Mode::Blocked
        } else {
            directive.mode
        };
        Some((pve.npc, mode, directive.target))
    }
    pub(in crate::simulation::combat) fn checked_reset_encounter(
        &self,
    ) -> Result<(EntityKey, theia_core::pursuit::Mode, Option<LifeKey>)> {
        let pve = self
            .pve
            .as_ref()
            .ok_or("Reset lost player life authority")?;
        if pve.error.is_some() {
            return Err("Reset pursuit authority failed".into());
        }
        let directive = pve
            .pursuit
            .as_ref()
            .ok_or("Reset lost pursuit")?
            .directive();
        Ok((pve.npc, directive.mode, directive.target))
    }
    pub(in crate::simulation) fn player_alive(&self, id: u64) -> bool {
        self.pve.is_none()
            || self
                .actors
                .get(&id)
                .and_then(|a| a.life.as_ref())
                .is_some_and(|p| p.state.alive())
    }
    pub(super) fn register_life(&mut self, world: &World, id: u64) -> Result<()> {
        let Some(pve) = &self.pve else { return Ok(()) };
        let position = world
            .global_position(id)
            .ok_or("Missing player life actor")?;
        let [x, _, z] = position.metres();
        let patch = world
            .procedural
            .as_ref()
            .and_then(|r| r.patch(position))
            .ok_or("Player life collision is not prepared")?;
        let mut home = patch
            .try_spawn(x, z)
            .ok_or("Player life has no supported home")?
            .metres();
        home[1] += f64::from(crate::world::PAWN_Y);
        let home = GlobalPosition::new(home).ok_or("Invalid player life home")?;
        world.validate_restored_player(home)?;
        let state = life::State::new(
            pve.profile.life().clone(),
            world.rate,
            world.entity_key(id).ok_or("Missing player life identity")?,
        );
        self.actor(id).life = Some(Player { state, home });
        Ok(())
    }
    pub(in crate::simulation) fn respawn_demands(&self) -> Vec<GlobalPosition> {
        self.actors
            .values()
            .filter_map(|a| a.life.as_ref())
            .filter(|p| !p.state.alive())
            .map(|p| p.home)
            .collect()
    }
    pub(super) fn ordinary_target_ready(&self) -> bool {
        self.reset.as_ref().is_none_or(|runtime| runtime.ready())
            && self.pve.as_ref().is_some_and(|p| {
                p.error.is_none()
                    && p.pursuit.as_ref().is_some_and(|r| {
                        matches!(
                            r.directive().mode,
                            theia_core::pursuit::Mode::Patrol
                                | theia_core::pursuit::Mode::Chase
                                | theia_core::pursuit::Mode::Attack
                        )
                    })
            })
    }
    pub(super) fn supports_white_melee(&self) -> bool {
        self.pve
            .as_ref()
            .is_some_and(|p| p.pursuit.is_some() && p.error.is_none())
    }
    pub(super) fn prepare_retaliation_attempt(
        &self,
        world: &World,
        attacker: EntityKey,
        victim: EntityKey,
        remaining: u32,
    ) -> Result<Option<retaliation::State>> {
        if remaining == 0 || world.entity_key(attacker.entity.value()) != Some(attacker) {
            return Ok(None);
        }
        let Some(key) = self
            .actors
            .get(&attacker.entity.value())
            .and_then(|a| a.life.as_ref())
            .filter(|p| p.state.alive())
            .map(|p| p.state.key())
        else {
            return Ok(None);
        };
        let Some(pve) = &self.pve else {
            return Ok(None);
        };
        if pve.npc != victim
            || pve.pursuit.as_ref().is_some_and(|p| {
                matches!(
                    p.directive().mode,
                    theia_core::pursuit::Mode::Return
                        | theia_core::pursuit::Mode::Blocked
                        | theia_core::pursuit::Mode::Dead
                )
            })
        {
            return Ok(None);
        }
        let mut policy = pve.policy.clone();
        policy
            .record_hit(world.tick, key)
            .map_err(|_| "Retaliation clock refused an attack")?;
        Ok(Some(policy))
    }
    pub(super) fn commit_retaliation_attempt(&mut self, policy: Option<retaliation::State>) {
        if let Some(policy) = policy {
            self.pve
                .as_mut()
                .expect("Prepared retaliation authority")
                .policy = policy;
        }
    }
    pub(super) fn note_retaliation_hit(
        &mut self,
        world: &World,
        attacker: EntityKey,
        victim: EntityKey,
        applied: u32,
        remaining: u32,
    ) {
        if applied == 0 {
            return;
        }
        match self.prepare_retaliation_attempt(world, attacker, victim, remaining) {
            Ok(policy) => self.commit_retaliation_attempt(policy),
            Err(error) => {
                if let Some(pve) = &mut self.pve {
                    pve.error = Some(error.to_string());
                }
            }
        }
    }
    fn reachable(&self, world: &World, npc: EntityKey, key: LifeKey, reach: f64) -> bool {
        let Some(player) = self
            .actors
            .get(&key.actor.entity.value())
            .and_then(|a| a.life.as_ref())
        else {
            return false;
        };
        if player.state.key() != key
            || !player.state.alive()
            || world.entity_key(key.actor.entity.value()) != Some(key.actor)
        {
            return false;
        }
        let Some(position) = world.global_position(key.actor.entity.value()) else {
            return false;
        };
        let Some(creature) = self
            .creatures
            .iter()
            .find(|c| c.key == npc && c.current > 0)
        else {
            return false;
        };
        let distance: f64 = position
            .metres()
            .into_iter()
            .zip(creature.position.metres())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        if !distance.is_finite() || distance > reach.powi(2) {
            return false;
        }
        let Some(patch) = world.procedural.as_ref().and_then(|r| r.patch(position)) else {
            return false;
        };
        if !patch.covers(creature.position, None) {
            return false;
        }
        let (Some(from), Some(to)) = (
            patch.frame.to_local(position),
            patch.frame.to_local(creature.position),
        ) else {
            return false;
        };
        patch.scene.line_clear(
            bevy::prelude::Vec3::from_array(from.metres()),
            bevy::prelude::Vec3::from_array(to.metres()),
        )
    }
    pub(in crate::simulation) fn holds_patrol(&self, world: &World) -> bool {
        self.pve.as_ref().is_some_and(|p| {
            p.policy.target().is_some_and(|key| {
                self.reachable(world, p.npc, key, p.profile.retaliation().reach_metres())
            })
        })
    }
    pub(super) fn damage_player(
        &mut self,
        world: &mut World,
        target: LifeKey,
        amount: u32,
    ) -> Result<Option<life::Damage>> {
        let id = target.actor.entity.value();
        let Some(player) = self.actors.get(&id).and_then(|a| a.life.as_ref()) else {
            return Ok(None);
        };
        if player.state.key() != target
            || !player.state.alive()
            || world.entity_key(id) != Some(target.actor)
        {
            return Ok(None);
        }
        let mut next = player.state.clone();
        let damage = next
            .damage(world.tick, target, amount)
            .map_err(|_| "Player life refused damage")?;
        if damage.died {
            world.reset_life_controls(id, None)?;
        }
        let actor = self.actors.get_mut(&id).unwrap();
        actor.life.as_mut().unwrap().state = next;
        if damage.died {
            let cast = actor.cast.take();
            let queued = actor.queued.take();
            actor.interrupted = false;
            if let Some(cast) = cast {
                self.cancel(target.actor, cast.sequence, Reason::Interrupted)
            }
            if let Some(queued) = queued {
                self.cancel(target.actor, queued.request.sequence, Reason::Interrupted)
            }
        }
        Ok(Some(damage))
    }
    pub(super) fn advance_retaliation(&mut self, world: &mut World) {
        let Some(mut pve) = self.pve.take() else {
            return;
        };
        let alive = self
            .creatures
            .iter()
            .any(|c| c.key == pve.npc && c.current > 0);
        let physical_ready = self
            .physical
            .as_ref()
            .is_none_or(|p| p.error.is_none() && self.events.len() + 3 <= self.step_event_budget());
        let physical_ready = physical_ready
            && self.white_melee.as_ref().is_none_or(|p| {
                p.error.is_none() && self.events.len() + 3 <= self.step_event_budget()
            });
        let target = pve.policy.target().and_then(|key| {
            self.actors
                .get(&key.actor.entity.value())
                .and_then(|a| a.life.as_ref())
                .map(|p| retaliation::Target {
                    key: p.state.key(),
                    alive: p.state.alive(),
                    reachable: physical_ready
                        && pve.error.is_none()
                        && pve
                            .pursuit
                            .as_ref()
                            .is_none_or(|r| r.directive().can_attack)
                        && self.reachable(
                            world,
                            pve.npc,
                            key,
                            pve.profile.retaliation().reach_metres(),
                        ),
                })
        });
        match pve.policy.advance(world.tick, alive, target) {
            Ok(retaliation::Event::Windup { .. }) => {
                pve.metrics.windups = pve.metrics.windups.saturating_add(1)
            }
            Ok(retaliation::Event::Cancelled { .. }) => {
                pve.metrics.cancelled = pve.metrics.cancelled.saturating_add(1)
            }
            Ok(retaliation::Event::Strike { target, amount }) => {
                match if self.white_melee.is_some() {
                    self.apply_white_retaliation(world, pve.npc, target)
                } else {
                    self.apply_physical_retaliation(world, pve.npc, target, amount)
                } {
                    Ok(Some(hit)) => {
                        pve.metrics.strikes = pve.metrics.strikes.saturating_add(1);
                        pve.metrics.damage =
                            pve.metrics.damage.saturating_add(u64::from(hit.applied));
                        pve.metrics.deaths = pve.metrics.deaths.saturating_add(u64::from(hit.died));
                        if hit.died {
                            // The old player's engagement ends in this same step;
                            // another player can arm a new, full windup afterwards.
                            pve.policy = retaliation::State::new(
                                pve.profile.retaliation().clone(),
                                world.rate,
                            );
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        if let Some(runtime) = &mut self.white_melee {
                            runtime.error = Some(error.to_string());
                        }
                        pve.error = Some(error.to_string());
                    }
                }
            }
            Ok(retaliation::Event::None) => {}
            Err(error) => pve.error = Some(format!("Retaliation clock failed: {error:?}")),
        }
        self.pve = Some(pve);
    }
    pub(in crate::simulation) fn advance_player_lives(&mut self, world: &mut World, now: u64) {
        if self.pve.is_none() {
            return;
        }
        let ids: Vec<_> = self
            .actors
            .iter()
            .filter_map(|(&id, a)| {
                a.life
                    .as_ref()
                    .filter(|p| p.state.respawn_at().is_some_and(|at| now >= at))
                    .map(|_| id)
            })
            .collect();
        for id in ids {
            let player = self.actors[&id].life.as_ref().unwrap();
            let home = player.home;
            if world.validate_restored_player(home).is_err() {
                let p = self.pve.as_mut().unwrap();
                p.metrics.held_respawns = p.metrics.held_respawns.saturating_add(1);
                continue;
            }
            let mut next = player.state.clone();
            let result = next
                .advance(now)
                .map_err(|_| "Player respawn clock failed".into())
                .and_then(|due| {
                    if due {
                        world.reset_life_controls(id, Some(home))
                    } else {
                        Err("Respawn deadline changed".into())
                    }
                });
            if let Err(error) = result {
                self.pve.as_mut().unwrap().error = Some(error.to_string());
                continue;
            }
            let actor = self.actors.get_mut(&id).unwrap();
            actor.life.as_mut().unwrap().state = next;
            actor.mana = self
                .resources
                .as_ref()
                .map(|p| Pool::new(p.maximum_mana(), p.maximum_mana()).unwrap());
            actor.cast = None;
            actor.queued = None;
            actor.interrupted = false;
            // Already applied effects may finish while dead, but never acquire
            // the new respawn generation. Existing logout has the same cleanup.
            if let Some(statuses) = &mut self.statuses {
                statuses.remove_source(id);
            }
            let p = self.pve.as_mut().unwrap();
            p.metrics.respawns = p.metrics.respawns.saturating_add(1);
        }
    }
}
impl World {
    pub fn enable_player_life(&mut self, profile: Arc<Profile>) -> Result<()> {
        let combat = self
            .combat
            .as_mut()
            .ok_or("Player life requires resource combat")?;
        let patrol = self
            .patrol
            .as_mut()
            .ok_or("Retaliation requires a prepared patrol")?;
        if self.tick != 0
            || !self.players.is_empty()
            || !self.predicted
            || combat.pve.is_some()
            || combat.creatures.len() != 1
            || combat.resources_hash != Some(profile.stats_hash())
            || patrol.profile_hash() != profile.creature_hash()
            || patrol.definition() != profile.creature()
            || !combat.creatures.iter().any(|c| c.key == patrol.key)
        {
            return Err(
                "Player life requires matching inactive prediction/stats/patrol profiles".into(),
            );
        }
        let pursuit = if let Some(settings) = patrol.aggro() {
            let worker = patrol
                .worker
                .take()
                .ok_or("Pursuit requires prepared navigation worker")?;
            Some(pursuit::Runtime::new(
                settings, patrol.key, self.rate, worker,
            )?)
        } else {
            None
        };
        combat.pve = Some(Authority {
            npc: patrol.key,
            policy: retaliation::State::new(profile.retaliation().clone(), self.rate),
            profile,
            error: None,
            metrics: Metrics::default(),
            pursuit,
        });
        Ok(())
    }
    pub fn player_life_profile(&self) -> Option<&Arc<Profile>> {
        self.combat.as_ref()?.pve.as_ref().map(|p| &p.profile)
    }
    pub fn player_alive(&self, id: u64) -> bool {
        self.players.contains_key(&id) && self.combat.as_ref().is_none_or(|c| c.player_alive(id))
    }
    pub fn player_life_error(&self) -> Option<&str> {
        self.combat.as_ref()?.pve.as_ref()?.error.as_deref()
    }
    pub fn player_life_metrics(&self) -> Option<Metrics> {
        Some(self.combat.as_ref()?.pve.as_ref()?.metrics)
    }
    pub fn player_life_snapshot(&self, id: u64) -> Option<Observation> {
        let combat = self.combat.as_ref()?;
        let pve = combat.pve.as_ref()?;
        let player = combat.actors.get(&id)?.life.as_ref()?;
        let key = player.state.key();
        Some(Observation {
            combat: if self.combat_membership_profile().is_some() {
                Some(if player.state.alive() {
                    self.combat_membership(id)?
                } else {
                    Default::default()
                })
            } else {
                None
            },
            profile: pve.profile.hash(),
            key,
            tick: self.tick,
            movement_stream: self
                .players
                .get(&id)?
                .commands
                .as_ref()?
                .state()
                .stream_epoch,
            health: player.state.health(),
            maximum_health: player.state.maximum_health(),
            respawn_at: player.state.respawn_at(),
            windup: if pve.policy.target() == Some(key) {
                pve.policy.windup_until().map(|completes| Windup {
                    creature: pve.npc,
                    target: key,
                    completes,
                })
            } else {
                None
            },
        })
    }
    pub(in crate::simulation) fn initialize_player_life(&mut self, id: u64) -> bool {
        let Some(mut combat) = self.combat.take() else {
            return true;
        };
        let result = combat.register_life(self, id).is_ok();
        self.combat = Some(combat);
        result
    }
    pub(in crate::simulation) fn advance_player_lives(&mut self, now: u64) {
        if let Some(mut combat) = self.combat.take() {
            combat.advance_player_lives(self, now);
            self.combat = Some(combat);
        }
    }
    fn reset_life_controls(&mut self, id: u64, home: Option<GlobalPosition>) -> Result<()> {
        let actor = self
            .players
            .get_mut(&id)
            .ok_or("Missing life movement actor")?;
        let stream = actor
            .commands
            .as_ref()
            .ok_or("Player life requires predicted movement")?
            .state()
            .stream_epoch
            .checked_add(1)
            .ok_or("Life movement stream exhausted")?;
        let timeline = crate::commands::Timeline::new(stream, self.rate, self.tick)?;
        actor.commands = Some(timeline);
        actor.intent = Intent::default();
        actor.applied_sequence = 0;
        actor.input_tick = self.tick;
        actor.body.reset(true);
        if let Some(dodge) = &mut actor.dodge {
            dodge.cancel();
        }
        if let Some(home) = home {
            actor.position = home;
        }
        // Keep the last validated chunk set: same-patch respawn needs no new
        // report, and a far home is held by the existing footprint readiness.
        Ok(())
    }
}

#[cfg(test)]
mod tests;
