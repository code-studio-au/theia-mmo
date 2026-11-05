//! Per-character async item owner. Generation-bound admission and retirement
//! survive until accepted queued and executing SQL finishes; the simulation only uses its port.
use super::{GameAdmissions, Phase};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};
use theia_core::{
    content::ContentId,
    inventory::{Bag, ItemCommand, Stack},
};
use theia_network::{
    Result,
    inventory_wire::{Outcome, Reason, Reply},
    renet_runtime::inventory::{Observation, Port, Request},
    simulation::rewards::Opportunity,
};
use theia_persistence::{Binding, Error, InventoryCatalog, Publisher, RewardCatalog, RewardClaim};
use tokio::sync::{mpsc, watch};
pub(super) struct Config {
    journal: Option<theia_persistence::QuestJournalCatalog>,
    vendor: Option<(
        theia_persistence::VendorCatalog,
        Arc<theia_network::vendor_wire::Profile>,
    )>,
    rewards: Option<RewardCatalog>,
    reward_wire: Option<Arc<theia_network::reward_wire::Profile>>,
    profile: InventoryCatalog,
    catalog: Arc<theia_core::inventory::Catalog>,
    seed: Bag,
    publisher: Publisher,
}
impl Config {
    pub(super) fn set_journal(
        &mut self,
        journal: theia_persistence::QuestJournalCatalog,
    ) -> Result<()> {
        if self.journal.is_some()
            || journal
                .inventory()
                .is_none_or(|i| i.hash() != self.profile.hash())
        {
            return Err("Invalid inventory journal binding".into());
        }
        self.journal = Some(journal);
        Ok(())
    }
    pub(super) fn profile_hash(&self) -> [u8; 32] {
        self.profile.hash()
    }

