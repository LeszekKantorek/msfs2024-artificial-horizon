# Test and acceptance strategy

> Procedures and criteria belong here. Record results and acceptance progress in the related GitHub issue or PR.

## Find the required procedure

Start with the change scope, then select the affected procedure:

| Task | Source | Use condition |
| --- | --- | --- |
| Select checks | [Validation scope](#validation-by-change-scope) | Every change |
| Reuse previous results | [Evidence reuse](#evidence-reuse) | Relevant validation already exists |
| Run application checks | [Automated checks](#automated-checks) | Application code, dependencies, or build changes |
| Validate presentation | [pfd-validate-presentation](../.agents/skills/pfd-validate-presentation/SKILL.md) | Rendering, geometry, lifecycle, or asset integration changes |
| Validate speed indications | [IAS/GS](#airspeed-and-ground-speed-acceptance), [TAS](#true-airspeed-acceptance), [ranges and trend](#airspeed-ranges-references-and-trend-acceptance) | Select the changed speed indication |
| Validate altitude indications | [Altitude/BARO](#altitude-and-pressure-acceptance), [alerts](#selected-altitude-and-alert-acceptance), [VSI](#vertical-speed-acceptance), [trend placement](#altitude-trend-placement-acceptance) | Select the changed right-side indication |
| Validate heading | [Heading acceptance](#heading-acceptance) | Heading changes or full-panel acceptance |
| Validate phones | [Real-phone procedure](#real-phone-acceptance-procedure) | Device behavior or feature acceptance |
| Supply controlled phone inputs | [Phone fixture setup](#phone-fixture-setup) | Limits, partial failures, or repeatable visual scenarios |
| Record binary or visual evidence | [Build provenance](#build-provenance), [screenshot comparisons](#screenshot-comparisons) | Results depend on the tested executable or captures |
| Change acceptance CI | [Validator checks](pr-acceptance.md#validate-changes-to-the-control) | Acceptance parser or workflow changes |

## Validation by change scope

Use the changed behavior and acceptance criteria to select local checks:

| Change | Required checks | Scope |
| --- | --- | --- |
| Documentation or context only | Diff, links, anchors, terminology, preserved requirements, and claims against current sources | No local application build or browser run |
| Session hooks or other development tooling | Changed behavior and failure paths | Application checks only when application inputs change |
| PR acceptance control | [Validator suite](pr-acceptance.md#validate-changes-to-the-control), permissions, and trusted-code boundary | Acceptance tooling |
| Rust or browser application code, dependencies, or build configuration | [Automated checks](#automated-checks) and relevant behavior checks below | Affected application behavior |
| PFD rendering, geometry, lifecycle, or module/asset integration | Both [desktop browser scripts](#responsive-pfd-layout-and-desktop-checks) and relevant screenshot inspection | Additional PFD evidence |
| Mobile behavior or feature acceptance | Applicable real iOS Safari and Android Chrome checks | Device evidence separate from desktop results |

Apply these scope rules:

* Use every applicable row when a change crosses boundaries.
* Preserve CI triggers and required GitHub statuses independently of this local matrix.
* Report checks prevented by explicit task restrictions.
* Leave the corresponding acceptance criteria unconfirmed when required evidence is missing.

### Evidence reuse

Check existing evidence before repeating a validation run:

1. Identify the tested revision and environment.
2. Compare relevant source, fixtures, dependencies, commands, configuration, and the tested artifact.
3. Explain why later changes do or do not affect the recorded result.
4. Select the action below.

| Evidence condition | Action |
| --- | --- |
| Relevant inputs and environment unchanged | Reuse the recorded result |
| Prose-only correction outside application inputs | Retain applicable application evidence |
| New summary of the same unchanged result | No additional run |
| Relevant changes, failures, incomplete logs, or unresolved concerns | Run affected checks again |
| Staged refactor without final integration evidence | Run required final checks on the combined outcome |

> Stage results alone do not establish final integration. A recorded final run remains reusable under the same conditions.

## Altitude and pressure acceptance

Issue [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) owns altitude and pressure evidence.
Use the [altitude contract](telemetry-contract.md#optional-altitude-and-pressure-setting) and [presentation rules](pfd-presentation.md#altitude-and-barometric-setting).

Automated coverage includes these scenarios:

| Check | Expected behavior |
| --- | --- |
| Rust types and publication | Finite signed feet, verbatim BARO strings, independent omission, shared age, attitude-only clearing, and stale suppression |
| Independent demo fixture | Six altitude sweeps, all segment endpoints, pressure changes, zero crossing, and repeated cycles |
| HTTP/SSE | Both clients receive the same altitude/pressure and sample age; unavailable snapshots omit extensions |
| Node tape geometry | Fixed pixel spacing, known tick positions, wider spans and more labels on taller tapes, signed ticks, and direction |
| Node readouts | 20-ft drums, carries, fractions, signs, and range boundaries |
| Node client and scheduler | Independent field normalization, old snapshots, unchanged callback shape, reconnect/resume, coalescing, and expiry before resize draw |
| Browser fixture | Both compositions, fixed pixel spacing through resize, retained nodes, wider portrait coverage, painted clearance, local failures/overflow, and global recovery |
| Release runtime | Embedded altitude modules with correct MIME, real Rust SSE values in SVG/ARIA, producer-formatted BARO text, and recovery after restart |

The existing [automated commands](#automated-checks) include the altitude Rust and Node tests.
Both [desktop browser scripts](#responsive-pfd-layout-and-desktop-checks) are required.
Altitude screenshots appear under `.local/pfd-browser/altitude-*`.
Inspect narrow portrait and short landscape captures, including long signed readouts over sky and ground.

### Tape footers and source-formatted BARO

Issue [#65](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/65) owns the revised footer and BARO acceptance.
The desktop layout script runs `tests/tape-footer-browser-checks.mjs` against both compositions.

| Check | Expected behavior |
| --- | --- |
| Rust, SSE, normalization, and SVG | `23.83 inHg`, `1026 hPa`, and `BARO` reach the readout unchanged |
| Availability and recovery | Missing, null, and non-string data show red `X`; empty and whitespace strings remain available; expiry and recovery follow the common sample deadline |
| Unit labels and scales | Literal `[KT]` and `[FT]` have the same font and visual height, with 1 px above and below desktop glyph bounds; both clips fill their allocated height |
| GS | Label and value share a 13 px font without a suffix; one-, two-, and three-digit values fit without overlap; opaque background matches BARO and the inset frame matches speed-trend magenta |
| BARO | Required examples fit at minimum width, centered in one cyan 11 px text node over opaque dark background with an inset cyan border |
| Background edges | Both 32 px footers meet their tape backgrounds and SVG edges at whole and fractional bounds, including simulated safe-area padding |
| IAS fallback | Four reference values fit between the warning and persistent `[KT]`, without a second unit heading |

Inspect `.local/pfd-browser/footer-*` screenshots at 320×480, 326×246, and 568×240 over both sky and ground.
Run the release-runtime script after building the final embedded Web assets.
For #65, apply the [shared phone procedure](#real-phone-acceptance-procedure) and inspect footer readability and edge seams throughout the layout scenarios.

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #23, with these altitude-specific checks:

1. Watch a complete cycle for climbing, descending, negative altitude, zero crossing, digit carries, and pressure changes.
2. Check the 100/20-ft markings, fixed pointer, rolling digits, aligned `[KT]`/`[FT]`, and source-formatted BARO text.
3. Compare orientations at the same zoom: tick spacing stays fixed and the taller tape shows more values.
4. Use [phone fixtures](#phone-fixture-setup) for independent missing/invalid altitude and pressure, both overflow boundaries, and long signed digits.

## HTTP startup checks

Prerequisites: Rust stable, rustfmt, and Clippy on Windows x64 MSVC.
Use committed `Cargo.lock`. These checks require no simulator or SDK.

1. Run the [automated checks](#automated-checks), including the build and tests.
2. Record the compiler version with results. CI records it in its logs.

   ```powershell
   rustc --version --verbose
   ```

3. Start the server.

   ```powershell
   cargo run --locked --bin msfs2024-artificial-horizon
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

Automated lifecycle tests cover normal shutdown and cancellation:

* Library tests signal shutdown independently and rebind within a five-second completion timeout.
* Cancellation tests abort a task or drop a polled server future after publication with two active SSE clients.
* Check producer/subscription closure, client EOF, and port reuse after cancellation.
* Permit the final pending sample before subscription closure.
* Public configuration rejects port zero.
* Internal listener tests use OS-assigned ports to avoid collisions.

> Desktop checks do not establish real-phone or simulator compatibility.

## Selected altitude and alert acceptance

Issue [#24](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) owns selected-altitude and alert evidence.
Use the [telemetry contract](telemetry-contract.md#optional-selected-altitude) and [presentation rules](pfd-presentation.md#selected-altitude-and-alerts).

Automated coverage exercises these boundaries:

| Boundary | Checks |
| --- | --- |
| Rust model and demo | Finite signed targets, independent omission, shared freshness, old publications, two SSE clients, 198-second checkpoints and unchanged 18-second cycles |
| Alert transitions | Both approach directions, exact thresholds, unrounded values, skipped bands, sustained values, target changes, capture, departure, return and repeat departure |
| Controlled time | 250-ms flash phases, completion at 5000 ms, delayed/early callbacks, identical fresh samples, resize, coalesced samples and expiry before draw |
| Invalidation | Local missing/overflow values, paused/stale/disconnected/waiting/invalid states, suspension, stale callbacks and silent recovery |
| SVG | Matching notched symbols in header and tape, readout above bug, signed capture, edge clearance, colors, availability and stable nodes |
| Release runtime | Embedded selected module, actual Rust target in SSE/SVG, pinned bug and recovery after restart |

Use these checks and artifacts for automated acceptance:

* The [automated Rust and Node commands](#automated-checks) include these tests.
* Both [desktop browser scripts](#responsive-pfd-layout-and-desktop-checks) are required for this feature.
* The selected SVG checks use real panel modules with injected monotonic time.
* They check bug paint visibility and minus-sign clearance at positive and negative capture without five-second wall-clock waits.
* Inspect screenshots at `.local/pfd-browser/selected-altitude-*` and `.local/pfd-runtime-selected-altitude.png`.

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #24, with these alert-specific checks:

1. Watch the 198-second demo for both approach directions, full flash intervals, target changes, and yellow deviation.
2. Check the pinned bug and selected value against sky and ground in portrait and short landscape, including 320 CSS px width.
3. Use [phone fixtures](#phone-fixture-setup) for target loss, malformed targets, both overflow boundaries, current-altitude loss, pressure-only loss, and return into capture.
4. Perform the shared loss and resume scenarios during an alert. Recovery requires fresh data and silent initialization.

## Heading acceptance

Use the [magnetic-heading contract](telemetry-contract.md#optional-magnetic-headings) and [presentation rules](pfd-presentation.md#current-and-selected-heading).
Run the existing Rust and Node commands, then both desktop browser scripts listed below.

| Check | Required evidence |
| --- | --- |
| Rust/SSE | Canonical `[0, 360)` values, valid zero, independent absence, shared sequence/age/expiry, clearing on state publications, and identical samples for two clients |
| Scenario | Independent phase checkpoints, both north crossings, +/-3 deg/s agreement, 360-sample heading repeat and unchanged 3960-sample altitude cycle |
| Node | Three digits, degree formatting, rounding near north, shortest angular distance, 180-degree left tie, whole bug edge pinning, and whole-label hiding |
| SVG | Real glyph and stroke bounds, fixed 4 CSS px/degree spacing, cyan notch capture, both edges, stable nodes and independent unavailable states |
| Lifecycle | Latest shared sample, expiry before drawing, resize without refreshed age, pause, disconnect, reconnect and background/resume without another subscription |
| Full panel | All 17 included elements, altitude trend, target alerts, and both orientations from 320 by 240 CSS px |

Inspect the browser evidence:

1. Inspect `.local/pfd-browser/heading-production-*` and `heading-layout-*` for north, capture, and both pinned edges.
2. Confirm the current 359-degree / selected 1-degree bug is exactly 8 CSS px right of center.
3. Confirm cyan remains around the black pointer at capture, without covering its text or border.
4. Inspect `.local/pfd-runtime-heading-north.png` for the compiled Rust demo's north crossing.
5. Record the tested commit, Windows/MSVC and browser versions, command results, and screenshot paths in #26 or its PR.

The fixture accepts these independent heading inputs:

```json
{"heading_deg":359,"selected_heading_deg":1}
{"heading_deg":1,"selected_heading_deg":359}
{"heading_deg":0,"selected_heading_deg":0}
{"heading_deg":354,"selected_heading_deg":90}
{"heading_deg":354,"selected_heading_deg":180}
{"heading_deg":354,"selected_heading_deg":null}
{"heading_deg":null,"selected_heading_deg":1}
{"heading_deg":360,"selected_heading_deg":1}
```

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #26, including full-panel results and the controlled heading scenarios above.

## Vertical-speed acceptance

Issue [#25](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/25) owns VSI evidence.
Use the [contract](telemetry-contract.md#optional-vertical-speed) and [presentation rules](pfd-presentation.md#vertical-speed) for units, ranges, and availability.

Automated acceptance covers these boundaries:

| Boundary | Checks |
| --- | --- |
| Rust model and SSE | Finite signed rates, zero, no telemetry limit, omission, state clearing, shared age, stale suppression, old snapshots, and matching clients |
| Scenario | Independent 3960-sample checkpoints, every jump and phase, analytic altitude/rate agreement, 25 ft/s producing +1500 ft/min, and unchanged other cycles |
| Node calculations | Signed zero, rounding, raw +/-9999 boundaries, invalid values, window geometry, recovered horizon width, and label/tick obstruction |
| Altitude trend | Source +1500 ft/min predicts +150 ft over six seconds and extends 60 px upward. Descent, clipping, zero, missing inputs and independent pressure |
| Shared lifecycle | Latest coherent frame, resize without renewed freshness, expiry before drawing, pause/disconnect, reconnect and background/resume without duplicate subscriptions |
| SVG | Shared font family/color, 13-pixel VSI text, matching right text edges and 51-by-27 window. Three-pixel painted gap, text bounds, target bug, label/tick clearance, nodes and partial loss |
| Release runtime | Embedded altitude/VSI/trend modules with GET/HEAD and JavaScript MIME, actual Rust ft/min in SSE/SVG, six-second climb prediction, hold and restart recovery |

Use these checks and artifacts:

* Run the [automated Rust and Node commands](#automated-checks) and both [desktop browser scripts](#responsive-pfd-layout-and-desktop-checks).
* Retain the selected-altitude alert regressions, including expiry while flashing and delayed callback cancellation.
* Inspect `.local/pfd-browser/vsi-*` for the minimum viewport, narrow portrait, short landscape, negative-altitude capture and long signed values.
* Inspect `.local/pfd-browser/altitude-trend-*` for upward/downward paint over the left ticks and below the turquoise bug.
* Inspect `.local/pfd-runtime-vertical-speed.png` for the compiled demo's +1500 ft/min climb beside altitude and its selected target.

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #25, with these VSI-specific checks:

1. Watch the 198-second altitude/VSI cycle for `0` during holds, positive climb, and negative descent directly below altitude.
2. Inspect matching readout styles, the narrower rectangle, selected target/bug, and pressure against sky and ground.
3. Check that the magenta trend extends up for climb, down for descent, and disappears during holds.
4. Check that the trend covers intersecting ticks and that the notched turquoise bug covers the trend.
5. Use [phone fixtures](#phone-fixture-setup) with the inputs below for signed limits, partial failures, and missing data.

Enter these JSON lines into the fixture terminal:

```json
{"vertical_speed_fpm":1500}
{"vertical_speed_fpm":-1500}
{"vertical_speed_fpm":0}
{"vertical_speed_fpm":2000.01}
{"vertical_speed_fpm":-2000.01}
{"vertical_speed_fpm":9999}
{"vertical_speed_fpm":-9999}
{"vertical_speed_fpm":-9999,"altitude_ft":-60,"selected_altitude_ft":-100}
{"vertical_speed_fpm":9999.01}
{"vertical_speed_fpm":-9999.01}
{"vertical_speed_fpm":null}
{"vertical_speed_fpm":1500,"altitude_ft":null,"baro_text":null}
{"vertical_speed_fpm":1500,"altitude_ft":8500,"baro_text":"29.92 inHg","hold":true}
{"hold":false}
```

### Altitude trend placement acceptance

[#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63) owns the corrected placement and paint order.
Use both [desktop browser scripts](#responsive-pfd-layout-and-desktop-checks) and inspect the production page and full layout fixture.

The automated checks cover these visible results:

| Scenario | Expected result |
| --- | --- |
| 10060 ft, +1500 / -1500 ft/min | Six-second prediction at 10210 / 9910 ft, with endpoints 60 CSS px above / below center |
| Tick crossing | The magenta 3 CSS px stroke at local `x = 2` covers the tick stroke and retains 0.5 CSS px clearance inside either side of its four-pixel clip |
| Positive and negative capture, either rate sign | The turquoise strip covers the trend and the black pointer fits the notch |
| Selected target above / below the scale, +10000 / -10000 ft/min | The clipped trend stays below the whole pinned bug at either edge |
| Readout overlap | The opaque altitude window and its border cover the trend and bug |
| VSI readout | Aligned text edges, contained glyphs, unchanged window dimensions and three-pixel painted gap |
| Invalid inputs and recovery | Zero or missing required data hides the trend. BARO/target loss does not hide it |
| Resize | Tape width and pixel spacing remain unchanged. The outside four pixels return to the horizon and heading |

Paint checks use transformed points that belong to both strokes or fills, including ancestor clips.
Negative checks temporarily reverse the relevant SVG order and require the same point to reveal the wrong object.
Restore the original order before the next check.
Wait for the SVG `viewBox` to match its CSS bounds before measuring resized geometry.

Screenshot and device acceptance use this procedure:

1. Capture matching before/after scenes at 320x480 portrait, 568x240 short landscape and the 320x240 minimum.
2. Include climb, descent, capture in both directions, negative-altitude capture and both pinned edges on production and fixture pages.
3. Record the input values, viewport, tested commit and browser version with the screenshots.
4. Repeat on actual iOS Safari and Android Chrome, including rotation, browser bars and safe areas.
5. Confirm readable digits and strokes, correct layering and recovery after source or connection loss.
6. Record device, OS/browser versions, commit, observations and screenshots in #63 or its PR.

Keep the device criterion unchecked until both real-device records exist.
Desktop viewport checks do not establish phone acceptance.

## Automated checks

Run these commands from the repository root on Windows x64 MSVC.
They match the checks in [CI](../.github/workflows/ci.yml).

Complete the [environment setup](../CONTRIBUTING.md#environment-setup) first.
Use committed `Cargo.lock` with `--locked`.
The checks require no simulator or SDK.

Use one target for Clippy, the build, and tests:

```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
$env:CARGO_BUILD_TARGET = 'x86_64-pc-windows-msvc'
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
node --test tests/*.test.mjs
cargo build --locked --release
```

CI runs these checks in the `Build & Test` job without restoring artifacts from previous workflow runs.
Cargo commands reuse artifacts in `target` within the job and during local development.
The final step builds the release executable at `target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe`.
Tests use the debug profile.
The release build compiles optimized artifacts separately.

> The Windows SimConnect module contains no SDK integration. Passing these checks does not establish simulator compatibility.
> Future SDK-dependent checks require documented prerequisites and distribution constraints.

The [PR acceptance control](pr-acceptance.md#validate-changes-to-the-control) has a separate Node suite and runs on Linux.
Its tooling dependencies do not affect browser controller checks or the application build.

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
| Telemetry validation | Direct internal-module tests, independent optional fields, old snapshots, copied profile/reference values, and unbounded finite speed input |
| Sample callbacks | Current status before attitude before full sample, with separate attitude copies, original attitude-only shape, and matching deadlines |
| DOM-free imports | `tests/helpers.test.mjs` imports visual owners/helpers/telemetry without `document` and exercises meaningful pure behavior |
| Speed calculations | `tests/airspeed.test.mjs` and `tests/airspeed-ranges.test.mjs` import readout, tape, references, IAS, and shared speed helpers directly |
| Horizon | `tests/horizon.test.mjs` imports `attitudeTransforms` from `web/horizon/index.js` for independent pose signs and extreme/default transforms |
| Frame scheduler | Latest-frame/layout coalescing, coherent resize and render, expiry before draw, cancellation on data loss, resize without renewing age, fresh identical values |
| Panel/status | Complete read-only frame delegation, shared attitude clip through resize/expiry, status without a sample, stale obscuring, separate source/transport state and announcements |
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

## Real-phone acceptance procedure

Use this procedure with the affected feature's scenarios and evidence owner.
Run it separately on actual iOS Safari and Android Chrome.
Desktop emulation does not establish device acceptance.

1. Identify the feature issue, tested build, and applicable scenarios before the run.
2. Start the compiled demo on the private LAN using the [phone setup](../README.md#use-your-phone). Open its printed URL on each phone.
3. Exercise the feature's cycle and controlled inputs, inspecting values, symbols, and local failure marks against both sky and ground.
4. Check portrait, short landscape, rotation, browser-bar changes, safe areas, and undistorted, readable indications without scrolling or collisions.
5. Use the [minimum usable dimensions](project-brief.md#mobile-presentation) and the feature's painted-bounds criteria when checking layout.
6. Exercise lock/unlock, app switching, and browser hide/restore. Require a fresh sample before valid indications return.
7. Disable and restore Wi-Fi. Check the unavailable cover and retry status, followed by fresh-data recovery without reload.
8. Stop and restart the server on the same port. Require recovery despite sequence restart and no duplicate subscriptions.
9. Record each device separately using the template below. Put the feature's observations and screenshots in its issue or PR.
10. Leave missing or failing device criteria unchecked. Keep the owning issue open until both required device results support acceptance.

Check uninterrupted freshness as well as recovery: no frozen valid-looking indication may survive loss or resume.
For simulator-specific checks, also apply the [manual acceptance matrix](#manual-acceptance-matrix).

Evidence template for the issue or PR:

| Field | Value |
| --- | --- |
| Build commit / local changes | |
| Date / viewport / zoom | |
| Phone model | |
| OS version | |
| Safari / Chrome version | |
| Network and server address | |
| Feature / controlled inputs / screenshots | Pass/fail, observations |
| Demo poses and directions | Pass/fail, observations |
| Portrait / landscape / safe areas | Pass/fail, observations |
| Lock / background / resume | Pass/fail, observations |
| Wi-Fi loss / recovery | Pass/fail, observations |
| Server restart / recovery | Pass/fail, observations |

### Phone fixture setup

Use controlled inputs for repeatable bounds, partial failures, or visual scenarios.
Start the development fixture on the PC's specific private LAN IP:

```powershell
$env:PFD_FIXTURE_HOST = '<PC LAN IP>'
node tests/browser-fixture-server.mjs
```

Open the printed address on the phone. Use `/` for production composition and `/layout.html` for all 17 illustration elements.
Enter the feature's JSON lines in the fixture server terminal.
The host defaults to loopback and accepts IP addresses only.
HTTP has no fixture-control route, and fixture assets are absent from the Rust binary.
Restrict firewall access to the trusted private subnet. Remove `PFD_FIXTURE_HOST` from the environment after the phone run.

## Attitude display: real-device phone acceptance

Apply the [shared phone procedure](#real-phone-acceptance-procedure) and record results in #4.
Add these attitude checks for each phone:

* [ ] Check that no DEMO label or top status bar appears.
* [ ] Observe the full current cycle and indications listed in [Demo behavior](#demo-behavior).
* [ ] Check that nose up lowers the horizon and right bank rotates it counterclockwise.

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

> The fixture server serves real web assets with synthetic SSE on loopback by default. Explicit LAN binding supports phone checks. It is separate from the release binary.
> These checks establish UI behavior, not simulator compatibility.

### PFD presentation validation procedure

Use [pfd-validate-presentation](../.agents/skills/pfd-validate-presentation/SKILL.md) for the presentation validation procedure.
The skill owns its steps; this document retains check commands, feature criteria, evidence requirements, and the real-phone procedure.

### Responsive PFD layout and desktop checks

Open `http://127.0.0.1:8082/layout.html` for the development-only arrangement of all 17 intended illustration items.
The `LAYOUT FIXTURE` badge distinguishes the development fixture from the compiled demo.
Use this page before changing layout geometry or adding the next PFD instrument.
No fixture page or fixture control script enters the Rust binary.

| Check | Expected result |
| --- | --- |
| 320x240, 326x246, 320x480, 390x664, 568x240, 667x280, 844x320 CSS px | No scrolling or indication-content collisions, with readable central attitude and supplemental values |
| Minimum usable area | 320 CSS px wide and 240 CSS px high after browser bars/safe areas |
| Rotation / viewport changes during streaming | Preserve values and sample deadline, with a round ball and undistorted symbols |
| Pitch scale / warnings | 2.5-degree intervals, with red chevrons starting at +60/-40 scale positions and pointing toward the horizon |
| Slip/skid | Center and both directions, with a local unavailable mark for missing or invalid data |
| Turn rate | +/-3 deg/s reaches the standard-rate marks, with an edge arrow beyond +/-6 deg/s |
| Heading strip | Production heading module, without ground-track or separate numeric selected-heading illustrations |

The scripted desktop checks use Playwright with an installed browser.
They are required for the PFD changes in the [validation matrix](#validation-by-change-scope) and optional for unrelated exploration.
It adds no application dependency or frontend build step.
Install the test tool in ignored local storage if it is not already available:

```powershell
npm install --prefix .local/browser --no-save playwright
$env:PLAYWRIGHT_MODULE = Join-Path $PWD '.local/browser/node_modules/playwright'
$env:BROWSER_CHANNEL = 'msedge'
node tests/pfd-browser.mjs
node tests/pfd-runtime-browser.mjs target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe
```

| Script | Input | Checks | Output |
| --- | --- | --- | --- |
| `pfd-browser.mjs` | Real web assets and isolated loopback fixture on an OS-assigned port | Viewports, poses, partial/old snapshots, stale states, resize, lifecycle, identity, frozen frames, hierarchy/paint order, shared clips, and background coverage | Screenshots under `.local/pfd-browser/`, including opacity candidates in both orientations against sky and ground |
| `pfd-runtime-browser.mjs` | Release binary built from the checked source and passed explicitly | Every current imported module with HTTP 200/JavaScript MIME, Rust demo SSE, coherent bank/turn state within 25 seconds, and restart recovery without reload | `.local/pfd-runtime-landscape.png` |

The scripted geometry checks include these regression cases:

* The coverage check must reject a deliberately narrowed background clip and pass after restoration.
* Check pitch geometry immediately after `ResizeObserver`, before the next animation callback.
* The 326x246 viewport provides a 320x240 usable panel with the normal 3-pixel margins.
* Import changes require matching Rust embedded routes, fixture allowlist/MIME, HTTP asset tests, and the runtime expected-module set.
* Check the shared panel attitude clip used by the bank pointer and fixed symbols, including resize while data is expired.
* Check matching world/pitch/chevron transforms, fixed aircraft/outline paths, and original SVG hierarchy across poses and resize.

Asset checks cover the current module graph:

* The [architecture tree](architecture.md#intended-source-layout) lists the runtime modules.
* `src/http/assets.rs` registers the current modules, HTML, and CSS through explicit mappings.
* `tests/http.rs` checks the explicit embedded routes and JavaScript MIME.
* Both browser scripts check retired routes return 404 without compatibility facades.

Retired paths map to these current owners:

| Retired route | Current owner |
| --- | --- |
| `/math.js`, `/svg.js` | `/helpers.js` |
| `/telemetry-client.js` | `/telemetry/client.js` |
| `/airspeed.js` | `/airspeed/index.js` and its visual objects |
| `/horizon.js`, `/fixed-symbols.js` | Their feature directories |
| `/airspeed/calculations.js`, `/horizon/calculations.js` | Pure functions in the implementing visual objects |

Pure checks follow the existing module boundaries:

* Pure functions remain testable from their visual modules in Node without DOM.
* Separate calculation files are not required for tests.
* Check observable boundaries, carry behavior, references, warnings, clipping, and recovery instead of duplicating private implementation formulas.

> Review the screenshots: geometry checks alone do not establish readability.
> Synthetic padding and page events do not establish actual cutout, browser-bar, or background behavior on phones.

Complete the separate phone acceptance for #20:

1. Run the real-device procedure on iOS Safari and Android Chrome for #20.
2. Include centered/left/right slip, both standard-rate turns, both chevron scenarios, orientation, browser bars, lock/resume, Wi-Fi and server recovery.
3. Record device, OS/browser, build and network details in the issue.
4. Keep #20 open while this hardware evidence is missing.

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

### Build provenance

Distinguish build execution from artifact creation:

* Cargo can reuse an existing artifact after a successful command.
* A changed executable hash establishes different bytes, not their source inputs.

Record these fields with binary-dependent validation:

| Field | Required evidence |
| --- | --- |
| Source | Commit, relevant uncommitted changes, and source checkout path |
| Build | Exact command, target/profile, toolchain, target directory, complete output, and native exit code |
| Artifact | Absolute executable path, size, timestamp, and structured SHA-256 value |
| Runtime check | Exact command, artifact path, environment, output, and native exit code |

Preserve usable artifact records:

* Store hash data as structured JSON or an object with path and hash fields.
* Avoid display tables that truncate paths or hash values.
* Link existing logs and artifacts from context entries instead of copying their full contents.

If a copied checkout and build cache leave artifact provenance uncertain:

1. Select a fresh target directory.
2. Rebuild from the source checkout into that target directory.
3. Record the new artifact before making claims about that binary.

> Command success or a runtime screenshot alone does not establish recompilation.

### Screenshot comparisons

Compare captures only after recording viewport, scale, browser, fonts, input state, capture method, and source/artifact identity.

1. Preserve baseline and candidate captures with their metadata.
2. Record the difference extent and location, including raw comparison output.
3. Inspect the changed area visually.
4. Assess whether the difference violates the applicable geometry, readability, color, or availability requirement.
5. Record a cause only when evidence establishes it.

Report uncertainty explicitly:

* Use "cause not established" when the mechanism behind a pixel difference remains unknown.
* Separate the observed difference, its impact assessment, and any possible cause.
* Do not present browser noise or rendering variation as an established cause without evidence.
* Leave affected acceptance unresolved when the evidence cannot determine the impact.

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
| Coordinated turn | Centered and both-direction slip/skid, with coordinated and uncoordinated +/-3 deg/s turns |
| IAS/GS | 0–2 s: zero. 2–8 s: acceleration to IAS 150 / GS 170 kt. 8–10 s: hold. 10–16 s: deceleration to zero. 16–18 s: zero |
| Sampling | One fresh sample every 50 ms, including unchanged poses. Skip missed ticks |
| Initial state | `waiting`, then `live` on fresh data |
| Expiry | No accepted sample for 1,000 ms makes attitude stale |
| Source failure | Never substitute demo for unavailable SimConnect |

### Fixtures and tests

| Fixture / suite | Coverage |
| --- | --- |
| `tests/fixtures/attitudes.json` | Independent degree/sign expectations for all seven demo poses |
| `tests/fixtures/pfd-samples.json` | Nine independent extended samples, repeated every 360 publications |
| `tests/fixtures/speed-samples.json` | Independent IAS/GS checkpoints during acceleration, hold, deceleration, and repeat |
| `tests/fixtures/airspeed-profile.json` | Independent fictional thresholds and named reference configuration |
| `tests/fixtures/speed-indications.json` | Source acceleration and six-second predictions at segment boundaries |
| `tests/fixtures/snapshots.json` | v1 wire shape, including null attitude in unavailable states |
| Telemetry tests | Age, stale threshold, fresh identical samples, cadence, skipped ticks, normalized bounds, roll wrap, non-finite rejection, source identity, slow independent consumers |
| Library tests | Sequence exhaustion, shutdown ownership, subscription closure, listener reuse |

Maintain fixtures and timing checks as follows:

* Keep fixtures synchronized with contract changes.
* Reuse fixtures for future browser tests.
* Use a paused monotonic Tokio clock instead of wall-clock sleeps for timing checks.

## Airspeed and ground-speed acceptance

Run the automated checks and both desktop browser scripts for airspeed implementation changes.
Check these speed-specific behaviors:

| Area | Expected behavior |
| --- | --- |
| Compatibility | Old snapshots show attitude with unavailable IAS/GS. Unsupported envelopes remain globally invalid |
| Input | Zero and fractional speeds are valid. Negative/non-finite/malformed/missing speeds affect only their own indication |
| Presentation | IAS major labels every 10 kt, minor ticks every 2 kt, fixed pointer, rolling digits, no negative scale labels |
| Rollover | Check 9.5 → 10, 99.5 → 100, and reverse transitions. Leading zeros stay blank |
| Display limits | 999 kt is readable. 999.01 and 1000 kt show `OVR`. Missing or invalid values show `X`, never zero |
| GS | 137.6 kt shows 138 beside the GS label, without a visible unit suffix. The accessible description retains knots |
| Lifecycle | Equal fresh speeds renew validity. Resize preserves the accepted deadline. Reconnect/resume require a new fresh sample without duplicate subscriptions |
| Integration | Two SSE clients receive matching IAS/GS. The compiled Rust demo supplies 150/170 kt during its hold segment |
| Fixture | Items 1/4/8 use the real speed module, without duplicate speed readouts. All other included items also use production modules |

### Real-phone checks

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #21.
Check IAS acceleration, hold, deceleration, and distinct GS, including digits, ticks, and local failure marks against sky and ground.
Use [phone fixtures](#phone-fixture-setup) for controlled rollover, limits, and failures with these inputs:

```json
{"ias_kt":99.5,"gs_kt":137.6}
{"ias_kt":100,"gs_kt":999}
{"ias_kt":999.01,"gs_kt":1000}
{"ias_kt":null,"gs_kt":137}
{"ias_kt":123,"gs_kt":null}
{"ias_kt":123,"gs_kt":137,"hold":true}
{"hold":false}
```

## True airspeed acceptance

Issue [#66](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/66) owns TAS acceptance.
Use the [speed contract](telemetry-contract.md#optional-speed-indications) and [TAS presentation](pfd-presentation.md#true-airspeed).

| Check | Expected behavior |
| --- | --- |
| Rust and SSE | Independently optional finite nonnegative TAS, unchanged v1 envelope, atomic publication, shared age, two matching clients, and clearing on omission or source loss |
| Demo | Independent speed checkpoints repeat, including simultaneous IAS 150, TAS 160, and GS 170 |
| Browser | Source-supplied TAS reaches SVG and ARIA; zero, fractions, rounding, 999, overflow, missing/null/malformed input, old snapshots, expiry and recovery have explicit expectations |
| Partial loss | Missing TAS preserves valid IAS/GS. Missing IAS or GS preserves TAS |
| Geometry | Stable production nodes in both compositions; glyph containment for one-, two-, three-digit values, X and OVR; inset frame, opaque background, and clearance from heading and tape |
| Runtime | Embedded TAS module supports HTTP/MIME and HEAD checks; real demo readouts recover after server restart without reload |

Run the [automated checks](#automated-checks) and both [desktop scripts](#responsive-pfd-layout-and-desktop-checks).
`true-airspeed-browser-checks.mjs` covers 320×480, 326×246, and 568×240 at whole/fractional bounds and simulated safe-area padding.
Inspect `.local/pfd-browser/tas-*` screenshots over sky and ground.

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #66, using [phone fixtures](#phone-fixture-setup) in both compositions.
Enter these controlled samples:

```json
{"ias_kt":150,"tas_kt":160,"gs_kt":170}
{"tas_kt":0}
{"tas_kt":85.5}
{"tas_kt":999}
{"tas_kt":999.01}
{"tas_kt":null}
{"tas_kt":"85"}
{"tas_kt":160,"omit_tas":true}
{"omit_tas":false,"tas_kt":160,"ias_kt":null}
{"ias_kt":150,"gs_kt":null}
{"gs_kt":170,"hold":true}
{"hold":false}
```

Check continuous backgrounds and readable TAS labels throughout the shared rotation, layout, and recovery scenarios.

## Airspeed ranges, references, and trend acceptance

Run the automated checks and both browser scripts for range, reference, or trend implementation changes.
The #22 checks use the existing demo and fixture transport.
The [contract](telemetry-contract.md#optional-airspeed-ranges-and-trend) defines the fictional profile and source acceleration.

| Area | Expected behavior |
| --- | --- |
| Profile | Five positive, strictly increasing thresholds and four named positive references at or below VNE. Invalid configuration affects only its indications |
| Ranges | Red below 40, white 40-85, green 50-110, yellow 110-140, red/white from 140 kt. White overlays the left half of green where their intervals overlap |
| Range clipping | Unbounded overspeed ranges clip to the tape. Literal expectations cover infinite bounds and preserve `NaN` coordinates without zero substitution |
| Range lane (#64) | Red, green, yellow, and striped bands share `x = tape width - 18` and 6 CSS px width. White uses the same left edge and 3 CSS px width, painted above green |
| Range paint (#64) | At IAS 65 kt, inspect 72 kt above the readout: white covers the left half and green remains visible on the right. Reversing their SVG order must fail the same paint check; restore the original order afterward |
| Trend thickness (#64) | Airspeed and altitude trends share a 3 CSS px stroke and the same magenta at their existing axes. Check positive/negative prediction and both clip boundaries |
| Scale clearance (#64) | Ticks and numbers move 2 CSS px left within the 1 px wider airspeed allocation. Original tick lengths, 1 px stroke, 12 px labels, reference speed anchors, and opaque IAS pointer remain readable |
| Boundaries | Values below, at, and above every threshold retain coherent geometry. Marker positions match configured speeds |
| Trend | Six-second prediction comes from source acceleration. Positive, negative, zero, missing, malformed, and overflowing calculations have independent expectations |
| VNE | IAS 140 kt makes the pointer red. IAS 110 with +5 kt/s predicts exactly 140 and makes digits yellow. Current readings never become predictions |
| References | R/X/Y/G appear at their scale positions. Off-tape markers hide without an edge substitute. Missing IAS shows R 55, X 65, Y 75, G 90 above GS |
| Reference edge cases | Fractional, tiny, and above-display-range values retain exact accessible values with bounded fallback text. Coincident and nearby labels stay separate with exact leader anchors |
| Partial loss | Invalid profile retains IAS/GS and a valid trend. Invalid acceleration retains profile and actual overspeed. Old snapshots retain attitude |
| Freshness | Irregular message receipt does not change source acceleration. Equal fresh samples renew validity. Expiry before painting and resize preserve the original deadline |
| Integration | Two SSE clients receive matching profile and acceleration. The release binary demonstrates a predictive warning, actual overspeed, hold with hidden trend, and restart recovery |
| Layout | Three IAS columns fit the window. Marker strokes stay within the tape. Fallback rows stay below local X and above GS at the minimum viewport |

The browser checks use independent expected positions and inspect computed colors, SVG bounds, clipping edges, and node identity.
Screenshots include `speed-approaching-vne.png`, `speed-at-vne.png`, and fallback views under `.local/pfd-browser/`.

`checkAirspeedAvailabilityTransitions` in `tests/airspeed-browser-checks.mjs` checks 40 loss/resize/recovery transitions through `pfd-browser.mjs`:

| Scenario | Coverage |
| --- | --- |
| Ten-step sequence | IAS loss, profile loss, combined loss with IAS recovering first, and combined loss with profile recovering first |
| Initial states | Actual overspeed and predictive warning |
| Pages | `/` and `/layout.html` |
| Resize | 844x320 to 326x246, alternating with 320x480 during recovery |

Transition assertions preserve independent availability and state resets:

* Check computed colors/visibility, state resets, exact replacement-profile ARIA, fresh fractional digits/tape offsets, and fallback/unit/badge behavior.
* GS remains independently available with nonzero painted bounds inside the panel and viewport.
* Production SVG node identity remains stable.
* The development fixture reuses the same production nodes and geometry operations.

### Real-phone checks for #22

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #22.
Inspect ranges, references, acceleration/deceleration, yellow predictive digits, red overspeed pointer, and constant-speed trend removal.
Use [phone fixtures](#phone-fixture-setup) for the controlled cases below in both `/` and `/layout.html`.

Enter these JSON lines into the fixture terminal. The fixture initially supplies the configured profile.

```json
{"ias_kt":55,"ias_acceleration_ktps":0}
{"ias_kt":65,"ias_acceleration_ktps":0}
{"ias_kt":75,"ias_acceleration_ktps":0}
{"ias_kt":90,"ias_acceleration_ktps":0}
{"ias_kt":110,"ias_acceleration_ktps":5}
{"ias_kt":139.99,"ias_acceleration_ktps":0}
{"ias_kt":140,"ias_acceleration_ktps":0}
{"ias_kt":140.01,"ias_acceleration_ktps":-5}
{"ias_kt":null,"gs_kt":137}
{"ias_kt":123,"ias_acceleration_ktps":null}
{"ias_kt":123,"ias_acceleration_ktps":5,"airspeed_profile":null}
{"attitude_only":true}
```

Restart the fixture to restore its initial profile after the partial-loss checks.
Check immediately below and above the five thresholds with the same input mechanism.
Real-phone acceptance requires passing records from both devices. Desktop checks provide separate evidence.

### Wider airspeed bands acceptance (#64)

Preserve matched before/after screenshots for `/` and `/layout.html` at 320x240, 320x480, and 568x240 CSS px.
Use IAS 25, 65, 95, 123, and 150 kt to cover low speed, white/green overlap, normal, caution, and striped overspeed.
Include rising and falling trends, range thresholds, readout overlap, and clipping at both tape edges.
Inspect painted bounds, labels, and V-speed references after resize settles.
Confirm that other PFD stroke widths remain unchanged.
The IAS/TAS/GS column gains 1 CSS px, the horizon loses 1 CSS px, and the altitude allocation remains unchanged.

Apply the [shared phone procedure](#real-phone-acceptance-procedure) on actual iOS Safari and Android Chrome.
Inspect the 6 px range lane, its 3 px white left overlay, and matching 3 px airspeed and altitude trends.
Check portrait/landscape, rotation, browser bars, safe areas, connection loss, and lock/background recovery.
Record the tested revision, device model, OS/browser versions, date, viewport, observations, and pass/fail.
Leave device acceptance unconfirmed until both records pass.

## Planned PFD demo acceptance

> Apply these procedures as #20-#26 introduce features. They do not describe existing functionality or establish live PFD compatibility.

* Each feature issue owns its automated and real-phone evidence.
* #26 links earlier evidence and records a full-panel run for all 17 elements.
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
| Layout acceptance | No scrolling, indication-content collisions, clipped warnings, or distorted symbols, with intentional background overlap allowed |
| Lifecycle | Wi-Fi recovery, server restart, lock/background/resume, source pause/stale/invalid states, partial failures |
| Recovery | Fresh sample before valid readings return. No duplicate subscription |

1. In #20, inspect layout fixtures for the complete intended arrangement before later slices add live demo values.
2. Check real-phone readability with each added instrument.
3. Run the [automated checks](#automated-checks) and follow [CONTRIBUTING](../CONTRIBUTING.md).
4. Update the feature's contract fixtures and procedures in the same PR.

> Desktop viewport checks do not establish mobile acceptance.

## Modular and layered PFD acceptance

These checks cover instrument composition, layers, and visual objects:

* #36 implements modular rendering.
* #37 implements layer containers and full-panel background coverage.
* #56 composes visual objects and keeps object-specific calculations with their implementations under [ADR 0005](adr/0005-visual-object-composition.md).
* Opacity acceptance requires real-phone results.
* Run the required automated checks above for each implementation PR.
* Record real iOS Safari and Android Chrome results separately from desktop evidence.

> This document defines procedures, not passing results.

| Owner | Checks |
| --- | --- |
| #36: modules | Preserve the current image, pose signs, combined transforms, slip/skid, turn rate, and local unavailable indications |
| #36: lifecycle | Coalesce latest samples, preserve deadlines through resize, cancel invalid work, and renew freshness for unchanged fresh values |
| #36: recovery | Expire without new samples, reject an expired sample before paint, and recover after reconnect and resume only with fresh data |
| #36: assets | Serve every imported module through the compiled Rust server and the development fixture server |
| #36: DOM ownership | Preserve SVG node identity across samples and resize. Check actual transformed geometry, not only transform strings |
| #37: layers | Fill the entire panel with sky/ground, clip pitch markings centrally, and preserve fixed references and layer order |
| #37: readability | Check digits, ticks, and value windows against sky and ground, and record the selected background opacity |
| #37: bounds | Permit intentional background overlap, reject content collisions, and include painted strokes in bounds checks |
| #37: status | No top bar, success labels, or DEMO label, with unavailable reasons on an opaque full-panel cover and accessible state changes |
| #37: invalidation | Obscure the expanded background on global loss, keeping optional failures local |
| #56: ownership | Same complete read-only frame at instrument entry points, concrete child inputs, immediate-parent shared hosts/clips, and no visual frame cache/timer |
| #56: composition | Preserve hierarchy, defs/paint order, SVG identity, ARIA, common attitude clip, and coherent once-per-frame transforms |
| #56: recovery | Repeat IAS/profile loss during resize and both recovery orders, with independently painted GS and reset warnings/references |
| #56: assets | Verify the complete module graph through explicit Rust/fixture registration and HTTP/MIME checks, with the listed retired routes returning 404 |

Record refactor validation against the final revision:

1. Preserve baseline screenshots.
2. Record the tested commit, environment, complete logs, native exits, and [build provenance](#build-provenance).
3. Describe and assess each [screenshot difference](#screenshot-comparisons) without inventing an unverified cause.
4. Run required final checks on the combined revision, following the [evidence reuse rules](#evidence-reuse).
5. Keep actual iOS Safari and Android Chrome acceptance unchecked until passing device evidence exists.
6. Record remaining checks under the issue and PR `Additional verification`.

Apply the [shared phone procedure](#real-phone-acceptance-procedure) for #37 and compare opacity with [phone fixtures](#phone-fixture-setup):

1. Open the full 17-element development fixture.
2. Compare `/layout.html?opacity=0.50`, `0.65`, and `0.80` against sky and ground.
3. Record the selected value and real-device evidence in #37.
4. Include extreme poses in the shared layout scenarios.
5. Repeat lifecycle checks after geometry changes without renewing the sample deadline.

Later feature slices extend these checks:

* Keep unfinished readings out of production.
* Repeat relevant layer checks with real demo indications.
* #24 additionally tests timed alert cancellation with a controlled monotonic clock.
* #26 collects full-panel acceptance after all preceding instrument steps.
