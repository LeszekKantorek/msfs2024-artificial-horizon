//! Repeatable degree fixtures; no simulator SDK conventions are assumed.
use crate::telemetry::{
    AirspeedProfile, Attitude, BaroSetting, BarometricAltitude, FlightSample, GroundSpeed,
    HPA_PER_INHG, Heading, IasAcceleration, IndicatedAirspeed, Publisher, SAMPLE_PERIOD,
    SelectedAltitude, SelectedHeading, SlipSkid, TelemetryError, TrueAirspeed, TurnRate,
    VSpeedReferences, VerticalSpeed,
};
use std::future::Future;
use tokio::time::{MissedTickBehavior, interval};
const CYCLE_SAMPLES: u64 = 3960;
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
    let (altitude, selected, vertical_speed) = altitude_scenario(index);
    let baro = [29.92, 30.12, 29.72][(step / 120) as usize];
    FlightSample::new(
        attitude,
        Some(SlipSkid::new(slip).expect("built-in ball positions are valid")),
        Some(TurnRate::new(turn).expect("built-in turn rates are valid")),
    )
    .with_heading(
        Some(Heading::new(heading_scenario(step)).expect("built-in heading is valid")),
        match phase {
            0 | 3 | 4 => Some(350.0),
            1 => Some(90.0),
            2 => Some(180.0),
            5 | 6 => Some(0.0),
            7 => None,
            _ => Some(354.0),
        }
        .map(|degrees| SelectedHeading::new(degrees).expect("built-in selected heading is valid")),
    )
    .with_altitude(
        Some(BarometricAltitude::new(altitude).expect("built-in altitude is valid")),
        Some(format!("{baro:.2} inHg")),
    )
    .with_baro_setting(Some(BaroSetting::new(baro * HPA_PER_INHG).unwrap()))
    .with_selected_altitude(
        selected.map(|feet| SelectedAltitude::new(feet).expect("built-in target is valid")),
    )
    .with_vertical_speed(
        vertical_speed
            .map(|fpm| VerticalSpeed::new(fpm).expect("built-in vertical speed is valid")),
    )
    .with_speeds(
        Some(IndicatedAirspeed::new(150.0 * fraction).expect("built-in IAS is valid")),
        Some(GroundSpeed::new(170.0 * fraction).expect("built-in GS is valid")),
    )
    .with_true_airspeed(Some(
        TrueAirspeed::new(160.0 * fraction).expect("built-in TAS is valid"),
    ))
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
// Continuous heading follows the existing signed turn-rate phases.
fn heading_scenario(step: u64) -> f64 {
    let elapsed = (step % 40) as f64 * SAMPLE_PERIOD.as_secs_f64();
    match step / 40 {
        3 => 354.0 - 3.0 * elapsed,
        4 => 348.0 + 3.0 * elapsed,
        5 => (354.0 + 3.0 * elapsed).rem_euclid(360.0),
        6 => (-3.0 * elapsed).rem_euclid(360.0),
        _ => 354.0,
    }
}
// Heights and their analytical slopes share the same segment definition.
fn altitude_scenario(index: u64) -> (f64, Option<f64>, Option<f64>) {
    let step = index % CYCLE_SAMPLES;
    let discontinuity = matches!(step, 0 | 60 | 120 | 240 | 300 | 360 | 2160);
    let samples_per_minute = 60.0 / SAMPLE_PERIOD.as_secs_f64();
    if step < 360 {
        let (start, end) = [
            (-120.0, 120.0),
            (980.0, 1020.0),
            (9980.0, 10020.0),
            (10020.0, 9980.0),
            (1020.0, 980.0),
            (120.0, -120.0),
        ][(step / 60) as usize];
        return (
            start + (end - start) * (step % 60) as f64 / 59.0,
            None,
            (!discontinuity).then_some((end - start) * samples_per_minute / 59.0),
        );
    }
    let (target, direction) = if step < 2160 {
        (10000.0, -1.0)
    } else {
        (1000.0, 1.0)
    };
    let local = (step - 360) % 1800;
    let (begin, end, from, to) = [
        (0, 120, 1500.0, 1500.0),
        (120, 920, 1500.0, 500.0),
        (920, 1320, 500.0, 0.0),
        (1320, 1440, 0.0, 0.0),
        (1440, 1680, 0.0, -300.0),
        (1680, 1800, -300.0, -300.0),
    ]
    .into_iter()
    .find(|&(begin, end, _, _)| (begin..end).contains(&local))
    .expect("built-in segments cover the approach");
    let slope = (to - from) / (end - begin) as f64;
    let distance = from + slope * (local - begin) as f64;
    (
        target + direction * distance,
        Some(target),
        (!discontinuity).then_some(direction * slope * samples_per_minute),
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
                index = (index + 1) % CYCLE_SAMPLES;
            }
        }
    }
}
