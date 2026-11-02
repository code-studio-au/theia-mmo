//! Actual encrypted Renet, real World HP and deterministic UDP impairment.
//! Inventory ownership is a synthetic committed fixture; no OIDC, SQL durability,
//! native rendering or MMO-scale-capacity claim is made here.
use super::*;
use crate::simulation::creatures;
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
use theia_core::{abilities, melee};
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
    include_bytes!("../../../../../../../content/gameplay/development-equipment-items.json");
const VITALS: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-character-stats.json");
const ABILITIES: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-classic-abilities.json");
const STATUSES: &[u8] =
    include_bytes!("../../../../../../../content/gameplay/development-statuses.json");
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
    physical: Arc<crate::simulation::combat::physical::Profile>,
    white: Arc<white_melee::Profile>,
    ability: Arc<Profile>,
    life: Arc<crate::player_life_wire::Profile>,
    membership: Option<Arc<crate::combat_membership_wire::Profile>>,
    progression: Option<Arc<crate::progression_wire::Profile>>,
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
    fixture_mode(rate, false)
}
fn fixture_mode(rate: TickRate, membership: bool) -> Fixture {
    let mut creature: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../../../content/gameplay/development-creature-patrol.json"
    ))
    .unwrap();
    creature["speed_metres_per_second"] = serde_json::json!(0.1);
    creature["aggro"] = serde_json::json!({"perception_metres":1.0,"leash_metres":35.0,"chase_speed":5.0,"return_speed":5.0,"los_grace_ms":2000,"engagement_ms":30000,"reacquire_ms":2000});
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
                "../../../../../../../content/gameplay/development-creature-retaliation.json"
            ),
            include_bytes!("../../../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    );
    world.enable_player_life(life.clone()).unwrap();
    world
        .enable_autoattack(
            include_bytes!("../../../../../../../content/gameplay/development-melee-weapons.json"),
            ITEMS,
        )
        .unwrap();
    world
        .enable_physical_damage(
            include_bytes!(
                "../../../../../../../content/gameplay/development-physical-damage-definitions.json"
            ),
            include_bytes!(
                "../../../../../../../content/gameplay/development-physical-damage-stats.json"
            ),
            ITEMS,
        )
        .unwrap();
    world
        .enable_white_melee(include_bytes!(
            "../../../../../../../content/gameplay/development-white-melee.json"
        ))
        .unwrap();
    let mut policy =
        include_bytes!("../../../../../../../content/gameplay/development-ability-damage.json")
            .to_vec();
    if membership {
        let mut raw: serde_json::Value = serde_json::from_slice(&policy).unwrap();
        raw["abilities"][0]["miss_bp"] = 0.into();
        raw["abilities"][0]["dodge_bp"] = 0.into();
        raw["abilities"][0]["critical_bp"] = 0.into();
        policy = serde_json::to_vec(&raw).unwrap();
    }
    world
        .enable_ability_damage(
            include_bytes!(
                "../../../../../../../content/gameplay/development-ability-starting.json"
            ),
            &policy,
            ABILITIES,
        )
        .unwrap();
    if membership {
        world
            .enable_combat_membership(include_bytes!(
                "../../../../../../../content/gameplay/development-combat-membership.json"
            ))
            .unwrap();
        // Explicit low-health two-hit fixture, not production wolf balance.
        world.combat.as_mut().unwrap().creatures[0].current = 20;
    }
    world
        .combat
        .as_mut()
        .unwrap()
        .ability_damage
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
    slots[1] = Stack::new(
        &catalog,
        ContentId::new("theia.dev.training-wand").unwrap(),
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
        white: world.white_melee_profile().unwrap().clone(),
        ability: world.ability_damage_profile().unwrap().clone(),
        life,
        membership: world.combat_membership_profile().cloned(),
        progression: None,
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
    deaths: usize,
    respawns: usize,
    facts: Vec<Fact>,
    ordinary: Vec<white_melee::Fact>,
    arcane_launch_tick: Option<u64>,
    arcane_fact_tick: Option<u64>,
    life: Vec<crate::simulation::combat::player_life::Observation>,
}
struct Peer {
    state: watch::Receiver<State>,
    seen: watch::Receiver<Seen>,
    melee: watch::Sender<Option<net::MeleeChoice>>,
    client: JoinHandle<Result<()>>,
    driver: JoinHandle<()>,
    collector: Option<JoinHandle<Seen>>,
    life: JoinHandle<Vec<crate::simulation::combat::player_life::Observation>>,
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
        let life = tokio::spawn(async move {
            let mut observed = Vec::new();
            while let Some(value) = life_rx.recv().await {
                assert!(observed.len() < 128);
                observed.push(value);
            }
            observed
        });
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
                            seen.deaths += usize::from(matches!(event, Event::Died { .. }));
                            seen.respawns += usize::from(matches!(event, Event::Respawned { .. }));
                            if let Event::ProjectileLaunched { ref ability, .. } = event
                                && ability.as_str() == "theia.dev.arcane-bolt"
                            {
                                seen.arcane_launch_tick = Some(batch.tick);
                            }
                            if let Event::WhiteMelee(ref fact) = event {
                                assert!(seen.ordinary.len() < 64);
                                seen.ordinary.push(fact.clone());
                            }
                            if let Event::AbilityDamage(fact) = event {
                                if matches!(fact.result.outcome, Outcome::Arcane(_)) {
                                    seen.arcane_fact_tick = Some(batch.tick);
                                }
                                assert!(
                                    seen.facts.len() < 64,
                                    "Ability UDP fixture exceeded its bounded capture"
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
        let driven_input = input.subscribe();
        let driver = tokio::spawn(async move {
            let mut driver = crate::prediction_driver::Driver::with_classic(source, false);
            loop {
                tokio::select! {_ = driver_stop.changed()=>break,_ = tokio::time::sleep(Duration::from_millis(8))=>{
                 let raw=driven.borrow().clone();if raw.error.is_some(){break;}let mut intent=*driven_input.borrow();if intent.facing.is_none(){intent.facing=Some(Facing::from_units(0));}
                 if let Some(driven)=driver.tick(&raw,intent,Some(&patch),std::time::Instant::now()).unwrap() && let Some(packet)=driven.packet {let _=commands.try_send(packet);}
                }}
            }
        });
        let client = tokio::spawn(async move {
            let result=net::run_client_gameplay(address,&token,inputs,publish.clone(),shutdown,session.identity,net::GameplayClient {
    classic_source:Some(session.source),dodge:None,player_life:Some(net::player_life::Client{profile:session.life,membership:session.membership,events:life_events}),lifecycle:None,dialogue:None,ready:readiness,commands:Some(command_rx),choices:Some(choice_rx),
    combat:Some(net::CombatActions{melee:Some(net::MeleeActions{catalog:Arc::new(melee::Catalog::from_json(include_bytes!("../../../../../../../content/gameplay/development-melee-weapons.json")).unwrap()),physical:Some(session.physical),white:Some(session.white),ability:Some(session.ability),choices:melee_rx}),catalog:session.abilities,casts:cast_rx,events}),quest:None,progression:session.progression,inventory:Some(net::inventory::Client{profile:session.items,commands:item_rx}),statuses:Some(Arc::new(theia_core::statuses::Catalog::from_json(STATUSES).unwrap())),
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
        tokio::time::timeout(Duration::from_secs(12),async {loop {let raw=self.state.borrow().clone();let seen=self.seen.borrow().clone();if predicate(&raw,&seen){return raw;}assert!(raw.error.is_none(),"Ability UDP client failed: {:?}",raw.error);tokio::select!{value=self.state.changed()=>value.expect("Ability client closed"),value=self.seen.changed()=>value.expect("Ability capture closed")}}}).await.expect("Ability UDP acceptance timed out")
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
        let life = self.life.await.unwrap();
        let mut seen = if let Some(collector) = self.collector {
            collector.await.unwrap()
        } else {
            assert_eq!(self.blocked.unwrap().len(), 1);
            Seen::default()
        };
        seen.life = life;
        seen
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_impaired_ability_and_white_results_preserve_private_hp_and_public_continuity_all_rates()
 {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let f = fixture(rate);
        let npc = f.npc;
        let session = f.session.clone();
        let observed = f._observed;
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "ability-wifi",
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
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        a.until(|_, seen| {
            seen.facts
                .iter()
                .any(|f| matches!(f.result.outcome, Outcome::MeleeSpecial(_)))
                && seen.ordinary.iter().any(|f| f.actor == owner)
        })
        .await;
        a.attack(None);
        a.until(|s, _| {
            s.autoattack
                .as_ref()
                .is_some_and(|a| a.phase == crate::simulation::combat::autoattack::Phase::Idle)
        })
        .await;
        a.until(|s, seen| {
            let lost: u32 = seen
                .ordinary
                .iter()
                .filter(|f| f.target_generation.is_some())
                .map(|f| f.result.health_lost)
                .sum();
            seen.ordinary
                .iter()
                .filter(|f| f.target_generation.is_some())
                .count()
                >= 2
                && s.player_life
                    .as_ref()
                    .and_then(|p| p.current())
                    .is_some_and(|p| p.health == 100 - lost)
        })
        .await;
        observed.send_modify(|value| {
            let bag = value.bag.as_ref().unwrap();
            value.bag = Some(
                bag.propose_equipment(
                    session.items.catalog(),
                    EquipmentCommand {
                        expected_revision: bag.revision(),
                        operation: EquipmentOperation::Equip { from: 1 },
                    },
                )
                .unwrap(),
            );
        });
        a.until(|s, _| {
            s.inventory.as_ref().is_some_and(|v| {
                v.bag
                    .main_hand()
                    .is_some_and(|i| i.item().as_str() == "theia.dev.training-wand")
            })
        })
        .await;
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        a.until(|s, seen| {
            seen.arcane_fact_tick.is_some() && s.combat.as_ref().is_some_and(|c| c.cast.is_none())
        })
        .await;
        b.until(|_, seen| seen.facts.len() >= 2 && seen.arcane_fact_tick.is_some())
            .await;
        let seen = a.seen.borrow().clone();
        let other = b.seen.borrow().clone();
        assert!(seen.arcane_fact_tick.unwrap() > seen.arcane_launch_tick.unwrap());
        assert!(other.ordinary.iter().all(|f| f.target_generation.is_none()));
        assert!(
            other
                .facts
                .iter()
                .all(|f| f.actor == owner && f.target == npc)
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
        assert!(a.state.borrow().error.is_none() && b.state.borrow().error.is_none());
        // A second public result after owner-only white damage must remain contiguous.
        observed.send_modify(|value| {
            let bag = value.bag.as_ref().unwrap();
            value.bag = Some(
                bag.propose_equipment(
                    session.items.catalog(),
                    EquipmentCommand {
                        expected_revision: bag.revision(),
                        operation: EquipmentOperation::Equip { from: 1 },
                    },
                )
                .unwrap(),
            );
        });
        a.until(|s, _| {
            s.inventory.as_ref().is_some_and(|v| {
                v.bag
                    .main_hand()
                    .is_some_and(|i| i.item().as_str() == "theia.dev.training-sword")
            }) && s.combat.as_ref().is_some_and(|c| c.gcd_until <= c.tick)
        })
        .await;
        a._casts
            .try_send(net::CastChoice {
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: npc.entity,
            })
            .unwrap();
        a.until(|_, s| s.facts.len() >= 3).await;
        b.until(|_, s| s.facts.len() >= 3).await;
        stop.send_replace(true);
        let owner_seen = a.finish(false).await;
        let public_seen = b.finish(false).await;
        let stats = server.await.unwrap().unwrap();
        let metrics = stats.ability_damage.unwrap();
        assert_eq!(metrics.base_draws, 3);
        assert_eq!(metrics.landing_draws, 3);
        assert_eq!((metrics.special.attempts, metrics.arcane.attempts), (2, 1));
        assert_eq!(
            metrics.critical_draws,
            owner_seen
                .facts
                .iter()
                .filter(|f| f.draws.critical.is_some())
                .count() as u64
        );
        assert_eq!(owner_seen.facts, public_seen.facts);
        assert_eq!(stats.physical_damage.unwrap().base_draws, 0);
        for r in relays {
            r.await.unwrap();
        }
        for c in controls {
            let c = c.lock().unwrap();
            assert!(
                c.stats.configured_drops > 0 && c.stats.duplicates > 0 && c.stats.reordered > 0
            );
        }
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stalled_ability_presentation_disconnects_without_dropping_resolved_facts() {
    let f = fixture(TickRate::Hz30);
    let npc = f.npc;
    let relay = relay::Relay::bind(relay::Profile {
        name: "ability-backpressure",
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
    peer._casts
        .try_send(net::CastChoice {
            ability: ContentId::new("theia.dev.sword-slash").unwrap(),
            target: npc.entity,
        })
        .unwrap();
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
    assert_eq!(stats.ability_damage.unwrap().base_draws, 1);
    relay.await.unwrap();
}
#[tokio::test]
async fn unbound_ability_ordinary_or_changed_catalogue_is_refused_before_network_io() {
    let f = fixture(TickRate::Hz30);
    for case in 0..4 {
        let (_input, input) = watch::channel(Intent::default());
        let (state, _) = watch::channel(State::default());
        let (_stop, shutdown) = watch::channel(false);
        let (_ready, ready) = watch::channel(None);
        let (_commands, commands) = mpsc::channel(1);
        let (_choices, choices) = watch::channel(None);
        let (_casts, casts) = mpsc::channel(1);
        let (events, _) = mpsc::channel(1);
        let mut white_data =
            include_bytes!("../../../../../../../content/gameplay/development-white-melee.json")
                .to_vec();
        white_data.push(b' ');
        let white = if case == 0 {
            None
        } else if case == 1 {
            Some(Arc::new(
                white_melee::Profile::new(&white_data, f.session.physical.clone()).unwrap(),
            ))
        } else {
            Some(f.session.white.clone())
        };
        let mut data: serde_json::Value = serde_json::from_slice(ABILITIES).unwrap();
        if case == 2 {
            data["global_cooldown_ms"] = 1600.into();
        }
        if case == 3 {
            data["abilities"][0]["projectile"]["speed_metres_per_second"] = 21.into();
        }
        let catalogue = if case >= 2 {
            Arc::new(abilities::Catalog::from_json(&serde_json::to_vec(&data).unwrap()).unwrap())
        } else {
            f.session.abilities.clone()
        };
        let error=net::run_client_gameplay("127.0.0.1:1".parse().unwrap(),&[],input,state,shutdown,f.session.identity,net::GameplayClient{classic_source:Some(f.session.source),dodge:None,player_life:None,lifecycle:None,dialogue:None,ready,commands:Some(commands),choices:None,combat:Some(net::CombatActions{catalog:catalogue,casts,events,melee:Some(net::MeleeActions{catalog:Arc::new(melee::Catalog::from_json(include_bytes!("../../../../../../../content/gameplay/development-melee-weapons.json")).unwrap()),physical:Some(f.session.physical.clone()),white,ability:Some(f.session.ability.clone()),choices})}),quest:None,progression:None,inventory:None,statuses:None}).await.unwrap_err();
        assert!(
            error.to_string().contains("Unbound ability-damage"),
            "{error}"
        );
    }
}

mod membership;

mod credit;

mod reset;
