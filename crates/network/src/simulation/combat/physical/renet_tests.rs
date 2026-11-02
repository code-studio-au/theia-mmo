//! Actual encrypted Renet, real World HP and deterministic UDP impairment.
//! Inventory ownership is a synthetic committed fixture; no OIDC, SQL durability,
//! native rendering or MMO-scale-capacity claim is made here.
use super::*;
use crate::test_relay as relay;
use crate::{
    client::State,
    combat_wire::Batch,
    renet_runtime::{self as net, AccountAdmissions, AdmissionLease, PlayerAdmission},
    wire::Intent,
};
use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};
use theia_core::{
    facing::Facing,
    inventory::{Bag, EquipmentCommand, EquipmentOperation, Stack},
    time::TickRate,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
const ITEMS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-equipment-items.json");
const VITALS: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-character-stats.json");
const ABILITIES: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-classic-abilities.json");
const STATUSES: &[u8] =
    include_bytes!("../../../../../../content/gameplay/development-statuses.json");
struct Policy(net::inventory::Port, GlobalPosition);
struct Lease(net::inventory::Port);
impl AdmissionLease for Lease {
    fn inventory(&self) -> Result<Option<net::inventory::Port>> {
        Ok(Some(self.0.clone()))
    }
    fn live(&self) -> Result<bool> {
        Ok(true)
    }
    fn record(&self, _: EntityKey, _: GlobalPosition, _: u64) -> Result<()> {
        Ok(())
    }
}
impl AccountAdmissions for Policy {
    fn consume(self: Arc<Self>, id: u64, _: &[u8; 256]) -> Result<Option<PlayerAdmission>> {
        Ok((1..=2).contains(&id).then(|| PlayerAdmission {
            position: self.1,
            lease: Box::new(Lease(self.0.clone())),
        }))
    }
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; 8]> {
        Ok([None; 8])
    }
}
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
#[derive(Clone)]
struct Session {
    source: [u8; 32],
    identity: [u8; 32],
    physical: Arc<Profile>,
    life: Arc<crate::player_life_wire::Profile>,
    abilities: Arc<abilities::Catalog>,
    patch: Arc<crate::procedural::Patch>,
    items: net::inventory::Profile,
}
struct Fixture {
    world: World,
    npc: EntityKey,
    session: Session,
    policy: Arc<Policy>,
    _observed: watch::Sender<net::inventory::Observation>,
    _requests: mpsc::Receiver<net::inventory::Request>,
}
fn fixture(rate: TickRate) -> Fixture {
    let mut creature: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../../content/gameplay/development-creature-patrol.json"
    ))
    .unwrap();
    creature["speed_metres_per_second"] = serde_json::json!(0.1);
    let creature = serde_json::to_vec(&creature).unwrap();
    let (mut world, npc) = creatures::tests::fixture_profile_at(rate, [8008., 8002.], &creature);
    world.enable_prediction().unwrap();
    world.enable_classic().unwrap();
    world
        .enable_status_combat(ABILITIES, 1000, 5000, VITALS, STATUSES)
        .unwrap();
    let life = Arc::new(
        crate::player_life_wire::Profile::from_json(
            VITALS,
            &creature,
            include_bytes!(
                "../../../../../../content/gameplay/development-creature-retaliation.json"
            ),
            include_bytes!("../../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    );
    world.enable_player_life(life.clone()).unwrap();
    world
        .enable_autoattack(
            include_bytes!("../../../../../../content/gameplay/development-melee-weapons.json"),
            ITEMS,
        )
        .unwrap();
    world
        .enable_physical_damage(
            include_bytes!(
                "../../../../../../content/gameplay/development-physical-damage-definitions.json"
            ),
            include_bytes!(
                "../../../../../../content/gameplay/development-physical-damage-stats.json"
            ),
            ITEMS,
        )
        .unwrap();
    world
        .combat
        .as_mut()
        .unwrap()
        .physical
        .as_mut()
        .unwrap()
        .random = outcomes::Random::seed_for_test([7; 32]);
    let source = world
        .combat
        .as_ref()
        .unwrap()
        .identity(world.patrol_identity.unwrap());
    let identity = world.identity();
    assert_eq!(identity, crate::classic::profile_identity(source));
    let patch = world
        .procedural
        .as_ref()
        .unwrap()
        .patch(GlobalPosition::new([8008., 0., 8002.]).unwrap())
        .unwrap()
        .clone();
    let mut position = patch.try_spawn(8008., 8002.).unwrap().metres();
    position[1] += f64::from(crate::world::PAWN_Y);
    let position = GlobalPosition::new(position).unwrap();
    let items = net::inventory::Profile::from_json(ITEMS).unwrap();
    let catalog = Arc::new(items.catalog().clone());
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(
        &catalog,
        ContentId::new("theia.dev.training-sword").unwrap(),
        1,
    );
    let bag = Bag::from_slots(&catalog, 0, slots)
        .unwrap()
        .propose_equipment(
            &catalog,
            EquipmentCommand {
                expected_revision: 0,
                operation: EquipmentOperation::Equip { from: 0 },
            },
        )
        .unwrap();
    let (observed, observations) = watch::channel(net::inventory::Observation {
        bag: Some(bag),
        ..Default::default()
    });
    let (requests, receiver) = mpsc::channel(1);
    let policy = Arc::new(Policy(
        net::inventory::Port {
            vendor: None,
            rewards: None,
            opportunities: None,
            catalog,
            hash: items.hash(),
            requests,
            state: observations,
        },
        position,
    ));
    let session = Session {
        source,
        identity,
        physical: world.physical_damage_profile().unwrap().clone(),
        life,
        abilities: Arc::new(abilities::Catalog::from_json(ABILITIES).unwrap()),
        patch,
        items,
    };
    Fixture {
        world,
        npc,
        session,
        policy,
        _observed: observed,
        _requests: receiver,
    }
}
#[derive(Clone, Default)]
struct Seen {
    batches: u64,
    facts: Vec<Fact>,
}
struct Peer {
    state: watch::Receiver<State>,
    seen: watch::Receiver<Seen>,
    melee: watch::Sender<Option<net::MeleeChoice>>,
    client: JoinHandle<Result<()>>,
    driver: JoinHandle<()>,
    collector: Option<JoinHandle<Seen>>,
    life: JoinHandle<()>,
    blocked: Option<mpsc::Receiver<Batch>>,
    _input: watch::Sender<Intent>,
    _ready: watch::Sender<Option<Vec<theia_core::coordinates::ChunkId>>>,
    _choices: watch::Sender<Option<crate::targeting::Choice>>,
    _casts: mpsc::Sender<net::CastChoice>,
    _items: mpsc::Sender<theia_core::inventory::ItemCommand>,
    _seen: watch::Sender<Seen>,
}
impl Peer {
    fn start(
        address: SocketAddr,
        token: Vec<u8>,
        session: Session,
        shutdown: watch::Receiver<bool>,
        blocked: bool,
    ) -> Self {
        let (input, inputs) = watch::channel(Intent::default());
        let (ready, readiness) = watch::channel(Some(session.patch.ids.iter().copied().collect()));
        let (commands, command_rx) = mpsc::channel(8);
        let (choices, choice_rx) = watch::channel(None);
        let (melee, melee_rx) = watch::channel(None);
        let (casts, cast_rx) = mpsc::channel(8);
        let (events, event_rx) = mpsc::channel::<Batch>(if blocked { 1 } else { 16 });
        let (item_commands, item_rx) = mpsc::channel(1);
        let (life_events, mut life_rx) = mpsc::channel(16);
        let (publish, state) = watch::channel(State::default());
        let (seen_tx, seen) = watch::channel(Seen::default());
        let life = tokio::spawn(async move { while life_rx.recv().await.is_some() {} });
        let (collector, blocked_receiver) = if blocked {
            (None, Some(event_rx))
        } else {
            let mut rx = event_rx;
            let observed = seen_tx.clone();
            (
                Some(tokio::spawn(async move {
                    let mut seen = Seen::default();
                    while let Some(batch) = rx.recv().await {
                        seen.batches += 1;
                        for event in batch.events {
                            if let Event::PhysicalDamage(fact) = event {
                                assert!(
                                    seen.facts.len() < 64,
                                    "Physical UDP fixture exceeded its bounded capture"
                                );
                                seen.facts.push(fact);
                            }
                        }
                        observed.send_replace(seen.clone());
                    }
                    seen
                })),
                None,
            )
        };
        let mut driver_stop = shutdown.clone();
        let driven = state.clone();
        let patch = session.patch.clone();
        let source = session.source;
        let driver = tokio::spawn(async move {
            let mut driver = crate::prediction_driver::Driver::with_classic(source, false);
            loop {
                tokio::select! {_ = driver_stop.changed()=>break,_ = tokio::time::sleep(Duration::from_millis(8))=>{
                 let raw=driven.borrow().clone();if raw.error.is_some(){break;}let intent=Intent{facing:Some(Facing::from_units(0)),..Default::default()};
                 if let Some(driven)=driver.tick(&raw,intent,Some(&patch),std::time::Instant::now()).unwrap() && let Some(packet)=driven.packet {let _=commands.try_send(packet);}
                }}
            }
        });
        let client = tokio::spawn(async move {
            let result=net::run_client_gameplay(address,&token,inputs,publish.clone(),shutdown,session.identity,net::GameplayClient {
    classic_source:Some(session.source),dodge:None,player_life:Some(net::player_life::Client{profile:session.life,membership:None,events:life_events}),lifecycle:None,dialogue:None,ready:readiness,commands:Some(command_rx),choices:Some(choice_rx),
    combat:Some(net::CombatActions{melee:Some(net::MeleeActions{catalog:Arc::new(melee::Catalog::from_json(include_bytes!("../../../../../../content/gameplay/development-melee-weapons.json")).unwrap()),physical:Some(session.physical),white:None,ability:None,choices:melee_rx}),catalog:session.abilities,casts:cast_rx,events}),quest:None,progression:None,inventory:Some(net::inventory::Client{profile:session.items,commands:item_rx}),statuses:Some(Arc::new(theia_core::statuses::Catalog::from_json(STATUSES).unwrap())),
   }).await;
            if let Err(error) = &result {
                publish.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        Self {
            state,
            seen,
            melee,
            client,
            driver,
            collector,
            life,
            blocked: blocked_receiver,
            _input: input,
            _ready: ready,
            _choices: choices,
            _casts: casts,
            _items: item_commands,
            _seen: seen_tx,
        }
    }
    async fn until(&mut self, predicate: impl Fn(&State, &Seen) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(12),async {loop {let raw=self.state.borrow().clone();let seen=self.seen.borrow().clone();if predicate(&raw,&seen){return raw;}assert!(raw.error.is_none(),"Physical UDP client failed: {:?}",raw.error);tokio::select!{value=self.state.changed()=>value.expect("Physical client closed"),value=self.seen.changed()=>value.expect("Physical capture closed")}}}).await.expect("Physical UDP acceptance timed out")
    }
    fn attack(&self, target: Option<EntityKey>) {
        self.melee.send_replace(Some(net::MeleeChoice {
            stream_epoch: self
                .state
                .borrow()
                .correction
                .unwrap()
                .state
                .input
                .stream_epoch,
            target,
        }));
    }
    async fn finish(self, expected_error: bool) -> Seen {
        self.driver.await.unwrap();
        let result = self.client.await.unwrap();
        assert_eq!(
            result.is_err(),
            expected_error,
            "Unexpected physical UDP client completion"
        );
        self.life.await.unwrap();
        if let Some(collector) = self.collector {
            collector.await.unwrap()
        } else {
            assert_eq!(self.blocked.unwrap().len(), 1);
            Seen::default()
        }
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_impaired_physical_results_preserve_owner_privacy_and_recipient_sequences_all_rates()
 {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let f = fixture(rate);
        let npc = f.npc;
        let session = f.session.clone();
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "physical-wifi",
                    delay_ms: 20,
                    jitter_ms: 30,
                    loss_every: 11,
                    duplicate_every: 17,
                })
            })
            .collect();
        let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
        let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, addresses.clone()).unwrap();
        let tokens: Vec<_> = (1..=2)
            .map(|id| issuer.issue(id, None, 60).unwrap().bytes().to_vec())
            .collect();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let relays: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_with_admissions(
            transport,
            shutdown.clone(),
            f.world,
            f.policy,
        ));
        let mut a = Peer::start(
            addresses[0],
            tokens[0].clone(),
            session.clone(),
            shutdown.clone(),
            false,
        );
        let mut b = Peer::start(
            addresses[1],
            tokens[1].clone(),
            session.clone(),
            shutdown,
            false,
        );
        let initial = a
            .until(|s, _| {
                s.autoattack.is_some()
                    && s.chunk_ack.is_some()
                    && s.correction.is_some()
                    && s.player_life.is_some()
                    && s.snapshot.as_ref().is_some_and(|v| v.players.len() == 2)
            })
            .await;
        b.until(|s, _| {
            s.autoattack.is_some()
                && s.chunk_ack.is_some()
                && s.correction.is_some()
                && s.player_life.is_some()
        })
        .await;
        let owner = initial
            .player_life
            .as_ref()
            .unwrap()
            .current()
            .unwrap()
            .key
            .actor;
        a.attack(Some(npc));
        a.until(|_, seen| {
            seen.facts
                .iter()
                .any(|f| f.actor == owner && f.target == npc)
        })
        .await;
        a.attack(None);
        a.until(|s, _| {
            s.autoattack
                .as_ref()
                .is_some_and(|a| a.phase == super::super::autoattack::Phase::Idle)
        })
        .await;
        a.until(|s, seen| {
            seen.facts
                .iter()
                .filter(|f| f.target_generation.is_some())
                .count()
                >= 2
                && s.player_life
                    .as_ref()
                    .and_then(|p| p.current())
                    .is_some_and(|p| p.health < 100)
        })
        .await;
        b.until(|_, seen| {
            seen.facts
                .iter()
                .any(|f| f.actor == owner && f.target == npc)
        })
        .await;
        assert!(
            b.seen
                .borrow()
                .facts
                .iter()
                .all(|f| f.target_generation.is_none())
        );
        assert_eq!(
            b.state
                .borrow()
                .player_life
                .as_ref()
                .unwrap()
                .current()
                .unwrap()
                .health,
            100
        );
        a.attack(Some(npc));
        b.until(|_, seen| {
            seen.facts
                .iter()
                .filter(|f| f.actor == owner && f.target == npc)
                .count()
                >= 2
        })
        .await;
        assert!(b.state.borrow().error.is_none());
        assert!(b.seen.borrow().batches >= 2);
        assert!(a.seen.borrow().batches > b.seen.borrow().batches);
        stop.send_replace(true);
        let seen_a = a.finish(false).await;
        let seen_b = b.finish(false).await;
        let stats = server.await.unwrap().unwrap();
        for task in relays {
            task.await.unwrap();
        }
        let metrics = stats.physical_damage.unwrap();
        assert!(metrics.outgoing_hits >= 2 && metrics.incoming_hits >= 2);
        assert_eq!(
            metrics.base_draws,
            metrics.outgoing_hits + metrics.incoming_hits
        );
        for fact in &seen_a.facts {
            let before = fact.remaining + fact.result.health_lost;
            let expected = if fact.target_generation.is_some() {
                assert_eq!(fact.target, owner);
                session.physical.creature_result(fact.roll, before).unwrap()
            } else {
                session
                    .physical
                    .weapon_result(
                        &ContentId::new("theia.dev.training-sword").unwrap(),
                        fact.roll,
                        before,
                    )
                    .unwrap()
            };
            assert_eq!(fact.result, expected);
        }
        assert!(seen_b.facts.iter().all(|f| f.target_generation.is_none()));
        assert!(controls.iter().all(|c| {
            let c = c.lock().unwrap();
            c.stats.configured_drops > 0
                && c.stats.duplicates > 0
                && c.stats.reordered > 0
                && c.stats.overflow == 0
        }));
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn physical_presentation_backpressure_disconnects_instead_of_dropping_facts() {
    let f = fixture(TickRate::Hz30);
    let npc = f.npc;
    let relay = relay::Relay::bind(relay::Profile {
        name: "physical-backpressure",
        delay_ms: 20,
        jitter_ms: 20,
        loss_every: 11,
        duplicate_every: 17,
    });
    let address = relay.address;
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let backend = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let token = issuer.issue(1, None, 60).unwrap().bytes().to_vec();
    let (stop, shutdown) = watch::channel(false);
    let _guard = Stop(stop.clone());
    let relay = tokio::spawn(relay.run(backend, shutdown.clone()));
    let server = tokio::spawn(net::run_server_with_admissions(
        transport,
        shutdown.clone(),
        f.world,
        f.policy,
    ));
    let mut peer = Peer::start(address, token, f.session, shutdown, true);
    peer.until(|s, _| {
        s.autoattack.is_some()
            && s.chunk_ack.is_some()
            && s.correction.is_some()
            && s.player_life.is_some()
    })
    .await;
    peer.attack(Some(npc));
    let failed = peer.until(|s, _| s.error.is_some()).await;
    assert!(
        failed
            .error
            .unwrap()
            .contains("Combat presentation queue full or closed")
    );
    stop.send_replace(true);
    peer.finish(true).await;
    let stats = server.await.unwrap().unwrap();
    assert!(stats.physical_damage.unwrap().base_draws >= 2);
    relay.await.unwrap();
}
