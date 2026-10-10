//! Repeatable degree fixtures; no simulator SDK conventions are assumed.
use crate::telemetry::{
    Attitude, FlightSample, GroundSpeed, IndicatedAirspeed, Publisher, SAMPLE_PERIOD, SlipSkid,
    TelemetryError, TurnRate,
};
use std::future::Future;
use tokio::time::{MissedTickBehavior, interval};
pub fn attitude(sample: u64) -> Attitude {
    let (pitch, roll) = match (sample / 40) % 7 {
        0 => (0.0, 0.0),
        1 => (15.0, 0.0),
        2 => (-15.0, 0.0),
        3 => (0.0, -30.0),
        4 => (0.0, 30.0),
        5 => (10.0, 25.0),
        _ => (-10.0, -25.0),
    };
    normalize_degrees(pitch, roll).expect("built-in demo poses are valid")
}
/// Nine two-second segments. The original attitude helper retains its seven-pose cycle.
pub fn sample(index: u64) -> FlightSample {
    let phase = (index / 40) % 9;
    let attitude = match phase {
        7 => Attitude::new(70.0, 0.0).unwrap(),
        8 => Attitude::new(-50.0, 0.0).unwrap(),
        _ => attitude(phase * 40),
    };
    let (slip, turn) = match phase {
        1 => (-0.5, 0.0),
        2 => (0.5, 0.0),
        3 => (0.0, -3.0),
        4 => (0.0, 3.0),
        5 => (1.0, 3.0),
        6 => (-1.0, -3.0),
        _ => (0.0, 0.0),
    };
    let step = index % 360;
    let fraction = match step {
        0..40 | 320..360 => 0.0,
        40..160 => (step - 40) as f64 / 120.0,
        160..200 => 1.0,
        _ => (320 - step) as f64 / 120.0,
    };
    FlightSample::new(
        attitude,
        Some(SlipSkid::new(slip).expect("built-in ball positions are valid")),
        Some(TurnRate::new(turn).expect("built-in turn rates are valid")),
    )
    .with_speeds(
        Some(IndicatedAirspeed::new(150.0 * fraction).expect("built-in IAS is valid")),
        Some(GroundSpeed::new(170.0 * fraction).expect("built-in GS is valid")),
    )
}
/// Provider boundary for verified degree/sign conventions.
pub fn normalize_degrees(pitch: f64, roll: f64) -> Result<Attitude, TelemetryError> {
    if !roll.is_finite() {
        return Err(TelemetryError::InvalidAttitude);
    }
    let wrapped = roll.rem_euclid(360.0);
    Attitude::new(
        pitch,
        if wrapped >= 180.0 {
            wrapped - 360.0
        } else {
            wrapped
        },
    )
}
pub async fn run(
    mut publisher: Publisher,
    shutdown: impl Future<Output = ()>,
) -> Result<(), TelemetryError> {
    let mut samples = interval(SAMPLE_PERIOD);
    let mut freshness = interval(SAMPLE_PERIOD);
    samples.set_missed_tick_behavior(MissedTickBehavior::Skip);
    freshness.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut index = 0;
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => return Ok(()),
            _ = freshness.tick() => publisher.expire()?,
            _ = samples.tick() => {
                publisher.publish_sample(sample(index))?;
                index = (index + 1) % 360;
            }
        }
    }
}
