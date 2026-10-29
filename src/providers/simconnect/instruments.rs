//! Last-value instrument observations owned exclusively by the native worker.
use crate::telemetry::*;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const IAS_WINDOW: Duration = Duration::from_millis(500);
const IAS_HISTORY_LIMIT: usize = 256;

#[derive(Default)]
pub(super) struct Instruments {
    values: [Option<(f64, Instant)>; super::native::INSTRUMENT_FIELDS.len()],
    ias_history: VecDeque<(f64, Instant)>,
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
            self.ias_history.clear();
            self.acceleration = None;
        }
    }
    pub fn receive(&mut self, index: usize, value: f64, now: Instant) {
        self.dirty = true;
        if index == 0 {
            self.acceleration = None;
            if !value.is_finite() || value < 0.0 {
                self.ias_history.clear();
            } else {
                if let Some(&(_, last_at)) = self.ias_history.back() {
                    if now < last_at || now.duration_since(last_at) >= IAS_WINDOW {
                        self.ias_history.clear();
                    } else if now == last_at {
                        self.ias_history.pop_back();
                        self.ias_history.push_back((value, now));
                        self.values[index] = Some((value, now));
                        return;
                    }
                }
                self.ias_history.push_back((value, now));
                while self.ias_history.len() > 2
                    && now.duration_since(self.ias_history[1].1) >= IAS_WINDOW
                {
                    self.ias_history.pop_front();
                }
                if self.ias_history.len() > IAS_HISTORY_LIMIT {
                    self.ias_history.clear();
                    self.ias_history.push_back((value, now));
                } else if let Some(&(previous, at)) = self.ias_history.front() {
                    let elapsed = now.duration_since(at);
                    if elapsed >= IAS_WINDOW {
                        self.acceleration = Some((value - previous) / elapsed.as_secs_f64())
                            .filter(|v| v.is_finite());
                    }
                }
            }
        }
        self.values[index] = value.is_finite().then_some((value, now));
    }
    pub fn expire(&mut self, now: Instant) {
        if self
            .ias_history
            .back()
            .is_some_and(|(_, at)| now.saturating_duration_since(*at) >= IAS_WINDOW)
        {
            if self.acceleration.take().is_some() {
                self.dirty = true;
            }
            self.ias_history.clear();
        }
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
        let baro = get(4).and_then(|pressure| {
            let standard = match get(super::native::BARO_STD_INDEX) {
                Some(0.0) => false,
                Some(1.0) => true,
                _ => return None,
            };
            BaroSetting::new(pressure * HPA_PER_INHG, standard).ok()
        });
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
            baro.map(|setting| format!("{:.2} inHg", setting.pressure_hpa() / HPA_PER_INHG)),
        )
        .with_baro_setting(baro)
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
        cache.receive(0, 100.5, now + Duration::from_millis(250));
        cache.receive(0, 101.0, now + Duration::from_millis(500));
        assert_eq!(cache.acceleration, Some(2.0));
        cache.receive(0, 110.0, now + Duration::from_secs(3));
        assert!(cache.acceleration.is_none());
        cache.clear();
        cache.receive(0, 111.0, now + Duration::from_secs(4));
        assert!(cache.acceleration.is_none());
    }
    #[test]
    fn ias_window_handles_timing_noise_and_bounded_history() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        for i in 0..=10 {
            cache.receive(
                0,
                100.0 + i as f64 * 0.1,
                now + Duration::from_millis(i * 50),
            );
            if i < 10 {
                assert!(cache.acceleration.is_none());
            }
        }
        assert!((cache.acceleration.unwrap() - 2.0).abs() < 1e-9);
        cache.receive(0, 101.0, now + Duration::from_millis(500));
        assert!(cache.acceleration.is_none());
        cache.receive(0, 100.0, now + Duration::from_millis(400));
        assert_eq!(cache.ias_history.len(), 1);
        cache.receive(0, 101.0, now + Duration::from_millis(1400));
        assert_eq!(cache.ias_history.len(), 1);
        cache.receive(0, f64::NAN, now + Duration::from_millis(1410));
        assert!(cache.ias_history.is_empty());
        for i in 0..400 {
            cache.receive(0, 100.0, now + Duration::from_micros(2_000_000 + i));
            assert!(cache.ias_history.len() <= IAS_HISTORY_LIMIT);
            assert!(cache.acceleration.is_none());
        }
    }
    #[test]
    fn acceleration_expires_at_its_window_without_expiring_ias() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        cache.receive(0, 100.0, now);
        cache.receive(0, 100.5, now + Duration::from_millis(250));
        cache.receive(0, 101.0, now + Duration::from_millis(500));
        cache.dirty = false;
        cache.expire(now + Duration::from_millis(999));
        assert_eq!(cache.acceleration, Some(2.0));
        cache.expire(now + Duration::from_millis(1000));
        assert!(cache.acceleration.is_none());
        assert!(cache.values[0].is_some());
        assert!(cache.dirty);
        cache.receive(0, 102.0, now + Duration::from_millis(1001));
        assert!(cache.acceleration.is_none());
    }
    #[test]
    fn equal_timestamp_does_not_keep_history_across_an_expired_window() {
        let now = Instant::now();
        let mut cache = Instruments::default();
        cache.receive(0, 100.0, now);
        cache.receive(0, 100.5, now + Duration::from_millis(250));
        cache.receive(0, 101.0, now + Duration::from_millis(500));
        cache.receive(0, 101.0, now + Duration::from_millis(500));
        assert!(cache.acceleration.is_none());
        cache.expire(now + Duration::from_millis(1000));
        assert!(cache.ias_history.is_empty());
        cache.receive(0, 102.0, now + Duration::from_millis(1100));
        assert!(cache.acceleration.is_none());
    }
    #[test]
    fn exact_window_gap_resets_regardless_of_expiry_order() {
        let now = Instant::now();
        for expire_first in [false, true] {
            let mut cache = Instruments::default();
            cache.receive(0, 100.0, now);
            cache.receive(0, 100.5, now + Duration::from_millis(250));
            cache.receive(0, 101.0, now + Duration::from_millis(500));
            assert_eq!(cache.acceleration, Some(2.0));
            let next = now + Duration::from_millis(1000);
            if expire_first {
                cache.expire(next);
            }
            cache.receive(0, 102.0, next);
            assert!(cache.acceleration.is_none());
            assert_eq!(cache.ias_history.len(), 1);
        }
    }
    #[test]
    fn ias_window_meets_noise_and_response_budgets() {
        let start = Instant::now();
        let mut cache = Instruments::default();
        let mut reached_90 = None;
        for i in 0..100_u64 {
            let t = i as f64 / 20.0;
            let speed = 100.0 + 2.0 * (t - 1.0).max(0.0);
            cache.receive(0, speed, start + Duration::from_millis(i * 50));
            if t >= 1.0 && cache.acceleration.is_some_and(|a| a >= 1.8) && reached_90.is_none() {
                reached_90 = Some(t - 1.0);
            }
            if t >= 3.0 {
                assert!((cache.acceleration.unwrap() - 2.0).abs() < 0.01);
            }
        }
        assert!(reached_90.unwrap() <= 0.6);
        cache.clear();
        for i in 0..100_u64 {
            let noisy = 100.0 + if i % 2 == 0 { -0.01 } else { 0.01 };
            cache.receive(0, noisy, start + Duration::from_millis(i * 50));
            if let Some(a) = cache.acceleration {
                assert!(a.abs() <= 0.05);
            }
        }
        cache.clear();
        let mut reversal_90 = None;
        for i in 0..140_u64 {
            let t = i as f64 / 20.0;
            let speed = if t < 1.0 {
                100.0
            } else if t <= 3.0 {
                100.0 + 2.0 * (t - 1.0)
            } else {
                104.0 - 2.0 * (t - 3.0)
            };
            cache.receive(0, speed, start + Duration::from_millis(i * 50));
            if t >= 3.0 && cache.acceleration.is_some_and(|a| a <= -1.8) && reversal_90.is_none() {
                reversal_90 = Some(t - 3.0);
            }
        }
        assert!(reversal_90.unwrap() <= 0.6);
    }
    #[test]
    fn baro_requires_fresh_pressure_and_explicit_valid_mode() {
        let now = Instant::now();
        for pressure_first in [true, false] {
            let mut cache = Instruments::default();
            let first = if pressure_first {
                (4, 29.92)
            } else {
                (17, 0.0)
            };
            let second = if pressure_first {
                (17, 0.0)
            } else {
                (4, 29.92)
            };
            cache.receive(first.0, first.1, now);
            assert!(snapshot(&cache).get("baro_setting").is_none());
            assert!(snapshot(&cache).get("baro_text").is_none());
            cache.receive(second.0, second.1, now);
            let reading = snapshot(&cache);
            assert_eq!(reading["baro_setting"]["standard"], false);
            assert!(
                (reading["baro_setting"]["pressure_hpa"].as_f64().unwrap() - 1013.2075).abs()
                    < 0.001
            );
            cache.receive(17, 1.0, now + Duration::from_millis(1));
            assert_eq!(snapshot(&cache)["baro_setting"]["standard"], true);
            cache.receive(17, 2.0, now + Duration::from_millis(2));
            assert!(snapshot(&cache).get("baro_setting").is_none());
            assert!(snapshot(&cache).get("baro_text").is_none());
            cache.receive(17, 0.0, now + Duration::from_millis(3));
            cache.expire(now + STALE_AFTER);
            assert!(snapshot(&cache).get("baro_setting").is_none());
        }
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
                cache.receive(0, value, base + Duration::from_micros(micros));
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
