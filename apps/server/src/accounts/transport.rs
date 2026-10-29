use super::*;
use axum::{
    body::Body,
    extract::State,
    http::{Request, header},
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
    // Callback is a top-level provider navigation protected by state/nonce/PKCE.
    // Native API requests omit Origin; browser API requests must be same-origin.
    if request.uri().path() != "/callback" {
        let mut origins = request.headers().get_all(header::ORIGIN).iter();
        if origins
            .next()
            .is_some_and(|v| v.as_bytes() != state.base.as_bytes())
            || origins.next().is_some()
        {
            state
                .counters
                .origin_rejections
                .fetch_add(1, Ordering::Relaxed);
            return finish(
                ApiError(StatusCode::FORBIDDEN, "Request origin not allowed").into_response(),
            );
        }
    }
    let Ok(_permit) = state.request_slots.clone().try_acquire_owned() else {
        state.counters.overload.fetch_add(1, Ordering::Relaxed);
        return finish(ApiError::busy().into_response());
    };
    finish(next.run(request).await)
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
