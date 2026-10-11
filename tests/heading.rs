use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{self, Attitude, FlightSample, Heading, STALE_AFTER, SelectedHeading, State},
};
use serde_json::Value;
use tokio::time::{Duration, advance};

fn sample(index: u64) -> Value {
    serde_json::to_value(demo::sample(index)).unwrap()
}

#[test]
fn headings_require_finite_canonical_magnetic_degrees_including_north() {
    for bad in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -0.001,
        360.0,
        720.0,
        f64::MAX,
    ] {
        assert!(Heading::new(bad).is_err());
        assert!(SelectedHeading::new(bad).is_err());
    }
    for degrees in [-0.0, 0.0, 0.001, 90.0, 180.0, 359.999] {
        assert_eq!(Heading::new(degrees).unwrap().degrees(), degrees);
        assert_eq!(SelectedHeading::new(degrees).unwrap().degrees(), degrees);
    }
}

#[test]
fn independent_heading_checkpoints_repeat_and_follow_source_turn_rate_across_north() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/heading-samples.json")).unwrap();
    for row in rows {
        for repeat in [0, 360, 3960, 7920] {
            let actual = sample(row["sample"].as_u64().unwrap() + repeat);
            for key in ["heading_deg", "selected_heading_deg", "turn_rate_dps"] {
                if let Some(expected) = row[key].as_f64() {
                    assert!(
                        (actual[key].as_f64().unwrap() - expected).abs() < 1e-9,
                        "{key} at {}",
                        row["sample"]
                    );
                } else {
                    assert!(actual.get(key).is_none());
                }
            }
        }
    }
    for index in 0..3960 {
        let current = sample(index);
        let next = sample(index + 1);
        let delta = (next["heading_deg"].as_f64().unwrap()
            - current["heading_deg"].as_f64().unwrap()
            + 180.0)
            .rem_euclid(360.0)
            - 180.0;
        assert!(
            (delta * 20.0 - current["turn_rate_dps"].as_f64().unwrap()).abs() < 1e-9,
            "turn rate at {index}"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn headings_share_freshness_and_clear_independently_on_old_publications_and_states() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    let empty = FlightSample::new(attitude, None, None);
    let heading = Heading::new(359.0).unwrap();
    let selected = SelectedHeading::new(1.0).unwrap();
    let full = empty.clone().with_heading(Some(heading), Some(selected));
    publisher.publish_sample(full.clone()).unwrap();
    advance(Duration::from_millis(999)).await;
    let current = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(current["heading_deg"], 359.0);
    assert_eq!(current["selected_heading_deg"], 1.0);
    assert_eq!(current["sample_age_ms"], 999);
    advance(Duration::from_millis(1)).await;
    let stale = serde_json::to_value(subscription.snapshot()).unwrap();
    assert_eq!(stale["state"], "stale");
    assert!(stale.get("heading_deg").is_none() && stale.get("selected_heading_deg").is_none());
    for (h, s) in [(Some(heading), None), (None, Some(selected)), (None, None)] {
        publisher
            .publish_sample(empty.clone().with_heading(h, s))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot.get("heading_deg").is_some(), h.is_some());
        assert_eq!(snapshot.get("selected_heading_deg").is_some(), s.is_some());
        assert_eq!(snapshot["sample_age_ms"], 0);
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
        assert!(
            snapshot.get("heading_deg").is_none() && snapshot.get("selected_heading_deg").is_none()
        );
    }
    publisher.publish_sample(full.clone()).unwrap();
    publisher.publish_sample(empty.clone()).unwrap();
    assert!(
        serde_json::to_value(subscription.snapshot())
            .unwrap()
            .get("heading_deg")
            .is_none()
    );
    publisher.publish_sample(full.clone()).unwrap();
    advance(STALE_AFTER).await;
    publisher.expire().unwrap();
    assert!(
        serde_json::to_value(subscription.snapshot())
            .unwrap()
            .get("selected_heading_deg")
            .is_none()
    );
}
