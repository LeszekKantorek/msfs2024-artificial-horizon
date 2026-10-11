//! Compile-time web assets and their HTTP routes.

use axum::{Router, http::header, routing::get};

const JAVASCRIPT: &str = "text/javascript; charset=utf-8";

// Each entry defines an exact route, response MIME type and embedded body.
const ASSETS: &[(&str, &str, &str)] = &[
    (
        "/heading.js",
        JAVASCRIPT,
        include_str!("../../web/heading.js"),
    ),
    (
        "/altitude/trend.js",
        JAVASCRIPT,
        include_str!("../../web/altitude/trend.js"),
    ),
    (
        "/altitude/vsi.js",
        JAVASCRIPT,
        include_str!("../../web/altitude/vsi.js"),
    ),
    (
        "/altitude/selected.js",
        JAVASCRIPT,
        include_str!("../../web/altitude/selected.js"),
    ),
    (
        "/altitude/index.js",
        JAVASCRIPT,
        include_str!("../../web/altitude/index.js"),
    ),
    (
        "/altitude/tape.js",
        JAVASCRIPT,
        include_str!("../../web/altitude/tape.js"),
    ),
    (
        "/altitude/readout.js",
        JAVASCRIPT,
        include_str!("../../web/altitude/readout.js"),
    ),
    (
        "/altitude/baro.js",
        JAVASCRIPT,
        include_str!("../../web/altitude/baro.js"),
    ),
    (
        "/helpers.js",
        JAVASCRIPT,
        include_str!("../../web/helpers.js"),
    ),
    (
        "/airspeed/helpers.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/helpers.js"),
    ),
    (
        "/airspeed/index.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/index.js"),
    ),
    (
        "/airspeed/ias.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/ias.js"),
    ),
    (
        "/airspeed/true-airspeed.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/true-airspeed.js"),
    ),
    (
        "/airspeed/ground-speed.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/ground-speed.js"),
    ),
    (
        "/airspeed/tape.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/tape.js"),
    ),
    (
        "/airspeed/readout.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/readout.js"),
    ),
    (
        "/airspeed/references.js",
        JAVASCRIPT,
        include_str!("../../web/airspeed/references.js"),
    ),
    (
        "/layout.js",
        JAVASCRIPT,
        include_str!("../../web/layout.js"),
    ),
    ("/panel.js", JAVASCRIPT, include_str!("../../web/panel.js")),
    (
        "/frame-scheduler.js",
        JAVASCRIPT,
        include_str!("../../web/frame-scheduler.js"),
    ),
    (
        "/fixed-symbols/index.js",
        JAVASCRIPT,
        include_str!("../../web/fixed-symbols/index.js"),
    ),
    (
        "/fixed-symbols/bank-scale.js",
        JAVASCRIPT,
        include_str!("../../web/fixed-symbols/bank-scale.js"),
    ),
    (
        "/fixed-symbols/aircraft.js",
        JAVASCRIPT,
        include_str!("../../web/fixed-symbols/aircraft.js"),
    ),
    (
        "/slip-skid.js",
        JAVASCRIPT,
        include_str!("../../web/slip-skid.js"),
    ),
    (
        "/turn-rate.js",
        JAVASCRIPT,
        include_str!("../../web/turn-rate.js"),
    ),
    (
        "/status.js",
        JAVASCRIPT,
        include_str!("../../web/status.js"),
    ),
    (
        "/horizon/index.js",
        JAVASCRIPT,
        include_str!("../../web/horizon/index.js"),
    ),
    (
        "/horizon/world.js",
        JAVASCRIPT,
        include_str!("../../web/horizon/world.js"),
    ),
    (
        "/horizon/pitch-scale.js",
        JAVASCRIPT,
        include_str!("../../web/horizon/pitch-scale.js"),
    ),
    (
        "/horizon/bank-pointer.js",
        JAVASCRIPT,
        include_str!("../../web/horizon/bank-pointer.js"),
    ),
    (
        "/",
        "text/html; charset=utf-8",
        include_str!("../../web/index.html"),
    ),
    (
        "/styles.css",
        "text/css; charset=utf-8",
        include_str!("../../web/styles.css"),
    ),
    ("/app.js", JAVASCRIPT, include_str!("../../web/app.js")),
    (
        "/telemetry/validation.js",
        JAVASCRIPT,
        include_str!("../../web/telemetry/validation.js"),
    ),
    (
        "/telemetry/client.js",
        JAVASCRIPT,
        include_str!("../../web/telemetry/client.js"),
    ),
];

pub(super) fn router() -> Router {
    let mut router = Router::new();
    for &(path, content_type, body) in ASSETS {
        router = router.route(
            path,
            get(move || async move { ([(header::CONTENT_TYPE, content_type)], body) }),
        );
    }
    router
}
