use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{
        self, Attitude, BarometricAltitude, BarometricSetting, FlightSample, STALE_AFTER, State,
    },
};
use serde_json::Value;
use tokio::time::{Duration, advance};

#[test]
fn altitude_and_pressure_validate_units_without_imposing_display_limits() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(BarometricAltitude::new(bad).is_err());
        assert!(BarometricSetting::new(bad).is_err());
    }
    for feet in [-f64::MAX, -10000.0, -0.1, 0.0, 99999.0, f64::MAX] {
        assert_eq!(BarometricAltitude::new(feet).unwrap().feet(), feet);
    }
    for bad in [0.0, -0.0, -1.0] {
        assert!(BarometricSetting::new(bad).is_err());
    }
    for setting in [0.001, 29.92, 99.99, f64::MAX] {
        assert_eq!(
            BarometricSetting::new(setting).unwrap().inches_of_mercury(),
            setting
        );
    }
}

#[test]
fn shared_demo_matches_independent_altitude_sweeps_pressure_changes_and_repeats() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/altitude-samples.json")).unwrap();
    for row in fixtures {
        for repeat in [0, 360, 720] {
            let sample =
                serde_json::to_value(demo::sample(row["sample"].as_u64().unwrap() + repeat))
                    .unwrap();
            assert!(
                (sample["altitude_ft"].as_f64().unwrap() - row["altitude_ft"].as_f64().unwrap())
                    .abs()
                    < 1e-10
            );
            assert_eq!(sample["baro_inhg"], row["baro_inhg"]);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn altitude_and_baro_clear_independently_and_share_sample_freshness() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    let altitude = Some(BarometricAltitude::new(-0.1).unwrap());
    let baro = Some(BarometricSetting::new(29.92).unwrap());
    let full = FlightSample::new(attitude, None, None).with_altitude(altitude, baro);
    publisher.publish_sample(full).unwrap();
    advance(Duration::from_millis(999)).await;
    let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(snapshot["altitude_ft"], -0.1);
    assert_eq!(snapshot["baro_inhg"], 29.92);
    assert_eq!(snapshot["sample_age_ms"], 999);
    publisher.publish_sample(full).unwrap();
    assert_eq!(
        serde_json::to_value(subscription.snapshot()).unwrap()["sample_age_ms"],
        0
    );
    for (altitude, baro) in [(altitude, None), (None, baro), (None, None)] {
        publisher
            .publish_sample(FlightSample::new(attitude, None, None).with_altitude(altitude, baro))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot.get("altitude_ft").is_some(), altitude.is_some());
        assert_eq!(snapshot.get("baro_inhg").is_some(), baro.is_some());
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
        publisher.publish_sample(full).unwrap();
        publisher.publish(state).unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert!(snapshot.get("altitude_ft").is_none());
        assert!(snapshot.get("baro_inhg").is_none());
    }
    publisher.publish_sample(full).unwrap();
    advance(STALE_AFTER).await;
    let stale = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(stale["state"], "stale");
    assert!(stale.get("altitude_ft").is_none());
    assert!(stale.get("baro_inhg").is_none());
}
