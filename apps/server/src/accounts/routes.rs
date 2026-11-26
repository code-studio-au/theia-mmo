use super::*;
use crate::authenticated::lifecycle::LeaseSnapshot;
use axum::{
    Json,
    extract::{OriginalUri, State, rejection::JsonRejection},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;
use theia_persistence::{Binding, CharacterId, CharacterName, Error as DatabaseError};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Empty {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Poll {
    challenge: String,
    proof: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Create {
    name: String,
    #[serde(default)]
    appearance: theia_core::appearance::Appearance,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Select {
    character_id: String,
}
fn json<T>(input: std::result::Result<Json<T>, JsonRejection>) -> Result<T> {
    input.map(|Json(value)| value).map_err(|error| {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError(StatusCode::PAYLOAD_TOO_LARGE, "Request body too large")
        } else {
            ApiError::bad()
        }
    })
}
fn database(error: DatabaseError) -> ApiError {
    match error {
        DatabaseError::NameUnavailable => {
            ApiError(StatusCode::CONFLICT, "Character name unavailable")
        }
        DatabaseError::CharacterLimit => ApiError(StatusCode::CONFLICT, "Character limit reached"),
        DatabaseError::Ownership
        | DatabaseError::CharacterMissing
        | DatabaseError::AccountMissing
        | DatabaseError::WorldMismatch => ApiError(StatusCode::NOT_FOUND, "Character unavailable"),
        _ => ApiError::busy(),
    }
}
pub(super) async fn ready() -> &'static str {
    // The listener only serves after provider discovery and database worker startup.
    "theia-account-ready\n"
}
pub(super) async fn start(
    State(state): State<Shared>,
    body: std::result::Result<Json<Empty>, JsonRejection>,
) -> Result<Json<serde_json::Value>> {
    json(body)?;
    let mut flows = state.flows()?;
    state.prune(&mut flows);
    if flows.pending.len() >= MAX_FLOWS {
        return Err(ApiError::busy());
    }
    if flows.starts.len() >= 32 {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Please wait before starting another login",
        ));
    }
    let attempt = state.provider.as_ref().ok_or_else(ApiError::bad)?.begin();
    let authorization_url = attempt.authorization_url().to_string();
    let csrf = attempt
        .authorization_url()
        .query_pairs()
        .find(|(name, _)| name == "state")
        .map(|(_, value)| value.into_owned())
        .ok_or_else(ApiError::internal)?;
    let challenge = random(16)?;
    let proof = random(32)?;
    if flows.pending.contains_key(&challenge) {
        return Err(ApiError::internal());
    }
    flows.pending.insert(
        challenge.clone(),
        Flow {
            proof: hash(&proof),
            state: hash(&csrf),
            deadline: Instant::now() + FLOW_LIFETIME,
            outcome: Outcome::Waiting(attempt),
        },
    );
    flows.starts.push_back(Instant::now());
    state.counters.login_starts.fetch_add(1, Ordering::Relaxed);
    // Deliberate front-channel response only to the native caller. Never log it.
    Ok(Json(
        serde_json::json!({"challenge":challenge,"proof":proof,"authorization_url":authorization_url}),
    ))
}
struct ExchangeGuard {
    state: Shared,
    challenge: String,
    done: bool,
}
impl Drop for ExchangeGuard {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        if let Ok(mut flows) = self.state.flows()
            && let Some(flow) = flows.pending.get_mut(&self.challenge)
        {
            flow.outcome = Outcome::Failed;
        }
        self.state
            .counters
            .login_failures
            .fetch_add(1, Ordering::Relaxed);
    }
}
pub(super) async fn callback(
    State(state): State<Shared>,
    OriginalUri(uri): OriginalUri,
) -> Result<Response> {
    let callback = format!("{}{uri}", state.base);
    let parsed = url::Url::parse(&callback).map_err(|_| ApiError::bad())?;
    let states: Vec<_> = parsed
        .query_pairs()
        .filter(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .collect();
    if states.len() != 1 || states[0].len() > 256 {
        return Err(ApiError::bad());
    }
    let hashed = hash(&states[0]);
    let (challenge, attempt) = {
        let mut flows = state.flows()?;
        state.prune(&mut flows);
        let challenge = flows
            .pending
            .iter()
            .find(|(_, flow)| bool::from(flow.state.ct_eq(&hashed)))
            .map(|(id, _)| id.clone())
            .ok_or_else(ApiError::bad)?;
        let flow = flows
            .pending
            .get_mut(&challenge)
            .ok_or_else(ApiError::bad)?;
        match std::mem::replace(&mut flow.outcome, Outcome::Exchanging) {
            Outcome::Waiting(attempt) => (challenge, attempt),
            other => {
                flow.outcome = other;
                return Err(ApiError::bad());
            }
        }
    };
    let mut exchange = ExchangeGuard {
        state: state.clone(),
        challenge: challenge.clone(),
        done: false,
    };
    let identity = attempt
        .finish(&callback)
        .await
        .map_err(|_| ApiError::unauthorized())?;
    let owner = state
        .publisher
        .resolve_owner(identity.issuer(), identity.subject())
        .await
        .map_err(database)?;
    {
        let mut flows = state.flows()?;
        state.prune(&mut flows);
        let flow = flows
            .pending
            .get_mut(&challenge)
            .ok_or_else(ApiError::unauthorized)?;
        if !matches!(flow.outcome, Outcome::Exchanging) {
            return Err(ApiError::unauthorized());
        }
        flow.outcome = Outcome::Ready { identity, owner };
        exchange.done = true;
    }
    state.counters.logins.fetch_add(1, Ordering::Relaxed);
    Ok((
        StatusCode::OK,
        "Login verified. Return to the game to finish signing in.",
    )
        .into_response())
}
pub(super) async fn poll(
    State(state): State<Shared>,
    body: std::result::Result<Json<Poll>, JsonRejection>,
) -> Result<Response> {
    let poll = json(body)?;
    if poll.challenge.len() != 32 || poll.proof.len() != 64 {
        return Err(ApiError::unauthorized());
    }
    let outcome = {
        let mut flows = state.flows()?;
        state.prune(&mut flows);
        let flow = flows
            .pending
            .get(&poll.challenge)
            .ok_or_else(ApiError::unauthorized)?;
        if !bool::from(flow.proof.ct_eq(&hash(&poll.proof))) {
            return Err(ApiError::unauthorized());
        }
        if matches!(flow.outcome, Outcome::Waiting(_) | Outcome::Exchanging) {
            return Ok((
                StatusCode::ACCEPTED,
                Json(serde_json::json!({"status":"pending"})),
            )
                .into_response());
        }
        flows
            .pending
            .remove(&poll.challenge)
            .ok_or_else(ApiError::unauthorized)?
            .outcome
    };
    match outcome {
        Outcome::Ready { identity, owner } => {
            let secret = state
                .sessions()?
                .issue(&identity, owner)
                .map_err(|_| ApiError::unauthorized())?;
            state.counters.polls.fetch_add(1, Ordering::Relaxed);
            Ok(Json(serde_json::json!({"session":secret.expose_secret()})).into_response())
        }
        _ => Err(ApiError::unauthorized()),
    }
}
pub(super) async fn cancel(
    State(state): State<Shared>,
    body: std::result::Result<Json<Poll>, JsonRejection>,
) -> Result<StatusCode> {
    let cancel = json(body)?;
    if cancel.challenge.len() != 32 || cancel.proof.len() != 64 {
        return Err(ApiError::unauthorized());
    }
    let mut flows = state.flows()?;
    state.prune(&mut flows);
    let flow = flows
        .pending
        .get(&cancel.challenge)
        .ok_or_else(ApiError::unauthorized)?;
    if !bool::from(flow.proof.ct_eq(&hash(&cancel.proof))) {
        return Err(ApiError::unauthorized());
    }
    flows.pending.remove(&cancel.challenge);
    state
        .counters
        .cancelled_flows
        .fetch_add(1, Ordering::Relaxed);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Hosted {
    #[serde(default)]
    reservation: Option<String>,
}
pub(super) async fn hosted(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: std::result::Result<Json<Hosted>, JsonRejection>,
) -> Result<Response> {
    let Hosted { reservation } = json(body)?;
    let verifier = state.cognito.as_ref().ok_or_else(ApiError::unauthorized)?;
    let identity = verifier
        .verify(bearer(&headers)?)
        .await
        .map_err(|_| ApiError::unauthorized())?;
    let owner = state
        .publisher
        .resolve_owner(identity.issuer(), identity.subject())
        .await
        .map_err(database)?;
    if state.lifecycle {
        let id = reservation
            .filter(|id| {
                id.len() == 36
                    && id.bytes().enumerate().all(|(i, b)| {
                        if [8, 13, 18, 23].contains(&i) {
                            b == b'-'
                        } else {
                            b.is_ascii_hexdigit()
                        }
                    })
            })
            .ok_or_else(ApiError::bad)?;
        let game = state
            .game
            .lock()
            .map_err(|_| ApiError::internal())?
            .clone()
            .ok_or_else(ApiError::busy)?;
        let join = crate::admissions::HostedJoin {
            id,
            subject: identity.subject().into(),
        };
        let leases = LeaseSnapshot::current().ok_or_else(ApiError::busy)?;
        game.hosted_join(owner, join, leases.at, |join| leases.allows(join))
            .map_err(|_| ApiError::busy())?;
    }
    let secret = state
        .sessions()?
        .issue(&identity, owner)
        .map_err(|_| ApiError::unauthorized())?;
    state.counters.logins.fetch_add(1, Ordering::Relaxed);
    Ok(Json(serde_json::json!({"session": secret.expose_secret()})).into_response())
}
pub(super) async fn list(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    let context = state.authorize(&headers)?;
    let characters = state
        .publisher
        .list_characters(*context.owner(), state.world)
        .await
        .map_err(database)?;
    state.validate(&context)?;
    state.counters.lists.fetch_add(1, Ordering::Relaxed);
    let entries: Vec<_> = characters
        .into_iter()
        .map(|c| serde_json::json!({"id":c.id.value().to_string(),"name":c.name.as_str(),"appearance":c.appearance}))
        .collect();
    Ok(Json(serde_json::json!({"characters":entries})))
}

pub(super) async fn website_preflight() -> StatusCode {
    StatusCode::NO_CONTENT
}

pub(super) async fn website_roster(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    let verifier = state
        .website_cognito
        .as_ref()
        .ok_or_else(ApiError::unauthorized)?;
    let identity = verifier
        .verify(bearer(&headers)?)
        .await
        .map_err(|_| ApiError::unauthorized())?;
    let progression = state
        .game
        .lock()
        .map_err(|_| ApiError::internal())?
        .as_ref()
        .and_then(|game| game.progression_catalog())
        .cloned();
    let characters = state
        .publisher
        .list_characters_for_identity(
            identity.issuer(),
            identity.subject(),
            state.world,
            progression,
        )
        .await
        .map_err(database)?;
    let entries: Vec<_> = characters
        .iter()
        .map(|character| {
            serde_json::json!({
                "name": character.name.as_str(),
                "level": character.progression.map(|progress| progress.level),
                "total_xp": character.progression.map(|progress| progress.total),
                "last_saved_unix": character.last_saved_unix,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({"characters": entries})))
}
pub(super) async fn create(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: std::result::Result<Json<Create>, JsonRejection>,
) -> Result<Json<serde_json::Value>> {
    let context = state.authorize(&headers)?;
    let create = json(body)?;
    let name = CharacterName::parse(&create.name).map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "Use 3–20 letters with single interior apostrophes or hyphens",
        )
    })?;
    let character = state
        .publisher
        .create_character_with_appearance(
            *context.owner(),
            state.world,
            name,
            state.spawn,
            create.appearance,
        )
        .await
        .map_err(database)?;
    state.validate(&context)?;
    state.counters.creations.fetch_add(1, Ordering::Relaxed);
    Ok(Json(
        serde_json::json!({"id":character.saved.binding.character.value().to_string(),"name":character.name.as_str(),"appearance":character.appearance}),
    ))
}
pub(super) async fn select(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: std::result::Result<Json<Select>, JsonRejection>,
) -> Result<Json<serde_json::Value>> {
    let context = state.authorize(&headers)?;
    let select = json(body)?;
    if select.character_id.len() > 64 {
        return Err(ApiError::bad());
    }
    let character = CharacterId::parse(&select.character_id).ok_or_else(ApiError::bad)?;
    let selected = state
        .publisher
        .select_character(
            Binding {
                owner: *context.owner(),
                character,
            },
            state.world,
        )
        .await
        .map_err(database)?;
    state.validate(&context)?;
    state.counters.selections.fetch_add(1, Ordering::Relaxed);
    Ok(Json(
        serde_json::json!({"id":selected.saved.binding.character.value().to_string(),"name":selected.name.as_str(),"appearance":selected.appearance,"game_admission":false}),
    ))
}
pub(super) async fn logout(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: std::result::Result<Json<Empty>, JsonRejection>,
) -> Result<StatusCode> {
    json(body)?;
    let value = credential(&headers)?;
    state.sessions()?.revoke(value);
    state.counters.logouts.fetch_add(1, Ordering::Relaxed);
    Ok(StatusCode::NO_CONTENT)
}
pub(super) async fn metrics(State(state): State<Shared>) -> Result<Json<Stats>> {
    Ok(Json(state.stats()?))
}

/// Reserve before the asynchronous SELECT; a concurrent final save cannot race a
/// stale character read into a replacement admission.
pub(super) async fn enter(
    State(state): State<Shared>,
    headers: HeaderMap,
    body: std::result::Result<Json<Select>, JsonRejection>,
) -> Result<Json<serde_json::Value>> {
    let context = state.authorize(&headers)?;
    let select = json(body)?;
    if select.character_id.len() > 64 {
        return Err(ApiError::bad());
    }
    let character = CharacterId::parse(&select.character_id).ok_or_else(ApiError::bad)?;
    let binding = Binding {
        owner: *context.owner(),
        character,
    };
    let gate = state
        .game
        .lock()
        .map_err(|_| ApiError::internal())?
        .clone()
        .ok_or_else(ApiError::busy)?;
    let admission_error = |error: crate::admissions::Error| match error {
        crate::admissions::Error::Unauthorized => ApiError::unauthorized(),
        crate::admissions::Error::Conflict => ApiError(
            StatusCode::CONFLICT,
            "Account or character already reserved",
        ),
        crate::admissions::Error::HostedJoinExpired => ApiError(
            StatusCode::CONFLICT,
            "Join reservation expired; launch the game again",
        ),
        _ => ApiError::busy(),
    };
    let reservation = if state.lifecycle {
        let leases = LeaseSnapshot::current()
            .ok_or_else(|| admission_error(crate::admissions::Error::HostedJoinExpired))?;
        gate.reserve_hosted(context, binding, |join| leases.allows(join))
    } else {
        gate.reserve(context, binding)
    }
    .map_err(admission_error)?;
    let selected = state
        .publisher
        .select_character(binding, state.world)
        .await
        .map_err(database)?;
    if reservation
        .check_gameplay_profiles(&state.publisher)
        .await
        .map_err(admission_error)?
        == theia_persistence::ProfileCompatibility::Incompatible
    {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "Character uses incompatible gameplay content; select a compatible character",
        ));
    }
    let ticket = reservation
        .issue_character(selected)
        .map_err(admission_error)?;
    // Deliberate private bearer response; never write ticket bytes to diagnostics.
    Ok(Json(
        serde_json::json!({"id":character.value().to_string(),"address":ticket.address().to_string(),"ticket":ticket.bytes(),"expires_in_seconds":20}),
    ))
}
