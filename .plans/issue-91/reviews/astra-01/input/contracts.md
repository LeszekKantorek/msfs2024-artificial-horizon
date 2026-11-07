# Proposed scenario and native contracts

Status: Proposed technical contract for the revised Task #91. JSONL, hold/ramp timing, native-only data, and fault/lifecycle coverage follow explicit discussion decisions.

## JSONL meaning

Each nonempty physical line contains one complete JSON object.
The file contains many records. JSON strings escape embedded line breaks.
Reject blank lines, comments, trailing commas, duplicate keys, unknown fields, unsupported record types, and unsupported format/version identifiers.
Accept LF and CRLF. Write UTF-8 without a BOM.

Read and validate the complete scenario before opening a simulated connection.
JSONL does not require streaming playback or append-while-running behavior.
External file changes take effect on the next launch.

## Scenario suite and documentation

Support multiple independently selectable JSONL files through the existing explicit scenario option.
The main flight narrative includes only recoverable lifecycle/connection changes.
Isolate fatal native configuration, malformed packet, and cleanup cases in separate files.
Do not continue a narrative after a terminal producer failure merely to reach later phases.

The scenario catalog documents purpose, assumptions, expected completion, and file links only.
Keep timing, native values, and fault parameters in JSONL.
Keep independent acceptance checkpoints in tests/fixtures rather than deriving them from the reader under test.

## Header and time

The first record is a header:

```jsonl
{"type":"scenario","format":"msfs-efi-simconnect-demo","version":1,"mode":"scripted","frame_period_ms":50,"duration_us":15000000,"end":"repeat","initial":{"sim_running":true,"pause":"off","flight":"demo-flight.flt"}}
```

| Field | Contract |
| --- | --- |
| `mode` | `scripted` or `observations`; never mix authored tracks and recorded observations |
| `frame_period_ms` | Positive integer scripted frame cadence; default scenario uses 50 ms. Reject this field in observations mode |
| `duration_us` | Positive integer scenario duration |
| `end` | `repeat` or `wait`; converted recordings use `wait` |
| `initial` | Sim-running flag, pause mode, and fictional loaded-flight identity or null |
| Time arithmetic | Checked integer arithmetic; segment durations in ms convert exactly to internal microseconds |
| Ordering | Per-track segments follow file order. Equal-time events and observations preserve record order |
| Validation | Positive segment durations; all timestamped records precede scenario end; repeat periods must be explicit and coherent |

The exact resource caps belong to implementation, but must be finite and documented.
Reject oversized lines/files, excessive records, invalid numeric values, and overflowing schedules before acquisition.

## Scripted signal tracks

The author-facing signal key is a scenario alias for one independently documented native SimVar.
It is not the client's request ID or definition ID.

```jsonl
{"type":"signal","id":"IAS","simvar":"AIRSPEED INDICATED","unit":"knots","repeat":false}
{"type":"hold","signal":"IAS","value":100,"duration_ms":5000}
{"type":"ramp","signal":"IAS","from":100,"to":150,"duration_ms":5000}
{"type":"hold","signal":"IAS","value":150,"duration_ms":5000}
```

Signal tracks start together at scenario time zero.
Each segment begins when the preceding segment on that track ends.
This supports independent speed, attitude, heading, and altitude cycles without a global cross-product of phases.

| Record | Meaning |
| --- | --- |
| `hold` | Emit current value at every requested scripted frame during the duration |
| `ramp` | Interpolate linearly from `from` at the segment start toward `to` at its end |
| `gap` | Emit no observations for that signal during the duration |
| Signal `repeat: true` | Repeat its complete segment sequence; its period equals the segment-duration sum |
| Signal `repeat: false` | Emit nothing after its final segment |
| Scenario `end: repeat` | Restart the scenario timeline after the declared duration; keep application sequence continuity |
| Scenario `end: wait` | Stop scenario observations but keep state queries, receiver, expiry, HTTP, and shutdown available |

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

Headings use explicit unwrapped ramp endpoints when crossing north.
The existing receiver normalizes native headings. Add no general angular interpolation or expression language.

## Native field catalog

The DLL independently recognizes the following currently consumed field pairs.
It validates the actual name, index, unit, datatype, and supported request configuration.
ASCII case differences permitted by the SDK are not spelling errors.
Support only the required unit pairs, including the attitude diagnostic variants; do not invent a general unit converter.

