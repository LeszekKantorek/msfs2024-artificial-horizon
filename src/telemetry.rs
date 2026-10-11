//! Validated attitude, monotonic freshness and bounded latest-value distribution.
use crate::providers::Source;
use serde::Serialize;
use tokio::{
    sync::watch,
    time::{Duration, Instant},
};
pub const SAMPLE_PERIOD: Duration = Duration::from_millis(50);
pub const STALE_AFTER: Duration = Duration::from_millis(1_000);
const MAX_SEQUENCE: u64 = (1 << 53) - 1;

/// Magnetic heading in degrees clockwise from north, within [0, 360).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Heading(f64);
impl Heading {
    pub fn new(degrees: f64) -> Result<Self, TelemetryError> {
        if !degrees.is_finite() || !(0.0..360.0).contains(&degrees) {
            return Err(TelemetryError::InvalidHeading);
        }
        Ok(Self(degrees))
    }
    pub fn degrees(self) -> f64 {
        self.0
    }
}
/// Source-selected magnetic heading with the same range as current heading.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SelectedHeading(Heading);
impl SelectedHeading {
    pub fn new(degrees: f64) -> Result<Self, TelemetryError> {
        Heading::new(degrees)
            .map(Self)
            .map_err(|_| TelemetryError::InvalidSelectedHeading)
    }
    pub fn degrees(self) -> f64 {
        self.0.degrees()
    }
}

/// Normalized degrees: positive nose up and positive right wing down.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Attitude {
    pitch_deg: f64,
    roll_deg: f64,
}
impl Attitude {
    pub fn new(pitch_deg: f64, roll_deg: f64) -> Result<Self, TelemetryError> {
        if !pitch_deg.is_finite()
            || !roll_deg.is_finite()
            || !(-90.0..=90.0).contains(&pitch_deg)
            || !(-180.0..180.0).contains(&roll_deg)
        {
            return Err(TelemetryError::InvalidAttitude);
        }
        Ok(Self {
            pitch_deg,
            roll_deg,
        })
    }
    pub fn pitch_deg(self) -> f64 {
        self.pitch_deg
    }
    pub fn roll_deg(self) -> f64 {
        self.roll_deg
    }
}
/// Normalized ball displacement: negative left, positive right, zero centered.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SlipSkid(f64);
impl SlipSkid {
    pub fn new(value: f64) -> Result<Self, TelemetryError> {
        if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
            return Err(TelemetryError::InvalidSlipSkid);
        }
        Ok(Self(value))
    }
    pub fn normalized(self) -> f64 {
        self.0
    }
}
/// Degrees per second: negative left and positive right.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TurnRate(f64);
impl TurnRate {
    pub fn new(degrees_per_second: f64) -> Result<Self, TelemetryError> {
        if !degrees_per_second.is_finite() {
            return Err(TelemetryError::InvalidTurnRate);
        }
        Ok(Self(degrees_per_second))
    }
    pub fn degrees_per_second(self) -> f64 {
        self.0
    }
}
/// Validated indicated airspeed in knots. Display limits do not restrict telemetry.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct IndicatedAirspeed(f64);
impl IndicatedAirspeed {
    pub fn new(knots: f64) -> Result<Self, TelemetryError> {
        if !knots.is_finite() || knots < 0.0 {
            return Err(TelemetryError::InvalidIndicatedAirspeed);
        }
        Ok(Self(knots))
    }
    pub fn knots(self) -> f64 {
        self.0
    }
}
/// Validated ground speed in knots, independently available from indicated airspeed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct GroundSpeed(f64);
impl GroundSpeed {
    pub fn new(knots: f64) -> Result<Self, TelemetryError> {
        if !knots.is_finite() || knots < 0.0 {
            return Err(TelemetryError::InvalidGroundSpeed);
        }
        Ok(Self(knots))
    }
    pub fn knots(self) -> f64 {
        self.0
    }
}
/// Validated true airspeed in knots, independently available from IAS and GS.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TrueAirspeed(f64);
impl TrueAirspeed {
    pub fn new(knots: f64) -> Result<Self, TelemetryError> {
        if !knots.is_finite() || knots < 0.0 {
            return Err(TelemetryError::InvalidTrueAirspeed);
        }
        Ok(Self(knots))
    }
    pub fn knots(self) -> f64 {
        self.0
    }
}
/// Barometric altitude in feet for the supplied pressure setting, including negatives.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct BarometricAltitude(f64);
impl BarometricAltitude {
    pub fn new(feet: f64) -> Result<Self, TelemetryError> {
        if !feet.is_finite() {
            return Err(TelemetryError::InvalidBarometricAltitude);
        }
        Ok(Self(feet))
    }
    pub fn feet(self) -> f64 {
        self.0
    }
}
/// Source-supplied vertical speed in feet per minute, without a display limit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct VerticalSpeed(f64);
impl VerticalSpeed {
    pub fn new(feet_per_minute: f64) -> Result<Self, TelemetryError> {
        if !feet_per_minute.is_finite() {
            return Err(TelemetryError::InvalidVerticalSpeed);
        }
        Ok(Self(feet_per_minute))
    }
    pub fn feet_per_minute(self) -> f64 {
        self.0
    }
}
/// Source-selected altitude in feet, including negatives, without a display limit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SelectedAltitude(f64);
impl SelectedAltitude {
    pub fn new(feet: f64) -> Result<Self, TelemetryError> {
        if !feet.is_finite() {
            return Err(TelemetryError::InvalidSelectedAltitude);
        }
        Ok(Self(feet))
    }
    pub fn feet(self) -> f64 {
        self.0
    }
}
/// Source-supplied IAS acceleration in knots per second, including deceleration.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct IasAcceleration(f64);
impl IasAcceleration {
    pub fn new(knots_per_second: f64) -> Result<Self, TelemetryError> {
        if !knots_per_second.is_finite() {
            return Err(TelemetryError::InvalidIasAcceleration);
        }
        Ok(Self(knots_per_second))
    }
    pub fn knots_per_second(self) -> f64 {
        self.0
    }
}

