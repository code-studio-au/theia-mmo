use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use lambda_runtime::{LambdaEvent, service_fn};
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use theia_realm_gateway::{
    Build, Phase, Result,
    aws::{Ec2, Store},
    now,
};

struct Gateway {
    store: Store,
    ec2: Ec2,
    issuer: String,
    client: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wake {
    reservation: String,
    build: Build,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Join {
    reservation: String,
}

fn response(status: u16, body: Value) -> Value {
    json!({"statusCode": status, "headers": {"content-type": "application/json", "cache-control": "no-store"}, "body": body.to_string()})
}

fn refusal(status: u16, message: &str) -> Value {
    response(status, json!({"error":message}))
}

fn tester<'a>(claims: &'a Value, issuer: &str, client: &str) -> Option<&'a str> {
    let member = claims["cognito:groups"]
        .as_array()
        .is_some_and(|groups| groups.iter().any(|group| group == "theia-testers"));
    let sub = claims["sub"].as_str()?;
    (claims["iss"] == issuer
        && claims["client_id"] == client
        && claims["token_use"] == "access"
        && claims["scope"]
            .as_str()?
            .split_ascii_whitespace()
            .any(|s| s == "theia-development/play")
        && member
        && valid_id(sub))
    .then_some(sub)
}

// HTTP API validates the token from this exact Authorization identity source.
// Decode its original group array instead of splitting the flattened context
// claim, which can make a group name containing punctuation appear to be two.
fn authorized_claims(event: &Value) -> Option<Value> {
    let verified = &event["requestContext"]["authorizer"]["jwt"]["claims"];
    let (scheme, token) = event["headers"]["authorization"]
        .as_str()?
        .split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") || token.len() > 8192 {
        return None;
    }
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _signature = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()?;
    let fields_match = ["iss", "sub", "client_id", "token_use"].iter().all(|name| {
        claims[name]
            .as_str()
            .is_some_and(|raw| verified[name].as_str() == Some(raw))
    });
    fields_match.then_some(claims)
}

fn valid_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

fn body<T: serde::de::DeserializeOwned>(event: &Value) -> Option<T> {
    let body = event["body"].as_str()?;
    if event["isBase64Encoded"] == true || body.len() > 2048 {
        return None;
    }
    serde_json::from_str(body).ok()
}

impl Gateway {
    async fn handle(&self, event: Value) -> Result<Value> {
        let claims = authorized_claims(&event);
        let Some(subject) = claims
            .as_ref()
            .and_then(|claims| tester(claims, &self.issuer, &self.client))
        else {
            return Ok(refusal(403, "Tester access required"));
        };
        match event["routeKey"].as_str() {
            Some("GET /v1/realm/status") => {
                let (_, state) = self.store.read().await?;
                let instance = self.ec2.state().await?;
                let phase = state.phase(instance, now());
                Ok(response(
                    200,
                    json!({"state":phase, "release":state.release}),
                ))
            }
            Some("POST /v1/realm/wake") => {
                let Some(wake) = body::<Wake>(&event).filter(|w| valid_id(&w.reservation)) else {
                    return Ok(refusal(400, "Invalid Play request"));
                };
                let at = now();
                let instance = self.ec2.state().await?;
                let outcome = self
                    .store
                    .change(|state| {
                        if !state
                            .release
                            .as_ref()
                            .is_some_and(|r| r.accepts(&wake.build))
                        {
                            return None;
                        }
                        let phase = state.phase(instance, at);
                        if matches!(
                            phase,
                            Phase::Maintenance | Phase::Error | Phase::ShuttingDown
                        ) {
                            return Some((phase, false));
                        }
                        if !state.reserve(&wake.reservation, subject, at) {
                            return None;
                        }
                        let start = state.claim_start(instance, at);
                        Some((state.phase(instance, at), start))
                    })
                    .await?;
                let Some((phase, start)) = outcome else {
                    return Ok(refusal(
                        409,
                        "Incompatible release or unavailable join reservation",
                    ));
                };
                if start && let Err(error) = self.ec2.start().await {
                    // A transport failure can follow a successful StartInstances. Leave the
                    // short claim in place and reconcile against EC2 on the next attempt.
                    eprintln!("EC2 start request failed: {error}");
                    return Ok(refusal(503, "Realm startup could not be confirmed"));
                }
                Ok(response(200, json!({"state":phase})))
            }
            Some("POST /v1/realm/reservation") | Some("DELETE /v1/realm/reservation") => {
                let Some(join) = body::<Join>(&event).filter(|j| valid_id(&j.reservation)) else {
                    return Ok(refusal(400, "Invalid reservation request"));
                };
                let remove = event["routeKey"] == "DELETE /v1/realm/reservation";
                let outcome = self
                    .store
                    .change(|state| {
                        let at = now();
                        if remove {
                            state.release(&join.reservation, subject);
                        } else if state.maintenance
                            || state.stopping.is_some()
                            || !state
                                .reservations
                                .get(&join.reservation)
                                .is_some_and(|r| r.subject == subject && r.expires > at)
                        {
                            return None;
                        } else {
                            state.reserve(&join.reservation, subject, at);
                        }
                        Some(())
                    })
                    .await?;
                Ok(if outcome.is_some() {
                    response(200, json!({}))
                } else {
                    refusal(409, "Join reservation expired")
                })
            }
            _ => Ok(refusal(404, "Unknown realm operation")),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let gateway = Gateway {
        store: Store::new(&config, env::var("THEIA_REALM_TABLE")?),
        ec2: Ec2::new(config, env::var("THEIA_INSTANCE_ID")?)?,
        issuer: env::var("THEIA_COGNITO_ISSUER")?,
        client: env::var("THEIA_COGNITO_CLIENT")?,
    };
    lambda_runtime::run(service_fn(|request: LambdaEvent<Value>| async {
        let response = gateway
            .handle(request.payload)
            .await
            .unwrap_or_else(|error| {
                eprintln!("Realm operation failed: {error}");
                refusal(503, "Realm temporarily unavailable")
            });
        Ok::<_, lambda_runtime::Error>(response)
    }))
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_tester_claims_required() {
        let mut claims = json!({"iss":"issuer","client_id":"launcher","token_use":"access",
            "scope":"openid theia-development/play","cognito:groups":["theia-testers"],
            "sub":"c4b5e5d0-9254-445c-aaa4-24f50ecdcbc9"});
        assert!(tester(&claims, "issuer", "launcher").is_some());
        claims["cognito:groups"] = json!(["other,theia-testers", "theia-testers-admin"]);
        assert!(tester(&claims, "issuer", "launcher").is_none());
        claims["cognito:groups"] = json!(["theia-testers"]);
        claims["token_use"] = json!("id");
        assert!(tester(&claims, "issuer", "launcher").is_none());
    }
    #[test]
    fn jwt_payload_requires_api_gateway_authorization_for_the_same_identity() {
        let claims = json!({"iss":"issuer","client_id":"launcher","token_use":"access",
            "sub":"c4b5e5d0-9254-445c-aaa4-24f50ecdcbc9",
            "cognito:groups":["theia-testers"],"scope":"theia-development/play"});
        let encoded = URL_SAFE_NO_PAD.encode(claims.to_string());
        let mut event = json!({"headers":{"authorization":format!("Bearer a.{encoded}.b")},
            "requestContext":{"authorizer":{"jwt":{"claims":claims}}}});
        assert!(authorized_claims(&event).is_some());
        event["requestContext"]["authorizer"]["jwt"]["claims"]["sub"] =
            json!("a different identity");
        assert!(authorized_claims(&event).is_none());
    }
}
