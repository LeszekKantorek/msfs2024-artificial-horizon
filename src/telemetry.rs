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

/// Independently available configured references in knots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct VSpeedReferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    vr_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vx_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vy_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vg_kt: Option<f64>,
}
fn positive_speed(value: Option<f64>) -> Option<f64> {
    value.filter(|v| v.is_finite() && *v > 0.0)
}
impl VSpeedReferences {
    pub fn new(vr: f64, vx: f64, vy: f64, vg: f64) -> Result<Self, TelemetryError> {
        if [vr, vx, vy, vg]
            .iter()
            .any(|v| positive_speed(Some(*v)).is_none())
        {
            return Err(TelemetryError::InvalidAirspeedProfile);
        }
        Ok(Self::partial(Some(vr), Some(vx), Some(vy), Some(vg)))
    }
    /// Discard an invalid reference without losing the other references.
    pub fn partial(vr: Option<f64>, vx: Option<f64>, vy: Option<f64>, vg: Option<f64>) -> Self {
        Self {
            vr_kt: positive_speed(vr),
            vx_kt: positive_speed(vx),
            vy_kt: positive_speed(vy),
            vg_kt: positive_speed(vg),
        }
    }
    fn is_empty(&self) -> bool {
        [self.vr_kt, self.vx_kt, self.vy_kt, self.vg_kt]
            .iter()
            .all(Option::is_none)
    }
}

/// Independent speed configuration. Range relationships are checked by each consumer.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AirspeedProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    vs0_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vs1_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vfe_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vno_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vne_kt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redline_kt: Option<f64>,
    #[serde(skip_serializing_if = "VSpeedReferences::is_empty")]
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
        if thresholds
            .iter()
            .any(|v| positive_speed(Some(*v)).is_none())
        {
            return Err(TelemetryError::InvalidAirspeedProfile);
        }
        Ok(Self::partial(thresholds.map(Some), references, None).expect("validated thresholds"))
    }
    /// Threshold order is VS0, VS1, VFE, VNO, VNE. Invalid fields are omitted.
    pub fn partial(
        thresholds: [Option<f64>; 5],
        references: VSpeedReferences,
        redline: Option<f64>,
    ) -> Option<Self> {
        let [vs0_kt, vs1_kt, vfe_kt, vno_kt, vne_kt] = thresholds.map(positive_speed);
        let redline_kt = positive_speed(redline);
        if [vs0_kt, vs1_kt, vfe_kt, vno_kt, vne_kt, redline_kt]
            .iter()
            .all(Option::is_none)
            && references.is_empty()
        {
            return None;
        }
        Some(Self {
            vs0_kt,
            vs1_kt,
            vfe_kt,
            vno_kt,
            vne_kt,
            redline_kt,
            references,
        })
    }
}

/// Conversion from inches of mercury to hectopascals.
pub const HPA_PER_INHG: f64 = 33.8638866667;

/// Active barometric pressure.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BaroSetting {
    pressure_hpa: f64,
}
impl BaroSetting {
    pub fn new(pressure_hpa: f64) -> Result<Self, TelemetryError> {
        if !pressure_hpa.is_finite() || pressure_hpa <= 0.0 {
            return Err(TelemetryError::InvalidBaroSetting);
        }
        Ok(Self { pressure_hpa })
    }
    pub fn pressure_hpa(&self) -> f64 {
        self.pressure_hpa
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
    #[serde(skip_serializing_if = "Option::is_none")]
    baro_setting: Option<BaroSetting>,
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
            baro_setting: None,
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
    pub fn with_baro_setting(mut self, setting: Option<BaroSetting>) -> Self {
        self.baro_setting = setting;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    baro_setting: Option<BaroSetting>,
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
    baro_setting: Option<BaroSetting>,
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
            baro_setting: attitude.and(self.baro_setting),
        }
    }
}
/// Single writer owns publication numbering and accepted sample time.
pub struct Publisher {
    sender: watch::Sender<Current>,
    current: Current,
}
/// Keeps the existing channel open without granting a second publication owner.
pub(crate) struct ChannelLifetime {
    sender: watch::Sender<Current>,
}
impl ChannelLifetime {
    pub(crate) async fn recover(self) -> Publisher {
        // A cancelled SimConnect future signals its native worker, which may still
        // own the publisher until native cleanup returns. Never race that owner.
        while self.sender.sender_count() != 1 {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let current = self.sender.borrow().clone();
        Publisher {
            sender: self.sender,
            current,
        }
    }
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
        baro_setting: None,
    };
    let (sender, receiver) = watch::channel(current.clone());
    (Publisher { sender, current }, Subscription { receiver })
}
impl Publisher {
    pub(crate) fn retain_channel(&self) -> ChannelLifetime {
        ChannelLifetime {
            sender: self.sender.clone(),
        }
    }

    /// Source identity is fixed when the caller creates the channel.
    pub fn source(&self) -> Source {
        self.current.source
    }

    /// Inspect the same latest state sent to subscribers, without another channel.
    pub fn snapshot(&self) -> Snapshot {
        self.current.snapshot()
    }

