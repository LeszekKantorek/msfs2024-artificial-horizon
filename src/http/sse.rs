//! Pull-driven SSE delivery: no per-client producer or sample queue.
use crate::telemetry::Subscription;
use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
};
use futures_util::stream;
use std::time::Duration;

pub(super) async fn events(State(subscription): State<Subscription>) -> Response {
    let stream = stream::unfold(
        (subscription, true),
        |(mut subscription, initial)| async move {
            let snapshot = if initial {
                subscription.snapshot_and_update()
            } else {
                match subscription.changed().await {
                    Ok(snapshot) => snapshot,
                    Err(_) => return None,
                }
            };
            let event = Event::default()
                .event("telemetry")
                .retry(Duration::from_secs(2))
                .json_data(snapshot);
            Some((event, (subscription, false)))
        },
    );
    let mut response = Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(10))
                .text("heartbeat"),
        )
        .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-transform"),
    );
    response
        .headers_mut()
        .insert("x-accel-buffering", HeaderValue::from_static("no"));
    response
}
