use super::*;
use chrono::{Duration as ChronoDuration, Utc};
use ed25519_dalek::{SigningKey, pkcs8::EncodePrivateKey};
use openidconnect::{
    AccessToken, Audience, AuthUrl, JsonWebKeySetUrl, PrivateSigningKey, ResponseTypes,
    StandardClaims, SubjectIdentifier,
    core::{
        CoreEdDsaPrivateSigningKey, CoreIdToken, CoreIdTokenClaims, CoreIdTokenFields,
        CoreJsonWebKeySet, CoreJwsSigningAlgorithm, CoreResponseType, CoreSubjectIdentifierType,
        CoreTokenType,
    },
};
const ISSUER: &str = "http://127.0.0.1:40123/realms/unit-test";
const REDIRECT: &str = "http://127.0.0.1:40124/callback";
fn signing(seed: u8) -> CoreEdDsaPrivateSigningKey {
    // Synthesized unit-test seed, never a production/provider credential.
    let pem = SigningKey::from_bytes(&[seed; 32])
        .to_pkcs8_pem(Default::default())
        .unwrap();
    CoreEdDsaPrivateSigningKey::from_ed25519_pem(&pem, None).unwrap()
}
fn provider() -> Arc<Provider> {
    let metadata = CoreProviderMetadata::new(
        IssuerUrl::new(ISSUER.into()).unwrap(),
        AuthUrl::new(format!("{ISSUER}/auth")).unwrap(),
        JsonWebKeySetUrl::new(format!("{ISSUER}/keys")).unwrap(),
        vec![ResponseTypes::new(vec![CoreResponseType::Code])],
        vec![CoreSubjectIdentifierType::Public],
        vec![CoreJwsSigningAlgorithm::EdDsa],
        Default::default(),
    )
    .set_jwks(CoreJsonWebKeySet::new(vec![
        signing(7).as_verification_key(),
    ]));
    Arc::new(Provider {
        client: CoreClient::from_provider_metadata(
            metadata,
            ClientId::new("unit-client".into()),
            None,
        )
        .set_redirect_uri(RedirectUrl::new(REDIRECT.into()).unwrap()),
        http: BoundedHttp::new(Url::parse(ISSUER).unwrap()).unwrap(),
        issuer: ISSUER.into(),
        redirect: Url::parse(REDIRECT).unwrap(),
    })
}
fn response(
    issuer: &str,
    audience: &str,
    nonce: &str,
    seed: u8,
    expires: i64,
) -> CoreTokenResponse {
    let token = CoreIdToken::new(
        CoreIdTokenClaims::new(
            IssuerUrl::new(issuer.into()).unwrap(),
            vec![Audience::new(audience.into())],
            Utc::now() + ChronoDuration::seconds(expires),
            Utc::now() - ChronoDuration::seconds(1),
            StandardClaims::new(SubjectIdentifier::new("synthetic-unit-subject".into())),
            Default::default(),
        )
        .set_nonce(Some(Nonce::new(nonce.into()))),
        &signing(seed),
        CoreJwsSigningAlgorithm::EdDsa,
        Some(&AccessToken::new("unit-access-token".into())),
        None,
    )
    .unwrap();
    CoreTokenResponse::new(
        AccessToken::new("unit-access-token".into()),
        CoreTokenType::Bearer,
        CoreIdTokenFields::new(Some(token), openidconnect::EmptyExtraTokenFields {}),
    )
}
#[test]
fn signed_claims_require_pinned_issuer_audience_nonce_key_expiry_and_access_hash() {
    let p = provider();
    let nonce = Nonce::new("expected-nonce".into());
    let valid = p
        .verify_response(
            &response(ISSUER, "unit-client", "expected-nonce", 7, 60),
            &nonce,
        )
        .unwrap();
    assert_eq!(valid.issuer(), ISSUER);
    assert_eq!(valid.subject(), "synthetic-unit-subject");
    for bad in [
        response(
            "http://127.0.0.1:40123/foreign",
            "unit-client",
            "expected-nonce",
            7,
            60,
        ),
        response(ISSUER, "foreign-client", "expected-nonce", 7, 60),
        response(ISSUER, "unit-client", "wrong-nonce", 7, 60),
        response(ISSUER, "unit-client", "expected-nonce", 8, 60),
        response(ISSUER, "unit-client", "expected-nonce", 7, -60),
    ] {
        assert!(matches!(p.verify_response(&bad, &nonce), Err(Error::Token)));
    }
    let mut bad = response(ISSUER, "unit-client", "expected-nonce", 7, 60);
    bad.set_access_token(AccessToken::new("substituted-access-token".into()));
    assert!(matches!(p.verify_response(&bad, &nonce), Err(Error::Token)));
}
#[test]
fn callbacks_reject_duplicate_foreign_and_mismatched_parameters() {
    let url = Url::parse(REDIRECT).unwrap();
    assert_eq!(
        callback_code(
            &url,
            ISSUER,
            "expected",
            &format!("{REDIRECT}?code=one&state=expected")
        )
        .unwrap(),
        "one"
    );
    for bad in [
        format!("{REDIRECT}?code=one&state=wrong"),
        format!("{REDIRECT}?code=one&state=expected&state=expected"),
        format!("{REDIRECT}?code=one&code=two&state=expected"),
        format!("{REDIRECT}?code=one&state=expected#fragment"),
        format!("{REDIRECT}?code=one&state=expected&iss=http://foreign.invalid"),
        format!("{REDIRECT}?error=access_denied&state=expected"),
        "http://127.0.0.1:9/callback?code=one&state=expected".into(),
        format!("{REDIRECT}?code={}&state=expected", "a".repeat(8192)),
    ] {
        assert!(matches!(
            callback_code(&url, ISSUER, "expected", &bad),
            Err(Error::Callback)
        ));
    }
}
#[tokio::test]
async fn configuration_and_expired_attempt_fail_before_network_exchange() {
    for bad in [
        "http://localhost:123/realm",
        "http://203.0.113.1:123/realm",
        "http://user:secret@127.0.0.1:123/realm",
        "http://127.0.0.1/realm",
        "http://127.0.0.1:123/realm#fragment",
        "https://127.0.0.1:123/realm",
        "HTTP://127.0.0.1:123/realm",
        "http://127.1:123/realm",
    ] {
        assert!(loopback_url(bad).is_err());
    }
    let mut attempt = provider().begin();
    attempt.created = Instant::now() - Duration::from_secs(121);
    assert!(matches!(
        attempt.finish("never-log-this").await,
        Err(Error::Expired)
    ));
    let text = format!("{:?} {}", Error::Token, Error::Callback);
    assert!(!text.contains("never-log-this"));
}
