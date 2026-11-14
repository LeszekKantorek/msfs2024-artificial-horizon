//! Validated, ordered SimConnect definition and numeric payload layout.
use serde::Deserialize;
use std::{collections::HashSet, ffi::CString, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub(super) enum Field {
    Pitch,
    Roll,
    IndicatedAirspeed,
    TrueAirspeed,
    GroundSpeed,
    BarometricAltitude,
    BaroSetting,
    VerticalSpeed,
    Heading,
    SlipSkid,
    TurnRate,
    Vs0,
    Vs1,
    RotationSpeed,
    MaximumAirspeed,
    AutopilotAvailable,
    SelectedHeading,
    SelectedAltitude,
    AutopilotAltRadioMode,
}

const FIELDS: [(Field, &str, &str); 19] = [
    (Field::Pitch, "pitch", "radians"),
    (Field::Roll, "roll", "radians"),
    (Field::IndicatedAirspeed, "indicated_airspeed", "knots"),
    (Field::TrueAirspeed, "true_airspeed", "knots"),
    (Field::GroundSpeed, "ground_speed", "knots"),
    (Field::BarometricAltitude, "barometric_altitude", "feet"),
    (Field::BaroSetting, "baro_setting", "inHg"),
    (Field::VerticalSpeed, "vertical_speed", "feet per minute"),
    (Field::Heading, "heading", "degrees"),
    (Field::SlipSkid, "slip_skid", "position 128"),
    (Field::TurnRate, "turn_rate", "degrees per second"),
    (Field::Vs0, "vs0", "knots"),
    (Field::Vs1, "vs1", "knots"),
    (Field::RotationSpeed, "rotation_speed", "knots"),
    (Field::MaximumAirspeed, "maximum_airspeed", "knots"),
    (Field::AutopilotAvailable, "autopilot_available", "bool"),
    (Field::SelectedHeading, "selected_heading", "degrees"),
    (Field::SelectedAltitude, "selected_altitude", "feet"),
    (
        Field::AutopilotAltRadioMode,
        "autopilot_alt_radio_mode",
        "bool",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NumericType {
    Int32,
    Int64,
    Float32,
    Float64,
}
impl NumericType {
    pub fn sdk(self) -> i32 {
        match self {
            Self::Int32 => 1,
            Self::Int64 => 2,
            Self::Float32 => 3,
            Self::Float64 => 4,
        }
    }
    pub fn size(self) -> usize {
        match self {
            Self::Int32 | Self::Float32 => 4,
            Self::Int64 | Self::Float64 => 8,
        }
    }
    pub fn decode(self, bytes: &[u8]) -> Option<f64> {
        Some(match self {
            Self::Int32 => i32::from_le_bytes(bytes.try_into().ok()?) as f64,
            Self::Int64 => {
                let value = i64::from_le_bytes(bytes.try_into().ok()?);
                let converted = value as f64;
                if converted as i128 != i128::from(value) {
                    return None;
                }
                converted
            }
            Self::Float32 => f32::from_le_bytes(bytes.try_into().ok()?) as f64,
            Self::Float64 => f64::from_le_bytes(bytes.try_into().ok()?),
        })
    }
}

#[derive(Debug)]
pub(super) struct Mapping {
    pub field: Field,
    pub field_name: String,
    pub simvar: CString,
    pub unit: CString,
    pub datatype: NumericType,
    pub type_name: String,
}
#[derive(Debug)]
pub struct Definition {
    pub(super) mappings: Vec<Mapping>,
    pub(super) payload_size: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    mappings: Vec<serde_json::Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    field: String,
    simvar: String,
    unit: String,
    #[serde(rename = "type")]
    datatype: String,
}

impl Definition {
    pub fn decode(&self, payload: &[u8]) -> Result<[Option<f64>; 19], String> {
        if payload.len() != self.payload_size {
            return Err(format!(
                "payload has {} bytes; expected {}",
                payload.len(),
                self.payload_size
            ));
        }
        let mut values = [None; 19];
        let mut offset = 0;
        for mapping in &self.mappings {
            let end = offset + mapping.datatype.size();
            values[mapping.field as usize] = mapping.datatype.decode(&payload[offset..end]);
            offset = end;
        }
        Ok(values)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }
    pub(super) fn parse(text: &str) -> Result<Self, String> {
        let document: Document =
            serde_json::from_str(text).map_err(|e| format!("invalid mapping JSON: {e}"))?;
        let mut seen = HashSet::new();
        let mut mappings = Vec::new();
        let mut payload_size = 0;
        for (index, value) in document.mappings.into_iter().enumerate() {
            let entry: Entry = serde_json::from_value(value)
                .map_err(|error| format!("mapping entry {index}: {error}"))?;
            let err = |message: &str| format!("mapping entry {index} ({}): {message}", entry.field);
            let &(field, _, unit_name) = FIELDS
                .iter()
                .find(|(_, name, _)| *name == entry.field)
                .ok_or_else(|| err("unknown field"))?;
            if !seen.insert(field) {
                return Err(err("duplicate field"));
            }
            if entry.unit != unit_name {
                return Err(err("unit has no conversion for this field"));
            }
            let datatype = match entry.datatype.as_str() {
                "SIMCONNECT_DATATYPE_INT32" => NumericType::Int32,
                "SIMCONNECT_DATATYPE_INT64" => NumericType::Int64,
                "SIMCONNECT_DATATYPE_FLOAT32" => NumericType::Float32,
                "SIMCONNECT_DATATYPE_FLOAT64" => NumericType::Float64,
                _ => return Err(err("unsupported SDK datatype")),
            };
            if entry.simvar.is_empty() {
                return Err(err("empty SimVar name"));
            }
            let simvar =
                CString::new(entry.simvar.as_str()).map_err(|_| err("SimVar contains NUL"))?;
            let unit = CString::new(entry.unit.as_str()).map_err(|_| err("unit contains NUL"))?;
            payload_size += datatype.size();
            mappings.push(Mapping {
                field,
                field_name: entry.field,
                simvar,
                unit,
                datatype,
                type_name: entry.datatype,
            });
        }
        for (id, field, _) in FIELDS {
            if !seen.contains(&id) {
                return Err(format!("missing required mapping {field}"));
            }
        }
        Ok(Self {
            mappings,
            payload_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_and_validation() {
        let text = include_str!("../../../default.json");
        let def = Definition::parse(text).unwrap();
        assert_eq!(def.mappings.len(), 19);
        assert_eq!(def.payload_size, 152);
        assert!(
            Definition::parse(&text.replace("SIMCONNECT_DATATYPE_FLOAT64", "FLOAT64")).is_err()
        );
        assert!(Definition::parse(&text.replace("\"pitch\"", "\"roll\"")).is_err());
        assert!(Definition::parse(&text.replace("\"radians\"", "\"degrees\"")).is_err());
    }
    #[test]
    fn integer_conversion_is_exact() {
        assert_eq!(
            NumericType::Int64.decode(&(1_i64 << 53).to_le_bytes()),
            Some((1_u64 << 53) as f64)
        );
        assert_eq!(
            NumericType::Int64.decode(&((1_i64 << 53) + 1).to_le_bytes()),
            None
        );
    }
    #[test]
    fn invalid_entries_identify_their_location() {
        let source: serde_json::Value =
            serde_json::from_str(include_str!("../../../default.json")).unwrap();
        for property in ["field", "simvar", "unit", "type"] {
            let mut document = source.clone();
            document["mappings"][3]
                .as_object_mut()
                .unwrap()
                .remove(property);
            let error = Definition::parse(&document.to_string()).unwrap_err();
            assert!(error.contains("entry 3"), "{error}");
        }
        let mut unknown = source.clone();
        unknown["mappings"][1]["field"] = "unknown".into();
        assert!(
            Definition::parse(&unknown.to_string())
                .unwrap_err()
                .contains("entry 1")
        );
        let mut missing = source;
        missing["mappings"].as_array_mut().unwrap().remove(0);
        assert!(
            Definition::parse(&missing.to_string())
                .unwrap_err()
                .contains("missing required mapping pitch")
        );
        assert!(Definition::parse("{").is_err());
    }
    #[test]
    fn reordered_mixed_numeric_layout() {
        let mut document: serde_json::Value =
            serde_json::from_str(include_str!("../../../default.json")).unwrap();
        let entries = document["mappings"].as_array_mut().unwrap();
        entries.swap(0, 2);
        entries[0]["type"] = "SIMCONNECT_DATATYPE_INT32".into();
        entries[1]["type"] = "SIMCONNECT_DATATYPE_FLOAT32".into();
        entries[2]["type"] = "SIMCONNECT_DATATYPE_INT64".into();
        let definition = Definition::parse(&document.to_string()).unwrap();
        let mut payload = Vec::new();
        payload.extend_from_slice(&123_i32.to_le_bytes());
        payload.extend_from_slice(&1.5_f32.to_le_bytes());
        payload.extend_from_slice(&2_i64.to_le_bytes());
        for _ in 3..19 {
            payload.extend_from_slice(&0_f64.to_le_bytes());
        }
        let values = definition.decode(&payload).unwrap();
        assert_eq!(values[0], Some(2.0));
        assert_eq!(values[1], Some(1.5));
        assert_eq!(values[2], Some(123.0));
        assert!(definition.decode(&payload[..payload.len() - 1]).is_err());
    }
}