| Native SimVar | Requested unit | Consumer meaning |
| --- | --- | --- |
| PLANE PITCH DEGREES | radians; degrees for diagnostics | Primary native pitch, with existing sign conversion |
| PLANE BANK DEGREES | radians; degrees for diagnostics | Primary native bank, with existing sign/wrap conversion |
| AIRSPEED INDICATED | knots | IAS and existing derived acceleration |
| AIRSPEED TRUE | knots | TAS |
| GROUND VELOCITY | knots | GS |
| INDICATED ALTITUDE:1 | feet | Barometric altitude |
| KOHLSMAN SETTING HG:1 | inHg | Numeric BARO after current conversion |
| VERTICAL SPEED | feet per minute | VSI |
| PLANE HEADING DEGREES MAGNETIC | degrees | Heading |
| TURN COORDINATOR BALL | position 128 | Ball, divided by 127 in the current consumer |
| TURN INDICATOR RATE | degrees per second | Turn rate |
| DESIGN SPEED VS0 | knots | Optional VS0 |
| DESIGN SPEED VS1 | knots | Optional VS1 |
| DESIGN SPEED MIN ROTATION | knots | Optional VR reference |
| AIRSPEED BARBER POLE | knots | Independent dynamic redline |
| AUTOPILOT AVAILABLE | bool, FLOAT64 representation | Target availability gate |
| AUTOPILOT HEADING LOCK DIR:0 | degrees | Selected heading |
| AUTOPILOT ALTITUDE LOCK VAR:0 | feet | Selected altitude |
| AUTOPILOT ALT RADIO MODE | bool, FLOAT64 representation | Barometric/radio target gate |

The index spellings in this table reflect current code, not universal SDK index rules.
Check each index against the authoritative per-variable contract before treating it as independently verified.
Do not copy a questionable consumer spelling into the emulator merely to make the application pass.

The fictional full VFE/VNO/VNE and VX/VY/VG profile is not generated through private SimVars.
Keep independent browser/presentation fixtures for those existing supported display fields.
Do not remove display capability because the Demo source becomes narrower.

## Flight and pause records

Use timestamped records on the same scenario clock:

```jsonl
{"type":"pause","at_us":2000000,"mode":"normal"}
{"type":"pause","at_us":4000000,"mode":"off"}
{"type":"pause","at_us":6000000,"mode":"active"}
{"type":"pause","at_us":8000000,"mode":"off"}
{"type":"flight","at_us":12000000,"active":false}
{"type":"flight","at_us":15000000,"active":true}
```

This example requires a scenario duration beyond 15 seconds.
It illustrates records rather than a complete file.

| Scenario condition | Native representation and expected receiver result |
| --- | --- |
| Normal pause | `Pause_EX1` flag 1; existing receiver reports paused |
| Active pause | `Pause_EX1` flag 4; existing receiver reports paused |
| Pause off | Flag 0; recovery still needs a new eligible attitude |
| Flight inactive | `Sim` becomes 0; retain loaded-flight identity but suppress live acquisition |
| Flight active | `Sim` becomes 1; a loaded flight, known pause-off state, and new eligible sample remain necessary |
| No loaded flight | Initial null flight or an explicit `flight_loaded` record with null identity; state query supplies no loaded-flight identity |
| Flight change | `flight_loaded` with another fictional identity; emit the subscribed change event and reset acquisition eligibility |
| Disconnect | A `disconnect` record produces QUIT and a scenario connection-unavailable interval |
| Reconnect | At the interval end, Open can succeed; return current state using fresh registrations and observations |

System-state queries return current `Sim` and `FlightLoaded` values, not only the last emitted event.
The initial Pause_EX1 response is required to establish pause knowledge.
Keep client-supplied event IDs and query IDs.

Do not change the current policy that either pause mode suppresses live telemetry.
Data arriving during pause must not authorize recovery after resume.
Use a separate fault fixture to send such data deliberately.

## Recorded observations

This section proposes the raw-replay principles for S6.
Its final supported record contract and timing policy are not implementation-ready until S6 completes.
S3 implements scripted mode; it must reject observations mode actionably until the later integrated replay outcome implements it.

Observations mode records data once at its recorded relative time.
It uses the same parser, field catalog, scheduler, native encoder, and receiver.

```jsonl
{"type":"observation","at_us":125000,"values":[{"simvar":"PLANE PITCH DEGREES","unit":"radians","value":-0.1},{"simvar":"PLANE BANK DEGREES","unit":"radians","value":0.2}]}
```

A values group preserves a known acquisition grouping, such as the paired attitude input.
Do not infer a fresh paired attitude from independently cached components.
Missing optional observations create a gap, not an inherited continuously sampled value.

Normalized snapshots, derived acceleration, sample age, wire sequence, and `source` are not native scenario values.
The new receiver supplies those outputs.
Conversion cannot promise exact historical ages or SSE publication sequence.

