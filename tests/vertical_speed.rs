use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{self, Attitude, FlightSample, State, VerticalSpeed},
};
use serde_json::Value;
use tokio::time::{Duration, advance};

fn sample(index: u64) -> Value {
    serde_json::to_value(demo::sample(index)).unwrap()
}

#[test]
fn climb_of_twenty_five_feet_per_second_is_positive_fifteen_hundred_fpm() {
    let start = sample(480);
    let later = sample(500);
    assert_eq!(
        later["altitude_ft"].as_f64().unwrap() - start["altitude_ft"].as_f64().unwrap(),
        25.0
    );
    assert_eq!(start["vertical_speed_fpm"], 1500.0);
    assert_eq!(later["vertical_speed_fpm"], 1500.0);
    assert_eq!(sample(2280)["vertical_speed_fpm"], -1500.0);
    assert_eq!(sample(3480)["vertical_speed_fpm"], 0.0);
}

#[test]
fn vertical_speed_validates_finite_signed_units_without_display_limits() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(VerticalSpeed::new(bad).is_err());
    }
    for fpm in [-f64::MAX, -10000.0, -0.0, 0.0, 2000.0, 9999.0, f64::MAX] {
        assert_eq!(VerticalSpeed::new(fpm).unwrap().feet_per_minute(), fpm);
    }
}

#[test]
fn independent_checkpoints_cover_slopes_phase_boundaries_jumps_and_repeated_cycles() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/vertical-speed-samples.json")).unwrap();
    for row in rows {
        for repeat in [0, 3960, 7920] {
            let actual = sample(row["sample"].as_u64().unwrap() + repeat);
            for key in ["altitude_ft", "selected_altitude_ft", "vertical_speed_fpm"] {
                match row[key].as_f64() {
                    Some(expected) => assert!(
                        (actual[key].as_f64().unwrap() - expected).abs() < 1e-9,
                        "{key} at {}",
                        row["sample"]
                    ),
                    None => assert!(actual.get(key).is_none(), "{key} at {}", row["sample"]),
                }
            }
        }
    }
    for index in 360..3960 {
        let current = sample(index);
        let next = sample(index + 1);
        if next["vertical_speed_fpm"].is_null() || current["vertical_speed_fpm"].is_null() {
            continue;
        }
        let change =
            next["altitude_ft"].as_f64().unwrap() - current["altitude_ft"].as_f64().unwrap();
        assert!((change * 1200.0 - current["vertical_speed_fpm"].as_f64().unwrap()).abs() < 1e-9);
    }
}

#[tokio::test(start_paused = true)]
async fn vertical_speed_shares_freshness_and_clears_on_old_samples_and_states() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    let empty = FlightSample::new(attitude, None, None);
    for fpm in [-1500.0, 0.0, 10000.0] {
        let full = empty.with_vertical_speed(Some(VerticalSpeed::new(fpm).unwrap()));
        publisher.publish_sample(full).unwrap();
        advance(Duration::from_millis(999)).await;
        let current = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(current["vertical_speed_fpm"], fpm);
        assert_eq!(current["sample_age_ms"], 999);
        assert!(current.get("altitude_ft").is_none());
        assert!(current.get("baro_inhg").is_none());
        advance(Duration::from_millis(1)).await;
        let stale = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(stale["state"], "stale");
        assert!(stale.get("vertical_speed_fpm").is_none());
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
                    .get("vertical_speed_fpm")
                    .is_none()
            );
        }
        publisher.publish_sample(full).unwrap();
        publisher.publish_sample(empty).unwrap();
        assert!(
            serde_json::to_value(subscription.snapshot())
                .unwrap()
                .get("vertical_speed_fpm")
                .is_none()
        );
    }
}
