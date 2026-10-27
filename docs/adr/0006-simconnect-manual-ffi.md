# ADR 0006: Minimal manual SimConnect FFI with runtime loading

| Date | Status |
| --- | --- |
| 2026-10-26 | Accepted manual bindings, runtime loading, and one native worker |
| 2026-10-27 | Amended application entry point, DLL selection, and diagnostics |
| 2026-10-27 | Accepted bounded live publication after evidence assessment |

## Context

The integration preserves these boundaries:

* [#5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) establishes native pitch and bank acquisition before HTTP/SSE and EFI integration in [#6](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6).
* The existing library, `Publisher`, latest-value watch channel, and browser contract remain suitable.
* Direct control over a small native API supports diagnostics without an SDK build dependency.
* [ADR 0001](0001-rust-http-sse.md) deferred FFI selection. Its process, transport, and normalized-state decisions remain unchanged.
* The accepted scope selects manual bindings. Wrapper selection is outside this iteration.
* The main application replaces the separate diagnostic executable.
* Default DLL selection beside the EXE and optional append logs replace the mandatory DLL override and generated logs.

## Decision

### Bindings and runtime

| Boundary | Accepted choice |
| --- | --- |
| Declarations | Minimal hand-written Rust declarations from official SDK headers |
| Header source | SDK 1.8.1, build 2603316 |
| Organization | Existing package under `src/providers/simconnect/mod.rs`, with focused internal modules |
| Loading | `LoadLibraryExW`, resolved exports, controlled DLL lifetime. No `SimConnect.lib` linkage |
| DLL selection | `SimConnect.dll` beside the EXE, independent of working directory, or an optional absolute override |
| Dependencies | Selected DLL directory and Windows system directory only |
| Execution | One long-running worker owns one active connection and all SimConnect calls |
| Memory | Validate message types and lengths, respect packed layouts, copy owned values, retain no native buffer across dispatch |
| Telemetry | Adapter normalization and publication through the caller's existing typed `Publisher` and watch channel |
| Entry point | Main executable with `--source demo` or `--source simconnect`. No HTTP/SSE in #5 |

The binding surface remains limited:

| Purpose | Exports |
| --- | --- |
| Connection | `SimConnect_Open`, `SimConnect_Close` |
| Dispatch | `SimConnect_GetNextDispatch` |
| Attitude | `SimConnect_AddToDataDefinition`, `SimConnect_RequestDataOnSimObject` |
| Lifecycle | `SimConnect_SubscribeToSystemEvent`, `SimConnect_RequestSystemState` |
| Correlation | `SimConnect_GetLastSentPacketID` |

Integration constraints preserve the existing architecture:

* Add declarations only for demonstrated requirements.
* Add no third-party SimConnect wrapper, binding generator, separate package, C++ helper, general SDK framework, or extra adapter layer.
* Expose no Cargo features or SDK build dependencies.
* Accept the existing publisher and shutdown signal from the caller. Create no hidden channel or sample queue.
* Keep demo independent of the SDK. Never load SimConnect for demo or select demo after simulator loss.
* Connect this same module to the server in #6 rather than adding another receiver.
* Resolve the full DLL path before opening diagnostics or loading native code.
* Apply explicit override precedence without fallback. Do not search `PATH`, the working directory, or SDK installations.

### Diagnostics

| Condition | Accepted behavior |
| --- | --- |
| Destination | Optional exact-file JSONL append with per-record session identity |
| Scenario | Filename and observations, without scenario configuration or metadata |
| No logging | No diagnostic file, expensive raw formatting, or diagnostic metadata collection |
| Open or initial write failure | Abort before DLL loading |
| Later write or flush failure | Disable diagnostics, continue acquisition and cleanup, report immediately and at shutdown |
| Outcomes | Separate acquisition result from diagnostic status. An interrupted journal does not establish a complete measurement |

The [feature guide](../simconnect.md#diagnostic-logging) owns operational details and record formats.

### Live-publication amendment, 2026-10-27

The [accepted evidence in #5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) supports removing the provisional live-to-`waiting` gate within the tested setup:

* Use native radians and local conversion. Requested degrees and repeated radians remain diagnostic only.
* Record raw values before normalization and publication when logging is enabled.
* Publish live only with running `Sim`, a loaded flight, zero pause flags, and a fresh finite, in-range sample.
* Invalidate retained attitude on pause, invalid input, disconnect, and flight or simulator transitions.
* Require a new sample after resume. Publish each eligible acquisition, including equal values, with its original receive time.
* Keep the 1000 ms freshness limit. Cached values, queries, and heartbeats do not refresh age.
* Use schema 4 for enabled publication. Historical schemas 1–3 retain their original waiting gate.

## Alternatives and consequences

| Alternative | Decision and consequence |
| --- | --- |
| Postpone native integration | Retain demo independently, but add acquisition to establish native compatibility |
| Third-party wrapper | Not selected under the accepted manual-binding scope |
| Minimal manual FFI | Select for direct ownership and focused declarations. Accept SDK maintenance and verification costs |
| Generated or full SDK bindings | Reject generation tooling and unused API surface |
| Import-library linking | Reject the native library requirement during linking |
| Separate crate or C++ ABI helper | Reject extra package or language boundaries. Use header inspection, focused Rust tests, and live evidence |
| Native callbacks | Defer unless receive and wait observations contradict the selected dispatch strategy |

Ownership has these consequences:

* The worker isolates native work from Tokio.
* The binding boundary owns DLL and function lifetimes. The adapter owns message interpretation, normalization, and application states.
* Cleanup stops reception, closes the connection, releases handles, and unloads the DLL after native use ends.
* A failed close retains the DLL mapping and notification handle until process exit.
* Dropping a Rust value does not establish that a blocking native call has stopped.
* The project maintains signatures, constants, layouts, and SDK compatibility.
* Header-derived Rust layout tests can repeat translation errors. They do not independently prove the SDK ABI.
* SDK artifacts, local installation paths, and generated session output remain outside commits.
* Distribution rights must be established in [#8](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8). Dynamic loading and other projects' practices establish no redistribution permission.

### Evidence limits

Evidence and decision records have separate scopes:

| Record | Establishes or limits |
| --- | --- |
| This ADR | Binding, ownership, loading, and publication decisions |
| SDK header source | Inspected declarations, not general simulator and DLL compatibility |
| [#5 acceptance record](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) | Tested environment, paired simulator observations, execution results, and accepted deferrals |
| [Feature limits](../simconnect.md#rationale-and-regression-lessons) | Current compatibility, readiness, angle, performance, and untested-path qualifications |
| Earlier simulator sessions | Candidate behavior under the former gate. The changed publication path has automated regression coverage |
| Runtime metadata | Environment context, not independent proof of a binary's source or compiler |

> Acceptance of this decision does not certify precise cockpit angles, universal readiness detection, or HTTP/SSE and mobile EFI integration.

### Reconsideration

| Trigger | Required response |
| --- | --- |
| Contradictory receive, wake, CPU, or shutdown observations | Reconsider the receive and wait strategy |
| Contradictory lifecycle or sign observations | Compare raw records with independent simulator observations before changing interpretation |
| Suspected ABI or buffer-boundary error | Suspend the affected unsafe native path |
| Contradictory availability | Keep the affected path unavailable and investigate before relying on it in #6 |
| New APIs, dependencies, helpers, queues, or telemetry contracts | Demonstrate the need and agree on a scope amendment |
