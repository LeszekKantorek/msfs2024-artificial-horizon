use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{
        self, AirspeedProfile, Attitude, FlightSample, IasAcceleration, STALE_AFTER, State,
        VSpeedReferences,
    },
};
use serde_json::Value;
use tokio::time::{Duration, advance};

fn profile_fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/airspeed-profile.json")).unwrap()
}

#[test]
fn profile_validates_numbers_without_coupling_thresholds_and_references() {
    let valid = [40.0, 50.0, 85.0, 110.0, 140.0];
    let refs = VSpeedReferences::new(55.0, 65.0, 75.0, 90.0).unwrap();
    for index in 0..5 {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0] {
            let mut v = valid;
            v[index] = bad;
            assert!(AirspeedProfile::new(v[0], v[1], v[2], v[3], v[4], refs).is_err());
        }
    }
    for v in [
        [40.0, 39.0, 85.0, 110.0, 140.0],
        [40.0, 50.0, 49.0, 110.0, 140.0],
        [40.0, 50.0, 85.0, 85.0, 140.0],
        [40.0, 50.0, 85.0, 110.0, 110.0],
    ] {
        assert!(AirspeedProfile::new(v[0], v[1], v[2], v[3], v[4], refs).is_ok());
    }
    for index in 0..4 {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0] {
            let mut v = [55.0, 65.0, 75.0, 90.0];
            v[index] = bad;
            assert!(VSpeedReferences::new(v[0], v[1], v[2], v[3]).is_err());
        }
    }
    assert!(
        AirspeedProfile::new(
            40.0,
            50.0,
            85.0,
            110.0,
            140.0,
            VSpeedReferences::new(55.0, 65.0, 75.0, 141.0).unwrap()
        )
        .is_ok()
    );
    assert_eq!(
        serde_json::to_value(demo::speed_profile()).unwrap(),
        profile_fixture()
    );
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(IasAcceleration::new(bad).is_err());
    }
    for v in [-25.0, 0.0, 25.0, f64::MAX] {
        assert_eq!(IasAcceleration::new(v).unwrap().knots_per_second(), v);
    }
}

#[test]
fn demo_matches_independent_acceleration_fixtures_at_segment_boundaries_and_repeats() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/speed-indications.json")).unwrap();
    for row in fixtures {
        for repeat in [0, 360, 720] {
            let sample =
                serde_json::to_value(demo::sample(row["sample"].as_u64().unwrap() + repeat))
                    .unwrap();
            assert_eq!(sample["airspeed_profile"], profile_fixture());
            assert_eq!(
                sample["ias_acceleration_ktps"].as_f64(),
                row["ias_acceleration_ktps"].as_f64()
            );
        }
    }
}

#[tokio::test(start_paused = true)]
async fn new_fields_share_age_clear_independently_and_never_survive_attitude_only_or_global_loss() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    let full = demo::sample(120);
    publisher.publish_sample(full.clone()).unwrap();
    advance(Duration::from_millis(999)).await;
    assert_eq!(
        serde_json::to_value(subscription.snapshot()).unwrap()["airspeed_profile"],
        profile_fixture()
    );
    publisher.publish_sample(full.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(subscription.snapshot()).unwrap()["sample_age_ms"],
        0
    );
    for (profile, acceleration) in [
        (Some(demo::speed_profile()), None),
        (None, Some(IasAcceleration::new(-5.0).unwrap())),
        (None, None),
    ] {
        publisher
            .publish_sample(
                FlightSample::new(attitude, None, None)
                    .with_airspeed_indications(profile, acceleration),
            )
            .unwrap();
        let value = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(value.get("airspeed_profile").is_some(), profile.is_some());
        assert_eq!(
            value.get("ias_acceleration_ktps").is_some(),
            acceleration.is_some()
        );
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
        let value = serde_json::to_value(subscription.snapshot()).unwrap();
        assert!(value.get("airspeed_profile").is_none());
        assert!(value.get("ias_acceleration_ktps").is_none());
    }
    publisher.publish_sample(full.clone()).unwrap();
    advance(STALE_AFTER).await;
    let stale = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(stale["state"], "stale");
    assert!(stale.get("airspeed_profile").is_none());
    assert!(stale.get("ias_acceleration_ktps").is_none());
}
