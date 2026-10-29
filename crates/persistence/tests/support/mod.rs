//! Destructive DB tests may run only against the runner's explicitly owned cluster.
use tokio::task::JoinHandle;
use tokio_postgres::{Client, NoTls};
pub async fn owned() -> (
    String,
    Client,
    JoinHandle<std::result::Result<(), tokio_postgres::Error>>,
) {
    let url = std::env::var("THEIA_TEST_DATABASE_URL").expect("Owned cluster environment required");
    let expected = std::path::PathBuf::from(
        std::env::var("THEIA_TEST_POSTGRES_DATA").expect("Owned data directory required"),
    );
    let marker = expected.parent().unwrap().join("theia-owned-test-cluster");
    assert_eq!(
        std::fs::read_to_string(marker).unwrap(),
        "Theia disposable PostgreSQL fixture v1\n"
    );
    let (admin, connection) = tokio_postgres::connect(&url, NoTls)
        .await
        .expect("Owned fixture connection failed");
    let driver = tokio::spawn(connection);
    let actual: String = admin
        .query_one("SELECT current_setting('data_directory')", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        std::path::Path::new(&actual),
        expected,
        "Refuse to mutate a database outside the owned fixture"
    );
    (url, admin, driver)
}
