# SimConnect integration and diagnostics

The main application selects SimConnect with `--source simconnect` for [#5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5).
One integration module receives user-aircraft pitch and bank and publishes through the existing telemetry channel.
Optional JSONL diagnostics retain native messages, raw values, and lifecycle hypotheses.
This mode does not start HTTP or display telemetry in the browser.
[#6](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6) will connect the same module to HTTP/SSE and validate the complete path to EFI.

No live simulator session is claimed by this document.
Issue #5 remains open until repeated simulator observations confirm units, signs, availability, dispatch, freshness, reconnection, and cleanup.
The accepted integration boundaries are recorded in [ADR 0006](adr/0006-simconnect-manual-ffi.md).

## Local requirements

Use Windows x64 and the Rust MSVC development environment from [CONTRIBUTING](../CONTRIBUTING.md#environment-setup).
Obtain the official MSFS2024 SDK separately and make its x64 `SimConnect.dll` available beside the application executable, or select it with `--dll`.
The declarations were inspected against SDK **1.8.1, build 2603316**.
This identifies the header source, not a verified simulator/DLL compatibility combination.
Record the actual DLL and simulator versions for every session.

The Rust build requires no SDK headers, import library, or installed simulator.
Demo mode never attempts to load SimConnect, including when the DLL is available.
Do not commit SDK files, local SDK paths, logs, captures, or generated output.
Dynamic loading does not establish redistribution permission; runtime packaging and SDK distribution conditions remain part of [#8](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8).

## Run a session

With `SimConnect.dll` beside the executable, run from the repository root in PowerShell:

```powershell
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --log ".local\simconnect-sessions\off-menu-flight.jsonl"
```

The default DLL location is the directory of the running executable, independent of the working directory.
For a Cargo run, place the DLL beside the executable in the applicable Cargo output directory.
An explicit absolute path takes precedence:

```powershell
$simconnectDll = Read-Host 'Absolute path to the official x64 SimConnect.dll'
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source simconnect --dll "$simconnectDll" --log ".local\simconnect-sessions\off-menu-flight.jsonl"
```

Select one scenario before each run and distinguish it through the log filename.
The application does not interpret that filename or store scenario metadata.
Repeating the command appends a new session to the same file without replacing earlier records.

| Option | Meaning |
| --- | --- |
| No source option or `--source demo` | Demo with HTTP/SSE, without loading SimConnect |
| `--source simconnect` | Native reception and telemetry publication without HTTP/SSE in #5 |
| `--dll <file>` | Optional absolute DLL override; default is `SimConnect.dll` beside the running executable |
| `--log <file>` | Optional exact JSONL file, absolute or relative to the working directory; append mode |
| `--help`, `--version` | CLI information without starting a session |
| `Ctrl+C` | Request controlled shutdown and wait for the worker to finish |

`--dll` and `--log` are accepted only for SimConnect.
Explicit HTTP listen-address/port options are rejected for SimConnect until #6.
Help and version do not open the DLL or log.

The console reports startup, the selected DLL, connections, state transitions, important errors, and shutdown even without logging.
Without `--log`, no diagnostic file or expensive raw-record formatting and environment collection is performed.
Acceptance sessions require logging and independent observations.

With logging enabled, the application creates missing parent directories and opens exactly the supplied file for append before loading the DLL.
Each run has a unique `session_id` on every record, its own record numbering, and monotonic time from session start.
Only one process may write a file at a time; a second writer receives a controlled open error.
A partial final line from an interrupted process is preserved and separated from the new session by a newline; it is not a valid record.
The selected DLL and running executable cannot be used as the log destination.

Failure to open the log or write its initial records aborts startup before DLL loading.
A later write or flush failure disables diagnostics for the rest of the run while reception and cleanup continue.
The console reports the failure immediately and again at shutdown; the module result retains the diagnostic failure separately from acquisition errors.
Keep that console output with the observations, since the failed journal cannot reliably record it.
A session without a final `session_end` record has an unconfirmed ending; interrupted logging never establishes a complete measurement.

Missing DLLs, incompatible architecture, and missing exports are controlled errors.
An absent default DLL reports its expected location and the `--dll` override.
Loading never searches `PATH`, the working directory, or SDK installations, and a failed explicit override never falls back to another DLL.
Dependency search is restricted to the selected DLL directory and the Windows system directory.
An unavailable simulator causes interruptible connection retries after 1, 2, 4, 8, then at most 10 seconds.
These intervals are experimental settings.
There is no automatic switch to demo.

## What the records mean

The worker requests four `FLOAT64` values in one definition for the user aircraft, once per simulator frame, without `CHANGED`:

| Index | Variable | Requested unit |
| --- | --- | --- |
| 0 | `PLANE PITCH DEGREES` | radians |
| 1 | `PLANE PITCH DEGREES` | degrees |
| 2 | `PLANE BANK DEGREES` | radians |
| 3 | `PLANE BANK DEGREES` | degrees |

With diagnostics enabled, each sample records all four raw values before normalization and publication.
Nonfinite numbers use JSON-compatible value/classification/bit records.
The candidate conversion checks radians/degrees agreement within `0.00001` degrees, negates pitch and bank, and wraps bank into `[-180, 180)`.
It validates finite pitch in `[-90, 90]` under the existing [telemetry contract](telemetry-contract.md#fields-and-conventions).
Negating both signs is an experimental hypothesis requiring cockpit observations.
The variable names and agreement between two requested units do not establish the correct physical sign.

`state_transition` separates `candidate_state` from the actual `published` snapshot.
The candidate combines `Sim`, `Pause_EX1`, `FlightLoaded`, and fresh valid samples.
No single event or nonempty flight filename proves active-flight availability.
All nonzero pause flags, including unknown bits, are treated conservatively as paused.
State changes clear retained attitude so resume and a new flight require a new valid sample.
New identical samples refresh candidate age; cached values, connection events, and system-state queries do not.
The freshness limit remains 1000 ms with a 50 ms control interval.

**A candidate `live` is published as `waiting`.**
The integration deliberately withholds live application telemetry while flight availability and signs are unverified.
Other unavailable candidate states use the existing `Publisher` and latest-value watch channel.
The log is diagnostic output, not another telemetry queue or distribution mechanism.
A `candidate_state: live` record is neither live acceptance nor evidence that menu detection works.

The receiver uses `GetNextDispatch`, a Windows notification event, a separate cancellation event, 50 ms waits, and batches of at most 128 messages.
Empty dispatches and their HRESULTs are aggregated in one-second summaries.
An empty queue or `E_FAIL` alone does not prove disconnection.
System-state queries every 5 seconds and a 15-second packet-silence watchdog are diagnostic hypotheses to test.
The watchdog can request reconnection; its label is not proof of connection loss.
Record unexpected queries, wake patterns, retries, or CPU use before accepting this receive strategy.

The records provide these diagnostic groups:

| Records | Analysis use |
| --- | --- |
| `session_start`, `environment`, `dll_identity` | Session identity, timing, configuration and selected DLL, executable/DLL SHA-256, runtime Rust/Git/Windows context, optional nearby SDK metadata |
| `dispatch` with an Open message | Application and SimConnect versions reported by the simulator |
| `connection_attempt`, `api_result`, `native_error` | Connection generation, HRESULT/Windows errors, requests, sent packet IDs, and exception correlation |
| `dispatch`, `attitude_raw`, `attitude_sample` | Message ID/size and owned decoded data; raw units and sample intervals before conversion; then candidate normalization |
| `state_transition`, `connection_lost`, `reconnect_wait` | Hypothesized lifecycle, actual publication, reason, and retry timing |
| `dispatch_summary` | Packet/empty-dispatch counts, notification/timeout wakes, last HRESULT, and packet silence |
| `notification_release`, `library_release`, `session_end` | Cleanup attempts and session outcome |

`session_id` identifies the run on every record. `record` numbering starts anew for each session.
`elapsed_us` is monotonic time since session start; `started_unix_ms` supplies a UTC reference.
No scenario field is written; use the filename and sidecar observations to organize scenarios.
Some times and intervals are strings to retain integer precision.
Logs flush at significant lifecycle/error records and at least once per second while records are being written.
Normal cleanup closes the connection, releases its notification handle, and unloads the DLL after native use ends.
A failed native close is an error and retains both the DLL mapping and its notification handle until process exit rather than recycling resources that may still be in use.
A release record describes the attempted path; inspect errors and the final process outcome as well.

## Record observations alongside the log

Before running, retain a local build record with the source revision, any dirty changes, build command/result, build-time Rust/Cargo versions, and executable SHA-256.
The log's runtime Git and `rustc` results describe the machine when launched; they cannot prove which source or compiler produced an existing executable.
Keep the build record and any local diff outside Git with the session artifacts.

Create an adjacent observations file for each scenario log, such as `off-menu-flight.observations.md`.
For appended runs, retain a separate entry keyed by `session_id` and repetition number.
Record these fields:

* Log filename, session ID, scenario name from the filename, repetition number, date, and local build-record reference.
* Windows, SDK, DLL, and simulator versions; aircraft, flight setup, and relevant simulator options.
* Observed action and cockpit attitude at each elapsed time from session start, including pause mode, menu/loading state, and any interruption method.
* Expected and observed units/signs, sample behavior, candidate and published states, retries, CPU measurements, and shutdown duration.
* Result for each question: confirmed, contradicted, or insufficient, with session IDs and relevant record numbers or elapsed times.
* Diagnostic status and any console write/flush error; mark incomplete journals as insufficient evidence where records are missing.

Use a stopwatch from launch and note its offset from the first log record when timing is approximate.
Record exact action times where possible; screenshots or video can support attitude observations.
Do not infer a sign, menu transition, or pause mode from log values without an independent simulator observation.
Retain unexpected outcomes as evidence rather than changing the scenario retrospectively.

## Repeatable simulator scenarios

The durations below are observation windows, not claims of measured performance or pass thresholds.
Allow loading and startup to finish before timing a stable phase.
Repeat the selected scenario at least twice; investigate inconsistent results with a narrower session.
Keep every run's session records and observations, including when multiple runs share one file.

### `off-menu-flight`

1. Close MSFS2024. Start the application in SimConnect mode and wait 30 seconds. Observe controlled connection errors and bounded retry intervals.
2. Start MSFS2024 and remain in its main menu for at least 30 seconds. Record when a connection opens and all lifecycle/sample behavior.
3. Load a flight. Record loading start/end and cockpit availability. Remain in an active, unpaused flight for 30 seconds.
4. Hold a steady attitude for another 30 seconds. Check new sample records and intervals even when values are equal.
5. Stop with `Ctrl+C` and retain cleanup records.

Expected evidence: no demo fallback, recovery when the simulator becomes available, and a reproducible distinction between menu/loading and observed active flight.
Actual publication must remain unavailable throughout this unverified integration, including `waiting` for candidate live.
If menu produces candidate live, mark the availability hypothesis contradicted; do not certify it from the publication guard alone.

### `attitude-signs`

1. Begin an active flight with a known aircraft and readable cockpit attitude reference. Record 15 seconds near level.
2. Hold nose-up, then nose-down attitudes for at least 10 seconds each. Note approximate cockpit pitch and elapsed times.
3. Hold left, then right banks for at least 10 seconds each. Note bank direction and approximate angle.
4. Hold one combined pitch/bank attitude for 10 seconds, then a combination with the opposite bank for 10 seconds.
5. Return near level for 15 seconds and stop the application.

Expected evidence: radians/degrees pairs agree, both observed physical signs match the application conventions after candidate conversion, and combined attitudes remain consistent.
The application requires positive nose-up pitch and positive right-wing-down roll.
If observations contradict either sign, retain the raw data and identify which conversion needs revision before any live publication is enabled.

### `pause-modes`

1. Begin an active, unpaused flight and record 20 seconds of samples.
2. Enter ordinary pause for 20 seconds. Record the exact pause action, flags, sample arrivals, and candidate/published transitions.
3. Resume for 20 seconds. Verify that a new valid sample is required; cached values must not make the candidate fresh.
4. Enter Active Pause for 20 seconds and record the same observations separately.
5. Resume for 20 seconds and stop.

Expected evidence: both pause modes are distinguishable from active flight, unavailable publication persists during pause, and resume does not refresh sample age by itself.
Record any continued sample stream or combined/unknown pause flags; numeric movement alone does not identify the pause mode.

### `flight-exit-new-flight`

1. Record 20 seconds in an active flight, then leave it for the main menu.
2. Remain in the menu for 30 seconds. Note whether old filenames, state values, or attitude samples persist.
3. Load another flight, preferably with a visibly different initial attitude or aircraft. Record the loading phase and 30 seconds after cockpit availability.
4. Stop the application.

Expected evidence: retained samples are invalidated across flight transitions and candidate freshness requires samples from the new active flight.
A retained filename or a successful state query must not be accepted as proof of flight availability.
Any menu candidate live is a failed hypothesis requiring a follow-up experiment.

### `shutdown-restart-interruption`

1. Record 20 seconds in an active flight. Close MSFS2024 while leaving the application running.
2. Wait at least 30 seconds. Observe Quit messages, dispatch results, watchdog behavior, state changes, and retries.
3. Restart MSFS2024. Observe the menu for 20 seconds, then load a flight and observe 30 seconds.
4. In a separate repetition, interrupt the connection using a recorded reproducible local method, if available. Do not label a normal shutdown as a separate interruption result.
5. Restore the simulator/connection and stop after another 30 seconds.

Expected evidence: loss never selects demo, retries remain interruptible, connection generation changes on reconnect, and cached attitude is not reused.
If an independent connection interruption cannot be produced, mark that part untested.
Distinguish a watchdog-triggered retry from an observed native disconnect.

### `cancel-and-cpu`

Use separate runs to stop with `Ctrl+C` during initial connection attempts, a maximum retry wait, idle/menu reception, active reception, pause, and reconnect.
Hold each phase for at least 30 seconds before cancellation when possible.
Also measure an idle/menu phase for 2 minutes and an active high-rate phase for 2 minutes, both with and without logging.
Compare the runs to assess the effect of diagnostics; retain console observations for runs without a journal.

For each phase, record process CPU time before/after the interval and its measured wall duration, or Task Manager CPU observations with the sampling method.
CPU-seconds divided by wall-seconds expresses use relative to one logical CPU; state any normalization used for whole-machine percentages.
For logged runs, compare idle and active phases with dispatch/wake counts to identify unexpected continuous polling.
Record machine load and logical CPU count so results remain interpretable.
No numeric CPU acceptance threshold has yet been established by a live session.

Measure from the cancellation request to process exit and state timing precision.
Inspect Close/release/session-end records for logged runs and retain console output for all runs.
Expected evidence: waiting and retrying can be cancelled promptly, continuous dispatch cannot starve cancellation, and normal shutdown releases the connection and handles before unloading the DLL.
A hung native call, missing exit, failed Close, or unexplained CPU load is a failed observation requiring investigation.

## Evaluate and retain evidence

Compare logs with the sidecar observations and the questions in each scenario.
Preserve raw evidence locally and publish only a suitable summary against the tested revision in #5 or its linked validation record.
Include actual versions, repetitions, confirmed results, contradictions, missing checks, and the next focused experiment.
Logs may contain local paths and flight filenames; inspect them before sharing.

Simulator-free tests check CLI source/options, executable-relative DLL selection, SDK-independent builds, layout, decoding, conversion hypotheses, freshness, loader errors, retry bounds, cancellation, and optional diagnostic failure paths.
They also check exact-file append sessions, distinct session IDs, writer exclusion, partial-tail handling, and publication through the supplied telemetry channel.
Use the shared [automated checks](testing/automated.md#automated-checks) for local validation.
An additional opt-in cancellation smoke check requires an explicitly selected official DLL:

```powershell
$env:SIMCONNECT_TEST_DLL = $simconnectDll
cargo test --locked --test simconnect official_dll_stops_on_requested_shutdown_and_uses_caller_channel -- --ignored --nocapture
Remove-Item Env:SIMCONNECT_TEST_DLL
```

This requests cancellation after 750 ms and retains its session log.
Run it with the simulator closed to check loading, unavailable connection attempts, and interruptible retry.
It does not replace console `Ctrl+C`, live acquisition, or the longer lifecycle scenarios.
Rust layout tests can repeat an incorrect header interpretation and do not prove ABI compatibility by themselves.
No automated test or console session certifies HTTP/SSE, multiple browser clients, or mobile EFI with live MSFS2024 data.
Those integration questions remain in #6 and its dependent acceptance work.
