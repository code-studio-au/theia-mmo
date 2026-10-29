//! Typed bounded account/character commands; never the superseding pose mailbox.
use super::{Publisher, WorkerStatus};
use crate::{
    Binding, Character, CharacterName, CharacterSummary, Error, OwnerId, Result, Store,
    store::validate_identity,
};
use theia_core::coordinates::GlobalPosition;
use tokio::sync::oneshot;
pub(super) enum Request {
    Owner {
        issuer: String,
        subject: String,
        reply: oneshot::Sender<Result<OwnerId>>,
    },
    Create {
        owner: OwnerId,
        world: [u8; 32],
        name: CharacterName,
        position: GlobalPosition,
        reply: oneshot::Sender<Result<Character>>,
    },
    List {
        owner: OwnerId,
        world: [u8; 32],
        reply: oneshot::Sender<Result<Vec<CharacterSummary>>>,
    },
    Select {
        binding: Binding,
        world: [u8; 32],
        reply: oneshot::Sender<Result<Character>>,
    },
}
#[derive(Clone, Copy)]
enum Operation {
    Owner,
    Create,
    List,
    Select,
}
impl Publisher {
    /// Trusted caller must use a verified provider identity. Success means COMMIT.
    pub async fn resolve_owner(&self, issuer: &str, subject: &str) -> Result<OwnerId> {
        validate_identity(issuer, subject)?;
        let (reply, answer) = oneshot::channel();
        self.account_request(
            Request::Owner {
                issuer: issuer.to_owned(),
                subject: subject.to_owned(),
                reply,
            },
            answer,
        )
        .await
    }
    /// Owner/spawn/world are trusted server inputs, never client claims.
    pub async fn create_character(
        &self,
        owner: OwnerId,
        world: [u8; 32],
        name: CharacterName,
        position: GlobalPosition,
    ) -> Result<Character> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.account_request(
            Request::Create {
                owner,
                world,
                name,
                position,
                reply,
            },
            answer,
        )
        .await
    }
    pub async fn list_characters(
        &self,
        owner: OwnerId,
        world: [u8; 32],
    ) -> Result<Vec<CharacterSummary>> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.account_request(
            Request::List {
                owner,
                world,
                reply,
            },
            answer,
        )
        .await
    }
    /// The binding owner must come from a valid server session. This does not
    /// claim a live character session, prepare collision or issue a game ticket.
    pub async fn select_character(&self, binding: Binding, world: [u8; 32]) -> Result<Character> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.account_request(
            Request::Select {
                binding,
                world,
                reply,
            },
            answer,
        )
        .await
    }
    async fn account_request<T: Send>(
        &self,
        request: Request,
        answer: oneshot::Receiver<Result<T>>,
    ) -> Result<T> {
        self.accounts
            .try_send(request)
            .map_err(|_| Error::Invalid("Account request queue full or closed"))?;
        tokio::time::timeout(std::time::Duration::from_secs(5), answer)
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|_| Error::Invalid("Account request worker closed"))?
    }
}
fn valid_world(world: [u8; 32]) -> Result<()> {
    if world == [0; 32] {
        Err(Error::Invalid("World content identity must be nonzero"))
    } else {
        Ok(())
    }
}
impl Request {
    pub(super) async fn run(self, store: &mut Store, status: &mut WorkerStatus) -> Result<()> {
        match self {
            Self::Owner {
                issuer,
                subject,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                respond(
                    store.resolve_owner(&issuer, &subject).await,
                    reply,
                    status,
                    Operation::Owner,
                )
            }
            Self::Create {
                owner,
                world,
                name,
                position,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                respond(
                    store.create_character(owner, world, name, position).await,
                    reply,
                    status,
                    Operation::Create,
                )
            }
            Self::List {
                owner,
                world,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                respond(
                    store.list_characters(owner, world).await,
                    reply,
                    status,
                    Operation::List,
                )
            }
            Self::Select {
                binding,
                world,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                respond(
                    store.select_character(binding, world).await,
                    reply,
                    status,
                    Operation::Select,
                )
            }
        }
    }
}
fn respond<T>(
    result: Result<T>,
    reply: oneshot::Sender<Result<T>>,
    status: &mut WorkerStatus,
    operation: Operation,
) -> Result<()> {
    status.processed_account_requests = status.processed_account_requests.saturating_add(1);
    let fatal = match &result {
        Ok(_) => {
            let counter = match operation {
                Operation::Owner => &mut status.resolved_owners,
                Operation::Create => &mut status.created_characters,
                Operation::List => &mut status.character_lists,
                Operation::Select => &mut status.character_selections,
            };
            *counter = counter.saturating_add(1);
            false
        }
        Err(
            Error::NameUnavailable
            | Error::CharacterLimit
            | Error::CharacterMissing
            | Error::AccountMissing
            | Error::Ownership
            | Error::WorldMismatch,
        ) => {
            status.rejected_account_requests = status.rejected_account_requests.saturating_add(1);
            false
        }
        Err(error) => {
            status.failed = true;
            status.error = Some(error.to_string());
            true
        }
    };
    let _ = reply.send(result);
    if fatal {
        Err(Error::Invalid("Persistent account operation failed"))
    } else {
        Ok(())
    }
}