    /// Live means a fresh callback from an active, unpaused source, not cached data.
    pub fn publish(&mut self, state: State) -> Result<(), TelemetryError> {
        self.publish_values(state, None, Instant::now())
    }
    /// Publish one fresh acquisition atomically, sharing sequence and age with attitude.
    pub fn publish_sample(&mut self, sample: FlightSample) -> Result<(), TelemetryError> {
        self.publish_sample_at(sample, Instant::now())
    }
    /// Preserve acquisition time when optional indications change or expire.
    pub(crate) fn publish_sample_at(
        &mut self,
        sample: FlightSample,
        received_at: Instant,
    ) -> Result<(), TelemetryError> {
        self.publish_values(State::Live(sample.attitude), Some(sample), received_at)
    }
    fn publish_values(
        &mut self,
        state: State,
        sample: Option<FlightSample>,
        received_at: Instant,
    ) -> Result<(), TelemetryError> {
        if self.current.sequence == MAX_SEQUENCE {
            return Err(TelemetryError::SequenceExhausted);
        }
        self.current.sequence += 1;
        if matches!(state, State::Live(_)) {
            self.current.accepted_at = Some(received_at);
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
        self.current.baro_setting = sample.as_ref().and_then(|s| s.baro_setting);
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
    #[error("barometric pressure must be finite and positive hPa")]
    InvalidBaroSetting,
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
    #[error("airspeed profile values must be positive finite speeds in knots")]
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
    #[tokio::test(start_paused = true)]
    async fn retained_channel_recovers_latest_sequence_age_and_clears_extensions() {
        let (mut publisher, mut subscription) = channel(Source::SimConnect);
        let lifetime = publisher.retain_channel();
        publisher
            .publish_sample(
                FlightSample::new(Attitude::new(12.0, -23.0).unwrap(), None, None)
                    .with_speeds(Some(IndicatedAirspeed::new(140.0).unwrap()), None),
            )
            .unwrap();
        subscription.snapshot_and_update();
        tokio::time::advance(Duration::from_millis(300)).await;
        drop(publisher);
        let mut recovered = lifetime.recover().await;
        recovered.publish(State::Disconnected).unwrap();
        let snapshot = subscription.changed().await.unwrap();
        assert_eq!(snapshot.sequence, 2);
        assert_eq!(snapshot.source, Source::SimConnect);
        assert_eq!(snapshot.sample_age_ms, Some(300));
        assert_eq!(snapshot.state, "disconnected");
        assert!(snapshot.attitude.is_none());
        assert!(snapshot.ias_kt.is_none());
        assert!(
            tokio::time::timeout(Duration::from_millis(1), subscription.changed())
                .await
                .is_err()
        );
        drop(recovered);
        assert!(subscription.changed().await.is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn recovery_waits_for_the_only_active_publisher() {
        let (publisher, mut subscription) = channel(Source::SimConnect);
        let lifetime = publisher.retain_channel();
        let mut recovery = Box::pin(lifetime.recover());
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut recovery)
                .await
                .is_err()
        );
        assert_eq!(subscription.snapshot().state, "waiting");
        drop(publisher);
        let mut recovered = recovery.await;
        recovered.publish(State::Disconnected).unwrap();
        assert_eq!(subscription.changed().await.unwrap().sequence, 1);
    }

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
    fn partial_profile_keeps_independent_values_and_omits_invalid_members() {
        let profile = AirspeedProfile::partial(
            [Some(40.0), Some(f64::NAN), None, Some(-1.0), None],
            VSpeedReferences::partial(Some(55.0), None, Some(f64::INFINITY), Some(90.0)),
            Some(140.0),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(profile).unwrap(),
            serde_json::json!({
                "vs0_kt": 40.0, "redline_kt": 140.0,
                "references": {"vr_kt": 55.0, "vg_kt": 90.0}
            })
        );
        assert!(
            AirspeedProfile::partial([None; 5], VSpeedReferences::default(), Some(0.0)).is_none()
        );
        // An incoherent pair is retained for the affected range's consumer;
        // unrelated references must not disappear with it.
        assert!(
            AirspeedProfile::new(
                90.0,
                40.0,
                80.0,
                70.0,
                60.0,
                VSpeedReferences::partial(Some(100.0), None, None, None)
            )
            .is_ok()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn replacing_partial_profile_preserves_original_acquisition_deadline() {
        let (mut publisher, subscription) = channel(Source::SimConnect);
        let received_at = Instant::now();
        let attitude = Attitude::new(0.0, 0.0).unwrap();
        publisher
            .publish_sample_at(
                FlightSample::new(attitude, None, None).with_airspeed_indications(
                    AirspeedProfile::partial(
                        [Some(40.0), None, None, None, None],
                        VSpeedReferences::default(),
                        None,
                    ),
                    None,
                ),
                received_at,
            )
            .unwrap();
        tokio::time::advance(Duration::from_millis(900)).await;
        publisher
            .publish_sample_at(FlightSample::new(attitude, None, None), received_at)
            .unwrap();
        let value = serde_json::to_value(subscription.snapshot()).unwrap();
        assert!(value.get("airspeed_profile").is_none());
        assert_eq!(value["sample_age_ms"], 900);
        tokio::time::advance(Duration::from_millis(100)).await;
        publisher.expire().unwrap();
        assert_eq!(subscription.snapshot().state, "stale");
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
