use super::*;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use tokio::task::JoinSet;
fn finish(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        "referrer-policy",
        axum::http::HeaderValue::from_static("no-referrer"),
    );
    response.headers_mut().insert(
        "x-content-type-options",
        axum::http::HeaderValue::from_static("nosniff"),
    );
    response
}
pub(super) async fn guard(
    State(state): State<Shared>,
    request: Request<Body>,
    next: Next,
) -> Response {
    state.counters.requests.fetch_add(1, Ordering::Relaxed);
    let valid_host = {
        let mut hosts = request.headers().get_all(header::HOST).iter();
        hosts
            .next()
            .is_some_and(|v| v.as_bytes() == state.host.as_bytes())
            && hosts.next().is_none()
    };
    if !valid_host || request.uri().authority().is_some() || request.uri().to_string().len() > 8192
    {
        state
            .counters
            .host_rejections
            .fetch_add(1, Ordering::Relaxed);
        return finish(ApiError::bad().into_response());
    }
    let website_route = request.uri().path() == WEBSITE_ROSTER_PATH
        && state.website_cognito.is_some()
        && matches!(*request.method(), Method::GET | Method::OPTIONS);
    let mut origins = request.headers().get_all(header::ORIGIN).iter();
    let origin = origins.next();
    let website_request =
        website_route && origin.is_some_and(|v| v.as_bytes() == WEBSITE_ORIGIN.as_bytes());
    // Callback navigation uses state/nonce/PKCE. Other game routes accept native or same-origin requests.
    if request.uri().path() != "/callback"
        && ((origin.is_some_and(|v| v.as_bytes() != state.base.as_bytes()) && !website_request)
            || origins.next().is_some())
    {
        state
            .counters
            .origin_rejections
            .fetch_add(1, Ordering::Relaxed);
        return finish(
            ApiError(StatusCode::FORBIDDEN, "Request origin not allowed").into_response(),
        );
    }
    let response = match state.request_slots.clone().try_acquire_owned() {
        Ok(_permit) => next.run(request).await,
        Err(_) => {
            state.counters.overload.fetch_add(1, Ordering::Relaxed);
            ApiError::busy().into_response()
        }
    };
    let mut response = finish(response);
    if website_request {
        let headers = response.headers_mut();
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static(WEBSITE_ORIGIN),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Authorization"),
        );
        headers.insert(header::VARY, HeaderValue::from_static("Origin"));
    }
    response
}
pub(super) async fn run(
    service: AccountService,
    mut shutdown: watch::Receiver<bool>,
    mut database: watch::Receiver<WorkerStatus>,
) -> Result<Stats> {
    let AccountService {
        listener,
        router,
        state,
        mut fatal,
    } = service;
    let mut connections = JoinSet::new();
    let outcome:Result<()>=async {
        if *shutdown.borrow() {return Ok(());}
        if database.borrow().failed || *fatal.borrow() {return Err(ApiError::internal());}
        loop {tokio::select! {biased;
            _=shutdown.changed()=>break,
            _=fatal.changed()=>return Err(ApiError::internal()),
            changed=database.changed()=>{if changed.is_err() || database.borrow().failed {return Err(ApiError::internal());}},
            finished=connections.join_next(),if !connections.is_empty()=>{
                if finished.is_some_and(|result|result.is_err()) {return Err(ApiError::internal());}
            }
            accepted=listener.accept()=>{
                let (socket,remote)=accepted.map_err(|_|ApiError::internal())?;
                let Ok(permit)=state.connection_slots.clone().try_acquire_owned() else {
                    state.counters.refused.fetch_add(1,Ordering::Relaxed);continue;
                };
                if !remote.ip().is_loopback() {continue;}
                state.counters.connections.fetch_add(1,Ordering::Relaxed);
                state.counters.peak_connections.fetch_max((64-state.connection_slots.available_permits()) as u64,Ordering::Relaxed);
                let router=router.clone();
                connections.spawn(async move {
                    let _permit=permit;
                    let mut builder=hyper::server::conn::http1::Builder::new();
                    builder.timer(TokioTimer::new()).header_read_timeout(Duration::from_secs(2))
                        .max_headers(32).max_buf_size(16384).max_header_size(16384).keep_alive(false);
                    let connection=builder.serve_connection(TokioIo::new(socket),TowerToHyperService::new(router));
                    // One request/connection; body reads and handler work share
                    // this bound. No accepted socket/task survives indefinitely.
                    let _=tokio::time::timeout(Duration::from_secs(15),connection).await;
                });
            }
        }}
        Ok(())
    }.await;
    connections.abort_all();
    while connections.join_next().await.is_some() {}
    outcome?;
    state.stats()
}