    pub(super) fn new(profile: InventoryCatalog, publisher: Publisher) -> Result<Self> {
        // Explicit opt-in development fixture, applied only to a missing bag.
        // This is not a client grant command or a production reward rule.
        let mut slots = std::array::from_fn(|_| None);
        for (slot, suffix, count) in [(0, "stone", 13_u32), (1, "stone", 15), (2, "token", 99)] {
            let id = ContentId::new(&format!("theia.dev.training-{suffix}")).unwrap();
            let limit = profile
                .catalog()
                .definition(&id)
                .ok_or("Development item fixture missing definition")?
                .maximum_stack();
            slots[slot] = Stack::new(profile.catalog(), id, count.min(limit));
        }
        if profile.catalog().equipment_enabled() {
            for (slot, suffix) in [(3, "sword"), (4, "wand")] {
                let id = ContentId::new(&format!("theia.dev.training-{suffix}")).unwrap();
                if profile
                    .catalog()
                    .definition(&id)
                    .is_none_or(|d| d.main_hand_appearance().is_none())
                {
                    return Err("Development equipment fixture missing eligible definition".into());
                }
                slots[slot] = Stack::new(profile.catalog(), id, 1);
            }
        }
        let seed = Bag::from_slots(profile.catalog(), 0, slots)
            .map_err(|_| "Invalid development inventory seed")?;
        Ok(Self {
            journal: None,
            vendor: None,
            rewards: None,
            reward_wire: None,
            catalog: Arc::new(profile.catalog().clone()),
            profile,
            seed,
            publisher,
        })
    }
    pub(super) fn set_vendor(
        &mut self,
        profile: theia_persistence::VendorCatalog,
        wire: Arc<theia_network::vendor_wire::Profile>,
    ) -> Result<()> {
        if self.vendor.is_some()
            || profile.items().hash() != self.profile.hash()
            || profile.hash() != wire.hash()
            || wire.item_hash() != self.profile.hash()
        {
            return Err("Vendor admission/profile mismatch".into());
        }
        self.vendor = Some((profile, wire));
        Ok(())
    }
    pub(super) fn with_rewards(mut self, rewards: RewardCatalog) -> Result<Self> {
        if rewards.catalog().has_quest_required() {
            return Err("Quest-only loot owner is not installed".into());
        }
        if rewards.items().hash() != self.profile.hash() {
            return Err("Reward/item admission profile mismatch".into());
        }
        self.rewards = Some(rewards);
        Ok(self)
    }
    pub(super) fn set_quest_loot(&mut self, rewards: RewardCatalog) -> Result<()> {
        if self.rewards.is_some()
            || !rewards.catalog().has_quest_required()
            || rewards.items().hash() != self.profile.hash()
            || rewards.quest_journal().is_none_or(|bound| {
                self.journal
                    .as_ref()
                    .is_none_or(|j| j.hash() != bound.hash())
            })
        {
            return Err("Quest loot requires the installed exact item journal".into());
        }
        self.rewards = Some(rewards);
        Ok(())
    }
    pub(super) fn set_reward_wire(
        &mut self,
        profile: Arc<theia_network::reward_wire::Profile>,
    ) -> Result<()> {
        if self
            .rewards
            .as_ref()
            .is_none_or(|r| r.hash() != profile.hash())
            || profile.item_hash() != self.profile.hash()
        {
            return Err("Reward admission/wire profile mismatch".into());
        }
        self.reward_wire = Some(profile);
        Ok(())
    }
    pub(super) fn start(
        &self,
        gate: Weak<GameAdmissions>,
        slot: usize,
        generation: u64,
        binding: Binding,
        world: [u8; 32],
    ) -> Session {
        let (requests, receiver) = mpsc::channel(1);
        let (opportunities, earned) = mpsc::channel(theia_core::rewards::MAX_DEFINITIONS);
        let (state, observations) = watch::channel(Observation::default());
        let (stop, stopping) = watch::channel(false);
        let done = Arc::new(AtomicBool::new(false));
        let guard = Finished {
            gate: gate.clone(),
            done: done.clone(),
            handled: false,
        };
        let owner = Owner {
            gate,
            slot,
            generation,
            binding,
            world,
            profile: self.profile.clone(),
            journal: self.journal.clone(),
            publisher: self.publisher.clone(),
            seed: self.seed.clone(),
            rewards: self.rewards.clone(),
            vendor: self.vendor.clone(),
        };
        let publication = state.clone();
        tokio::spawn(async move {
            let mut guard = guard;
            let result = owner.run(receiver, earned, stopping, &state).await;
            if result.is_err()
                && let Some(gate) = guard.gate.upgrade()
            {
                gate.fail_closed();
            }
            state.send_modify(|s| {
                s.failed = result.is_err();
                s.closed = true;
            });
            guard.handled = true;
        });
        Session {
            publish: publication,
            port: Port {
                documents: None,
                vendor: self.vendor.as_ref().map(|v| v.1.clone()),
                rewards: self.reward_wire.clone(),
                opportunities: self.rewards.as_ref().map(|_| opportunities),
                catalog: self.catalog.clone(),
                hash: self.profile.hash(),
                requests,
                state: observations,
            },
            stop,
            done,
        }
    }
}
pub(super) struct Session {
    pub(super) publish: watch::Sender<Observation>,
    pub(super) port: Port,
    stop: watch::Sender<bool>,
    done: Arc<AtomicBool>,
}
impl Session {
    pub(super) fn stop(&self) {
        self.stop.send_replace(true);
    }
    pub(super) fn finished(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.stop();
    }
}
struct Finished {
    gate: Weak<GameAdmissions>,
    done: Arc<AtomicBool>,
    handled: bool,
}
impl Drop for Finished {
    fn drop(&mut self) {
        // A panic/cancel can abandon an answer while its SQL job is executing.
        // Deny all new admission instead of releasing ownership ambiguously.
        if !self.handled
            && let Some(gate) = self.gate.upgrade()
        {
            gate.fail_closed();
        }
        self.done.store(true, Ordering::Release);
    }
}
struct Owner {
    vendor: Option<(
        theia_persistence::VendorCatalog,
        Arc<theia_network::vendor_wire::Profile>,
    )>,
    rewards: Option<RewardCatalog>,
    gate: Weak<GameAdmissions>,
    slot: usize,
    generation: u64,
    binding: Binding,
    world: [u8; 32],
    profile: InventoryCatalog,
    journal: Option<theia_persistence::QuestJournalCatalog>,
    publisher: Publisher,
    seed: Bag,
}
impl Owner {
    fn retained_authorized(&self) -> Result<bool> {
        let Some(gate) = self.gate.upgrade() else {
            return Ok(false);
        };
        let state = gate.lock()?;
        Ok(state.slots[self.slot].as_ref().is_some_and(|e| {
            e.generation == self.generation
                && e.binding == self.binding
                && matches!(e.phase, Phase::Consumed | Phase::Active | Phase::Retiring)
        }))
    }
    fn authorized(&self) -> Result<bool> {
        let Some(gate) = self.gate.upgrade() else {
            return Ok(false);
        };
        let state = gate.lock()?;
        let Some(entry) = state.slots[self.slot].as_ref().filter(|e| {
            e.generation == self.generation && matches!(e.phase, Phase::Consumed | Phase::Active)
        }) else {
            return Ok(false);
        };
        Ok(gate.authorized(entry)?)
    }
    async fn run(
        self,
        mut requests: mpsc::Receiver<Request>,
        mut earned: mpsc::Receiver<Opportunity>,
        mut stop: watch::Receiver<bool>,
        state: &watch::Sender<Observation>,
    ) -> Result<()> {
        if if self.rewards.is_some() {
            !self.retained_authorized()?
        } else {
            *stop.borrow() || !self.authorized()?
        } {
            return Ok(());
        }
        let mut bag = self
            .publisher
            .ensure_inventory(self.binding, self.world, &self.profile, &self.seed)
            .await?;
        if let Some(catalog) = &self.journal {
            let applied = self
                .publisher
                .ensure_quest_journal(self.binding, self.world, catalog)
                .await?;
            self.publish_journal(applied)?;
        }
        let mut interest_versions = self.loot_versions()?;
        let (mut claims, mut interest_center) = if let Some(profile) = &self.rewards {
            self.load_claims(profile).await?
        } else {
            (Vec::new(), None)
        };
        let has_life = self.rewards.as_ref().is_some_and(|p| {
            p.catalog()
                .definitions()
                .any(|d| d.scope() == theia_core::rewards::Scope::PerCreatureLife)
        });
        let mut interest_timer = tokio::time::interval(std::time::Duration::from_millis(500));
        interest_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut reward_generation = 1;
        if self.rewards.is_some() {
            state.send_modify(|s| s.rewards = Some(owned_rewards(reward_generation, &claims)));
        }
        if (*stop.borrow() || !self.authorized()?) && self.rewards.is_none() {
            return Ok(());
        }
        let mut merged = Ok(());
        state.send_modify(|s| merged = super::quest_items::merge_bag(&mut s.bag, bag.clone()));
        merged?;
        let mut sequence = 0;
        let mut retiring = false;
        loop {
            if retiring || *stop.borrow() || !self.authorized()? {
                // Close both acceptance points before draining. Already accepted
                // commands retain the character generation through retirement;
                // session expiry must not cancel an item intent after enqueue.
                requests.close();
                earned.close();
                retiring = true;
            }
            let request = if retiring {
                match requests.recv().await {
                    Some(request) => request,
                    None => break,
                }
            } else {
                tokio::select! {biased;
                    _=stop.changed()=>{ retiring = true; continue; },
                    _=interest_timer.tick(),if has_life=>{
                        let position = self.loot_position()?;
                        let versions = self.loot_versions()?;
                        let changed = versions != interest_versions || interest_center.is_none_or(|old| old.metres().into_iter().zip(position.metres()).map(|(a,b)|(a-b)*(a-b)).sum::<f64>() >= 64.);
                        if changed {
                            let profile = self.rewards.as_ref().ok_or("Loot profile missing")?;
                            let (next, center) = self.load_claims(profile).await?;
                            interest_center = center;
                            // Sample before the query: a concurrent commit must remain dirty.
                            interest_versions = versions;
                            if next != claims {
                                claims = next;
                                reward_generation = reward_generation.checked_add(1).ok_or("Reward generation exhausted")?;
                                state.send_modify(|s| s.rewards=Some(owned_rewards(reward_generation,&claims)));
                            }
                        }
                        continue;
                    },
                    reward=earned.recv(),if self.rewards.is_some()=>{
                        if let Some(reward)=reward {
                            if self.offer(&reward, &mut claims).await? {
                                reward_generation = reward_generation.checked_add(1).ok_or("Reward generation exhausted")?;
                                state.send_modify(|s|s.rewards=Some(owned_rewards(reward_generation,&claims)));
                            }
                            continue;
                        }
                        retiring = true;
                        continue;
                    },
                    request=requests.recv()=>match request {Some(r)=>r,None=>break},
                }
            };
            if !self.retained_authorized()? {
                return Err("Accepted item intent lost its character generation".into());
            }
            if request.sequence <= sequence {
                return Err("Discontinuous inventory owner request".into());
            }
            sequence = request.sequence;
            // Deliberately not selected against stop: accepted critical work
            // finishes before Finished permits character ownership release.
            let applied = if let Some(journal) = &self.journal {
                match request.command {
                    ItemCommand::Vendor(command) => {
                        let (profile, wire) =
                            self.vendor.as_ref().ok_or("Vendor owner profile missing")?;
                        self.publisher
                            .purchase_vendor_with_quest_journal(
                                self.binding,
                                self.world,
                                profile,
                                &wire.purchase(command)?,
                                journal,
                            )
                            .await
                    }
                    ItemCommand::RewardClaim(command) => {
                        let profile = self.rewards.as_ref().ok_or("Reward profile missing")?;
                        if profile.quest_journal().is_some() {
                            self.publisher
                                .pickup_quest_reward(
                                    self.binding,
                                    self.world,
                                    profile,
                                    command.claim,
                                    command.expected_revision,
                                )
                                .await
                        } else {
                            self.publisher
                                .claim_reward_with_quest_journal(
                                    self.binding,
                                    self.world,
                                    profile,
                                    command.claim,
                                    command.expected_revision,
                                    journal,
                                )
                                .await
                        }
                    }
                    command => {
                        self.publisher
                            .mutate_items_with_quest_journal(
                                self.binding,
                                self.world,
                                &self.profile,
                                command,
                                journal,
                            )
                            .await
                    }
                }
            } else {
                match request.command {
                    ItemCommand::Vendor(command) => {
                        let (profile, wire) =
                            self.vendor.as_ref().ok_or("Vendor owner profile missing")?;
                        let purchase = wire.purchase(command)?;
                        self.publisher
                            .purchase_vendor(self.binding, self.world, profile, &purchase)
                            .await
                    }
                    ItemCommand::RewardClaim(command) => {
                        let profile = self
                            .rewards
                            .as_ref()
                            .ok_or("Reward claim profile is not installed")?;
                        self.publisher
                            .claim_reward(
                                self.binding,
                                self.world,
                                profile,
                                command.claim,
                                command.expected_revision,
                            )
                            .await
                    }
                    ItemCommand::Backpack(command) => {
                        self.publisher
                            .mutate_inventory(self.binding, self.world, &self.profile, command)
                            .await
                    }
                    ItemCommand::OpenDocument(_) => {
                        return Err("Document owner is not installed".into());
                    }
                    ItemCommand::Equipment(command) => {
                        self.publisher
                            .mutate_equipment(self.binding, self.world, &self.profile, command)
                            .await
                    }
                }
            };
            if matches!(request.command, ItemCommand::RewardClaim(_)) {
                let profile = self.rewards.as_ref().ok_or("Reward profile missing")?;
                let (next, _) = self.load_claims(profile).await?;
                if next != claims {
                    reward_generation = reward_generation
                        .checked_add(1)
                        .ok_or("Reward generation exhausted")?;
                    claims = next;
                }
            }
            let outcome = match applied {
                Ok(mut applied) => {
                    if let Some(journal) = applied.journal.take() {
                        self.publish_journal(*journal)?;
                    }
                    bag = applied.bag;
                    if applied.replayed {
                        Outcome::Replayed
                    } else {
                        Outcome::Applied
                    }
                }
                Err(error) => {
                    let reason = reason(&error).ok_or(error)?;
                    // A full queue means this intent was never submitted. All
                    // ambiguous deadlines/closed-worker errors are fatal above.
                    if reason != Reason::Busy
                        && let Some(journal) = &self.journal
                    {
                        let saved = self
                            .publisher
                            .load_quest_journal(self.binding, self.world, journal)
                            .await?;
                        bag = saved
                            .inventory
                            .clone()
                            .ok_or("Item journal returned no bag")?;
                        self.publish_journal(saved)?;
                    } else if reason != Reason::Busy {
                        bag = self
                            .publisher
                            .load_inventory(self.binding, self.world, &self.profile)
                            .await?;
                    }
                    Outcome::Rejected(reason)
                }
            };
            if *stop.borrow() || !self.authorized()? {
                retiring = true;
                continue;
            }
            let mut merged = Ok(());
            state.send_modify(|s| {
                merged = super::quest_items::merge_bag(&mut s.bag, bag.clone());
                s.reply = Some(Reply { sequence, outcome });
                s.rewards = self
                    .rewards
                    .as_ref()
                    .map(|_| owned_rewards(reward_generation, &claims));
            });
            merged?;
        }
        // Every ordinary exit closes acceptance before draining the finite set
        // of trusted facts, including logout and a closed client request port.
        earned.close();
        while let Some(reward) = earned.recv().await {
            self.offer(&reward, &mut claims).await?;
        }
        Ok(())
    }
    fn publish_journal(&self, applied: theia_persistence::QuestJournalApplied) -> Result<()> {
        let catalog = self.journal.as_ref().ok_or("Item journal not configured")?;
        self.gate
            .upgrade()
            .ok_or("Item journal admission gate closed")?
            .publish_quest_items(
                self.slot,
                self.generation,
                self.binding,
                catalog.hash(),
                applied,
                None,
            )
    }
    fn loot_position(&self) -> Result<theia_core::coordinates::GlobalPosition> {
        let gate = self.gate.upgrade().ok_or("Loot owner gate closed")?;
        let state = gate.lock()?;
        state.slots[self.slot]
            .as_ref()
            .filter(|e| e.generation == self.generation && e.binding == self.binding)
            .and_then(|e| e.saved.as_ref().map(|s| s.position))
            .ok_or_else(|| "Loot owner lost its retained pose".into())
    }
    // Dirty hints only; Store queries still lock and derive authoritative need.
    // Release the gate before borrowing either watch to avoid lock inversion.
    fn loot_versions(&self) -> Result<(Option<u64>, Option<u64>)> {
        if self
            .rewards
            .as_ref()
            .is_none_or(|r| r.quest_journal().is_none())
        {
            return Ok((None, None));
        }
        let gate = self.gate.upgrade().ok_or("Loot owner gate closed")?;
        let (items, journal) = {
            let state = gate.lock()?;
            let entry = state.slots[self.slot]
                .as_ref()
                .filter(|e| e.generation == self.generation && e.binding == self.binding)
                .ok_or("Loot owner lost character generation")?;
            (
                entry
                    .inventory
                    .as_ref()
                    .ok_or("Loot inventory missing")?
                    .publish
                    .clone(),
                entry
                    .progression
                    .as_ref()
                    .ok_or("Loot journal missing")?
                    .publish
                    .clone(),
            )
        };
        let bag = items.borrow().bag.as_ref().map(|b| b.revision());
        let head = journal.borrow().journal.as_ref().map(|j| j.revision);
        Ok((bag, head))
    }
    async fn load_claims(
        &self,
        profile: &RewardCatalog,
    ) -> Result<(
        Vec<RewardClaim>,
        Option<theia_core::coordinates::GlobalPosition>,
    )> {
        if profile
            .catalog()
            .definitions()
            .any(|d| d.scope() == theia_core::rewards::Scope::PerCreatureLife)
        {
            let position = self.loot_position()?;
            Ok((
                if profile.quest_journal().is_some() {
                    self.publisher
                        .list_quest_reward_claims_nearby(
                            self.binding,
                            self.world,
                            profile,
                            position,
                        )
                        .await?
                } else {
                    self.publisher
                        .list_reward_claims_nearby(self.binding, self.world, profile, position)
                        .await?
                },
                Some(position),
            ))
        } else {
            Ok((
                self.publisher
                    .list_reward_claims(self.binding, self.world, profile)
                    .await?,
                None,
            ))
        }
    }
    async fn offer(&self, fact: &Opportunity, claims: &mut Vec<RewardClaim>) -> Result<bool> {
        if !self.retained_authorized()? {
            return Err("Earned reward lost its character generation".into());
        }
        let profile = self
            .rewards
            .as_ref()
            .ok_or("Reward opportunity without profile")?;
        let definition = profile
            .catalog()
            .definition(&fact.reward)
            .ok_or("Unknown earned reward definition")?;
        let occurrence = definition
            .occurrence(&fact.event)
            .map_err(|_| "Earned reward creature mismatch")?;
        if claims.iter().any(|c| {
            c.reward == fact.reward
                && match occurrence {
                    theia_core::rewards::Occurrence::OncePerCharacter => c.corpse.is_none(),
                    theia_core::rewards::Occurrence::CreatureLife(life) => {
                        c.corpse.is_some_and(|s| s.life == life)
                    }
                }
        }) {
            return Ok(false);
        }
        // Persist the original fatal fact. Offering never mutates the backpack
        // or possession journal. SQL uniqueness retains consumed life receipts.
        if profile.quest_journal().is_some() {
            self.publisher
                .generate_quest_loot_for_kill(
                    self.binding,
                    self.world,
                    profile,
                    &fact.reward,
                    &fact.event,
                    fact.position,
                )
                .await?;
        } else {
            self.publisher
                .roll_reward_for_kill(
                    self.binding,
                    self.world,
                    profile,
                    &fact.reward,
                    &fact.event,
                    fact.position,
                )
                .await?;
        }
        // The store owns the bounded pending page; never accumulate unbounded
        // life history in a resident owner or suppress a later life by recipe.
        let (next, _) = self.load_claims(profile).await?;
        let changed = next != *claims;
        *claims = next;
        Ok(changed)
    }
}
fn owned_rewards(
    generation: u64,
    claims: &[RewardClaim],
) -> theia_network::renet_runtime::inventory::OwnedRewards {
    theia_network::renet_runtime::inventory::OwnedRewards {
        generation,
        available: claims
            .iter()
            .filter(|c| c.consumed_revision.is_none())
            .map(|c| theia_network::reward_wire::Available {
                reward: c.reward.clone(),
                claim: c.id,
                corpse: c.corpse,
            })
            .collect(),
    }
}
fn reason(error: &Error) -> Option<Reason> {
    use theia_core::inventory::Error as Core;
    Some(match error {
        Error::VendorCommand(theia_core::vendors::Error::InsufficientCost) => Reason::EmptySource,
        Error::VendorCommand(theia_core::vendors::Error::NoCapacity) => Reason::Full,
        Error::VendorCommand(theia_core::vendors::Error::StaleRevision) => Reason::Stale,
        Error::VendorCommand(theia_core::vendors::Error::RevisionExhausted) => Reason::Exhausted,
        Error::VendorCommand(theia_core::vendors::Error::UnknownOffer) => Reason::Conflict,
        Error::InventoryBusy => Reason::Busy,
        Error::InventoryStale | Error::InventoryCommand(Core::StaleRevision) => Reason::Stale,
        Error::InventoryConflict => Reason::Conflict,
        Error::RewardMissing | Error::RewardConsumed | Error::RewardNotNeeded => Reason::Conflict,
        Error::RewardCommand(theia_core::rewards::Error::NoCapacity) => Reason::Full,
        Error::RewardCommand(theia_core::rewards::Error::StaleRevision) => Reason::Stale,
        Error::RewardCommand(theia_core::rewards::Error::RevisionExhausted) => Reason::Exhausted,
        Error::InventoryCommand(Core::InvalidSlot) => Reason::Slot,
        Error::InventoryCommand(Core::SameSlot) => Reason::SameSlot,
        Error::InventoryCommand(Core::EmptySource) => Reason::EmptySource,
        Error::InventoryCommand(Core::DestinationFull) => Reason::Full,
        Error::InventoryCommand(Core::DestinationOccupied) => Reason::Occupied,
        Error::InventoryCommand(Core::InvalidAmount) => Reason::Amount,
        Error::InventoryCommand(Core::RevisionExhausted) => Reason::Exhausted,
        Error::InventoryCommand(Core::EquipmentDisabled) => Reason::EquipmentDisabled,
        Error::InventoryCommand(Core::NotEquippable) => Reason::NotEquippable,
        Error::InventoryCommand(Core::EmptyHand) => Reason::EmptyHand,
        _ => return None,
    })
}
