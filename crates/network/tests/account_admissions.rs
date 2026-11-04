//! Real encrypted runtime with a deliberately synthetic admission policy. These
//! tests cover authority lifecycle hooks, not OIDC ownership or SQL durability.
#[path = "support/relay.rs"]
mod relay;
use std::{
    net::UdpSocket,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use theia_core::{
    coordinates::{ChunkId, GlobalPosition},
    identity::EntityKey,
};
use theia_network::{
    Result,
    client::State,
    procedural::Patch,
    renet_runtime::{self as net, AccountAdmissions, AdmissionLease, PlayerAdmission},
    simulation::World,
    wire::Intent,
};
use tokio::sync::watch;
#[derive(Default)]
struct Policy {
    live: AtomicBool,
    state: Mutex<Ledger>,
}
#[derive(Default)]
struct Ledger {
    code: u8,
    spent: bool,
    position: Option<GlobalPosition>,
    records: Vec<(EntityKey, GlobalPosition, u64)>,
    dropped: usize,
    inventory: Option<net::inventory::Port>,
}
struct Lease(Arc<Policy>);
impl AdmissionLease for Lease {
    fn inventory(&self) -> Result<Option<net::inventory::Port>> {
        Ok(self.0.state.lock().unwrap().inventory.clone())
    }
    fn live(&self) -> Result<bool> {
        Ok(self.0.live.load(Ordering::SeqCst))
    }
    fn record(&self, actor: EntityKey, position: GlobalPosition, tick: u64) -> Result<()> {
        let mut state = self.0.state.lock().unwrap();
        if let Some((old, _, previous)) = state.records.last() {
            assert!(*old == actor && *previous <= tick);
        }
        // Keep bounded samples; the latest position is the retirement contract.
        if state.records.len() == 128 {
            state.records.remove(0);
        }
        state.records.push((actor, position, tick));
        Ok(())
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.state.lock().unwrap().dropped += 1;
    }
}
impl AccountAdmissions for Policy {
    fn consume(self: Arc<Self>, client: u64, data: &[u8; 256]) -> Result<Option<PlayerAdmission>> {
        let mut state = self.state.lock().unwrap();
        if client != 1 || state.spent || state.code != data[0] || !self.live.load(Ordering::SeqCst)
        {
            return Ok(None);
        }
        state.spent = true;
        let position = state.position.unwrap();
        drop(state);
        Ok(Some(PlayerAdmission {
            position,
            lease: Box::new(Lease(self)),
        }))
    }
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; 8]> {
        let mut result = [None; 8];
        result[0] = self.state.lock().unwrap().position;
        Ok(result)
    }
}
struct Client {
    casts: tokio::sync::mpsc::Sender<net::CastChoice>,
    combat_events: tokio::sync::mpsc::Receiver<theia_network::combat_wire::Batch>,
    state: watch::Receiver<State>,
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<Result<()>>,
    _input: watch::Sender<Intent>,
    commands: tokio::sync::mpsc::Sender<theia_network::prediction_wire::Commands>,
    items: Option<tokio::sync::mpsc::Sender<theia_core::inventory::ItemCommand>>,
    dialogue: Option<tokio::sync::mpsc::Sender<net::dialogue::Intent>>,
}
impl Client {
    fn start(address: std::net::SocketAddr, ticket: Vec<u8>, world: [u8; 32]) -> Self {
        Self::start_profile(address, ticket, world, None)
    }
    fn start_profile(
        address: std::net::SocketAddr,
        ticket: Vec<u8>,
        world: [u8; 32],
        profile: Option<net::inventory::Profile>,
    ) -> Self {
        Self::start_profiles(address, ticket, world, profile, None)
    }
    fn start_profiles(
        address: std::net::SocketAddr,
        ticket: Vec<u8>,
        world: [u8; 32],
        profile: Option<net::inventory::Profile>,
        combat_catalog: Option<Arc<theia_core::abilities::Catalog>>,
    ) -> Self {
        let (input, inputs) = watch::channel(Intent::default());
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let (ready, readiness) = watch::channel(Some(
            (119..=121)
                .flat_map(|x| (99..=101).map(move |z| ChunkId { x, z }))
                .collect(),
        ));
        let (commands, packets) = tokio::sync::mpsc::channel(1);
        let (items, item_requests) = tokio::sync::mpsc::channel(1);
        let inventory_enabled = profile.is_some();
        let vendor_enabled = profile.as_ref().is_some_and(|p| p.vendor().is_some());
        let (dialogue, dialogue_actions) = tokio::sync::mpsc::channel(4);
        let (casts, cast_requests) = tokio::sync::mpsc::channel(1);
        let (events, combat_events) = tokio::sync::mpsc::channel(16);
        let (choice, choices) = watch::channel(None);
        let task = tokio::spawn(async move {
            let _keep = (ready, choice);
            let combat_enabled = combat_catalog.is_some();
            if let Some(profile) = profile {
                return net::run_client_gameplay(
                    address,
                    &ticket,
                    inputs,
                    states,
                    shutdown,
                    world,
                    net::GameplayClient {
                        avatar_life: None,
                        public_attacks: None,
                        journal: None,
                        classic_source: None,
                        dodge: None,
                        player_life: None,
                        lifecycle: None,
                        dialogue: if vendor_enabled {
                            Some(net::dialogue::Client {
                                profile: Arc::new(
                                    theia_network::dialogue::Profile::from_json(include_bytes!(
                                        "../../../content/gameplay/development-dialogue.json"
                                    ))
                                    .unwrap(),
                                ),
                                actions: dialogue_actions,
                            })
                        } else {
                            None
                        },
                        quest: None,
                        progression: None,
                        ready: readiness,
                        commands: Some(packets),
                        choices: combat_enabled.then_some(choices),
                        combat: combat_catalog.map(|catalog| net::CombatActions {
                            melee: None,
                            catalog,
                            casts: cast_requests,
                            events,
                        }),
                        statuses: combat_enabled.then(|| {
                            Arc::new(
                                theia_core::statuses::Catalog::from_json(include_bytes!(
                                    "../../../content/gameplay/development-statuses.json"
                                ))
                                .unwrap(),
                            )
                        }),
                        inventory: Some(net::inventory::Client {
                            profile,
                            commands: item_requests,
                        }),
                    },
                )
                .await;
            }
            net::run_client_predicted(
                address, &ticket, inputs, states, shutdown, world, readiness, packets,
            )
            .await
        });
        Self {
            casts,
            combat_events,
            state,
            stop,
            task,
            _input: input,
            commands,
            items: inventory_enabled.then_some(items),
            dialogue: vendor_enabled.then_some(dialogue),
        }
    }
    async fn admitted(&mut self) -> State {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let value = self.state.borrow().clone();
                if value.id.is_some() {
                    return value;
                }
                self.state.changed().await.unwrap();
            }
        })
        .await
        .unwrap()
    }
    async fn move_until_recorded(&self, policy: &Policy, start_x: f64) -> u64 {
        // Refresh bounded, ordered input while waiting for authoritative movement.
        // A single command two ticks ahead can expire during relay/scheduler delay.
        tokio::time::timeout(Duration::from_secs(2), async {
            let mut last_tick = 0;
            let mut sent_commands = 0;
            loop {
                if policy
                    .state
                    .lock()
                    .unwrap()
                    .records
                    .last()
                    .is_some_and(|(_, position, _)| position.metres()[0] > start_x + 0.01)
                {
                    break sent_commands;
                }
                let state = self.state.borrow().clone();
                let correction = state.correction.unwrap();
                let tick = correction.state.input.tick + theia_network::commands::FUTURE_TICKS - 2;
                if tick > last_tick {
                    sent_commands += 1;
                    self.commands
                        .send(theia_network::prediction_wire::Commands {
                            epoch: state.welcome.as_ref().unwrap().epoch,
                            stream_epoch: correction.state.input.stream_epoch,
                            ready_sequence: 1,
                            commands: vec![theia_network::commands::Command {
                                sequence: sent_commands,
                                tick,
                                intent: Intent {
                                    facing: None,
                                    axes: [1000, 0],
                                    sprint: false,
                                    jump: false,
                                    dodge: None,
                                },
                            }],
                        })
                        .await
                        .unwrap();
                    last_tick = tick;
                }
                tokio::time::sleep(Duration::from_millis(15)).await;
            }
        })
        .await
        .expect("Fresh commands must produce recorded authoritative movement")
    }
    async fn close(self) {
        self.stop.send_replace(true);
        let _ = self.task.await.unwrap();
    }
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let state = self.state.borrow().clone();
                if predicate(&state) {
                    return state;
                }
                self.state.changed().await.unwrap();
            }
        })
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Encrypted gameplay observation timed out: {error}; combat={:?}",
                self.state.borrow().combat
            )
        })
    }
}
struct Routing([Arc<Policy>; 2]);
impl AccountAdmissions for Routing {
    fn consume(self: Arc<Self>, client: u64, data: &[u8; 256]) -> Result<Option<PlayerAdmission>> {
        if !(1..=2).contains(&client) {
            return Ok(None);
        }
        self.0[client as usize - 1].clone().consume(1, data)
    }
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; 8]> {
        let mut positions = [None; 8];
        for (i, policy) in self.0.iter().enumerate() {
            positions[i] = policy.state.lock().unwrap().position;
        }
        Ok(positions)
    }
}
/// Synthetic item owner for routing/latency contracts only. It deliberately does
/// no SQL; Applied below is a crafted protocol response, not durability evidence.
fn synthetic_items(
    count: u32,
    released: watch::Receiver<bool>,
    stop: watch::Receiver<bool>,
) -> (
    net::inventory::Port,
    tokio::task::JoinHandle<()>,
    Arc<std::sync::atomic::AtomicU64>,
) {
    synthetic_items_with_profile(count, released, stop, false)
}
fn synthetic_items_with_profile(
    count: u32,
    released: watch::Receiver<bool>,
    stop: watch::Receiver<bool>,
    equipment: bool,
) -> (
    net::inventory::Port,
    tokio::task::JoinHandle<()>,
    Arc<std::sync::atomic::AtomicU64>,
) {
    synthetic_items_with_options(count, released, stop, equipment, false)
}
fn synthetic_items_with_options(
    count: u32,
    mut released: watch::Receiver<bool>,
    mut stop: watch::Receiver<bool>,
    equipment: bool,
    coalesced: bool,
) -> (
    net::inventory::Port,
    tokio::task::JoinHandle<()>,
    Arc<std::sync::atomic::AtomicU64>,
) {
    use theia_core::{
        content::ContentId,
        inventory::{Bag, Stack},
    };
    let bytes: &[u8] = if equipment {
        include_bytes!("../../../content/gameplay/development-equipment-items.json")
    } else {
        include_bytes!("../../../content/gameplay/development-items.json")
    };
    let profile = net::inventory::Profile::from_json(bytes).unwrap();
    let catalog = Arc::new(profile.catalog().clone());
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(
        &catalog,
        ContentId::new("theia.dev.training-stone").unwrap(),
        count,
    );
    if equipment {
        for (slot, name) in [(1, "sword"), (2, "wand")] {
            slots[slot] = Stack::new(
                &catalog,
                ContentId::new(&format!("theia.dev.training-{name}")).unwrap(),
                1,
            );
        }
    }
    let mut bag = Bag::from_slots(&catalog, 0, slots).unwrap();
    let (requests, mut commands) = tokio::sync::mpsc::channel::<net::inventory::Request>(1);
    let (publish, state) = watch::channel(net::inventory::Observation {
        bag: Some(bag.clone()),
        ..Default::default()
    });
    let port = net::inventory::Port {
        vendor: None,
        rewards: None,
        opportunities: None,
        catalog: catalog.clone(),
        hash: profile.hash(),
        requests,
        state,
    };
    let received = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let observed = received.clone();
    let task = tokio::spawn(async move {
        loop {
            let command = tokio::select! {_=stop.changed()=>break,command=commands.recv()=>match command {Some(c)=>c,None=>break}};
            observed.fetch_add(1, Ordering::SeqCst);
            while !*released.borrow() {
                tokio::select! {_=stop.changed()=>return,_=released.changed()=>{}};
            }
            let outcome = match command.command.propose(&bag, &catalog) {
                Ok(next) => {
                    bag = next;
                    if coalesced {
                        let from = bag.slots().iter().position(Option::is_some).unwrap() as u8;
                        bag = bag
                            .propose(
                                &catalog,
                                theia_core::inventory::Command {
                                    expected_revision: bag.revision(),
                                    operation: theia_core::inventory::Operation::Destroy {
                                        from,
                                        amount: 1,
                                    },
                                },
                            )
                            .unwrap();
                    }
                    theia_network::inventory_wire::Outcome::Applied
                }
                Err(theia_core::inventory::Error::StaleRevision) => {
                    theia_network::inventory_wire::Outcome::Rejected(
                        theia_network::inventory_wire::Reason::Stale,
                    )
                }
                Err(theia_core::inventory::Error::NotEquippable) => {
                    theia_network::inventory_wire::Outcome::Rejected(
                        theia_network::inventory_wire::Reason::NotEquippable,
                    )
                }
                Err(_) => panic!("Unexpected synthetic proposal"),
            };
            publish.send_replace(net::inventory::Observation {
                bag: Some(bag.clone()),
                reply: Some(theia_network::inventory_wire::Reply {
                    sequence: command.sequence,
                    outcome,
                }),
                ..Default::default()
            });
        }
    });
    (port, task, received)
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_item_ports_keep_two_actors_separate_and_do_not_block_movement_while_pending() {
    encrypted_item_ports(false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_coalesced_item_reply_preserves_later_removal_privacy_and_movement() {
    encrypted_item_ports(true).await;
}
async fn encrypted_item_ports(coalesced: bool) {
    use theia_core::inventory::{Command, Operation};
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let identity = world.identity();
    let patch = Patch::build(ChunkId { x: 120, z: 100 });
    let (stop, shutdown) = watch::channel(false);
    let (release_a, waiting) = watch::channel(false);
    let (_release_b, ready) = watch::channel(true);
    let (items_a, task_a, received_a) =
        synthetic_items_with_options(13, waiting, shutdown.clone(), false, coalesced);
    let (items_b, task_b, _) = synthetic_items(7, ready, shutdown.clone());
    let policies: [Arc<Policy>; 2] = std::array::from_fn(|i| {
        let policy = Arc::new(Policy::default());
        policy.live.store(true, Ordering::SeqCst);
        let mut body = patch
            .try_spawn(15400. + 4. * i as f64, 12860.)
            .unwrap()
            .metres();
        body[1] += f64::from(theia_network::world::PAWN_Y);
        {
            let mut state = policy.state.lock().unwrap();
            state.code = i as u8 + 1;
            state.position = GlobalPosition::new(body);
            state.inventory = Some(if i == 0 {
                items_a.clone()
            } else {
                items_b.clone()
            });
        }
        policy
    });
    let authority = Arc::new(Routing(policies.clone()));
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let server = tokio::spawn(net::run_server_with_admissions(
        transport, shutdown, world, authority,
    ));
    let profile = net::inventory::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-items.json"
    ))
    .unwrap();
    let mut a = Client::start_profile(
        address,
        issuer
            .issue(1, Some(&[1; 256]), 30)
            .unwrap()
            .bytes()
            .to_vec(),
        identity,
        Some(profile.clone()),
    );
    let mut b = Client::start_profile(
        address,
        issuer
            .issue(2, Some(&[2; 256]), 30)
            .unwrap()
            .bytes()
            .to_vec(),
        identity,
        Some(profile),
    );
    a.until(|s| s.inventory.is_some() && s.correction.is_some() && s.chunk_ack.is_some())
        .await;
    b.until(|s| s.inventory.is_some() && s.correction.is_some() && s.chunk_ack.is_some())
        .await;
    a.items
        .as_ref()
        .unwrap()
        .send(
            Command {
                expected_revision: 0,
                operation: Operation::Move { from: 0, to: 3 },
            }
            .into(),
        )
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while received_a.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    b.items
        .as_ref()
        .unwrap()
        .send(
            Command {
                expected_revision: 0,
                operation: Operation::Split {
                    from: 0,
                    to: 4,
                    amount: 2,
                },
            }
            .into(),
        )
        .await
        .unwrap();
    let done = b
        .until(|s| s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 1))
        .await;
    assert_eq!(
        done.inventory.as_ref().unwrap().bag.slots()[4]
            .as_ref()
            .unwrap()
            .count(),
        2
    );
    assert_eq!(
        done.inventory.as_ref().unwrap().bag.slots()[0]
            .as_ref()
            .unwrap()
            .count(),
        5
    );
    assert_eq!(
        a.state.borrow().inventory.as_ref().unwrap().bag.revision(),
        0
    );
    assert!(a.state.borrow().inventory_pending);
    let x = policies[0].state.lock().unwrap().position.unwrap().metres()[0];
    a.move_until_recorded(&policies[0], x).await;
    release_a.send_replace(true);
    let done = a
        .until(|s| {
            s.inventory
                .as_ref()
                .is_some_and(|i| i.bag.revision() == if coalesced { 2 } else { 1 })
                && !s.inventory_pending
        })
        .await;
    assert_eq!(
        done.inventory.as_ref().unwrap().bag.slots()[3]
            .as_ref()
            .unwrap()
            .count(),
        if coalesced { 12 } else { 13 }
    );
    assert!(done.inventory.as_ref().unwrap().bag.slots()[0].is_none());
    a.items
        .as_ref()
        .unwrap()
        .send(
            Command {
                expected_revision: 0,
                operation: Operation::Move { from: 0, to: 3 },
            }
            .into(),
        )
        .await
        .unwrap();
    let rejected = a
        .until(|s| {
            s.inventory
                .as_ref()
                .and_then(|i| i.reply)
                .is_some_and(|r| r.sequence == 2)
        })
        .await;
    assert_eq!(
        rejected.inventory.unwrap().reply.unwrap().outcome,
        theia_network::inventory_wire::Outcome::Rejected(
            theia_network::inventory_wire::Reason::Stale
        )
    );
    assert_eq!(
        b.state.borrow().inventory.as_ref().unwrap().bag.slots()[0]
            .as_ref()
            .unwrap()
            .count(),
        5
    );
    a.close().await;
    b.close().await;
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert_eq!(stats.max_players, 2);
    assert_eq!(stats.inventory_requests_forwarded, 3);
    assert!(stats.inventory_states_sent >= 5);
    assert!(stats.queued_commands > 0);
    assert!(stats.max_control_batch <= 8 && stats.max_event_queue_bytes <= 8192);
    task_a.await.unwrap();
    task_b.await.unwrap();
}
#[tokio::test]
async fn saved_collision_revocation_replay_reuse_and_shutdown_keep_exact_lease_lifecycle() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let identity = world.identity();
    let patch = Patch::build(ChunkId { x: 120, z: 100 });
    let mut body = patch.try_spawn(15400., 12860.).unwrap().metres();
    body[1] += f64::from(theia_network::world::PAWN_Y);
    let position = GlobalPosition::new(body).unwrap();
    assert!(!world.terrain_prepared_for(position));
    let policy = Arc::new(Policy::default());
    policy.live.store(true, Ordering::SeqCst);
    {
        let mut state = policy.state.lock().unwrap();
        state.code = 1;
        state.position = Some(position);
    }
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let backend = socket.local_addr().unwrap();
    let relay = relay::Relay::bind(relay::Profile::BASELINE);
    let address = relay.address;
    let relay_control = relay.control.clone();
    let (relay_stop, relay_shutdown) = watch::channel(false);
    let relay_task = tokio::spawn(relay.run(backend, relay_shutdown));
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let authority = policy.clone();
    let server = tokio::spawn(net::run_server_with_admissions(
        transport, shutdown, world, authority,
    ));
    let first = issuer
        .issue(1, Some(&[1; 256]), 30)
        .unwrap()
        .bytes()
        .to_vec();
    let mut client = Client::start(address, first.clone(), identity);
    let initial = client.admitted().await;
    assert_eq!(initial.id, Some(1));
    {
        let state = policy.state.lock().unwrap();
        assert!(state.spent);
        assert_eq!(state.records[0].1, position);
    }
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        policy.state.lock().unwrap().dropped,
        0,
        "A live idle session must remain admitted"
    );
    let sent_commands = client
        .move_until_recorded(&policy, position.metres()[0])
        .await;
    let old_packets: Vec<_> = relay_control
        .lock()
        .unwrap()
        .captures
        .iter()
        .cloned()
        .collect();
    assert!(!old_packets.is_empty());
    policy.live.store(false, Ordering::SeqCst);
    tokio::time::timeout(Duration::from_secs(2), async {
        while policy.state.lock().unwrap().dropped != 1 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    client.close().await;
    // Reuse is a synthetic policy decision here; real durable release is tested
    // with the server's account gate and owned PostgreSQL separately.
    {
        let mut state = policy.state.lock().unwrap();
        state.code = 2;
        state.spent = false;
        state.records.clear();
    }
    policy.live.store(true, Ordering::SeqCst);
    let replay = Client::start(address, first, identity);
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(!policy.state.lock().unwrap().spent);
    assert!(replay.state.borrow().id.is_none());
    replay.close().await;
    let second = issuer
        .issue(1, Some(&[2; 256]), 30)
        .unwrap()
        .bytes()
        .to_vec();
    let mut replacement = Client::start(address, second, identity);
    let fresh = replacement.admitted().await;
    assert_eq!(fresh.id, Some(2));
    tokio::time::sleep(Duration::from_millis(150)).await;
    let before = replacement.state.borrow().correction.unwrap().state.input;
    assert_eq!(before.applied_sequence, 0);
    {
        let mut relay = relay_control.lock().unwrap();
        relay.stats.injected_old_packets += old_packets.len() as u64;
        relay.inject.extend(old_packets);
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    let after = replacement.state.borrow().correction.unwrap().state.input;
    assert_eq!(after.stream_epoch, 1);
    assert_eq!(
        after.applied_sequence, 0,
        "Old encrypted command packets must not control the replacement actor"
    );
    assert_eq!(
        policy
            .state
            .lock()
            .unwrap()
            .records
            .last()
            .unwrap()
            .1
            .metres()[0],
        position.metres()[0]
    );
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert_eq!(stats.persistent_spawn_admissions, 2);
    assert_eq!(
        policy.state.lock().unwrap().dropped,
        2,
        "Shutdown drops the joined lease before returning"
    );
    assert!((1..=sent_commands).contains(&stats.queued_commands));
    replacement.close().await;
    relay_stop.send_replace(true);
    relay_task.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_equipment_streams_keep_two_actors_separate_under_impaired_udp() {
    use theia_core::inventory::{Command, EquipmentCommand, EquipmentOperation as Op, Operation};
    use theia_network::inventory_wire::{Outcome, Reason};
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let identity = world.identity();
    fn has_appearance(s: &State, actor: u64, hand: Option<&str>) -> bool {
        s.appearance.as_ref().is_some_and(|p| {
            p.actors.iter().any(|a| {
                a.key.entity.value() == actor && a.main_hand.as_ref().map(|id| id.as_str()) == hand
            })
        })
    }
    let patch = Patch::build(ChunkId { x: 120, z: 100 });
    let (stop, shutdown) = watch::channel(false);
    let (release_a, waiting) = watch::channel(false);
    let (_release_b, ready) = watch::channel(true);
    let (items_a, task_a, received_a) =
        synthetic_items_with_profile(13, waiting, shutdown.clone(), true);
    let (items_b, task_b, _) = synthetic_items_with_profile(7, ready, shutdown.clone(), true);
    let policies: [Arc<Policy>; 2] = std::array::from_fn(|i| {
        let policy = Arc::new(Policy::default());
        policy.live.store(true, Ordering::SeqCst);
        let mut body = patch
            .try_spawn(15400. + 4. * i as f64, 12860.)
            .unwrap()
            .metres();
        body[1] += f64::from(theia_network::world::PAWN_Y);
        {
            let mut state = policy.state.lock().unwrap();
            state.code = i as u8 + 1;
            state.position = GlobalPosition::new(body);
            state.inventory = Some(if i == 0 {
                items_a.clone()
            } else {
                items_b.clone()
            });
        }
        policy
    });
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let backend = socket.local_addr().unwrap();
    let mut relay_addresses = Vec::new();
    let mut relay_tasks = Vec::new();
    let mut controls = Vec::new();
    let (relay_stop, relay_shutdown) = watch::channel(false);
    for _ in 0..2 {
        let relay = relay::Relay::bind(relay::Profile {
            name: "equipment-impaired",
            delay_ms: 20,
            jitter_ms: 30,
            loss_every: 7,
            duplicate_every: 9,
        });
        relay_addresses.push(relay.address);
        controls.push(relay.control.clone());
        relay_tasks.push(tokio::spawn(relay.run(backend, relay_shutdown.clone())));
    }
    let (transport, issuer) = net::secure_issuer(socket, relay_addresses.clone()).unwrap();
    let server = tokio::spawn(net::run_server_with_admissions(
        transport,
        shutdown,
        world,
        Arc::new(Routing(policies)),
    ));
    let profile = net::inventory::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let mut a = Client::start_profile(
        relay_addresses[0],
        issuer
            .issue(1, Some(&[1; 256]), 30)
            .unwrap()
            .bytes()
            .to_vec(),
        identity,
        Some(profile.clone()),
    );
    let mut b = Client::start_profile(
        relay_addresses[1],
        issuer
            .issue(2, Some(&[2; 256]), 30)
            .unwrap()
            .bytes()
            .to_vec(),
        identity,
        Some(profile),
    );
    a.until(|s| s.inventory.is_some() && s.chunk_ack.is_some())
        .await;
    b.until(|s| s.inventory.is_some() && s.chunk_ack.is_some())
        .await;
    a.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 0,
                operation: Op::Equip { from: 1 },
            }
            .into(),
        )
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while received_a.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    b.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 0,
                operation: Op::Equip { from: 2 },
            }
            .into(),
        )
        .await
        .unwrap();
    let other = b
        .until(|s| s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 1))
        .await;
    assert_eq!(
        other
            .inventory
            .as_ref()
            .unwrap()
            .bag
            .main_hand()
            .unwrap()
            .item()
            .as_str(),
        "theia.dev.training-wand"
    );
    assert_eq!(
        other.inventory.as_ref().unwrap().bag.slots()[0]
            .as_ref()
            .unwrap()
            .count(),
        7
    );
    assert_eq!(
        a.state.borrow().inventory.as_ref().unwrap().bag.revision(),
        0,
        "Pending response cannot invent equipped state"
    );
    let a_id = a.state.borrow().id.unwrap();
    let b_id = other.id.unwrap();
    a.until(|s| has_appearance(s, b_id, Some("theia.dev.simple-wand")))
        .await;
    assert!(has_appearance(&a.state.borrow(), a_id, None));
    release_a.send_replace(true);
    let equipped = a
        .until(|s| s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 1))
        .await;
    assert_eq!(
        equipped
            .inventory
            .as_ref()
            .unwrap()
            .bag
            .main_hand()
            .unwrap()
            .item()
            .as_str(),
        "theia.dev.training-sword"
    );
    let public_before_move = b
        .until(|s| has_appearance(s, a_id, Some("theia.dev.simple-sword")))
        .await
        .appearance
        .unwrap();
    a.items
        .as_ref()
        .unwrap()
        .send(
            Command {
                expected_revision: 1,
                operation: Operation::Move { from: 0, to: 3 },
            }
            .into(),
        )
        .await
        .unwrap();
    let moved = a
        .until(|s| s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 2))
        .await;
    assert_eq!(
        moved.inventory.as_ref().unwrap().bag.main_hand(),
        equipped.inventory.as_ref().unwrap().bag.main_hand()
    );
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(
        b.state.borrow().appearance.as_ref(),
        Some(&public_before_move),
        "Private backpack changes must not publish appearance updates"
    );
    a.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 2,
                operation: Op::Equip { from: 3 },
            }
            .into(),
        )
        .await
        .unwrap();
    let rejected = a
        .until(|s| {
            s.inventory
                .as_ref()
                .and_then(|i| i.reply)
                .is_some_and(|r| r.sequence == 3)
        })
        .await;
    assert_eq!(
        rejected.inventory.as_ref().unwrap().reply.unwrap().outcome,
        Outcome::Rejected(Reason::NotEquippable)
    );
    assert_eq!(
        rejected.inventory.as_ref().unwrap().bag,
        moved.inventory.as_ref().unwrap().bag
    );
    a.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 2,
                operation: Op::Unequip { to: 1 },
            }
            .into(),
        )
        .await
        .unwrap();
    let returned = a
        .until(|s| s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 3))
        .await;
    assert!(
        returned
            .inventory
            .as_ref()
            .unwrap()
            .bag
            .main_hand()
            .is_none()
    );
    assert_eq!(
        returned.inventory.as_ref().unwrap().bag.slots()[1]
            .as_ref()
            .unwrap()
            .item()
            .as_str(),
        "theia.dev.training-sword"
    );
    assert_eq!(
        b.state.borrow().inventory.as_ref().unwrap().bag,
        other.inventory.as_ref().unwrap().bag,
        "Another peer's item state cannot overwrite this character"
    );
    b.until(|s| has_appearance(s, a_id, None)).await;
    a.close().await;
    b.until(|s| {
        s.appearance
            .as_ref()
            .is_some_and(|p| p.actors.iter().all(|a| a.key.entity.value() != a_id))
    })
    .await;
    assert!(has_appearance(
        &b.state.borrow(),
        b_id,
        Some("theia.dev.simple-wand")
    ));
    b.close().await;
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert_eq!(stats.inventory_requests_forwarded, 5);
    assert!(stats.appearance_states_sent >= 4);
    assert_eq!(stats.max_players, 2);
    assert!(stats.max_event_queue_bytes <= 8192);
    task_a.await.unwrap();
    task_b.await.unwrap();
    relay_stop.send_replace(true);
    for task in relay_tasks {
        task.await.unwrap();
    }
    for control in controls {
        let stats = &control.lock().unwrap().stats;
        assert!(
            stats.configured_drops > 0
                && stats.duplicates > 0
                && stats.forwarded_packets.iter().sum::<u64>() > 0
        );
        assert!(stats.queue_high_water <= relay::QUEUE_LIMIT && stats.overflow == 0);
    }
}

