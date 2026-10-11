use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{self, Attitude, FlightSample, STALE_AFTER, SelectedAltitude, State},
};
use serde_json::Value;
use tokio::time::{Duration, advance};

#[test]
fn selected_altitude_validates_signed_finite_feet_without_a_display_limit() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(SelectedAltitude::new(bad).is_err());
    }
    for feet in [-f64::MAX, -10000.0, -0.1, 0.0, 99999.0, f64::MAX] {
        assert_eq!(SelectedAltitude::new(feet).unwrap().feet(), feet);
    }
}

#[test]
fn demo_matches_independent_198_second_checkpoints_and_keeps_other_cycles() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/selected-altitude-samples.json")).unwrap();
    for row in fixtures {
        let index = row["sample"].as_u64().unwrap();
        for repeat in [0, 3960, 7920] {
            let sample = serde_json::to_value(demo::sample(index + repeat)).unwrap();
            assert_eq!(sample["altitude_ft"].as_f64(), row["altitude_ft"].as_f64());
            assert_eq!(
                sample["selected_altitude_ft"].as_f64(),
                row["selected_altitude_ft"].as_f64()
            );
        }
    }
    for index in 0..3960 {
        let mut current = serde_json::to_value(demo::sample(index)).unwrap();
        let mut original = serde_json::to_value(demo::sample(index % 360)).unwrap();
        for sample in [&mut current, &mut original] {
            sample.as_object_mut().unwrap().remove("altitude_ft");
            sample.as_object_mut().unwrap().remove("vertical_speed_fpm");
            sample
                .as_object_mut()
                .unwrap()
                .remove("selected_altitude_ft");
        }
        assert_eq!(current, original);
    }
}

#[tokio::test(start_paused = true)]
async fn selected_altitude_is_optional_and_clears_with_the_shared_deadline() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    let empty = FlightSample::new(attitude, None, None);
    for feet in [-120.0, 0.0, 100000.0] {
        let full = empty.with_selected_altitude(Some(SelectedAltitude::new(feet).unwrap()));
        publisher.publish_sample(full).unwrap();
        advance(Duration::from_millis(999)).await;
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["selected_altitude_ft"], feet);
        assert_eq!(snapshot["sample_age_ms"], 999);
        publisher.publish_sample(full).unwrap();
        assert_eq!(
            serde_json::to_value(subscription.snapshot()).unwrap()["sample_age_ms"],
            0
        );
        advance(STALE_AFTER).await;
        let expired = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(expired["state"], "stale");
        assert!(expired.get("selected_altitude_ft").is_none());
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
            assert!(
                serde_json::to_value(subscription.snapshot())
                    .unwrap()
                    .get("selected_altitude_ft")
                    .is_none()
            );
        }
        publisher.publish_sample(full).unwrap();
        publisher.publish_sample(empty).unwrap();
        let old = serde_json::to_value(subscription.snapshot()).unwrap();
        assert!(old.get("selected_altitude_ft").is_none());
        assert_eq!(old["attitude"]["pitch_deg"], 0.0);
    }
}
