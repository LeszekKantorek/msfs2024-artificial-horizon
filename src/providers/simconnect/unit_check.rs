//! Diagnostic observations only; repeated reads never decide telemetry validity.
use super::{journal::number, lifecycle::UNIT_TOLERANCE_DEG};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Serialize)]
pub(super) struct Observation {
    pub primary_mismatch: bool,
    pub repeats_changed: bool,
    pub repeats_nonfinite: bool,
}

impl Observation {
    pub fn anomalous(&self) -> bool {
        self.primary_mismatch || self.repeats_changed || self.repeats_nonfinite
    }
}

fn agree(radians: f64, degrees: f64) -> bool {
    radians.is_finite()
        && degrees.is_finite()
        && (radians.to_degrees() - degrees).abs() <= UNIT_TOLERANCE_DEG
}

pub(super) fn observe(values: [f64; 4], repeated: Option<[f64; 2]>) -> Observation {
    Observation {
        primary_mismatch: !agree(values[0], values[1]) || !agree(values[2], values[3]),
        repeats_changed: repeated.is_some_and(|[pitch, bank]| {
            pitch.to_bits() != values[0].to_bits() || bank.to_bits() != values[2].to_bits()
        }),
        repeats_nonfinite: repeated.is_some_and(|items| items.iter().any(|x| !x.is_finite())),
    }
}

fn finite(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

fn axis(radians: f64, degrees: f64, repeated: Option<f64>) -> Value {
    let converted = radians.to_degrees();
    let repeat = repeated.map(|value| {
        let repeat_deg = value.to_degrees();
        let between = if converted.is_finite() && degrees.is_finite() && repeat_deg.is_finite() {
            Some(
                degrees >= converted.min(repeat_deg) - UNIT_TOLERANCE_DEG
                    && degrees <= converted.max(repeat_deg) + UNIT_TOLERANCE_DEG,
            )
        } else {
            None
        };
        json!({
            "radians": number(value),
            "as_degrees": finite(repeat_deg),
            "radians_minus_degrees_deg": finite(repeat_deg - degrees),
            "matches_degrees": agree(value, degrees),
            "original_repeat_bits_equal": radians.to_bits() == value.to_bits(),
            "repeat_minus_original_deg": finite(repeat_deg - converted),
            "degrees_within_radian_read_bounds": between,
        })
    });
    json!({
        "radians": number(radians), "degrees": number(degrees),
        "radians_as_degrees": finite(converted),
        "radians_minus_degrees_deg": finite(converted - degrees),
        "units_agree": agree(radians, degrees), "repeat": repeat,
    })
}

pub(super) fn details(values: [f64; 4], repeated: Option<[f64; 2]>) -> Value {
    json!({
        "unit_tolerance_deg": UNIT_TOLERANCE_DEG,
        "observation": observe(values, repeated),
        "pitch": axis(values[0], values[1], repeated.map(|v| v[0])),
        "bank": axis(values[2], values[3], repeated.map(|v| v[1])),
        "interpretation": "diagnostic evidence only; equal repeated reads do not prove atomicity",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_pairs_and_repeats_do_not_generate_an_anomaly() {
        let v = [
            0.25_f64,
            0.25_f64.to_degrees(),
            -0.5,
            (-0.5_f64).to_degrees(),
        ];
        assert!(!observe(v, Some([v[0], v[2]])).anomalous());
        assert!(!observe(v, None).anomalous());
    }

    #[test]
    fn changing_reads_are_distinguished_from_equal_repeats_with_wrong_units() {
        let v = [1_f64.to_radians(), 1.1, -2_f64.to_radians(), -2.1];
        let changed = Some([1.1_f64.to_radians(), -2.1_f64.to_radians()]);
        let o = observe(v, changed);
        assert!(o.primary_mismatch && o.repeats_changed && !o.repeats_nonfinite);
        let d = details(v, changed);
        for name in ["pitch", "bank"] {
            assert_eq!(d[name]["repeat"]["matches_degrees"], true);
            assert_eq!(d[name]["repeat"]["degrees_within_radian_read_bounds"], true);
        }
        let d = details(v, Some([v[0], v[2]]));
        assert_eq!(d["pitch"]["repeat"]["original_repeat_bits_equal"], true);
        assert_eq!(d["pitch"]["repeat"]["matches_degrees"], false);
        assert_eq!(
            d["pitch"]["repeat"]["degrees_within_radian_read_bounds"],
            false
        );
    }

    #[test]
    fn repeats_can_change_even_if_original_unit_pair_matches() {
        let v = [0., 0., 0., 0.];
        let o = observe(v, Some([0.01, 0.]));
        assert!(!o.primary_mismatch && o.repeats_changed);
        assert!(o.anomalous());
    }

    #[test]
    fn nonfinite_and_overflowed_diagnostics_are_json_safe() {
        let v = [f64::NAN, 1., f64::MAX, f64::MAX];
        let o = observe(v, Some([f64::INFINITY, f64::NEG_INFINITY]));
        assert!(o.primary_mismatch && o.repeats_nonfinite);
        let d = details(v, Some([f64::INFINITY, f64::NEG_INFINITY]));
        assert!(d["pitch"]["repeat"]["as_degrees"].is_null());
        assert!(d["bank"]["radians_as_degrees"].is_null());
        assert!(d["pitch"]["repeat"]["degrees_within_radian_read_bounds"].is_null());
        assert!(serde_json::to_string(&d).is_ok());
    }

    #[test]
    fn observed_historical_pair_is_not_misreported_as_conversion_roundoff() {
        let values = [
            -0.031706,
            (-0.031706_f64).to_degrees(),
            -0.005221730230741045,
            -0.29662718785636477,
        ];
        let d = details(values, Some([values[0], values[2]]));
        assert_eq!(d["bank"]["units_agree"], false);
        assert!(
            (d["bank"]["radians_minus_degrees_deg"].as_f64().unwrap() + 0.002555916120970614).abs()
                < 1e-14
        );
        assert_eq!(d["bank"]["repeat"]["original_repeat_bits_equal"], true);
        // Diagnostic changes cannot authorize the original inconsistent four fields.
        assert!(super::super::lifecycle::candidate_attitude(values).is_err());
    }
}
