use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use msfs2024_artificial_horizon::http::router;
use tower::ServiceExt;

#[tokio::test]
async fn embedded_page_assets_and_liveness_match_the_contract() {
    for (path, content_type, expected) in [
        ("/", "text/html; charset=utf-8", "No telemetry available"),
        ("/styles.css", "text/css; charset=utf-8", "font-family"),
        ("/health", "application/json", "{\"status\":\"ok\"}"),
        (
            "/app.js",
            "text/javascript; charset=utf-8",
            "createTelemetryClient",
        ),
        (
            "/telemetry-client.js",
            "text/javascript; charset=utf-8",
            "EventSource",
        ),
    ] {
        let response = router()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], content_type);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains(expected));
        if path == "/health" {
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(body).unwrap(),
                serde_json::json!({"status": "ok"})
            );
        }
    }
}

#[tokio::test]
async fn static_router_does_not_create_a_telemetry_source() {
    for path in ["/missing", "/api/v1/events"] {
        let response = router()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
