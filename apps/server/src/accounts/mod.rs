//! In-process development account HTTP boundary. No renderer, transport-ticket
//! issuer or simulation-step dependency; native game admission remains separate.
mod routes;
mod transport;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use ring::{
    digest::{SHA256, digest},
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    net::SocketAddr,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use theia_auth::{LoginAttempt, Provider, SessionAuthorization, Sessions, VerifiedIdentity};
use theia_core::coordinates::GlobalPosition;
use theia_persistence::{OwnerId, Publisher, WorkerStatus};
use tokio::{
    net::TcpListener,
    sync::{Semaphore, watch},
};
const MAX_FLOWS: usize = 16;
const FLOW_LIFETIME: Duration = Duration::from_secs(120);
pub type Result<T> = std::result::Result<T, ApiError>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    pub issuer: String,
    pub client_id: String,
}
/// Static/redacted errors only. Never format provider/SQL errors, bearer headers,
/// raw callback queries or request bodies in responses/diagnostics.
#[derive(Clone, Copy)]
pub struct ApiError(StatusCode, &'static str);
impl ApiError {
    fn bad() -> Self {
        Self(StatusCode::BAD_REQUEST, "Invalid request")
    }
    fn unauthorized() -> Self {
        Self(StatusCode::UNAUTHORIZED, "Account authorization required")
    }
    fn busy() -> Self {
        Self(
            StatusCode::SERVICE_UNAVAILABLE,
            "Account service busy; try again",
        )
    }
    fn internal() -> Self {
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Account service unavailable",
        )
    }
}
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.1)
    }
}
impl fmt::Debug for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for ApiError {}
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.0, axum::Json(serde_json::json!({"error":self.1}))).into_response()
    }
}
enum Outcome {
    Waiting(LoginAttempt),
    Exchanging,
    Ready {
        identity: VerifiedIdentity,
        owner: OwnerId,
    },
    Failed,
}
struct Flow {
    proof: [u8; 32],
    state: [u8; 32],
    deadline: Instant,
    outcome: Outcome,
}
#[derive(Default)]
struct Flows {
    pending: BTreeMap<String, Flow>,
    starts: VecDeque<Instant>,
}
#[derive(Default)]
struct Counters {
    connections: AtomicU64,
    refused: AtomicU64,
    peak_connections: AtomicU64,
    requests: AtomicU64,
    host_rejections: AtomicU64,
    origin_rejections: AtomicU64,
    overload: AtomicU64,
    login_starts: AtomicU64,
    logins: AtomicU64,
    login_failures: AtomicU64,
    expired_flows: AtomicU64,
    cancelled_flows: AtomicU64,
    polls: AtomicU64,
    lists: AtomicU64,
    creations: AtomicU64,
    selections: AtomicU64,
    logouts: AtomicU64,
}
#[derive(Serialize)]
pub struct Stats {
    pub connections: u64,
    pub refused_connections: u64,
    pub peak_connections: u64,
    pub requests: u64,
    pub host_rejections: u64,
    pub origin_rejections: u64,
    pub overloaded_requests: u64,
    pub login_starts: u64,
    pub verified_logins: u64,
    pub failed_logins: u64,
    pub expired_challenges: u64,
    pub cancelled_challenges: u64,
    pub consumed_polls: u64,
    pub character_lists: u64,
    pub created_characters: u64,
    pub selected_characters: u64,
    pub logouts: u64,
    pub pending_logins: usize,
    pub active_account_sessions: usize,
}
struct Inner {
    provider: Arc<Provider>,
    publisher: Publisher,
    sessions: Mutex<Sessions<OwnerId>>,
    flows: Mutex<Flows>,
    request_slots: Arc<Semaphore>,
    connection_slots: Arc<Semaphore>,
    counters: Counters,
    base: String,
    host: String,
    world: [u8; 32],
    spawn: GlobalPosition,
    fatal: watch::Sender<bool>,
}
type Shared = Arc<Inner>;
impl Inner {
    fn flows(&self) -> Result<MutexGuard<'_, Flows>> {
        self.flows.lock().map_err(|_| {
            self.fatal.send_replace(true);
            ApiError::internal()
        })
    }
    fn sessions(&self) -> Result<MutexGuard<'_, Sessions<OwnerId>>> {
        self.sessions.lock().map_err(|_| {
            self.fatal.send_replace(true);
            ApiError::internal()
        })
    }
    fn prune(&self, flows: &mut Flows) {
        let now = Instant::now();
        let before = flows.pending.len();
        flows.pending.retain(|_, flow| flow.deadline > now);
        self.counters
            .expired_flows
            .fetch_add((before - flows.pending.len()) as u64, Ordering::Relaxed);
        while flows
            .starts
            .front()
            .is_some_and(|time| now.duration_since(*time) >= Duration::from_secs(60))
        {
            flows.starts.pop_front();
        }
    }
    fn authorize(&self, headers: &axum::http::HeaderMap) -> Result<SessionAuthorization<OwnerId>> {
        let value = credential(headers)?;
        self.sessions()?
            .authorize(value)
            .map_err(|_| ApiError::unauthorized())
    }
    fn validate(&self, context: &SessionAuthorization<OwnerId>) -> Result<OwnerId> {
        self.sessions()?
            .validate(context)
            .map_err(|_| ApiError::unauthorized())
    }
    fn stats(&self) -> Result<Stats> {
        let pending_logins = {
            let mut flows = self.flows()?;
            self.prune(&mut flows);
            flows.pending.len()
        };
        let active_account_sessions = self.sessions()?.active();
        let c = &self.counters;
        let n = |a: &AtomicU64| a.load(Ordering::Relaxed);
        Ok(Stats {
            connections: n(&c.connections),
            refused_connections: n(&c.refused),
            peak_connections: n(&c.peak_connections),
            requests: n(&c.requests),
            host_rejections: n(&c.host_rejections),
            origin_rejections: n(&c.origin_rejections),
            overloaded_requests: n(&c.overload),
            login_starts: n(&c.login_starts),
            verified_logins: n(&c.logins),
            failed_logins: n(&c.login_failures),
            expired_challenges: n(&c.expired_flows),
            cancelled_challenges: n(&c.cancelled_flows),
            consumed_polls: n(&c.polls),
            character_lists: n(&c.lists),
            created_characters: n(&c.creations),
            selected_characters: n(&c.selections),
            logouts: n(&c.logouts),
            pending_logins,
            active_account_sessions,
        })
    }
}
fn credential(headers: &axum::http::HeaderMap) -> Result<&str> {
    let mut values = headers.get_all(axum::http::header::AUTHORIZATION).iter();
    let value = values
        .next()
        .ok_or_else(ApiError::unauthorized)?
        .to_str()
        .map_err(|_| ApiError::unauthorized())?;
    if values.next().is_some() {
        return Err(ApiError::unauthorized());
    }
    let (scheme, token) = value.split_once(' ').ok_or_else(ApiError::unauthorized)?;
    if !scheme.eq_ignore_ascii_case("Bearer") || token.len() != 64 {
        return Err(ApiError::unauthorized());
    }
    Ok(token)
}
fn hash(value: &str) -> [u8; 32] {
    digest(&SHA256, value.as_bytes())
        .as_ref()
        .try_into()
        .unwrap()
}
fn random(bytes: usize) -> Result<String> {
    // Only fixed small sizes are used for public challenge IDs/private proofs.
    let mut data = [0; 32];
    if bytes > data.len() {
        return Err(ApiError::internal());
    }
    SystemRandom::new()
        .fill(&mut data[..bytes])
        .map_err(|_| ApiError::internal())?;
    Ok(data[..bytes].iter().map(|b| format!("{b:02x}")).collect())
}
pub struct AccountService {
    listener: TcpListener,
    router: Router,
    state: Shared,
    fatal: watch::Receiver<bool>,
}
impl AccountService {
    /// The caller owns the realm's database worker and must provide the real saved
    /// world identity and a prepared, supported default body-centre spawn.
    pub async fn prepare(
        config: Config,
        publisher: Publisher,
        world: [u8; 32],
        spawn: GlobalPosition,
    ) -> Result<Self> {
        if !config.listen.ip().is_loopback() || world == [0; 32] {
            return Err(ApiError::bad());
        }
        let listener = TcpListener::bind(config.listen)
            .await
            .map_err(|_| ApiError::internal())?;
        let address = listener.local_addr().map_err(|_| ApiError::internal())?;
        let base = format!("http://{address}");
        let host = address.to_string();
        let provider = Provider::discover_loopback(
            &config.issuer,
            &config.client_id,
            &format!("{base}/callback"),
        )
        .await
        .map_err(|_| ApiError::internal())?;
        let (fatal, fatal_receiver) = watch::channel(false);
        let state = Arc::new(Inner {
            provider,
            publisher,
            sessions: Mutex::new(Sessions::default()),
            flows: Mutex::new(Flows::default()),
            request_slots: Arc::new(Semaphore::new(16)),
            connection_slots: Arc::new(Semaphore::new(64)),
            counters: Counters::default(),
            base,
            host,
            world,
            spawn,
            fatal,
        });
        let router = Router::new()
            .route("/auth/start", post(routes::start))
            .route("/callback", get(routes::callback))
            .route("/auth/poll", post(routes::poll))
            .route("/auth/cancel", post(routes::cancel))
            .route("/auth/logout", post(routes::logout))
            .route("/characters", get(routes::list).post(routes::create))
            .route("/characters/select", post(routes::select))
            .route("/development/auth-metrics", get(routes::metrics))
            .layer(DefaultBodyLimit::max(4096))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                transport::guard,
            ))
            .with_state(state.clone());
        Ok(Self {
            listener,
            router,
            state,
            fatal: fatal_receiver,
        })
    }
    pub fn address(&self) -> &str {
        &self.state.base
    }
    pub async fn run(
        self,
        shutdown: watch::Receiver<bool>,
        database: watch::Receiver<WorkerStatus>,
    ) -> Result<Stats> {
        transport::run(self, shutdown, database).await
    }
}