/// Four configured references in knots. Names remain explicit on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct VSpeedReferences {
    vr_kt: f64,
    vx_kt: f64,
    vy_kt: f64,
    vg_kt: f64,
}
impl VSpeedReferences {
    pub fn new(vr: f64, vx: f64, vy: f64, vg: f64) -> Result<Self, TelemetryError> {
        if [vr, vx, vy, vg].iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return Err(TelemetryError::InvalidAirspeedProfile);
        }
        Ok(Self {
            vr_kt: vr,
            vx_kt: vx,
            vy_kt: vy,
            vg_kt: vg,
        })
    }
}

/// Validated speed configuration. This type does not establish aircraft performance.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AirspeedProfile {
    vs0_kt: f64,
    vs1_kt: f64,
    vfe_kt: f64,
    vno_kt: f64,
    vne_kt: f64,
    references: VSpeedReferences,
}
impl AirspeedProfile {
    pub fn new(
        vs0: f64,
        vs1: f64,
        vfe: f64,
        vno: f64,
        vne: f64,
        references: VSpeedReferences,
    ) -> Result<Self, TelemetryError> {
        let thresholds = [vs0, vs1, vfe, vno, vne];
        if thresholds.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || thresholds.windows(2).any(|pair| pair[0] >= pair[1])
            || [
                references.vr_kt,
                references.vx_kt,
                references.vy_kt,
                references.vg_kt,
            ]
            .iter()
            .any(|v| *v > vne)
        {
            return Err(TelemetryError::InvalidAirspeedProfile);
        }
        Ok(Self {
            vs0_kt: vs0,
            vs1_kt: vs1,
            vfe_kt: vfe,
            vno_kt: vno,
            vne_kt: vne,
            references,
        })
    }
}

