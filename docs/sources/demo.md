# Demo provider

Demo uses the project-owned `demo.dll` through the same native receiver and HTTP/SSE path as SimConnect. Its values are synthetic and do not describe a real aircraft.

## Requirements and usage

* Build the workspace to place `demo.dll` beside the executable. The DLL requires no simulator or official SDK.
* Omit `--source`, select `--source demo.dll`, or provide an absolute path whose basename is `demo.dll`.
* Select an authored JSONL file for every Demo run with `--scenario <path>`. Relative scenario paths resolve against the working directory. Restart to load changes.
* Use `--log <path>` for optional synthetic native diagnostics. The browser remains read-only.
* Demo never replaces a failed SimConnect selection.

## Scenario timing

The `demo-scenarios/demo-scenario.jsonl` scenario uses a 50 ms synthetic frame and a 198-second continuously looping pass. Independent tracks use their own periods and restart at each pass boundary. Scheduler delays can skip frames. Source receipt and freshness follow the shared [SimConnect receiver](simconnect.md#delivery-frequency-and-loop-timing).

### Attitude and coordinated turn

Nine two-second attitude and turn phases use native pitch and bank radians, ball position, and turn rate. The input is in `demo-scenarios/demo-scenario.jsonl`. The [attitude feature](../efi/attitude.md#demo-behavior) owns presentation checks.

### Speed cycle and profile

IAS, TAS, and GS repeat over 18 seconds with a two-second zero hold, six-second increase, two-second maximum hold, six-second decrease, and two-second zero hold. Their maxima are 150, 160, and 170 kt. Native VS0, VS1, and minimum rotation are supplied; fictional VFE/VNO/VNE and VX/VY/VG are not. The existing receiver estimates IAS acceleration. Independent presentation fixtures retain their full-profile coverage.

### Altitude and pressure

The 198-second altitude pass starts with six three-second sweeps and then two 90-second selected-altitude phases. Pressure cycles through 29.92, 30.12, and 29.72 inHg. The [altitude feature](../efi/altitude.md#presentation) owns presentation checks.

### Selected-altitude phases

After the initial sweeps, the baseline approaches and departs 10000 ft, then 1000 ft. Selected altitude is absent during the initial sweeps. Independent checkpoints remain in the existing fixtures.

### Vertical speed

The native VSI track follows the scripted altitude phases and omits values at authored jumps. The receiver applies grouped-sample freshness and omits invalid optional values.

### Heading cycle

The magnetic heading and selected-heading tracks repeat every 18 seconds. The selected heading is absent during the authored gap. The receiver normalizes heading values.

## Native Demo playback

The application uses the native Demo DLL through the shared receiver. [ADR 0010](../adr/0010-native-demo.md) records the native playback decision. The [scenario catalog](demo-scenarios.md) identifies scenarios; the [SimConnect guide](simconnect.md#common-native-consumer) owns the native catalog and error outcomes.

The `demo-scenarios/demo-scenario.jsonl` file uses the same parser as other scenarios. It preserves applicable native-input phases and independent boundary checks, but does not supply fictional VFE/VNO/VNE or VX/VY/VG. IAS acceleration remains the receiver estimator. Existing full-profile presentation fixtures and requirements remain valid. Recorded-session conversion, diagnostic schema 4/5 import, and observations mode are excluded. Forward scenario-to-output verification relates the selected input, revision and timing to normalized log/SSE observations; it does not reconstruct the scenario from a recording.

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
| `frame_period_ms` | Positive integer duration of a synthetic simulation frame; baseline scenario uses 50 ms. Apply the client's request semantics to those frames |
| `duration_us` | Positive integer duration of one pass through the scenario steps |
| `initial` | Scenario-authored Sim-running flag, pause mode, and loaded-flight identity or null; applied once at playback initialization |
| Time arithmetic | Checked integer arithmetic; segment durations in ms convert exactly to internal microseconds |
| Ordering | Per-track segments follow file order. Equal-time events and observations preserve record order |
| Validation | Positive segment durations; all timestamped records fall within one pass; track periods must be explicit and coherent |

The Demo parser limits files to 4 MiB, lines to 65,536 bytes, and files to 100,000 records. It rejects larger inputs, invalid numeric values, and overflowing schedules before acquisition.

The receive queue holds at most 8,192 packets. On overflow, dispatch reports a failure rather than silently discarding a lifecycle transition. The scheduler skips missed frame observations and evaluates current values on the next frame. These policies are Demo behavior; SDK overflow and missed-frame behavior remain unverified.

Open validates the scenario before starting time. The first successful Open starts the monotonic clock; registration overlaps scenario time. The origin persists across reconnects. The DLL owns the emitter, connection state, and receive buffers. Open borrows the notification handle; successful Close stops and joins the emitter before releasing its use. Do not rewrite queued packets on dispatch. The [native consumer contract](simconnect.md#dispatch-and-error-limits) records the unresolved scheduler, empty-queue, and overflow evidence.

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
Only the playback cursor returns to the first step. Each pass restarts every track at its first segment. If a repeating track is partway through a segment at the pass boundary, that segment is cut short and the next pass starts its first segment. The track period need not divide `duration_us`. For example, a three-second repeating track in an eight-second pass starts at offsets 0, 3, and 6 seconds, then restarts at the next pass boundary at 8 seconds. This restarts track position only; it does not reset the clock, lifecycle state, or queued packets.
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
For grouped requests, Demo encodes a missing optional floating-point observation as synthetic NaN. The receiver omits that value in the new sample. A missing pitch or bank suppresses the grouped packet, as does a missing integer member. This synthetic behavior does not establish official missing-variable behavior.

#### Request-driven emission and unchanged values

Demo must match SimConnect delivery semantics for the requests used by the application.
The current instrument requests use SIMCONNECT_PERIOD_SIM_FRAME, flags 0, origin 0, interval 0, and limit 0.
They request observations on each simulation frame without a change-only filter or a transmission-count limit.
The [SimConnect request contract](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/events-and-data/simconnect_requestdataonsimobject/) distinguishes flags 0 from SIMCONNECT_DATA_REQUEST_FLAG_CHANGED.
The application does not request the CHANGED flag. Do not enable equivalent filtering implicitly in Demo.

* During `hold`, send the value on every due requested frame throughout the segment, even when it equals the previous value.
* During `ramp`, evaluate the current value and send it on every due requested frame. Numeric equality does not suppress a sample with flags 0.
* During `gap`, single-field requests send no observation. Grouped optional floating-point gaps use synthetic NaN; attitude or integer gaps suppress the packet. Never substitute zero or resend a previous value.
* Lifecycle events and ordinary state-query replies remain independent of a signal gap.
* Generate packets independently of GetNextDispatch calls. Requested cadence controls production; dispatch retrieves already queued packets.

These rules apply while the request and source state permit delivery and no explicit scenario fault suppresses it.
Do not promise samples through a simulated disconnect or infer pause-time delivery from this rule; match the corresponding native state contract.
Do not hard-code change-only emission or ignore the request arguments to make Demo appear to work.
Support the used API subset; any later request-mode change requires matching Demo behavior before claiming compatibility.

For a healthy five-second IAS hold at 100, repeated packets carrying 100 are distinct observations.
Each accepted grouped packet refreshes the common sample receipt time.
The 1000 ms expiry concerns missing packets, not an unchanged numeric value.
Keep these receive timestamps separate from the scenario clock, which continues increasing.

Headings use explicit unwrapped ramp endpoints when crossing north.
The existing receiver normalizes native headings. Add no general angular interpolation or expression language.


### Sparse inputs and registration

A scenario may provide any subset of signals, only lifecycle events, or no signal values. The DLL accepts supported registrations even when the scenario supplies no matching value. Unknown names and injected registration failures remain distinct from missing values. Missing values never become zero. A grouped optional floating-point gap becomes synthetic NaN; missing attitude or integer values suppress the packet. Single-field requests remain silent for a gap. This is Demo behavior, not proof of official behavior.

For integer datatypes, Demo sends only finite integral values within the requested type's range. A fractional or out-of-range value suppresses the complete packet instead of saturating it. This conversion policy is synthetic; it does not establish the official DLL's conversion behavior.

Unreceived readings are internally unavailable (`None`). Existing SSE represents unavailable attitude and sample age as null and omits absent optional instrument fields. State, source, and sequence remain present. A received zero is a real observation subject to normal validation and freshness.

### Lifecycle and faults

Timestamped records use `at_us` as a pass-relative offset. `pause` carries `mode` (`off`, `normal`, or `active`); `flight` carries `active`; `flight_loaded` carries a string or null `flight`; and `disconnect` carries a positive `unavailable_ms`. Normal pause models the Escape menu with `Pause_EX1=8`, active pause uses `4`, and off uses `0`. Sim-running, loaded-flight identity, pause, and connection state remain distinct. A newly eligible attitude is required after resume, flight change, or reconnect. State queries return current `Sim` and `FlightLoaded`; subscription supplies initial pause knowledge.

The [SimConnect event reference](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/general/simconnect_subscribetosystemevent/) defines `Pause_EX1` as a bitmask: `1` is full pause, `2` is a legacy pause with sound, `4` is Active Pause, and `8` pauses the player simulation. The documented flags can be combined. Demo scenarios model the observed Escape-menu and Active Pause states, not every combination.

In one MSFS 2024 observation on 2026-11-08, simulator version `12.2.282174.999`, SIM_FRAME samples continued during both Active Pause and the Escape menu. During Active Pause, IAS changed while the observed attitude remained constant. During the Escape menu, the observed attitude and IAS remained constant. The observed `Pause_EX1` flags were `4` for Active Pause and `8` for the Escape menu; the latter does not establish behavior for full pause (`1`). These observations do not establish behavior for every flight or simulator version.

The application's paused publication gate is separate from native sample delivery. The scenario clock continues; baseline scenarios author holds across pause intervals rather than silently freezing ramps.

Fault records are opt-in and absent from the healthy default. The table distinguishes the implemented subset from deferred vocabulary. Each record is a JSON object. Timestamped records use nonnegative integer `at_us` within the pass.

| `type` | Fields and meaning | Status |
| --- | --- | --- |
| `api_fault` | `operation`: exact export name string; `occurrence`: positive integer; `hresult`: signed 32-bit integer. Optional `selector` is an object with string `simvar` and/or `unit` | Implemented for Open, AddToDataDefinition, RequestDataOnSimObject, SubscribeToSystemEvent, RequestSystemState, GetLastSentPacketID, and Close. Selectors apply to definition and single-field request calls |
| `exception` | `at_us`; `sent_call_occurrence`: positive integer; `sent_operation`: export name string; `code` and `index`: unsigned integers. Correlate with a packet ID captured from a successful call | Implemented for AddToDataDefinition, RequestDataOnSimObject, SubscribeToSystemEvent, and RequestSystemState |
| `raw_packet` | `at_us`; `bytes_hex`: even-length hexadecimal string; `returned_length`: unsigned integer no larger than the decoded owned allocation | Implemented |
| `null_dispatch` | `at_us`; `returned_length`: unsigned integer. Return a null pointer without dereferencing it | Deferred |
| `disconnect` | `at_us`; `unavailable_ms`: positive integer. Emit QUIT, then fail Open until the interval expires | Implemented lifecycle record |
| `flight_loaded` | `at_us`; `flight`: string or null. Update queried state and subscribed transition | Implemented lifecycle record |
| `transport_silence` | `at_us`; `duration_ms`: positive integer. Suppress all dispatch packets, including state replies, without emitting QUIT | Implemented |

An `api_fault` occurrence counts matching calls since the scenario was loaded, including failed Open calls and reconnects. Selectors match the name and unit of the definition or single-field request, not a request ID. When several rules match one call, each matching counter advances and the first triggered rule supplies the result. The HRESULT must be negative and occurrence positive. Unknown operations and selectors fail scenario validation. Raw packet bytes are retained in DLL-owned storage, and the advertised length cannot exceed the decoded allocation.

Malformed JSON is a configuration error; a valid raw-packet instruction deliberately emits the authored native bytes. An exception that references an unavailable sent call reports a scenario error on standard error and terminates the Demo source without fabricating a packet ID. Terminal fault scenarios are separate files. Failed Close stops Demo's emitter but leaves native ownership retained for process exit. Transport silence suppresses state replies as well as samples; after the interval, subscribed state is supplied again. Null dispatch and host failure injection remain outside the implemented Demo vocabulary. Host failures require launch or harness fixtures, not invented native records.

### Selection and setup

The CLI selects `demo.dll` by default or a supported absolute DLL path. Basename matching is case insensitive, including `DEMO.dll`; export spelling remains exact. Bare supported names resolve beside the executable. Relative paths containing directories and unsupported basenames are rejected. A scenario path is required for Demo, accepted only for Demo, and resolved against the caller's working directory.

HTTP binds before Demo opens. On its first Open, the DLL reads `--scenario <path>` or `--scenario=<path>` from the process arguments and validates the complete file. Relative paths use the process working directory. The DLL caches the scenario and fault counters across retries and reconnects. The first successful Open starts the clock. Invalid or unreadable input produces a path and diagnostic on standard error and returns the Demo-specific customer HRESULT `0xA0040001`. The producer stops without retrying; HTTP and SSE remain available. There is no fallback scenario or worker thread on configuration failure. Rust panics must not cross the DLL ABI.

The customer HRESULT has this terminal meaning for any Demo Open result, including an injected fault with the same value. Demo also returns it from dispatch when an exception record references an unavailable sent call. Other injected Open failures remain retryable. The official SimConnect DLL is unaffected. The Demo DLL depends on the host process passing the scenario argument; another executable can use the DLL by passing the same argument.

### Forward scenario evidence

The [independent checkpoint fixture](../../tests/fixtures/demo-evidence.md) supplies a focused scenario and expected hold, ramp, gap, pause, resume, and later-pass observations. It supplies independent expectations for integrated checks. Record the scenario hash, application and DLL identities, run reference, and bounded window with each result.

| Authored native input | Receipt diagnostic | Eligible final SSE value or rule |
| --- | --- | --- |
| Pitch and bank, radians | `grouped_sample` records raw values and the validation error, if any | Negate both, convert to degrees, and wrap roll to `[-180, 180)`. Reject invalid pitch or nonfinite attitude. A grouped packet requires both components. |
| IAS, TAS, GS in knots; altitude in feet; VSI in feet per minute; turn rate in degrees per second | `grouped_sample` identifies the native name, index, value, and active gate | `ias_kt`, `tas_kt`, `gs_kt`, `altitude_ft`, `vertical_speed_fpm`, and `turn_rate_dps` retain valid native values. Speeds must be nonnegative. |
| Magnetic heading and selected heading, degrees | `grouped_sample` | Wrap to `[0, 360)`. Selected heading also requires a finite nonzero `AUTOPILOT AVAILABLE`. |
| Ball position | `grouped_sample` | Divide by 127 for `slip_skid`; reject values outside `[-1, 1]`. |
| KOHLSMAN setting, inHg | `grouped_sample` | Multiply by `33.8638866667` for `baro_setting.pressure_hpa`; the receiver also formats `baro_text` in inHg. Invalid pressure is omitted. |
| VS0, VS1, minimum rotation, and barber pole, knots | `grouped_sample` | Positive valid values populate the corresponding partial `airspeed_profile`; unsupported profile fields remain absent. |
| Autopilot availability, selected altitude, and radio mode | `grouped_sample` for each | `selected_altitude_ft` requires finite nonzero availability and radio mode equal to 0. Each gate and target must remain fresh. |
| IAS history | Grouped sample IAS values | `ias_acceleration_ktps` derives from valid IAS acquisitions and their receiver times over a 500 ms window; it is not an authored native field. |

A receipt is not proof of a final value: the worker logs raw observations even when lifecycle blocks publication. The `state_transition` record includes a published snapshot only on a transition, not for every instrument update. Capture SSE for other final checkpoints. SSE delivers latest snapshots and can coalesce publications, so do not assume a one-to-one packet count or exact wall-clock equality.

The complete sample expires after 1000 ms without a grouped packet. Equal packets refresh its common receipt time. Live output requires active, unpaused lifecycle and a fresh valid attitude. Pause, flight changes, and disconnect clear cached values; a new eligible sample is required for recovery. Invalid optional fields are omitted, while unavailable attitude is null. The [SimConnect guide](simconnect.md#telemetry-acquisition-and-availability) defines the current gates and normalization.

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
