use super::*;
use crate::documents::fixtures::*;
#[tokio::test]
async fn invalid_document_open_requests_refuse_before_queue_acceptance() {
    let p = profiles();
    let who = binding();
    let world = [3; 32];
    let (latest, _latest) = watch::channel(None);
    let (accounts, _accounts) = mpsc::channel(16);
    let (inventory, mut requests) = mpsc::channel(16);
    let publisher = Publisher {
        latest,
        submitted: Arc::new(AtomicU64::new(0)),
        accounts,
        inventory,
    };
    assert!(matches!(
        publisher.enqueue_document_open(who, world, &p.legacy, open(&p.journal)),
        Err(Error::Invalid("Document journal binding required"))
    ));
    assert!(matches!(
        publisher
            .open_quest_document(who, world, &p.legacy, open(&p.journal))
            .await,
        Err(Error::Invalid("Document journal binding required"))
    ));
    for command in invalid(&p.journal) {
        assert!(matches!(
            publisher.enqueue_document_open(who, world, &p.journal, command.clone()),
            Err(Error::DocumentProfileMismatch | Error::DocumentCommand(_))
        ));
        assert!(matches!(
            publisher
                .open_quest_document(who, world, &p.journal, command)
                .await,
            Err(Error::DocumentProfileMismatch | Error::DocumentCommand(_))
        ));
    }
    assert!(matches!(
        requests.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    ));
    assert_eq!(publisher.inventory.capacity(), 16);
}
