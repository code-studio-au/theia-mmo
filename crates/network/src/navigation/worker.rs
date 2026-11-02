//! One prepared local navigation package, one outstanding query, one reply.
//! No file/transport/SQL access and no waiting or searching on the owner thread.
//! Landmass has no hard CPU deadline; late replies fail publication instead.
use super::{MAX_ROUTE_POINTS, Store, query::Query};
use crate::Result;
use std::sync::{Mutex, mpsc};
use theia_core::{coordinates::GlobalPosition, identity::EntityKey, player_life::LifeKey};

pub const MAX_REQUEST_TICKS: u64 = 120;

/// Caller increments revision whenever the destination or engagement changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag {
    pub creature: EntityKey,
    pub target: Option<LifeKey>,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Request {
    pub tag: Tag,
    pub start: GlobalPosition,
    pub end: GlobalPosition,
    pub submitted: u64,
    pub expires: u64,
}
impl Request {
    fn valid(self) -> bool {
        self.tag.revision != 0
            && self
                .tag
                .target
                .is_none_or(|t| t.actor.epoch == self.tag.creature.epoch)
            && self
                .expires
                .checked_sub(self.submitted)
                .is_some_and(|age| (1..=MAX_REQUEST_TICKS).contains(&age))
    }
}

#[derive(Debug)]
pub struct Reply {
    pub navigation: [u8; 32],
    pub request: Request,
    pub route: std::result::Result<Vec<GlobalPosition>, String>,
}
impl Reply {
    /// Only exposes successful points for the exact still-current engagement,
    /// navigation incarnation and simulation-time window. Consumers must also
    /// retain collision/grounding authority while following these points.
    pub fn current(&self, navigation: [u8; 32], tag: Tag, now: u64) -> Option<&[GlobalPosition]> {
        (self.navigation == navigation
            && self.request.tag == tag
            && self.request.valid()
            && (self.request.submitted..=self.request.expires).contains(&now))
        .then(|| self.route.as_deref().ok())
        .flatten()
        .filter(|p| (2..=MAX_ROUTE_POINTS).contains(&p.len()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitError {
    Busy,
    Invalid,
    Closed,
}

pub struct Worker {
    navigation: [u8; 32],
    sender: mpsc::SyncSender<Request>,
    receiver: Mutex<mpsc::Receiver<Reply>>,
    pending: Option<Request>,
}
impl Worker {
    /// Startup/loader work only. Reads and validates the bounded package before
    /// constructing its static query. Never call this from World::step.
    pub fn load(store: &Store) -> Result<Self> {
        let tiles = store
            .manifest()
            .tiles
            .iter()
            .map(|tile| store.tile(tile.id))
            .collect::<Result<Vec<_>>>()?;
        Self::spawn(Query::new(&tiles)?, store.identity())
    }
    pub fn spawn(query: Query, navigation: [u8; 32]) -> Result<Self> {
        let (sender, jobs) = mpsc::sync_channel::<Request>(1);
        let (results, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("creature-path".into())
            .spawn(move || {
                while let Ok(request) = jobs.recv() {
                    let route = query
                        .route(request.start, request.end)
                        .map_err(|e| e.to_string());
                    if results
                        .send(Reply {
                            navigation,
                            request,
                            route,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })?;
        Ok(Self {
            navigation,
            sender,
            receiver: Mutex::new(receiver),
            pending: None,
        })
    }
    pub fn navigation(&self) -> [u8; 32] {
        self.navigation
    }
    pub fn pending(&self) -> Option<Request> {
        self.pending
    }
    /// Admission is nonblocking. The single pending slot includes queued,
    /// computing and completed-but-unpolled work, so replies cannot accumulate.
    pub fn submit(&mut self, request: Request) -> std::result::Result<(), SubmitError> {
        if !request.valid() {
            return Err(SubmitError::Invalid);
        }
        if self.pending.is_some() {
            return Err(SubmitError::Busy);
        }
        self.sender.try_send(request).map_err(|error| match error {
            mpsc::TrySendError::Full(_) => SubmitError::Busy,
            mpsc::TrySendError::Disconnected(_) => SubmitError::Closed,
        })?;
        self.pending = Some(request);
        Ok(())
    }
    pub fn poll(&mut self) -> std::result::Result<Option<Reply>, SubmitError> {
        match self.receiver.get_mut().unwrap().try_recv() {
            Ok(reply) => {
                let expected = self.pending.take();
                if expected != Some(reply.request) || reply.navigation != self.navigation {
                    return Err(SubmitError::Invalid);
                }
                Ok(Some(reply))
            }
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                Err(SubmitError::Closed)
            }
        }
    }
}
