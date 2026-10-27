# SimConnect integration

The SimConnect provider receives user-aircraft pitch and bank and records optional diagnostics.

## Requirements

Receive SimConnect data through one module and publish validated attitude to the application's existing telemetry channel.
Preserve the [telemetry contract](telemetry-contract.md#freshness-and-status), including unavailable states and the original sample age.
[#5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) records completed acquisition acceptance.
HTTP/SSE and EFI integration belong to [#6](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6).

Use Windows x64 MSVC with the [development environment](../CONTRIBUTING.md#environment-setup).
Obtain the official x64 `SimConnect.dll` separately.
The Rust build requires no SDK headers, import library, or installed simulator.
[ADR 0006](adr/0006-simconnect-manual-ffi.md#decision) identifies the inspected SDK declarations and binding strategy.

Keep SDK files, installation paths, and generated diagnostics out of Git.
DLL packaging and redistribution conditions belong to [#8](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8).

## Usage

### Run a session

1. Place the official `SimConnect.dll` beside the executable.
   For Cargo runs, use the applicable Cargo output directory.
2. From the repository root, start reception:

   ```powershell
   cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --log ".local\simconnect-sessions\flight.jsonl"
   ```

3. Check the console for the selected DLL and connection status.
   An active, unpaused flight produces `State: live` after a new sample meets the availability checks.
4. Press `Ctrl+C` to stop reception and close the connection.

Omit `--log` when diagnostic recording is unnecessary.
To select a DLL in another directory, provide its absolute path:

```powershell
$simconnectDll = Read-Host 'Absolute path to the official x64 SimConnect.dll'
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --dll "$simconnectDll" --log ".local\simconnect-sessions\flight.jsonl"
```

### Command-line options

The source controls which options are available:

| Option | Behavior |
| --- | --- |
| No source option or `--source demo` | Demo with HTTP/SSE, without loading SimConnect |
| `--source simconnect` | Native acquisition and local telemetry publication, without an HTTP server |
| `--dll <file>` | Optional absolute override. The default is `SimConnect.dll` beside the running EXE |
| `--log <file>` | Optional exact JSONL destination, absolute or relative to the working directory |
| `--help`, `--version` | Information without opening a DLL or log |
| `Ctrl+C` | Controlled shutdown that waits for the native worker |

`--dll` and `--log` are available only for SimConnect.
Explicit HTTP address and port options are rejected in this mode.
There is no `--simconnect` or `--scenario` option.
Use filenames to distinguish diagnostic scenarios. The application does not interpret those names as metadata.

### DLL selection and errors

The working directory does not affect the default DLL location.
Loading never searches `PATH`, the working directory, or SDK installations.
A failed explicit selection never falls back to another DLL.
Dependencies are searched only beside the selected DLL and in the Windows system directory.

Missing DLLs, incompatible architecture, and missing exports produce errors.
If the default DLL is missing, place it beside the executable or select an absolute path with `--dll`.
The console reports the selected DLL, connections, state changes, important errors, and shutdown, including without logging.

## Implementation

### Data and availability

The worker requests `PLANE PITCH DEGREES` and `PLANE BANK DEGREES` as `FLOAT64` radians once per simulator frame, without `CHANGED`.
Despite their names, the [SimVars' native unit](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simvars/aircraft-simvars/aircraft-miscellaneous-variables/) is radians.
The adapter negates both signs, converts locally to degrees, and wraps bank to the [telemetry range](telemetry-contract.md#fields-and-conventions):

| Output | Convention |
| --- | --- |
| `pitch_deg` | Finite degrees, positive nose up, in `[-90, 90]` |
| `roll_deg` | Finite degrees, positive right wing down, in `[-180, 180)` |

Invalid or nonfinite primary input is rejected rather than clamped.
Requested degree values and repeated radians are diagnostic only.
Disagreement or nonfinite probe values cannot invalidate otherwise valid primary radians.

Live publication requires running `Sim`, a loaded flight, known zero `Pause_EX1` flags, and a fresh valid sample.
No single event or nonempty filename establishes readiness.
The lifecycle uses these states:

| State | Meaning |
| --- | --- |
| `waiting` | Lifecycle information is incomplete or inactive, or a new sample is required |
| `live` | A valid sample is available from an active, unpaused flight |
| `paused` | Pause flags are nonzero, including unknown bits |
| `stale` | No new valid sample for 1000 ms while expecting live data |
| `disconnected` | The native connection is unavailable |
| `invalid` | Current input fails normalization or validation |

Pause, invalid data, disconnect, and flight or simulator transitions invalidate retained attitude.
Resume and a new flight require a new valid sample.
Every eligible acquisition publishes with its original receive time, including identical angles.

Cached values, heartbeats, and system-state queries never refresh age.
Freshness is evaluated every 50 ms. Unavailable publication carries no attitude.
Samples can continue to arrive during pause.
Ordinary pause and menu can share flags, so these signals do not uniquely identify the UI phase.

### Reception and resource ownership

One native worker owns all SimConnect calls.
It uses `GetNextDispatch`, Windows notification and cancellation events, and batches of at most 128 messages.
An empty queue or `E_FAIL` alone does not establish disconnection.

State queries run every 5 seconds.
A 15-second packet-silence watchdog can request reconnection, but does not prove native connection loss.
Unavailable connections retry after 1, 2, 4, 8, then at most 10 seconds.
The waits are interruptible. Connection loss never switches to demo.

Shutdown stops reception and closes the connection.
The worker releases notification handles and the DLL before the application joins the worker.
If native close fails, the process retains the DLL mapping and notification handle until exit because native code might still use them.
A cleanup record describes the attempted path. The final outcome and errors still matter.

### Diagnostic logging

Logs append to exactly the supplied file and create missing parent directories.
Existing data is preserved, and only one writer can use the file.
A partial trailing line is retained and separated from the next session.
The partial line is not a valid record.
The selected DLL and executable are rejected as log destinations.

Each record contains a unique `session_id` for the run, a per-session `record` number, and monotonic `elapsed_us`.
`started_unix_ms` provides the UTC reference. Some integers use strings to preserve precision.
Nonfinite numbers retain JSON-compatible value, classification, and bit-pattern fields.
Logs flush at significant events, at least once per second while recording, and at shutdown.

Failure to open the log or write its initial records aborts startup before DLL loading.
A later write or flush failure disables diagnostics while acquisition and cleanup continue.
The console warns immediately and at shutdown.
`DiagnosticStatus` distinguishes disabled, completed, and failed logging independently of the native result.

A missing final `session_end` leaves termination unconfirmed.
Without `--log`, the application creates no journal and skips diagnostic environment collection and raw-record formatting.

#### Payload profiles

The connection selects its profile when opened and retains it until closed:

| Profile | Fields in order | Payload bytes | Packet bytes |
| --- | --- | --- | --- |
| No logging | pitch radians, bank radians | 16 | 56 |
| Logging enabled | pitch radians, pitch degrees, bank radians, bank degrees, repeated pitch radians, repeated bank radians | 48 | 88 |

A later logging failure does not change the active definition.
A subsequent connection can use two fields.
The decoder validates the configured field count, flags, and payload bounds, then copies data into owned values.
Definition errors do not silently fall back to another profile.

The user-aircraft selector `SIMCONNECT_OBJECT_ID_USER_AIRCRAFT` is a proxy for requests.
Responses are correlated by request and definition IDs.
Their actual `object_id` need not equal the proxy and is not pinned across flights.
See the official [request API](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/events-and-data/simconnect_requestdataonsimobject/).

#### Record reference

The JSONL records describe these operations and values:

| Records | Content |
| --- | --- |
| `session_start`, `environment`, `dll_identity` | Session configuration, executable and DLL identities, runtime environment, and SDK metadata when available |
| `dispatch` | Message ID, size, and decoded native data. Open includes reported simulator and SimConnect versions |
| `connection_attempt`, `api_result`, `native_error` | Connection generation, API results, sent-packet IDs, and exception correlation |
| `attitude_definition` | Selected profile with two or six fields |
| `attitude_raw`, `attitude_sample` | Raw values, bits, and intervals before normalization and publication, then normalized attitude or validation error |
| `attitude_unit_check` | Diagnostic unit disagreement, changed bits in repeated radians, or nonfinite probes |
| `state_transition`, `connection_lost`, `reconnect_wait` | Lifecycle decision, published snapshot, reason, and reconnect timing |
| `dispatch_summary` | Aggregated packet and empty-dispatch counts, notification and timeout wakes, last HRESULT, and silence |
| `notification_release`, `library_release`, `session_end` | Cleanup attempts and final session outcome |

`attitude_raw.data.sample_index` is per connection.
`repeated_radians` holds the extra pitch and bank probes.
Unit comparisons retain a `0.00001`-degree tolerance, differences, bit equality, and bounds from repeated reads.
`affects_sample_validity: false` separates these observations from primary-radian validation.

Rejected attitude packets retain `message: "Unknown"` and `rejected_simobject_data`.
The latter contains received headers, expected request and definition IDs, and match results, without interpreting rejected payloads.
Other unsupported message IDs remain `Unknown`.

Schema 4 records `live_publication_enabled: true` and the bounded `validation_scope`.
The retained `candidate_state` and `candidate_attitude` names use `candidate_is_hypothesis: false`.
Historical schemas retain their original semantics.
Schemas 1–3 withheld candidate live, and schema 2 could reject cross-unit disagreements.

## Testing

Use the [validation scope matrix](testing/index.md#validation-by-change-scope) and the feature checks below.
The [automated commands](testing/automated.md#automated-checks) and [evidence procedure](testing/evidence.md) provide shared validation guidance.

### Automated checks

Simulator-free regressions cover the acquisition boundaries:

| Boundary | Coverage |
| --- | --- |
| Configuration | Source selection, DLL paths, CLI errors, demo independence, and no HTTP listener in SimConnect mode |
| Native data | ABI layout, payload decoding, normalization, and rejected packets |
| Telemetry | New and equal acquisitions, original receive time, stale data, invalid input, and lifecycle invalidation |
| Diagnostics and resources | Append sessions, writer exclusion, protected destinations, diagnostic failures, retry bounds, cancellation, and cleanup |

For the optional smoke check, set `$simconnectDll` to the official DLL's absolute path.
Close the simulator before running the check:

```powershell
$env:SIMCONNECT_TEST_DLL = $simconnectDll
cargo test --locked --test simconnect official_dll_stops_on_requested_shutdown_and_uses_caller_channel -- --ignored --nocapture
Remove-Item Env:SIMCONNECT_TEST_DLL
```

This checks loading, unavailable connection attempts, and requested shutdown after 750 ms.
It does not establish active-flight behavior. Rust layout tests alone do not prove SDK ABI compatibility.

### Repeatable simulator scenarios

Use existing evidence before requesting a manual run.
For a concrete change, contradiction, or additional claim, select only the affected scenario:

| Scenario | Expected behavior or observation |
| --- | --- |
| Startup and reconnect | Unavailable startup, menu and loading, fresh active data, simulator restart, and new samples after reconnect. No demo fallback |
| Attitude | Level, nose up and down, left and right bank, and combined poses against the cockpit |
| Pause and flight change | Unavailable attitude during ordinary pause, Active Pause, and menu. New samples restore availability after resume or another flight |
| Unit diagnostics | Valid primary radians remain valid despite diagnostic discrepancies. Retain surrounding anomalous packets |
| CPU or cancellation | Inspect the affected stable phase with or without logging, or investigate the specific shutdown failure |

Retain the build, source, artifact, and version identities, log and session IDs, and independent simulator actions relevant to the question.
Keep observations and raw logs locally. Log values alone do not identify physical maneuvers or UI phases.

Runtime Git and compiler metadata cannot establish which source and compiler produced an existing binary.
Use [build provenance](testing/evidence.md#build-provenance).

Process CPU time divided by elapsed wall time measures consumption relative to one logical CPU.
State the normalization when reporting percentages across all logical CPUs.

Console observations without logging support connection, availability, and shutdown behavior.
They do not establish numerical attitude agreement.

### Live-publication evidence

The [completed acceptance record in #5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) contains tested versions, session results, and historical corrections.
[ADR 0006](adr/0006-simconnect-manual-ffi.md#live-publication-amendment-2026-10-27) explains the decision to enable live publication.

## Rationale and regression lessons

The adapter uses native radians because independent requested units do not guarantee simultaneous acquisition.
Degree and repeated-radian probes remain diagnostic. The cause of these diagnostic discrepancies remains unresolved.
The [ADR](adr/0006-simconnect-manual-ffi.md) records binding, ownership, and publication decisions.

The [acceptance record](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) states the limits of the completed validation:

* Validation covers the tested setup. It does not certify every aircraft, UI phase, or simulator and DLL version.
* Precise cockpit angles and independently timed cockpit readiness remain unverified.
* The publication change has automated regression coverage. Earlier physical sessions used the provisional waiting gate.
* Short active CPU observations are not a benchmark. Menu CPU is deferred, and no numerical shutdown latency is claimed.
* The application was not tested on a clean machine without the SDK. Interruption during a blocked native call was not tested. Operating-system unload was not independently verified.
