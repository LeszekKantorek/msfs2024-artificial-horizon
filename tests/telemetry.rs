use msfs2024_artificial_horizon::{
    Source,
    providers::demo,
    telemetry::{
        self, Attitude, FlightSample, GroundSpeed, IndicatedAirspeed, STALE_AFTER, SlipSkid, State,
        TurnRate,
    },
};
use serde_json::{Value, json};
use tokio::time::{Duration, advance};
fn value(snapshot: telemetry::Snapshot) -> Value {
    serde_json::to_value(snapshot).unwrap()
}

#[test]
fn speed_types_reject_invalid_data_and_preserve_display_overflow() {
    for speed in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01] {
        assert!(IndicatedAirspeed::new(speed).is_err());
        assert!(GroundSpeed::new(speed).is_err());
    }
    for speed in [0.0, 99.5, 999.0, 1000.0, f64::MAX] {
        assert_eq!(IndicatedAirspeed::new(speed).unwrap().knots(), speed);
        assert_eq!(GroundSpeed::new(speed).unwrap().knots(), speed);
    }
}

#[test]
fn speed_demo_matches_independent_acceleration_hold_deceleration_and_repeat_fixtures() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/speed-samples.json")).unwrap();
    for row in fixtures {
        for repeat in [0, 360, 720] {
            let sample =
                serde_json::to_value(demo::sample(row["sample"].as_u64().unwrap() + repeat))
                    .unwrap();
            for key in ["ias_kt", "gs_kt"] {
                assert!((sample[key].as_f64().unwrap() - row[key].as_f64().unwrap()).abs() < 1e-10);
            }
        }
    }
}

#[tokio::test(start_paused = true)]
async fn speeds_share_freshness_and_clear_independently_on_partial_attitude_only_and_global_loss() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(0.0, 0.0).unwrap();
    let full = FlightSample::new(attitude, None, None).with_speeds(
        Some(IndicatedAirspeed::new(99.5).unwrap()),
        Some(GroundSpeed::new(120.0).unwrap()),
    );
    publisher.publish_sample(full).unwrap();
    advance(Duration::from_millis(999)).await;
    assert_eq!(value(subscription.snapshot())["ias_kt"], 99.5);
    publisher.publish_sample(full).unwrap();
    assert_eq!(value(subscription.snapshot())["sample_age_ms"], 0);
    for (ias, gs) in [
        (Some(IndicatedAirspeed::new(0.0).unwrap()), None),
        (None, Some(GroundSpeed::new(1000.0).unwrap())),
        (None, None),
    ] {
        publisher
            .publish_sample(FlightSample::new(attitude, None, None).with_speeds(ias, gs))
            .unwrap();
        let snapshot = value(subscription.snapshot());
        assert_eq!(snapshot.get("ias_kt").is_some(), ias.is_some());
        assert_eq!(snapshot.get("gs_kt").is_some(), gs.is_some());
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
        let snapshot = value(subscription.snapshot());
        assert!(snapshot.get("ias_kt").is_none());
        assert!(snapshot.get("gs_kt").is_none());
    }
    publisher.publish_sample(full).unwrap();
    advance(STALE_AFTER).await;
    let stale = value(subscription.snapshot());
    assert_eq!(stale["state"], "stale");
    assert!(stale.get("ias_kt").is_none());
    assert!(stale.get("gs_kt").is_none());
}

#[test]
fn pfd_demo_matches_independent_fixtures_through_every_segment_and_repeat() {
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/pfd-samples.json")).unwrap();
    for mut fixture in fixtures {
        let start = fixture
            .as_object_mut()
            .unwrap()
            .remove("sample")
            .unwrap()
            .as_u64()
            .unwrap();
        for index in start..start + 40 {
            for repeat in [0, 360, 720] {
                let mut actual = serde_json::to_value(demo::sample(index + repeat)).unwrap();
                actual.as_object_mut().unwrap().remove("ias_kt");
                actual.as_object_mut().unwrap().remove("gs_kt");
                assert_eq!(actual, fixture);
            }
        }
    }
}

#[test]
fn optional_indications_reject_invalid_numbers_without_clamping() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(SlipSkid::new(value).is_err());
        assert!(TurnRate::new(value).is_err());
    }
    for value in [-1.001, 1.001] {
        assert!(SlipSkid::new(value).is_err());
    }
    for value in [-1.0, 0.0, 1.0] {
        assert_eq!(SlipSkid::new(value).unwrap().normalized(), value);
    }
    // The display range is not a telemetry validity limit.
    for value in [-12.0, -3.0, 0.0, 3.0, 12.0] {
        assert_eq!(TurnRate::new(value).unwrap().degrees_per_second(), value);
    }
}

#[tokio::test(start_paused = true)]
async fn extended_publication_shares_age_and_clears_absent_or_unavailable_indications() {
    let (mut publisher, subscription) = telemetry::channel(Source::Demo);
    let attitude = Attitude::new(10.0, 25.0).unwrap();
    publisher.publish_sample(demo::sample(200)).unwrap();
    let full = value(subscription.snapshot());
    assert_eq!(
        full,
        json!({"schema_version":1,"sequence":1,"source":"demo","state":"live",
        "sample_age_ms":0,"attitude":{"pitch_deg":10.0,"roll_deg":25.0},"slip_skid":1.0,"turn_rate_dps":3.0,"ias_kt":150.0,"gs_kt":170.0})
    );
    advance(Duration::from_millis(999)).await;
    assert_eq!(value(subscription.snapshot())["slip_skid"], 1.0);
    advance(Duration::from_millis(1)).await;
    let stale = value(subscription.snapshot());
    assert_eq!(stale["state"], "stale");
    assert!(stale.get("slip_skid").is_none());
    assert!(stale.get("turn_rate_dps").is_none());
    for (slip, turn) in [
        (Some(SlipSkid::new(-0.5).unwrap()), None),
        (None, Some(TurnRate::new(-3.0).unwrap())),
        (None, None),
    ] {
        publisher
            .publish_sample(FlightSample::new(attitude, slip, turn))
            .unwrap();
        let partial = value(subscription.snapshot());
        assert_eq!(partial["sample_age_ms"], 0);
        assert_eq!(partial.get("slip_skid").is_some(), slip.is_some());
        assert_eq!(partial.get("turn_rate_dps").is_some(), turn.is_some());
    }
    publisher.publish_sample(demo::sample(200)).unwrap();
    publisher.publish(State::Live(attitude)).unwrap();
    let old = value(subscription.snapshot());
    assert!(old.get("slip_skid").is_none());
    assert!(old.get("turn_rate_dps").is_none());
    for state in [
        State::Paused,
        State::Waiting,
        State::Disconnected,
        State::Invalid,
        State::Stale,
    ] {
        publisher.publish_sample(demo::sample(200)).unwrap();
        publisher.publish(state).unwrap();
        let unavailable = value(subscription.snapshot());
        assert!(unavailable.get("slip_skid").is_none());
        assert!(unavailable.get("turn_rate_dps").is_none());
    }
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