// Synthetic item owner, actual encrypted transport and live world permissions.
// Real SQL/logout durability is qualified separately by the admission lease probe.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_vendor_rejects_closed_session_then_commits_accepted_trade_after_close() {
    use theia_core::{
        content::ContentId,
        inventory::{Bag, Command, Operation, Stack},
        vendors,
    };
    use theia_network::{
        dialogue::{Action, Profile},
        inventory_wire::{Outcome, Reason, Reply},
    };
    let profile = net::inventory::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap()
    .with_vendor(include_bytes!(
        "../../../content/gameplay/development-vendor.json"
    ))
    .unwrap();
    let vendor = Arc::new(profile.vendor().unwrap().clone());
    let mut world = World::with_procedural([15400., 12860.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let guide = world
        .spawn_development_guide(
            Arc::new(
                Profile::from_json(include_bytes!(
                    "../../../content/gameplay/development-dialogue.json"
                ))
                .unwrap(),
            ),
            [15402., 12860.],
        )
        .unwrap();
    world.enable_vendor(vendor.clone()).unwrap();
    let identity = world.identity();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(
        profile.catalog(),
        ContentId::new("theia.dev.training-token").unwrap(),
        2,
    );
    let bag = Bag::from_slots(profile.catalog(), 0, slots).unwrap();
    let (requests, mut receiver) = tokio::sync::mpsc::channel::<net::inventory::Request>(1);
    let (publish, state) = watch::channel(net::inventory::Observation {
        bag: Some(bag.clone()),
        ..Default::default()
    });
    let policy = Arc::new(Policy::default());
    policy.live.store(true, Ordering::SeqCst);
    {
        let mut ledger = policy.state.lock().unwrap();
        ledger.code = 1;
        ledger.position = world.development_player_spawn(1);
        ledger.inventory = Some(net::inventory::Port {
            vendor: Some(vendor.clone()),
            rewards: None,
            opportunities: None,
            catalog: Arc::new(profile.catalog().clone()),
            hash: profile.hash(),
            requests,
            state,
        });
    }
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(net::run_server_with_admissions(
        transport, shutdown, world, policy,
    ));
    let ticket = issuer
        .issue(1, Some(&[1; 256]), 30)
        .unwrap()
        .bytes()
        .to_vec();
    let mut client = Client::start_profile(address, ticket, identity, Some(profile.clone()));
    client
        .until(|s| s.inventory.is_some() && s.dialogue.is_some() && s.guide.is_some())
        .await;
    let command = vendors::Command {
        expected_revision: 0,
        dialogue_revision: 1,
        npc: guide.key,
        offer: 0,
    };
    client
        .items
        .as_ref()
        .unwrap()
        .send(command.into())
        .await
        .unwrap();
    let refused = client
        .until(|s| {
            s.inventory
                .as_ref()
                .and_then(|i| i.reply)
                .is_some_and(|r| r.sequence == 1)
                && !s.inventory_pending
        })
        .await;
    assert_eq!(
        refused.inventory.unwrap().reply.unwrap().outcome,
        Outcome::Rejected(Reason::Conflict)
    );
    assert!(receiver.try_recv().is_err());
    client
        .dialogue
        .as_ref()
        .unwrap()
        .send(net::dialogue::Intent {
            expected_revision: 0,
            action: Action::Open(guide.key),
        })
        .await
        .unwrap();
    client
        .until(|s| {
            s.dialogue
                .as_ref()
                .is_some_and(|v| v.revision == 1 && v.open.is_some())
                && !s.dialogue_pending
        })
        .await;
    client
        .items
        .as_ref()
        .unwrap()
        .send(command.into())
        .await
        .unwrap();
    let accepted = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(accepted.sequence, 2);
    assert_eq!(accepted.command, command.into());
    client
        .dialogue
        .as_ref()
        .unwrap()
        .send(net::dialogue::Intent {
            expected_revision: 1,
            action: Action::Close,
        })
        .await
        .unwrap();
    let closed = client
        .until(|s| {
            s.dialogue
                .as_ref()
                .is_some_and(|v| v.revision == 2 && v.open.is_none())
                && !s.dialogue_pending
        })
        .await;
    assert!(closed.inventory_pending);
    assert_eq!(closed.inventory.as_ref().unwrap().bag, bag);
    let committed = vendor
        .catalog()
        .propose(&bag, profile.catalog(), &vendor.purchase(command).unwrap())
        .unwrap();
    publish.send_replace(net::inventory::Observation {
        bag: Some(committed.clone()),
        reply: Some(Reply {
            sequence: 2,
            outcome: Outcome::Applied,
        }),
        ..Default::default()
    });
    let applied = client
        .until(|s| {
            s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 1) && !s.inventory_pending
        })
        .await;
    assert_eq!(applied.inventory.unwrap().bag, committed);
    client
        .items
        .as_ref()
        .unwrap()
        .send(
            Command {
                expected_revision: 1,
                operation: Operation::Move { from: 0, to: 2 },
            }
            .into(),
        )
        .await
        .unwrap();
    let next = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.sequence, 3);
    let moved = next.command.propose(&committed, profile.catalog()).unwrap();
    publish.send_replace(net::inventory::Observation {
        bag: Some(moved),
        reply: Some(Reply {
            sequence: 3,
            outcome: Outcome::Applied,
        }),
        ..Default::default()
    });
    client
        .until(|s| {
            s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 2) && !s.inventory_pending
        })
        .await;
    client.close().await;
    stop.send_replace(true);
    assert_eq!(
        server.await.unwrap().unwrap().inventory_requests_forwarded,
        3
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_melee_uses_each_actors_committed_gear_and_cancels_on_unequip() {
    melee_delivery(false, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_avoidance_resolves_once_after_commit_and_reaches_both_real_peers() {
    melee_delivery(true, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_projectile_launch_precedes_health_change_and_one_impact_reaches_both_peers() {
    melee_delivery(false, true).await;
}
async fn melee_delivery(miss: bool, projectile: bool) {
    use theia_core::{
        abilities::Catalog,
        content::ContentId,
        inventory::{EquipmentCommand, EquipmentOperation as Op},
    };
    use theia_network::simulation::combat::{Event, Outcome, Reason};
    let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-melee-abilities.json"
    ))
    .unwrap();
    if miss {
        data["schema"] = serde_json::json!(5);
        data["abilities"][1]["hit_table"] =
            serde_json::json!({"hit_bp":0,"miss_bp":10000,"deflect_bp":0});
    }
    if projectile {
        data["schema"] = serde_json::json!(6);
        data["abilities"][1]["projectile"] = serde_json::json!({"speed_metres_per_second":2.,"maximum_distance_metres":6.,"lifetime_ms":3000,"target_radius_metres":0.2});
        data["abilities"][1]["hit_table"] =
            serde_json::json!({"hit_bp":10000,"miss_bp":0,"deflect_bp":0});
    }
    let abilities = serde_json::to_vec(&data).unwrap();
    let mut world = World::with_procedural([15400., 12860.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let target = world.spawn_development_target([15402., 12860.]).unwrap();
    world
        .enable_status_combat(
            &abilities,
            100,
            5000,
            include_bytes!("../../../content/gameplay/development-character-stats.json"),
            include_bytes!("../../../content/gameplay/development-statuses.json"),
        )
        .unwrap();
    let identity = world.identity();
    let patch = Patch::build(ChunkId { x: 120, z: 100 });
    let (stop, shutdown) = watch::channel(false);
    let (release_a, waiting) = watch::channel(false);
    let (_release_b, ready) = watch::channel(true);
    let (items_a, task_a, received_a) =
        synthetic_items_with_profile(13, waiting, shutdown.clone(), true);
    let (items_b, task_b, _) = synthetic_items_with_profile(7, ready, shutdown.clone(), true);
    let policies: [Arc<Policy>; 2] = std::array::from_fn(|i| {
        let policy = Arc::new(Policy::default());
        policy.live.store(true, Ordering::SeqCst);
        let mut p = patch.try_spawn(15400., 12860. + i as f64).unwrap().metres();
        p[1] += f64::from(theia_network::world::PAWN_Y);
        {
            let mut ledger = policy.state.lock().unwrap();
            ledger.code = i as u8 + 1;
            ledger.position = GlobalPosition::new(p);
            ledger.inventory = Some(if i == 0 {
                items_a.clone()
            } else {
                items_b.clone()
            });
        }
        policy
    });
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let server = tokio::spawn(net::run_server_with_admissions(
        transport,
        shutdown,
        world,
        Arc::new(Routing(policies)),
    ));
    let profile = net::inventory::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let catalog = Arc::new(Catalog::from_json(&abilities).unwrap());
    let token = |id, code| {
        issuer
            .issue(id, Some(&[code; 256]), 30)
            .unwrap()
            .bytes()
            .to_vec()
    };
    let mut a = Client::start_profiles(
        address,
        token(1, 1),
        identity,
        Some(profile.clone()),
        Some(catalog.clone()),
    );
    let mut b =
        Client::start_profiles(address, token(2, 2), identity, Some(profile), Some(catalog));
    a.until(|s| s.inventory.is_some() && s.chunk_ack.is_some() && s.combat.is_some())
        .await;
    b.until(|s| s.inventory.is_some() && s.chunk_ack.is_some() && s.combat.is_some())
        .await;
    let choice = || net::CastChoice {
        ability: ContentId::new("theia.dev.sword-slash").unwrap(),
        target: target.entity,
    };
    a.casts.send(choice()).await.unwrap();
    a.until(|s| {
        s.combat.as_ref().is_some_and(|c| {
            c.reply.is_some_and(|r| {
                r.sequence == 1 && r.outcome == Outcome::Rejected(Reason::EquipmentRequired)
            })
        })
    })
    .await;
    a.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 0,
                operation: Op::Equip { from: 1 },
            }
            .into(),
        )
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while received_a.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    a.casts.send(choice()).await.unwrap();
    a.until(|s| {
        s.combat.as_ref().is_some_and(|c| {
            c.reply.is_some_and(|r| {
                r.sequence == 2 && r.outcome == Outcome::Rejected(Reason::EquipmentRequired)
            })
        })
    })
    .await;
    release_a.send_replace(true);
    a.until(|s| s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 1))
        .await;
    b.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 0,
                operation: Op::Equip { from: 2 },
            }
            .into(),
        )
        .await
        .unwrap();
    b.until(|s| s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 1))
        .await;
    b.casts.send(choice()).await.unwrap();
    b.until(|s| {
        s.combat.as_ref().is_some_and(|c| {
            c.reply
                .is_some_and(|r| r.outcome == Outcome::Rejected(Reason::EquipmentRequired))
        })
    })
    .await;
    a.casts.send(choice()).await.unwrap();
    a.until(|s| {
        s.combat
            .as_ref()
            .is_some_and(|c| c.cast.as_ref().is_some_and(|c| c.sequence == 3))
    })
    .await;
    a.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 1,
                operation: Op::Unequip { to: 1 },
            }
            .into(),
        )
        .await
        .unwrap();
    let unequipped = a
        .until(|s| {
            s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 2)
                && s.combat.as_ref().is_some_and(|c| c.cast.is_none())
        })
        .await;
    assert_eq!(unequipped.combat.unwrap().targets[0].current, 100);
    a.items
        .as_ref()
        .unwrap()
        .send(
            EquipmentCommand {
                expected_revision: 2,
                operation: Op::Equip { from: 1 },
            }
            .into(),
        )
        .await
        .unwrap();
    a.until(|s| {
        s.inventory.as_ref().is_some_and(|i| i.bag.revision() == 3)
            && s.combat
                .as_ref()
                .is_some_and(|c| c.gcd_until <= c.tick && c.cooldowns.is_empty())
    })
    .await;
    a.casts.send(choice()).await.unwrap();
    if projectile {
        let launched = a
            .until(|s| {
                s.combat.as_ref().is_some_and(|c| {
                    c.cast.is_none()
                        && c.reply.is_some_and(|r| r.sequence == 4)
                        && c.targets[0].current == 100
                })
            })
            .await;
        let tick = launched.combat.unwrap().tick;
        b.until(|s| {
            // An observer's unchanged combat view is intentionally not
            // republished merely because a projectile launched. Use its live
            // movement clock while checking the retained authoritative health.
            s.snapshot.as_ref().is_some_and(|s| s.tick >= tick)
                && s.combat
                    .as_ref()
                    .is_some_and(|c| c.targets[0].current == 100)
        })
        .await;
    }
    a.until(|s| {
        s.combat.as_ref().is_some_and(|c| {
            c.targets[0].current == if miss { 100 } else { 80 }
                && c.cast.is_none()
                && c.reply.is_some_and(|r| r.sequence == 4)
        })
    })
    .await;
    b.until(|s| {
        s.combat
            .as_ref()
            .is_some_and(|c| c.targets[0].current == if miss { 100 } else { 80 })
    })
    .await;
    let mut cancelled = false;
    let mut avoided = [0usize; 2];
    let mut launches = [Vec::new(), Vec::new()];
    let mut impacts = [Vec::new(), Vec::new()];
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            for (index, peer) in [&mut a, &mut b].into_iter().enumerate() {
                while let Ok(batch) = peer.combat_events.try_recv() {
                    for event in batch.events {
                        if matches!(event, Event::ProjectileLaunched { sequence: 4, .. }) {
                            launches[index].push(batch.tick);
                        }
                        if matches!(event, Event::Damage { sequence: 4, .. }) {
                            impacts[index].push(batch.tick);
                        }
                        cancelled |= index == 0
                            && matches!(
                                event,
                                Event::Cancelled {
                                    sequence: 3,
                                    reason: Reason::EquipmentRequired,
                                    ..
                                }
                            );
                        avoided[index] += usize::from(matches!(
                            event,
                            Event::Avoided {
                                sequence: 4,
                                kind: theia_network::simulation::combat::Avoidance::Miss,
                                ..
                            }
                        ));
                    }
                }
            }
            if cancelled
                && (!miss || avoided == [1, 1])
                && (!projectile
                    || (launches.iter().all(|v| v.len() == 1)
                        && impacts.iter().all(|v| v.len() == 1)))
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("Reliable outcomes did not reach both encrypted peers");
    assert_eq!(avoided, if miss { [1, 1] } else { [0, 0] });
    if projectile {
        assert_eq!(launches[0], launches[1]);
        assert_eq!(impacts[0], impacts[1]);
        assert!(
            impacts[0][0] >= launches[0][0] + 20,
            "Travel must delay impact at 30 Hz"
        );
    }
    a.close().await;
    b.close().await;
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    if miss || projectile {
        let outcomes = stats.hit_outcomes.unwrap();
        assert_eq!(outcomes.resolved, 1);
        assert_eq!(outcomes.random_draws, 1);
        assert_eq!(outcomes.misses, u64::from(miss));
        assert_eq!(outcomes.hits, u64::from(projectile));
    } else {
        assert!(stats.hit_outcomes.is_none());
    }
    if projectile {
        let travel = stats.projectiles.unwrap();
        assert_eq!(travel.launched, 1);
        assert_eq!(travel.impacts, 1);
        assert_eq!(travel.peak_active, 1);
    }
    task_a.await.unwrap();
    task_b.await.unwrap();
}
