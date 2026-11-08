use serde::Deserialize;
use std::collections::HashSet;

pub const CATALOG: &[(&str, &str)] = &[
    ("PLANE PITCH DEGREES", "radians"),
    ("PLANE PITCH DEGREES", "degrees"),
    ("PLANE BANK DEGREES", "radians"),
    ("PLANE BANK DEGREES", "degrees"),
    ("AIRSPEED INDICATED", "knots"),
    ("AIRSPEED TRUE", "knots"),
    ("GROUND VELOCITY", "knots"),
    ("INDICATED ALTITUDE:1", "feet"),
    ("KOHLSMAN SETTING HG:1", "inHg"),
    ("VERTICAL SPEED", "feet per minute"),
    ("PLANE HEADING DEGREES MAGNETIC", "degrees"),
    ("TURN COORDINATOR BALL", "position 128"),
    ("TURN INDICATOR RATE", "degrees per second"),
    ("DESIGN SPEED VS0", "knots"),
    ("DESIGN SPEED VS1", "knots"),
    ("DESIGN SPEED MIN ROTATION", "knots"),
    ("AIRSPEED BARBER POLE", "knots"),
    ("AUTOPILOT AVAILABLE", "bool"),
    ("AUTOPILOT HEADING LOCK DIR:0", "degrees"),
    ("AUTOPILOT ALTITUDE LOCK VAR:0", "feet"),
    ("AUTOPILOT ALT RADIO MODE", "bool"),
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Initial {
    pub sim_running: bool,
    pub pause: Pause,
    pub flight: Option<String>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    pub simvar: Option<String>,
    pub unit: Option<String>,
}
#[derive(Clone)]
pub struct ApiFault {
    pub operation: String,
    pub occurrence: u64,
    pub hresult: i32,
    pub selector: Option<Selector>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pause {
    Off,
    Normal,
    Active,
}
impl Pause {
    pub fn flags(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Normal => 8,
            Self::Active => 4,
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Record {
    Scenario {
        format: String,
        version: u32,
        mode: String,
        frame_period_ms: u64,
        duration_us: u64,
        initial: Initial,
    },
    Signal {
        id: String,
        simvar: String,
        unit: String,
        repeat: bool,
    },
    Hold {
        signal: String,
        value: f64,
        duration_ms: u64,
    },
    Ramp {
        signal: String,
        from: f64,
        to: f64,
        duration_ms: u64,
    },
    Gap {
        signal: String,
        duration_ms: u64,
    },
    Pause {
        at_us: u64,
        mode: Pause,
    },
    Flight {
        at_us: u64,
        active: bool,
    },
    FlightLoaded {
        at_us: u64,
        flight: Option<String>,
    },
    Disconnect {
        at_us: u64,
        unavailable_ms: u64,
    },
    TransportSilence {
        at_us: u64,
        duration_ms: u64,
    },
    ApiFault {
        operation: String,
        occurrence: u64,
        hresult: i32,
        selector: Option<Selector>,
    },
    RawPacket {
        at_us: u64,
        bytes_hex: String,
        returned_length: u32,
    },
    Exception {
        at_us: u64,
        sent_operation: String,
        sent_call_occurrence: usize,
        code: u32,
        index: u32,
    },
}
#[derive(Clone)]
pub enum Event {
    Pause(Pause),
    Flight(bool),
    FlightLoaded(Option<String>),
    Disconnect(u64),
    TransportSilence(u64),
    RawPacket {
        bytes: Vec<u8>,
        returned_length: u32,
    },
    Exception {
        sent_operation: String,
        sent_call_occurrence: usize,
        code: u32,
        index: u32,
    },
}
#[derive(Clone)]
pub struct TimedEvent {
    pub at: u64,
    pub order: usize,
    pub event: Event,
}
#[derive(Clone)]
enum SegmentValue {
    Hold(f64),
    Ramp(f64, f64),
    Gap,
}
#[derive(Clone)]
struct Segment {
    order: usize,
    end: u64,
    value: SegmentValue,
}
#[derive(Clone)]
struct Track {
    id: String,
    declaration_line: usize,
    name: String,
    unit: String,
    repeat: bool,
    segments: Vec<Segment>,
    duration: u64,
}
#[derive(Clone)]
pub struct Scenario {
    pub frame_us: u64,
    pub duration: u64,
    pub initial: Initial,
    pub events: Vec<TimedEvent>,
    pub faults: Vec<ApiFault>,
    tracks: Vec<Track>,
}
impl Scenario {
    pub fn parse(text: &str, source: &str) -> Result<Self, String> {
        if text.len() > 4 * 1024 * 1024 {
            return Err(format!("{source}: file exceeds 4 MiB"));
        }
        let mut scenario: Option<Self> = None;
        let mut ids = HashSet::new();
        let mut pairs = HashSet::new();
        for (i, line) in text.lines().enumerate() {
            let loc = format!("{source}: line {}", i + 1);
            if i >= 100_000 || line.len() > 65536 {
                return Err(format!("{loc}: record or line limit exceeded"));
            }
            let record: Record = serde_json::from_str(line).map_err(|e| format!("{loc}: {e}"))?;
            match record {
                Record::Scenario {
                    format,
                    version,
                    mode,
                    frame_period_ms,
                    duration_us,
                    initial,
                } if i == 0 => {
                    if format != "msfs-efi-simconnect-demo"
                        || version != 1
                        || mode != "scripted"
                        || frame_period_ms == 0
                        || duration_us == 0
                    {
                        return Err(format!("{loc}: invalid scenario header"));
                    }
                    let frame_us = frame_period_ms
                        .checked_mul(1000)
                        .ok_or_else(|| format!("{loc}: frame_period_ms overflow"))?;
                    if initial
                        .flight
                        .as_ref()
                        .is_some_and(|s| s.len() >= 260 || s.contains('\0'))
                    {
                        return Err(format!("{loc}: initial.flight is invalid"));
                    }
                    scenario = Some(Self {
                        frame_us,
                        duration: duration_us,
                        initial,
                        events: vec![],
                        faults: vec![],
                        tracks: vec![],
                    });
                }
                Record::Scenario { .. } => {
                    return Err(format!("{loc}: scenario header must be first and unique"));
                }
                record => {
                    let s = scenario
                        .as_mut()
                        .ok_or_else(|| format!("{loc}: missing scenario header"))?;
                    match record {
                        Record::Signal {
                            id,
                            simvar,
                            unit,
                            repeat,
                        } => {
                            if id.is_empty()
                                || !ids.insert(id.clone())
                                || !pairs.insert((simvar.clone(), unit.clone()))
                                || !CATALOG.contains(&(simvar.as_str(), unit.as_str()))
                            {
                                return Err(format!(
                                    "{loc}: invalid or duplicate signal id or simvar/unit"
                                ));
                            }
                            s.tracks.push(Track {
                                id,
                                declaration_line: i + 1,
                                name: simvar,
                                unit,
                                repeat,
                                segments: vec![],
                                duration: 0,
                            });
                        }
                        Record::Hold {
                            signal,
                            value,
                            duration_ms,
                        } => s.segment(&loc, &signal, duration_ms, SegmentValue::Hold(value), i)?,
                        Record::Ramp {
                            signal,
                            from,
                            to,
                            duration_ms,
                        } => {
                            s.segment(&loc, &signal, duration_ms, SegmentValue::Ramp(from, to), i)?
                        }
                        Record::Gap {
                            signal,
                            duration_ms,
                        } => s.segment(&loc, &signal, duration_ms, SegmentValue::Gap, i)?,
                        Record::Pause { at_us, mode } => {
                            s.event(&loc, at_us, Event::Pause(mode), i)?
                        }
                        Record::Flight { at_us, active } => {
                            s.event(&loc, at_us, Event::Flight(active), i)?
                        }
                        Record::FlightLoaded { at_us, flight } => {
                            if flight
                                .as_ref()
                                .is_some_and(|v| v.len() >= 260 || v.contains('\0'))
                            {
                                return Err(format!("{loc}: invalid flight"));
                            }
                            s.event(&loc, at_us, Event::FlightLoaded(flight), i)?;
                        }
                        Record::Disconnect {
                            at_us,
                            unavailable_ms,
                        } => {
                            let duration = unavailable_ms
                                .checked_mul(1000)
                                .filter(|v| *v > 0)
                                .ok_or_else(|| format!("{loc}: invalid unavailable_ms"))?;
                            at_us
                                .checked_add(duration)
                                .ok_or_else(|| format!("{loc}: disconnect deadline overflow"))?;
                            s.event(&loc, at_us, Event::Disconnect(duration), i)?;
                        }
                        Record::TransportSilence { at_us, duration_ms } => {
                            let duration = duration_ms
                                .checked_mul(1000)
                                .filter(|v| *v > 0)
                                .ok_or_else(|| format!("{loc}: invalid duration_ms"))?;
                            at_us
                                .checked_add(duration)
                                .ok_or_else(|| format!("{loc}: silence deadline overflow"))?;
                            s.event(&loc, at_us, Event::TransportSilence(duration), i)?;
                        }
                        Record::ApiFault {
                            operation,
                            occurrence,
                            hresult,
                            selector,
                        } => {
                            if ![
                                "SimConnect_Open",
                                "SimConnect_AddToDataDefinition",
                                "SimConnect_SubscribeToSystemEvent",
                                "SimConnect_RequestDataOnSimObject",
                                "SimConnect_RequestSystemState",
                                "SimConnect_GetLastSentPacketID",
                                "SimConnect_Close",
                            ]
                            .contains(&operation.as_str())
                                || occurrence == 0
                                || hresult >= 0
                            {
                                return Err(format!(
                                    "{loc}: unsupported api_fault operation, occurrence, or hresult"
                                ));
                            }
                            if let Some(ref filter) = selector
                                && (![
                                    "SimConnect_AddToDataDefinition",
                                    "SimConnect_RequestDataOnSimObject",
                                ]
                                .contains(&operation.as_str())
                                    || (filter.simvar.is_none() && filter.unit.is_none())
                                    || !CATALOG.iter().any(|(name, unit)| {
                                        filter.simvar.as_deref().is_none_or(|v| v == *name)
                                            && filter.unit.as_deref().is_none_or(|v| v == *unit)
                                    }))
                            {
                                return Err(format!("{loc}: unsupported api_fault selector"));
                            }
                            s.faults.push(ApiFault {
                                operation,
                                occurrence,
                                hresult,
                                selector,
                            });
                        }
                        Record::RawPacket {
                            at_us,
                            bytes_hex,
                            returned_length,
                        } => {
                            if bytes_hex.len() % 2 != 0
                                || !bytes_hex.bytes().all(|b| b.is_ascii_hexdigit())
                            {
                                return Err(format!(
                                    "{loc}: bytes_hex must contain complete hexadecimal bytes"
                                ));
                            }
                            let bytes: Vec<u8> = bytes_hex
                                .as_bytes()
                                .as_chunks::<2>()
                                .0
                                .iter()
                                .map(|pair| {
                                    let digit = |c: u8| (c as char).to_digit(16).unwrap() as u8;
                                    digit(pair[0]) * 16 + digit(pair[1])
                                })
                                .collect();
                            if returned_length as usize > bytes.len() {
                                return Err(format!(
                                    "{loc}: returned_length exceeds decoded allocation"
                                ));
                            }
                            s.event(
                                &loc,
                                at_us,
                                Event::RawPacket {
                                    bytes,
                                    returned_length,
                                },
                                i,
                            )?;
                        }
                        Record::Exception {
                            at_us,
                            sent_operation,
                            sent_call_occurrence,
                            code,
                            index,
                        } => {
                            if sent_call_occurrence == 0
                                || ![
                                    "SimConnect_AddToDataDefinition",
                                    "SimConnect_RequestDataOnSimObject",
                                    "SimConnect_SubscribeToSystemEvent",
                                    "SimConnect_RequestSystemState",
                                ]
                                .contains(&sent_operation.as_str())
                            {
                                return Err(format!("{loc}: unsupported exception call reference"));
                            }
                            s.event(
                                &loc,
                                at_us,
                                Event::Exception {
                                    sent_operation,
                                    sent_call_occurrence,
                                    code,
                                    index,
                                },
                                i,
                            )?;
                        }
                        Record::Scenario { .. } => unreachable!(),
                    }
                }
            }
        }
        let mut s = scenario.ok_or_else(|| format!("{source}: missing scenario header"))?;
        if let Some(track) = s.tracks.iter().find(|t| t.segments.is_empty()) {
            return Err(format!(
                "{source}: line {}: signal id {:?} has no segments",
                track.declaration_line, track.id
            ));
        }
        s.events.sort_by_key(|e| e.at);
        Ok(s)
    }
    fn segment(
        &mut self,
        loc: &str,
        id: &str,
        ms: u64,
        value: SegmentValue,
        order: usize,
    ) -> Result<(), String> {
        let t = self
            .tracks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("{loc}: unknown signal {id}"))?;
        if ms == 0 {
            return Err(format!("{loc}: duration_ms must be positive"));
        }
        if t.unit == "radians" {
            let convertible = |number: f64| number.to_degrees().is_finite();
            let invalid = match &value {
                SegmentValue::Hold(v) => !convertible(*v),
                SegmentValue::Ramp(from, to) => !convertible(*from) || !convertible(*to),
                SegmentValue::Gap => false,
            };
            if invalid {
                return Err(format!(
                    "{loc}: signal {id:?} value cannot be represented in degrees"
                ));
            }
        }
        let n = ms
            .checked_mul(1000)
            .and_then(|n| t.duration.checked_add(n))
            .ok_or_else(|| format!("{loc}: duration_ms overflow"))?;
        t.duration = n;
        t.segments.push(Segment {
            end: n,
            value,
            order,
        });
        Ok(())
    }
    fn event(&mut self, loc: &str, at: u64, event: Event, order: usize) -> Result<(), String> {
        if at >= self.duration {
            return Err(format!("{loc}: at_us outside pass"));
        }
        self.events.push(TimedEvent { at, event, order });
        Ok(())
    }
    #[cfg(test)]
    pub fn value(&self, name: &str, unit: &str, elapsed: u64) -> Option<f64> {
        self.ordered_value(name, unit, elapsed)
            .map(|(value, _)| value)
    }
    pub fn ordered_value(&self, name: &str, unit: &str, elapsed: u64) -> Option<(f64, usize)> {
        let (track, convert) = self
            .tracks
            .iter()
            .find(|t| t.name == name && t.unit == unit)
            .map(|t| (t, false))
            .or_else(|| {
                if (name == "PLANE PITCH DEGREES" || name == "PLANE BANK DEGREES")
                    && (unit == "degrees" || unit == "radians")
                {
                    self.tracks
                        .iter()
                        .find(|t| t.name == name && (t.unit == "degrees" || t.unit == "radians"))
                        .map(|t| (t, true))
                } else {
                    None
                }
            })?;
        let mut time = elapsed % self.duration;
        if track.repeat {
            time %= track.duration;
        } else if time >= track.duration {
            return None;
        }
        let mut start = 0;
        for segment in &track.segments {
            if time < segment.end {
                let value = match segment.value {
                    SegmentValue::Hold(v) => v,
                    SegmentValue::Ramp(a, b) => {
                        let fraction = (time - start) as f64 / (segment.end - start) as f64;
                        let interpolated = if a.is_sign_negative() != b.is_sign_negative() {
                            a * (1.0 - fraction) + b * fraction
                        } else {
                            a + (b - a) * fraction
                        };
                        interpolated.clamp(a.min(b), a.max(b))
                    }
                    SegmentValue::Gap => return None,
                };
                return Some((
                    if convert {
                        if unit == "degrees" {
                            value.to_degrees()
                        } else {
                            value.to_radians()
                        }
                    } else {
                        value
                    },
                    segment.order,
                ));
            }
            start = segment.end;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn header() -> &'static str {
        r#"{"type":"scenario","format":"msfs-efi-simconnect-demo","version":1,"mode":"scripted","frame_period_ms":50,"duration_us":15000000,"initial":{"sim_running":true,"pause":"off","flight":"demo.flt"}}"#
    }
    #[test]
    fn scenario_suite_parses_and_tour_matches_independent_checkpoints() {
        let fault_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../demo-scenarios/errors");
        for entry in std::fs::read_dir(fault_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                let input = std::fs::read_to_string(&path).unwrap();
                Scenario::parse(&input, &path.display().to_string()).unwrap();
            }
        }
        let tour = include_str!("../../../demo-scenarios/flight-tour.jsonl");
        for (name, input) in [
            ("tour", tour),
            (
                "lifecycle",
                include_str!("../../../demo-scenarios/simulator-lifecycle.jsonl"),
            ),
            (
                "availability",
                include_str!("../../../demo-scenarios/availability-gaps.jsonl"),
            ),
            (
                "ias-only",
                include_str!("../../../demo-scenarios/ias-only.jsonl"),
            ),
            (
                "event-only",
                include_str!("../../../demo-scenarios/event-only.jsonl"),
            ),
            (
                "no-signal",
                include_str!("../../../demo-scenarios/no-signal.jsonl"),
            ),
            (
                "startup",
                include_str!("../../../demo-scenarios/errors/startup-retry.jsonl"),
            ),
            (
                "optional",
                include_str!("../../../demo-scenarios/errors/optional-definition.jsonl"),
            ),
            (
                "subscribe",
                include_str!("../../../demo-scenarios/errors/primary-subscribe.jsonl"),
            ),
            (
                "truncated",
                include_str!("../../../demo-scenarios/errors/truncated-packet.jsonl"),
            ),
        ] {
            Scenario::parse(input, name).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
        let scenario = Scenario::parse(tour, "tour").unwrap();
        let checkpoints: Vec<serde_json::Value> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/flight-tour-checkpoints.json"
        ))
        .unwrap();
        for row in checkpoints {
            let time = row["time_us"].as_u64().unwrap();
            for (key, native, unit) in [
                ("ias", "AIRSPEED INDICATED", "knots"),
                ("altitude", "INDICATED ALTITUDE:1", "feet"),
                ("vsi", "VERTICAL SPEED", "feet per minute"),
                ("heading", "PLANE HEADING DEGREES MAGNETIC", "degrees"),
            ] {
                let actual = scenario.value(native, unit, time).unwrap();
                let expected = row[key].as_f64().unwrap();
                assert!(
                    (actual - expected).abs() < 1e-9,
                    "{key} at {time}: expected {expected}, got {actual}"
                );
            }
        }
        // All tracks hold their values for both authored pause windows.
        for track in &scenario.tracks {
            for (start, end) in [(12_000_000, 14_000_000), (16_000_000, 18_000_000)] {
                let first = scenario.value(&track.name, &track.unit, start);
                for t in (start..end).step_by(50_000) {
                    assert_eq!(
                        scenario.value(&track.name, &track.unit, t),
                        first,
                        "{} at {t}",
                        track.id
                    );
                }
            }
        }
    }
    #[test]
    fn fault_records_validate_operations_selectors_and_readable_lengths() {
        for invalid in [
            r#"{"type":"api_fault","operation":"SimConnect_Unknown","occurrence":1,"hresult":-1}"#,
            r#"{"type":"api_fault","operation":"SimConnect_Open","occurrence":0,"hresult":-1}"#,
            r#"{"type":"api_fault","operation":"SimConnect_Open","occurrence":1,"hresult":0}"#,
            r#"{"type":"api_fault","operation":"SimConnect_Open","occurrence":1,"hresult":-1,"selector":{"simvar":"AIRSPEED INDICATED"}}"#,
            r#"{"type":"api_fault","operation":"SimConnect_AddToDataDefinition","occurrence":1,"hresult":-1,"selector":{"unit":"knot"}}"#,
            r#"{"type":"raw_packet","at_us":1,"bytes_hex":"0","returned_length":0}"#,
            r#"{"type":"raw_packet","at_us":1,"bytes_hex":"zz","returned_length":1}"#,
            r#"{"type":"raw_packet","at_us":1,"bytes_hex":"00","returned_length":2}"#,
            r#"{"type":"transport_silence","at_us":1,"duration_ms":0}"#,
            r#"{"type":"exception","at_us":1,"sent_operation":"SimConnect_RequestDataOnSimObject","sent_call_occurrence":0,"code":2,"index":0}"#,
        ] {
            assert!(
                Scenario::parse(&format!("{}\n{}", header(), invalid), "test").is_err(),
                "{invalid}"
            );
        }
        let valid = format!(
            "{}\n{}\n{}",
            header(),
            r#"{"type":"api_fault","operation":"SimConnect_AddToDataDefinition","occurrence":2,"hresult":-1,"selector":{"unit":"knots"}}"#,
            r#"{"type":"raw_packet","at_us":1,"bytes_hex":"01020304","returned_length":2}"#
        );
        let scenario = Scenario::parse(&valid, "test").unwrap();
        assert_eq!(scenario.faults.len(), 1);
        assert!(
            matches!(&scenario.events[0].event, Event::RawPacket { bytes, returned_length: 2 } if bytes == &[1,2,3,4])
        );
    }
    #[test]
    fn holds_ramps_gaps_and_passes_follow_absolute_time() {
        let input = format!(
            "{}\n{}",
            header(),
            r#"{"type":"signal","id":"ias","simvar":"AIRSPEED INDICATED","unit":"knots","repeat":false}
{"type":"hold","signal":"ias","value":100,"duration_ms":5000}
{"type":"ramp","signal":"ias","from":100,"to":150,"duration_ms":5000}
{"type":"hold","signal":"ias","value":150,"duration_ms":1000}
{"type":"gap","signal":"ias","duration_ms":4000}"#
        );
        let s = Scenario::parse(&input, "fixture").unwrap();
        for t in [0, 50_000, 4_999_999] {
            assert_eq!(s.value("AIRSPEED INDICATED", "knots", t), Some(100.0));
        }
        assert_eq!(
            s.value("AIRSPEED INDICATED", "knots", 7_500_000),
            Some(125.0)
        );
        assert_eq!(
            s.value("AIRSPEED INDICATED", "knots", 10_000_000),
            Some(150.0)
        );
        assert_eq!(s.value("AIRSPEED INDICATED", "knots", 11_000_000), None);
        assert_eq!(
            s.value("AIRSPEED INDICATED", "knots", 15_000_000),
            Some(100.0)
        );
    }
    #[test]
    fn sparse_empty_and_invalid_inputs() {
        assert!(Scenario::parse(header(), "empty").is_ok());
        assert!(
            Scenario::parse(
                &format!(
                    "{}\n{}",
                    header(),
                    r#"{"type":"pause","at_us":1,"mode":"normal"}"#
                ),
                "events"
            )
            .is_ok()
        );
        assert!(Scenario::parse(&format!("{}\n{}",header(),r#"{"type":"signal","id":"bad","simvar":"AIRSPEED INDICATED","unit":"knot","repeat":true}"#),"bad").err().unwrap().contains("line 2"));
        assert!(
            Scenario::parse(
                &format!(
                    "{}\n{}",
                    header(),
                    r#"{"type":"flight","at_us":1,"active":true,"active":false}"#
                ),
                "bad"
            )
            .is_err()
        );
    }

    #[test]
    fn extreme_ramps_remain_finite_and_radians_must_convert() {
        let input = format!(
            "{}\n{}",
            header(),
            r#"{"type":"signal","id":"ias","simvar":"AIRSPEED INDICATED","unit":"knots","repeat":false}
{"type":"ramp","signal":"ias","from":-1e308,"to":1e308,"duration_ms":5000}"#
        );
        let scenario = Scenario::parse(&input, "extreme.jsonl").unwrap();
        for (time, expected_sign) in [
            (0, -1),
            (1_250_000, -1),
            (2_500_000, 0),
            (3_750_000, 1),
            (4_999_999, 1),
        ] {
            let value = scenario.value("AIRSPEED INDICATED", "knots", time).unwrap();
            assert!(value.is_finite());
            assert_eq!(
                if value < 0.0 {
                    -1
                } else if value > 0.0 {
                    1
                } else {
                    0
                },
                expected_sign
            );
        }
        let angular = format!(
            "{}\n{}",
            header(),
            r#"{"type":"signal","id":"pitch","simvar":"PLANE PITCH DEGREES","unit":"radians","repeat":false}
{"type":"hold","signal":"pitch","value":1e308,"duration_ms":1000}"#
        );
        let error = Scenario::parse(&angular, "angular.jsonl").err().unwrap();
        assert!(
            error.contains("angular.jsonl: line 3")
                && error.contains("pitch")
                && error.contains("degrees")
        );
    }

    #[test]
    fn missing_segment_identifies_declaration_line_and_signal_id() {
        let input = format!(
            "{}\n{}",
            header(),
            r#"{"type":"signal","id":"missing","simvar":"AIRSPEED INDICATED","unit":"knots","repeat":false}"#
        );
        let error = Scenario::parse(&input, "missing.jsonl").err().unwrap();
        assert!(
            error.contains("missing.jsonl: line 2")
                && error.contains("signal id")
                && error.contains("missing")
        );
    }
    #[test]
    fn embedded_default_is_valid() {
        Scenario::parse(
            include_str!("../../../demo-scenarios/demo-scenario.jsonl"),
            "default",
        )
        .unwrap();
    }

    #[test]
    fn default_matches_independent_native_checkpoints() {
        let s = Scenario::parse(
            include_str!("../../../demo-scenarios/demo-scenario.jsonl"),
            "default",
        )
        .unwrap();
        for (name, unit, time, expected) in [
            ("AIRSPEED INDICATED", "knots", 5_000_000, 75.0),
            ("AIRSPEED TRUE", "knots", 8_000_000, 160.0),
            ("GROUND VELOCITY", "knots", 10_000_000, 170.0),
            ("INDICATED ALTITUDE:1", "feet", 2_950_000, 120.0),
            ("INDICATED ALTITUDE:1", "feet", 3_000_000, 980.0),
            ("INDICATED ALTITUDE:1", "feet", 18_000_000, 8500.0),
            ("AUTOPILOT ALTITUDE LOCK VAR:0", "feet", 18_000_000, 10000.0),
            ("AUTOPILOT ALTITUDE LOCK VAR:0", "feet", 108_000_000, 1000.0),
            (
                "PLANE HEADING DEGREES MAGNETIC",
                "degrees",
                7_000_000,
                351.0,
            ),
            ("KOHLSMAN SETTING HG:1", "inHg", 6_000_000, 30.12),
            ("TURN COORDINATOR BALL", "position 128", 10_000_000, 127.0),
        ] {
            let actual = s.value(name, unit, time).unwrap();
            assert!(
                (actual - expected).abs() < 1e-8,
                "{name} at {time}: {actual}"
            );
        }
        assert_eq!(
            s.value("AUTOPILOT ALTITUDE LOCK VAR:0", "feet", 17_000_000),
            None
        );
        assert_eq!(s.value("VERTICAL SPEED", "feet per minute", 0), None);
        assert_eq!(
            s.value("AIRSPEED INDICATED", "knots", 198_000_000),
            Some(0.0)
        );
        assert!(
            (s.value("PLANE PITCH DEGREES", "radians", 2_000_000)
                .unwrap()
                + std::f64::consts::PI / 12.0)
                .abs()
                < 1e-8
        );
    }
}
