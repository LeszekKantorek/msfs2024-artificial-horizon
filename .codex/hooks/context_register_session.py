#!/usr/bin/env python3
"""Register a session checkpoint without opening its transcript."""

import csv
import datetime
import json
import os
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path

FIELDS = ("session_date", "session_id", "path", "reviewed_date")


@contextmanager
def queue_lock(directory):
    """Shared protocol for the independently installed hook and queue reader."""
    with (directory / ".sessions.lock").open("a+b") as handle:
        deadline = time.monotonic() + 1
        while True:
            try:
                if os.name == "nt":
                    import msvcrt
                    handle.seek(0)
                    msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
                else:
                    import fcntl
                    fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except OSError:
                if time.monotonic() >= deadline:
                    raise TimeoutError("session queue is busy; retry later")
                time.sleep(0.02)
        try:
            yield
        finally:
            if os.name == "nt":
                handle.seek(0)
                msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(handle, fcntl.LOCK_UN)


def validate_row(row, source):
    if (set(row) != set(FIELDS)
            or any(not isinstance(value, str) for value in row.values())
            or not row["session_id"].strip() or not row["path"].strip()):
        raise ValueError("invalid session record: " + str(source))
    for field in ("session_date", "reviewed_date"):
        value = row[field]
        if field == "reviewed_date" and not value:
            continue
        try:
            if datetime.date.fromisoformat(value).isoformat() != value:
                raise ValueError()
        except ValueError:
            raise ValueError("invalid session date in " + str(source)) from None


def read_sessions(directory):
    """Read one snapshot of the current CSV log."""
    sessions = {}
    path = directory / "sessions.log"
    if not path.exists():
        return sessions
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.reader(handle, strict=True)
        if next(reader, None) != list(FIELDS):
            raise ValueError("invalid session log header: " + str(path))
        for values in reader:
            if len(values) != len(FIELDS):
                raise ValueError("invalid session record: " + str(path))
            row = dict(zip(FIELDS, values))
            validate_row(row, path)
            if row["session_id"] in sessions:
                raise ValueError("duplicate session_id in session log: " + row["session_id"])
            sessions[row["session_id"]] = row
    return sessions


def write_sessions(path, sessions):
    """Write the log atomically, retrying one failed write under the caller's lock."""
    for row in sessions.values():
        validate_row(row, path)
    for attempt in range(2):
        try:
            write_sessions_once(path, sessions)
            return
        except OSError:
            if attempt == 1:
                raise
            time.sleep(0.05)


def write_sessions_once(path, sessions):
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", newline="", dir=path.parent,
                                         suffix=".tmp", delete=False) as handle:
            temporary = Path(handle.name)
            writer = csv.writer(handle, lineterminator="\n")
            writer.writerow(FIELDS)
            for row in sessions.values():
                writer.writerow([row[field] for field in FIELDS])
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


REGISTER_EVENTS = {"Stop", "Interrupt", "PreCompact", "SessionEnd"}


def register(directory, session_id, transcript_path):
    if not isinstance(transcript_path, str) or not transcript_path.strip():
        return
    directory.mkdir(parents=True, exist_ok=True)
    with queue_lock(directory):
        sessions = read_sessions(directory)
        previous = sessions.get(session_id)
        sessions[session_id] = {
            "session_date": previous["session_date"] if previous else datetime.date.today().isoformat(),
            "session_id": session_id,
            "path": transcript_path,
            "reviewed_date": previous["reviewed_date"] if previous else "",
        }
        write_sessions(directory / "sessions.log", sessions)


def main():
    try:
        event = json.load(sys.stdin)
    except (ValueError, OSError):
        return 0
    if not isinstance(event, dict) or event.get("hook_event_name") not in REGISTER_EVENTS:
        return 0
    session_id, cwd = event.get("session_id"), event.get("cwd")
    if not isinstance(session_id, str) or not session_id or not isinstance(cwd, str) or not cwd:
        return 0
    try:
        result = subprocess.run(
            ["git", "-C", cwd, "rev-parse", "--show-toplevel"], check=True,
            capture_output=True, text=True, timeout=1,
        )
    except (OSError, subprocess.SubprocessError):
        return 0
    transcript = event.get("transcript_path")
    if isinstance(transcript, str) and transcript.strip():
        transcript = str((Path(cwd) / transcript).resolve())
    else:
        transcript = None
    try:
        register(Path(result.stdout.strip()) / ".context", session_id, transcript)
    except (OSError, ValueError, TypeError, csv.Error) as exc:
        print("context-skills: could not register session: %s" % exc, file=sys.stderr)
        return 1
    if event["hook_event_name"] in {"Stop", "PreCompact"}:
        print("{}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
