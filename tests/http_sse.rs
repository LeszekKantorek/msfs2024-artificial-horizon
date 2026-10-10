use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures_util::FutureExt;
use http_body_util::BodyExt;
use msfs2024_artificial_horizon::{
    Source,
    http::router_with_telemetry,
    telemetry::{self, Attitude, STALE_AFTER, State},
};
use serde_json::{Value, json};
use tokio::time::{Duration, advance};
use tower::ServiceExt;

async fn subscribe(
    subscription: telemetry::Subscription,
    last_id: Option<&str>,
) -> axum::response::Response {
    let mut request = Request::builder().uri("/api/v1/events");
    if let Some(id) = last_id {
        request = request.header("last-event-id", id);
    }
    router_with_telemetry(subscription)
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}
async fn frame(body: &mut Body) -> String {
    let bytes = body.frame().await.unwrap().unwrap().into_data().unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}
fn snapshot(frame: &str) -> Value {
    assert!(frame.ends_with("\n\n"));
    assert!(frame.lines().any(|line| line == "event: telemetry"));
    assert!(frame.lines().any(|line| line == "retry: 2000"));
    assert!(!frame.lines().any(|line| line.starts_with("id:")));
    serde_json::from_str(
        frame
            .lines()
            .find_map(|line| line.strip_prefix("data: "))
            .unwrap(),
    )
    .unwrap()
}

#[tokio::test(start_paused = true)]
async fn extended_samples_reach_two_clients_with_shared_values_and_delivery_time_age() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    publisher
        .publish_sample(msfs2024_artificial_horizon::providers::demo::sample(240))
        .unwrap();
    let mut first = subscribe(subscription.clone(), None).await.into_body();
    let mut second = subscribe(subscription.clone(), None).await.into_body();
    advance(Duration::from_millis(50)).await;
    let a = snapshot(&frame(&mut first).await);
    let b = snapshot(&frame(&mut second).await);
    assert_eq!(a, b);
    assert_eq!(a["slip_skid"], -1.0);
    assert_eq!(a["turn_rate_dps"], -3.0);
    assert_eq!(a["ias_kt"], 100.0);
    assert!((a["gs_kt"].as_f64().unwrap() - 113.33333333333333).abs() < 1e-10);
    assert_eq!(a["sample_age_ms"], 50);
    publisher.publish(State::Paused).unwrap();
    for body in [&mut first, &mut second] {
        let unavailable = snapshot(&frame(body).await);
        assert_eq!(unavailable["state"], "paused");
        assert!(unavailable.get("slip_skid").is_none());
        assert!(unavailable.get("turn_rate_dps").is_none());
        assert!(unavailable.get("ias_kt").is_none());
        assert!(unavailable.get("gs_kt").is_none());
    }
}

#[tokio::test]
async fn source_failure_does_not_change_http_liveness_or_allow_control_requests() {
    let (mut publisher, subscription) = telemetry::channel(Source::SimConnect);
    publisher.publish(State::Disconnected).unwrap();
    let router = router_with_telemetry(subscription);
    let health = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    let bytes = health.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap(),
        json!({"status":"ok"})
    );
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test(start_paused = true)]
async fn immediate_state_headers_and_no_duplicate_initial_publication() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    publisher
        .publish(State::Live(Attitude::new(5.0, 15.0).unwrap()))
        .unwrap();
    let response = subscribe(subscription, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    assert_eq!(
        response.headers()["cache-control"],
        "no-cache, no-transform"
    );
    assert_eq!(response.headers()["x-accel-buffering"], "no");
    assert!(!response.headers().contains_key("content-encoding"));
    let mut body = response.into_body();
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/snapshots.json")).unwrap();
    assert_eq!(snapshot(&frame(&mut body).await), fixtures[1]);
    assert!(body.frame().now_or_never().is_none());
    publisher.publish(State::Paused).unwrap();
    assert_eq!(snapshot(&frame(&mut body).await)["state"], "paused");
}

