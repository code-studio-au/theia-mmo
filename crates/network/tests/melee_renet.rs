//! Actual encrypted Renet + real World swings. Synthetic committed gear owner;
//! no OIDC, PostgreSQL durability or native presentation claim.
#[path = "support/relay.rs"]
#[allow(dead_code)]
mod relay;
use std::{net::UdpSocket, sync::Arc, time::Duration};
use theia_core::{
    content::ContentId,
    coordinates::{ChunkId, GlobalPosition},
    facing::Facing,
    identity::EntityKey,
    inventory::{Bag, EquipmentCommand, EquipmentOperation, Stack},
    time::TickRate,
};
use theia_network::{
    Result,
    client::State,
    combat_wire::Batch,
    renet_runtime::{self as net, AccountAdmissions, AdmissionLease, PlayerAdmission},
    simulation::{
        World,
        combat::{
            Event,
            autoattack::{self, Outcome, Pause, Phase},
        },
    },
    wire::Intent,
};
use tokio::sync::{mpsc, watch};
const WEAPONS: &[u8] = include_bytes!("../../../content/gameplay/development-melee-weapons.json");
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const ABILITIES: &[u8] = include_bytes!("../../../content/gameplay/development-abilities.json");
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
        Ok((id == 1).then(|| PlayerAdmission {
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
async fn until(state: &mut watch::Receiver<State>, pred: impl Fn(&State) -> bool) -> State {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let view = state.borrow().clone();
            assert!(view.error.is_none(), "{:?}", view.error);
            if pred(&view) {
                return view;
            }
            state
                .changed()
                .await
                .expect("client ended before acceptance");
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "melee UDP observation timeout: {:?}",
            state.borrow().clone()
        )
    })
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_impaired_engage_pause_resume_cancel_and_damage_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        scenario(rate, false, false).await;
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn melee_presentation_backpressure_disconnects_without_dropping_damage() {
    scenario(TickRate::Hz30, true, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn explicit_ability_facing_rejection_and_suspending_cast_use_encrypted_udp_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        scenario(rate, false, true).await;
    }
}
async fn scenario(rate: TickRate, blocked_presentation: bool, explicit_rules: bool) {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(world.set_tick_rate(rate));
    world.enable_prediction().unwrap();
    let target = world.spawn_development_target([8000., 7998.]).unwrap();
    let mut ability_bytes = ABILITIES.to_vec();
    const STATS: &[u8] =
        include_bytes!("../../../content/gameplay/development-character-stats.json");
    const STATUSES: &[u8] = include_bytes!("../../../content/gameplay/development-statuses.json");
    let source = if explicit_rules {
        let mut raw: serde_json::Value = serde_json::from_slice(ABILITIES).unwrap();
        raw["schema"] = serde_json::json!(7);
        for ability in raw["abilities"].as_array_mut().unwrap() {
            ability["mana_cost"] = serde_json::json!(0);
            ability["combat"] = serde_json::json!({"minimum_range_metres":0.0,"facing":{"kind":"frontal","arc_degrees":180.0},"autoattack":{"kind":"suspend"}});
        }
        ability_bytes = serde_json::to_vec(&raw).unwrap();
        let base = theia_network::simulation::combat::status_profile_identity(
            world.identity(),
            &ability_bytes,
            100,
            5000,
            STATS,
            STATUSES,
        )
        .unwrap();
        world.enable_classic().unwrap();
        world
            .enable_status_combat(&ability_bytes, 100, 5000, STATS, STATUSES)
            .unwrap();
        autoattack::profile_identity(base, WEAPONS, ITEMS).unwrap()
    } else {
        world.enable_combat(ABILITIES, 100, 5000).unwrap();
        let source = autoattack::profile_identity(world.identity(), WEAPONS, ITEMS).unwrap();
        world.enable_classic().unwrap();
        source
    };
    let ability_id = theia_core::abilities::Catalog::from_json(&ability_bytes)
        .unwrap()
        .abilities()[0]
        .id()
        .clone();
    world.enable_autoattack(WEAPONS, ITEMS).unwrap();
    let identity = world.identity();
    assert_eq!(identity, theia_network::classic::profile_identity(source));
    let patch = theia_network::procedural::Patch::build(ChunkId { x: 62, z: 62 });
    let mut position = patch.try_spawn(8000., 8000.).unwrap().metres();
    position[1] += f64::from(theia_network::world::PAWN_Y);
    let position = GlobalPosition::new(position).unwrap();
    let profile = net::inventory::Profile::from_json(ITEMS).unwrap();
    let catalog = Arc::new(profile.catalog().clone());
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
    let (requests, _requests) = mpsc::channel(1);
    let policy = Arc::new(Policy(
        net::inventory::Port {
            vendor: None,
            rewards: None,
            opportunities: None,
            catalog,
            hash: profile.hash(),
            requests,
            state: observations,
        },
        position,
    ));
    let relay = relay::Relay::bind(relay::Profile {
        name: "melee-wifi",
        delay_ms: 20,
        jitter_ms: 25,
        loss_every: 10,
        duplicate_every: 17,
    });
    let address = relay.address;
    let relay_control = relay.control.clone();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let backend = socket.local_addr().unwrap();
    let directory = std::env::temp_dir().join(format!(
        "theia-melee-{}-{}-{blocked_presentation}-{explicit_rules}",
        std::process::id(),
        rate.hz()
    ));
    let transport = net::issue_local_advertised(socket, &directory, vec![address]).unwrap();
    let ticket = std::fs::read(directory.join("client-1.token")).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let _stop = Stop(stop.clone());
    let relay_task = tokio::spawn(relay.run(backend, shutdown.clone()));
    let server = tokio::spawn(net::run_server_with_admissions(
        transport,
        shutdown.clone(),
        world,
        policy,
    ));
    let (_input, input) = watch::channel(Intent::default());
    let (_ready, ready) = watch::channel(Some(patch.ids.iter().copied().collect()));
    let (commands, command_rx) = mpsc::channel(8);
    let (choices, choice_rx) = watch::channel(None);
    let (melee, melee_rx) = watch::channel(None);
    let (cast_requests, casts) = mpsc::channel(8);
    let (events, mut event_rx) = mpsc::channel::<Batch>(if blocked_presentation { 1 } else { 16 });
    let (_items, item_rx) = mpsc::channel(1);
    let (publish, mut state) = watch::channel(State::default());
    let mut driver_stop = shutdown.clone();
    let client = tokio::spawn(async move {
        let result = net::run_client_gameplay(
            address,
            &ticket,
            input,
            publish.clone(),
            shutdown,
            identity,
            net::GameplayClient {
                avatar_life: None,
                public_attacks: None,
                journal: None,
                classic_source: Some(source),
                dodge: None,
                player_life: None,
                lifecycle: None,
                dialogue: None,
                ready,
                commands: Some(command_rx),
                choices: Some(choice_rx),
                combat: Some(net::CombatActions {
                    catalog: Arc::new(
                        theia_core::abilities::Catalog::from_json(&ability_bytes).unwrap(),
                    ),
                    casts,
                    events,
                    melee: Some(net::MeleeActions {
                        physical: None,
                        white: None,
                        ability: None,
                        catalog: Arc::new(theia_core::melee::Catalog::from_json(WEAPONS).unwrap()),
                        choices: melee_rx,
                    }),
                }),
                quest: None,
                progression: None,
                inventory: Some(net::inventory::Client {
                    profile,
                    commands: item_rx,
                }),
                statuses: explicit_rules
                    .then(|| Arc::new(theia_core::statuses::Catalog::from_json(STATUSES).unwrap())),
            },
        )
        .await;
        if let Err(e) = &result {
            publish.send_modify(|s| s.error = Some(e.to_string()));
        }
        result
    });
    let (facing, heading) = watch::channel(Facing::from_units(0));
    let driven_state = state.clone();
    let driver_task = tokio::spawn(async move {
        let mut driver = theia_network::prediction_driver::Driver::with_classic(source, false);
        loop {
            tokio::select! {
                _ = driver_stop.changed() => break,
                _ = tokio::time::sleep(Duration::from_millis(8)) => {
                    let raw = driven_state.borrow().clone();
                    if raw.error.is_some() {break;}
                    let intent = Intent { facing:Some(*heading.borrow()), ..Default::default() };
                    if let Some(driven) = driver.tick(&raw,intent,Some(&patch),std::time::Instant::now()).unwrap()
                        && let Some(packet) = driven.packet {
                        let _ = commands.try_send(packet);
                    }
                }
            }
        }
    });
    let initial = until(&mut state, |s| {
        s.autoattack.is_some() && s.chunk_ack.is_some() && s.correction.is_some()
    })
    .await;
    let c = initial.correction.unwrap().state;
    choices.send_replace(Some(theia_network::targeting::Choice {
        target: Some(target.entity),
    }));
    melee.send_replace(Some(net::MeleeChoice {
        stream_epoch: c.input.stream_epoch,
        target: Some(target),
    }));
    let hit = until(&mut state, |s| {
        s.autoattack.as_ref().is_some_and(|v| v.swing_sequence == 1)
    })
    .await;
    assert_eq!(
        hit.combat
            .as_ref()
            .unwrap()
            .targets
            .iter()
            .find(|h| h.entity == target.entity)
            .unwrap()
            .current,
        85
    );
    if blocked_presentation {
        let error = tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let view = state.borrow().clone();
                if let Some(error) = view.error {
                    break error;
                }
                state.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        assert!(
            error.contains("Combat presentation queue full or closed"),
            "{error}"
        );
        stop.send_replace(true);
        driver_task.await.unwrap();
        assert!(client.await.unwrap().is_err());
        server.await.unwrap().unwrap();
        relay_task.await.unwrap();
        assert_eq!(event_rx.len(), 1);
        std::fs::remove_dir_all(directory).unwrap();
        return;
    }
    facing.send_replace(Facing::from_units(32768));
    let paused = until(&mut state, |s| {
        s.autoattack
            .as_ref()
            .is_some_and(|v| v.phase == Phase::Paused(Pause::NotFacing))
    })
    .await;
    assert_eq!(paused.autoattack.as_ref().unwrap().target, Some(target));
    if explicit_rules {
        cast_requests
            .send(net::CastChoice {
                ability: ability_id.clone(),
                target: target.entity,
            })
            .await
            .unwrap();
        until(&mut state, |s| {
            s.combat.as_ref().and_then(|c| c.reply).is_some_and(|r| {
                r.outcome
                    == theia_network::simulation::combat::Outcome::Rejected(
                        theia_network::simulation::combat::Reason::NotFacing,
                    )
            })
        })
        .await;
        assert_eq!(
            state.borrow().correction.unwrap().state.pose.facing,
            Some(Facing::from_units(32768))
        );
    }
    facing.send_replace(Facing::from_units(0));
    if explicit_rules {
        until(&mut state, |s| {
            s.correction
                .is_some_and(|c| c.state.pose.facing == Some(Facing::from_units(0)))
        })
        .await;
        cast_requests
            .send(net::CastChoice {
                ability: ability_id,
                target: target.entity,
            })
            .await
            .unwrap();
        until(&mut state, |s| {
            s.combat.as_ref().and_then(|c| c.reply).is_some_and(|r| {
                matches!(
                    r.outcome,
                    theia_network::simulation::combat::Outcome::Started { .. }
                )
            })
        })
        .await;
        until(&mut state, |s| {
            s.combat.as_ref().is_some_and(|c| {
                c.cast.is_none()
                    && c.targets
                        .iter()
                        .any(|h| h.entity == target.entity && h.current < 85)
            })
        })
        .await;
    }
    let resumed = until(&mut state, |s| {
        s.autoattack.as_ref().is_some_and(|v| v.swing_sequence >= 2)
    })
    .await;
    let stream = resumed.correction.unwrap().state.input.stream_epoch;
    melee.send_replace(Some(net::MeleeChoice {
        stream_epoch: stream,
        target: None,
    }));
    let stopped = until(&mut state, |s| {
        s.autoattack.as_ref().is_some_and(|v| {
            v.phase == Phase::Idle && v.reply.is_some_and(|r| r.outcome == Outcome::Stopped)
        })
    })
    .await;
    assert_eq!(
        stopped.targeting.as_ref().unwrap().selected,
        Some(target.entity)
    );
    let mut swings = 0;
    while let Ok(batch) = event_rx.try_recv() {
        for e in batch.events {
            if matches!(e, Event::MeleeDamage { .. }) {
                swings += 1;
            }
        }
    }
    assert!(swings >= 2);
    // A mismatched local stream never becomes a valid current-life attack.
    melee.send_replace(Some(net::MeleeChoice {
        stream_epoch: stream + 1,
        target: Some(target),
    }));
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert_eq!(
        state.borrow().autoattack.as_ref().unwrap().phase,
        Phase::Idle
    );
    let _ = &observed;
    stop.send_replace(true);
    driver_task.await.unwrap();
    client.await.unwrap().unwrap();
    let stats = server.await.unwrap().unwrap();
    relay_task.await.unwrap();
    assert_eq!(stats.melee_requests_accepted, 2);
    assert!(stats.melee_states_sent >= 4);
    assert_eq!(stats.combat_backpressure_disconnects, 0);
    let relay = relay_control.lock().unwrap();
    assert!(relay.stats.configured_drops > 0);
    assert!(relay.stats.duplicates > 0);
    std::fs::remove_dir_all(directory).unwrap();
}
