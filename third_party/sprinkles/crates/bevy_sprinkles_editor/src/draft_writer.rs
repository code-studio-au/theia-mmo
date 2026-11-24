//! Bounded background writer for particle draft snapshots.
use std::collections::HashMap;
use std::io;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::{Duration, Instant};

use crate::drafts::{Draft, DraftId, DraftStore};

const MAX_QUEUED: usize = 8;
const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;

enum Job {
    Capture { session: u64, draft: Draft },
    Flush(mpsc::Sender<()>),
}

/// Completion is reported to the editor; an accepted job is not yet durable.
pub struct CaptureResult {
    pub session: u64,
    pub result: Result<DraftId, String>,
}

pub struct DraftWriter {
    sender: SyncSender<Job>,
    results: Receiver<CaptureResult>,
}

impl DraftWriter {
    pub fn start(store: DraftStore) -> io::Result<Self> {
        let (sender, jobs) = mpsc::sync_channel::<Job>(MAX_QUEUED);
        let (finished, results) = mpsc::channel();
        std::thread::Builder::new()
            .name("particle-draft-writer".into())
            .spawn(move || {
                let mut sessions: HashMap<u64, DraftId> = HashMap::new();
                while let Ok(job) = jobs.recv() {
                    match job {
                        Job::Flush(done) => {
                            let _ = done.send(());
                        }
                        Job::Capture { session, draft } => {
                            let result = match sessions.get(&session) {
                                Some(id) => match store.update(id, &draft) {
                                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                                        store.create(&draft)
                                    }
                                    result => result.map(|()| id.clone()),
                                },
                                None => store.create(&draft),
                            };
                            if let Ok(id) = &result {
                                sessions.insert(session, id.clone());
                            }
                            if finished
                                .send(CaptureResult {
                                    session,
                                    result: result.map_err(|error| error.to_string()),
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
            })?;
        Ok(Self { sender, results })
    }

    /// Never blocks the editor. A full queue leaves the caller responsible for retrying.
    pub fn capture(&self, session: u64, draft: Draft) -> io::Result<()> {
        if draft.snapshot.len() > MAX_SNAPSHOT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Draft snapshot is too large",
            ));
        }
        self.sender
            .try_send(Job::Capture { session, draft })
            .map_err(|error| match error {
                TrySendError::Full(_) => {
                    io::Error::new(io::ErrorKind::WouldBlock, "Draft writer is busy")
                }
                TrySendError::Disconnected(_) => {
                    io::Error::new(io::ErrorKind::BrokenPipe, "Draft writer stopped")
                }
            })
    }

    /// Wait a bounded time for jobs already accepted by the writer.
    pub fn flush(&self, timeout: Duration) -> io::Result<()> {
        let deadline = Instant::now() + timeout;
        let (done, receiver) = mpsc::channel();
        let mut job = Job::Flush(done);
        loop {
            match self.sender.try_send(job) {
                Ok(()) => break,
                Err(TrySendError::Full(returned)) if Instant::now() < deadline => {
                    job = returned;
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(TrySendError::Full(_)) => {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Draft writer queue did not drain",
                    ));
                }
                Err(TrySendError::Disconnected(_)) => {
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Draft writer stopped",
                    ));
                }
            }
        }
        receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Draft writer did not finish"))
    }

    pub fn try_result(&self) -> Option<CaptureResult> {
        self.results.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    #[test]
    fn writer_reports_failures_without_changing_the_target() {
        let dir = std::env::temp_dir().join(format!(
            "theia-draft-failure-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let occupied = dir.join("not-a-directory");
        std::fs::write(&occupied, b"keep").unwrap();
        let writer = DraftWriter::start(DraftStore::new(occupied.clone())).unwrap();
        writer
            .capture(7, Draft::new(None, "snapshot".into()))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let result = loop {
            assert!(Instant::now() < deadline, "draft writer timed out");
            if let Some(result) = writer.try_result() {
                break result;
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        assert_eq!(result.session, 7);
        assert!(result.result.is_err());
        assert_eq!(std::fs::read(occupied).unwrap(), b"keep");
        drop(writer);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn writer_serializes_updates_per_session_without_touching_project_files() {
        let dir = std::env::temp_dir().join(format!(
            "theia-draft-writer-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("effect.ron");
        std::fs::write(&source, b"original").unwrap();
        let store = DraftStore::new(dir.join("drafts"));
        let writer = DraftWriter::start(store.clone()).unwrap();
        writer
            .capture(1, Draft::new(Some(source.clone()), "first".into()))
            .unwrap();
        writer
            .capture(1, Draft::new(Some(source.clone()), "latest".into()))
            .unwrap();
        writer
            .capture(2, Draft::new(None, "untitled".into()))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut results = Vec::new();
        while results.len() < 3 {
            assert!(Instant::now() < deadline, "draft writer timed out");
            if let Some(result) = writer.try_result() {
                results.push(result);
            } else {
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        assert_eq!(
            results
                .iter()
                .map(|result| result.session)
                .collect::<Vec<_>>(),
            vec![1, 1, 2]
        );
        let first = results.remove(0).result.unwrap();
        let latest = results.remove(0).result.unwrap();
        let other = results.remove(0).result.unwrap();
        assert_eq!(first, latest);
        assert_ne!(latest, other);
        assert_eq!(store.load(&latest).unwrap().snapshot, "latest");
        assert_eq!(store.load(&other).unwrap().snapshot, "untitled");
        store.discard(&latest).unwrap();
        writer
            .capture(1, Draft::new(Some(source.clone()), "after discard".into()))
            .unwrap();
        assert!(writer.flush(Duration::from_secs(10)).is_ok());
        let recreated = writer.try_result().unwrap().result.unwrap();
        assert_ne!(recreated, latest);
        assert_eq!(store.load(&recreated).unwrap().snapshot, "after discard");
        assert_eq!(std::fs::read(source).unwrap(), b"original");
        drop(writer);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