#[tokio::test(start_paused = true)]
async fn reconnect_ignores_last_event_id_and_slow_clients_skip_history() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let mut first = subscribe(subscription.clone(), None).await.into_body();
    let mut slow = subscribe(subscription.clone(), None).await.into_body();
    assert_eq!(snapshot(&frame(&mut first).await)["sequence"], 0);
    assert_eq!(snapshot(&frame(&mut slow).await)["sequence"], 0);
    for _ in 0..10_000 {
        publisher
            .publish(State::Live(Attitude::new(10.0, 25.0).unwrap()))
            .unwrap();
    }
    assert_eq!(snapshot(&frame(&mut first).await)["sequence"], 10_000);
    assert_eq!(snapshot(&frame(&mut slow).await)["sequence"], 10_000);
    drop(first);
    publisher.publish(State::Disconnected).unwrap();
    assert_eq!(snapshot(&frame(&mut slow).await)["state"], "disconnected");
    for id in ["1", "10001", "999999999", "not-a-sequence"] {
        let mut reconnected = subscribe(subscription.clone(), Some(id)).await.into_body();
        assert_eq!(snapshot(&frame(&mut reconnected).await)["sequence"], 10_001);
        assert!(reconnected.frame().now_or_never().is_none());
    }
}

#[tokio::test(start_paused = true)]
async fn unavailable_states_age_stale_and_heartbeat_are_independent() {
    let (mut publisher, subscription) = telemetry::channel(Source::SimConnect);
    let mut body = subscribe(subscription.clone(), None).await.into_body();
    assert_eq!(snapshot(&frame(&mut body).await)["state"], "waiting");
    publisher
        .publish(State::Live(Attitude::new(5.0, 15.0).unwrap()))
        .unwrap();
    assert_eq!(snapshot(&frame(&mut body).await)["state"], "live");
    advance(STALE_AFTER).await;
    publisher.expire().unwrap();
    let stale = snapshot(&frame(&mut body).await);
    assert_eq!(stale["state"], "stale");
    assert_eq!(stale["attitude"], Value::Null);
    assert_eq!(stale["sample_age_ms"], 1000);
    advance(Duration::from_secs(10)).await;
    let heartbeat = frame(&mut body).await;
    assert_eq!(heartbeat, ": heartbeat\n\n");
    let current = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(current["sequence"], stale["sequence"]);
    assert_eq!(current["sample_age_ms"], 11_000);
    for (state, name) in [
        (State::Invalid, "invalid"),
        (State::Disconnected, "disconnected"),
        (State::Paused, "paused"),
        (State::Waiting, "waiting"),
    ] {
        publisher.publish(state).unwrap();
        let received = snapshot(&frame(&mut body).await);
        assert_eq!(received["state"], name);
        assert_eq!(received["attitude"], Value::Null);
        assert_eq!(received["source"], "simconnect");
    }
    publisher
        .publish(State::Live(Attitude::new(0.0, 0.0).unwrap()))
        .unwrap();
    assert_eq!(snapshot(&frame(&mut body).await)["sample_age_ms"], json!(0));
    drop(publisher);
    assert!(body.frame().await.is_none());
}

#[tokio::test(start_paused = true)]
async fn age_is_computed_when_body_is_read_and_pending_final_state_precedes_eof() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    publisher
        .publish(State::Live(Attitude::new(0.0, 0.0).unwrap()))
        .unwrap();
    let mut body = subscribe(subscription, None).await.into_body();
    advance(STALE_AFTER).await;
    let initial = snapshot(&frame(&mut body).await);
    assert_eq!(initial["state"], "stale");
    assert_eq!(initial["sample_age_ms"], 1000);
    publisher.publish(State::Invalid).unwrap();
    drop(publisher);
    assert_eq!(snapshot(&frame(&mut body).await)["state"], "invalid");
    assert!(body.frame().await.is_none());
}
