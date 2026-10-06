use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{self, Attitude, STALE_AFTER, State},
};
use serde_json::{Value, json};
use tokio::time::{Duration, advance};
fn value(snapshot: telemetry::Snapshot) -> Value {
    serde_json::to_value(snapshot).unwrap()
}

#[test]
fn attitudes_match_independent_fixtures_and_repeat() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/attitudes.json")).unwrap();
    for pose in fixtures {
        let start = pose["sample"].as_u64().unwrap();
        for sample in start..start + 40 {
            for repeat in [0, 280, 560] {
                let attitude = demo::attitude(sample + repeat);
                assert_eq!(attitude.pitch_deg(), pose["pitch_deg"].as_f64().unwrap());
                assert_eq!(attitude.roll_deg(), pose["roll_deg"].as_f64().unwrap());
            }
        }
    }
}

#[test]
fn rejects_invalid_attitude_and_normalizes_roll_at_provider_boundary() {
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Attitude::new(number, 0.0).is_err());
        assert!(Attitude::new(0.0, number).is_err());
        assert!(demo::normalize_degrees(0.0, number).is_err());
    }
    for pitch in [-90.001, 90.001] {
        assert!(demo::normalize_degrees(pitch, 0.0).is_err());
    }
    for pitch in [-90.0, 90.0] {
        assert!(Attitude::new(pitch, -180.0).is_ok());
    }
    for roll in [-180.001, 180.0, 360.0] {
        assert!(Attitude::new(0.0, roll).is_err());
    }
    for (raw, expected) in [
        (180.0, -180.0),
        (540.0, -180.0),
        (-540.0, -180.0),
        (360.0, 0.0),
        (-360.0, 0.0),
        (181.0, -179.0),
        (-181.0, 179.0),
    ] {
        assert_eq!(
            demo::normalize_degrees(15.0, raw).unwrap(),
            Attitude::new(15.0, expected).unwrap()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn wire_snapshots_match_fixtures_and_unavailable_states_preserve_age() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/snapshots.json")).unwrap();
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    assert_eq!(value(subscription.snapshot()), fixtures[0]);
    publisher
        .publish(State::Live(Attitude::new(5.0, 15.0).unwrap()))
        .unwrap();
    assert_eq!(value(subscription.snapshot()), fixtures[1]);
    // Each explicit unavailable state has independently specified sequence and age.
    for (index, state) in [
        State::Paused,
        State::Waiting,
        State::Disconnected,
        State::Invalid,
    ]
    .into_iter()
    .enumerate()
    {
        let (mut publisher, subscription) = telemetry::channel(Source::Demo);
        publisher
            .publish(State::Live(Attitude::new(5.0, 15.0).unwrap()))
            .unwrap();
        for _ in 0..index {
            publisher.publish(State::Paused).unwrap();
        }
        advance(Duration::from_millis(50 * (index as u64 + 1))).await;
        publisher.publish(state).unwrap();
        assert_eq!(value(subscription.snapshot()), fixtures[index + 2]);
        advance(STALE_AFTER).await;
        publisher.expire().unwrap();
        assert_eq!(
            value(subscription.snapshot())["state"],
            fixtures[index + 2]["state"]
        );
    }
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    for _ in 0..6 {
        publisher
            .publish(State::Live(Attitude::new(5.0, 15.0).unwrap()))
            .unwrap();
    }
    advance(STALE_AFTER).await;
    publisher.expire().unwrap();
    assert_eq!(value(subscription.snapshot()), fixtures[6]);
    let (_, simulator) = telemetry::channel(Source::SimConnect);
    assert_eq!(value(simulator.snapshot())["source"], "simconnect");
}

#[tokio::test(start_paused = true)]
async fn age_expires_without_publication_and_recovers_only_on_fresh_sample() {
    let (mut publisher, mut subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(10.0, 25.0).unwrap();
    publisher.publish(State::Live(attitude)).unwrap();
    subscription.changed().await.unwrap();
    advance(Duration::from_millis(999)).await;
    assert_eq!(value(subscription.snapshot())["state"], "live");
    advance(Duration::from_millis(1)).await;
    let stale = value(subscription.snapshot());
    assert_eq!(stale["state"], "stale");
    assert_eq!(stale["attitude"], Value::Null);
    assert_eq!(stale["sample_age_ms"], 1000);
    publisher.expire().unwrap();
    assert_eq!(value(subscription.changed().await.unwrap())["sequence"], 2);
    publisher.expire().unwrap();
    assert_eq!(value(subscription.snapshot())["sequence"], 2);
    publisher.publish(State::Live(attitude)).unwrap();
    assert_eq!(value(subscription.snapshot())["sample_age_ms"], 0);
    advance(Duration::from_millis(50)).await;
    publisher.publish(State::Live(attitude)).unwrap();
    assert_eq!(value(subscription.snapshot())["sample_age_ms"], 0);
}

#[tokio::test(start_paused = true)]
async fn slow_consumers_get_only_latest_and_reading_does_not_refresh_age() {
    let (mut publisher, mut slow) = telemetry::channel(Source::Demo);
    let mut second = slow.clone();
    for _ in 0..10_000 {
        publisher.publish(State::Live(demo::attitude(40))).unwrap();
    }
    advance(Duration::from_millis(100)).await;
    for subscription in [&mut slow, &mut second] {
        let snapshot = value(subscription.changed().await.unwrap());
        assert_eq!(snapshot["sequence"], 10_000);
        assert_eq!(snapshot["sample_age_ms"], 100);
        assert_eq!(
            snapshot["attitude"],
            json!({"pitch_deg":15.0,"roll_deg":0.0})
        );
    }
    drop(publisher);
    assert!(slow.changed().await.is_err());
    assert!(second.changed().await.is_err());
}

#[tokio::test(start_paused = true)]
async fn demo_has_twenty_hertz_cadence_skips_backlog_and_closes_on_shutdown() {
    let (publisher, mut subscription) = telemetry::channel(Source::Demo);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(demo::run(publisher, async {
        let _ = stopped.await;
    }));
    assert_eq!(value(subscription.changed().await.unwrap())["sequence"], 1);
    advance(Duration::from_millis(49)).await;
    tokio::task::yield_now().await;
    assert_eq!(value(subscription.snapshot())["sequence"], 1);
    advance(Duration::from_millis(1)).await;
    assert_eq!(value(subscription.changed().await.unwrap())["sequence"], 2);
    advance(Duration::from_millis(500)).await;
    let snapshot = value(subscription.changed().await.unwrap());
    assert_eq!(snapshot["sequence"], 3);
    tokio::task::yield_now().await;
    assert_eq!(value(subscription.snapshot())["sequence"], 3);
    assert_eq!(snapshot["sample_age_ms"], 0);
    stop.send(()).unwrap();
    task.await.unwrap().unwrap();
    assert!(subscription.changed().await.is_err());
}
