//! Synchronous bounded bridge to an asynchronous, character-bound item owner.
//! No SQL or provider dependencies. The authority must finish executing work
//! before releasing a retiring character; transport queueing is never COMMIT.
use std::sync::Arc;
use theia_core::inventory::{Bag, Catalog, ItemCommand};
use tokio::sync::{mpsc, watch};
#[derive(Clone, Debug, Default)]
pub struct Observation {
    /// None means loading, never an invented empty backpack.
    pub bag: Option<Bag>,
    pub reply: Option<crate::inventory_wire::Reply>,
    /// Durable availability only; None means loading or rewards disabled.
    pub rewards: Option<OwnedRewards>,
    pub closed: bool,
    pub failed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedRewards {
    pub generation: u64,
    pub available: Vec<crate::reward_wire::Available>,
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub sequence: u64,
    pub command: ItemCommand,
}
#[derive(Clone)]
pub struct Port {
    pub vendor: Option<Arc<crate::vendor_wire::Profile>>,
    pub rewards: Option<Arc<crate::reward_wire::Profile>>,
    /// Trusted authority facts only. Never populated from a client request.
    pub opportunities: Option<mpsc::Sender<theia_core::content::ContentId>>,
    pub catalog: Arc<Catalog>,
    pub hash: [u8; 32],
    /// Bound to a server-owned character, never a client-supplied UUID.
    pub requests: mpsc::Sender<Request>,
    pub state: watch::Receiver<Observation>,
}
#[derive(Clone)]
pub struct Profile {
    catalog: Arc<Catalog>,
    hash: [u8; 32],
    rewards: Option<Arc<crate::reward_wire::Profile>>,
    vendor: Option<Arc<crate::vendor_wire::Profile>>,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> crate::Result<Self> {
        let catalog = Catalog::from_json(bytes)?;
        Ok(Self {
            catalog: Arc::new(catalog),
            hash: crate::terrain::sha256(bytes),
            rewards: None,
            vendor: None,
        })
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn with_rewards(mut self, bytes: &[u8]) -> crate::Result<Self> {
        self.rewards = Some(Arc::new(crate::reward_wire::Profile::from_json(
            bytes, &self,
        )?));
        Ok(self)
    }
    pub fn with_vendor(mut self, bytes: &[u8]) -> crate::Result<Self> {
        self.vendor = Some(Arc::new(crate::vendor_wire::Profile::from_json(
            bytes, &self,
        )?));
        Ok(self)
    }
    pub fn vendor(&self) -> Option<&crate::vendor_wire::Profile> {
        self.vendor.as_deref()
    }
    pub fn rewards(&self) -> Option<&crate::reward_wire::Profile> {
        self.rewards.as_deref()
    }
}
pub struct Client {
    pub profile: Profile,
    pub commands: mpsc::Receiver<ItemCommand>,
}
pub(super) struct ClientState {
    config: Client,
    pub(super) view: Option<crate::inventory_wire::State>,
    pub(super) appearance: Option<crate::appearance_wire::State>,
    pub(super) rewards: Option<crate::reward_wire::State>,
    sequence: u64,
    pending: Option<(u64, ItemCommand, tokio::time::Instant)>,
}
impl ClientState {
    pub(super) fn new(config: Client) -> crate::Result<Self> {
        if config.commands.max_capacity() != 1 {
            return Err("Inventory intent queue must have capacity one".into());
        }
        Ok(Self {
            config,
            view: None,
            appearance: None,
            rewards: None,
            sequence: 0,
            pending: None,
        })
    }
    pub(super) fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn check_deadline(
        &self,
        now: tokio::time::Instant,
        started: tokio::time::Instant,
    ) -> crate::Result<()> {
        if (self.view.is_none() && now.duration_since(started) > std::time::Duration::from_secs(8))
            || (self.config.profile.catalog().equipment_enabled()
                && self.appearance.is_none()
                && now.duration_since(started) > std::time::Duration::from_secs(8))
            || (self.config.profile.rewards().is_some()
                && self.rewards.is_none()
                && now.duration_since(started) > std::time::Duration::from_secs(8))
            || self.pending.is_some_and(|(_, _, sent)| {
                now.duration_since(sent) > std::time::Duration::from_secs(8)
            })
        {
            return Err("Authoritative inventory observation timed out".into());
        }
        Ok(())
    }
    pub(super) fn receive_rewards(
        &mut self,
        bytes: &[u8],
        actor: theia_core::identity::EntityKey,
    ) -> crate::Result<()> {
        let profile = self
            .config
            .profile
            .rewards()
            .ok_or("Unexpected reward observation")?;
        let view = crate::reward_wire::State::decode(bytes, profile.catalog(), profile.hash())?;
        if view.actor != actor
            || self.rewards.as_ref().is_some_and(|old| {
                view.generation < old.generation
                    || (view.generation == old.generation && view != *old)
            })
        {
            return Err("Reward actor/generation observation mismatch".into());
        }
        self.rewards = Some(view);
        Ok(())
    }
    pub(super) fn receive_appearance(
        &mut self,
        bytes: &[u8],
        actor: theia_core::identity::EntityKey,
    ) -> crate::Result<()> {
        let view = crate::appearance_wire::State::decode(
            bytes,
            self.config.profile.catalog(),
            self.config.profile.hash(),
        )?;
        if view.epoch != actor.epoch.value()
            || self
                .appearance
                .as_ref()
                .is_some_and(|old| old.tick > view.tick)
            || (self.view.is_some() && !view.actors.iter().any(|a| a.key == actor))
        {
            return Err("Appearance epoch/tick/owner observation mismatch".into());
        }
        self.appearance = Some(view);
        Ok(())
    }
    pub(super) fn receive(
        &mut self,
        bytes: &[u8],
        actor: theia_core::identity::EntityKey,
    ) -> crate::Result<()> {
        use crate::inventory_wire::Outcome;
        let state = crate::inventory_wire::State::decode(
            bytes,
            &self.config.profile.catalog,
            self.config.profile.hash,
        )?;
        if state.actor != actor || state.reply.is_some_and(|r| r.sequence > self.sequence) {
            return Err("Foreign or unsent inventory acknowledgement".into());
        }
        let prior_sequence = self
            .view
            .as_ref()
            .and_then(|s| s.reply)
            .map_or(0, |r| r.sequence);
        let sequence = state.reply.map_or(0, |r| r.sequence);
        if self
            .view
            .as_ref()
            .is_some_and(|old| state.bag.revision() < old.bag.revision())
            || sequence < prior_sequence
        {
            return Err("Authoritative inventory state regressed".into());
        }
        if sequence > prior_sequence {
            let (pending, command, _) = self
                .pending
                .ok_or("Inventory reply without a pending intent")?;
            if sequence != pending {
                return Err("Inventory reply did not match pending intent".into());
            }
            if matches!(
                state.reply.unwrap().outcome,
                Outcome::Applied | Outcome::Replayed
            ) && command.expected_revision().checked_add(1) != Some(state.bag.revision())
            {
                return Err("Inventory success revision did not match intent".into());
            }
            self.pending = None;
        }
        self.view = Some(state);
        Ok(())
    }
    pub(super) fn next(
        &mut self,
        actor: theia_core::identity::EntityKey,
        now: tokio::time::Instant,
    ) -> crate::Result<Option<Vec<u8>>> {
        if self.pending.is_some() || self.view.is_none() {
            return Ok(None);
        }
        let Ok(command) = self.config.commands.try_recv() else {
            return Ok(None);
        };
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or("Inventory sequence exhausted")?;
        let bytes = if let ItemCommand::Vendor(command) = command {
            self.config
                .profile
                .vendor()
                .ok_or("Vendor intent requires enabled profile")?
                .purchase(command)?;
            crate::vendor_wire::Request {
                actor,
                sequence,
                command,
            }
            .encode()?
        } else if let ItemCommand::RewardClaim(command) = command {
            self.config
                .profile
                .rewards()
                .ok_or("Reward intent requires reward profile")?;
            crate::reward_wire::Request {
                actor,
                sequence,
                command,
            }
            .encode()?
        } else if self.config.profile.catalog().equipment_enabled() {
            crate::inventory_wire::ItemRequest {
                actor,
                sequence,
                command,
            }
            .encode()?
        } else {
            let ItemCommand::Backpack(command) = command else {
                return Err("Equipment intent requires profile 2".into());
            };
            crate::inventory_wire::Request {
                actor,
                sequence,
                command,
            }
            .encode()?
        };
        self.sequence = sequence;
        self.pending = Some((sequence, command, now));
        Ok(Some(bytes))
    }
}
pub(super) struct Peer {
    pub(super) port: Port,
    sequence: u64,
    pending: Option<u64>,
    sent: Option<crate::inventory_wire::State>,
    rewards_sent: Option<crate::reward_wire::State>,
    local_reply: Option<crate::inventory_wire::Reply>,
}
pub(super) enum View {
    Loading,
    Closed,
    State(Box<crate::inventory_wire::State>),
}
impl Peer {
    pub(super) fn new(port: Port) -> crate::Result<Self> {
        if port.hash == [0; 32] || port.requests.max_capacity() != 1 {
            return Err("Invalid inventory owner port capacity/profile".into());
        }
        if port
            .vendor
            .as_ref()
            .is_some_and(|p| p.item_hash() != port.hash)
        {
            return Err("Vendor/item owner profile mismatch".into());
        }
        if port
            .rewards
            .as_ref()
            .is_some_and(|r| r.item_hash() != port.hash)
        {
            return Err("Reward/item owner profile mismatch".into());
        }
        if port.rewards.is_some() != port.opportunities.is_some()
            || port
                .opportunities
                .as_ref()
                .is_some_and(|q| q.max_capacity() != theia_core::rewards::MAX_DEFINITIONS)
        {
            return Err("Invalid trusted reward port capacity/profile".into());
        }
        Ok(Self {
            port,
            sequence: 0,
            pending: None,
            sent: None,
            rewards_sent: None,
            local_reply: None,
        })
    }
    #[cfg(test)]
    pub(super) fn request(
        &mut self,
        actor: theia_core::identity::EntityKey,
        bytes: &[u8],
    ) -> crate::Result<bool> {
        self.request_checked(actor, bytes, None)
    }
    pub(super) fn request_in_world(
        &mut self,
        world: &crate::simulation::World,
        id: u64,
        bytes: &[u8],
    ) -> crate::Result<bool> {
        self.request_checked(
            world.entity_key(id).ok_or("Missing item actor")?,
            bytes,
            Some((world, id)),
        )
    }
    fn request_checked(
        &mut self,
        actor: theia_core::identity::EntityKey,
        bytes: &[u8],
        world: Option<(&crate::simulation::World, u64)>,
    ) -> crate::Result<bool> {
        let message = if bytes.starts_with(crate::vendor_wire::REQUEST_TAG) {
            self.port
                .vendor
                .as_ref()
                .ok_or("Vendor request requires enabled profile")?;
            let request = crate::vendor_wire::Request::decode(bytes)?;
            crate::inventory_wire::ItemRequest {
                actor: request.actor,
                sequence: request.sequence,
                command: request.command.into(),
            }
        } else if bytes.starts_with(crate::reward_wire::REQUEST_TAG) {
            self.port
                .rewards
                .as_ref()
                .ok_or("Reward request requires reward profile")?;
            let request = crate::reward_wire::Request::decode(bytes)?;
            crate::inventory_wire::ItemRequest {
                actor: request.actor,
                sequence: request.sequence,
                command: request.command.into(),
            }
        } else if self.port.catalog.equipment_enabled() {
            crate::inventory_wire::ItemRequest::decode(bytes)?
        } else {
            let old = crate::inventory_wire::Request::decode(bytes)?;
            crate::inventory_wire::ItemRequest {
                actor: old.actor,
                sequence: old.sequence,
                command: old.command.into(),
            }
        };
        if message.actor != actor || message.sequence.saturating_sub(self.sequence) > 128 {
            return Err("Foreign or discontinuous inventory command".into());
        }
        if message.sequence <= self.sequence {
            return Ok(false);
        }
        if self.pending.is_some() || self.port.state.borrow().bag.is_none() {
            return Err("Inventory intent before load or while another is pending".into());
        }
        if let ItemCommand::Vendor(command) = message.command
            && world.is_none_or(|(world, id)| !world.vendor_eligible(id, command))
        {
            self.sequence = message.sequence;
            self.pending = Some(message.sequence);
            self.local_reply = Some(crate::inventory_wire::Reply {
                sequence: message.sequence,
                outcome: crate::inventory_wire::Outcome::Rejected(
                    crate::inventory_wire::Reason::Conflict,
                ),
            });
            return Ok(true);
        }
        self.port
            .requests
            .try_send(Request {
                sequence: message.sequence,
                command: message.command,
            })
            .map_err(|_| "Inventory owner queue unavailable")?;
        self.sequence = message.sequence;
        self.pending = Some(message.sequence);
        Ok(true)
    }
    pub(super) fn rewards_view(
        &self,
        actor: theia_core::identity::EntityKey,
    ) -> crate::Result<Option<crate::reward_wire::State>> {
        let Some(profile) = &self.port.rewards else {
            return Ok(None);
        };
        let observation = self.port.state.borrow();
        if observation.failed || observation.closed {
            return Ok(None);
        }
        let Some(owned) = &observation.rewards else {
            return Ok(None);
        };
        let state = crate::reward_wire::State {
            actor,
            generation: owned.generation,
            profile_hash: profile.hash(),
            available: owned.available.clone(),
        };
        if let Some(old) = &self.rewards_sent {
            if state.generation < old.generation
                || (state.generation == old.generation && state != *old)
            {
                return Err("Reward owner generation regressed or aliased".into());
            }
            if state == *old {
                return Ok(None);
            }
        }
        Ok(Some(state))
    }
    pub(super) fn rewards_sent(&mut self, state: crate::reward_wire::State) {
        self.rewards_sent = Some(state);
    }
    pub(super) fn view(&self, actor: theia_core::identity::EntityKey) -> crate::Result<View> {
        if self.port.state.has_changed().is_err() {
            return Err("Inventory owner closed unexpectedly".into());
        }
        let observed = self.port.state.borrow().clone();
        if observed.failed {
            return Err("Inventory owner failed".into());
        }
        if observed.closed {
            return Ok(View::Closed);
        }
        let Some(bag) = observed.bag else {
            return Ok(View::Loading);
        };
        if observed.reply.is_some_and(|r| r.sequence > self.sequence) {
            return Err("Unsolicited inventory owner reply".into());
        }
        let reply = match (observed.reply, self.local_reply) {
            (Some(a), Some(b)) => Some(if a.sequence >= b.sequence { a } else { b }),
            (a, b) => a.or(b),
        };
        let state = crate::inventory_wire::State {
            actor,
            catalog_hash: self.port.hash,
            bag,
            reply,
        };
        if let Some(old) = &self.sent {
            let old_sequence = old.reply.map_or(0, |r| r.sequence);
            let sequence = state.reply.map_or(0, |r| r.sequence);
            if state.bag.revision() < old.bag.revision() || sequence < old_sequence {
                return Err("Inventory owner state regressed".into());
            }
            if *old == state {
                return Ok(View::Loading);
            }
        }
        Ok(View::State(Box::new(state)))
    }
    pub(super) fn sent(&mut self, state: crate::inventory_wire::State) {
        if state
            .reply
            .is_some_and(|r| Some(r.sequence) == self.pending)
        {
            self.pending = None;
        }
        self.sent = Some(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory_wire::{Outcome, Reply, State};
    use theia_core::{
        content::ContentId,
        identity::{EntityId, EntityKey, WorldEpoch},
        inventory::{Command, Operation, Stack},
    };
    const DATA: &[u8] = include_bytes!("../../../../content/gameplay/development-items.json");
    #[tokio::test]
    async fn vendor_refusals_and_commits_share_the_item_sequence_without_early_success() {
        use crate::{
            dialogue::{Action, Request as Talk},
            inventory_wire::{Outcome, Reason},
            simulation::World,
        };
        let profile = Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap()
        .with_vendor(include_bytes!(
            "../../../../content/gameplay/development-vendor.json"
        ))
        .unwrap();
        let vendor = Arc::new(profile.vendor().unwrap().clone());
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        let guide = world
            .spawn_development_guide(
                Arc::new(
                    crate::dialogue::Profile::from_json(include_bytes!(
                        "../../../../content/gameplay/development-dialogue.json"
                    ))
                    .unwrap(),
                ),
                [8062., 8000.],
            )
            .unwrap();
        world.enable_vendor(vendor.clone()).unwrap();
        let id = world.join().unwrap();
        let actor = world.entity_key(id).unwrap();
        assert!(world.nearby_guide(id).is_some());
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = Stack::new(
            profile.catalog(),
            ContentId::new("theia.dev.training-token").unwrap(),
            2,
        );
        let bag = Bag::from_slots(profile.catalog(), 0, slots).unwrap();
        let (requests, mut receiver) = mpsc::channel(1);
        let (publish, state) = watch::channel(Observation {
            bag: Some(bag.clone()),
            ..Default::default()
        });
        let mut peer = Peer::new(Port {
            vendor: Some(vendor.clone()),
            rewards: None,
            opportunities: None,
            catalog: profile.catalog.clone(),
            hash: profile.hash(),
            requests,
            state,
        })
        .unwrap();
        let (sender, commands) = mpsc::channel(1);
        let mut client = ClientState::new(Client { profile, commands }).unwrap();
        let View::State(initial) = peer.view(actor).unwrap() else {
            panic!()
        };
        client.receive(&encode(&client, &initial), actor).unwrap();
        peer.sent(*initial);
        let command = theia_core::vendors::Command {
            expected_revision: 0,
            dialogue_revision: 1,
            npc: guide.key,
            offer: 0,
        };
        sender.try_send(command.into()).unwrap();
        let bytes = client
            .next(actor, tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        assert!(peer.request_in_world(&world, id, &bytes).unwrap());
        assert!(
            receiver.try_recv().is_err(),
            "Closed-session intent never reaches the item owner"
        );
        let View::State(refusal) = peer.view(actor).unwrap() else {
            panic!()
        };
        assert_eq!(
            refusal.reply.unwrap().outcome,
            Outcome::Rejected(Reason::Conflict)
        );
        client.receive(&encode(&client, &refusal), actor).unwrap();
        peer.sent(*refusal);
        assert!(!client.pending());
        assert!(
            matches!(peer.view(actor).unwrap(), View::Loading),
            "An earlier owner observation cannot roll back the local refusal"
        );
        world
            .interact(
                id,
                &Talk {
                    sequence: 1,
                    expected_revision: 0,
                    action: Action::Open(guide.key),
                },
            )
            .unwrap();
        sender.try_send(command.into()).unwrap();
        let bytes = client
            .next(actor, tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        assert!(peer.request_in_world(&world, id, &bytes).unwrap());
        let accepted = receiver.try_recv().unwrap();
        assert_eq!(accepted.sequence, 2);
        assert_eq!(accepted.command, command.into());
        sender
            .try_send(
                Command {
                    expected_revision: 1,
                    operation: Operation::Move { from: 0, to: 2 },
                }
                .into(),
            )
            .unwrap();
        assert!(
            client
                .next(actor, tokio::time::Instant::now())
                .unwrap()
                .is_none()
        );
        world
            .interact(
                id,
                &Talk {
                    sequence: 2,
                    expected_revision: 1,
                    action: Action::Close,
                },
            )
            .unwrap();
        assert!(client.pending());
        assert_eq!(client.view.as_ref().unwrap().bag, bag);
        // This observation represents the async owner's durable reply. A pure
        // proposal here tests protocol ordering, not real persistence.
        let committed = vendor
            .catalog()
            .propose(
                &bag,
                client.config.profile.catalog(),
                &vendor.purchase(command).unwrap(),
            )
            .unwrap();
        publish.send_replace(Observation {
            bag: Some(committed.clone()),
            reply: Some(Reply {
                sequence: 2,
                outcome: Outcome::Applied,
            }),
            ..Default::default()
        });
        let View::State(applied) = peer.view(actor).unwrap() else {
            panic!()
        };
        client.receive(&encode(&client, &applied), actor).unwrap();
        peer.sent(*applied);
        assert!(!client.pending());
        assert_eq!(client.view.as_ref().unwrap().bag, committed);
        let bytes = client
            .next(actor, tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        assert!(peer.request_in_world(&world, id, &bytes).unwrap());
        assert_eq!(receiver.try_recv().unwrap().sequence, 3);
    }
    #[tokio::test]
    async fn rewards_share_item_ordering_but_offer_observations_do_not_acknowledge_items() {
        use theia_core::rewards::{ClaimCommand, ClaimId};
        let profile = Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap()
        .with_rewards(include_bytes!(
            "../../../../content/gameplay/development-rewards.json"
        ))
        .unwrap();
        let (sender, commands) = mpsc::channel(1);
        let initial = State {
            actor: actor(),
            catalog_hash: profile.hash(),
            bag: Bag::empty(profile.catalog()),
            reply: None,
        };
        let reward = profile.rewards().unwrap();
        let offered = crate::reward_wire::State {
            actor: actor(),
            generation: 2,
            profile_hash: reward.hash(),
            available: vec![crate::reward_wire::Available {
                reward: reward.catalog().definitions().next().unwrap().id().clone(),
                claim: ClaimId::new([17; 16]).unwrap(),
            }],
        };
        let encoded_offer = offered.encode(reward.catalog(), reward.hash()).unwrap();
        let mut client = ClientState::new(Client { profile, commands }).unwrap();
        client.receive(&encode(&client, &initial), actor()).unwrap();
        client.receive_rewards(&encoded_offer, actor()).unwrap();
        let claim = ClaimCommand {
            claim: offered.available[0].claim,
            expected_revision: 0,
        };
        sender.try_send(claim.into()).unwrap();
        let frame = client
            .next(actor(), tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        assert_eq!(
            crate::reward_wire::Request::decode(&frame).unwrap().command,
            claim
        );
        sender
            .try_send(
                Command {
                    expected_revision: 1,
                    operation: Operation::Move { from: 0, to: 1 },
                }
                .into(),
            )
            .unwrap();
        client.receive_rewards(&encoded_offer, actor()).unwrap();
        assert!(client.pending());
        assert_eq!(client.view.as_ref().unwrap().bag.revision(), 0);
        assert!(
            client
                .next(actor(), tokio::time::Instant::now())
                .unwrap()
                .is_none()
        );
        let bad = crate::reward_wire::State {
            generation: 1,
            ..offered.clone()
        };
        let reward = client.config.profile.rewards().unwrap();
        assert!(
            client
                .receive_rewards(
                    &bad.encode(reward.catalog(), reward.hash()).unwrap(),
                    actor()
                )
                .is_err()
        );
        let mut next = initial.clone();
        next.bag = Bag::from_parts(
            client.config.profile.catalog(),
            1,
            initial.bag.slots().clone(),
            None,
        )
        .unwrap();
        next.reply = Some(Reply {
            sequence: 1,
            outcome: Outcome::Applied,
        });
        client.receive(&encode(&client, &next), actor()).unwrap();
        assert!(!client.pending());
        assert_eq!(
            crate::inventory_wire::ItemRequest::decode(
                &client
                    .next(actor(), tokio::time::Instant::now())
                    .unwrap()
                    .unwrap()
            )
            .unwrap()
            .sequence,
            2
        );
        // Equal generations cannot alias different private observations.
        let changed = crate::reward_wire::State {
            available: vec![],
            ..offered
        };
        let reward = client.config.profile.rewards().unwrap();
        assert!(
            client
                .receive_rewards(
                    &changed.encode(reward.catalog(), reward.hash()).unwrap(),
                    actor()
                )
                .is_err()
        );
    }
    fn actor() -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(9).unwrap(),
        }
    }
    fn fixture() -> (ClientState, mpsc::Sender<ItemCommand>, State) {
        let profile = Profile::from_json(DATA).unwrap();
        let (sender, commands) = mpsc::channel(1);
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = Stack::new(
            profile.catalog(),
            ContentId::new("theia.dev.training-stone").unwrap(),
            13,
        );
        let view = State {
            actor: actor(),
            catalog_hash: profile.hash(),
            bag: Bag::from_slots(profile.catalog(), 0, slots).unwrap(),
            reply: None,
        };
        (
            ClientState::new(Client { profile, commands }).unwrap(),
            sender,
            view,
        )
    }
    fn intent() -> Command {
        Command {
            expected_revision: 0,
            operation: Operation::Move { from: 0, to: 1 },
        }
    }
    #[tokio::test]
    async fn equipment_and_backpack_share_one_pending_stream_with_profile_two_observations() {
        use theia_core::inventory::{EquipmentCommand, EquipmentOperation};
        let profile = Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let (sender, commands) = mpsc::channel(1);
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = Stack::new(
            profile.catalog(),
            ContentId::new("theia.dev.training-sword").unwrap(),
            1,
        );
        slots[1] = Stack::new(
            profile.catalog(),
            ContentId::new("theia.dev.training-stone").unwrap(),
            13,
        );
        let mut state = State {
            actor: actor(),
            catalog_hash: profile.hash(),
            bag: Bag::from_slots(profile.catalog(), 0, slots).unwrap(),
            reply: None,
        };
        let mut client = ClientState::new(Client { profile, commands }).unwrap();
        client.receive(&encode(&client, &state), actor()).unwrap();
        let first: ItemCommand = EquipmentCommand {
            expected_revision: 0,
            operation: EquipmentOperation::Equip { from: 0 },
        }
        .into();
        sender.try_send(first).unwrap();
        let frame = client
            .next(actor(), tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        let request = crate::inventory_wire::ItemRequest::decode(&frame).unwrap();
        assert_eq!(request.command, first);
        assert_eq!(request.sequence, 1);
        let second: ItemCommand = Command {
            expected_revision: 1,
            operation: Operation::Move { from: 1, to: 0 },
        }
        .into();
        sender.try_send(second).unwrap();
        assert!(
            client
                .next(actor(), tokio::time::Instant::now())
                .unwrap()
                .is_none()
        );
        state.bag = first
            .propose(&state.bag, client.config.profile.catalog())
            .unwrap();
        state.reply = Some(Reply {
            sequence: 1,
            outcome: Outcome::Applied,
        });
        client.receive(&encode(&client, &state), actor()).unwrap();
        let frame = client
            .next(actor(), tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        let request = crate::inventory_wire::ItemRequest::decode(&frame).unwrap();
        assert_eq!(request.command, second);
        assert_eq!(request.sequence, 2);
        state.bag = second
            .propose(&state.bag, client.config.profile.catalog())
            .unwrap();
        state.reply = Some(Reply {
            sequence: 2,
            outcome: Outcome::Applied,
        });
        client.receive(&encode(&client, &state), actor()).unwrap();
        assert!(!client.pending());
        assert_eq!(
            client
                .view
                .as_ref()
                .unwrap()
                .bag
                .main_hand()
                .unwrap()
                .item()
                .as_str(),
            "theia.dev.training-sword"
        );
        assert_eq!(
            client.view.as_ref().unwrap().bag.slots()[0]
                .as_ref()
                .unwrap()
                .count(),
            13
        );
        let public = crate::appearance_wire::State {
            epoch: actor().epoch.value(),
            tick: 2,
            catalog_hash: client.config.profile.hash(),
            actors: vec![crate::appearance_wire::Actor {
                key: actor(),
                main_hand: Some(ContentId::new("theia.dev.simple-sword").unwrap()),
            }],
        };
        let encode_public = |s: &crate::appearance_wire::State| {
            s.encode(
                client.config.profile.catalog(),
                client.config.profile.hash(),
            )
            .unwrap()
        };
        let valid_public = encode_public(&public);
        let mut old_public = public.clone();
        old_public.tick = 1;
        let old_public = encode_public(&old_public);
        let mut foreign_public = public.clone();
        foreign_public.epoch = 8;
        foreign_public.actors[0].key.epoch = WorldEpoch::new(8).unwrap();
        let foreign_public = encode_public(&foreign_public);
        let mut missing_owner = public.clone();
        missing_owner.actors.clear();
        let missing_owner = encode_public(&missing_owner);
        client.receive_appearance(&valid_public, actor()).unwrap();
        assert!(client.receive_appearance(&old_public, actor()).is_err());
        assert!(client.receive_appearance(&foreign_public, actor()).is_err());
        assert!(client.receive_appearance(&missing_owner, actor()).is_err());
        assert_eq!(client.appearance, Some(public));
        let started = tokio::time::Instant::now();
        assert!(
            client
                .check_deadline(started + std::time::Duration::from_secs(9), started)
                .is_ok()
        );
        client.appearance = None;
        assert!(
            client
                .check_deadline(started + std::time::Duration::from_secs(9), started)
                .is_err()
        );
        let (requests, _receiver) = mpsc::channel(1);
        let (_publish, observations) = watch::channel(Observation {
            bag: Some(state.bag.clone()),
            ..Default::default()
        });
        let mut peer = Peer::new(Port {
            vendor: None,
            rewards: None,
            opportunities: None,
            catalog: client.config.profile.catalog.clone(),
            hash: client.config.profile.hash(),
            requests,
            state: observations,
        })
        .unwrap();
        let wrong = crate::inventory_wire::Request {
            actor: actor(),
            sequence: 1,
            command: intent(),
        }
        .encode()
        .unwrap();
        assert!(
            peer.request(actor(), &wrong).is_err(),
            "Profile 2 cannot downgrade to IR01"
        );
    }
    fn encode(client: &ClientState, state: &State) -> Vec<u8> {
        state
            .encode(
                client.config.profile.catalog(),
                client.config.profile.hash(),
            )
            .unwrap()
    }
    fn applied(client: &ClientState, initial: &State) -> State {
        State {
            bag: initial
                .bag
                .propose(client.config.profile.catalog(), intent())
                .unwrap(),
            reply: Some(Reply {
                sequence: 1,
                outcome: Outcome::Applied,
            }),
            ..initial.clone()
        }
    }
    #[tokio::test]
    async fn client_waits_for_load_and_a_real_matching_reply_before_sending_another_intent() {
        let (mut client, sender, initial) = fixture();
        sender.try_send(intent().into()).unwrap();
        assert!(
            client
                .next(actor(), tokio::time::Instant::now())
                .unwrap()
                .is_none()
        );
        assert!(!client.pending());
        client.receive(&encode(&client, &initial), actor()).unwrap();
        let bytes = client
            .next(actor(), tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        assert_eq!(
            crate::inventory_wire::Request::decode(&bytes)
                .unwrap()
                .sequence,
            1
        );
        assert!(client.pending());
        // Even an accidentally queued second UI action cannot run concurrently.
        sender.try_send(intent().into()).unwrap();
        assert!(
            client
                .next(actor(), tokio::time::Instant::now())
                .unwrap()
                .is_none()
        );
        assert_eq!(client.config.commands.len(), 1);
        let success = applied(&client, &initial);
        client.receive(&encode(&client, &success), actor()).unwrap();
        assert!(!client.pending());
        assert_eq!(client.view.as_ref().unwrap().bag.revision(), 1);
        let bytes = client
            .next(actor(), tokio::time::Instant::now())
            .unwrap()
            .unwrap();
        assert_eq!(
            crate::inventory_wire::Request::decode(&bytes)
                .unwrap()
                .sequence,
            2
        );
    }
    #[tokio::test]
    async fn client_rejects_foreign_unsent_regressed_and_false_success_observations() {
        let (mut client, sender, initial) = fixture();
        let other = EntityKey {
            entity: EntityId::new(10).unwrap(),
            ..actor()
        };
        let mut foreign = initial.clone();
        foreign.actor = other;
        assert!(client.receive(&encode(&client, &foreign), actor()).is_err());
        let success = applied(&client, &initial);
        assert!(client.receive(&encode(&client, &success), actor()).is_err());
        client.receive(&encode(&client, &initial), actor()).unwrap();
        sender.try_send(intent().into()).unwrap();
        client.next(actor(), tokio::time::Instant::now()).unwrap();
        let mut false_success = success.clone();
        false_success.bag = Bag::from_slots(
            client.config.profile.catalog(),
            5,
            success.bag.slots().clone(),
        )
        .unwrap();
        assert!(
            client
                .receive(&encode(&client, &false_success), actor())
                .is_err()
        );
        assert!(
            client.pending(),
            "Invalid acknowledgement cannot clear pending work"
        );
        client.receive(&encode(&client, &success), actor()).unwrap();
        let mut rollback = initial.clone();
        rollback.reply = Some(Reply {
            sequence: 1,
            outcome: Outcome::Rejected(crate::inventory_wire::Reason::Stale),
        });
        assert!(
            client
                .receive(&encode(&client, &rollback), actor())
                .is_err()
        );
        assert!(client.receive(&encode(&client, &initial), actor()).is_err());
        let mut unsent = success.clone();
        unsent.reply.as_mut().unwrap().sequence = 2;
        assert!(client.receive(&encode(&client, &unsent), actor()).is_err());
        let mut wrong_catalog = encode(&client, &success);
        wrong_catalog[20] ^= 1;
        assert!(client.receive(&wrong_catalog, actor()).is_err());
    }
    #[tokio::test]
    async fn peer_port_only_routes_its_actor_once_and_waits_until_an_observation_is_queued() {
        let (client, _, initial) = fixture();
        let (sender, mut requests) = mpsc::channel(1);
        let (publish, state) = watch::channel(Observation {
            bag: Some(initial.bag.clone()),
            ..Observation::default()
        });
        let mut peer = Peer::new(Port {
            vendor: None,
            rewards: None,
            opportunities: None,
            catalog: client.config.profile.catalog.clone(),
            hash: client.config.profile.hash,
            requests: sender,
            state,
        })
        .unwrap();
        let request = crate::inventory_wire::Request {
            actor: actor(),
            sequence: 1,
            command: intent(),
        };
        let bytes = request.encode().unwrap();
        assert!(peer.request(actor(), &bytes).unwrap());
        assert_eq!(requests.try_recv().unwrap().command, intent().into());
        assert!(!peer.request(actor(), &bytes).unwrap());
        assert!(requests.try_recv().is_err());
        let other = EntityKey {
            entity: EntityId::new(10).unwrap(),
            ..actor()
        };
        assert!(peer.request(other, &bytes).is_err());
        let next = crate::inventory_wire::Request {
            sequence: 2,
            ..request
        }
        .encode()
        .unwrap();
        assert!(peer.request(actor(), &next).is_err());
        let success = applied(&client, &initial);
        publish.send_replace(Observation {
            bag: Some(success.bag.clone()),
            reply: success.reply,
            ..Observation::default()
        });
        let View::State(view) = peer.view(actor()).unwrap() else {
            panic!("Missing queued reply")
        };
        assert!(
            peer.request(actor(), &next).is_err(),
            "Merely observing completion cannot skip reliable queueing"
        );
        peer.sent(*view);
        assert!(peer.request(actor(), &next).unwrap());
        publish.send_modify(|s| s.reply.as_mut().unwrap().sequence = 99);
        assert!(peer.view(actor()).is_err());
        publish.send_replace(Observation {
            closed: true,
            ..Observation::default()
        });
        assert!(matches!(peer.view(actor()).unwrap(), View::Closed));
        publish.send_modify(|s| s.failed = true);
        assert!(peer.view(actor()).is_err());
    }
    #[tokio::test]
    async fn inventory_queues_and_unacknowledged_work_have_explicit_bounds() {
        let (mut client, sender, initial) = fixture();
        let profile = client.config.profile.clone();
        let (_, commands) = mpsc::channel(2);
        assert!(ClientState::new(Client { profile, commands }).is_err());
        let now = tokio::time::Instant::now();
        let later = now + std::time::Duration::from_secs(9);
        assert!(client.check_deadline(later, now).is_err());
        client.receive(&encode(&client, &initial), actor()).unwrap();
        assert!(client.check_deadline(later, now).is_ok());
        sender.try_send(intent().into()).unwrap();
        client.next(actor(), now).unwrap();
        assert!(client.check_deadline(later, now).is_err());
        let (requests, _) = mpsc::channel(2);
        let (_, state) = watch::channel(Observation::default());
        assert!(
            Peer::new(Port {
                vendor: None,
                rewards: None,
                opportunities: None,
                catalog: client.config.profile.catalog.clone(),
                hash: client.config.profile.hash,
                requests,
                state
            })
            .is_err()
        );
    }
}
