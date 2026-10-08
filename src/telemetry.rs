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
}
#[derive(Clone, Debug)]
struct Current {
    source: Source,
    sequence: u64,
    state: State,
    accepted_at: Option<Instant>,
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
    };
    let (sender, receiver) = watch::channel(current.clone());
    (Publisher { sender, current }, Subscription { receiver })
}
impl Publisher {
    /// Live means a fresh callback from an active, unpaused source, not cached data.
    pub fn publish(&mut self, state: State) -> Result<(), TelemetryError> {
        if self.current.sequence == MAX_SEQUENCE {
            return Err(TelemetryError::SequenceExhausted);
        }
        self.current.sequence += 1;
        if matches!(state, State::Live(_)) {
            self.current.accepted_at = Some(Instant::now());
        }
        self.current.state = state;
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
    #[error(
        "attitude must contain finite normalized degrees within pitch [-90, 90] and roll [-180, 180)"
    )]
    InvalidAttitude,
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
