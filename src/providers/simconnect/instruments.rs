//! Last-value instrument observations owned exclusively by the native worker.
use super::mapping::Field;
use crate::telemetry::*;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const IAS_HISTORY_LIMIT: usize = 5;

#[derive(Default)]
pub(super) struct Instruments {
    values: [Option<f64>; 19],
    ias_history: VecDeque<(f64, Instant)>,
    acceleration: Option<f64>,
}
impl Instruments {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn receive(&mut self, field: Field, value: f64, now: Instant) {
        let index = field as usize;
        if field == Field::IndicatedAirspeed {
            self.acceleration = None;
            if !value.is_finite() || value < 0.0 {
                self.ias_history.clear();
            } else {
                if let Some(&(_, last)) = self.ias_history.back() {
                    if now < last {
                        self.ias_history.clear();
                    } else if now == last {
                        self.ias_history.pop_back();
                    }
                }
                self.ias_history.push_back((value, now));
                if self.ias_history.len() > IAS_HISTORY_LIMIT {
                    self.ias_history.pop_front();
                }
                if self.ias_history.len() == IAS_HISTORY_LIMIT {
                    let origin = self.ias_history[0].1;
                    let points = self
                        .ias_history
                        .iter()
                        .map(|(v, t)| (t.duration_since(origin).as_secs_f64(), *v));
                    let mean_t = points.clone().map(|(t, _)| t).sum::<f64>() / 5.0;
                    let mean_v = points.clone().map(|(_, v)| v).sum::<f64>() / 5.0;
                    let numerator = points
                        .clone()
                        .map(|(t, v)| (t - mean_t) * (v - mean_v))
                        .sum::<f64>();
                    let denominator = points
                        .clone()
                        .map(|(t, _)| (t - mean_t).powi(2))
                        .sum::<f64>();
                    self.acceleration = Some(numerator / denominator).filter(|v| v.is_finite());
                }
            }
        }
        self.values[index] = value.is_finite().then_some(value);
    }
    pub fn clear_if_stale(&mut self, now: Instant, stale_after: Duration) {
        if self
            .ias_history
            .back()
            .is_some_and(|(_, at)| now.saturating_duration_since(*at) >= stale_after)
        {
            self.ias_history.clear();
            self.acceleration = None;
        }
    }
    pub fn sample(&self, attitude: Attitude) -> FlightSample {
        let get = |field: Field| self.values[field as usize];
        let profile = AirspeedProfile::partial(
            [get(Field::Vs0), get(Field::Vs1), None, None, None],
            VSpeedReferences::partial(get(Field::RotationSpeed), None, None, None),
            get(Field::MaximumAirspeed),
        );
        let ap = get(Field::AutopilotAvailable).is_some_and(|value| value != 0.0);
        let baro = get(Field::BaroSetting)
            .and_then(|pressure| BaroSetting::new(pressure * HPA_PER_INHG).ok());
        FlightSample::new(
            attitude,
            get(Field::SlipSkid).and_then(|v| SlipSkid::new(v / 127.0).ok()),
            get(Field::TurnRate).and_then(|v| TurnRate::new(v).ok()),
        )
        .with_speeds(
            get(Field::IndicatedAirspeed).and_then(|v| IndicatedAirspeed::new(v).ok()),
            get(Field::GroundSpeed).and_then(|v| GroundSpeed::new(v).ok()),
        )
        .with_true_airspeed(get(Field::TrueAirspeed).and_then(|v| TrueAirspeed::new(v).ok()))
        .with_altitude(
            get(Field::BarometricAltitude).and_then(|v| BarometricAltitude::new(v).ok()),
            baro.map(|setting| format!("{:.2} inHg", setting.pressure_hpa() / HPA_PER_INHG)),
        )
        .with_baro_setting(baro)
        .with_vertical_speed(get(Field::VerticalSpeed).and_then(|v| VerticalSpeed::new(v).ok()))
        .with_heading(
            get(Field::Heading).and_then(|v| Heading::new(v.rem_euclid(360.0)).ok()),
            get(Field::SelectedHeading)
                .filter(|_| ap)
                .and_then(|v| SelectedHeading::new(v.rem_euclid(360.0)).ok()),
        )
        .with_selected_altitude(
            get(Field::SelectedAltitude)
                .filter(|_| ap && get(Field::AutopilotAltRadioMode) == Some(0.0))
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
    fn optional_values_are_validated_independently() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        cache.receive(Field::IndicatedAirspeed, f64::NAN, now);
        cache.receive(Field::BarometricAltitude, -50.0, now);
        cache.receive(Field::Vs0, 40.0, now);
        cache.receive(Field::RotationSpeed, 55.0, now);
        let state = snapshot(&cache);
        assert!(state.get("ias_kt").is_none());
        assert_eq!(state["altitude_ft"], -50.0);
        assert_eq!(state["airspeed_profile"]["vs0_kt"], 40.0);
        assert_eq!(state["airspeed_profile"]["references"]["vr_kt"], 55.0);
        cache.clear_if_stale(now + STALE_AFTER, STALE_AFTER);
        assert_eq!(snapshot(&cache)["altitude_ft"], -50.0);
    }
    #[test]
    fn trend_regresses_last_five_distinct_samples_and_resets() {
        let start = Instant::now();
        let mut cache = Instruments::default();
        for i in 0..6 {
            cache.receive(
                Field::IndicatedAirspeed,
                100.0 + i as f64 * 2.0,
                start + Duration::from_secs(i),
            );
            if i < 4 {
                assert!(cache.acceleration.is_none());
            }
        }
        assert_eq!(cache.ias_history.len(), 5);
        assert!((cache.acceleration.unwrap() - 2.0).abs() < 1e-9);
        cache.receive(
            Field::IndicatedAirspeed,
            110.0,
            start + Duration::from_secs(5),
        );
        assert_eq!(cache.ias_history.len(), 5);
        cache.receive(Field::IndicatedAirspeed, 100.0, start);
        assert_eq!(cache.ias_history.len(), 1);
        cache.receive(Field::IndicatedAirspeed, f64::NAN, start);
        assert!(cache.ias_history.is_empty());
        cache.receive(Field::IndicatedAirspeed, 100.0, start);
        cache.clear_if_stale(
            start + Duration::from_millis(750),
            Duration::from_millis(750),
        );
        assert!(cache.ias_history.is_empty());
    }
    #[test]
    fn stale_gap_resets_before_new_point_in_either_expiry_order() {
        let start = Instant::now();
        let threshold = Duration::from_millis(750);
        for timer_first in [false, true] {
            let mut cache = Instruments::default();
            for i in 0..5 {
                cache.receive(
                    Field::IndicatedAirspeed,
                    100.0 + i as f64,
                    start + Duration::from_millis(i * 100),
                );
            }
            assert!(cache.acceleration.is_some());
            let next = start + Duration::from_millis(1150);
            if timer_first {
                cache.clear_if_stale(next, threshold);
            }
            // Acquisition checks the gap even if the monitor has not run.
            cache.clear_if_stale(next, threshold);
            for i in 0..5 {
                cache.receive(
                    Field::IndicatedAirspeed,
                    120.0 + i as f64,
                    next + Duration::from_millis(i * 100),
                );
                if i < 4 {
                    assert!(cache.acceleration.is_none());
                }
            }
            assert!((cache.acceleration.unwrap() - 10.0).abs() < 1e-9);
        }
    }
    #[test]
    fn regression_uses_all_unevenly_spaced_points_and_replacement() {
        let start = Instant::now();
        let mut cache = Instruments::default();
        for (ms, value) in [
            (0, 100.0),
            (100, 101.0),
            (300, 104.0),
            (600, 102.0),
            (1000, 110.0),
        ] {
            cache.receive(
                Field::IndicatedAirspeed,
                value,
                start + Duration::from_millis(ms),
            );
        }
        let first = cache.acceleration.unwrap();
        assert!((first - 8.636363636363637).abs() < 1e-9);
        cache.receive(
            Field::IndicatedAirspeed,
            112.0,
            start + Duration::from_millis(1000),
        );
        assert_eq!(cache.ias_history.len(), 5);
        assert!(cache.acceleration.unwrap() > first);
        cache.clear();
        for i in 0..5 {
            cache.receive(
                Field::IndicatedAirspeed,
                100.0,
                start + Duration::from_millis(i * 100),
            );
        }
        assert_eq!(cache.acceleration, Some(0.0));
        cache.clear();
        for i in 0..5 {
            cache.receive(
                Field::IndicatedAirspeed,
                100.0 - i as f64,
                start + Duration::from_millis(i * 100),
            );
        }
        assert!((cache.acceleration.unwrap() + 10.0).abs() < 1e-9);
        cache.clear();
        for i in 0..5 {
            cache.receive(
                Field::IndicatedAirspeed,
                f64::MAX,
                start + Duration::from_millis(i * 100),
            );
        }
        assert!(cache.acceleration.is_none());
    }
    #[test]
    fn baro_requires_valid_pressure() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        cache.receive(Field::BaroSetting, 29.92, now);
        let reading = snapshot(&cache);
        assert!(
            (reading["baro_setting"]["pressure_hpa"].as_f64().unwrap() - 1013.2075).abs() < 0.001
        );
        cache.receive(Field::BaroSetting, 0.0, now + STALE_AFTER);
        assert!(snapshot(&cache).get("baro_setting").is_none());
    }
    #[test]
    #[ignore = "requires an explicitly supplied local IAS replay projection"]
    fn replay_local_ias_projection() {
        use std::io::{BufRead, BufReader};
        let path = std::env::var_os("IAS_REPLAY_PATH")
            .expect("set IAS_REPLAY_PATH to the local projection");
        let input = BufReader::new(std::fs::File::open(path).unwrap());
        let base = Instant::now();
        let mut cache = Instruments::default();
        let mut magnitudes = Vec::new();
        let mut samples = 0;
        for line in input.lines() {
            let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
            if row["kind"] == "state_transition" || row["kind"] == "session_end" {
                cache.clear();
            } else if row["kind"] == "instrument_sample" && row["active"] == true {
                let micros: u64 = row["elapsed_us"].as_str().unwrap().parse().unwrap();
                let value = row["value_knots"].as_f64().unwrap_or(f64::NAN);
                let at = base + Duration::from_micros(micros);
                cache.clear_if_stale(at, STALE_AFTER);
                cache.receive(Field::IndicatedAirspeed, value, at);
                samples += 1;
                if let Some(value) = cache.acceleration {
                    magnitudes.push(value.abs());
                }
            }
        }
        magnitudes.sort_by(f64::total_cmp);
        assert!(!magnitudes.is_empty());
        let quantile =
            |fraction: f64| magnitudes[((magnitudes.len() - 1) as f64 * fraction) as usize];
        println!(
            "samples={samples} estimates={} p95={} p99={} max={} max_offset={}",
            magnitudes.len(),
            quantile(0.95),
            quantile(0.99),
            magnitudes.last().unwrap(),
            magnitudes.last().unwrap() * 6.0
        );
    }
    #[test]
    fn autopilot_targets_use_availability_and_radio_reference_not_master() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        for (i, v) in [
            (Field::Heading, 360.0),
            (Field::AutopilotAvailable, 1.0),
            (Field::SelectedHeading, 0.0),
            (Field::SelectedAltitude, 3000.0),
            (Field::AutopilotAltRadioMode, 0.0),
        ] {
            cache.receive(i, v, now);
        }
        let state = snapshot(&cache);
        assert_eq!(state["heading_deg"], 0.0);
        assert_eq!(state["selected_heading_deg"], 0.0);
        assert_eq!(state["selected_altitude_ft"], 3000.0);
        cache.receive(Field::AutopilotAltRadioMode, 1.0, now);
        assert!(snapshot(&cache).get("selected_altitude_ft").is_none());
    }

    #[test]
    fn native_bool_minus_one_enables_selected_targets() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        for (index, value) in [
            (Field::AutopilotAvailable, -1.0),
            (Field::SelectedHeading, 270.0),
            (Field::SelectedAltitude, 3000.0),
            (Field::AutopilotAltRadioMode, 0.0),
        ] {
            cache.receive(index, value, now);
        }
        let state = snapshot(&cache);
        assert_eq!(state["selected_heading_deg"], 270.0);
        assert_eq!(state["selected_altitude_ft"], 3000.0);
        cache.receive(Field::AutopilotAltRadioMode, -1.0, now);
        let radio_mode = snapshot(&cache);
        assert_eq!(radio_mode["selected_heading_deg"], 270.0);
        assert!(radio_mode.get("selected_altitude_ft").is_none());
        cache.receive(Field::AutopilotAvailable, 0.0, now);
        let unavailable = snapshot(&cache);
        assert!(unavailable.get("selected_heading_deg").is_none());
        assert!(unavailable.get("selected_altitude_ft").is_none());
    }
}
