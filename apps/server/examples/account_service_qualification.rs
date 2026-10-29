//! Owned HTTP/provider/database fixture only. No game authority/transport starts.
#[path = "../../../crates/persistence/tests/support/mod.rs"]
mod support;
use theia_core::coordinates::GlobalPosition;
use theia_persistence::{RealmId, Store, Worker};
use theia_server::accounts::{AccountService, Config};
use tokio::sync::watch;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000006").unwrap();
    let store = Store::connect_loopback(&url, realm).await?;
    let (publisher, worker) = Worker::start(store);
    let status = worker.subscribe_status();
    let config = Config {
        listen: std::env::var("THEIA_TEST_ACCOUNT_BIND")?.parse()?,
        issuer: std::env::var("THEIA_TEST_OIDC_ISSUER")?,
        client_id: std::env::var("THEIA_TEST_OIDC_CLIENT")?,
    };
    let service = AccountService::prepare(
        config,
        publisher,
        [92; 32],
        GlobalPosition::new([8060.25, -17.125, 8000.5]).unwrap(),
    )
    .await?;
    println!(
        "account_service_ready {}",
        serde_json::json!({"address":service.address()})
    );
    let (stop, shutdown) = watch::channel(false);
    let run = service.run(shutdown, status);
    tokio::pin!(run);
    let result = tokio::select! {
        result=&mut run=>result,
        _=tokio::signal::ctrl_c()=>{stop.send_replace(true);run.await},
    };
    let stored = worker.shutdown().await;
    println!(
        "account_service_summary {}",
        serde_json::json!({"http":result.as_ref().ok(),"worker":stored.as_ref().ok(),"failed":result.is_err()||stored.is_err(),"game_admission":false})
    );
    drop(admin);
    driver.await.map_err(|_| "Owned fixture driver stopped")??;
    result?;
    stored?;
    Ok(())
}
