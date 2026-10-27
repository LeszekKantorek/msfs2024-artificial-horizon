# SimConnect integration and diagnostics

SimConnect mode receives user-aircraft pitch and bank and publishes through the application's telemetry channel.
Select it with `--source simconnect`; it starts no HTTP server or browser display.
[#6](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6) connects this same module to HTTP/SSE and EFI.
Completed validation and historical results belong to [#5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5); design decisions belong to [ADR 0006](adr/0006-simconnect-manual-ffi.md).

## Local requirements

Use Windows x64 MSVC with the [development environment](../CONTRIBUTING.md#environment-setup).
Obtain the official x64 `SimConnect.dll` separately.
The Rust build requires no SDK headers, import library or installed simulator; [ADR 0006](adr/0006-simconnect-manual-ffi.md#decision) identifies the inspected SDK declarations and binding strategy.
Keep SDK files, installation paths and generated diagnostics out of Git.
DLL packaging and redistribution conditions remain in [#8](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8).

## Run a session

With `SimConnect.dll` beside the executable, run from the repository root:

```powershell
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --log ".local\simconnect-sessions\flight.jsonl"
```

For Cargo runs, place the DLL beside the EXE in the applicable Cargo output directory.
The working directory does not affect DLL selection.
To override the location:

```powershell
$simconnectDll = Read-Host 'Absolute path to the official x64 SimConnect.dll'
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --dll "$simconnectDll" --log ".local\simconnect-sessions\flight.jsonl"
```

Omit `--log` when diagnostic recording is unnecessary.

| Option | Behavior |
| --- | --- |
| No source option or `--source demo` | Demo with HTTP/SSE; never loads SimConnect |
| `--source simconnect` | Native acquisition and local telemetry publication |
| `--dll <file>` | Optional absolute override; default is `SimConnect.dll` beside the running EXE |
| `--log <file>` | Optional exact JSONL destination, absolute or relative to the working directory |
| `--help`, `--version` | Show information without opening a DLL or log |
| `Ctrl+C` | Request shutdown and wait for the native worker |

`--dll` and `--log` are available only for SimConnect.
Explicit HTTP address/port options are rejected in this mode.
There is no `--simconnect` or `--scenario` option; distinguish diagnostic scenarios by log filenames, which are not interpreted as metadata.

DLL loading never searches `PATH`, the working directory or SDK installations.
A failed explicit selection never falls back to another DLL.
Dependencies are searched only beside the selected DLL and in the Windows system directory.
Missing DLLs, wrong architecture and missing exports produce controlled errors.
Console messages report the selected DLL, connections, state transitions, important errors and shutdown, including without logging.

## Data and availability

The worker requests `PLANE PITCH DEGREES` and `PLANE BANK DEGREES` as `FLOAT64` radians once per simulator frame, without `CHANGED`.
Despite their names, the [SimVars' native unit](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simvars/aircraft-simvars/aircraft-miscellaneous-variables/) is radians.
The adapter converts locally to the [telemetry conventions](telemetry-contract.md#fields-and-conventions):

| Output | Convention |
| --- | --- |
| `pitch_deg` | Finite degrees, positive nose up, in `[-90, 90]` |
| `roll_deg` | Finite degrees, positive right wing down, in `[-180, 180)` |

Both native signs are negated; bank is wrapped into the output range.
Invalid or nonfinite input is rejected rather than clamped.
Requested degree values and repeated radians are diagnostic only: disagreement or nonfinite probe values cannot invalidate otherwise valid primary radians.
The SDK does not establish atomic acquisition across requested units or repeated fields.

Live publication requires running `Sim`, a nonempty loaded flight, known zero `Pause_EX1` flags and a fresh valid sample.
No single event or filename establishes readiness.
State handling follows the [freshness contract](telemetry-contract.md#freshness-and-status):

| State | Meaning |
| --- | --- |
| `waiting` | Lifecycle information is incomplete/inactive, or a new sample is required |
| `live` | A valid sample is available from an active, unpaused flight |
| `paused` | Pause flags are nonzero, including unknown bits |
| `stale` | No new valid sample for 1000 ms while expecting live data |
| `disconnected` | The native connection is unavailable |
| `invalid` | Current input fails normalization or validation |

Pause, invalid data, disconnect and flight/simulator transitions invalidate retained attitude.
Resume and a new flight require a new valid sample.
Every eligible acquisition publishes, including identical angles, with its original receive time.
Cached values, heartbeats and system-state queries never refresh age.
Freshness is evaluated every 50 ms; unavailable publication carries no attitude.
Sample reception may continue while paused.
Ordinary pause and menu can share flags, so these signals are not a unique UI-phase classifier.

One native worker owns all SimConnect calls.
It uses `GetNextDispatch`, Windows notification and cancellation events, and batches of at most 128 messages.
An empty queue or `E_FAIL` alone does not establish disconnection.
State queries run every 5 seconds; a 15-second packet-silence watchdog can request reconnection but does not prove native connection loss.
Unavailable connections retry after 1, 2, 4, 8, then at most 10 seconds; waits are interruptible.
Connection loss never switches to demo.

Shutdown stops reception, closes the connection, releases notification handles and the DLL, then joins the worker.
A failed native close retains the DLL mapping and notification handle until process exit to avoid releasing resources still in native use.
A cleanup record describes the attempted path; the final outcome and errors still matter.

## Diagnostic logging

Logs append to exactly the supplied file, creating missing parent directories.
Existing data is preserved, and only one writer may use the file.
A partial trailing line is retained and separated from the next session; it is not a valid record.
The selected DLL and executable are rejected as log destinations.

Each record contains a unique per-run `session_id`, a per-session `record` number and monotonic `elapsed_us`.
`started_unix_ms` provides the UTC reference; some integers use strings to preserve precision.
Nonfinite numbers retain JSON-compatible value, classification and bit-pattern fields.
Logs flush at significant events, at least once per second while recording, and at shutdown.

Failure to open the log or write its initial records aborts before DLL loading.
A later write/flush failure disables diagnostics while acquisition and cleanup continue.
The console warns immediately and at shutdown; `DiagnosticStatus` distinguishes disabled, completed and failed logging independently of the native result.
A missing final `session_end` leaves termination unconfirmed.
Without `--log`, the application creates no journal and avoids diagnostic environment collection and raw-record formatting.

### Payload profiles

The connection selects its profile when opened and retains it until closed:

| Profile | Fields in order | Payload / complete packet |
| --- | --- | --- |
| No logging | pitch radians, bank radians | 16 / 56 bytes |
| Logging enabled | pitch radians, pitch degrees, bank radians, bank degrees, repeated pitch radians, repeated bank radians | 48 / 88 bytes |

A later logging failure does not change the active definition; a subsequent connection can use two fields.
The decoder validates the configured field count, flags and payload bounds and copies into owned values.
Definition errors do not silently fall back to another profile.

The user-aircraft selector `SIMCONNECT_OBJECT_ID_USER_AIRCRAFT` is a request-side proxy.
Responses are correlated by request/definition IDs; their actual `object_id` need not equal that proxy and is not pinned across flights.
See the official [request API](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/events-and-data/simconnect_requestdataonsimobject/).

### Record reference

| Records | Content |
| --- | --- |
| `session_start`, `environment`, `dll_identity` | Session/configuration, executable/DLL identities, runtime environment and SDK metadata when available |
| `dispatch` | Message ID/size and decoded native data; Open includes reported simulator/SimConnect versions |
| `connection_attempt`, `api_result`, `native_error` | Connection generation, API results, sent-packet IDs and exception correlation |
| `attitude_definition` | Selected two- or six-field profile |
| `attitude_raw`, `attitude_sample` | Raw values/bits and intervals before normalization/publication, then normalized attitude or validation error |
| `attitude_unit_check` | Diagnostic unit disagreement, changed repeated-radian bits or nonfinite probes |
| `state_transition`, `connection_lost`, `reconnect_wait` | Lifecycle decision, published snapshot, reason and reconnect timing |
| `dispatch_summary` | Aggregated packet/empty-dispatch counts, notification/timeout wakes, last HRESULT and silence |
| `notification_release`, `library_release`, `session_end` | Cleanup attempts and final session outcome |

`attitude_raw.data.sample_index` is per connection; `repeated_radians` holds the extra pitch/bank probes.
Unit comparisons retain a `0.00001`-degree tolerance, differences, bit equality and repeated-read bounds.
`affects_sample_validity: false` separates these observations from primary-radian validation; their producer-side cause remains unresolved.
Rejected attitude packets retain `message: "Unknown"` and `rejected_simobject_data` with received headers, expected request/definition IDs and match results, without interpreting rejected payloads.
Other unsupported message IDs remain `Unknown`.

Schema 4 records `live_publication_enabled: true` and the bounded `validation_scope`.
The retained `candidate_state` and `candidate_attitude` names use `candidate_is_hypothesis: false`.
Historical schemas retain their original semantics: schemas 1–3 withheld candidate live, and schema 2 could reject cross-unit disagreements.
The [ADR amendment](adr/0006-simconnect-manual-ffi.md#live-publication-amendment-2026-10-27) records the gate-release rationale.

## Live-publication evidence

[#5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) owns the completed acceptance summary, tested versions, session results and historical corrections.
[ADR 0006](adr/0006-simconnect-manual-ffi.md#live-publication-amendment-2026-10-27) records why that evidence supports live publication with the current lifecycle, validity and freshness checks.

The evidence is bounded to the tested setup, not every aircraft, UI phase or simulator/DLL version.
Precise cockpit angles and independently timed cockpit readiness are not certified.
The publication change has automated regression coverage; earlier physical sessions used the provisional waiting gate.
Short active CPU observations are not a benchmark; menu CPU is deferred and no numerical shutdown latency is claimed.
A clean SDK-free machine, arbitrary blocked-native-call interruption and independent OS unload verification were not separately tested.
HTTP/SSE, multiple clients and live mobile EFI remain #6; packaging remains #8.

## Repeatable simulator scenarios

Use existing evidence before requesting a manual run.
For a concrete change, contradiction or additional claim, select only the affected scenario:

| Scenario | Observe |
| --- | --- |
| Startup, flight and reconnect | Unavailable startup, menu/loading, fresh active data, simulator termination/restart and new samples after reconnect; no demo fallback |
| Attitude | Level, nose up/down, left/right bank and combined poses against the cockpit; signs and any requested quantitative comparison |
| Pause and flight change | Ordinary pause, Active Pause, resume, menu exit and another flight; unavailable attitude and new-sample recovery despite continued packets |
| Unit diagnostics | Primary-radian validity, degree/repeat probes and surrounding anomalous packets; discrepancies cannot cause false invalids |
| CPU or cancellation | Only the affected stable phase, with/without logging as relevant; process CPU over measured wall time or the specific shutdown failure |

Retain the build/source/artifact identity, versions, log/session ID and independent simulator actions relevant to the question.
Keep observations and raw logs locally; log values alone do not identify physical maneuvers or UI phases.
Runtime Git/compiler metadata cannot establish which source/compiler produced an existing binary; use [build provenance](testing/evidence.md#build-provenance).
CPU-seconds divided by wall-seconds measures consumption relative to one logical CPU; state any whole-machine normalization.
For no-log runs, console observations support connection/availability/shutdown, not numerical attitude agreement.

Use the shared [automated checks](testing/automated.md#automated-checks) for simulator-free regression coverage.
An optional official-DLL cancellation smoke requires an explicitly selected DLL and a closed simulator:

```powershell
$env:SIMCONNECT_TEST_DLL = $simconnectDll
cargo test --locked --test simconnect official_dll_stops_on_requested_shutdown_and_uses_caller_channel -- --ignored --nocapture
Remove-Item Env:SIMCONNECT_TEST_DLL
```

This checks loading, unavailable connection attempts and requested shutdown after 750 ms.
It does not establish active-flight behavior, and Rust layout tests alone do not prove SDK ABI compatibility.
