//! Session-local diagnostic records, independent of telemetry distribution.
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub(super) struct Journal {
    writer: BufWriter<File>,
    path: PathBuf,
    started: Instant,
    last_flush: Instant,
    records: u64,
}

impl Journal {
    pub(super) fn create(directory: &Path, scenario: &str) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        let directory = directory.canonicalize()?;
        let started_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();
        let mut suffix = 0_u32;
        let (path, file) = loop {
            let path = directory.join(format!(
                "session-{started_unix_ms}-{}-{suffix}.jsonl",
                std::process::id()
            ));
            match OpenOptions::new().create_new(true).write(true).open(&path) {
                Ok(file) => break (path, file),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    suffix = suffix
                        .checked_add(1)
                        .ok_or_else(|| io::Error::other("Session filenames exhausted"))?;
                }
                Err(error) => return Err(error),
            }
        };
        let mut journal = Self {
            writer: BufWriter::new(file),
            path,
            started: Instant::now(),
            last_flush: Instant::now(),
            records: 0,
        };
        journal.record("session_start", json!({
            "schema_version": 1, "scenario": scenario,
            "started_unix_ms": started_unix_ms.to_string(), "clock": "UTC Unix epoch milliseconds",
            "process_id": std::process::id(), "diagnostic_only": true,
            "availability_mapping_verified": false, "attitude_signs_verified": false,
        }), true)?;
        Ok(journal)
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn record(
        &mut self,
        kind: &str,
        data: impl Serialize,
        flush: bool,
    ) -> io::Result<()> {
        let record = json!({ "record": self.records,
            "elapsed_us": self.started.elapsed().as_micros().to_string(), "kind": kind, "data": data });
        serde_json::to_writer(&mut self.writer, &record)?;
        self.writer.write_all(b"\n")?;
        self.records += 1;
        if flush || self.last_flush.elapsed() >= Duration::from_secs(1) {
            self.flush()?;
        }
        Ok(())
    }

    pub(super) fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.last_flush = Instant::now();
        Ok(())
    }
}

pub(super) fn fingerprint(path: &Path) -> io::Result<Value> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(json!({ "path": path, "size_bytes": metadata.len(),
        "modified_unix_ms": metadata.modified().ok().and_then(|at| at.duration_since(UNIX_EPOCH).ok()).map(|at| at.as_millis().to_string()),
        "sha256": format!("{:x}", hash.finalize()) }))
}

pub(super) fn number(value: f64) -> Value {
    json!({ "value": if value.is_finite() { Some(value) } else { None },
        "classification": if value.is_nan() { "nan" } else if value == f64::INFINITY { "positive_infinity" }
            else if value == f64::NEG_INFINITY { "negative_infinity" } else { "finite" },
        "bits_hex": format!("{:016x}", value.to_bits()) })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nonfinite_values_retain_bits_and_classification_in_valid_json() {
        for (value, class) in [
            (f64::NAN, "nan"),
            (f64::INFINITY, "positive_infinity"),
            (f64::NEG_INFINITY, "negative_infinity"),
        ] {
            let encoded = serde_json::to_string(&number(value)).unwrap();
            let decoded: Value = serde_json::from_str(&encoded).unwrap();
            assert!(decoded["value"].is_null());
            assert_eq!(decoded["classification"], class);
            assert_eq!(decoded["bits_hex"], format!("{:016x}", value.to_bits()));
        }
        assert_eq!(number(-0.0)["bits_hex"], "8000000000000000");
    }
    #[test]
    fn sessions_never_overwrite_and_records_are_json_lines() {
        let directory = std::env::temp_dir().join(format!(
            "simconnect-journal-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut first = Journal::create(&directory, "level").unwrap();
        let first_path = first.path().to_owned();
        let second = Journal::create(&directory, "level").unwrap();
        assert_ne!(first_path, second.path());
        first.record("sample", number(1.0), true).unwrap();
        let content = fs::read_to_string(&first_path).unwrap();
        let records: Vec<Value> = content
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["data"]["scenario"], "level");
        assert_eq!(records[1]["record"], 1);
        drop(first);
        drop(second);
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn write_and_flush_failures_are_returned() {
        let read_only = File::open("Cargo.toml").unwrap();
        let mut journal = Journal {
            writer: BufWriter::new(read_only),
            path: PathBuf::new(),
            started: Instant::now(),
            last_flush: Instant::now(),
            records: 0,
        };
        assert!(journal.record("error", json!({}), true).is_err());
    }
}
