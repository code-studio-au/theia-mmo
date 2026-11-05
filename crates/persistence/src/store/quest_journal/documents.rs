//! First committed authorized opens share the existing quest-event receipts.
//! Successful retries recover the original view, never a fresh-stage read.
use super::*;
use crate::DocumentOpenCommand;
fn receipt_id(command: &DocumentOpenCommand) -> NonZeroU128 {
    let mut bytes = b"theia-document-open-request-v1\0".to_vec();
    bytes.extend(command.request_id.get().to_le_bytes());
    let hash = ring::digest::digest(&ring::digest::SHA256, &bytes);
    NonZeroU128::new(u128::from_le_bytes(hash.as_ref()[..16].try_into().unwrap()).max(1)).unwrap()
}
fn command_hash(catalog: &QuestJournalCatalog, command: &DocumentOpenCommand) -> [u8; 32] {
    let mut bytes = b"theia-document-open-command-v1\0".to_vec();
    bytes.extend(catalog.hash());
    bytes.extend(command.profile_hash);
    bytes.extend(command.request_id.get().to_le_bytes());
    bytes.extend(command.expected_revision.to_le_bytes());
    bytes.push(command.slot);
    bytes.extend((command.document.as_str().len() as u16).to_le_bytes());
    bytes.extend(command.document.as_str().as_bytes());
    ring::digest::digest(&ring::digest::SHA256, &bytes)
        .as_ref()
        .try_into()
        .unwrap()
}
impl Store {
    /// Caller establishes session entitlement. Ok authorizes this exact original
    /// document open after COMMIT; it is not proof of human attention. The caller
    /// retains the request ID/document/profile when constructing its view reply.
    pub async fn open_quest_document(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        command: &DocumentOpenCommand,
    ) -> Result<QuestJournalApplied> {
        let documents = catalog.check_document_binding()?;
        documents.validate_open(command)?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let result = apply_event_with_bag(
            &tx,
            self.realm,
            binding,
            world,
            catalog,
            receipt_id(command),
            |journal, bag| {
                let bag = bag.ok_or(Error::QuestJournalData)?;
                documents
                    .catalog()
                    .resolve_owned(
                        bag,
                        command.expected_revision,
                        command.slot,
                        &command.document,
                    )
                    .map_err(Error::DocumentCommand)?;
                let observation = journal
                    .prepare_document_read(
                        catalog.catalog(),
                        documents.catalog(),
                        &command.document,
                    )
                    .map_err(Error::QuestJournalCommand)?;
                journal
                    .propose_document_read(catalog.catalog(), documents.catalog(), &observation)
                    .map_err(Error::QuestJournalCommand)
            },
            command_hash(catalog, command),
        )
        .await?;
        bounded(tx.commit()).await?;
        Ok(result)
    }
}
