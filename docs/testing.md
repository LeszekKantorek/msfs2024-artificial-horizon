# Test and acceptance strategy

This document defines verification procedures and acceptance criteria. Record
execution results and acceptance progress in the related GitHub issue or PR.

## HTTP startup checks

Use Windows x64 MSVC with the toolchain in `rust-toolchain.toml`. Run the check
commands in README, including default and `simconnect` feature tests. Dependencies
are resolved from committed Cargo.lock; no simulator or SimConnect SDK is required.

Run `cargo run --locked --bin main`, then request `http://127.0.0.1:8080/`,
`/styles.css`, and `/health`. Expect HTML, CSS, and JSON `{"status":"ok"}`.
Check `--help`, `--version`, and rejected arguments. A page without telemetry
must indicate unavailable data rather than show a valid-looking attitude.

Press Ctrl+C, confirm successful exit, then start again on the same port.
Library tests independently signal shutdown and rebind the socket with a five-second
completion timeout. Public configuration rejects port zero; internal listener
tests use OS-assigned ports to avoid collisions.

For LAN, explicitly bind the private interface or `0.0.0.0` and verify the printed
interface URL. Desktop checks do not establish real-phone or simulator
compatibility; validate those with the manual acceptance matrix below.

## Automated checks

Run formatting, Clippy with warnings denied, and locked dependency tests on Windows
x64 MSVC only for the demo/default feature set. The SimConnect feature gets a Windows
build check once its SDK prerequisites and distribution constraints are known.

Test behavior at these boundaries:

- Library/CLI: typed configuration works without parsing process arguments; clap
  help/version succeed and invalid CLI values produce actionable errors.
- Model: unit/sign normalization, roll wrap, invalid numbers, serialization, and
  monotonic sample age. Use fixtures independent of the implementation formula.
- Providers: deterministic trajectories, unchanged-but-fresh samples, lifecycle
  transitions, reconnect backoff, and shutdown/handle ownership.
- HTTP/SSE: `/health` reports HTTP liveness independently of source readiness;
  immediate full snapshot, event framing, MIME/cache headers, reconnect
  without replay, shared acquisition, two clients, slow consumers, and cleanup.
- Browser: pure attitude transforms, message validation, source/transport status,
  receive timeout, and fresh-data requirement after background/resume.

Use known expected geometry: nose up lowers the horizon; positive right bank
rotates the world counterclockwise around the fixed aircraft reference. Include
combined pitch/bank fixtures to catch transform-order errors.

## Manual acceptance matrix

| Scenario | Expected result |
| --- | --- |
| Level / +/- pitch / +/- bank / combined | Instrument agrees with known demo pose and simulator cockpit |
| Portrait / landscape / 320 CSS px | Scales and status remain readable without horizontal scrolling |
| Simulator not running / flight not loaded | Disconnected or waiting; no fabricated live attitude |
| Pause / menu / flight exit / simulator restart | Explicit state followed by automatic recovery on fresh data |
| Network loss / Wi-Fi recovery | Reconnecting/stale, then current state without manual reload |
| HTTP server restart | Browser reconnects; sequence restart is accepted |
| Background tab / phone lock / resume | Fresh sample required before returning to live |
| Two phones / deliberately slow receiver | Independent delivery and bounded memory |
| Source stops sampling but SSE stays open | Stale within one second on active page; heartbeat cannot mask it |
| Invalid values | Visible invalid state; no level-flight fallback |

Run mobile acceptance on real iOS Safari and Android Chrome. Desktop emulation
helps development but does not establish mobile lifecycle compatibility.

## Measurements and evidence

Record OS, CPU, phone models, browser versions, simulator/SDK/aircraft versions,
build commit, source rate, network conditions, duration, and observed failures.
Run two phones for at least 30 minutes and record memory/subscriber trends.

Measure browser receipt-to-render using its monotonic clock and frame callbacks.
For total latency, use either an instrumented source with measured clock offset
and uncertainty or a synchronized/high-frame-rate recording of source and phone.
Include method uncertainty and p50/p95 results; do not subtract unrelated device
timestamps. The proposed p95 <=150 ms target covers source acquisition through
visible rendering on a healthy LAN, not receipt-to-render alone.

Attach results to the validation issue. Keep tasks requiring hardware open until
observed; unit tests cannot certify actual simulator or phone behavior.
