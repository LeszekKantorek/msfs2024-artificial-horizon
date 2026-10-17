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
        (
            "/heading.js",
            "text/javascript; charset=utf-8",
            "createHeading",
        ),
        (
            "/altitude/trend.js",
            "text/javascript; charset=utf-8",
            "createAltitudeTrend",
        ),
        (
            "/altitude/selected.js",
            "text/javascript; charset=utf-8",
            "createSelectedAltitude",
        ),
        (
            "/altitude/index.js",
            "text/javascript; charset=utf-8",
            "createAltitude",
        ),
        (
            "/altitude/tape.js",
            "text/javascript; charset=utf-8",
            "createTape",
        ),
        (
            "/altitude/readout.js",
            "text/javascript; charset=utf-8",
            "createReadout",
        ),
        (
            "/altitude/baro.js",
            "text/javascript; charset=utf-8",
            "createBaro",
        ),
        ("/", "text/html; charset=utf-8", "telemetry-status"),
        (
            "/altitude/vsi.js",
            "text/javascript; charset=utf-8",
            "createVsi",
        ),
        ("/styles.css", "text/css; charset=utf-8", "font-family"),
        ("/health", "application/json", "{\"status\":\"ok\"}"),
        (
            "/app.js",
            "text/javascript; charset=utf-8",
            "createTelemetryClient",
        ),
        (
            "/horizon/index.js",
            "text/javascript; charset=utf-8",
            "createHorizon",
        ),
        (
            "/horizon/world.js",
            "text/javascript; charset=utf-8",
            "createWorld",
        ),
        (
            "/horizon/pitch-scale.js",
            "text/javascript; charset=utf-8",
            "createPitchScale",
        ),
        (
            "/horizon/bank-pointer.js",
            "text/javascript; charset=utf-8",
            "createBankPointer",
        ),
        (
            "/airspeed/index.js",
            "text/javascript; charset=utf-8",
            "createAirspeed",
        ),
        (
            "/airspeed/ias.js",
            "text/javascript; charset=utf-8",
            "createIas",
        ),
        (
            "/airspeed/true-airspeed.js",
            "text/javascript; charset=utf-8",
            "createTrueAirspeed",
        ),
        (
            "/airspeed/ground-speed.js",
            "text/javascript; charset=utf-8",
            "createGroundSpeed",
        ),
        (
            "/airspeed/tape.js",
            "text/javascript; charset=utf-8",
            "createTape",
        ),
        (
            "/airspeed/readout.js",
            "text/javascript; charset=utf-8",
            "createReadout",
        ),
        (
            "/airspeed/references.js",
            "text/javascript; charset=utf-8",
            "createReferences",
        ),
        (
            "/airspeed/helpers.js",
            "text/javascript; charset=utf-8",
            "speedState",
        ),
        ("/layout.js", "text/javascript; charset=utf-8", "pfdLayout"),
        ("/helpers.js", "text/javascript; charset=utf-8", "clamp"),
        ("/panel.js", "text/javascript; charset=utf-8", "createPanel"),
        (
            "/frame-scheduler.js",
            "text/javascript; charset=utf-8",
            "createFrameScheduler",
        ),
        (
            "/fixed-symbols/index.js",
            "text/javascript; charset=utf-8",
            "createFixedSymbols",
        ),
        (
            "/fixed-symbols/bank-scale.js",
            "text/javascript; charset=utf-8",
            "createBankScale",
        ),
        (
            "/fixed-symbols/aircraft.js",
            "text/javascript; charset=utf-8",
            "createAircraft",
        ),
        (
            "/slip-skid.js",
            "text/javascript; charset=utf-8",
            "createSlipSkid",
        ),
        (
            "/turn-rate.js",
            "text/javascript; charset=utf-8",
            "createTurnRate",
        ),
        (
            "/status.js",
            "text/javascript; charset=utf-8",
            "createStatusView",
        ),
        (
            "/helpers.js",
            "text/javascript; charset=utf-8",
            "svgElement",
        ),
        (
            "/helpers.js",
            "text/javascript; charset=utf-8",
            "setAttributes",
        ),
        (
            "/telemetry/validation.js",
            "text/javascript; charset=utf-8",
            "normalizeOptionalIndications",
        ),
        (
            "/telemetry/client.js",
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
async fn compact_vsi_altitude_trend_and_heading_assets_support_get_and_head() {
    for (path, export) in [
        ("/altitude/vsi.js", "createVsi"),
        ("/altitude/trend.js", "createAltitudeTrend"),
        ("/heading.js", "createHeading"),
    ] {
        for method in ["GET", "HEAD"] {
            let response = router()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                response.headers()["content-type"],
                "text/javascript; charset=utf-8"
            );
            let body = response.into_body().collect().await.unwrap().to_bytes();
            if method == "HEAD" {
                assert!(body.is_empty());
            } else {
                assert!(std::str::from_utf8(&body).unwrap().contains(export));
            }
        }
    }
    let old_route = router()
        .oneshot(
            Request::builder()
                .uri("/vertical-speed.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(old_route.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn static_router_does_not_create_a_telemetry_source() {
    for path in [
        "/missing",
        "/api/v1/events",
        "/layout.html",
        "/layout-fixture.js",
        "/tests/browser/helpers/layout-fixture.js",
        "/math.js",
        "/svg.js",
        "/telemetry-client.js",
        "/airspeed.js",
        "/horizon.js",
        "/fixed-symbols.js",
        "/airspeed/calculations.js",
        "/horizon/calculations.js",
    ] {
        let response = router()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
