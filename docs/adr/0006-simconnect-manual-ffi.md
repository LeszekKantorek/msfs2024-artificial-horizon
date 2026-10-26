# ADR 0006: Minimal manual SimConnect FFI with runtime loading

| Date | Status |
| --- | --- |
| 2026-10-26 | Accepted integration boundaries; receive, normalization, and lifecycle hypotheses require live evidence |

## Context

[#5](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) investigates real user-aircraft pitch and bank before the production provider in [#6](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6).
The existing Rust library, dedicated acquisition boundary, `Publisher`, latest-value watch channel, HTTP/SSE, and browser contracts remain suitable.
The experiment needs direct control over a small native API surface and reproducible diagnostics without an SDK build dependency.
The accepted integration plan selects manual bindings; repeating a third-party wrapper selection is outside this iteration.

This refines [ADR 0001](0001-rust-http-sse.md), which deferred FFI selection.
Its process, transport, and normalized latest-value decisions remain unchanged.
Runtime loading replaces any implication that SimConnect requires an optional Cargo dependency or feature.
The package exposes no Cargo features.

## Decision

| Boundary | Accepted choice |
| --- | --- |
| Native declarations | Minimal hand-written Rust declarations from official MSFS2024 SDK headers |
| Organization | Existing package under `src/providers/simconnect.rs`, with focused internal modules as needed |
| Loading | `LoadLibraryExW`, resolved exports, and controlled DLL lifetime; no `SimConnect.lib` linkage |
| Diagnostic DLL selection | Required absolute path; dependency search restricted to the DLL directory and Windows system directory |
| Execution | One long-running worker owns one active connection and all direct SimConnect calls |
| Memory boundary | Validate message type/length, respect packed receive layouts, copy into owned Rust values, retain no native buffer across dispatch |
| Telemetry | Normalize in the adapter and publish through existing typed `Publisher`/watch APIs |
| Experiment | Separate `simconnect-probe` console target; no HTTP/browser integration |
| Source failure | Explicit unavailable states and retries; never select demo automatically |

The initial API surface has eight exports: `SimConnect_Open`, `SimConnect_Close`, `SimConnect_GetNextDispatch`, `SimConnect_AddToDataDefinition`, `SimConnect_RequestDataOnSimObject`, `SimConnect_SubscribeToSystemEvent`, `SimConnect_RequestSystemState`, and `SimConnect_GetLastSentPacketID`.
Add a declaration only for a demonstrated requirement.
SDK 1.8.1, build 2603316, is the inspected header source.
This does not claim that any live SDK/DLL/simulator combination has passed acceptance.

No wrapper crate, binding generator, separate package, C++ helper, general SDK framework, new telemetry queue, or extra high-level adapter layer is introduced.
The diagnostic journal is local experiment output, separate from latest-state distribution.
Demo and normal builds remain independent of SDK installation; demo never loads SimConnect.

## Alternatives and rationale

| Option | Consequence | Decision |
| --- | --- | --- |
| Retain the demo and postpone the experiment | Preserves current behavior but cannot establish real units, lifecycle, or native compatibility | Retain demo independently; add the focused experiment |
| Third-party Rust wrapper | Delegates API coverage and maintenance but adds an external integration abstraction | Superseded by the accepted manual-binding scope |
| Minimal manual FFI | Keeps ownership, debugging, compatibility changes, and the used declarations within the project | Select; accept declaration maintenance and verification costs |
| Generated/full SDK bindings | Adds generation tooling or unused API surface | Reject for this small, explicit scope |
| Import-library linking | Requires the native library during linking | Reject; runtime loading keeps SDK files outside default builds |
| Separate crate or C++ ABI helper | Adds a package or language/tool boundary | Reject; manual header inspection, focused Rust tests, and live checks are the accepted verification approach |
| Native callbacks | Requires callback/context lifetime handling | Retain as a possible alternative only if the preferred dispatch experiment proves inadequate |

The worker isolates native work from Tokio and provides one connection owner.
The binding boundary owns DLL/function lifetimes; the adapter owns message interpretation, normalization, and application states.
Cleanup stops processing, closes the connection, releases handles, and unloads the DLL after native use ends.
A failed close retains the mapping and its notification handle until process exit as a conservative failure path.
Dropping a Rust value is not assumed to interrupt a blocking native call.

The project accepts maintenance responsibility for signatures, constants, layouts, and SDK changes.
Header-derived Rust size/alignment/offset tests can reproduce a mistaken translation.
They support verification but do not independently prove the ABI; real integration evidence remains necessary.

## Experimental choices and evidence limits

The [probe reference](../simconnect-probe.md) owns the implemented diagnostic behavior, repeatable scenarios, and observation procedure.
Execution results and acceptance evidence belong in #5 or its linked validation record.

These choices remain hypotheses:

* `GetNextDispatch` with a Windows notification event, separate cancellation event, bounded batches, and 50 ms waits provides reliable reception with acceptable CPU use and shutdown.
* Five-second state queries and a 15-second packet-silence watchdog provide useful connection diagnostics.
* `Sim`, `Pause_EX1`, and `FlightLoaded`, combined with fresh samples, can distinguish active flight from menu/loading and both pause modes.
* Negating pitch and bank, with bank wrapping, matches the existing positive nose-up/right-wing-down contract.

The probe requests each attitude variable in both radians and degrees and records raw values alongside candidate normalization.
The existing 1000 ms freshness limit remains unchanged.
New equal samples can be fresh; cached values and heartbeat queries cannot refresh sample age.
An unverified candidate live state is deliberately published as `waiting`, so diagnostic hypotheses cannot become live application telemetry.
That guard does not prove correct flight detection or signs.

No live run, confirmed physical sign, verified menu/pause mapping, reliable dispatch/wait strategy, or successful reconnect/cleanup measurement is established by accepting this ADR.
Runtime version records and executable hashes identify context and bytes; separate local build evidence is needed for source/compiler provenance.
Repeated logs must be paired with observed simulator actions and attitudes.
Successful console acquisition would still not establish production-provider, HTTP/SSE, multi-client, or mobile acceptance.

## Consequences and reconsideration

The selected DLL must remain available and compatible at runtime.
Absolute-path loading is the probe's reproducibility and safe-search policy; final release discovery is not settled here.
No SDK headers, libraries, binaries, local installation paths, or generated session output enter commits.
Dynamic linking grants no redistribution right.
Legal and practical SDK/DLL distribution conditions must be established before release packaging in [#8](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8).
No conclusion is drawn from other projects' distribution practices.

Reconsider the receive/wait strategy if measured events, empty queues, CPU use, or shutdown contradict it.
Reconsider lifecycle and sign conversion only against raw records paired with independent simulator observations.
Suspend an unsafe native path when ABI or bounds handling is suspect.
If active flight cannot be distinguished reliably from menu or pause, retain unavailable publication and investigate before #6.
Changes requiring additional API surface, packages, queues, helper languages, or altered telemetry contracts require a demonstrated need and an agreed scope amendment.
