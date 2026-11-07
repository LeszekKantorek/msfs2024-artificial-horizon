# Demo provider

Demo publishes deterministic synthetic flight samples through the same telemetry channel and HTTP/SSE path as SimConnect.
All subscribers observe one producer's scenario state.

## Requirements and usage

* Select Demo at startup with `--source demo`, or omit the source option for the CLI default.
* Follow the [startup procedure](../../README.md#start-the-demo) and [private LAN setup](../../README.md#use-your-phone).
* Require Windows x64 MSVC, but no simulator, SDK headers, or SimConnect DLL.
* Keep acquisition, scenario values, selected targets, and the fictional speed profile in Rust.
* Keep the browser read-only. It receives values without generating flight data or sending setting/control commands.
* Preserve the [v1 field meanings and optional-value rules](../telemetry-contract.md).
* Never select Demo automatically after a SimConnect failure.

## Public API

The `providers::demo` module exposes these functions:

| Interface | Contract |
| --- | --- |
| `run(publisher, shutdown)` | Consume the supplied `Publisher` and await shutdown with output `()`. Return `Result<(), TelemetryError>` |
| `sample(index)` | Return an extended `FlightSample` for the deterministic sample index |
| `attitude(sample)` | Return the original seven-pose `Attitude` cycle, with 40 samples per pose |
| `speed_profile()` | Return validated fictional thresholds and named references |
| `normalize_degrees(pitch, roll)` | Wrap finite roll to `[-180, 180)` and validate attitude. Return `TelemetryError` for invalid input |

The runtime and its caller have these responsibilities:

* `run` owns separate sample and freshness intervals of 50 ms and skips missed ticks.
* Each sample tick publishes `sample(index)` and advances the index modulo 3960.
* Freshness ticks call `Publisher::expire()` even without sample publication.
* Shutdown takes priority over both ticks.
* The caller supplies a channel identified as `Source::Demo` and owns HTTP and process signals.

## Scenario timing

Playback follows these timing rules:

* Scenario times describe nominal playback at 20 Hz.
* The runtime advances by published sample index, so skipped ticks do not advance the scenario by elapsed wall time.

| Scenario | Samples | Nominal period |
| --- | --- | --- |
| Extended attitude, coordinated turn, speeds, heading, and pressure | 360 | 18 seconds |
| Altitude, selected altitude, and vertical speed | 3960 | 198 seconds |
| Standalone `attitude` helper | 280 | 14 seconds |

Scenario cycles preserve these relationships:

* The full producer cycle contains eleven short cycles.
* The standalone seven-pose helper remains compatible with its original fixtures.
* `sample` adds two extreme-pitch poses without changing that helper.

### Attitude and coordinated turn

Each row covers 40 samples, or two nominal seconds:

| Time | Pitch (degrees) | Roll (degrees) | Slip/skid | Turn rate (deg/s) |
| --- | --- | --- | --- | --- |
| 0–2 s | 0 | 0 | 0 | 0 |
| 2–4 s | 15 | 0 | -0.5 | 0 |
| 4–6 s | -15 | 0 | 0.5 | 0 |
| 6–8 s | 0 | -30 | 0 | -3 |
| 8–10 s | 0 | 30 | 0 | 3 |
| 10–12 s | 10 | 25 | 1 | 3 |
| 12–14 s | -10 | -25 | -1 | -3 |
| 14–16 s | 70 | 0 | 0 | 0 |
| 16–18 s | -50 | 0 | 0 | 0 |

The [attitude feature](../efi/attitude.md#demo-behavior) owns expected visual behavior and regression checks.

### Speed cycle and profile

IAS, TAS, and GS share one fraction while retaining independent values:

| Time | Fraction | IAS acceleration |
| --- | --- | --- |
| 0–2 s | Zero | 0 kt/s |
| 2–8 s | Linear increase from zero to one | +25 kt/s |
| 8–10 s | One | 0 kt/s |
| 10–16 s | Linear decrease to zero | -25 kt/s |
| 16–18 s | Zero | 0 kt/s |

Speed calculations use these values:

* Multiply the fraction by 150 kt for IAS, 160 kt for TAS, and 170 kt for GS.
* At sample `step = index % 360`, the increasing fraction is `(step - 40) / 120`.
* The decreasing fraction is `(320 - step) / 120`.

The demo supplies this fictional configuration:

> These values do not establish the performance of a real aircraft.

```json
{
  "airspeed_profile": {
    "vs0_kt": 40.0, "vs1_kt": 50.0, "vfe_kt": 85.0,
    "vno_kt": 110.0, "vne_kt": 140.0,
    "references": {"vr_kt": 55.0, "vx_kt": 65.0, "vy_kt": 75.0, "vg_kt": 90.0}
  },
  "ias_acceleration_ktps": 25.0
}
```

Acceleration and fixtures follow these rules:

* The demo acceleration is +25 kt/s during acceleration, zero during hold and rest, and -25 kt/s during deceleration.
* At each segment boundary, the new segment supplies the acceleration.
* Independent fixtures reside in `airspeed-profile.json` and `speed-indications.json`.

### Altitude and pressure

The first 18 seconds of the shared 198-second Rust altitude demo contain six three-second sweeps.
Each sweep includes both endpoints over 60 samples:

| Time | Start altitude (ft) | End altitude (ft) |
| --- | --- | --- |
| 0–3 s | -120 | 120 |
| 3–6 s | 980 | 1020 |
| 6–9 s | 9980 | 10020 |
| 9–12 s | 10020 | 9980 |
| 12–15 s | 1020 | 980 |
| 15–18 s | 120 | -120 |

Pressure, display references, and validation follow these rules:

* The producer cycles through `29.92 inHg`, `30.12 inHg`, and `29.72 inHg` at six-second intervals.
* Transitions between sweeps are synthetic test cases, not a physical flight trajectory or a pressure-response model.
* `tests/fixtures/altitude-samples.json` contains independent checkpoints and cycle boundaries.
* [Altitude presentation](../efi/altitude.md#presentation) defines the display limits and rolling drums.
* Convert scenario pressure to `baro_setting.pressure_hpa` with `HPA_PER_INHG`. The browser owns display-unit formatting.

### Selected-altitude phases

The shared altitude, target, and vertical-speed demo repeats after 3960 samples at 20 Hz:

| Time | Altitude / target |
| --- | --- |
| 0-18 seconds | Six signed sweeps, without selected altitude |
| 18-108 seconds | Approach 10000 ft from 8500 ft, hold, then depart to 10300 ft |
| 108-198 seconds | Approach 1000 ft from 2500 ft, hold, then depart to 700 ft |

Each 90-second approach uses this schedule:

| Duration | Altitude relative to target |
| --- | --- |
| 6 seconds | Hold 1500 ft away |
| 40 seconds | Move to 500 ft away |
| 20 seconds | Reach target |
| 6 seconds | Hold target |
| 12 seconds | Depart by 300 ft |
| 6 seconds | Hold deviation |

Related cycles and validation remain shared:

* Attitude, speeds, coordinated-turn values, and pressure use 18-second cycles.
* `tests/fixtures/selected-altitude-samples.json` contains independent checkpoints.
* See [selected-altitude presentation](../efi/selected-altitude.md#presentation) for alert rules and display validity.

### Vertical speed

The demo derives altitude and VSI from shared segment definitions:

| Segment | VSI |
| --- | --- |
| First six altitude sweeps | Analytic slope, approximately +/-4881.36 or +/-813.56 ft/min |
| Target approach and departure | +1500 ft/min upward, -1500 ft/min downward |
| Holds | Zero |
| First sample after artificial jumps | Omitted at cycle indices 0, 60, 120, 240, 300, 360, and 2160 |
| Continuous phase boundary | Rate of the phase beginning at that sample |

VSI calculations and validation use these rules:

* `tests/fixtures/vertical-speed-samples.json` supplies independent segment and jump checkpoints.
* See [VSI presentation](../efi/vertical-speed.md#presentation) for display limits and [VSI acceptance](../efi/vertical-speed.md#testing) for checks.
* The six sweeps use `(end - start) / 59` feet per sample to include both endpoints. Multiply that slope by 1200 samples per minute to obtain the analytic VSI.
* Target phases interpolate over their sample intervals and use the next phase's value at each boundary.

### Heading cycle

The heading demo repeats every 360 samples at 20 Hz, following source turn rate:

| Time | Current heading | Selected heading | Turn rate |
| --- | --- | --- | --- |
| 0-2 seconds | 354 degrees | 350 degrees | 0 deg/s |
| 2-4 seconds | 354 degrees | 90 degrees | 0 deg/s |
| 4-6 seconds | 354 degrees | 180 degrees | 0 deg/s |
| 6-8 seconds | 354 to 348 degrees | 350 degrees | -3 deg/s |
| 8-10 seconds | 348 to 354 degrees | 350 degrees | +3 deg/s |
| 10-12 seconds | 354 to 0 degrees | 0 degrees | +3 deg/s |
| 12-14 seconds | 0 to 354 degrees | 0 degrees | -3 deg/s |
| 14-16 seconds | 354 degrees | Unavailable | 0 deg/s |
| 16-18 seconds | 354 degrees | 354 degrees | 0 deg/s |

Heading boundaries and validation follow these rules:

* Phase boundaries use the starting phase's values.
* Normalization wraps calculations into `[0, 360)`.
* The producer wraps after 3960 samples.
* The altitude cycle remains 198 seconds.
* `tests/fixtures/heading-samples.json` supplies independent checkpoints, including both north crossings.
* See [heading presentation](../efi/heading.md#presentation) and [heading acceptance](../efi/heading.md#testing).

## Planned native Demo playback

The accepted native Demo design is not implemented yet. The preceding sections describe the current direct provider. [ADR 0009](../adr/0009-native-demo.md) records the change and its relationship to earlier decisions. The [scenario catalog](demo-scenarios.md) identifies planned scenarios; the [SimConnect guide](simconnect.md#planned-common-native-consumer) owns the native catalog and error outcomes.

The future default is embedded from repository-root `demo_scenario.jsonl`; explicit files use the same parser. It preserves applicable native-input phases and independent boundary checks, but does not supply fictional VFE/VNO/VNE or VX/VY/VG. IAS acceleration remains the receiver estimator. Existing full-profile presentation fixtures and requirements remain valid. Recorded-session conversion, diagnostic schema 4/5 import, and observations mode are excluded. Forward scenario-to-output verification relates the selected input, revision and timing to normalized log/SSE observations; it does not reconstruct the scenario from a recording.

### JSONL meaning

Each nonempty physical line contains one complete JSON object.
The file contains many records. JSON strings escape embedded line breaks.
Reject blank lines, comments, trailing commas, duplicate keys, unknown fields, unsupported record types, and unsupported format/version identifiers.
Accept LF and CRLF. Write UTF-8 without a BOM.

Read and validate the complete scenario before opening a simulated connection.
JSONL does not require streaming playback or append-while-running behavior.
External file changes take effect on the next launch.

### Scenario suite and documentation

Support multiple independently selectable JSONL files through an explicit scenario option.
The main flight narrative includes only recoverable lifecycle/connection changes.
Isolate fatal native configuration, malformed packet, and cleanup cases in separate files.
Do not continue a narrative after a terminal producer failure merely to reach later phases.

The scenario catalog documents purpose, assumptions, expected completion, and file links only.
Keep timing, native values, and fault parameters in JSONL.
Keep independent acceptance checkpoints in tests/fixtures rather than deriving them from the reader under test.

### Header and time

The first record is a header:

```jsonl
{"type":"scenario","format":"msfs-efi-simconnect-demo","version":1,"mode":"scripted","frame_period_ms":50,"duration_us":15000000,"initial":{"sim_running":true,"pause":"off","flight":"demo-flight.flt"}}
```

| Field | Contract |
| --- | --- |
| `mode` | `scripted`; recorded-session playback is excluded |
| `frame_period_ms` | Positive integer duration of a synthetic simulation frame; default scenario uses 50 ms. Apply the client's request semantics to those frames |
| `duration_us` | Positive integer duration of one pass through the scenario steps |
| `initial` | Scenario-authored Sim-running flag, pause mode, and loaded-flight identity or null; applied once at playback initialization |
| Time arithmetic | Checked integer arithmetic; segment durations in ms convert exactly to internal microseconds |
| Ordering | Per-track segments follow file order. Equal-time events and observations preserve record order |
| Validation | Positive segment durations; all timestamped records fall within one pass; track periods must be explicit and coherent |

The exact resource caps belong to implementation, but must be finite and documented.
Reject oversized lines/files, excessive records, invalid numeric values, and overflowing schedules before acquisition.

Configure validates without starting time. The first successful Open starts the monotonic clock; registration overlaps scenario time. The origin persists across reconnects. The DLL owns the emitter, connection state, and receive buffers. Open borrows the notification handle; successful Close stops and joins the emitter before releasing its use. Do not rewrite queued packets on dispatch. The [native consumer contract](simconnect.md#dispatch-and-error-limits) records the unresolved scheduler, empty-queue, and overflow evidence.

### Scripted signal tracks

The author-facing signal key is a scenario alias for one supported native SimVar.
It is not the client's request ID or definition ID.

```jsonl
{"type":"signal","id":"IAS","simvar":"AIRSPEED INDICATED","unit":"knots","repeat":false}
{"type":"hold","signal":"IAS","value":100,"duration_ms":5000}
{"type":"ramp","signal":"IAS","from":100,"to":150,"duration_ms":5000}
{"type":"hold","signal":"IAS","value":150,"duration_ms":5000}
```

Each `signal` declares `id`, `simvar`, `unit`, and Boolean `repeat`. Each `hold` has `signal`, finite numeric `value`, and positive integer `duration_ms`. Each `ramp` has `signal`, finite numeric `from` and `to`, and positive integer `duration_ms`. Each `gap` has `signal` and positive integer `duration_ms`. The signal IDs must be unique and each segment must name a declared signal. Unsupported native pairs are rejected.

Signal tracks start together at the beginning of each pass.
Each segment begins when the preceding segment on that track ends.
This supports independent speed, attitude, heading, and altitude cycles without a global cross-product of phases.

| Record | Meaning |
| --- | --- |
| `hold` | Emit current value at every requested scripted frame during the duration |
| `ramp` | Interpolate linearly from `from` at the segment start toward `to` at its end |
| `gap` | Emit no observations for that signal during the duration |
| Signal `repeat: true` | Repeat its complete segment sequence; its period equals the segment-duration sum |
| Signal `repeat: false` | Emit nothing after its final segment until the next scenario pass |
| Last scenario step | Return to the first step for the next pass on the same continuously increasing clock |

Demo DLL is a playback mechanism. It executes scenario instructions without inventing lifecycle transitions.
Playback always loops while the source remains running. There is no selectable `end` policy or implicit wait at the file boundary.
The monotonic clock never resets, rewinds, or pauses at a pass boundary, during a simulated pause, or across reconnects.
Only the playback cursor returns to the first step.
Record `at_us` values are offsets within a pass, not replacements for the clock value.
For pass number n starting at zero, a record is due at origin + n * duration_us + at_us.

Apply header `initial` once when playback is initialized. Do not reapply it on later passes or reconnects.
After initialization, pause, Sim-running, and loaded-flight state change only through explicit scenario records.
At a pass boundary, retain the state left by the preceding steps until another step changes it.
Emit no synthetic resume, flight activation, flight load, disconnect, or reconnect merely because the cursor loops.
If each pass needs a particular starting state, author the corresponding records at the beginning of the step sequence.
Do not clear pending packets, recreate the connection, or reset call-occurrence counters at a pass boundary.

A five-second 100-knot hold followed by a five-second ramp from 100 to 150 knots has value 125 knots at 2.5 seconds into the ramp. Values use the declared native unit.

Segments use half-open intervals: start inclusive, end exclusive.
The next segment supplies the boundary value.
The terminal hold in this example makes 150 observable at 10 seconds.
Without that hold, a repeated ten-second track jumps back to 100 at its boundary.
Its last 50 ms sample would be 149.5; do not claim it emitted the endpoint.
Inclusive legacy checkpoints must use an explicit endpoint hold or matching authored slope.
Do not silently replace the current inclusive altitude sweeps with different interpolation.

A hold is not a single received observation held inside the receiver.
A five-second gap does not send repeated cached values.
Missing optional observations expire according to the existing receiver's rules; they do not disappear instantly by implication.
If a required component of a grouped native definition is in a gap, suppress that complete requested packet.
For example, a missing pitch must not be paired with cached pitch and a current bank to create a fresh attitude.

#### Request-driven emission and unchanged values

Demo must match SimConnect delivery semantics for the requests used by the application.
The current instrument requests use SIMCONNECT_PERIOD_SIM_FRAME, flags 0, origin 0, interval 0, and limit 0.
They request observations on each simulation frame without a change-only filter or a transmission-count limit.
The [SimConnect request contract](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/events-and-data/simconnect_requestdataonsimobject/) distinguishes flags 0 from SIMCONNECT_DATA_REQUEST_FLAG_CHANGED.
The application does not request the CHANGED flag. Do not enable equivalent filtering implicitly in Demo.

* During `hold`, send the value on every due requested frame throughout the segment, even when it equals the previous value.
* During `ramp`, evaluate the current value and send it on every due requested frame. Numeric equality does not suppress a sample with flags 0.
* During `gap`, send no observations for that signal. Do not substitute zero or resend its previous value.
* Preserve independent request routing. A signal gap does not suppress unrelated signals, lifecycle events, or ordinary state-query replies.
* Generate packets independently of GetNextDispatch calls. Requested cadence controls production; dispatch retrieves already queued packets.

These rules apply while the request and source state permit delivery and no explicit scenario fault suppresses it.
Do not promise samples through a simulated disconnect or infer pause-time delivery from this rule; match the corresponding native state contract.
Do not hard-code change-only emission or ignore the request arguments to make Demo appear to work.
Support the used API subset; any later request-mode change requires matching Demo behavior before claiming compatibility.

For a healthy five-second IAS hold at 100, repeated packets carrying 100 are distinct observations.
Each received IAS observation refreshes IAS acquisition time in the server.
The existing 1000 ms expiry concerns missing observations, not an unchanged numeric value; ALT updates do not refresh IAS.
Keep these receive timestamps separate from the scenario clock, which continues increasing.

Headings use explicit unwrapped ramp endpoints when crossing north.
The existing receiver normalizes native headings. Add no general angular interpolation or expression language.


### Sparse inputs and registration

A scenario may provide any subset of signals, only lifecycle events, or no signal values. The DLL accepts supported registrations even when the scenario supplies no matching value. Unknown names and injected registration failures remain distinct from missing values. Missing values produce no samples and never become zero. A grouped definition emits no packet when any required component is missing; independent requests and events continue.

Unreceived readings are internally unavailable (`None`). Existing SSE represents unavailable attitude and sample age as null and omits absent optional instrument fields. State, source, and sequence remain present. A received zero is a real observation subject to normal validation and freshness.

### Lifecycle and faults

Timestamped records use `at_us` as a pass-relative offset. `pause` carries `mode` (`off`, `normal`, or `active`); `flight` carries `active`; `flight_loaded` carries a string or null `flight`; and `disconnect` carries a positive `unavailable_ms`. Normal pause maps to `Pause_EX1=1`, active pause to `4`, and off to `0`. Sim-running, loaded-flight identity, pause, and connection state remain distinct. A newly eligible attitude is required after resume, flight change, or reconnect. State queries return current `Sim` and `FlightLoaded`; subscription supplies initial pause knowledge.

Native pause delivery is an implementation verification requirement. The working assumption is continued messages with frozen values in both pause modes; it is not established SDK behavior. Verify each mode separately, including SIM_FRAME, events, state queries, and resume. The application's paused publication gate is separate. The scenario clock continues; baseline scenarios author holds across pause intervals rather than silently freezing ramps.

Fault records are opt-in and absent from the healthy default. `api_fault` selects an export, positive call occurrence, signed HRESULT, and optional field selector. `exception` identifies a sent call occurrence and operation, code, and index. `raw_packet` supplies owned bytes and a bounded returned length; `null_dispatch` explicitly tests a null pointer. `transport_silence` suppresses all dispatch packets for a positive duration, including state replies. Malformed JSON is a setup error; malformed native packets are deliberate runtime fixtures. Never return an unreadable non-null pointer or advertise more bytes than the allocation. Keep call counters across passes and reconnects. Stop the emitter before an injected Close failure. Fatal cases are isolated; coverage may be staged and deferred cases identified.

### Selection and setup

The future CLI selects `demo.dll` by default or a supported absolute DLL path. Basename matching is case insensitive, including `DEMO.dll`; export spelling remains exact. Bare supported names resolve beside the executable. Relative paths containing directories and unsupported basenames are rejected. A scenario path is accepted only for Demo and is resolved against the caller's working directory. The current startup commands remain in the README until this CLI is implemented.

The private setup ABI is:

```c
int32_t EfiDemo_ConfigureV1(const uint16_t *scenario_path,
                            uint8_t *error_utf8,
                            uint32_t error_capacity);
```

HTTP binds before Demo setup. The private `EfiDemo_ConfigureV1` call validates the complete file before Open; invalid or unreadable input returns `E_FAIL` and an actionable bounded explanation while HTTP remains available. It never falls back to another scenario. Setup does not publish, start the clock, or report parser errors through GetNextDispatch. The official DLL is never called with this export. Configure accepts a null path for the embedded default or an absolute NUL-terminated UTF-16 path, plus a nonempty writable UTF-8 error buffer and capacity. It synchronously copies arguments, writes a bounded NUL-terminated explanation on failure, and is called only without an open connection. Rust panics must not cross the DLL ABI.

## Validation

Select validation by scope:

* Use the [telemetry and Demo checks](../testing/runtime.md#telemetry-and-demo-checks) for publication, lifecycle, and fixture validation.
* Use [EFI feature pages](../efi/index.md#features) for instrument-specific rendering and acceptance scenarios.

| Boundary | Expected evidence |
| --- | --- |
| Original and extended attitude | `tests/fixtures/attitudes.json` and `tests/fixtures/pfd-samples.json`, including the seven-pose helper and nine-segment sample cycle |
| Speeds | `tests/fixtures/speed-samples.json`, including simultaneous IAS 150, TAS 160, and GS 170 kt |
| Profile and acceleration | `tests/fixtures/airspeed-profile.json` and `tests/fixtures/speed-indications.json` |
| Shared scenario | Independent altitude, target, VSI, and heading checkpoints referenced in the scenario sections |
| Multiple clients | Matching source samples from one acquisition channel, without per-client scenarios |
| Optional indications | Current attitude-only publication clears optional indications while preserving valid attitude |

> Demo and desktop evidence do not establish simulator compatibility or actual iOS Safari and Android Chrome acceptance.

Keep those results separate under the [device procedure](../testing/devices.md#real-phone-acceptance-procedure).
