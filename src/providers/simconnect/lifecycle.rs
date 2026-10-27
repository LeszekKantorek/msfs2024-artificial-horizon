//! Conservative flight availability from lifecycle signals and fresh samples.
use crate::telemetry::{Attitude, STALE_AFTER, State, TelemetryError};
use std::time::Instant;

pub(super) const EVENT_SIM: u32 = 10;
pub(super) const EVENT_PAUSE: u32 = 11;
pub(super) const EVENT_FLIGHT: u32 = 12;
pub(super) const STATE_SIM: u32 = 20;
pub(super) const STATE_FLIGHT: u32 = 21;

#[derive(Default)]
pub(super) struct Lifecycle {
    connected: bool,
    sim_running: Option<bool>,
    pause: Option<u32>,
    flight: Option<String>,
    sample: Option<(Instant, Attitude)>,
    invalid: bool,
}

pub(super) struct Evaluation {
    pub state: State,
    pub reason: &'static str,
}

impl Lifecycle {
    pub(super) fn connect(&mut self) {
        *self = Self {
            connected: true,
            ..Self::default()
        };
    }
    pub(super) fn disconnect(&mut self) {
        *self = Self::default();
    }

    pub(super) fn sim(&mut self, running: bool) {
        if self.sim_running != Some(running) {
            self.sample = None;
            self.invalid = false;
        }
        self.sim_running = Some(running);
    }

    pub(super) fn pause(&mut self, flags: u32) {
        // Any transition requires a fresh sample; unknown bits are treated
        // conservatively as paused until the recorded flags are investigated.
        if self.pause != Some(flags) {
            self.sample = None;
            self.invalid = false;
        }
        self.pause = Some(flags);
    }

    pub(super) fn flight(&mut self, filename: &str, transition: bool) {
        if transition || self.flight.as_deref() != Some(filename) {
            self.sample = None;
            self.invalid = false;
        }
        self.flight = Some(filename.to_owned());
    }

    pub(super) fn sample(&mut self, attitude: Result<Attitude, TelemetryError>, now: Instant) {
        // Observations while paused/inactive are logged by the caller, but never
        // retained for a later resume. A loaded filename alone proves no flight.
        if self.sim_running == Some(true)
            && self.pause == Some(0)
            && self.flight.as_ref().is_some_and(|path| !path.is_empty())
        {
            match attitude {
                Ok(value) => {
                    self.sample = Some((now, value));
                    self.invalid = false;
                }
                Err(_) => {
                    self.sample = None;
                    self.invalid = true;
                }
            }
        }
    }

    pub(super) fn evaluate(&self, now: Instant) -> Evaluation {
        let (state, reason) = if !self.connected {
            (State::Disconnected, "connection_unavailable")
        } else if self.pause.is_some_and(|flags| flags != 0) {
            (State::Paused, "explicit_pause_flags")
        } else if self.sim_running != Some(true)
            || self.pause.is_none()
            || !self.flight.as_ref().is_some_and(|path| !path.is_empty())
        {
            (State::Waiting, "incomplete_or_inactive_lifecycle")
        } else if self.invalid {
            (State::Invalid, "attitude_validation_failed")
        } else if let Some((received, attitude)) = self.sample {
            if now.saturating_duration_since(received) >= STALE_AFTER {
                (State::Stale, "no_new_valid_sample")
            } else {
                (
                    State::Live(attitude),
                    "active_unpaused_flight_and_fresh_valid_sample",
                )
            }
        } else {
            (State::Waiting, "new_valid_sample_required")
        };
        Evaluation { state, reason }
    }
}

/// Native radians are the only angular input; convert locally and preserve the
/// operator-confirmed signs. Separate SDK degree reads are diagnostic observations.
pub(super) fn candidate_attitude(radians: [f64; 2]) -> Result<Attitude, TelemetryError> {
    let [pitch_rad, bank_rad] = radians;
    if !radians.iter().all(|number| number.is_finite()) {
        return Err(TelemetryError::InvalidAttitude);
    }
    let pitch_deg = pitch_rad.to_degrees();
    let bank_deg = bank_rad.to_degrees();
    Attitude::new(-pitch_deg, (-bank_deg + 180.0).rem_euclid(360.0) - 180.0)
}

