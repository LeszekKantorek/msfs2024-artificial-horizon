# SimConnect integration

User-aircraft pitch and bank reception with optional diagnostics.

## Requirements

| Requirement | Contract |
| --- | --- |
| Runtime | Windows x64 MSVC. Use the [development environment](../CONTRIBUTING.md#environment-setup) |
| DLL | Official x64 `SimConnect.dll`, obtained separately |
| Build | No SDK headers, import library, or installed simulator required |
| Acquisition | One module and the caller-owned telemetry channel |
| Telemetry | Existing [states, validity, and freshness](telemetry-contract.md#freshness-and-status) |
| Scope | Local publication in #5. HTTP/SSE and EFI integration in [#6](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6) |
| Distribution | Packaging and DLL redistribution conditions in [#8](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8) |
| Repository | No SDK files, installation paths, or generated diagnostics in Git |

## Usage

### Run a session

1. Place the official `SimConnect.dll` beside the executable.
   For Cargo runs, use the applicable Cargo output directory.
2. From the repository root, start reception:

   ```powershell
   cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --log ".local\simconnect-sessions\flight.jsonl"
   ```

3. Check the console for the selected DLL and connection status.
   An active, unpaused flight produces `State: live` after a new sample passes availability checks.
4. Press `Ctrl+C` to stop reception and close the connection.

For a DLL in another directory, provide its absolute path:

```powershell
$simconnectDll = Read-Host 'Absolute path to the official x64 SimConnect.dll'
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --dll "$simconnectDll" --log ".local\simconnect-sessions\flight.jsonl"
```

### Command-line options

| Option | Behavior |
| --- | --- |
| No source option or `--source demo` | Demo with HTTP/SSE, without loading SimConnect |
| `--source simconnect` | Native acquisition and local publication, without HTTP |
| `--dll <file>` | Optional absolute override. Default: `SimConnect.dll` beside the running EXE |
| `--log <file>` | Optional exact JSONL destination, absolute or relative to the working directory |
| `--help`, `--version` | Information without opening a DLL or log |
| `Ctrl+C` | Controlled shutdown that waits for the native worker |

The CLI applies these restrictions:

* `--dll` and `--log` require SimConnect.
* Explicit HTTP address and port options are rejected in SimConnect mode.
* `--simconnect` and `--scenario` are unsupported.
* Log filenames distinguish scenarios. The application does not interpret them as metadata.

### DLL selection and errors

| Property | Behavior |
| --- | --- |
| Default location | Beside the EXE, independent of the working directory |
| Search | No `PATH`, working-directory, or SDK search. No fallback after explicit selection |
| Dependencies | Selected DLL directory and Windows system directory only |
| Errors | Missing DLL, incompatible architecture, or missing exports |
| Missing default DLL | Place it beside the EXE or select an absolute `--dll` path |
| Console | DLL selection, connections, state changes, errors, and shutdown, including without logging |

## Implementation

### Data and availability

Attitude acquisition uses these conventions:

* Request `PLANE PITCH DEGREES` and `PLANE BANK DEGREES` as `FLOAT64` radians once per simulator frame, without `CHANGED`.
* Use the [native radian unit](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simvars/aircraft-simvars/aircraft-miscellaneous-variables/) despite the variable names.
* Negate both signs, convert locally to degrees, and wrap bank to the [telemetry range](telemetry-contract.md#fields-and-conventions).
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
| Open or initial write failure | Abort startup before DLL loading |
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
* Rejected attitudes retain `message: "Unknown"` and `rejected_simobject_data`: received headers, expected request and definition IDs, and match results, without interpreting rejected payloads.
* Other unsupported message IDs remain `Unknown`.
* Schema 4 records `live_publication_enabled: true` and bounded `validation_scope`.
* Retained `candidate_state` and `candidate_attitude` names use `candidate_is_hypothesis: false`.
* Historical schemas 1–3 withheld candidate live. Schema 2 could reject cross-unit disagreements.

## Testing

Use the [validation scope matrix](testing/index.md#validation-by-change-scope), [automated commands](testing/automated.md#automated-checks), and [evidence procedure](testing/evidence.md).

### Automated checks

| Boundary | Coverage |
| --- | --- |
| Configuration | Source selection, DLL paths, CLI errors, demo independence, no HTTP listener in SimConnect mode |
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
* Keep local logs, session IDs, versions, independent observations, and [build provenance](testing/evidence.md#build-provenance). Runtime metadata alone does not establish a binary's source.
* Divide process CPU time by elapsed wall time for one logical CPU. State normalization across all logical CPUs.
* Use console observations without logging for connection, availability, and shutdown claims. Do not infer numerical angles.

### Live-publication evidence

Acceptance and rationale have separate sources:

* [#5 acceptance record](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5): tested versions, session results, historical corrections.
* [ADR 0006](adr/0006-simconnect-manual-ffi.md#live-publication-amendment-2026-10-27): decision to enable live publication.

## Rationale and regression lessons

| Decision | Reason |
| --- | --- |
| Native radians | Independent requested units do not guarantee simultaneous acquisition |
| Diagnostic degree and repeat probes | No effect on canonical validity. The cause of discrepancies remains unresolved |
| Native bindings and ownership | Defined in [ADR 0006](adr/0006-simconnect-manual-ffi.md) |

The [completed validation](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) has these limits:

| Area | Evidence limit |
| --- | --- |
| Compatibility | Tested setup only, not every aircraft, UI phase, or simulator and DLL version |
| Cockpit comparison | No certification of precise angles or independently timed readiness |
| Live publication | Automated regressions for the change. Earlier physical sessions used the provisional waiting gate |
| Performance | Short active CPU observations, not a benchmark. Menu CPU deferred. No numerical shutdown latency |
| Other environments and interruption | Untested: clean machine without SDK, interruption during a blocked native call, independent OS unload verification |