Finite values use JSON numbers.
A narrowly typed raw-value object may preserve explicit IEEE-754 bit patterns for recorded nonfinite input and fault fixtures.
That object is native input, never a way to restore an invalid value to normalized live telemetry.

The revised converter mapping remains an investigation prerequisite.
Do not implement conversion by inverting normalized FlightSample values and calling the result a raw recording.

## Fault injection

Faults are explicit scenario records. They are not enabled in the default healthy Demo.

The proposed record vocabulary is deliberately finite:

| Record | Required fields and meaning |
| --- | --- |
| `api_fault` | `operation` exact export name, positive `occurrence`, signed 32-bit `hresult`; optional `selector` with `simvar` and/or `unit` for definition/request association |
| `exception` | `at_us`, positive `sent_call_occurrence`, `sent_operation`, unsigned `code`, and unsigned `index`; correlate against that actual call's captured packet ID |
| `raw_packet` | `at_us`, even-length `bytes_hex`, and `returned_length`; returned length cannot exceed the owned decoded byte allocation |
| `null_dispatch` | `at_us` and unsigned `returned_length`; return a null pointer with no dereference to exercise the receiver's initial invariant check |
| `disconnect` | `at_us` and positive `unavailable_ms`; emit QUIT, then fail Open until the interval expires |
| `flight_loaded` | `at_us` and `flight` string or null; update queried state and any subscribed transition |
| `transport_silence` | `at_us` and positive `duration_ms`; suppress all dispatch packets, including system-state replies, without emitting QUIT |

Field selectors resolve through registered definitions instead of hard-coded request IDs.
Rule occurrence means the nth matching call since Configure, before its injected result is applied.
An exception referencing a failed or unavailable sent call is a scenario error, not a fabricated packet ID.
Malformed JSONL is rejected during setup; malformed native bytes are an intentional runtime fixture.

| Fault family | Proposed controls | Expected check |
| --- | --- | --- |
| Synchronous API failure | Export name, one-based matching call occurrence, optional whitelisted argument selector, HRESULT | Correct source failure, retry, or optional-field isolation |
| Asynchronous exception | Correlation with a captured sent-packet ID, exception code, parameter index | Optional request failure stays isolated; uncorrelated exceptions follow current fatal behavior |
| Dispatch result | Empty E_FAIL or another selected failure | Empty queue does not imply disconnect; other native failures follow reconnect policy |
| Packet structure | Explicit owned byte buffer, advertised length, header size/ID, request/definition/count corruption | Decoder rejects unsafe shape or ignores unsupported messages under its documented behavior |
| Data values | Nonfinite/out-of-range primary or optional values | Invalid primary suppresses live; invalid optional field does not remove independent data |
| Cleanup failure | Close HRESULT failure | Existing retain-mapping and notification ownership remain safe until process exit |
| Signal silence | Scheduled gap and resumed new data | Stale timing and fresh-sample recovery |
| Connection loss | QUIT plus finite unavailable interval | Retry and recovery without replaying the same disconnect forever |

Call occurrence counters last from scenario configuration until unload, including failed Open and reconnect attempts.
Connection-scoped rule counters are permitted only when explicitly selected by a rule.
Scenario restart must not accidentally rearm a startup-only failure on every retry.

Never return an invalid non-null pointer to unreadable memory.
Advertised readable length must not exceed the owned allocation.
Use internal header/count corruption to exercise rejection safely.
Null-pointer invariant tests may return null explicitly; they must not dereference it.

Injected Close failure stops Demo's own emitter before returning failure.
The consumer still retains its mapping/handle conservatively.
Test that case in a child process so deliberate retained resources do not pollute other cases.

Do not add permanently blocked calls, executable scripting, arbitrary callbacks, or a general packet language.

Transport silence differs from a signal gap.
System-state replies can keep the existing watchdog alive when ordinary flight data is absent.
During transport silence, API calls keep their configured synchronous results and dispatch returns the normal empty-queue result.
The interval uses the established scenario clock, persists across reconnects, and ends with current state plus new observations.

Initial connection unavailability uses Open call-occurrence failure before the playback origin.
The first later registered dispatch still starts scenario time once.
No second startup clock is required for that case.

Missing DLL/exports, journal I/O failures, host event failures, worker panic, and HTTP errors need launch/harness fixtures.
Do not claim a JSONL instruction controls host operations outside the Demo DLL.

## Scheduling and resource ownership