pub(super) fn state_name(state: State) -> &'static str {
    match state {
        State::Waiting => "waiting",
        State::Live(_) => "live",
        State::Paused => "paused",
        State::Stale => "stale",
        State::Disconnected => "disconnected",
        State::Invalid => "invalid",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    fn active() -> Lifecycle {
        let mut lifecycle = Lifecycle::default();
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("test.flt", true);
        lifecycle
    }
    fn attitude() -> Attitude {
        Attitude::new(10.0, 20.0).unwrap()
    }

    #[test]
    fn new_equal_samples_are_fresh_but_time_without_samples_expires() {
        let mut lifecycle = active();
        let at = Instant::now();
        lifecycle.sample(Ok(attitude()), at);
        lifecycle.sample(Ok(attitude()), at + Duration::from_millis(900));
        assert!(matches!(
            lifecycle.evaluate(at + Duration::from_millis(1800)).state,
            State::Live(_)
        ));
        assert!(matches!(
            lifecycle.evaluate(at + Duration::from_millis(1900)).state,
            State::Stale
        ));
    }

    #[test]
    fn pause_and_menu_never_reuse_cached_attitude_on_resume() {
        let mut lifecycle = active();
        let at = Instant::now();
        lifecycle.sample(Ok(attitude()), at);
        lifecycle.pause(4);
        lifecycle.sample(Ok(attitude()), at);
        assert!(matches!(lifecycle.evaluate(at).state, State::Paused));
        lifecycle.pause(0);
        assert!(matches!(lifecycle.evaluate(at).state, State::Waiting));
        lifecycle.sample(Ok(attitude()), at);
        lifecycle.sim(false);
        lifecycle.sim(true);
        assert!(matches!(lifecycle.evaluate(at).state, State::Waiting));
        lifecycle.sample(Ok(attitude()), at);
        lifecycle.flight("test.flt", true);
        assert!(matches!(lifecycle.evaluate(at).state, State::Waiting));
    }

    #[test]
    fn live_requires_complete_lifecycle_and_a_new_sample_after_reconnect() {
        let at = Instant::now();
        let mut lifecycle = Lifecycle::default();
        lifecycle.connect();
        lifecycle.sample(Ok(attitude()), at);
        assert!(matches!(lifecycle.evaluate(at).state, State::Waiting));
        lifecycle = active();
        lifecycle.sample(Ok(attitude()), at);
        let evaluated = lifecycle.evaluate(at);
        assert!(matches!(evaluated.state, State::Live(_)));
        lifecycle.disconnect();
        assert!(matches!(lifecycle.evaluate(at).state, State::Disconnected));
        lifecycle.connect();
        assert!(matches!(lifecycle.evaluate(at).state, State::Waiting));
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("new.flt", true);
        assert!(matches!(lifecycle.evaluate(at).state, State::Waiting));
        lifecycle.sample(Ok(attitude()), at);
        assert!(matches!(lifecycle.evaluate(at).state, State::Live(_)));
    }

    #[test]
    fn invalid_requires_new_valid_sample_and_explicit_pause_has_priority() {
        let mut lifecycle = active();
        let at = Instant::now();
        lifecycle.sample(Err(TelemetryError::InvalidAttitude), at);
        assert!(matches!(lifecycle.evaluate(at).state, State::Invalid));
        lifecycle.pause(1);
        assert!(matches!(lifecycle.evaluate(at).state, State::Paused));
        lifecycle.pause(0);
        lifecycle.sample(Ok(attitude()), at);
        assert!(matches!(lifecycle.evaluate(at).state, State::Live(_)));
    }

    #[test]
    fn canonical_radians_preserve_signs_ranges_and_finite_validation() {
        let value = candidate_attitude([-15_f64.to_radians(), -30_f64.to_radians()]).unwrap();
        assert!((value.pitch_deg() - 15.).abs() < 1e-12);
        assert!((value.roll_deg() - 30.).abs() < 1e-12);
        assert_eq!(
            candidate_attitude([0.0, -std::f64::consts::PI])
                .unwrap()
                .roll_deg(),
            -180.0
        );
        for radians in [
            [f64::NAN, 0.],
            [0., f64::INFINITY],
            [f64::NEG_INFINITY, 0.],
            [91_f64.to_radians(), 0.],
            [-91_f64.to_radians(), 0.],
            [0., f64::MAX],
        ] {
            assert!(candidate_attitude(radians).is_err());
        }
    }

    #[test]
    fn live_unit_check_failures_have_valid_canonical_radians() {
        for (radians, expected_pitch, expected_roll) in [
            (
                [-0.029813224462023905, 0.005893836994290791],
                1.7081719353501539,
                -0.337691984910933,
            ),
            (
                [0.04284653517251412, -0.14671478319603343],
                -2.4549256321438957,
                8.406137869309607,
            ),
        ] {
            let value = candidate_attitude(radians).unwrap();
            assert!((value.pitch_deg() - expected_pitch).abs() < 1e-12);
            assert!((value.roll_deg() - expected_roll).abs() < 1e-12);
            let mut lifecycle = active();
            let now = Instant::now();
            lifecycle.sample(Ok(value), now);
            assert!(matches!(lifecycle.evaluate(now).state, State::Live(_)));
        }
    }
}
