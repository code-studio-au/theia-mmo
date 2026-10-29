use super::*;
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
    let attempt = state.provider.begin();
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
        .map(|c| serde_json::json!({"id":c.id.value().to_string(),"name":c.name.as_str()}))
        .collect();
    Ok(Json(serde_json::json!({"characters":entries})))
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
        .create_character(*context.owner(), state.world, name, state.spawn)
        .await
        .map_err(database)?;
    state.validate(&context)?;
    state.counters.creations.fetch_add(1, Ordering::Relaxed);
    Ok(Json(
        serde_json::json!({"id":character.saved.binding.character.value().to_string(),"name":character.name.as_str()}),
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
        serde_json::json!({"id":selected.saved.binding.character.value().to_string(),"name":selected.name.as_str(),"game_admission":false}),
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