The DLL owns a monotonic clock, a small emitter thread, connection state, and its receive buffers.
It signals the notification handle passed to Open.
Successful Close stops and joins the emitter before releasing that handle's borrowed use.

### Clock origin and registration

Configure validates data but does not start scenario time.
Open creates a connection but does not advance scenario time before initial registration.
Start the timeline once, at the first GetNextDispatch invocation after at least one successful periodic data request.
The current consumer performs all initial optional registrations before that invocation.
The origin is established before applying any fault rule on that invocation.

Before that origin, subscriptions and queries receive the declared initial lifecycle values.
Scenario events and recorded observations have not elapsed or been discarded during initial registration.
The emitter may wait in a dormant state until the timeline starts.
Keep the established origin across reconnects; never restart it on every Open.

The scenario clock continues through pause and inactive-flight periods.
Those periods affect emitted native state and sample eligibility, not the duration accounting.
Default healthy emission suppresses primary samples while paused or inactive.
An explicit fault fixture may send them to exercise the receiver's conservative gate.

### Delivery and lateness

| Timing situation | Proposed behavior |
| --- | --- |
| Healthy scripted execution | Evaluate active tracks at absolute frame deadlines |
| Missed scripted frames | Skip obsolete data frames and evaluate the current value; refresh pending scripted values at dispatch so old encoded values do not become newly live |
| Lifecycle transitions | Preserve their order before delivering subsequent current values |
| Observation replay | Preserve ordered timestamps and gaps; emit each observation once |
| Excessively late recorded delivery | Stop replaying data and report unavailable/error; never silently replay stale observations as newly live |
| Delivery tolerance | Unresolved for observations mode; S6 must establish the policy before raw replay is implemented |
| Repeat | Rebase by declared duration, not by accumulated sleep intervals |
| Finite completion | Remain available for state queries and shutdown; no cached fresh samples |
| Reconnection | Keep scenario time and one-shot fault counters across connection generations; re-register the new handle against current state |

One retained proposal is a fixed 100 ms observation lateness budget.
Review found no evidence for that value and no established enqueue-versus-dispatch boundary.
It is not an S3/S4 acceptance requirement or the selected default behavior.
S6 must compare a dispatch-time cutoff with paced timeline suspension and define the chosen observation policy.
The comparison must preserve gaps/order without feeding old queued observations as newly fresh data.
Scripted timing is approximate. No hard real-time or exact native-timing claim is made.

The DLL must bound pending data and control records.
Overflow is an explicit source failure, not silent loss of lifecycle transitions.
Do not introduce a downstream telemetry history queue.

## DLL setup and common receiver

The official API subset consists of these exact exports:

* SimConnect_Open.
* SimConnect_Close.
* SimConnect_GetNextDispatch.
* SimConnect_AddToDataDefinition.
* SimConnect_RequestDataOnSimObject.
* SimConnect_SubscribeToSystemEvent.
* SimConnect_RequestSystemState.
* SimConnect_GetLastSentPacketID.

Use the current Windows x64 C ABI and header-derived packed receive layouts.
The Demo implementation must not import the application's production field table or normalization formulas.
The ABI declarations and test expectations need independent authoritative review.

Demo alone also exports a versioned setup call:

```c
int32_t EfiDemo_ConfigureV1(
    const uint16_t *scenario_path,
    uint8_t *error_utf8,
    uint32_t error_capacity);
```

* A null path selects the embedded default. Otherwise the path is absolute and NUL-terminated UTF-16.
* The caller supplies a nonempty writable error buffer. Failure writes a bounded NUL-terminated UTF-8 explanation.
* The call consumes/copies arguments synchronously and returns HRESULT. No pointers cross module ownership.
* Call setup after loading Demo and before Open. Setup validates the entire file without starting emission.
* Setup may occur only with no open connection. Rust panics never cross either DLL ABI.
* The official DLL is never asked for this export.
* Scenario failure stops that selected producer, leaves HTTP available, and never selects another scenario.

The private setup export exists only because the official eight functions have no scenario-path argument.
Do not repurpose the application name, environment variables, or current directory as hidden scenario configuration.

## Independent verification

The later implementation must prove these distinct claims:

* Every used export, name, unit, index, datatype, request option, diagnostic field shape, and lifecycle name is exercised.
* Changing a consumer spelling or unit alone makes an independent check fail.
* Changing caller-supplied IDs or field order changes returned routing and payload order correctly.
* Native values and normalized expected results come from independent fixtures, not the same conversion code.
* Real loader integration reaches HTTP/SSE and embedded EFI with source identity Demo.
* Emulator checks do not certify the official simulator, aircraft-specific availability, cockpit signs, or phone acceptance.
