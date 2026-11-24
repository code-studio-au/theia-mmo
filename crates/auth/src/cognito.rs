//! Verify the access token at the realm boundary before resolving an owner.
use crate::{Error, Result, VerifiedIdentity, transport::BoundedHttp};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ring::signature::{RSA_PKCS1_2048_8192_SHA256, RsaPublicKeyComponents};
use serde::Deserialize;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use url::Url;

const PLAY_SCOPE: &str = "theia-development/play";
const TESTER_GROUP: &str = "theia-testers";
const KEY_LIFETIME: Duration = Duration::from_secs(900);
const REFRESH_INTERVAL: Duration = Duration::from_secs(30);
pub const MAX_TOKEN_BYTES: usize = 8192;

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    alg: String,
    #[serde(rename = "use")]
    purpose: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct Header {
    alg: String,
    kid: String,
}

#[derive(Deserialize)]
struct Claims {
    iss: String,
    client_id: String,
    token_use: String,
    sub: String,
    scope: String,
    iat: u64,
    exp: u64,
    nbf: Option<u64>,
    #[serde(rename = "cognito:groups", default)]
    groups: Vec<String>,
}

struct Keys {
    set: Jwks,
    loaded: Instant,
    attempted: Instant,
}

pub struct Verifier {
    issuer: String,
    client_id: String,
    url: String,
    http: BoundedHttp,
    keys: Mutex<Keys>,
}

impl Verifier {
    pub async fn discover(issuer: &str, client_id: &str) -> Result<Self> {
        let origin = Url::parse(issuer).map_err(|_| Error::Configuration)?;
        let pool = issuer
            .strip_prefix("https://cognito-idp.ap-southeast-2.amazonaws.com/ap-southeast-2_")
            .ok_or(Error::Configuration)?;
        if !valid_id(pool) || !valid_id(client_id) {
            return Err(Error::Configuration);
        }
        let url = format!("{issuer}/.well-known/jwks.json");
        let http = BoundedHttp::new(origin)?;
        let set = fetch_keys(&http, &url).await?;
        let now = Instant::now();
        Ok(Self {
            issuer: issuer.to_owned(),
            client_id: client_id.to_owned(),
            url,
            http,
            keys: Mutex::new(Keys {
                set,
                loaded: now,
                attempted: now,
            }),
        })
    }

    pub async fn verify(&self, token: &str) -> Result<VerifiedIdentity> {
        if token.len() > MAX_TOKEN_BYTES {
            return Err(Error::Token);
        }
        let (signed, signature) = token.rsplit_once('.').ok_or(Error::Token)?;
        let (header, claims) = signed.split_once('.').ok_or(Error::Token)?;
        let header: Header = decode(header)?;
        if header.alg != "RS256" || header.kid.is_empty() || header.kid.len() > 128 {
            return Err(Error::Token);
        }
        let mut keys = self.keys.lock().await;
        let stale = keys.loaded.elapsed() >= KEY_LIFETIME;
        let known = keys.set.keys.iter().any(|key| key.kid == header.kid);
        if (stale || !known) && keys.attempted.elapsed() >= REFRESH_INTERVAL {
            keys.attempted = Instant::now();
            keys.set = fetch_keys(&self.http, &self.url).await?;
            keys.loaded = Instant::now();
        }
        if keys.loaded.elapsed() >= KEY_LIFETIME {
            return Err(Error::Token);
        }
        let key = keys
            .set
            .keys
            .iter()
            .find(|key| key.kid == header.kid)
            .ok_or(Error::Token)?;
        verify_signature(key, signed.as_bytes(), signature)?;
        drop(keys);

        let claims: Claims = decode(claims)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Token)?
            .as_secs();
        claims.identity(&self.issuer, &self.client_id, now)
    }
}

impl Claims {
    fn identity(self, issuer: &str, client_id: &str, now: u64) -> Result<VerifiedIdentity> {
        if self.iss != issuer
            || self.client_id != client_id
            || self.token_use != "access"
            || !self
                .scope
                .split_ascii_whitespace()
                .any(|scope| scope == PLAY_SCOPE)
            || !self.groups.iter().any(|group| group == TESTER_GROUP)
            || !valid_id(&self.sub)
            || self.exp <= now
            || self.iat > now.saturating_add(30)
            || self.nbf.is_some_and(|value| value > now)
            || !(1..=3600).contains(&self.exp.saturating_sub(self.iat))
        {
            return Err(Error::Token);
        }
        Ok(VerifiedIdentity {
            issuer: self.iss,
            subject: self.sub,
            expires_unix: self.exp,
        })
    }
}

async fn fetch_keys(http: &BoundedHttp, url: &str) -> Result<Jwks> {
    let request = http::Request::get(url)
        .body(Vec::new())
        .map_err(|_| Error::Http)?;
    let response = http.send(request).await?;
    if !response.status().is_success() || response.body().len() > 32768 {
        return Err(Error::Discovery);
    }
    let jwks: Jwks = serde_json::from_slice(response.body()).map_err(|_| Error::Discovery)?;
    if !(1..=16).contains(&jwks.keys.len()) {
        return Err(Error::Discovery);
    }
    Ok(jwks)
}

fn decode<T: serde::de::DeserializeOwned>(value: &str) -> Result<T> {
    let decoded = URL_SAFE_NO_PAD.decode(value).map_err(|_| Error::Token)?;
    serde_json::from_slice(&decoded).map_err(|_| Error::Token)
}

fn valid_id(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
}

fn verify_signature(key: &Jwk, message: &[u8], signature: &str) -> Result<()> {
    if key.kty != "RSA" || key.alg != "RS256" || key.purpose != "sig" {
        return Err(Error::Token);
    }
    let n = URL_SAFE_NO_PAD.decode(&key.n).map_err(|_| Error::Token)?;
    let e = URL_SAFE_NO_PAD.decode(&key.e).map_err(|_| Error::Token)?;
    let signature = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| Error::Token)?;
    RsaPublicKeyComponents { n, e }
        .verify(&RSA_PKCS1_2048_8192_SHA256, message, &signature)
        .map_err(|_| Error::Token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn access_requires_expected_pool_client_tester_scope_and_fresh_token() {
        let valid = json!({
            "iss":"pool", "client_id":"launcher", "token_use":"access", "sub":"tester-id",
            "scope":"openid theia-development/play", "cognito:groups":["theia-testers"],
            "iat":30, "exp":100,
        });
        let verify = |value| {
            serde_json::from_value::<Claims>(value)
                .unwrap()
                .identity("pool", "launcher", 90)
        };
        assert_eq!(verify(valid.clone()).unwrap().subject(), "tester-id");
        for (field, invalid) in [
            ("iss", json!("another-pool")),
            ("client_id", json!("another-client")),
            ("token_use", json!("id")),
            ("scope", json!("theia-development/play-other")),
            ("cognito:groups", json!(["other"])),
            ("exp", json!(90)),
            ("iat", json!(121)),
            ("nbf", json!(91)),
            ("sub", json!("")),
        ] {
            let mut token = valid.clone();
            token[field] = invalid;
            assert!(verify(token).is_err(), "{field}");
        }
    }
}
