#!/usr/bin/env python3
"""Register a session checkpoint without opening its transcript."""

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


REGISTER_EVENTS = {"Stop", "Interrupt", "PreCompact", "SessionEnd"}


def register(directory, session_id, transcript_path):
    directory.mkdir(parents=True, exist_ok=True)
    with queue_lock(directory):
        sessions = read_sessions(directory)
        previous = sessions.get(session_id)
        now = datetime.datetime.now(datetime.timezone.utc)
        if previous:
            path, row = previous
            # Keep distinct checkpoints if the clock repeats or moves back.
            last = datetime.datetime.fromisoformat(row["updated_at"].replace("Z", "+00:00"))
            now = max(now, last + datetime.timedelta(microseconds=1))
        else:
            number = len(sessions) + 1
            path = directory / ("session-%06d.json" % number)
            while path.exists():
                number += 1
                path = directory / ("session-%06d.json" % number)
        write_record(path, {
            "session_id": session_id,
            "updated_at": now.isoformat(),
            "transcript_path": transcript_path,
            "reviewed_at": None,
        })


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
    if isinstance(transcript, str) and transcript:
        transcript = str((Path(cwd) / transcript).resolve())
    else:
        transcript = None
    try:
        register(Path(result.stdout.strip()) / ".context" / "sessions", session_id, transcript)
    except (OSError, ValueError, TypeError) as exc:
        print("context-skills: could not register session: %s" % exc, file=sys.stderr)
        return 1
    if event["hook_event_name"] in {"Stop", "PreCompact"}:
        print("{}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
