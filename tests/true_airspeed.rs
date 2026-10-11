use msfs2024_artificial_horizon::{
    Source,
    telemetry::{
        self, Attitude, FlightSample, GroundSpeed, IndicatedAirspeed, STALE_AFTER, State,
        TrueAirspeed,
    },
};
use serde_json::Value;
use tokio::time::{Duration, advance};

#[test]
fn true_airspeed_rejects_invalid_numbers_without_a_telemetry_display_limit() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01] {
        assert!(TrueAirspeed::new(value).is_err());
    }
    for value in [0.0, 85.5, 999.0, 999.01, 1000.0, f64::MAX] {
        assert_eq!(TrueAirspeed::new(value).unwrap().knots(), value);
    }
}

#[tokio::test(start_paused = true)]
async fn true_airspeed_is_independent_and_shares_the_sample_deadline() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(5.0, 15.0).unwrap();
    let full = FlightSample::new(attitude, None, None)
        .with_speeds(
            Some(IndicatedAirspeed::new(150.0).unwrap()),
            Some(GroundSpeed::new(170.0).unwrap()),
        )
        .with_true_airspeed(Some(TrueAirspeed::new(160.0).unwrap()));
    for (ias, gs, tas) in [
        (true, true, true),
        (true, true, false),
        (false, true, true),
        (true, false, true),
        (false, false, true),
    ] {
        let sample = full
            .clone()
            .with_speeds(
                ias.then(|| IndicatedAirspeed::new(150.0).unwrap()),
                gs.then(|| GroundSpeed::new(170.0).unwrap()),
            )
            .with_true_airspeed(tas.then(|| TrueAirspeed::new(160.0).unwrap()));
        publisher.publish_sample(sample).unwrap();
        let wire = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(wire.get("ias_kt").is_some(), ias);
        assert_eq!(wire.get("gs_kt").is_some(), gs);
        assert_eq!(wire.get("tas_kt").is_some(), tas);
        assert_eq!(wire["attitude"]["pitch_deg"], 5.0);
        assert_eq!(wire["schema_version"], 1);
    }
    publisher.publish_sample(full.clone()).unwrap();
    advance(Duration::from_millis(999)).await;
    let fresh: Value = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(fresh["tas_kt"], 160.0);
    assert_eq!(fresh["sample_age_ms"], 999);
    let retained = subscription.snapshot();
    advance(Duration::from_millis(1)).await;
    let stale = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(stale["state"], "stale");
    for key in ["ias_kt", "gs_kt", "tas_kt"] {
        assert!(stale.get(key).is_none());
    }
    assert_eq!(serde_json::to_value(retained).unwrap()["tas_kt"], 160.0);
    publisher.publish_sample(full.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(subscription.snapshot()).unwrap()["sample_age_ms"],
        0
    );
    for state in [
        State::Live(attitude),
        State::Waiting,
        State::Paused,
        State::Stale,
        State::Disconnected,
        State::Invalid,
    ] {
        publisher.publish_sample(full.clone()).unwrap();
        publisher.publish(state).unwrap();
        assert!(
            serde_json::to_value(subscription.snapshot())
                .unwrap()
                .get("tas_kt")
                .is_none()
        );
    }
    publisher.publish_sample(full).unwrap();
    advance(STALE_AFTER).await;
    publisher.expire().unwrap();
    assert!(
        serde_json::to_value(subscription.snapshot())
            .unwrap()
            .get("tas_kt")
            .is_none()
    );
    publisher
        .publish_sample(FlightSample::new(attitude, None, None))
        .unwrap();
    let old = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(old["state"], "live");
    assert!(old.get("tas_kt").is_none());
}