/// One fresh acquisition with independently available optional indications.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FlightSample {
    attitude: Attitude,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading_deg: Option<Heading>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_heading_deg: Option<SelectedHeading>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slip_skid: Option<SlipSkid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_rate_dps: Option<TurnRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ias_kt: Option<IndicatedAirspeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gs_kt: Option<GroundSpeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tas_kt: Option<TrueAirspeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    airspeed_profile: Option<AirspeedProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ias_acceleration_ktps: Option<IasAcceleration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    altitude_ft: Option<BarometricAltitude>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_altitude_ft: Option<SelectedAltitude>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vertical_speed_fpm: Option<VerticalSpeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baro_text: Option<String>,
}
impl FlightSample {
    pub fn new(
        attitude: Attitude,
        slip_skid: Option<SlipSkid>,
        turn_rate: Option<TurnRate>,
    ) -> Self {
        Self {
            attitude,
            heading_deg: None,
            selected_heading_deg: None,
            slip_skid,
            turn_rate_dps: turn_rate,
            ias_kt: None,
            gs_kt: None,
            tas_kt: None,
            airspeed_profile: None,
            ias_acceleration_ktps: None,
            altitude_ft: None,
            selected_altitude_ft: None,
            vertical_speed_fpm: None,
            baro_text: None,
        }
    }
    pub fn attitude(&self) -> Attitude {
        self.attitude
    }
    pub fn with_heading(
        mut self,
        heading: Option<Heading>,
        selected: Option<SelectedHeading>,
    ) -> Self {
        self.heading_deg = heading;
        self.selected_heading_deg = selected;
        self
    }
    pub fn with_altitude(
        mut self,
        altitude: Option<BarometricAltitude>,
        baro_text: Option<String>,
    ) -> Self {
        self.altitude_ft = altitude;
        self.baro_text = baro_text;
        self
    }
    pub fn with_selected_altitude(mut self, altitude: Option<SelectedAltitude>) -> Self {
        self.selected_altitude_ft = altitude;
        self
    }
    pub fn with_vertical_speed(mut self, speed: Option<VerticalSpeed>) -> Self {
        self.vertical_speed_fpm = speed;
        self
    }
    pub fn with_speeds(mut self, ias: Option<IndicatedAirspeed>, gs: Option<GroundSpeed>) -> Self {
        self.ias_kt = ias;
        self.gs_kt = gs;
        self
    }
    pub fn with_true_airspeed(mut self, speed: Option<TrueAirspeed>) -> Self {
        self.tas_kt = speed;
        self
    }
    pub fn with_airspeed_indications(
        mut self,
        profile: Option<AirspeedProfile>,
        acceleration: Option<IasAcceleration>,
    ) -> Self {
        self.airspeed_profile = profile;
        self.ias_acceleration_ktps = acceleration;
        self
    }
}
/// Unavailable states cannot carry attitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Waiting,
    Live(Attitude),
    Paused,
    Stale,
    Disconnected,
    Invalid,
}
/// Immutable wire snapshot. Obtain a new one immediately before serialization.
#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    schema_version: u8,
    sequence: u64,
    source: Source,
    state: &'static str,
    sample_age_ms: Option<u64>,
    attitude: Option<Attitude>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading_deg: Option<Heading>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_heading_deg: Option<SelectedHeading>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slip_skid: Option<SlipSkid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_rate_dps: Option<TurnRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ias_kt: Option<IndicatedAirspeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gs_kt: Option<GroundSpeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tas_kt: Option<TrueAirspeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    airspeed_profile: Option<AirspeedProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ias_acceleration_ktps: Option<IasAcceleration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    altitude_ft: Option<BarometricAltitude>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_altitude_ft: Option<SelectedAltitude>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vertical_speed_fpm: Option<VerticalSpeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baro_text: Option<String>,
}
#[derive(Clone, Debug)]
struct Current {
    source: Source,
    sequence: u64,
    state: State,
    accepted_at: Option<Instant>,
    heading: Option<Heading>,
    selected_heading: Option<SelectedHeading>,
    slip_skid: Option<SlipSkid>,
    turn_rate: Option<TurnRate>,
    ias: Option<IndicatedAirspeed>,
    gs: Option<GroundSpeed>,
    tas: Option<TrueAirspeed>,
    airspeed_profile: Option<AirspeedProfile>,
    ias_acceleration: Option<IasAcceleration>,
    altitude: Option<BarometricAltitude>,
    selected_altitude: Option<SelectedAltitude>,
    vertical_speed: Option<VerticalSpeed>,
    baro: Option<String>,
}
impl Current {
    fn snapshot(&self) -> Snapshot {
        let age = self
            .accepted_at
            .map(|at| Instant::now().saturating_duration_since(at));
        let state =
            if matches!(self.state, State::Live(_)) && age.is_some_and(|age| age >= STALE_AFTER) {
                State::Stale
            } else {
                self.state
            };
        let (state, attitude) = match state {
            State::Waiting => ("waiting", None),
            State::Live(attitude) => ("live", Some(attitude)),
            State::Paused => ("paused", None),
            State::Stale => ("stale", None),
            State::Disconnected => ("disconnected", None),
            State::Invalid => ("invalid", None),
        };
        Snapshot {
            schema_version: 1,
            sequence: self.sequence,
            source: self.source,
            state,
            sample_age_ms: age.map(|age| u64::try_from(age.as_millis()).unwrap_or(u64::MAX)),
            attitude,
            heading_deg: attitude.and(self.heading),
            selected_heading_deg: attitude.and(self.selected_heading),
            slip_skid: attitude.and(self.slip_skid),
            turn_rate_dps: attitude.and(self.turn_rate),
            ias_kt: attitude.and(self.ias),
            gs_kt: attitude.and(self.gs),
            tas_kt: attitude.and(self.tas),
            airspeed_profile: attitude.and(self.airspeed_profile),
            ias_acceleration_ktps: attitude.and(self.ias_acceleration),
            altitude_ft: attitude.and(self.altitude),
            selected_altitude_ft: attitude.and(self.selected_altitude),
            vertical_speed_fpm: attitude.and(self.vertical_speed),
            baro_text: attitude.and(self.baro.clone()),
        }
    }
}
/// Single writer owns publication numbering and accepted sample time.
pub struct Publisher {
    sender: watch::Sender<Current>,
    current: Current,
}
#[derive(Clone)]
pub struct Subscription {
    receiver: watch::Receiver<Current>,
}
pub fn channel(source: Source) -> (Publisher, Subscription) {
    let current = Current {
        source,
        sequence: 0,
        state: State::Waiting,
        accepted_at: None,
        heading: None,
        selected_heading: None,
        slip_skid: None,
        turn_rate: None,
        ias: None,
        gs: None,
        tas: None,
        airspeed_profile: None,
        ias_acceleration: None,
        altitude: None,
        selected_altitude: None,
        vertical_speed: None,
        baro: None,
    };
    let (sender, receiver) = watch::channel(current.clone());
    (Publisher { sender, current }, Subscription { receiver })
}
impl Publisher {
    /// Live means a fresh callback from an active, unpaused source, not cached data.
    pub fn publish(&mut self, state: State) -> Result<(), TelemetryError> {
        self.publish_values(state, None)
    }
    /// Publish one fresh acquisition atomically, sharing sequence and age with attitude.
    pub fn publish_sample(&mut self, sample: FlightSample) -> Result<(), TelemetryError> {
        self.publish_values(State::Live(sample.attitude), Some(sample))
    }
    fn publish_values(
        &mut self,
        state: State,
        sample: Option<FlightSample>,
    ) -> Result<(), TelemetryError> {
        if self.current.sequence == MAX_SEQUENCE {
            return Err(TelemetryError::SequenceExhausted);
        }
        self.current.sequence += 1;
        if matches!(state, State::Live(_)) {
            self.current.accepted_at = Some(Instant::now());
        }
        self.current.state = state;
        self.current.heading = sample.as_ref().and_then(|s| s.heading_deg);
        self.current.selected_heading = sample.as_ref().and_then(|s| s.selected_heading_deg);
        self.current.slip_skid = sample.as_ref().and_then(|s| s.slip_skid);
        self.current.turn_rate = sample.as_ref().and_then(|s| s.turn_rate_dps);
        self.current.ias = sample.as_ref().and_then(|s| s.ias_kt);
        self.current.gs = sample.as_ref().and_then(|s| s.gs_kt);
        self.current.tas = sample.as_ref().and_then(|s| s.tas_kt);
        self.current.airspeed_profile = sample.as_ref().and_then(|s| s.airspeed_profile);
        self.current.ias_acceleration = sample.as_ref().and_then(|s| s.ias_acceleration_ktps);
        self.current.altitude = sample.as_ref().and_then(|s| s.altitude_ft);
        self.current.selected_altitude = sample.as_ref().and_then(|s| s.selected_altitude_ft);
        self.current.vertical_speed = sample.as_ref().and_then(|s| s.vertical_speed_fpm);
        self.current.baro = sample.and_then(|s| s.baro_text);
        self.sender.send_replace(self.current.clone());
        Ok(())
    }
    /// Call on an independent freshness tick, including when source callbacks stop.
    pub fn expire(&mut self) -> Result<(), TelemetryError> {
        if matches!(self.current.state, State::Live(_))
            && self
                .current
                .accepted_at
                .is_some_and(|at| at.elapsed() >= STALE_AFTER)
        {
            self.publish(State::Stale)?;
        }
        Ok(())
    }
}
impl Subscription {
    /// Obtain current state and acknowledge it atomically, including on subscription.
    pub fn snapshot_and_update(&mut self) -> Snapshot {
        self.receiver.borrow_and_update().snapshot()
    }

