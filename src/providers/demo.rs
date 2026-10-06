//! Repeatable degree fixtures; no simulator SDK conventions are assumed.
use crate::telemetry::{Attitude, Publisher, SAMPLE_PERIOD, State, TelemetryError};
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
    let mut sample = 0;
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => return Ok(()),
            _ = freshness.tick() => publisher.expire()?,
            _ = samples.tick() => {
                publisher.publish(State::Live(attitude(sample)))?;
                sample = (sample + 1) % 280;
            }
        }
    }
}
