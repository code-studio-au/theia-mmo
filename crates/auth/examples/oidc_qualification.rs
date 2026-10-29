//! Private pipe protocol for the owned real-IdP qualifier. Authorization URLs and
//! callback codes travel only over parent/child pipes, never diagnostic logs.
use std::io::{BufRead, Read, Write};
use theia_auth::{Error, Provider};
fn input() -> Result<String, Error> {
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .take(16385)
        .read_line(&mut line)
        .map_err(|_| Error::Callback)?;
    if line.len() > 16384 {
        return Err(Error::Callback);
    }
    serde_json::from_str(&line).map_err(|_| Error::Callback)
}
fn output(value: serde_json::Value) {
    println!("{value}");
    std::io::stdout().flush().unwrap();
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let issuer = std::env::var("THEIA_TEST_OIDC_ISSUER").map_err(|_| Error::Configuration)?;
    let client = std::env::var("THEIA_TEST_OIDC_CLIENT").map_err(|_| Error::Configuration)?;
    let redirect = std::env::var("THEIA_TEST_OIDC_REDIRECT").map_err(|_| Error::Configuration)?;
    let provider = Provider::discover_loopback(&issuer, &client, &redirect).await?;
    let mut first = None;
    for case in [
        "success",
        "wrong_state",
        "wrong_nonce",
        "wrong_pkce",
        "replay_callback",
        "success_again",
    ] {
        let attempt = provider.begin();
        output(
            serde_json::json!({"case":case,"authorization_url":attempt.authorization_url().as_str()}),
        );
        let result = attempt.finish(&input()?).await;
        match case {
            "success" | "success_again" => {
                let identity = result?;
                if identity.issuer() != issuer {
                    return Err(Error::Token);
                }
                let key = (identity.issuer().to_owned(), identity.subject().to_owned());
                if let Some(first) = &first {
                    if first != &key {
                        return Err(Error::Token);
                    }
                } else {
                    first = Some(key);
                }
            }
            "wrong_state" | "replay_callback" => {
                if !matches!(result, Err(Error::Callback)) {
                    return Err(Error::Callback);
                }
            }
            "wrong_nonce" => {
                if !matches!(result, Err(Error::Token)) {
                    return Err(Error::Token);
                }
            }
            "wrong_pkce" => {
                if !matches!(result, Err(Error::Exchange)) {
                    return Err(Error::Exchange);
                }
            }
            _ => unreachable!(),
        }
        output(serde_json::json!({"case":case,"passed":true}));
    }
    output(
        serde_json::json!({"passed":true,"same_verified_subject_across_logins":true,"game_admission":false}),
    );
    Ok(())
}
