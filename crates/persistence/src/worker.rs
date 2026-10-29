//! Latest-only ordinary checkpoints and a bounded account-command queue. SQL
//! belongs to this asynchronous owner;
//! producers copy at most eight records and never await a database operation.
mod accounts;
use crate::{Error, Result, Saved, SavedCharacter, Store, store::MAX_BATCH};
use accounts::Request;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
#[derive(Clone)]
struct Batch {
    generation: u64,
    records: [Option<SavedCharacter>; MAX_BATCH],
}
impl Batch {
    fn records(&self) -> Vec<SavedCharacter> {
        self.records.iter().flatten().copied().collect()
    }
}
#[derive(Clone)]
pub struct Publisher {
    latest: watch::Sender<Option<Batch>>,
    submitted: Arc<AtomicU64>,
    accounts: mpsc::Sender<Request>,
}
impl Publisher {
    /// Queue acceptance only. Successful publication never means database commit.
    pub fn publish(&self, records: &[SavedCharacter]) -> Result<u64> {
        if records.is_empty() || records.len() > MAX_BATCH {
            return Err(Error::Invalid("Checkpoint batch outside bounded capacity"));
        }
        if self.latest.is_closed() {
            return Err(Error::Invalid("Persistence worker is closed"));
        }
        // Validate the bounded copy before changing the mailbox/generation.
        let mut array = [None; MAX_BATCH];
        for (i, record) in records.iter().enumerate() {
            if record.sequence == 0
                || record.sequence > i64::MAX as u64
                || record.world == [0; 32]
                || records[..i]
                    .iter()
                    .any(|old| old.binding.character == record.binding.character)
            {
                return Err(Error::Invalid("Invalid or duplicate checkpoint record"));
            }
            array[i] = Some(*record);
        }
        let mut generation = 0;
        // Serialize generation allocation with replacement: concurrent producers
        // cannot overwrite a newer generation after allocating an older one.
        self.latest.send_modify(|slot| {
            let next = slot.as_ref().map_or(0, |b| b.generation).checked_add(1);
            if let Some(next) = next {
                generation = next;
                *slot = Some(Batch {
                    generation,
                    records: array,
                });
            }
        });
        if generation == 0 {
            return Err(Error::Invalid("Checkpoint mailbox generation exhausted"));
        }
        self.submitted.fetch_add(1, Ordering::Relaxed);
        Ok(generation)
    }
    pub fn submissions(&self) -> u64 {
        self.submitted.load(Ordering::Relaxed)
    }
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct WorkerStatus {
    pub completed_generation: u64,
    pub processed_batches: u64,
    pub committed_records: u64,
    pub stale_records: u64,
    pub coalesced_batches: u64,
    pub resolved_owners: u64,
    pub processed_account_requests: u64,
    pub rejected_account_requests: u64,
    pub created_characters: u64,
    pub character_lists: u64,
    pub character_selections: u64,
    pub failed: bool,
    pub error: Option<String>,
}
pub struct Worker {
    stop: watch::Sender<bool>,
    status: watch::Receiver<WorkerStatus>,
    task: Option<JoinHandle<Result<WorkerStatus>>>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.send_replace(true);
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
impl Worker {
    pub fn start(store: Store) -> (Publisher, Self) {
        let (latest, receiver) = watch::channel(None);
        let (stop, shutdown) = watch::channel(false);
        let (report, status) = watch::channel(WorkerStatus::default());
        let (accounts, requests) = mpsc::channel(16);
        let task = tokio::spawn(run(store, receiver, requests, shutdown, report));
        (
            Publisher {
                latest,
                submitted: Arc::new(AtomicU64::new(0)),
                accounts,
            },
            Self {
                stop,
                status,
                task: Some(task),
            },
        )
    }
    pub fn subscribe_status(&self) -> watch::Receiver<WorkerStatus> {
        self.status.clone()
    }
    pub fn status(&self) -> WorkerStatus {
        self.status.borrow().clone()
    }
    /// Flush the most recent accepted snapshot and close the owning connection.
    /// Call after stopping publication, outside any simulation step.
    pub async fn shutdown(mut self) -> Result<WorkerStatus> {
        self.stop.send_replace(true);
        let result = self
            .task
            .as_mut()
            .unwrap()
            .await
            .map_err(|_| Error::Invalid("Persistence worker stopped unexpectedly"))?;
        self.task.take();
        result
    }
}
async fn apply(
    store: &mut Store,
    batch: Option<Batch>,
    status: &mut WorkerStatus,
    report: &watch::Sender<WorkerStatus>,
) -> Result<()> {
    let Some(batch) = batch else {
        return Ok(());
    };
    if batch.generation <= status.completed_generation {
        return Ok(());
    }
    // No watch borrow is retained across SQL/COMMIT.
    match store.checkpoint(&batch.records()).await {
        Ok(outcomes) => {
            status.coalesced_batches = status
                .coalesced_batches
                .saturating_add(batch.generation - status.completed_generation - 1);
            status.completed_generation = batch.generation;
            status.processed_batches = status.processed_batches.saturating_add(1);
            status.committed_records = status
                .committed_records
                .saturating_add(outcomes.iter().filter(|o| **o == Saved::Committed).count() as u64);
            status.stale_records = status
                .stale_records
                .saturating_add(outcomes.iter().filter(|o| **o == Saved::Stale).count() as u64);
            report.send_replace(status.clone());
            Ok(())
        }
        Err(error) => {
            status.failed = true;
            status.error = Some(error.to_string());
            report.send_replace(status.clone());
            Err(error)
        }
    }
}
async fn run(
    mut store: Store,
    mut latest: watch::Receiver<Option<Batch>>,
    mut requests: mpsc::Receiver<Request>,
    mut stop: watch::Receiver<bool>,
    report: watch::Sender<WorkerStatus>,
) -> Result<WorkerStatus> {
    let mut status = WorkerStatus::default();
    loop {
        // Check stop before another job; fair selection between account requests
        // and superseding checkpoints prevents either queue monopolizing SQL.
        let ending = if *stop.borrow() {
            true
        } else {
            tokio::select! {
                _=stop.changed()=>true,
                error=store.connection_stopped()=>{
                    status.failed=true;status.error=Some(error.to_string());report.send_replace(status);
                    return Err(error);
                }
                request=requests.recv()=>match request {
                    Some(request)=>{
                        let result=request.run(&mut store,&mut status).await;
                        report.send_replace(status.clone());
                        result?;
                        continue;
                    }
                    None=>true,
                },
                result=latest.changed()=>result.is_err(),
            }
        };
        let batch = { latest.borrow_and_update().clone() };
        apply(&mut store, batch, &mut status, &report).await?;
        if ending {
            break;
        }
    }
    if let Err(error) = store.close().await {
        status.failed = true;
        status.error = Some(error.to_string());
        report.send_replace(status);
        return Err(error);
    }
    Ok(status)
}
