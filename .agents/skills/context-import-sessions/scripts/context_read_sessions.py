#!/usr/bin/env python3
"""Read session checkpoint records or mark a checkpoint processed by context-gather."""

import argparse
import datetime
import json
import os
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path

FIELDS = {"session_id", "updated_at", "transcript_path", "reviewed_at"}


@contextmanager
def queue_lock(directory):
    """Shared protocol for the independently installed hook and queue reader."""
    with (directory / ".queue.lock").open("a+b") as handle:
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


def read_sessions(directory):
    sessions = {}
    for path in sorted(directory.glob("*.json")):
        row = json.loads(path.read_text(encoding="utf-8"))
        if (not isinstance(row, dict) or set(row) != FIELDS
                or not isinstance(row["session_id"], str) or not row["session_id"]
                or not isinstance(row["updated_at"], str)
                or not isinstance(row["transcript_path"], (str, type(None)))
                or not isinstance(row["reviewed_at"], (str, type(None)))):
            raise ValueError("invalid session record: " + str(path))
        if row["session_id"] in sessions:
            raise ValueError("duplicate session_id in queue: " + str(path))
        sessions[row["session_id"]] = (path, row)
    return sessions


def write_record(path, row):
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                         suffix=".tmp", delete=False) as handle:
            temporary = Path(handle.name)
            json.dump(row, handle, ensure_ascii=False, indent=2)
            handle.write("\n")
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)



def inventory(directory, session_id=None, expected_update=None):
    if not directory.exists():
        if session_id:
            raise ValueError("session is not in the queue")
        return {}
    with queue_lock(directory):
        sessions = read_sessions(directory)
        if session_id:
            if expected_update is None:
                raise ValueError("--updated-at is required when marking a review")
            if session_id not in sessions:
                raise ValueError("session is not in the queue")
            path, row = sessions[session_id]
            if row["updated_at"] != expected_update:
                raise ValueError("session checkpoint changed; review its new evidence first")
            if row["reviewed_at"] is None:
                row["reviewed_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
                write_record(path, row)
        return sessions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project_root", type=Path)
    parser.add_argument("--mark-reviewed", metavar="SESSION_ID")
    parser.add_argument("--updated-at", help="Exact updated_at of the checkpoint actually reviewed")
    args = parser.parse_args()
    if bool(args.mark_reviewed) != (args.updated_at is not None):
        parser.error("--mark-reviewed and --updated-at must be supplied together")
    try:
        result = subprocess.run(
            ["git", "-C", str(args.project_root.resolve()), "rev-parse", "--show-toplevel"],
            check=True, capture_output=True, text=True, timeout=1,
        )
        directory = Path(result.stdout.strip()) / ".context" / "sessions"
        sessions = inventory(directory, args.mark_reviewed, args.updated_at)
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        print("context-skills: could not inspect queue: %s" % exc, file=sys.stderr)
        return 2
    pending = [(path, row) for path, row in sessions.values() if row["reviewed_at"] is None]
    print("queued: %d, reviewed: %d, pending: %d" % (
        len(sessions), len(sessions) - len(pending), len(pending)))
    for path, row in pending:
        transcript = row["transcript_path"]
        available = bool(transcript) and Path(transcript).is_file()
        print("%s  %s  transcript=%s  record=%s" % (
            row["session_id"], row["updated_at"],
            "available" if available else "missing", path,
        ))
    return 0


if __name__ == "__main__":
    sys.exit(main())
