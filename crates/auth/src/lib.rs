//! Provider-neutral OIDC boundary. Initial loopback qualification only; no game
//! session/character authorization is established by a verified identity alone.
mod session;
mod transport;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreTokenResponse},
};
pub use session::{SessionAuthorization, SessionSecret, SessionStats, Sessions};
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use transport::BoundedHttp;
use url::Url;

type OidcClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Configuration,
    Http,
    Discovery,
    Callback,
    Expired,
    Exchange,
    Token,
    Capacity,
    Session,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Configuration => "Invalid development identity-provider configuration",
            Self::Http => "Bounded identity-provider HTTP operation failed",
            Self::Discovery => "Identity-provider discovery failed",
            Self::Callback => "Invalid authorization callback",
            Self::Expired => "Authorization attempt expired",
            Self::Exchange => "Authorization code exchange failed",
            Self::Token => "Identity token verification failed",
            Self::Capacity => "Account session capacity reached",
            Self::Session => "Invalid or expired account session",
        })
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for Error {}

pub struct Provider {
    client: OidcClient,
    http: BoundedHttp,
    issuer: String,
    redirect: Url,
}
impl Provider {
    /// Explicit development-only boundary. No implicit production HTTP exception,
    /// environment proxy, browser/password handling or caller-supplied issuer.
    pub async fn discover_loopback(
        issuer: &str,
        client_id: &str,
        redirect: &str,
    ) -> Result<Arc<Self>> {
        let issuer_url = loopback_url(issuer)?;
        let redirect_url = loopback_url(redirect)?;
        if client_id.is_empty()
            || client_id.len() > 128
            || client_id.chars().any(char::is_control)
            || issuer_url.query().is_some()
            || redirect_url.query().is_some()
            || redirect_url.path() != "/callback"
        {
            return Err(Error::Configuration);
        }
        let http = BoundedHttp::new(issuer_url)?;
        let metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(issuer.to_owned()).map_err(|_| Error::Configuration)?,
            &|request| {
                let http = http.clone();
                async move { http.send(request).await }
            },
        )
        .await
        .map_err(|_| Error::Discovery)?;
        if metadata.issuer().as_str() != issuer {
            return Err(Error::Discovery);
        }
        // Discovery and JWKS fetches have already passed the same-origin HTTP
        // guard. Authorization/token endpoints must meet it before browser use.
        http.validate(metadata.authorization_endpoint().url())?;
        let token = metadata.token_endpoint().ok_or(Error::Discovery)?;
        http.validate(token.url())?;
        let client =
            CoreClient::from_provider_metadata(metadata, ClientId::new(client_id.to_owned()), None)
                .set_redirect_uri(
                    RedirectUrl::new(redirect.to_owned()).map_err(|_| Error::Configuration)?,
                );
        Ok(Arc::new(Self {
            client,
            http,
            issuer: issuer.to_owned(),
            redirect: redirect_url,
        }))
    }
    pub fn begin(self: &Arc<Self>) -> LoginAttempt {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();
        LoginAttempt {
            provider: self.clone(),
            url,
            state,
            nonce,
            verifier,
            created: Instant::now(),
        }
    }
}
/// Owns one state/nonce/PKCE tuple. Consuming completion prevents application-side
/// reuse; it never logs a callback, authorization code or token response.
pub struct LoginAttempt {
    provider: Arc<Provider>,
    url: Url,
    state: CsrfToken,
    nonce: Nonce,
    verifier: PkceCodeVerifier,
    created: Instant,
}
impl LoginAttempt {
    /// Contains one-use authorization state. Pass directly to the system browser;
    /// do not include in diagnostics or public qualification artifacts.
    pub fn authorization_url(&self) -> &Url {
        &self.url
    }
    pub async fn finish(self, callback: &str) -> Result<VerifiedIdentity> {
        if self.created.elapsed() > Duration::from_secs(120) {
            return Err(Error::Expired);
        }
        let code = callback_code(
            &self.provider.redirect,
            &self.provider.issuer,
            self.state.secret(),
            callback,
        )?;
        let response = self
            .provider
            .client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|_| Error::Exchange)?
            .set_pkce_verifier(self.verifier)
            .request_async(&|request| {
                let http = self.provider.http.clone();
                async move { http.send(request).await }
            })
            .await
            .map_err(|_| Error::Exchange)?;
        self.provider.verify_response(&response, &self.nonce)
    }
}
impl Provider {
    fn verify_response(
        &self,
        response: &CoreTokenResponse,
        nonce: &Nonce,
    ) -> Result<VerifiedIdentity> {
        let token = response.id_token().ok_or(Error::Token)?;
        let verifier = self.client.id_token_verifier();
        let claims = token.claims(&verifier, nonce).map_err(|_| Error::Token)?;
        if let Some(expected) = claims.access_token_hash() {
            let actual = AccessTokenHash::from_token(
                response.access_token(),
                token.signing_alg().map_err(|_| Error::Token)?,
                token.signing_key(&verifier).map_err(|_| Error::Token)?,
            )
            .map_err(|_| Error::Token)?;
            if actual != *expected {
                return Err(Error::Token);
            }
        }
        let subject = claims.subject().as_str();
        if subject.is_empty() || subject.len() > 255 || subject.chars().any(char::is_control) {
            return Err(Error::Token);
        }
        let expires_unix =
            u64::try_from(claims.expiration().timestamp()).map_err(|_| Error::Token)?;
        Ok(VerifiedIdentity {
            issuer: self.issuer.clone(),
            subject: subject.to_owned(),
            expires_unix,
        })
    }
}
/// Server-side verified identity, not a bearer credential or a game session.
/// No deserialization/unchecked constructor permits client ownership claims.
pub struct VerifiedIdentity {
    issuer: String,
    subject: String,
    expires_unix: u64,
}
impl VerifiedIdentity {
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
    pub fn expires_unix(&self) -> u64 {
        self.expires_unix
    }
}
fn loopback_url(value: &str) -> Result<Url> {
    if value.len() > 2048 {
        return Err(Error::Configuration);
    }
    let url = Url::parse(value).map_err(|_| Error::Configuration)?;
    if url.scheme() != "http"
        || url.as_str() != value
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.port().is_none()
        || !url.host_str().is_some_and(|s| {
            s.trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|a| a.is_loopback())
        })
    {
        return Err(Error::Configuration);
    }
    Ok(url)
}
fn callback_code(expected: &Url, issuer: &str, state: &str, value: &str) -> Result<String> {
    if value.len() > 8192 {
        return Err(Error::Callback);
    }
    let actual = Url::parse(value).map_err(|_| Error::Callback)?;
    if actual.origin() != expected.origin()
        || actual.path() != expected.path()
        || !actual.username().is_empty()
        || actual.password().is_some()
        || actual.fragment().is_some()
    {
        return Err(Error::Callback);
    }
    let mut code = None;
    let mut received_state = None;
    for (key, value) in actual.query_pairs() {
        match key.as_ref() {
            "code" if code.is_none() && !value.is_empty() && value.len() <= 4096 => {
                code = Some(value.into_owned())
            }
            "state" if received_state.is_none() && !value.is_empty() && value.len() <= 256 => {
                received_state = Some(value.into_owned())
            }
            "session_state" => {}
            "iss" if value == issuer => {}
            _ => return Err(Error::Callback),
        }
    }
    let received_state = received_state.ok_or(Error::Callback)?;
    if !bool::from(received_state.as_bytes().ct_eq(state.as_bytes())) {
        return Err(Error::Callback);
    }
    code.ok_or(Error::Callback)
}

#[cfg(test)]
mod tests;
