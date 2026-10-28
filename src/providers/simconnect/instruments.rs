//! Last-value instrument observations owned exclusively by the native worker.
use crate::telemetry::*;
use std::time::Instant;

#[derive(Default)]
pub(super) struct Instruments {
    values: [Option<(f64, Instant)>; 17],
    previous_ias: Option<(f64, Instant)>,
    acceleration: Option<f64>,
    pub dirty: bool,
}
impl Instruments {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn remove(&mut self, index: usize) {
        self.values[index] = None;
        self.dirty = true;
        if index == 0 {
            self.previous_ias = None;
            self.acceleration = None;
        }
    }
    pub fn receive(&mut self, index: usize, value: f64, now: Instant) {
        self.dirty = true;
        if index == 0 {
            self.acceleration = self.previous_ias.and_then(|(previous, at)| {
                let elapsed = now.saturating_duration_since(at);
                (value.is_finite() && value >= 0.0 && !elapsed.is_zero() && elapsed < STALE_AFTER)
                    .then(|| (value - previous) / elapsed.as_secs_f64())
                    .filter(|v| v.is_finite())
            });
            self.previous_ias = (value.is_finite() && value >= 0.0).then_some((value, now));
        }
        self.values[index] = value.is_finite().then_some((value, now));
    }
    pub fn expire(&mut self, now: Instant) {
        for index in 0..self.values.len() {
            if self.values[index]
                .is_some_and(|(_, at)| now.saturating_duration_since(at) >= STALE_AFTER)
            {
                self.remove(index);
            }
        }
    }
    pub fn sample(&self, attitude: Attitude) -> FlightSample {
        let get = |i: usize| self.values[i].map(|(v, _)| v);
        let profile = AirspeedProfile::partial(
            [get(9), get(10), None, None, None],
            VSpeedReferences::partial(get(11), None, None, None),
            get(12),
        );
        let ap = get(13) == Some(1.0);
        FlightSample::new(
            attitude,
            get(7).and_then(|v| SlipSkid::new(v / 127.0).ok()),
            get(8).and_then(|v| TurnRate::new(v).ok()),
        )
        .with_speeds(
            get(0).and_then(|v| IndicatedAirspeed::new(v).ok()),
            get(2).and_then(|v| GroundSpeed::new(v).ok()),
        )
        .with_true_airspeed(get(1).and_then(|v| TrueAirspeed::new(v).ok()))
        .with_altitude(
            get(3).and_then(|v| BarometricAltitude::new(v).ok()),
            get(4).filter(|v| *v > 0.0).map(|v| format!("{v:.2} inHg")),
        )
        .with_vertical_speed(get(5).and_then(|v| VerticalSpeed::new(v).ok()))
        .with_heading(
            get(6).and_then(|v| Heading::new(v.rem_euclid(360.0)).ok()),
            get(14)
                .filter(|_| ap)
                .and_then(|v| SelectedHeading::new(v.rem_euclid(360.0)).ok()),
        )
        .with_selected_altitude(
            get(15)
                .filter(|_| ap && get(16) == Some(0.0))
                .and_then(|v| SelectedAltitude::new(v).ok()),
        )
        .with_airspeed_indications(
            profile,
            self.acceleration.and_then(|v| IasAcceleration::new(v).ok()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    fn snapshot(cache: &Instruments) -> serde_json::Value {
        let (mut publisher, subscription) =
            crate::telemetry::channel(crate::providers::Source::SimConnect);
        publisher
            .publish_sample(cache.sample(Attitude::new(0.0, 0.0).unwrap()))
            .unwrap();
        serde_json::to_value(subscription.snapshot()).unwrap()
    }
    #[test]
    fn optional_values_remain_independent_and_expire() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        cache.receive(0, f64::NAN, now);
        cache.receive(3, -50.0, now);
        cache.receive(9, 40.0, now);
        cache.receive(11, 55.0, now);
        let state = snapshot(&cache);
        assert!(state.get("ias_kt").is_none());
        assert_eq!(state["altitude_ft"], -50.0);
        assert_eq!(state["airspeed_profile"]["vs0_kt"], 40.0);
        assert_eq!(state["airspeed_profile"]["references"]["vr_kt"], 55.0);
        cache.expire(now + STALE_AFTER);
        assert!(snapshot(&cache).get("altitude_ft").is_none());
    }
    #[test]
    fn trend_uses_acquisitions_and_resets_after_gap_or_invalid_data() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        cache.receive(0, 100.0, now);
        assert!(cache.acceleration.is_none());
        cache.receive(0, 101.0, now + Duration::from_millis(500));
        assert_eq!(cache.acceleration, Some(2.0));
        cache.receive(0, 110.0, now + Duration::from_secs(3));
        assert!(cache.acceleration.is_none());
        cache.clear();
        cache.receive(0, 111.0, now + Duration::from_secs(4));
        assert!(cache.acceleration.is_none());
    }
    #[test]
    fn autopilot_targets_use_availability_and_radio_reference_not_master() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        for (i, v) in [(6, 360.0), (13, 1.0), (14, 0.0), (15, 3000.0), (16, 0.0)] {
            cache.receive(i, v, now);
        }
        let state = snapshot(&cache);
        assert_eq!(state["heading_deg"], 0.0);
        assert_eq!(state["selected_heading_deg"], 0.0);
        assert_eq!(state["selected_altitude_ft"], 3000.0);
        cache.receive(16, 1.0, now);
        assert!(snapshot(&cache).get("selected_altitude_ft").is_none());
    }
}