    /// Does not acknowledge a pending change; suitable for immediate delivery.
    pub fn snapshot(&self) -> Snapshot {
        self.receiver.borrow().snapshot()
    }
    /// Acknowledges the newest publication, skipping intermediate samples.
    pub async fn changed(&mut self) -> Result<Snapshot, watch::error::RecvError> {
        self.receiver.changed().await?;
        Ok(self.receiver.borrow_and_update().snapshot())
    }
}
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("heading must be finite magnetic degrees within [0, 360)")]
    InvalidHeading,
    #[error("selected heading must be finite magnetic degrees within [0, 360)")]
    InvalidSelectedHeading,
    #[error("vertical speed must be finite feet per minute")]
    InvalidVerticalSpeed,
    #[error("selected altitude must be finite feet")]
    InvalidSelectedAltitude,
    #[error("barometric altitude must be finite feet")]
    InvalidBarometricAltitude,
    #[error("IAS acceleration must be finite knots per second")]
    InvalidIasAcceleration,
    #[error(
        "airspeed profile requires increasing positive finite thresholds and positive references at or below VNE, in knots"
    )]
    InvalidAirspeedProfile,
    #[error("indicated airspeed must be finite nonnegative knots")]
    InvalidIndicatedAirspeed,
    #[error("true airspeed must be finite nonnegative knots")]
    InvalidTrueAirspeed,
    #[error("ground speed must be finite nonnegative knots")]
    InvalidGroundSpeed,
    #[error(
        "attitude must contain finite normalized degrees within pitch [-90, 90] and roll [-180, 180)"
    )]
    InvalidAttitude,
    #[error("slip/skid must be finite normalized displacement within [-1, 1]")]
    InvalidSlipSkid,
    #[error("turn rate must be finite degrees per second")]
    InvalidTurnRate,
    #[error("telemetry sequence exhausted the JSON safe integer range")]
    SequenceExhausted,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn closing_sse_body_releases_its_subscription() {
        use axum::{body::Body, http::Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let (publisher, subscription) = channel(Source::Demo);
        let response = crate::http::router_with_telemetry(subscription.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut body = response.into_body();
        body.frame().await.unwrap().unwrap();
        assert_eq!(publisher.sender.receiver_count(), 2);
        drop(body);
        assert_eq!(publisher.sender.receiver_count(), 1);
    }
    #[test]
    fn sequence_exhaustion_does_not_publish_or_wrap() {
        let (mut publisher, subscription) = channel(Source::Demo);
        publisher.current.sequence = MAX_SEQUENCE - 1;
        publisher.publish(State::Paused).unwrap();
        assert!(matches!(
            publisher.publish(State::Waiting),
            Err(TelemetryError::SequenceExhausted)
        ));
        assert_eq!(subscription.snapshot().sequence, MAX_SEQUENCE);
        assert_eq!(subscription.snapshot().state, "paused");
    }
}
