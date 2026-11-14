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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Source,
        telemetry::{self, Attitude, State},
    };
    use axum::{body::Body, http::Request};
    use futures_util::FutureExt;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn connect(subscription: telemetry::Subscription) -> Body {
        crate::http::router_with_telemetry(subscription)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .into_body()
    }
    async fn read(body: &mut Body) -> serde_json::Value {
        let frame = body.frame().await.unwrap().unwrap().into_data().unwrap();
        let text = std::str::from_utf8(&frame).unwrap();
        serde_json::from_str(
            text.lines()
                .find_map(|line| line.strip_prefix("data: "))
                .unwrap(),
        )
        .unwrap()
    }
    #[tokio::test(start_paused = true)]
    async fn concurrent_and_reconnecting_bodies_read_current_scheduled_state() {
        let (mut publisher, subscription, _schedule) =
            telemetry::scheduled_channel(Source::Demo, 2000);
        tokio::task::yield_now().await;
        let mut first = connect(subscription.clone()).await;
        let mut second = connect(subscription.clone()).await;
        assert_eq!(read(&mut first).await["state"], "waiting");
        assert_eq!(read(&mut second).await["state"], "waiting");
        publisher
            .publish(State::Live(Attitude::new(1.0, 2.0).unwrap()))
            .unwrap();
        let initial = read(&mut first).await;
        assert_eq!(read(&mut second).await["sequence"], initial["sequence"]);
        tokio::time::advance(Duration::from_millis(1500)).await;
        publisher
            .publish(State::Live(Attitude::new(3.0, 4.0).unwrap()))
            .unwrap();
        assert!(first.frame().now_or_never().is_none());
        let mut reconnect = connect(subscription).await;
        let current = read(&mut reconnect).await;
        assert_eq!(current["state"], "live");
        assert_eq!(current["attitude"]["pitch_deg"], 3.0);
        assert_eq!(current["sample_age_ms"], 0);
        assert!(reconnect.frame().now_or_never().is_none());
        tokio::time::advance(Duration::from_millis(500)).await;
        tokio::task::yield_now().await;
        let next = read(&mut reconnect).await;
        assert!(next["sequence"].as_u64().unwrap() > current["sequence"].as_u64().unwrap());
        assert_eq!(next["attitude"]["pitch_deg"], 3.0);
        assert_eq!(read(&mut first).await["sequence"], next["sequence"]);
        assert_eq!(read(&mut second).await["sequence"], next["sequence"]);
    }
}
