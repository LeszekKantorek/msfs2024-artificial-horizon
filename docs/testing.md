# Test and acceptance strategy

The HTTP skeleton has configuration, CLI, HTTP response, binding, and shutdown
tests. Telemetry, provider lifecycle, SSE, and instrument checks below become
mandatory as their corresponding implementation is added.

## Skeleton acceptance

Use Windows x64 MSVC with the toolchain in `rust-toolchain.toml`. Run the check
commands in README, including default and `simconnect` feature tests. Dependencies
are resolved from committed Cargo.lock; no simulator or SimConnect SDK is required.

Run `cargo run --locked --bin main`, then request `http://127.0.0.1:8080/`,
`/styles.css`, and `/health`. Expect HTML, CSS, and JSON `{"status":"ok"}`.
The page must state that telemetry is unavailable. `/api/v1/events` returns 404
until issue #3 implements SSE. Check `--help`, `--version`, and rejected arguments.

Press Ctrl+C, confirm successful exit, then start again on the same port.
Library tests independently signal shutdown and rebind the socket with a five-second
completion timeout. Public configuration rejects port zero; internal listener
tests use OS-assigned ports to avoid collisions.

For LAN, explicitly bind the private interface or `0.0.0.0` and verify the printed
interface URL. A desktop check establishes only the placeholder's layout; real
phone, simulator, and telemetry acceptance still belongs to later issues.

### Local skeleton evidence (2026-10-06)

Windows x64 MSVC, Rust 1.98.1: formatting, warning-free Clippy, all ten tests
in both default and `simconnect` configurations, and explicit target build passed.
`cargo run --locked --bin main` served HTML/CSS and the expected health JSON.
An isolated hidden Windows console received the real `CTRL_C_EVENT`: two runs
exited with code 0 and `Server stopped.`, reusing the same listening port.
The automation terminal's inherited signal-ignore flag was disabled in the
validation launcher; it was not changed in the application.

Playwright with installed Microsoft Edge checked 1280x800 and 320x568 viewports;
both rendered the placeholder without horizontal overflow. Screenshots were
visually inspected. No real phone or MSFS compatibility is claimed by these checks.

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
