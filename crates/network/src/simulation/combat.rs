//! Optional bounded server combat. No transport, renderer, file I/O or wall clock.
pub mod ability_damage;
pub mod autoattack;
pub mod credit;
pub mod membership;
mod outcomes;
pub mod physical;
pub mod player_life;
mod projectiles;
pub mod reset;
mod statuses;
pub mod threat;
pub mod white_melee;
use super::World;
pub use outcomes::Metrics as OutcomeMetrics;
pub use projectiles::Metrics as ProjectileMetrics;
pub use statuses::{MAX_ACTIVE_STATUSES, MAX_TARGET_STATUSES, Status};
use std::{collections::BTreeMap, time::Duration};
use theia_core::{
    abilities::{Ability, Catalog, Effect},
    content::ContentId,
    coordinates::GlobalPosition,
    identity::EntityKey,
    resources::{Pool, Profile, Vitals},
};

pub const MAX_STEP_EVENTS: usize = 64;
pub const MAX_PROJECTILE_STEP_EVENTS: usize = 128;
pub const DEVELOPMENT_HEALTH: u32 = 100;
pub const DEVELOPMENT_RESPAWN_MS: u32 = 5000;
struct Configuration {
    resources_hash: Option<[u8; 32]>,
    catalog: Catalog,
    digest: [u8; 32],
    resources: Option<Profile>,
    statuses: Option<theia_core::statuses::Catalog>,
}
fn configuration(
    bytes: &[u8],
    health: u32,
    respawn_ms: u32,
    resources: Option<&[u8]>,
    statuses: Option<&[u8]>,
) -> crate::Result<Configuration> {
    if health == 0 || health > 1_000_000 || !(100..=60_000).contains(&respawn_ms) {
        return Err("Combat health/respawn configuration exceeds bounded prototype".into());
    }
    let catalog = Catalog::from_json(bytes)?;
    let profile = resources.map(Profile::from_json).transpose()?;
    let status_catalog = statuses
        .map(theia_core::statuses::Catalog::from_json)
        .transpose()?;
    if catalog.schema() != 1 && profile.is_none() {
        return Err("Resource-cost catalogs require an explicit stats profile".into());
    }
    if (catalog.schema() >= 3 && status_catalog.is_none())
        || (status_catalog.is_some() && profile.is_none())
    {
        return Err("Status abilities require explicit status and resource profiles".into());
    }
    for ability in catalog.abilities() {
        if let Effect::ApplyStatus { status } = ability.effect()
            && status_catalog
                .as_ref()
                .and_then(|c| c.definition(&status))
                .is_none()
        {
            return Err("Ability references an unknown status definition".into());
        }
    }
    if let Some(profile) = &profile
        && catalog
            .abilities()
            .iter()
            .any(|a| a.mana_cost() > profile.maximum_mana())
    {
        return Err("Ability cost exceeds the configured maximum mana".into());
    }
    let mut identity = b"theia-combat-profile-v2\0".to_vec();
    identity.extend(crate::terrain::sha256(bytes));
    identity.extend(health.to_le_bytes());
    identity.extend(respawn_ms.to_le_bytes());
    if let Some(bytes) = resources {
        identity.extend(b"theia-resource-profile-v1\0");
        identity.extend(crate::terrain::sha256(bytes));
    }
    if let Some(bytes) = statuses {
        identity.extend(b"theia-status-profile-v1\0");
        identity.extend(crate::terrain::sha256(bytes));
    }
    Ok(Configuration {
        resources_hash: resources.map(crate::terrain::sha256),
        catalog,
        digest: crate::terrain::sha256(&identity),
        resources: profile,
        statuses: status_catalog,
    })
}
fn world_digest(base: [u8; 32], config: [u8; 32]) -> [u8; 32] {
    let mut identity = b"theia-combat-world-v2\0".to_vec();
    identity.extend(base);
    identity.extend(config);
    crate::terrain::sha256(&identity)
}
/// Bind the validated catalog/configuration on both ends before admission.
/// `base` is the targeting-profile world identity, not the raw terrain digest.
pub fn profile_identity(
    base: [u8; 32],
    bytes: &[u8],
    health: u32,
    respawn_ms: u32,
) -> crate::Result<[u8; 32]> {
    let config = configuration(bytes, health, respawn_ms, None, None)?;
    Ok(world_digest(base, config.digest))
}
pub fn resource_profile_identity(
    base: [u8; 32],
    bytes: &[u8],
    health: u32,
    respawn_ms: u32,
    stats: &[u8],
) -> crate::Result<[u8; 32]> {
    let config = configuration(bytes, health, respawn_ms, Some(stats), None)?;
    Ok(world_digest(base, config.digest))
}
pub fn status_profile_identity(
    base: [u8; 32],
    bytes: &[u8],
    health: u32,
    respawn_ms: u32,
    stats: &[u8],
    statuses: &[u8],
) -> crate::Result<[u8; 32]> {
    Ok(world_digest(
        base,
        configuration(bytes, health, respawn_ms, Some(stats), Some(statuses))?.digest,
    ))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub epoch: u64,
    pub sequence: u64,
    pub ability: ContentId,
    pub target: EntityKey,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    UnknownAbility,
    UnknownTarget,
    Dead,
    OutOfRange,
    NotReady,
    Obstructed,
    Moving,
    Interrupted,
    Unavailable,
    Superseded,
    TimeExhausted,
    InsufficientMana,
    StatusCapacity,
    EquipmentRequired,
    NotFacing,
    TooClose,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ignored,
    Rejected(Reason),
    Started { completes: u64 },
    Queued { at: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub sequence: u64,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cast {
    pub sequence: u64,
    pub ability: ContentId,
    pub target: EntityKey,
    pub started: u64,
    pub completes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queued {
    pub request: Request,
    pub at: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    ProjectileLaunched {
        ability: ContentId,
        actor: EntityKey,
        sequence: u64,
        target: EntityKey,
        position: GlobalPosition,
        target_position: GlobalPosition,
    },
    ProjectileEnded {
        ability: ContentId,
        actor: EntityKey,
        sequence: u64,
        target: EntityKey,
        position: GlobalPosition,
        reason: ProjectileEnd,
    },
    Avoided {
        ability: ContentId,
        actor: EntityKey,
        sequence: u64,
        target: EntityKey,
        position: GlobalPosition,
        kind: Avoidance,
    },
    Started {
        actor: EntityKey,
        cast: Cast,
        position: GlobalPosition,
    },
    Queued {
        actor: EntityKey,
        queued: Queued,
    },
    Cancelled {
        actor: EntityKey,
        sequence: u64,
        reason: Reason,
    },
    /// Explicit physical-profile facts; legacy event codecs reject this variant.
    PhysicalDamage(physical::Fact),
    WhiteMelee(white_melee::Fact),
    AbilityDamage(ability_damage::Fact),
    /// One authoritative basic-weapon hit, independent of ability cast events.
    MeleeDamage {
        weapon: ContentId,
        actor: EntityKey,
        sequence: u64,
        target: EntityKey,
        position: GlobalPosition,
        amount: u32,
        remaining: u32,
    },
    Damage {
        ability: ContentId,
        actor: EntityKey,
        sequence: u64,
        target: EntityKey,
        position: GlobalPosition,
        amount: u32,
        remaining: u32,
    },
    Died {
        target: EntityKey,
        position: GlobalPosition,
        respawn_at: u64,
    },
    Respawned {
        previous: EntityKey,
        target: EntityKey,
        position: GlobalPosition,
    },
}
impl Event {
    pub(crate) fn private_player_victim(&self) -> Option<EntityKey> {
        match self {
            Self::PhysicalDamage(fact) if fact.target_generation.is_some() => Some(fact.target),
            Self::WhiteMelee(fact) if fact.target_generation.is_some() => Some(fact.target),
            _ => None,
        }
    }
    /// Existing development last-hit policy. Incoming player damage is never
    /// a creature kill opportunity; durable idempotency remains with the owner.
    pub(super) fn fatal_player_hit(&self) -> Option<(EntityKey, EntityKey)> {
        match self {
            Self::AbilityDamage(fact)
                if fact.remaining == 0 && fact.result.damage.health_lost > 0 =>
            {
                Some((fact.actor, fact.target))
            }
            Self::Damage {
                actor,
                target,
                remaining: 0,
                ..
            }
            | Self::MeleeDamage {
                actor,
                target,
                remaining: 0,
                ..
            } => Some((*actor, *target)),
            Self::WhiteMelee(fact)
                if fact.remaining == 0
                    && fact.result.health_lost > 0
                    && fact.actor_generation.is_some()
                    && fact.target_generation.is_none() =>
            {
                Some((fact.actor, fact.target))
            }
            Self::PhysicalDamage(fact)
                if fact.remaining == 0
                    && fact.result.health_lost > 0
                    && fact.actor_generation.is_some()
                    && fact.target_generation.is_none() =>
            {
                Some((fact.actor, fact.target))
            }
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileEnd {
    Expired,
    Obstructed,
    Unavailable,
    SourceGone,
    TargetGone,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avoidance {
    Miss,
    Deflect,
}
#[derive(Clone, Debug)]
pub struct CastResult {
    pub reply: Reply,
    /// Immediate command outcomes, at most cancellation + replacement/start.
    pub events: Vec<Event>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Health {
    pub key: EntityKey,
    pub position: GlobalPosition,
    pub current: u32,
    pub maximum: u32,
    pub respawn_at: Option<u64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub vitals: Option<Vitals>,
    pub tick: u64,
    pub reply: Option<Reply>,
    pub cast: Option<Cast>,
    pub queued: Option<Queued>,
    pub gcd_until: u64,
    pub cooldowns: Vec<(ContentId, u64)>,
    pub targets: Vec<Health>,
}
#[derive(Default)]
struct Actor {
    autoattack: autoattack::Player,
    equipment: Equipment,
    life: Option<player_life::Player>,
    mana: Option<Pool>,
    sequence: u64,
    reply: Option<Reply>,
    cast: Option<Cast>,
    cast_credit_encounter: Option<theia_core::creature_credit::EncounterKey>,
    queued: Option<Queued>,
    gcd_until: u64,
    cooldowns: BTreeMap<ContentId, u64>,
    interrupted: bool,
}
/// A trusted committed inventory observation. This has no client wire setter.
#[derive(Default)]
struct Equipment {
    committed: Option<([u8; 32], u64, Option<ContentId>)>,
    ready: bool,
}
impl Equipment {
    fn observe(&mut self, value: Option<([u8; 32], u64, Option<ContentId>)>) {
        self.ready = false;
        let Some((hash, revision, hand)) = value else {
            return;
        };
        if hash == [0; 32] {
            return;
        }
        if let Some((previous_hash, previous_revision, previous_hand)) = &self.committed
            && (hash != *previous_hash
                || revision < *previous_revision
                || (revision == *previous_revision && &hand != previous_hand))
        {
            return;
        }
        self.committed = Some((hash, revision, hand));
        self.ready = true;
    }
    fn permits(&self, ability: &Ability) -> bool {
        ability.requires_main_hand().is_none_or(|required| {
            self.ready
                && self
                    .committed
                    .as_ref()
                    .and_then(|(_, _, hand)| hand.as_ref())
                    == Some(required)
        })
    }
}
struct Hit {
    credit_encounter: Option<theia_core::creature_credit::EncounterKey>,
    ability: ContentId,
    actor: EntityKey,
    sequence: u64,
    target: EntityKey,
    amount: u32,
}
pub(super) struct Authority {
    autoattack: Option<autoattack::Profile>,
    physical: Option<physical::Runtime>,
    white_melee: Option<white_melee::Runtime>,
    ability_damage: Option<ability_damage::Runtime>,
    membership: Option<membership::Runtime>,
    credit: Option<credit::Runtime>,
    reset: Option<reset::Runtime>,
    threat: Option<threat::Runtime>,
    public_attack: Option<std::sync::Arc<crate::creature_attack_wire::Profile>>,
    public_life: Option<std::sync::Arc<crate::avatar_life_wire::Profile>>,
    catalog_digest: [u8; 32],
    projectiles: Vec<projectiles::Active>,
    projectile_metrics: ProjectileMetrics,
    random: Option<outcomes::Random>,
    outcome_metrics: OutcomeMetrics,
    pve: Option<player_life::Authority>,
    resources_hash: Option<[u8; 32]>,
    statuses: Option<statuses::Registry>,
    resources: Option<Profile>,
    catalog: Catalog,
    digest: [u8; 32],
    respawn_ticks: u64,
    creatures: Vec<Health>,
    actors: BTreeMap<u64, Actor>,
    events: Vec<Event>,
}
impl Authority {
    fn resolve_damage(
        &mut self,
        world: &mut World,
        hit: Hit,
        table: Option<theia_core::hit_outcomes::Table>,
    ) {
        if self.ability_damage.is_some() {
            self.resolve_ability_damage(world, hit);
            return;
        }
        use theia_core::hit_outcomes::{Eligibility, Outcome};
        let result = if let Some(table) = table {
            let Some(random) = &mut self.random else {
                self.cancel(hit.actor, hit.sequence, Reason::Unavailable);
                return;
            };
            let deflect = world.creature_can_deflect(hit.target);
            let result = random.resolve(table, Eligibility { deflect });
            self.outcome_metrics.random_draws = random.draws();
            result
        } else {
            Outcome::Hit
        };
        self.outcome_metrics.resolved = self.outcome_metrics.resolved.saturating_add(1);
        match result {
            Outcome::Hit => {
                self.outcome_metrics.hits = self.outcome_metrics.hits.saturating_add(1);
                self.damage(world, hit);
            }
            Outcome::Miss | Outcome::Deflect => {
                let kind = if result == Outcome::Miss {
                    self.outcome_metrics.misses = self.outcome_metrics.misses.saturating_add(1);
                    Avoidance::Miss
                } else {
                    self.outcome_metrics.deflects = self.outcome_metrics.deflects.saturating_add(1);
                    Avoidance::Deflect
                };
                if let Some(creature) = self
                    .creatures
                    .iter()
                    .find(|c| c.key == hit.target && c.current > 0)
                {
                    self.events.push(Event::Avoided {
                        ability: hit.ability,
                        actor: hit.actor,
                        sequence: hit.sequence,
                        target: hit.target,
                        position: creature.position,
                        kind,
                    });
                }
            }
        }
    }
    fn damage(&mut self, world: &mut World, hit: Hit) {
        if self.reset.is_some() && !self.ordinary_target_ready() {
            return;
        }
        self.apply_damage(
            world,
            hit.actor,
            hit.target,
            hit.amount,
            |amount, remaining, position| Event::Damage {
                ability: hit.ability,
                actor: hit.actor,
                sequence: hit.sequence,
                target: hit.target,
                position,
                amount,
                remaining,
            },
        );
    }
    fn apply_damage(
        &mut self,
        world: &mut World,
        actor: EntityKey,
        target: EntityKey,
        amount: u32,
        event: impl FnOnce(u32, u32, GlobalPosition) -> Event,
    ) {
        let Some(creature) = self
            .creatures
            .iter_mut()
            .find(|c| c.key == target && c.current > 0)
        else {
            return;
        };
        let applied = amount.min(creature.current);
        creature.current -= applied;
        self.events
            .push(event(applied, creature.current, creature.position));
        let remaining = creature.current;
        if creature.current == 0 {
            let at = world.tick.saturating_add(self.respawn_ticks);
            creature.respawn_at = Some(at);
            world.retire_target(target);
            self.events.push(Event::Died {
                target,
                position: creature.position,
                respawn_at: at,
            });
            if let Some(statuses) = &mut self.statuses {
                statuses.remove_target(target);
            }
        }
        self.note_retaliation_hit(world, actor, target, applied, remaining);
    }
    fn create(
        world: &World,
        bytes: &[u8],
        health: u32,
        respawn_ms: u32,
        stats: Option<&[u8]>,
        status_data: Option<&[u8]>,
    ) -> crate::Result<Self> {
        let Configuration {
            resources_hash,
            catalog,
            digest,
            resources,
            statuses,
        } = configuration(bytes, health, respawn_ms, stats, status_data)?;
        let random = catalog
            .abilities()
            .iter()
            .any(|a| a.hit_table().is_some())
            .then(outcomes::Random::new)
            .transpose()?;
        let event_capacity = if catalog.schema() >= 6 {
            MAX_PROJECTILE_STEP_EVENTS
        } else {
            MAX_STEP_EVENTS
        };
        Ok(Self {
            autoattack: None,
            physical: None,
            white_melee: None,
            ability_damage: None,
            membership: None,
            credit: None,
            reset: None,
            threat: None,
            public_attack: None,
            public_life: None,
            catalog_digest: crate::terrain::sha256(bytes),
            projectiles: Vec::with_capacity(projectiles::MAX_ACTIVE),
            projectile_metrics: ProjectileMetrics::default(),
            random,
            outcome_metrics: OutcomeMetrics::default(),
            pve: None,
            resources_hash,
            statuses: statuses.map(statuses::Registry::new),
            resources,
            catalog,
            digest,
            respawn_ticks: world
                .rate
                .ticks_for(Duration::from_millis(u64::from(respawn_ms)))
                .unwrap(),
            creatures: world
                .targets
                .values()
                .map(|t| Health {
                    key: t.key,
                    position: t.position,
                    current: health,
                    maximum: health,
                    respawn_at: None,
                })
                .collect(),
            actors: BTreeMap::new(),
            events: Vec::with_capacity(event_capacity),
        })
    }
    fn actor(&mut self, id: u64) -> &mut Actor {
        let mana = self
            .resources
            .as_ref()
            .map(|p| Pool::new(p.maximum_mana(), p.maximum_mana()).unwrap());
        self.actors.entry(id).or_insert_with(|| Actor {
            mana,
            ..Default::default()
        })
    }
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let base = world_digest(base, self.digest);
        let base = self.pve.as_ref().map_or(base, |p| {
            crate::player_life_wire::profile_identity(base, &p.profile)
        });
        let base = self.autoattack.as_ref().map_or(base, |p| p.identity(base));
        let base = self
            .physical
            .as_ref()
            .map_or(base, |p| p.profile.identity(base));
        let base = self
            .white_melee
            .as_ref()
            .map_or(base, |p| p.profile.identity(base));
        let base = self
            .ability_damage
            .as_ref()
            .map_or(base, |p| p.profile.identity(base));
        let base = self.membership.as_ref().map_or(base, |p| p.identity(base));
        let base = self.credit.as_ref().map_or(base, |p| p.identity(base));
        let base = self.reset.as_ref().map_or(base, |p| p.identity(base));
        let base = self.threat.as_ref().map_or(base, |p| p.identity(base));
        let base = self
            .public_attack
            .as_ref()
            .map_or(base, |p| p.identity(base));
        self.public_life.as_ref().map_or(base, |p| p.identity(base))
    }
    pub(super) fn leave(&mut self, id: u64, rate: theia_core::time::TickRate) {
        if let Some(credit) = &mut self.credit {
            credit.leave(id);
        }
        if let Some(membership) = &mut self.membership {
            membership.leave(id);
        }
        if self.threat.is_none()
            && let Some(pve) = &mut self.pve
        {
            pve.retired_player(id, rate);
        }
        self.actors.remove(&id);
        if let Some(statuses) = &mut self.statuses {
            statuses.remove_source(id);
        }
    }
    pub(super) fn interrupt(&mut self, id: u64) {
        if let Some(actor) = self.actors.get_mut(&id) {
            actor.interrupted = true;
        }
    }
    pub(super) fn move_target(&mut self, key: EntityKey, position: GlobalPosition) -> bool {
        let Some(target) = self
            .creatures
            .iter_mut()
            .find(|c| c.key == key && c.current > 0)
        else {
            return false;
        };
        target.position = position;
        true
    }
    pub(super) fn forget(&mut self, key: EntityKey, now: u64) -> bool {
        self.invalidate_autoattack(key, now);
        if let Some(statuses) = &mut self.statuses {
            statuses.remove_target(key);
        }
        let count = self.creatures.len();
        self.creatures.retain(|c| c.key != key);
        self.creatures.len() != count
    }
    fn ready(world: &World, id: u64) -> bool {
        let Some(actor) = world.players.get(&id) else {
            return false;
        };
        world
            .procedural
            .as_ref()
            .and_then(|r| r.patch(actor.position))
            .is_some_and(|p| p.covers(actor.position, actor.ready.as_ref()))
    }
    fn validate(
        &self,
        world: &World,
        id: u64,
        target: EntityKey,
        ability: &Ability,
        moved: bool,
    ) -> std::result::Result<(), Reason> {
        let actor = world.players.get(&id).ok_or(Reason::Interrupted)?;
        if !self.player_alive(id) {
            return Err(Reason::Interrupted);
        }
        if ability.requires_main_hand().is_some()
            && !self
                .actors
                .get(&id)
                .is_some_and(|a| a.equipment.permits(ability))
        {
            return Err(Reason::EquipmentRequired);
        }
        let health = self
            .creatures
            .iter()
            .find(|c| c.key == target)
            .ok_or(Reason::UnknownTarget)?;
        if health.current == 0 {
            return Err(Reason::Dead);
        }
        if !Self::ready(world, id) {
            return Err(Reason::NotReady);
        }
        if !ability.cast_while_moving()
            && (moved
                || actor.intent.axes != [0, 0]
                || actor.body.vertical_speed().abs() > 0.001
                || actor
                    .dodge
                    .is_some_and(|d| d.snapshot().policy.active.is_some()))
        {
            return Err(Reason::Moving);
        }
        let a = actor.position.metres();
        let b = health.position.metres();
        let distance2: f64 = a.into_iter().zip(b).map(|(a, b)| (a - b) * (a - b)).sum();
        if let Some(rules) = ability.combat_rules() {
            if distance2 < rules.minimum_range_metres().powi(2) {
                return Err(Reason::TooClose);
            }
            if let Some(arc) = rules.arc()
                && !actor
                    .facing
                    .is_some_and(|f| arc.contains(f, actor.position, health.position))
            {
                return Err(Reason::NotFacing);
            }
        }
        if distance2 > ability.range_metres().powi(2) {
            return Err(Reason::OutOfRange);
        }
        let patch = world
            .procedural
            .as_ref()
            .and_then(|r| r.patch(actor.position))
            .ok_or(Reason::NotReady)?;
        if !patch.covers(health.position, None) {
            return Err(Reason::NotReady);
        }
        let from = patch
            .frame
            .to_local(actor.position)
            .ok_or(Reason::NotReady)?;
        let to = patch
            .frame
            .to_local(health.position)
            .ok_or(Reason::NotReady)?;
        if !patch.scene.line_clear(
            bevy::prelude::Vec3::from_array(from.metres()),
            bevy::prelude::Vec3::from_array(to.metres()),
        ) {
            return Err(Reason::Obstructed);
        }
        Ok(())
    }
    fn available(&self, id: u64, ability: &Ability, now: u64) -> u64 {
        self.actors.get(&id).map_or(now, |actor| {
            now.max(actor.cast.as_ref().map_or(now, |c| c.completes))
                .max(if ability.uses_global_cooldown() {
                    actor.gcd_until
                } else {
                    now
                })
                .max(actor.cooldowns.get(ability.id()).copied().unwrap_or(now))
        })
    }
    fn start(
        &mut self,
        world: &World,
        id: u64,
        request: Request,
    ) -> std::result::Result<Event, Reason> {
        let mana_cost = self
            .catalog
            .ability(request.ability.as_str())
            .unwrap()
            .mana_cost();
        if self
            .catalog
            .ability(request.ability.as_str())
            .unwrap()
            .projectile()
            .is_some()
            && !self.projectile_capacity(id)
        {
            self.projectile_metrics.capacity_refusals += 1;
            return Err(Reason::Unavailable);
        }
        let timing = self
            .catalog
            .timing(request.ability.as_str(), world.rate)
            .unwrap();
        let completes = world
            .tick
            .checked_add(timing.cast_ticks.max(1))
            .ok_or(Reason::TimeExhausted)?;
        let gcd = world
            .tick
            .checked_add(timing.global_cooldown_ticks)
            .ok_or(Reason::TimeExhausted)?;
        let cooldown = world
            .tick
            .checked_add(timing.cooldown_ticks)
            .ok_or(Reason::TimeExhausted)?;
        if let Effect::ApplyStatus { status } = self
            .catalog
            .ability(request.ability.as_str())
            .unwrap()
            .effect()
        {
            let statuses = self.statuses.as_ref().ok_or(Reason::Unavailable)?;
            let source = world.entity_key(id).ok_or(Reason::Interrupted)?;
            statuses.can_apply(source, request.target, &status)?;
            let timing = statuses
                .catalog
                .definition(&status)
                .ok_or(Reason::Unavailable)?
                .timing(world.rate);
            theia_core::statuses::Schedule::start(completes, timing)
                .map_err(|_| Reason::TimeExhausted)?;
        }
        let cast = Cast {
            sequence: request.sequence,
            ability: request.ability,
            target: request.target,
            started: world.tick,
            completes,
        };
        let swing_plan = self.plan_ability_swing(
            world,
            id,
            self.catalog.ability(cast.ability.as_str()).unwrap(),
        )?;
        let actor = self.actor(id);
        if let Some(mana) = &mut actor.mana
            && !mana.spend(mana_cost)
        {
            return Err(Reason::InsufficientMana);
        }
        self.commit_ability_swing(id, swing_plan);
        let credit_encounter = self.capture_credit();
        let actor = self.actor(id);
        actor.cast = Some(cast.clone());
        actor.cast_credit_encounter = credit_encounter;
        if timing.global_cooldown_ticks > 0 {
            actor.gcd_until = gcd;
        }
        if timing.cooldown_ticks > 0 {
            actor.cooldowns.insert(cast.ability.clone(), cooldown);
        }
        Ok(Event::Started {
            actor: world.entity_key(id).unwrap(),
            cast,
            position: world.players[&id].position,
        })
    }
    fn request(&mut self, world: &World, id: u64, request: Request) -> CastResult {
        let ignored = CastResult {
            reply: Reply {
                sequence: request.sequence,
                outcome: Outcome::Ignored,
            },
            events: vec![],
        };
        let Some(actor_key) = world.entity_key(id) else {
            return ignored;
        };
        let previous = self.actors.get(&id).map_or(0, |a| a.sequence);
        if request.epoch != world.epoch()
            || request.target.epoch.value() != world.epoch()
            || request.sequence <= previous
            || request.sequence.saturating_sub(previous) > 128
        {
            return ignored;
        }
        self.actor(id).sequence = request.sequence;
        let mut events = Vec::with_capacity(2);
        let outcome = (|| {
            if self.actors[&id].interrupted {
                return Err(Reason::Interrupted);
            }
            let ability = self
                .catalog
                .ability(request.ability.as_str())
                .ok_or(Reason::UnknownAbility)?
                .clone();
            self.validate(world, id, request.target, &ability, false)?;
            let at = self.available(id, &ability, world.tick);
            if at > world.tick {
                let window = self
                    .catalog
                    .timing(request.ability.as_str(), world.rate)
                    .unwrap()
                    .queue_window_ticks;
                if at - world.tick > window {
                    return Err(Reason::Unavailable);
                }
                let actor = self.actors.get_mut(&id).unwrap();
                if let Some(old) = actor.queued.take() {
                    events.push(Event::Cancelled {
                        actor: actor_key,
                        sequence: old.request.sequence,
                        reason: Reason::Superseded,
                    });
                }
                let queued = Queued { request, at };
                actor.queued = Some(queued.clone());
                events.push(Event::Queued {
                    actor: actor_key,
                    queued,
                });
                Ok(Outcome::Queued { at })
            } else {
                let event = self.start(world, id, request)?;
                let completes = self.actors[&id].cast.as_ref().unwrap().completes;
                events.push(event);
                Ok(Outcome::Started { completes })
            }
        })()
        .unwrap_or_else(Outcome::Rejected);
        let reply = Reply {
            sequence: self.actors[&id].sequence,
            outcome,
        };
        self.actors.get_mut(&id).unwrap().reply = Some(reply);
        CastResult { reply, events }
    }
    fn cancel(&mut self, key: EntityKey, sequence: u64, reason: Reason) {
        self.events.push(Event::Cancelled {
            actor: key,
            sequence,
            reason,
        });
    }
    fn advance(&mut self, world: &mut World, before: &BTreeMap<u64, GlobalPosition>) {
        self.events.clear();
        if let Some(profile) = &self.resources {
            for actor in self.actors.values_mut() {
                if actor.life.as_ref().is_some_and(|p| !p.state.alive()) {
                    continue;
                }
                if let Some(mana) = &mut actor.mana {
                    mana.regenerate(world.rate, profile.mana_regeneration_per_second());
                }
            }
        }
        for creature in &mut self.creatures {
            if creature.respawn_at.is_some_and(|at| at <= world.tick) {
                let Some(next) = world.next_id.checked_add(1) else {
                    continue;
                };
                let previous = creature.key;
                creature.key.entity = theia_core::identity::EntityId::new(next).unwrap();
                if let Some(home) = world.patrol_respawn(previous, creature.key) {
                    creature.position = home;
                }
                if let Some(pve) = &mut self.pve {
                    pve.respawned_creature(previous, creature.key, world.rate);
                }
                creature.current = creature.maximum;
                creature.respawn_at = None;
                world.next_id = next;
                world.targets.insert(
                    next,
                    crate::targeting::Target {
                        key: creature.key,
                        position: creature.position,
                    },
                );
                self.events.push(Event::Respawned {
                    previous,
                    target: creature.key,
                    position: creature.position,
                });
            }
        }
        self.advance_projectiles(world);
        let ids: Vec<_> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(key) = world.entity_key(id) else {
                self.actors.remove(&id);
                continue;
            };
            if !self.player_alive(id) {
                continue;
            }
            let moved = before.get(&id).is_some_and(|old| {
                let a = old.metres();
                let b = world.players[&id].position.metres();
                a.into_iter()
                    .zip(b)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f64>()
                    > 0.000001
            });
            if self.actors[&id].interrupted {
                let actor = self.actors.get_mut(&id).unwrap();
                let cast = actor.cast.take();
                let queued = actor.queued.take();
                actor.interrupted = false;
                if let Some(cast) = cast {
                    self.cancel(key, cast.sequence, Reason::Interrupted);
                }
                if let Some(queued) = queued {
                    self.cancel(key, queued.request.sequence, Reason::Interrupted);
                }
                continue;
            }
            if let Some(cast) = self.actors[&id].cast.clone() {
                let ability = self.catalog.ability(cast.ability.as_str()).unwrap();
                let failure = if !Self::ready(world, id) {
                    Some(Reason::NotReady)
                } else if !self.actors[&id].equipment.permits(ability) {
                    Some(Reason::EquipmentRequired)
                } else if !self
                    .creatures
                    .iter()
                    .any(|c| c.key == cast.target && c.current > 0)
                {
                    Some(Reason::Dead)
                } else if !ability.cast_while_moving()
                    && (moved
                        || world.players[&id].intent.axes != [0, 0]
                        || world.players[&id].body.vertical_speed().abs() > 0.001
                        || world.players[&id]
                            .dodge
                            .is_some_and(|d| d.snapshot().policy.active.is_some()))
                {
                    Some(Reason::Moving)
                } else if world.tick >= cast.completes {
                    self.validate(world, id, cast.target, ability, moved).err()
                } else {
                    None
                };
                if let Some(reason) = failure {
                    self.actors.get_mut(&id).unwrap().cast = None;
                    self.cancel(key, cast.sequence, reason);
                } else if world.tick >= cast.completes {
                    let hit_table = ability.hit_table();
                    let projectile = ability.projectile();
                    match ability.effect() {
                        Effect::Damage { amount } => {
                            let hit = Hit {
                                credit_encounter: self.actors[&id].cast_credit_encounter,
                                ability: cast.ability.clone(),
                                actor: key,
                                sequence: cast.sequence,
                                target: cast.target,
                                amount,
                            };
                            if let Some(profile) = projectile {
                                self.launch_projectile(world, hit, profile);
                            } else {
                                self.resolve_damage(world, hit, hit_table);
                            }
                        }
                        Effect::ApplyStatus { status } => {
                            let applied = self
                                .statuses
                                .as_mut()
                                .ok_or(Reason::Unavailable)
                                .and_then(|s| s.apply(world.tick, world.rate, key, &cast, &status));
                            if let Err(reason) = applied {
                                self.cancel(key, cast.sequence, reason);
                            }
                        }
                    }
                    self.actors.get_mut(&id).unwrap().cast = None;
                }
            }
            if let Some(queued) = self.actors[&id].queued.clone() {
                let ability = self
                    .catalog
                    .ability(queued.request.ability.as_str())
                    .unwrap();
                let dead = !self
                    .creatures
                    .iter()
                    .any(|c| c.key == queued.request.target && c.current > 0);
                if dead || !Self::ready(world, id) {
                    self.actors.get_mut(&id).unwrap().queued = None;
                    self.cancel(
                        key,
                        queued.request.sequence,
                        if dead { Reason::Dead } else { Reason::NotReady },
                    );
                } else if self.available(id, ability, world.tick) <= world.tick {
                    self.actors.get_mut(&id).unwrap().queued = None;
                    match self.validate(world, id, queued.request.target, ability, moved) {
                        Err(reason) => self.cancel(key, queued.request.sequence, reason),
                        Ok(()) => {
                            let sequence = queued.request.sequence;
                            match self.start(world, id, queued.request) {
                                Ok(event) => self.events.push(event),
                                Err(reason) => self.cancel(key, sequence, reason),
                            }
                        }
                    }
                } else if world.tick >= queued.at {
                    self.actors.get_mut(&id).unwrap().queued = None;
                    self.cancel(key, queued.request.sequence, Reason::Unavailable);
                }
            }
            self.actors
                .get_mut(&id)
                .unwrap()
                .cooldowns
                .retain(|_, at| *at > world.tick);
        }
        self.advance_statuses(world);
        self.advance_autoattack(world);
        self.advance_retaliation(world);
        self.retire_invalid_autoattacks(world);
        // A later actor may kill a target after an earlier actor was already
        // visited. Publish one coherent completed-step state, with every cast
        // on that retired life cancelled in this same step.
        for id in self.actors.keys().copied().collect::<Vec<_>>() {
            let actor = self.actors.get_mut(&id).unwrap();
            let dead_cast = actor.cast.as_ref().is_some_and(|cast| {
                !self
                    .creatures
                    .iter()
                    .any(|c| c.key == cast.target && c.current > 0)
            });
            let dead_queue = actor.queued.as_ref().is_some_and(|queued| {
                !self
                    .creatures
                    .iter()
                    .any(|c| c.key == queued.request.target && c.current > 0)
            });
            let cast = if dead_cast { actor.cast.take() } else { None };
            let queued = if dead_queue {
                actor.queued.take()
            } else {
                None
            };
            let key = world.entity_key(id).unwrap();
            if let Some(cast) = cast {
                self.cancel(key, cast.sequence, Reason::Dead);
            }
            if let Some(queued) = queued {
                self.cancel(key, queued.request.sequence, Reason::Dead);
            }
        }
        debug_assert!(self.events.len() <= self.step_event_budget());
    }
}
impl World {
    pub fn enable_combat(
        &mut self,
        catalog: &[u8],
        health: u32,
        respawn_ms: u32,
    ) -> crate::Result<()> {
        self.enable_combat_config(catalog, health, respawn_ms, None, None)
    }
    pub fn enable_resource_combat(
        &mut self,
        catalog: &[u8],
        health: u32,
        respawn_ms: u32,
        stats: &[u8],
    ) -> crate::Result<()> {
        self.enable_combat_config(catalog, health, respawn_ms, Some(stats), None)
    }
    pub fn enable_status_combat(
        &mut self,
        catalog: &[u8],
        health: u32,
        respawn_ms: u32,
        stats: &[u8],
        statuses: &[u8],
    ) -> crate::Result<()> {
        self.enable_combat_config(catalog, health, respawn_ms, Some(stats), Some(statuses))
    }
    pub fn combat_status_catalog(&self) -> Option<&theia_core::statuses::Catalog> {
        self.combat.as_ref()?.statuses.as_ref().map(|s| &s.catalog)
    }
    pub fn combat_status_snapshot(&self, actor: u64) -> Option<Vec<Status>> {
        let authority = self.combat.as_ref()?;
        let statuses = authority.statuses.as_ref()?;
        Some(statuses.views(self.players.get(&actor)?.position, &authority.creatures))
    }
    fn enable_combat_config(
        &mut self,
        catalog: &[u8],
        health: u32,
        respawn_ms: u32,
        stats: Option<&[u8]>,
        statuses: Option<&[u8]>,
    ) -> crate::Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.procedural.is_none()
            || self.targets.is_empty()
            || self.combat.is_some()
        {
            return Err(
                "Combat requires inactive graybox world with prepared development targets".into(),
            );
        }
        let authority = Authority::create(self, catalog, health, respawn_ms, stats, statuses)?;
        if authority.catalog.schema() >= 7 && !self.classic {
            return Err("Explicit ability rules require classic independent facing".into());
        }
        self.combat = Some(authority);
        Ok(())
    }
    pub fn combat_enabled(&self) -> bool {
        self.combat.is_some()
    }
    pub fn combat_catalog(&self) -> Option<&Catalog> {
        self.combat.as_ref().map(|c| &c.catalog)
    }
    pub fn combat_outcome_metrics(&self) -> Option<OutcomeMetrics> {
        self.combat
            .as_ref()
            .filter(|c| c.catalog.schema() >= 5)
            .map(|c| c.outcome_metrics.clone())
    }
    /// Server-side bridge only: committed character inventory, never appearance
    /// or pending item commands. Exact session identity prevents reuse on rejoin.
    pub(crate) fn observe_combat_equipment(
        &mut self,
        actor: EntityKey,
        committed: Option<([u8; 32], u64, Option<ContentId>)>,
    ) {
        if self.entity_key(actor.entity.value()) != Some(actor) {
            return;
        }
        if let Some(authority) = self.combat.as_mut() {
            authority.observe_gear(actor.entity.value(), committed, self.tick, self.rate);
        }
    }
    pub fn cast(&mut self, actor: u64, request: Request) -> CastResult {
        let Some(mut authority) = self.combat.take() else {
            return CastResult {
                reply: Reply {
                    sequence: request.sequence,
                    outcome: Outcome::Ignored,
                },
                events: vec![],
            };
        };
        let result = authority.request(self, actor, request);
        self.combat = Some(authority);
        result
    }
    /// Events from the most recently completed fixed step; not a reliable outbox.
    /// A transport adopting combat must publish them before advancing again.
    pub fn combat_events(&self) -> &[Event] {
        self.combat.as_ref().map_or(&[], |c| &c.events)
    }
    pub fn combat_snapshot(&self, actor: u64) -> Option<Snapshot> {
        let focus = self.players.get(&actor)?.position;
        let authority = self.combat.as_ref()?;
        let state = authority.actors.get(&actor);
        let mut targets: Vec<_> = authority
            .creatures
            .iter()
            .filter(|c| crate::targeting::relevant(focus, c.position))
            .cloned()
            .collect();
        targets.sort_by_key(|c| c.key.entity);
        Some(Snapshot {
            vitals: authority.resources.as_ref().map(|p| {
                Vitals::new(
                    if authority.pve.is_some() {
                        state
                            .and_then(|a| a.life.as_ref())
                            .map_or(0, |p| p.state.health())
                    } else {
                        p.maximum_health()
                    },
                    p.maximum_health(),
                    state
                        .and_then(|a| a.mana)
                        .map_or(p.maximum_mana(), |m| m.current()),
                    p.maximum_mana(),
                )
                .unwrap()
            }),
            tick: self.tick,
            reply: state.and_then(|a| a.reply),
            cast: state.and_then(|a| a.cast.clone()),
            queued: state.and_then(|a| a.queued.clone()),
            gcd_until: state.map_or(0, |a| a.gcd_until),
            cooldowns: state.map_or_else(Vec::new, |a| {
                a.cooldowns
                    .iter()
                    .map(|(id, at)| (id.clone(), *at))
                    .collect()
            }),
            targets,
        })
    }
    pub(super) fn advance_combat(&mut self, before: BTreeMap<u64, GlobalPosition>) {
        if let Some(mut authority) = self.combat.take() {
            authority.advance(self, &before);
            self.combat = Some(authority);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3;
    use std::sync::Arc;
    #[test]
    fn actual_world_sight_rejects_a_wall_at_start_and_rechecks_it_at_completion() {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        let target = world.spawn_development_target([8008., 8000.]).unwrap();
        world
            .enable_combat(
                include_bytes!("../../../../content/gameplay/development-abilities.json"),
                100,
                1000,
            )
            .unwrap();
        let actor = world.join().unwrap();
        let position = world.global_position(actor).unwrap();
        let original = world
            .procedural
            .as_ref()
            .unwrap()
            .patch(position)
            .unwrap()
            .clone();
        assert!(world.client_ready(
            actor,
            &crate::wire::ChunkReady {
                version: crate::wire::VERSION,
                epoch: world.epoch(),
                sequence: 1,
                chunks: original.ids.iter().map(|id| [id.x, id.z]).collect()
            }
        ));
        let from = Vec3::from_array(original.frame.to_local(position).unwrap().metres());
        let to = Vec3::from_array(
            original
                .frame
                .to_local(world.targets[&target.entity.value()].position)
                .unwrap()
                .metres(),
        );
        let mut geometry = original.scene.colliders.clone();
        geometry.push(crate::physics::PlacedCollider {
            shape: avian3d::prelude::Collider::cuboid(0.1, 8., 16.),
            center: (from + to) / 2.,
            layer: crate::physics::SOLID,
        });
        let wall = Arc::new(crate::procedural::Patch {
            owner: original.owner,
            frame: original.frame,
            ids: original.ids.clone(),
            scene: crate::physics::StaticScene::new(geometry),
        });
        world
            .procedural
            .as_mut()
            .unwrap()
            .patches
            .insert(original.owner, wall.clone());
        let request = |world: &World, sequence| Request {
            epoch: world.epoch(),
            sequence,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target,
        };
        assert_eq!(
            world.cast(actor, request(&world, 1)).reply.outcome,
            Outcome::Rejected(Reason::Obstructed)
        );
        world
            .procedural
            .as_mut()
            .unwrap()
            .patches
            .insert(original.owner, original.clone());
        assert!(matches!(
            world.cast(actor, request(&world, 2)).reply.outcome,
            Outcome::Started { .. }
        ));
        world
            .procedural
            .as_mut()
            .unwrap()
            .patches
            .insert(original.owner, wall);
        for _ in 0..30 {
            world.step();
        }
        assert!(matches!(
            world.combat_events(),
            [Event::Cancelled {
                reason: Reason::Obstructed,
                ..
            }]
        ));
        assert_eq!(
            world.combat_snapshot(actor).unwrap().targets[0].current,
            100
        );
    }
}

#[cfg(test)]
mod melee_tests;
#[cfg(test)]
mod outcome_tests;
#[cfg(test)]
mod projectile_tests;
