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
/// One fresh acquisition with independently available optional indications.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct FlightSample {
    attitude: Attitude,
    #[serde(skip_serializing_if = "Option::is_none")]
    slip_skid: Option<SlipSkid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_rate_dps: Option<TurnRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ias_kt: Option<IndicatedAirspeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gs_kt: Option<GroundSpeed>,
}
impl FlightSample {
    pub fn new(
        attitude: Attitude,
        slip_skid: Option<SlipSkid>,
        turn_rate: Option<TurnRate>,
    ) -> Self {
        Self {
            attitude,
            slip_skid,
            turn_rate_dps: turn_rate,
            ias_kt: None,
            gs_kt: None,
        }
    }
    pub fn attitude(self) -> Attitude {
        self.attitude
    }
    pub fn with_speeds(mut self, ias: Option<IndicatedAirspeed>, gs: Option<GroundSpeed>) -> Self {
        self.ias_kt = ias;
        self.gs_kt = gs;
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
    slip_skid: Option<SlipSkid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_rate_dps: Option<TurnRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ias_kt: Option<IndicatedAirspeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gs_kt: Option<GroundSpeed>,
}
#[derive(Clone, Debug)]
struct Current {
    source: Source,
    sequence: u64,
    state: State,
    accepted_at: Option<Instant>,
    slip_skid: Option<SlipSkid>,
    turn_rate: Option<TurnRate>,
    ias: Option<IndicatedAirspeed>,
    gs: Option<GroundSpeed>,
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
            slip_skid: attitude.and(self.slip_skid),
            turn_rate_dps: attitude.and(self.turn_rate),
            ias_kt: attitude.and(self.ias),
            gs_kt: attitude.and(self.gs),
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
        slip_skid: None,
        turn_rate: None,
        ias: None,
        gs: None,
    };
    let (sender, receiver) = watch::channel(current.clone());
    (Publisher { sender, current }, Subscription { receiver })
}
impl Publisher {
    /// Live means a fresh callback from an active, unpaused source, not cached data.
    pub fn publish(&mut self, state: State) -> Result<(), TelemetryError> {
        self.publish_values(state, None, None, None, None)
    }
    /// Publish one fresh acquisition atomically, sharing sequence and age with attitude.
    pub fn publish_sample(&mut self, sample: FlightSample) -> Result<(), TelemetryError> {
        self.publish_values(
            State::Live(sample.attitude),
            sample.slip_skid,
            sample.turn_rate_dps,
            sample.ias_kt,
            sample.gs_kt,
        )
    }
    fn publish_values(
        &mut self,
        state: State,
        slip_skid: Option<SlipSkid>,
        turn_rate: Option<TurnRate>,
        ias: Option<IndicatedAirspeed>,
        gs: Option<GroundSpeed>,
    ) -> Result<(), TelemetryError> {
        if self.current.sequence == MAX_SEQUENCE {
            return Err(TelemetryError::SequenceExhausted);
        }
        self.current.sequence += 1;
        if matches!(state, State::Live(_)) {
            self.current.accepted_at = Some(Instant::now());
        }
        self.current.state = state;
        self.current.slip_skid = slip_skid;
        self.current.turn_rate = turn_rate;
        self.current.ias = ias;
        self.current.gs = gs;
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
    #[error("indicated airspeed must be finite nonnegative knots")]
    InvalidIndicatedAirspeed,
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
