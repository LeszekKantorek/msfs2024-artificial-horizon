# SimConnect integration

User-aircraft flight instrument reception with optional diagnostics.

## Requirements

| Requirement | Contract |
| --- | --- |
| Runtime | Windows x64 MSVC. Use the [development environment](../../CONTRIBUTING.md#environment-setup) |
| DLL | Official x64 `SimConnect.dll`, obtained separately |
| Build | No SDK headers, import library, or installed simulator required |
| Acquisition | One native worker in the provider module within the existing Rust package. Publish normalized telemetry through the caller-owned channel |
| Telemetry | Existing [states, validity, and freshness](../telemetry-contract.md#freshness-and-status) |
| Scope | Standard aircraft attitude and available optional instrument data through the same HTTP/SSE and browser contract as Demo. Each optional indication is independent |
| Distribution | Packaging and DLL redistribution conditions in [#8](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8) |
| Repository | No SDK files, installation paths, or generated diagnostics in Git |

> Runtime DLL loading does not establish redistribution permission.

## Usage

### Run a session

1. Place the official `SimConnect.dll` beside the executable.
   For Cargo runs, use the applicable Cargo output directory.
2. From the repository root, start the application:

   ```powershell
   cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --log ".local\simconnect-sessions\flight.jsonl"
   ```

3. Check the console for the selected DLL and connection status.
   An active, unpaused flight produces `State: live` after a new sample passes availability checks.
4. Open the printed HTTP URL. For phone access, use the [private LAN setup](../../README.md#use-your-phone) with `--source simconnect`.
5. Press `Ctrl+C` to close HTTP clients and wait for native reception to stop.

For a DLL in another directory, provide its absolute path:

```powershell
$simconnectDll = Read-Host 'Absolute path to the official x64 SimConnect.dll'
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --dll "$simconnectDll" --log ".local\simconnect-sessions\flight.jsonl"
```

### Command-line options

| Option | Behavior |
| --- | --- |
| No source option or `--source demo` | Demo with HTTP/SSE, without loading SimConnect |
| `--source simconnect` | Native acquisition with HTTP/SSE and available flight instruments |
| `--listen-address`, `--port` | HTTP settings for either source. Defaults are `127.0.0.1` and `8080` |
| `--dll <file>` | Optional absolute override. Default: `SimConnect.dll` beside the running EXE |
| `--log <file>` | Optional exact JSONL destination, absolute or relative to the working directory |
| `--help`, `--version` | Information without opening a DLL or log |
| `Ctrl+C` | Controlled shutdown that waits for the native worker |

The CLI applies these restrictions:

* `--dll` and `--log` require SimConnect.
* Invalid HTTP settings and a relative DLL override are rejected before starting components.
* `--simconnect` and `--scenario` are unsupported.
* Log filenames distinguish scenarios. The application does not interpret them as metadata.

### DLL selection and errors

| Property | Behavior |
| --- | --- |
| Default location | Beside the EXE, independent of the working directory |
| Search | No `PATH`, working-directory, or SDK search. No fallback after explicit selection |
| Dependencies | Selected DLL directory and Windows system directory only |
| Errors | Missing DLL, incompatible architecture, or missing exports stop the producer. HTTP remains available |
| Missing default DLL | Place it beside the EXE or select an absolute `--dll` path |
| Console | DLL selection, connections, state changes, errors, and shutdown, including without logging |

## Public API

The `providers::simconnect` module exports the acquisition interface independently of HTTP and process arguments.

| Interface | Contract |
| --- | --- |
| `SimConnectConfig` | Optional `dll: Option<PathBuf>` and `log: Option<PathBuf>`. Both default to `None` |
| `resolve_dll(override_path)` | Return the selected path or `SimConnectError`. Validate path syntax without loading the DLL |
| `run(config, publisher, shutdown)` | Consume a caller-supplied `Publisher` with `Source::SimConnect`. Await caller-owned shutdown with output `io::Result<()>` and return `RunReport` |
| `RunReport::result` | Acquisition outcome as `Result<(), SimConnectError>` |
| `RunReport::diagnostics` | Independent `DiagnosticStatus`: disabled, completed, or failed |
| `SimConnectError` | Configuration, I/O, native, telemetry, or worker-panic failure |

* The provider creates no hidden telemetry channel and never starts HTTP.
* Configuration contains no scenario, process arguments, or HTTP settings.
* DLL resolution uses the same executable-relative default for library and CLI calls, before opening diagnostics or loading native code.
* Dropping the running future signals worker cancellation. Normal shutdown also waits for the worker and reports cleanup results.
* A permanently blocked native call has no guaranteed completion deadline.

Application startup and source-failure isolation follow the [application lifecycle](../architecture.md#application-lifecycle).

## Implementation

* The provider uses minimal manual bindings and runtime DLL loading within the existing Rust package.
* [ADR 0006](../adr/0006-simconnect-manual-ffi.md) defines binding, loading, and publication decisions.
* [ADR 0007](../adr/0007-application-composition.md) defines composition, source-failure isolation, and shutdown.

### Application availability

The application validates configuration, creates the waiting telemetry channel, and binds HTTP before starting the producer.
A busy port prevents acquisition from starting. HTTP does not wait for DLL loading or simulator readiness.

| Condition | Application behavior |
| --- | --- |
| Simulator absent or disconnected | HTTP serves unavailable data while the existing provider retries |
| Producer exits or panics | Existing SSE streams stay open. New clients receive `disconnected` |
| DLL load or export failure | HTTP stays available. Console diagnostics identify the failure without Demo fallback |
| Ctrl+C or HTTP failure | Signal both components and wait for their results. HTTP failure returns an application error |

Detailed source errors stay in console diagnostics and the optional log. The SSE schema is unchanged.
`/health` reports HTTP liveness, independently of simulator availability.

### Optional instrument observations

The native worker requests these optional observations:

| Observation | Source units or selection |
| --- | --- |
| IAS, TAS, GS | Knots |
| KOHLSMAN pressure, indicated altitude | inHg and feet, respectively, from altimeter index 1 |
| VSI | Feet per minute |
| Magnetic heading | Degrees |
| Slip/skid ball position | Position |
| Turn rate | Degrees per second |
| Autopilot targets | Active slot 0. Target visibility does not require the autopilot master to be engaged. Radio-referenced altitude targets are omitted |

The cockpit control corresponding to altimeter index 1 depends on the aircraft and requires live verification.

Optional fields follow these availability rules:

* Each request has its own definition and exception boundary. Failed fields become unavailable without stopping attitude or HTTP.
* Last observations expire independently after 1000 ms. Pauses, flight changes, and reconnect clear cached values.
* Additional observations and expiry publications preserve the attitude receive time.

Speed profiles have these source limits:

* VS0, VS1, project minimum rotation, and current redline are independently available. Other thresholds and references remain absent unless their exact standard source is established.
* Minimum rotation is a project reference, not a Vr calculated by an FMS based on weight and flap position.
* General climb speed is not substituted for Vy. Current flap limits are not substituted for fixed VFE. Dynamic redline is not fixed VNE.
* No aircraft configuration reader or addon-specific avionics bridge is provided.

IAS trend uses a bounded 0.5-second window of valid worker acquisitions and their monotonic receive times. The trend describes knots per real second, including when simulation speed differs from real time.

* Acceleration expires after 0.5 seconds without a new IAS sample. IAS retains its independent one-second freshness.
* The history resets after invalid input, a stale gap, pause, or flight change.

### Data and availability

Attitude acquisition uses these conventions:

* Request `PLANE PITCH DEGREES` and `PLANE BANK DEGREES` as `FLOAT64` radians once per simulator frame, without `CHANGED`.
* Use the [native radian unit](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simvars/aircraft-simvars/aircraft-miscellaneous-variables/) despite the variable names.
* Negate both signs, convert locally to degrees, and wrap bank to the [telemetry range](../telemetry-contract.md#fields-and-conventions).
* Reject invalid or nonfinite primary input. Do not clamp it.
* Treat requested degrees and repeated radians as diagnostics. Disagreement or nonfinite probes cannot invalidate otherwise valid primary radians.

| Output | Convention |
| --- | --- |
| `pitch_deg` | Finite degrees, positive nose up, in `[-90, 90]` |
| `roll_deg` | Finite degrees, positive right wing down, in `[-180, 180)` |

Availability follows these rules:

* Live requires running `Sim`, a loaded flight, known zero `Pause_EX1` flags, and a fresh valid sample.
* No single event or nonempty filename establishes readiness.
* Pause, invalid data, disconnect, and flight or simulator transitions invalidate retained attitude.
* Resume and a new flight require a new valid sample.
* Every eligible acquisition publishes with its original receive time, including identical angles.
* Cached values, heartbeats, and system-state queries never refresh age.
* Freshness is evaluated every 50 ms. Unavailable publication carries no attitude.
* Samples can continue during pause. Ordinary pause and menu can share flags, so the flags do not uniquely identify the UI phase.

| State | Meaning |
| --- | --- |
| `waiting` | Incomplete or inactive lifecycle, or a new sample required |
| `live` | Valid sample from an active, unpaused flight |
| `paused` | Nonzero pause flags, including unknown bits |
| `stale` | No new valid sample for 1000 ms while expecting live data |
| `disconnected` | Native connection unavailable |
| `invalid` | Input fails normalization or validation |

### Reception and resource ownership

One native worker owns all SimConnect calls:

| Operation | Behavior |
| --- | --- |
| Dispatch | `GetNextDispatch`, Windows notification and cancellation events, at most 128 messages per batch |
| Empty queue | An empty dispatch or `E_FAIL` alone does not establish disconnection |
| State queries | Every 5 seconds |
| Silence watchdog | Reconnect after 15 seconds without packets. This diagnostic trigger does not prove native connection loss |
| Retry | Interruptible waits of 1, 2, 4, 8, then at most 10 seconds |
| Source loss | No automatic switch to demo |

Normal shutdown uses this order:

1. Stop reception.
2. Close the native connection.
3. Release notification handles and the DLL.
4. Join the worker.

Cleanup preserves these limits:

* A failed native close retains the DLL mapping and notification handle until exit because native code might still use them.
* Cleanup records describe attempted paths. Inspect the final outcome and errors.

### Diagnostic logging

| Property | Behavior |
| --- | --- |
| Destination | Append to the exact file, creating missing parent directories |
| Existing data | Preserve previous records and allow only one writer |
| Interrupted tail | Retain the partial line and separate it from the next session. The partial line is not a valid record |
| Protected files | Reject the selected DLL and executable as destinations |
| Record identity | Unique `session_id` per run and per-session `record` numbering |
| Time | Monotonic `elapsed_us`, UTC reference in `started_unix_ms`, strings for some integers to preserve precision |
| Nonfinite values | JSON-compatible value, classification, and bit-pattern fields |
| Flush | Significant events, at least once per second while recording, and shutdown |
| No logging | No journal, diagnostic environment collection, or raw-record formatting |

Diagnostic failures remain independent of acquisition:

| Condition | Result |
| --- | --- |
| Open or initial write failure | Abort producer startup before DLL loading. HTTP remains available |
| Later write or flush failure | Disable diagnostics, continue acquisition and cleanup, warn immediately and at shutdown |
| Missing `session_end` | Unconfirmed termination |
| Final result | `DiagnosticStatus` distinguishes disabled, completed, and failed logging independently of the native outcome |

#### Payload profiles

The connection fixes its profile when opened:

| Profile | Fields in order | Payload bytes | Packet bytes |
| --- | --- | --- | --- |
| No logging | pitch radians, bank radians | 16 | 56 |
| Logging enabled | pitch radians, pitch degrees, bank radians, bank degrees, repeated pitch radians, repeated bank radians | 48 | 88 |

Decoding preserves these rules:

* A later logging failure leaves the active definition unchanged. A subsequent connection can use two fields.
* The decoder validates the configured field count, flags, and payload bounds, then copies data into owned values.
* Definition errors do not silently select another profile.
* `SIMCONNECT_OBJECT_ID_USER_AIRCRAFT` is a proxy for requests.
* The decoder correlates responses by request and definition IDs. Their `object_id` need not equal the proxy or remain constant across flights.

Reference: [SimConnect request API](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/events-and-data/simconnect_requestdataonsimobject/).

#### Record reference

| Records | Content |
| --- | --- |
| `session_start`, `environment`, `dll_identity` | Session configuration, executable and DLL identities, runtime environment, available SDK metadata |
| `dispatch` | Message ID, size, decoded data. Open includes reported simulator and SimConnect versions |
| `connection_attempt`, `api_result`, `native_error` | Connection generation, API results, sent-packet IDs, exception correlation |
| `attitude_definition` | Selected profile with two or six fields |
| `attitude_raw`, `attitude_sample` | Raw values, bits, and intervals before normalization and publication, then normalized attitude or validation error |
| `attitude_unit_check` | Diagnostic unit disagreement, changed bits in repeated radians, nonfinite probes |
| `state_transition`, `connection_lost`, `reconnect_wait` | Lifecycle decision, published snapshot, reason, reconnect timing |
| `dispatch_summary` | Packet and empty-dispatch counts, notification and timeout wakes, last HRESULT, silence |
| `notification_release`, `library_release`, `session_end` | Cleanup attempts and final session outcome |

Interpret records with these constraints:

* `attitude_raw.data.sample_index` counts samples per connection.
* `repeated_radians` contains the extra pitch and bank probes.
* Unit comparisons retain a `0.00001`-degree tolerance, differences, bit equality, and bounds from repeated reads.
* `affects_sample_validity: false` separates diagnostic observations from primary-radian validation.
* Rejected attitudes retain `message: "Unknown"` and `rejected_simobject_data`. The latter contains received headers, expected request and definition IDs, and match results. Rejected payloads are not interpreted.
* Other unsupported message IDs remain `Unknown`.
* Schema 4 records `live_publication_enabled: true` and bounded `validation_scope`.
* Retained `candidate_state` and `candidate_attitude` names use `candidate_is_hypothesis: false`.
* Historical schemas 1–3 withheld candidate live. Schema 2 could reject cross-unit disagreements.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope), [automated commands](../testing/automated.md#automated-checks), and [evidence procedure](../testing/evidence.md).

### Automated checks

| Boundary | Coverage |
| --- | --- |
| Configuration | Source selection, DLL paths, CLI errors, Demo independence, HTTP settings for both sources |
| Application integration | Startup ordering, source-failure isolation, retained SSE, shutdown, multiple clients and attitude-only publication |
| Native data | ABI layout, payload decoding, normalization, rejected packets |
| Telemetry | New and equal acquisitions, original receive time, stale data, invalid input, lifecycle invalidation |
| Diagnostics and resources | Append sessions, writer exclusion, protected destinations, diagnostic failures, retry bounds, cancellation, cleanup |

Run the optional official-DLL smoke check:

1. Set `$simconnectDll` to the official DLL's absolute path.
2. Close the simulator.
3. Run the check:

   ```powershell
   $env:SIMCONNECT_TEST_DLL = $simconnectDll
   cargo test --locked --test simconnect official_dll_stops_on_requested_shutdown_and_uses_caller_channel -- --ignored --nocapture
   Remove-Item Env:SIMCONNECT_TEST_DLL
   ```

The smoke check has these limits:

* It covers loading, unavailable connection attempts, and requested shutdown after 750 ms.
* It does not establish active-flight behavior. Rust layout tests alone do not prove SDK ABI compatibility.

### Repeatable simulator scenarios

Select manual checks only for a concrete change, contradiction, or additional claim:

| Scenario | Expected behavior or observation |
| --- | --- |
| Startup and reconnect | Unavailable startup, menu and loading, fresh active data, simulator restart, new samples after reconnect. No demo fallback |
| Attitude | Level, nose up and down, left and right bank, combined poses against the cockpit |
| Pause and flight change | Unavailable attitude during ordinary pause, Active Pause, and menu. New samples after resume or another flight |
| Unit diagnostics | Valid primary radians despite diagnostic discrepancies. Retain surrounding anomalous packets |
| CPU or cancellation | Affected stable phase with or without logging, or the specific shutdown failure |

Use these evidence rules:

* Reuse existing results before requesting another run.
* Keep local logs, session IDs, versions, independent observations, and [build provenance](../testing/evidence.md#build-provenance). Runtime metadata alone does not establish a binary's source.
* Divide process CPU time by elapsed wall time for one logical CPU. State normalization across all logical CPUs.
* Use console observations without logging for connection, availability, and shutdown claims. Do not infer numerical angles.

### Integrated HTTP/SSE and device acceptance

Use the final release artifact and the [real-device procedure](../testing/devices.md#real-phone-acceptance-procedure).
Run the following scenarios separately on actual iOS Safari and Android Chrome.
Keep the receiver's #5 evidence separate from these end-to-end results.

| Scenario | Expected result |
| --- | --- |
| Missing DLL or exports | Page and health remain available. SSE reports unavailable SimConnect without Demo fallback |
| Level, nose up and down, left and right bank, combined poses | EFI orientation agrees with cockpit observations. Do not infer a precise angular accuracy claim |
| Ordinary pause, Active Pause, flight exit, restart | Unavailable attitude during loss. Recovery requires a new valid active sample |
| Two clients and a slow receiver | Both use one acquisition connection. A slow client does not delay the other or block reception |
| Producer termination after live data | Existing SSE stays open and old attitude clears. A new client receives the current unavailable state |
| Silence and invalid samples | `stale` or `invalid`, without frozen valid-looking attitude. Heartbeat does not renew freshness |
| Attitude-only live sample | Other EFI indications remain unavailable |
| Rotation, background, lock/resume, Wi-Fi recovery | Readable status and fresh-data recovery without reload or duplicate subscriptions |
| Ctrl+C with active clients | Client connections end and the port is reusable. Record native cleanup outcome separately |

Use controlled producers for deterministic fault tests without changing the production source choices.
Record the tested revision, binary and DLL identities, environment, exact inputs, observed behavior, and missing checks.
Desktop fixture tests cannot certify the real simulator path or either mobile browser.

### Live-publication evidence

The #5 records cover the receiver and local publication. They do not establish the integrated HTTP/SSE-to-EFI path or actual behavior on iOS and Android.
Integrated simulator and mobile acceptance require separate runs against the final application artifact.

Acceptance and rationale have separate sources:

* [#5 acceptance record](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5): tested environments and evidence limits.
* [ADR 0006](../adr/0006-simconnect-manual-ffi.md#live-publication-amendment): decision to enable live publication.

## Rationale and regression lessons

| Decision | Reason |
| --- | --- |
| Native radians | Independent requested units do not guarantee simultaneous acquisition |
| Diagnostic degree and repeat probes | No effect on canonical validity. The cause of discrepancies remains unresolved |
| Native bindings and ownership | Defined in [ADR 0006](../adr/0006-simconnect-manual-ffi.md) |

The [validation evidence](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) has these limits:

| Area | Evidence limit |
| --- | --- |
| Compatibility | Tested setup only, not every aircraft, UI phase, or simulator and DLL version |
| Cockpit comparison | No certification of precise angles or independently timed readiness |
| Live publication | Automated regression tests cover live publication. Physical sessions using the waiting gate do not establish behavior with live publication enabled |
| Performance | Short active CPU observations, not a benchmark. Menu CPU and numerical shutdown latency are unmeasured |
| Other environments and interruption | Untested: clean machine without SDK, interruption during a blocked native call, independent OS unload verification |
