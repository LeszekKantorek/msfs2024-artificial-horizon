//! Repeatable degree fixtures; no simulator SDK conventions are assumed.
use crate::telemetry::{
    AirspeedProfile, Attitude, BarometricAltitude, BarometricSetting, FlightSample, GroundSpeed,
    IasAcceleration, IndicatedAirspeed, Publisher, SAMPLE_PERIOD, SlipSkid, TelemetryError,
    TurnRate, VSpeedReferences,
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
    // Separate synthetic sweeps exercise signs and carries; jumps are not a flight model.
    let (start, end) = [
        (-120.0, 120.0),
        (980.0, 1020.0),
        (9980.0, 10020.0),
        (10020.0, 9980.0),
        (1020.0, 980.0),
        (120.0, -120.0),
    ][(step / 60) as usize];
    let altitude = start + (end - start) * (step % 60) as f64 / 59.0;
    let baro = [29.92, 30.12, 29.72][(step / 120) as usize];
    FlightSample::new(
        attitude,
        Some(SlipSkid::new(slip).expect("built-in ball positions are valid")),
        Some(TurnRate::new(turn).expect("built-in turn rates are valid")),
    )
    .with_altitude(
        Some(BarometricAltitude::new(altitude).expect("built-in altitude is valid")),
        Some(BarometricSetting::new(baro).expect("built-in pressure setting is valid")),
    )
    .with_speeds(
        Some(IndicatedAirspeed::new(150.0 * fraction).expect("built-in IAS is valid")),
        Some(GroundSpeed::new(170.0 * fraction).expect("built-in GS is valid")),
    )
    .with_airspeed_indications(
        Some(speed_profile()),
        Some(
            IasAcceleration::new(match step {
                40..160 => 25.0,
                200..320 => -25.0,
                _ => 0.0,
            })
            .expect("built-in IAS acceleration is valid"),
        ),
    )
}
/// Fictional demo configuration, not verified performance of any real aircraft.
pub fn speed_profile() -> AirspeedProfile {
    AirspeedProfile::new(
        40.0,
        50.0,
        85.0,
        110.0,
        140.0,
        VSpeedReferences::new(55.0, 65.0, 75.0, 90.0).expect("built-in references are valid"),
    )
    .expect("built-in speed thresholds are coherent")
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
