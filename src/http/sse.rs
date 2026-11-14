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
        (subscription, true, None),
        |(mut subscription, initial, previous_projection)| async move {
            let mut first = initial;
            loop {
                let snapshot = if first {
                    first = false;
                    subscription.snapshot_and_update()
                } else {
                    match subscription.changed().await {
                        Ok(snapshot) => snapshot,
                        Err(_) => return None,
                    }
                };
                if previous_projection.is_some_and(|seq| snapshot.commits_projected_stale(seq)) {
                    continue;
                }
                let projection = snapshot.projected_stale_sequence();
                let event = Event::default()
                    .event("telemetry")
                    .retry(Duration::from_secs(2))
                    .json_data(snapshot);
                return Some((event, (subscription, false, projection)));
            }
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
    async fn projected_expiry_suppresses_only_its_own_committed_transition() {
        let (mut publisher, subscription) = telemetry::channel(Source::Demo);
        publisher
            .publish(State::Live(Attitude::new(1.0, 2.0).unwrap()))
            .unwrap();
        tokio::time::advance(Duration::from_millis(750)).await;
        let mut body = connect(subscription.clone()).await;
        assert_eq!(read(&mut body).await["state"], "stale");
        publisher.expire().unwrap();
        assert!(body.frame().now_or_never().is_none());
        publisher
            .publish(State::Live(Attitude::new(3.0, 4.0).unwrap()))
            .unwrap();
        assert_eq!(read(&mut body).await["state"], "live");
        tokio::time::advance(Duration::from_millis(750)).await;
        publisher.expire().unwrap();
        assert_eq!(read(&mut body).await["state"], "stale");
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_and_reconnecting_bodies_read_current_state() {
        let (mut publisher, subscription, _schedule) =
            telemetry::managed_channel(Source::Demo, 2000).unwrap();
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
        let update = read(&mut first).await;
        assert_eq!(update["attitude"]["pitch_deg"], 3.0);
        let mut reconnect = connect(subscription).await;
        let current = read(&mut reconnect).await;
        assert_eq!(current["state"], "live");
        assert_eq!(current["attitude"]["pitch_deg"], 3.0);
        assert_eq!(current["sample_age_ms"], 0);
        assert!(reconnect.frame().now_or_never().is_none());
        assert_eq!(read(&mut second).await["sequence"], current["sequence"]);
        tokio::time::advance(Duration::from_millis(2000)).await;
        tokio::task::yield_now().await;
        assert_eq!(read(&mut reconnect).await["state"], "stale");
        assert_eq!(read(&mut first).await["state"], "stale");
    }
}
