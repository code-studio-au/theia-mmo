//! Opt-in trusted development issuer adapter. SQL never enters World::step.
use std::{future::Future, time::Duration};
use theia_network::{
    Result,
    procedural::Source,
    renet_runtime::{AuthorityCheckpoint, CheckpointBridge},
    simulation::World,
    wire::MAX_PLAYERS,
};
use theia_persistence::{Publisher, RealmId, SavedCharacter, Store, Worker};
use tokio::{sync::watch, time::MissedTickBehavior};
pub struct Prepared {
    store: Store,
    records: [SavedCharacter; MAX_PLAYERS],
    bridge: CheckpointBridge,
    source: watch::Receiver<Option<AuthorityCheckpoint>>,
    epoch: u64,
    seeded: usize,
    restored: usize,
}
pub async fn prepare(world: &mut World) -> Result<Prepared> {
    let runtime = world
        .procedural()
        .ok_or("Persistent development requires saved terrain")?;
    if !matches!(&runtime.source, Source::Saved(_)) || runtime.speed != 1. || world.dev_tour {
        return Err(
            "Persistent development requires saved terrain, normal speed and no tour".into(),
        );
    }
    let identity = runtime.source.identity();
    let realm = RealmId::parse(
        &std::env::var("THEIA_DEV_REALM_ID")
            .unwrap_or_else(|_| "74686569-6100-0000-0000-000000000001".into()),
    )
    .ok_or("Invalid development realm UUID")?;
    let url = std::env::var("THEIA_DATABASE_URL")
        .map_err(|_| "Development persistence requires THEIA_DATABASE_URL")?;
    let mut store = Store::connect_loopback(&url, realm).await?;
    let mut records = Vec::with_capacity(MAX_PLAYERS);
    let mut seeded = 0;
    let mut restored = 0;
    for slot in 1..=MAX_PLAYERS as u8 {
        let binding = store.development_binding(slot).await?;
        if let Some(record) = store.load(binding, identity).await? {
            records.push(record);
            restored += 1;
        } else {
            let position = world
                .development_player_spawn(slot)
                .ok_or("Development character spawn lacks prepared support")?;
            world.validate_restored_player(position)?;
            records.push(store.seed(binding, identity, position).await?);
            seeded += 1;
        }
    }
    let positions: Vec<_> = records.iter().map(|r| r.position).collect();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            world.poll_terrain_keeping(&positions)?;
            if positions.iter().all(|p| world.terrain_prepared_for(*p)) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        for position in &positions {
            world.validate_restored_player(*position)?;
        }
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await
    .map_err(|_| "Persistent character collision preparation timed out")??;
    let records: [SavedCharacter; MAX_PLAYERS] = records
        .try_into()
        .map_err(|_| "Persistent binding capacity mismatch")?;
    let (bridge, source) = CheckpointBridge::channel(records.map(|r| Some(r.position)));
    println!(
        "persistence_prepared realm={} seeded_records={seeded} loaded_records={restored} issuer=development-local",
        realm.value()
    );
    Ok(Prepared {
        store,
        records,
        bridge,
        source,
        epoch: world.epoch(),
        seeded,
        restored,
    })
}
impl Prepared {
    pub fn into_parts(
        self,
        shutdown: watch::Sender<bool>,
    ) -> (CheckpointBridge, impl Future<Output = Result<()>>) {
        let Self {
            store,
            records,
            bridge,
            source,
            epoch,
            seeded,
            restored,
        } = self;
        let (publisher, worker) = Worker::start(store);
        let evidence = Evidence {
            seeded_records: seeded,
            loaded_records: restored,
            source_epoch: epoch,
            ..Default::default()
        };
        (
            bridge,
            manage(records, source, evidence, publisher, worker, shutdown),
        )
    }
}
#[derive(serde::Serialize, Default)]
struct Evidence {
    seeded_records: usize,
    loaded_records: usize,
    actual_actor_slots: usize,
    source_tick: u64,
    source_epoch: u64,
    publications: u64,
}
async fn manage(
    mut records: [SavedCharacter; MAX_PLAYERS],
    mut source: watch::Receiver<Option<AuthorityCheckpoint>>,
    mut evidence: Evidence,
    publisher: Publisher,
    worker: Worker,
    shutdown: watch::Sender<bool>,
) -> Result<()> {
    let mut status = worker.subscribe_status();
    let mut sequence = records.iter().map(|r| r.sequence).max().unwrap_or(0);
    let mut seen = [false; MAX_PLAYERS];
    let mut dirty = false;
    let mut cadence = tokio::time::interval(Duration::from_secs(1));
    cadence.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let result:Result<()>=async {
        loop {
            tokio::select! {biased;
                changed=status.changed()=>{
                    if changed.is_err() || status.borrow().failed {return Err("Persistent checkpoint worker failed".into());}
                }
                changed=source.changed()=>{
                    let snapshot={*source.borrow_and_update()};
                    if let Some(snapshot)=snapshot {
                        if snapshot.epoch!=evidence.source_epoch || snapshot.tick<evidence.source_tick {return Err("Authority checkpoint identity/tick mismatch".into());}
                        evidence.source_tick=snapshot.tick;
                        for (i,actor) in snapshot.slots.iter().enumerate() {
                            if let Some(actor)=actor {
                                if actor.actor.epoch.value()!=evidence.source_epoch {return Err("Authority checkpoint actor epoch mismatch".into());}
                                dirty|=!seen[i] || records[i].position!=actor.position;
                                seen[i]=true;records[i].position=actor.position;
                            }
                        }
                    }
                    if changed.is_err() {break;}
                }
                _=cadence.tick()=>{
                    if dirty {publish(&mut records,&seen,&mut sequence,&publisher)?;dirty=false;}
                }
            }
        }
        if dirty {publish(&mut records,&seen,&mut sequence,&publisher)?;}
        Ok(())
    }.await;
    if result.is_err() {
        shutdown.send_replace(true);
    }
    let stored = worker.shutdown().await;
    evidence.actual_actor_slots = seen.iter().filter(|s| **s).count();
    evidence.publications = publisher.submissions();
    let positions:Vec<_>=records.iter().enumerate().filter(|(i,_)|seen[*i]).map(|(i,r)|serde_json::json!({"client_id":i+1,"position":r.position.metres(),"sequence":r.sequence})).collect();
    println!(
        "persistence_summary {}",
        serde_json::json!({"source":evidence,"positions":positions,"worker":stored.as_ref().ok(),"failed":result.is_err() || stored.is_err()})
    );
    result?;
    stored?;
    Ok(())
}
fn publish(
    records: &mut [SavedCharacter; MAX_PLAYERS],
    seen: &[bool; MAX_PLAYERS],
    sequence: &mut u64,
    publisher: &Publisher,
) -> Result<()> {
    *sequence = sequence
        .checked_add(1)
        .filter(|n| *n <= i64::MAX as u64)
        .ok_or("Persistent checkpoint sequence exhausted")?;
    let mut batch = Vec::with_capacity(MAX_PLAYERS);
    for (i, record) in records.iter_mut().enumerate() {
        if seen[i] {
            record.sequence = *sequence;
            batch.push(*record);
        }
    }
    if !batch.is_empty() {
        publisher.publish(&batch)?;
    }
    Ok(())
}
