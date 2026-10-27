//! Optional diagnostics, independent of acquisition and telemetry distribution.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, FILE_SHARE_READ, GetFileInformationByHandle,
};

/// Final diagnostic outcome, independent of the integration's native result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticStatus {
    Disabled,
    Completed { path: PathBuf },
    Failed { path: PathBuf, error: String },
}

pub(super) struct Journal {
    active: Option<ActiveJournal>,
    status: DiagnosticStatus,
}

struct ActiveJournal {
    writer: BufWriter<File>,
    started: Instant,
    last_flush: Instant,
    records: u64,
    session_id: String,
}

impl Journal {
    pub(super) fn open(path: Option<&Path>, protected_paths: &[&Path]) -> io::Result<Self> {
        let Some(path) = path else {
            return Ok(Self {
                active: None,
                status: DiagnosticStatus::Disabled,
            });
        };
        // A missing DLL must not become a diagnostic log.
        let resolved = resolve_path(path)?;
        for protected in protected_paths {
            if resolved
                .to_string_lossy()
                .eq_ignore_ascii_case(&resolve_path(protected)?.to_string_lossy())
            {
                return Err(protected_path_error(path));
            }
        }
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        // Deny write/delete sharing across processes and aliases. Allow readers.
        let mut file = OpenOptions::new().create(true).append(true).read(true)
            .share_mode(FILE_SHARE_READ).open(path).map_err(|error| io::Error::new(error.kind(),
                format!("Cannot open diagnostic log {} (another writer may already be using it): {error}", path.display())))?;
        let identity = file_identity(&file)?;
        for protected in protected_paths {
            match File::open(protected) {
                Ok(protected_file) if file_identity(&protected_file)? == identity => {
                    return Err(protected_path_error(path));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        // Preserve an interrupted record byte for byte, then start on a new line.
        if file.metadata()?.len() != 0 {
            file.seek(SeekFrom::End(-1))?;
            let mut last = [0];
            file.read_exact(&mut last)?;
            if last[0] != b'\n' {
                file.write_all(b"\n")?;
            }
        }
        Self::start(file, path.to_owned())
    }

    fn start(file: File, path: PathBuf) -> io::Result<Self> {
        static NEXT_SESSION: AtomicU64 = AtomicU64::new(0);
        let started_unix_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let session_id = format!(
            "{started_unix_ns}-{}-{}",
            std::process::id(),
            NEXT_SESSION.fetch_add(1, Ordering::Relaxed)
        );
        let mut active = ActiveJournal {
            writer: BufWriter::new(file),
            started: Instant::now(),
            last_flush: Instant::now(),
            records: 0,
            session_id,
        };
        if let Err(error) = active.record("session_start", json!({
            "schema_version": 4, "started_unix_ms": (started_unix_ns / 1_000_000).to_string(),
            "clock": "UTC Unix epoch milliseconds", "process_id": std::process::id(),
            "live_publication_enabled": true,
            "validation_scope": "recorded MSFS2024 sessions; see docs/simconnect.md#live-publication-evidence",
        }), true) {
            // Dropping BufWriter retries writes; discard the failed buffer instead.
            let _ = active.writer.into_parts();
            return Err(error);
        }
        Ok(Self {
            active: Some(active),
            status: DiagnosticStatus::Completed { path },
        })
    }

    pub(super) fn enabled(&self) -> bool {
        self.active.is_some()
    }

    /// The closure is never evaluated after logging is disabled or has failed.
    pub(super) fn record(&mut self, kind: &str, data: impl FnOnce() -> Value, flush: bool) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        if let Err(error) = active.record(kind, data(), flush) {
            self.fail(error);
        }
    }

    pub(super) fn flush(&mut self) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        if let Err(error) = active.flush() {
            self.fail(error);
        }
    }

    /// Flush and release the writer before returning the final status to the caller.
    pub(super) fn finish(&mut self) {
        self.flush();
        if let Some(active) = self.active.take() {
            let _ = active.writer.into_parts();
        }
    }

    pub(super) fn status(&self) -> DiagnosticStatus {
        self.status.clone()
    }

    fn fail(&mut self, error: io::Error) {
        let DiagnosticStatus::Completed { path } = &self.status else {
            return;
        };
        eprintln!(
            "Diagnostic logging failed for {}: {error}. SimConnect acquisition continues without diagnostics; this measurement is incomplete.",
            path.display()
        );
        self.status = DiagnosticStatus::Failed {
            path: path.clone(),
            error: error.to_string(),
        };
        if let Some(active) = self.active.take() {
            let _ = active.writer.into_parts();
        }
    }

    #[cfg(test)]
    pub(super) fn failing_for_test() -> Self {
        Self::with_failing_writer(BufWriter::with_capacity(
            1,
            File::open("Cargo.toml").unwrap(),
        ))
    }

    #[cfg(test)]
    fn with_failing_writer(writer: BufWriter<File>) -> Self {
        Self {
            active: Some(ActiveJournal {
                writer,
                started: Instant::now(),
                last_flush: Instant::now(),
                records: 0,
                session_id: "injected-failure".into(),
            }),
            status: DiagnosticStatus::Completed {
                path: PathBuf::from("injected-failure.jsonl"),
            },
        }
    }
}

impl ActiveJournal {
    fn record(&mut self, kind: &str, data: Value, flush: bool) -> io::Result<()> {
        let record = json!({ "session_id": self.session_id, "record": self.records,
            "elapsed_us": self.started.elapsed().as_micros().to_string(), "kind": kind, "data": data });
        serde_json::to_writer(&mut self.writer, &record)?;
        self.writer.write_all(b"\n")?;
        self.records += 1;
        if flush || self.last_flush.elapsed() >= Duration::from_secs(1) {
            self.flush()?;
        }
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.last_flush = Instant::now();
        Ok(())
    }
}

fn protected_path_error(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "Diagnostic log {} must not refer to the selected DLL or running executable",
            path.display()
        ),
    )
}

