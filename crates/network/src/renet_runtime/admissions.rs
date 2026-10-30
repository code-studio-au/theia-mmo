//! Bounded synchronous account-admission boundary. Implementations own identity,
//! sessions and async persistence; no provider or database work enters this loop.
use crate::Result;
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};

/// A consumed, server-authorized reservation. Dropping a never-joined lease frees
/// it; dropping a recorded lease must retain its final pose until durable commit.
pub trait AdmissionLease: Send + Sync {
    fn live(&self) -> Result<bool>;
    fn record(&self, actor: EntityKey, position: GlobalPosition, tick: u64) -> Result<()>;
    fn progression(&self) -> Result<Option<super::progression::Port>> {
        Ok(None)
    }
    fn inventory(&self) -> Result<Option<super::inventory::Port>> {
        Ok(None)
    }
}
pub struct PlayerAdmission {
    pub position: GlobalPosition,
    pub lease: Box<dyn AdmissionLease>,
}
pub trait AccountAdmissions: Send + Sync {
    /// None is an expected rejected/expired/replayed ticket; Err fails authority.
    /// `data` comes only from the encrypted transport, never a gameplay message.
    fn consume(
        self: std::sync::Arc<Self>,
        client_id: u64,
        data: &[u8; 256],
    ) -> Result<Option<PlayerAdmission>>;
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; super::MAX_PLAYERS]>;
}
pub(super) struct Pending {
    pub admission: PlayerAdmission,
    pub deadline: tokio::time::Instant,
}
pub(super) fn capture(world: &super::World, peer: &super::Peer) -> Result<()> {
    if let Some(lease) = &peer.admission {
        lease.record(
            world
                .entity_key(peer.actor)
                .ok_or("Admission actor missing")?,
            world
                .global_position(peer.actor)
                .ok_or("Admission pose missing")?,
            world.completed_tick(),
        )?;
    }
    Ok(())
}
