//! Purchases share the existing sixteen-job critical item queue and worker.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{Binding, Error, InventoryApplied, Result, Store, VendorCatalog};
use theia_core::{
    inventory::MAX_REVISION,
    vendors::{Error as CommandError, Purchase},
};
use tokio::sync::oneshot;
pub(super) struct Request {
    binding: Binding,
    world: [u8; 32],
    profile: VendorCatalog,
    purchase: Purchase,
    reply: oneshot::Sender<Result<InventoryApplied>>,
}
impl Publisher {
    /// Queue acceptance is not purchase success. Caller retains accepted work
    /// through retirement; once executing, a dropped answer does not cancel SQL.
    pub async fn purchase_vendor(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &VendorCatalog,
        purchase: &Purchase,
    ) -> Result<InventoryApplied> {
        valid_world(world)?;
        if purchase.expected_revision > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        if profile.catalog().offer(&purchase.offer).is_none() {
            return Err(Error::VendorCommand(CommandError::UnknownOffer));
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Vendor(Request {
                binding,
                world,
                profile: profile.clone(),
                purchase: purchase.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
}
impl Request {
    pub(super) async fn run(self, store: &mut Store, status: &mut WorkerStatus) -> Result<()> {
        if self.reply.is_closed() {
            return Ok(());
        }
        let result = store
            .purchase_vendor(self.binding, self.world, &self.profile, &self.purchase)
            .await;
        if let Ok(applied) = &result {
            status.vendor_committed = status
                .vendor_committed
                .saturating_add(u64::from(!applied.replayed));
            status.vendor_replayed = status
                .vendor_replayed
                .saturating_add(u64::from(applied.replayed));
            status.inventory_committed = status
                .inventory_committed
                .saturating_add(u64::from(!applied.replayed));
            status.inventory_replayed = status
                .inventory_replayed
                .saturating_add(u64::from(applied.replayed));
        } else {
            status.vendor_rejected = status.vendor_rejected.saturating_add(1);
        }
        respond(result, self.reply, status)
    }
}