// Resolve existing ancestors too, so missing files through directory aliases are
// compared before creation. The handle check below also covers hardlinks.
fn resolve_path(path: &Path) -> io::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    match fs::canonicalize(&absolute) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = absolute.parent().ok_or(error)?;
            let name = absolute.file_name().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Diagnostic path must name a file",
                )
            })?;
            Ok(resolve_path(parent)?.join(name))
        }
        Err(error) => Err(error),
    }
}

fn file_identity(file: &File) -> io::Result<(u32, u32, u32)> {
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: File owns this live handle; information is an aligned, writable
    // output structure and no borrowed native data escapes the call.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((
        information.dwVolumeSerialNumber,
        information.nFileIndexHigh,
        information.nFileIndexLow,
    ))
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
    struct TemporaryDirectory(PathBuf);
    impl TemporaryDirectory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "simconnect-journal-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }
    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
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
    fn disabled_logging_does_not_evaluate_records() {
        let mut journal = Journal::open(None, &[Path::new("invalid protected path")]).unwrap();
        journal.record("never", || panic!("disabled metadata was evaluated"), true);
        journal.finish();
        assert!(!journal.enabled());
        assert_eq!(journal.status(), DiagnosticStatus::Disabled);
    }
    #[test]
    fn sessions_append_with_distinct_ids_and_independent_counters() {
        let directory = TemporaryDirectory::new();
        let path = directory.path("nested/exact.jsonl");
        let mut first = Journal::open(Some(&path), &[]).unwrap();
        first.record("sample", || number(1.0), true);
        first.finish();
        let original = fs::read(&path).unwrap();
        let mut second = Journal::open(Some(&path), &[]).unwrap();
        second.record("session_end", || json!({"outcome":"completed"}), true);
        second.finish();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.as_bytes().starts_with(&original));
        let records: Vec<Value> = content
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 4);
        assert_ne!(records[0]["session_id"], records[2]["session_id"]);
        for pair in records.as_chunks::<2>().0 {
            assert_eq!(pair[0]["session_id"], pair[1]["session_id"]);
            assert_eq!(pair[0]["record"], 0);
            assert_eq!(pair[1]["record"], 1);
            assert!(
                pair[0]["elapsed_us"]
                    .as_str()
                    .unwrap()
                    .parse::<u128>()
                    .unwrap()
                    <= pair[1]["elapsed_us"]
                        .as_str()
                        .unwrap()
                        .parse::<u128>()
                        .unwrap()
            );
            assert!(pair[0]["data"].get("scenario").is_none());
        }
        assert_eq!(second.status(), DiagnosticStatus::Completed { path });
    }
    #[test]
    fn interrupted_tail_is_preserved_and_separated() {
        let directory = TemporaryDirectory::new();
        let path = directory.path("interrupted.jsonl");
        let partial = b"{\"old_partial\":";
        fs::write(&path, partial).unwrap();
        let mut journal = Journal::open(Some(&path), &[]).unwrap();
        journal.finish();
        let content = fs::read(&path).unwrap();
        assert_eq!(&content[..partial.len()], partial);
        assert_eq!(content[partial.len()], b'\n');
        let line = std::str::from_utf8(&content[partial.len() + 1..]).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(line).unwrap()["kind"],
            "session_start"
        );
    }
    #[test]
    fn writer_lock_rejects_processes_and_aliases_while_readers_work() {
        let directory = TemporaryDirectory::new();
        let path = directory.path("locked.jsonl");
        let alias = directory.path("hardlink.jsonl");
        let mut journal = Journal::open(Some(&path), &[]).unwrap();
        fs::hard_link(&path, &alias).unwrap();
        assert!(Journal::open(Some(&alias), &[]).is_err());
        assert!(fs::read_to_string(&path).unwrap().contains("session_start"));
        let child_name = format!(
            "{}::writer_lock_child",
            module_path!().split_once("::").unwrap().1
        );
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &child_name])
            .env("SIMCONNECT_JOURNAL_LOCK_CHILD_PATH", &path)
            .status()
            .unwrap();
        assert!(status.success());
        journal.finish();
        let mut reopened = Journal::open(Some(&alias), &[]).unwrap();
        reopened.finish();
    }
    #[test]
    fn writer_lock_child() {
        let Some(path) = std::env::var_os("SIMCONNECT_JOURNAL_LOCK_CHILD_PATH") else {
            return;
        };
        assert!(Journal::open(Some(Path::new(&path)), &[]).is_err());
    }
    #[test]
    fn protected_names_and_hardlinks_are_rejected_without_appending() {
        let directory = TemporaryDirectory::new();
        for name in ["SimConnect.dll", "application.exe"] {
            let protected = directory.path(name);
            fs::write(&protected, b"protected bytes").unwrap();
            let alias = directory.path(&format!("{name}.jsonl"));
            fs::hard_link(&protected, &alias).unwrap();
            assert!(Journal::open(Some(&protected), &[&protected]).is_err());
            assert!(Journal::open(Some(&directory.0.join(".").join(name)), &[&protected]).is_err());
            assert!(
                Journal::open(Some(&directory.path(&name.to_uppercase())), &[&protected]).is_err()
            );
            assert!(Journal::open(Some(&alias), &[&protected]).is_err());
            assert_eq!(fs::read(&protected).unwrap(), b"protected bytes");
        }
        let missing = directory.path("missing/SimConnect.dll");
        assert!(Journal::open(Some(&missing), &[&missing]).is_err());
        assert!(!missing.exists());
    }
    #[test]
    fn initial_open_and_initial_write_failures_are_fatal() {
        let directory = TemporaryDirectory::new();
        assert!(Journal::open(Some(&directory.0), &[]).is_err());
        assert!(
            Journal::start(
                File::open("Cargo.toml").unwrap(),
                directory.path("injected.jsonl")
            )
            .is_err()
        );
    }
    #[test]
    fn later_write_failure_disables_logging_and_preserves_failure() {
        let mut journal = Journal::failing_for_test();
        journal.record("failure", || json!({}), false);
        assert!(!journal.enabled());
        let failed = journal.status();
        assert!(matches!(failed, DiagnosticStatus::Failed { .. }));
        journal.record("later", || panic!("failed logger evaluated a record"), true);
        journal.flush();
        journal.finish();
        assert_eq!(journal.status(), failed);
    }
    #[test]
    fn later_flush_failure_disables_logging_and_preserves_failure() {
        let mut journal =
            Journal::with_failing_writer(BufWriter::new(File::open("Cargo.toml").unwrap()));
        journal.record("buffered", || json!({}), false);
        assert!(journal.enabled());
        journal.finish();
        assert!(!journal.enabled());
        assert!(matches!(journal.status(), DiagnosticStatus::Failed { .. }));
    }
}
