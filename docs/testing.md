# Test and acceptance strategy

> Procedures and criteria belong here. Record results and acceptance progress in the related GitHub issue or PR.

## HTTP startup checks

Prerequisites: Rust stable, rustfmt, and Clippy on Windows x64 MSVC.
Use committed `Cargo.lock`. These checks require no simulator or SDK.

1. Run the [automated checks](#automated-checks), including default and `simconnect` feature tests.
2. Record the compiler version with results. CI records it in its logs.

   ```powershell
   rustc --version --verbose
   ```

3. Start the server.

   ```powershell
   cargo run --locked --bin main
   ```

4. Check the endpoints and CLI behavior below.
5. Press `Ctrl+C`.
6. Check successful exit.
7. Restart on the same port.

| Check | Expected result |
| --- | --- |
| `http://127.0.0.1:8080/` | HTML |
| `http://127.0.0.1:8080/styles.css` | CSS |
| `http://127.0.0.1:8080/health` | JSON `{"status":"ok"}` |
| `--help`, `--version` | Successful output |
| Invalid arguments | Actionable errors |
| Page without telemetry | Unavailable indication, no credible attitude |
| Explicit LAN bind: private interface or `0.0.0.0` | Printed URL uses the correct interface |

Library tests signal shutdown independently and rebind within a five-second completion timeout.
Public configuration rejects port zero. Internal listener tests use OS-assigned ports to avoid collisions.

> Desktop checks do not establish real-phone or simulator compatibility.

## Automated checks

Run these commands from the repository root on Windows x64 MSVC.
They match the checks in [CI](../.github/workflows/ci.yml).

Complete the [environment setup](../CONTRIBUTING.md#environment-setup) first.
Use committed `Cargo.lock` with `--locked`.
Default checks and the current SDK-free `simconnect` feature require no simulator or SDK.

Run the default checks and build:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
node --test tests/*.test.mjs
cargo build --locked --target x86_64-pc-windows-msvc
```

Check the current SimConnect module boundary:

```powershell
cargo clippy --locked --all-targets --features simconnect -- -D warnings
cargo test --locked --all-targets --features simconnect
```

> The current `simconnect` feature contains no SDK integration. Passing these checks does not establish simulator compatibility.
> Future SDK-dependent checks require documented prerequisites and distribution constraints.

### Coverage boundaries

| Boundary | Required coverage |
| --- | --- |
| Library/CLI | Typed configuration without process parsing, help/version, actionable invalid-value errors |
| Model | Units/signs, roll wrap, invalid numbers, serialization, monotonic age, fixtures independent of implementation formulas |
| Providers | Deterministic trajectories, unchanged fresh samples, lifecycle transitions, reconnect backoff, shutdown/handle ownership |
| HTTP/SSE | Health independent of source, immediate full snapshot, framing, MIME/cache headers, reconnect without replay, shared acquisition, two clients, slow consumers, cleanup |
| Browser | Pure transforms, message validation, separate source/transport status, receive timeout, fresh data after background/resume |

Known geometry:

* Nose up lowers the horizon.
* Positive right bank rotates the world counterclockwise around the fixed aircraft reference.
* Combined pitch/bank fixtures detect transform-order errors.

## SSE and browser controller checks

```powershell
cargo test --locked --test http_sse
node --test tests/*.test.mjs
```

Node checks require Node.js 24, without npm install or a browser framework.
These checks run in Windows CI.

| Suite | Coverage / method |
| --- | --- |
| HTTP SSE | Framing/headers, immediate state, ignored `Last-Event-ID`, invalid/unavailable/stale states, delivery-time age, independent slow consumers, heartbeat comments, channel closure |
| HTTP timing | Real Axum bodies and a paused monotonic clock. Do not collect an infinite stream |
| Socket unit tests | Subscriber release and write/shutdown deadlines with bounded duplex transport |
| Local TCP | Two active SSE connections and port reuse after shutdown |
| Browser controller | Independent fixtures, one EventSource/retry timer, two-second retry, sequence reset, freshness, malformed messages, suspend/resume |
| Horizon | Pose signs, nested transforms, angle bounds, local-to-panel geometry |
| Frame scheduler | Latest-frame coalescing, expiry before draw, cancellation on data loss, resize without renewing age, fresh identical values |
| Panel/status | Complete read-only frame delegation, status without a sample, immediate stale obscuring, independent source/transport labels |
| Coordinated turn | Typed bounds, independent optional failures, old snapshots, shared sample age, +/-3 deg/s, resize without refreshing age |

### Desktop exploration

1. Start demo and open the page.
2. Check full-panel sky/ground, no source badge, and no top status bar.
3. Stop the server.
4. Check connection loss and retry indication.
5. Restart on the same port without reloading the page.
6. Check automatic recovery despite sequence restart.
7. Hide and restore the page.
8. Check that live attitude requires a new snapshot.
9. Inspect the network stream for named telemetry events and no duplicate subscriptions.

> Label these results as desktop evidence, not iOS/Android compatibility.

## Manual acceptance matrix

Run on real iOS Safari and Android Chrome.
Desktop emulation does not establish mobile lifecycle compatibility.

| Scenario | Expected result |
| --- | --- |
| Level / +/- pitch / +/- bank / combined | Agreement with known demo pose and simulator cockpit |
| Portrait / landscape / 320 CSS px | Readable scales/status without horizontal scrolling |
| Simulator absent / flight not loaded | Disconnected or waiting, no fabricated live attitude |
| Pause / menu / flight exit / simulator restart | Explicit state, then automatic recovery on fresh data |
| Network loss / Wi-Fi recovery | Reconnecting/stale, then current state without reload |
| HTTP server restart | Reconnect accepts sequence restart |
| Background / phone lock / resume | Fresh sample required before live |
| Two phones / deliberately slow receiver | Independent delivery, bounded memory |
| Source stops, SSE remains open | Stale within one second on active page. Heartbeats cannot mask it |
| Invalid values | Visible invalid state, no level-flight fallback |

## Attitude display: real-device phone acceptance

Use this checklist for each phone. Store completed results in issue #4.
Issue closure requires both real-device results.
Label desktop findings and automated tests separately.

* [ ] Start demo on the PC's private LAN address with the [README command](../README.md#use-your-phone).
* [ ] Open the printed URL on each phone.
* [ ] Check that no DEMO label or top status bar appears.
* [ ] Observe the full current cycle and indications listed in [Demo behavior](#demo-behavior).
* [ ] Check that nose up lowers the horizon and right bank rotates it counterclockwise.
* [ ] Rotate between portrait and landscape.
* [ ] Check readable scales/status, no horizontal scrolling, and no overlap with browser bars or safe areas.
* [ ] Exercise lock/unlock, app switching, and browser hide/restore.
* [ ] Check that live attitude requires a fresh snapshot, without restoring a frozen valid-looking indication.
* [ ] Disable and restore Wi-Fi.
* [ ] Check obscured attitude and retry status during loss, then automatic recovery without reload.
* [ ] Stop and restart the server on the same port.
* [ ] Check connection loss and automatic recovery despite sequence restart.

Evidence template for the issue or PR:

| Field | Value |
| --- | --- |
| Build commit / local changes | |
| Phone model | |
| OS version | |
| Safari / Chrome version | |
| Network and server address | |
| Demo poses and directions | Pass/fail, observations |
| Portrait / landscape / safe areas | Pass/fail, observations |
| Lock / background / resume | Pass/fail, observations |
| Wi-Fi loss / recovery | Pass/fail, observations |
| Server restart / recovery | Pass/fail, observations |

### Controlled browser fixtures

1. Start the development-only fixture server.

   ```powershell
   node tests/browser-fixture-server.mjs
   ```

2. Open `http://127.0.0.1:8082`.
3. Enter one JSON line at a time in the server terminal.
4. Inspect the result after each line.

```json
{"state":"live","pitch_deg":10,"roll_deg":25,"hold":false,"malformed":false}
{"state":"paused"}
{"state":"disconnected"}
{"state":"invalid"}
{"state":"waiting"}
{"state":"stale"}
{"state":"live","hold":false}
{"hold":true}
{"hold":false,"malformed":true}
{"malformed":false}
{"state":"live","slip_skid":-1,"turn_rate_dps":-3}
{"slip_skid":1,"turn_rate_dps":3}
{"slip_skid":"bad","turn_rate_dps":3}
{"slip_skid":0,"turn_rate_dps":"bad"}
{"attitude_only":true}
{"attitude_only":false,"slip_skid":0,"turn_rate_dps":0}
```

| Scenario | Check |
| --- | --- |
| `hold: true` | Comments continue without samples. Live attitude becomes stale after one second |
| Malformed JSON | Instrument becomes obscured |
| Pitch +/-90 degrees, roll -180 / 179.999 degrees | No uncovered background |
| Reconnect or background/resume | Check the logged active SSE count for parallel subscriptions from one page |

> The fixture server serves real web assets with synthetic SSE on loopback only. It is separate from the release binary.
> These checks establish UI behavior, not simulator compatibility.

### Responsive PFD layout and desktop checks

Open `http://127.0.0.1:8082/layout.html` for the development-only arrangement of all 18 intended illustration items.
The `LAYOUT FIXTURE` badge distinguishes future illustrative values from telemetry.
Use this page before changing layout geometry or adding the next PFD instrument.
No fixture page, script, or future instrument values enter the Rust binary.

| Check | Expected result |
| --- | --- |
| 320x240, 326x246, 320x480, 390x664, 568x240, 667x280, 844x320 CSS px | No scrolling or overlap; readable central attitude and supplemental values |
| Minimum usable area | 320 CSS px wide and 240 CSS px high after browser bars/safe areas |
| Rotation / viewport changes during streaming | Preserve values and sample deadline; round ball and undistorted symbols |
| Pitch scale / warnings | 2.5-degree intervals; red chevrons start at +60/-40 scale positions and point toward the horizon |
| Slip/skid | Center and both directions; a local unavailable mark for missing or invalid data |
| Turn rate | +/-3 deg/s reaches the standard-rate marks; beyond +/-6 deg/s shows an edge arrow |
| Reserved future regions | No fake readings or unfinished instrument scales on the production page |

An optional scripted desktop run uses Playwright with an installed browser.
It adds no application dependency or frontend build step.
Install the test tool in ignored local storage if it is not already available:

```powershell
npm install --prefix .local/browser --no-save playwright
$env:PLAYWRIGHT_MODULE = Join-Path $PWD '.local/browser/node_modules/playwright'
$env:BROWSER_CHANNEL = 'msedge'
node tests/pfd-browser.mjs
node tests/pfd-runtime-browser.mjs
```

The script starts an isolated loopback fixture server on an OS-assigned port.
It checks real assets, viewports, known poses, old/partial snapshots, stale states, resize, and page lifecycle handlers.
Screenshots, including opacity candidates in both orientations against sky and ground, are saved under `.local/pfd-browser/` for visual inspection. The 326x246 viewport provides a 320x240 usable panel with the normal 3-pixel margins.
Review the screenshots; passing geometry checks alone do not establish readability.
Synthetic padding and page events do not establish actual cutout, browser-bar, or background behavior on phones.
The runtime script uses the compiled Windows binary from the required build.
It checks embedded assets and Rust demo SSE, then stops and restarts its server to check recovery without reloading the page.

Run the documented real-device procedure on iOS Safari and Android Chrome for #20.
Include centered/left/right slip, both standard-rate turns, both chevron scenarios, orientation, browser bars, lock/resume, Wi-Fi and server recovery.
Record device, OS/browser, build and network details in the issue.
Keep #20 open while this hardware evidence is missing.

## Measurements and evidence

| Owner | Scope |
| --- | --- |
| #7 | Original simulator reliability/performance |
| #20-#26 | Planned PFD demo acceptance, without expanding #7 or #8 |

Record:

* OS, CPU, phone models, and browser versions.
* Simulator, SDK, and aircraft versions.
* Build commit, source rate, network conditions, duration, and failures.
* Memory/subscriber trends with two phones for at least 30 minutes.

| Measurement | Method |
| --- | --- |
| Receipt-to-render | Browser monotonic clock and frame callbacks |
| Total latency | Instrumented source with measured clock offset/uncertainty, or synchronized/high-frame-rate recording of source and phone |
| Report | Method uncertainty and p50/p95. Never subtract unrelated device timestamps |
| Proposed target | p95 <= 150 ms from source acquisition through visible rendering on healthy LAN |

Attach results to the validation issue.
Keep hardware-dependent tasks open until observations support acceptance.
Unit tests cannot certify simulator or phone behavior.

## Telemetry and demo checks

```powershell
cargo test --locked --test telemetry
```

These tests require no simulator or SDK.

### Demo behavior

| Property | Expected behavior |
| --- | --- |
| Cycle | Nine poses over 18 seconds, each held for two seconds |
| Poses | Level, nose up/down, left/right bank, two combined attitudes, then +70/-50 pitch |
| Coordinated turn | Centered and both-direction slip/skid; coordinated and uncoordinated +/-3 deg/s turns |
| Sampling | One fresh sample every 50 ms, including unchanged poses. Skip missed ticks |
| Initial state | `waiting`, then `live` on fresh data |
| Expiry | No accepted sample for 1,000 ms makes attitude stale |
| Source failure | Never substitute demo for unavailable SimConnect |

### Fixtures and tests

| Fixture / suite | Coverage |
| --- | --- |
| `tests/fixtures/attitudes.json` | Independent degree/sign expectations for all seven demo poses |
| `tests/fixtures/pfd-samples.json` | Nine independent extended samples, repeated every 360 publications |
| `tests/fixtures/snapshots.json` | v1 wire shape, including null attitude in unavailable states |
| Telemetry tests | Age, stale threshold, fresh identical samples, cadence, skipped ticks, normalized bounds, roll wrap, non-finite rejection, source identity, slow independent consumers |
| Library tests | Sequence exhaustion, shutdown ownership, subscription closure, listener reuse |

* Keep fixtures synchronized with contract changes.
* Reuse fixtures for future browser tests.
* Use a paused monotonic Tokio clock instead of wall-clock sleeps for timing checks.

## Planned PFD demo acceptance

> Apply these procedures as #20-#26 introduce features. They do not describe existing functionality or establish live PFD compatibility.

* Each feature issue owns its automated and real-phone evidence.
* #26 links earlier evidence and records a full-panel run for all 18 elements.
* Use the device/build/network evidence fields above.
* Preserve original simulator acceptance scope.

| Area | Scenarios and acceptance |
| --- | --- |
| End-to-end values | Independent fixtures agree across typed Rust data, deterministic demo, SSE, and display. Clients share values |
| Compatibility | Old v1 snapshots display attitude. Missing PFD fields display unavailable, never zero. No excluded-instrument placeholders |
| Instrument failures | Missing/invalid optional values affect only their indication. Source/transport loss invalidates the panel |
| Timing | Controlled clocks check five-second altitude alerts, target-change reset, freshness, and expiry before pending draw. No wall-clock sleeps |
| Attitude/turn geometry | Pitch/bank signs and extremes, slip/skid both ways, +/-3 deg/s turns |
| Tape geometry | IAS/altitude rolling digits, speed boundaries, six-second trend, altitude bugs outside tape, VSI limits, heading wrap at 359/0 |
| Mobile layout | From 320 CSS px, portrait, short landscape, safe areas, browser-bar changes, rotation while streaming |
| Layout acceptance | No scrolling, overlap, clipped warnings, or distorted symbols |
| Lifecycle | Wi-Fi recovery, server restart, lock/background/resume, source pause/stale/invalid states, partial failures |
| Recovery | Fresh sample before valid readings return. No duplicate subscription |

1. In #20, inspect layout fixtures for the complete intended arrangement before later slices add live demo values.
2. Check real-phone readability with each added instrument.
3. Run the [automated checks](#automated-checks) and follow [CONTRIBUTING](../CONTRIBUTING.md).
4. Update the feature's contract fixtures and procedures in the same PR.

> Desktop viewport checks do not establish mobile acceptance.

## Modular and layered PFD acceptance

#36 implements modular rendering. #37 implements layer containers and full-panel background coverage; opacity acceptance requires real-phone results. This document defines procedures, not passing results.
Run the required automated checks above for each implementation PR.
Record real iOS Safari and Android Chrome results separately from desktop evidence.

| Owner | Checks |
| --- | --- |
| #36: modules | Preserve the current image, pose signs, combined transforms, slip/skid, turn rate, and local unavailable indications |
| #36: lifecycle | Coalesce latest samples; preserve deadlines through resize; cancel invalid work; renew freshness for unchanged fresh values |
| #36: recovery | Expire without new samples; reject an expired sample before paint; recover after reconnect and resume only with fresh data |
| #36: assets | Serve every imported module through the compiled Rust server and the development fixture server |
| #36: DOM ownership | Preserve SVG node identity across samples and resize. Check actual transformed geometry, not only transform strings |
| #37: layers | Fill the entire panel with sky/ground; clip pitch markings centrally; preserve fixed references and layer order |
| #37: readability | Check digits, ticks, and value windows against sky and ground; record the selected background opacity |
| #37: bounds | Permit intentional background overlap; reject content collisions; include painted strokes in bounds checks |
| #37: status | No top bar or success labels; no DEMO label; unavailable reasons on an opaque full-panel cover; accessible state changes |
| #37: invalidation | Obscure the expanded background on global loss; keep optional failures local |

1. Use the full 18-element development fixture without shipping unfinished readings. Compare `/layout.html?opacity=0.50`, `0.65`, and `0.80` on both sky and ground; record the chosen value and device evidence in #37.
2. Check 320 CSS px page width and 240 CSS px usable height after browser bars and safe areas, as defined in the brief.
3. Check portrait, short landscape, orientation changes, extreme poses, and no scrolling.
4. Repeat lifecycle checks after geometry changes without renewing the sample deadline.

Later feature slices repeat relevant layer checks with real demo indications.
#24 additionally tests timed alert cancellation with a controlled monotonic clock.
#26 collects full-panel acceptance after all preceding instrument steps.
