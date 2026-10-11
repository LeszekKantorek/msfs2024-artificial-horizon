use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{self, Attitude, BarometricAltitude, FlightSample, STALE_AFTER, State},
};
use serde_json::Value;
use tokio::time::{Duration, advance};

#[test]
fn altitude_validates_feet_without_imposing_display_limits() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(BarometricAltitude::new(bad).is_err());
    }
    for feet in [-f64::MAX, -10000.0, -0.1, 0.0, 99999.0, f64::MAX] {
        assert_eq!(BarometricAltitude::new(feet).unwrap().feet(), feet);
    }
}

#[test]
fn shared_demo_matches_independent_altitude_sweeps_pressure_changes_and_repeats() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/altitude-samples.json")).unwrap();
    for row in fixtures {
        for repeat in [0, 3960, 7920] {
            let sample =
                serde_json::to_value(demo::sample(row["sample"].as_u64().unwrap() + repeat))
                    .unwrap();
            assert!(
                (sample["altitude_ft"].as_f64().unwrap() - row["altitude_ft"].as_f64().unwrap())
                    .abs()
                    < 1e-10
            );
            assert_eq!(sample["baro_text"], row["baro_text"]);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn altitude_and_baro_clear_independently_and_share_sample_freshness() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    let altitude = Some(BarometricAltitude::new(-0.1).unwrap());
    let baro = Some(String::from("29.92 inHg"));
    let full = FlightSample::new(attitude, None, None).with_altitude(altitude, baro.clone());
    publisher.publish_sample(full.clone()).unwrap();
    advance(Duration::from_millis(999)).await;
    let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(snapshot["altitude_ft"], -0.1);
    assert_eq!(snapshot["baro_text"], "29.92 inHg");
    assert_eq!(snapshot["sample_age_ms"], 999);
    publisher.publish_sample(full.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(subscription.snapshot()).unwrap()["sample_age_ms"],
        0
    );
    for (altitude, baro) in [(altitude, None), (None, baro), (None, None)] {
        let has_baro = baro.is_some();
        publisher
            .publish_sample(FlightSample::new(attitude, None, None).with_altitude(altitude, baro))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot.get("altitude_ft").is_some(), altitude.is_some());
        assert_eq!(snapshot.get("baro_text").is_some(), has_baro);
        assert_eq!(snapshot["attitude"]["pitch_deg"], 0.0);
    }
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
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert!(snapshot.get("altitude_ft").is_none());
        assert!(snapshot.get("baro_text").is_none());
    }
    publisher.publish_sample(full.clone()).unwrap();
    advance(STALE_AFTER).await;
    let stale = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(stale["state"], "stale");
    assert!(stale.get("altitude_ft").is_none());
    assert!(stale.get("baro_text").is_none());
}

#[test]
fn baro_text_is_preserved_across_publication_and_independent_snapshots() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    for text in [
        "23.83 inHg",
        "1026 hPa",
        "BARO",
        "",
        "  BARO  ",
        "<b>BARO</b>",
    ] {
        let sample =
            FlightSample::new(attitude, None, None).with_altitude(None, Some(text.to_owned()));
        assert_eq!(serde_json::to_value(&sample).unwrap()["baro_text"], text);
        publisher.publish_sample(sample).unwrap();
        for _ in 0..2 {
            let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
            assert_eq!(snapshot["baro_text"], text);
            assert!(snapshot.get("altitude_ft").is_none());
        }
    }
    publisher
        .publish_sample(FlightSample::new(attitude, None, None))
        .unwrap();
    assert!(
        serde_json::to_value(subscription.snapshot())
            .unwrap()
            .get("baro_text")
            .is_none()
    );
}
