use super::*;
use crate::documents::fixtures::*;
#[tokio::test]
async fn document_open_requires_bound_metadata_and_valid_request_before_store_io() {
    let p = profiles();
    let mut store = Store {
        client: None,
        driver: None,
        realm: RealmId::random(),
        session: uuid::Uuid::new_v4(),
    };
    let who = binding();
    let world = [3; 32];
    assert!(matches!(
        store
            .open_quest_document(who, world, &p.legacy, &open(&p.journal))
            .await,
        Err(Error::Invalid("Document journal binding required"))
    ));
    for command in invalid(&p.journal) {
        assert!(matches!(
            store
                .open_quest_document(who, world, &p.journal, &command)
                .await,
            Err(Error::DocumentProfileMismatch | Error::DocumentCommand(_))
        ));
    }
}
