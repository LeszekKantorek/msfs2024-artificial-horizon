# Proposed scenario and native contracts

Status: Proposed technical contract for the revised Task #91. JSONL, hold/ramp timing, native-only data, and fault/lifecycle coverage follow explicit discussion decisions.

The [post-review receive-path clarification](architecture.md#receive-path-clarification-after-the-astra-review) governs notification and buffer ownership.
Its fidelity requirements qualify the unverified pause, overflow, error-channel, and replay policies below.

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
{"type":"scenario","format":"msfs-efi-simconnect-demo","version":1,"mode":"scripted","frame_period_ms":50,"duration_us":15000000,"initial":{"sim_running":true,"pause":"off","flight":"demo-flight.flt"}}
```

| Field | Contract |
| --- | --- |
| `mode` | `scripted`; recorded-session playback is outside #91 |
| `frame_period_ms` | Positive integer duration of a synthetic simulation frame; default scenario uses 50 ms. Apply the client's request semantics to those frames |
| `duration_us` | Positive integer duration of one pass through the scenario steps |
| `initial` | Scenario-authored Sim-running flag, pause mode, and loaded-flight identity or null; applied once at playback initialization |
| Time arithmetic | Checked integer arithmetic; segment durations in ms convert exactly to internal microseconds |
| Ordering | Per-track segments follow file order. Equal-time events and observations preserve record order |
| Validation | Positive segment durations; all timestamped records fall within one pass; track periods must be explicit and coherent |

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

### Request-driven emission and unchanged values

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

## Native field catalog

The DLL independently recognizes the following currently consumed field pairs.
It validates the actual name, index, unit, datatype, and supported request configuration.
ASCII case differences permitted by the SDK are not spelling errors.
Support only the required unit pairs, including the attitude diagnostic variants; do not invent a general unit converter.

### Registration and sparse scenarios

Scenario completeness is not a requirement. A scenario may provide all signals, one signal, only lifecycle events, or no signal values.
Accept a supported registration even when the scenario has no value for that parameter.
Missing scenario values are not unknown SimVars or failed registrations.
Keep malformed names, unsupported definitions, and explicitly injected registration errors distinct from missing values.

Generate observations only from values supplied by the scenario and requested through the native API.
An IAS-only ramp updates IAS only. Do not add altitude, attitude, or other values to complete the scenario or satisfy UI readiness.
Do not synthesize zero values or reuse expired observations to fill missing data.
The existing grouped-packet rule still applies: missing components suppress that packet rather than changing its registered binary layout.
This does not invalidate the scenario or prevent independent requests and lifecycle events from being served.

No signal values means no signal-data packets. Ordinary API control replies remain governed by the SimConnect contract.
The explicit transport-silence fixture covers suppression of control replies as well.

### Default values and SSE representation

The default value of an unreceived instrument reading is null: no available value, represented internally by None.
Neither the server, UI, nor Demo substitutes zero for missing data.
A received zero remains a real numeric observation, subject to that field's validation and freshness rules.

Preserve the existing SSE representation:

* `attitude` and `sample_age_ms` are null when unavailable under their respective contracts.
* Absent optional instrument fields are omitted from the JSON snapshot. Omission and null both mean an unavailable reading to the client.
* `state`, `source`, and `sequence` remain present; they are metadata, not optional instrument readings.

The UI retains unavailable presentation until eligible data arrives. Do not retain expired readings or turn absence into a fresh sample.
This decision withdraws the earlier initial-zero preference and does not change the current lifecycle or attitude gates.

### Supported fields

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

Use pass-relative timestamped records scheduled on the continuously increasing playback clock:

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
| Flight inactive | `Sim` becomes 0; retain loaded-flight identity; the application's lifecycle gate suppresses live publication |
| Flight active | `Sim` becomes 1; a loaded flight, known pause-off state, and new eligible sample remain necessary |
| No loaded flight | Initial null flight or an explicit `flight_loaded` record with null identity; state query supplies no loaded-flight identity |
| Flight change | Explicit `flight_loaded` with another fictional identity; emit the subscribed change event. The application clears acquisition eligibility under its own rules |
| Disconnect | A `disconnect` record produces QUIT and a scenario connection-unavailable interval |
| Reconnect | At the interval end, Open can succeed; return current state using fresh registrations and observations |

System-state queries return current `Sim` and `FlightLoaded` values, not only the last emitted event.
The initial Pause_EX1 response is required to establish pause knowledge.
Keep client-supplied event IDs and query IDs.

Keep the application's existing paused-state publication gate. It does not establish that the native DLL stops sending packets.
Data arriving during pause must not authorize recovery after resume.

The user's working assumption is that normal and active pause continue sending messages and repeat frozen values until resume.
Verify the two pause modes separately against the target SimConnect runtime during implementation, including SIM_FRAME data, system events, state-query replies, and resume.
The [system-event reference](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/general/simconnect_subscribetosystemevent/) documents pause events, initial state notification, and PauseFrame, but does not establish all SimVar delivery behavior during either pause mode.
Paused data is not inherently a fault fixture. Adjust Demo to observed SDK behavior rather than suppressing packets to match the application's publication gate.
Keep the scenario clock advancing. Author explicit holds across pause intervals in baseline scenarios; do not silently pause a ramp's timeline or invent catch-up semantics. Resolve any automatic freeze/resume rule from implementation evidence before claiming parity.

## Scenario-to-output evidence

Only forward verification is required: scenario -> Demo DLL -> native receiver -> normalized result -> log/SSE.
Recorded-flight conversion, diagnostic schema 4/5 mapping, observations mode, and reconstruction of the original scenario are outside #91 by explicit user decision.

* Identify the exact scenario and application/DLL revision for each run. Use a file hash and run timing in the evidence record, without adding private fields to SimConnect packets.
* Relate a checked output to its input signal, segment and pass/time window. A ramp value can be calculated from its endpoints and elapsed time; it need not occur literally in the file.
* Distinguish direct values from documented sign/unit normalization, gates, expiry, and derived IAS acceleration. Keep expected checkpoints independent of the implementation under test.
* Identify whether a log record represents native receipt, normalization, or publication. A raw diagnostic value alone does not prove the final SSE value; capture SSE where the existing log lacks that output.
* Allow documented scheduling/receipt tolerances and latest-state coalescing. Do not require one SSE record for every native packet or preserve historical receipt timestamps.
* Include unchanged hold values, a ramp, missing data/gaps, pause/resume, and a second pass. Missing readings remain unavailable; unrelated updates do not refresh them.

S6 establishes this evidence mapping for S8. Routine checks remain in the implementation Subtasks. No new recording protocol, reverse converter, or alternate Publisher is required.

Finite scenario values use JSON numbers. Explicit native-invalid values in fault scenarios need a narrowly typed representation (such as IEEE-754 bits); do not accept invalid authoring as an ordinary signal value.
## Fault injection

Faults are explicit scenario records. They are not enabled in the default healthy Demo.
Coverage may be staged. Retain the inventory, but S5 must identify implemented cases and explicitly deferred rare API/exception/packet cases. The healthy Demo and flight narrative do not wait for exhaustive fault coverage.
Implemented cases must honor their documented API channel. An immediate HRESULT and a later SIMCONNECT_RECV_EXCEPTION are distinct mechanisms; neither is assumed to occur in ordinary healthy playback.

### Empty queue and dispatch result

The [GetNextDispatch reference](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/general/simconnect_getnextdispatch/) says that an empty queue sets the received message's dwID to SIMCONNECT_RECV_ID_NULL. This is a message type, not a null pointer.
The same reference lists S_OK for success and E_FAIL as generic failure. It does not enumerate E_FAIL conditions or specify the complete HRESULT/pointer/length combination for an empty queue.
The current Connection::next treats E_FAIL as no packet. This is an observed code policy, not proof that every SDK E_FAIL means an empty queue.
During implementation, establish the actual empty-queue result tuple and notification behavior against the target runtime; reproduce it in Demo and assess the common receiver if needed. A NULL message must not create false data freshness or an endless successful drain.
Do not use dispatch E_FAIL to report invalid scenario setup: the current receiver would interpret it as no packet. Report setup errors before Open through the private setup call.

### Fault vocabulary

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
| Dispatch result | Explicit HRESULT or empty-queue behavior verified against the target runtime | Distinguish current consumer handling from SDK guarantees; see the dispatch contract below |
| Packet structure | Explicit owned byte buffer, advertised length, header size/ID, request/definition/count corruption | Decoder rejects unsafe shape or ignores unsupported messages under its documented behavior |
| Data values | Nonfinite/out-of-range primary or optional values | Invalid primary suppresses live; invalid optional field does not remove independent data |
| Cleanup failure | Close HRESULT failure | Existing retain-mapping and notification ownership remain safe until process exit |
| Signal silence | Scheduled gap and resumed new data | Stale timing and fresh-sample recovery |
| Connection loss | QUIT plus finite unavailable interval | Retry and recovery without replaying the same disconnect forever |

Call occurrence counters last from scenario configuration until unload, including failed Open and reconnect attempts.
Connection-scoped rule counters are permitted only when explicitly selected by a rule.
Looping the steps or reconnecting must not rearm a startup-only failure by resetting its call-occurrence counter.

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
The first later successful Open starts scenario time once.
No second startup clock is required for that case.

Missing DLL/exports, journal I/O failures, host event failures, worker panic, and HTTP errors need launch/harness fixtures.
Do not claim a JSONL instruction controls host operations outside the Demo DLL.

## Scheduling and resource ownership

The DLL owns a monotonic clock, a small emitter thread, connection state, and its receive buffers.
It signals the notification handle passed to Open.
Successful Close stops and joins the emitter before releasing that handle's borrowed use.

### Clock origin and registration

Configure validates data but does not start scenario time.
Start the timeline once at the first successful Open, independently of GetNextDispatch.
Registration overlaps scenario time. Only registered requests receive generated observations.
An authored initial hold can provide setup time without coupling the DLL to the application's registration sequence.

Before that origin, subscriptions and queries receive the declared initial lifecycle values.
Events can elapse during registration. The DLL must not conceal late subscription or slow consumption.
Record startup/registration timing when relating scenario values to output evidence.
Keep the established origin across reconnects; never restart it on every Open.

The scenario clock continues through pause, inactive-flight periods, and every pass boundary.
Those periods affect emitted native state and sample eligibility, not the duration accounting.
Native delivery during pause/inactive flight follows the verified SimConnect contract; do not infer it from the application's publication gate. The pause assumption and verification boundary are stated above.

### Delivery and lateness

| Timing situation | Proposed behavior |
| --- | --- |
| Healthy scripted execution | Evaluate active tracks at absolute frame deadlines |
| Missed scripted frames | Distinguish scheduler lateness from slow consumption; never rewrite an already queued packet at dispatch. Verify missed-frame and overflow policy against the target runtime |
| Lifecycle transitions | Preserve their order before delivering subsequent current values |
| Last step | Schedule the first step of the next pass at its later absolute deadline; retain the clock, lifecycle state, and call-occurrence counters |
| Reconnection | Keep scenario time and one-shot fault counters across connection generations; re-register the new handle against current state |

Scripted timing is approximate. No hard real-time or exact native-timing claim is made.

The DLL must bound pending data and control records.
An explicit source failure on overflow remains a proposal, not verified SDK behavior. Establish the target runtime behavior during implementation before selecting a compatible policy; document any limit instead of silently dropping lifecycle transitions.
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
* Bind HTTP first, then load Demo and call setup before Open. Setup validates the entire file without starting emission. Invalid/unreadable scenarios return E_FAIL with an actionable bounded error explanation; stop this producer while HTTP remains available. This E_FAIL belongs to private setup, not GetNextDispatch.
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
