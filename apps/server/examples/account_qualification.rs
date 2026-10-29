//! Owned real-provider/database qualification via private parent/child pipes.
//! This exercises account boundaries, not native game admission or a public API.
#[path = "../../../crates/persistence/tests/support/mod.rs"]
mod support;
use std::io::{BufRead, Read, Write};
use theia_auth::{Provider, Sessions};
use theia_persistence::{RealmId, Store, Worker};
fn input() -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut text = String::new();
    std::io::stdin().lock().take(16385).read_line(&mut text)?;
    if text.len() > 16384 {
        return Err("Oversized private qualification message".into());
    }
    serde_json::from_str(&text).map_err(|_| "Invalid private qualification message".into())
}
fn output(value: serde_json::Value) {
    println!("{value}");
    std::io::stdout().flush().unwrap();
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let issuer = std::env::var("THEIA_TEST_OIDC_ISSUER")?;
    let client = std::env::var("THEIA_TEST_OIDC_CLIENT")?;
    let redirect = std::env::var("THEIA_TEST_OIDC_REDIRECT")?;
    let (database, admin, driver) = support::owned().await;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000003").unwrap();
    let store = Store::connect_loopback(&database, realm).await?;
    let (publisher, worker) = Worker::start(store);
    let provider = Provider::discover_loopback(&issuer, &client, &redirect).await?;
    let mut sessions = Sessions::default();
    output(serde_json::json!({"request":"prior_session"}));
    let previous = input()?;
    if let Some(credential) = previous["credential"].as_str()
        && sessions.authenticate(credential).is_ok()
    {
        return Err("Prior process credential unexpectedly resumed".into());
    }
    let attempt = provider.begin();
    output(
        serde_json::json!({"request":"authorization","authorization_url":attempt.authorization_url().as_str()}),
    );
    let callback = input()?;
    let callback = callback.as_str().ok_or("Missing private callback")?;
    let identity = attempt.finish(callback).await?;
    let owner = publisher
        .resolve_owner(identity.issuer(), identity.subject())
        .await?;
    if let Some(expected) = previous["owner"].as_str()
        && owner.value().to_string() != expected
    {
        return Err("Persistent verified owner changed across restart".into());
    }
    let credential = sessions.issue(&identity, owner)?;
    if sessions.authenticate(credential.expose_secret())? != owner {
        return Err("Account session owner mismatch".into());
    }
    let superseded = sessions.issue(&identity, owner)?;
    if sessions.authenticate(credential.expose_secret()).is_ok() {
        return Err("Superseded credential accepted".into());
    }
    if sessions.authenticate(superseded.expose_secret())? != owner {
        return Err("Replacement account session failed".into());
    }
    let logout = sessions.issue(&identity, owner)?;
    sessions.revoke(logout.expose_secret());
    if sessions.authenticate(logout.expose_secret()).is_ok() {
        return Err("Revoked credential accepted".into());
    }
    let final_credential = sessions.issue(&identity, owner)?;
    if sessions.authenticate(final_credential.expose_secret())? != owner {
        return Err("Final issued account session failed".into());
    }
    // Session -> owner -> realm list -> owned character. The client never supplies
    // an owner, spawn, world hash or checkpoint sequence.
    let world = [91; 32];
    let session_owner = sessions.authenticate(final_credential.expose_secret())?;
    let entries = publisher.list_characters(session_owner, world).await?;
    let character_id = if entries.is_empty() {
        publisher
            .create_character(
                session_owner,
                world,
                theia_persistence::CharacterName::parse("Adventurer")?,
                theia_core::coordinates::GlobalPosition::new([8060.25, -17.125, 8000.5]).unwrap(),
            )
            .await?
            .saved
            .binding
            .character
    } else {
        if entries.len() != 1 {
            return Err("Unexpected owned qualification character count".into());
        }
        entries[0].id
    };
    let session_owner = sessions.authenticate(final_credential.expose_secret())?;
    let selected = publisher
        .select_character(
            theia_persistence::Binding {
                owner: session_owner,
                character: character_id,
            },
            world,
        )
        .await?;
    if selected.saved.binding.owner != owner || selected.name.as_str() != "Adventurer" {
        return Err("Owned session character selection mismatch".into());
    }
    if let Some(expected) = previous["character"].as_str()
        && character_id.value().to_string() != expected
    {
        return Err("Owned character identity changed across restart".into());
    }
    let status = worker.shutdown().await?;
    if status.failed || status.resolved_owners != 1 || status.character_selections != 1 {
        return Err("Persistent account worker acknowledgement mismatch".into());
    }
    // Private bearer credential is returned only through the owned pipe, so the
    // second process can prove that restart invalidates process-local sessions.
    output(
        serde_json::json!({"passed":true,"owner":owner.value().to_string(),
        "credential":final_credential.expose_secret(),"selected_character":character_id.value().to_string(),"revocation_checked":true,
        "supersession_checked":true,"committed_owner_resolutions":status.resolved_owners,"game_admission":false}),
    );
    drop(admin);
    driver.await.map_err(|_| "Owned fixture driver stopped")??;
    Ok(())
}
