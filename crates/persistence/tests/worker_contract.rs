//! Actual row locks and backend loss, never a mocked database or actor count.
mod support;
use std::{
    task::Poll,
    time::{Duration, Instant},
};
use theia_core::coordinates::GlobalPosition;
use theia_persistence::{Binding, CharacterId, OwnerId, RealmId, Store, Worker};
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn stalled_database_coalesces_bounded_jobs_flushes_latest_and_reports_backend_loss() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let binding = Binding {
        character: CharacterId::random(),
        owner: OwnerId::random(),
    };
    let world = [71; 32];
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let initial = store
        .seed(
            binding,
            world,
            GlobalPosition::new([8060., -17., 8000.]).unwrap(),
        )
        .await
        .unwrap();
    let transaction = admin.transaction().await.unwrap();
    transaction.query_one("SELECT character_id FROM theia.characters WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await.unwrap();
    let (publisher, worker) = Worker::start(store);
    let mut record = initial;
    record.sequence = 1;
    publisher.publish(&[record]).unwrap();
    let deadline = Instant::now() + Duration::from_millis(350);
    loop {
        let blocked:bool=transaction.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE application_name='theia-development-persistence' AND wait_event_type='Lock')",&[]).await.unwrap().get(0);
        if blocked {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Worker never reached real row lock"
        );
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    let started = Instant::now();
    for sequence in 2..=10_001 {
        record.sequence = sequence;
        record.position =
            GlobalPosition::new([8060. + sequence as f64 / 1000., -17., 8000.]).unwrap();
        publisher.publish(&[record]).unwrap();
    }
    assert!(
        started.elapsed() < Duration::from_millis(250),
        "Producer was blocked by SQL"
    );
    assert_eq!(publisher.submissions(), 10_001);
    assert_eq!(
        worker.status().completed_generation,
        0,
        "Queued data cannot be reported committed"
    );
    // The in-flight checkpoint is blocked by the real row lock. Fill the fixed
    // account request queue without awaiting SQL, then reject the seventeenth.
    let mut owner_requests: Vec<_> = (0..16)
        .map(|_| {
            Box::pin(publisher.resolve_owner("http://127.0.0.1:40123/worker", "synthetic-subject"))
        })
        .collect();
    for request in &mut owner_requests {
        std::future::poll_fn(|cx| {
            assert!(request.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
    }
    assert!(
        publisher
            .resolve_owner("http://127.0.0.1:40123/worker", "overflow")
            .await
            .is_err()
    );
    drop(owner_requests.pop()); // Cancelled caller must not become a detached job.
    transaction.commit().await.unwrap();
    let mut expected_owner = None;
    for request in owner_requests {
        let owner = request.await.unwrap();
        if let Some(expected) = expected_owner {
            assert_eq!(owner, expected);
        } else {
            expected_owner = Some(owner);
        }
    }
    let status = worker.shutdown().await.unwrap();
    assert_eq!(status.resolved_owners, 15);

    assert!(!status.failed);
    assert_eq!(status.completed_generation, 10_001);
    assert_eq!(status.processed_batches, 2);
    assert_eq!(status.coalesced_batches, 9_999);
    assert_eq!(status.committed_records, 2);
    assert_eq!(status.stale_records, 0);
    assert!(
        publisher.publish(&[record]).is_err(),
        "Closed worker must reject new publications"
    );
    let store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(store.load(binding, world).await.unwrap(), Some(record));
    let (publisher, worker) = Worker::start(store);
    let rows=admin.query("SELECT pid FROM pg_stat_activity WHERE application_name='theia-development-persistence'",&[]).await.unwrap();
    assert_eq!(rows.len(), 1, "Never terminate an ambiguous backend");
    let pid: i32 = rows[0].get(0);
    let killed: bool = admin
        .query_one("SELECT pg_terminate_backend($1)", &[&pid])
        .await
        .unwrap()
        .get(0);
    assert!(killed);
    record.sequence += 1;
    // Connection loss must be reported even with no later job/publication.
    let deadline = Instant::now() + Duration::from_secs(2);
    while !worker.status().failed {
        assert!(
            Instant::now() < deadline,
            "Backend failure was not reported"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(worker.shutdown().await.is_err());
    assert!(publisher.publish(&[record]).is_err());
    // Cancel shutdown while the worker is genuinely blocked in SQL. Dropping
    // the future must abort the owning worker, not detach a future DB writer.
    let store = Store::connect_loopback(&url, realm).await.unwrap();
    let committed = store.load(binding, world).await.unwrap().unwrap();
    let transaction = admin.transaction().await.unwrap();
    transaction.query_one("SELECT character_id FROM theia.characters WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()]).await.unwrap();
    let (publisher, worker) = Worker::start(store);
    let mut cancelled = committed;
    cancelled.sequence += 1;
    cancelled.position = GlobalPosition::new([1., 1., 1.]).unwrap();
    publisher.publish(&[cancelled]).unwrap();
    let deadline = Instant::now() + Duration::from_millis(350);
    loop {
        let blocked:bool=transaction.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE application_name='theia-development-persistence' AND wait_event_type='Lock')",&[]).await.unwrap().get(0);
        if blocked {
            break;
        }
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    assert!(
        tokio::time::timeout(Duration::from_millis(10), worker.shutdown())
            .await
            .is_err()
    );
    transaction.commit().await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    let store = loop {
        match Store::connect_loopback(&url, realm).await {
            Ok(store) => break store,
            Err(theia_persistence::Error::RealmBusy) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(5)).await
            }
            Err(error) => panic!("Cancelled worker retained its connection: {error}"),
        }
    };
    assert_eq!(store.load(binding, world).await.unwrap(), Some(committed));
    assert!(publisher.publish(&[cancelled]).is_err());
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
