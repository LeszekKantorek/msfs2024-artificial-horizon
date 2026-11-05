# ADR 0009: Agent-operated live development through the production application

Decision status: Accepted.

## Context and intent

[Task #111](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/111) requires repeatable agent-operated checks against a flight already running in MSFS2024.
The human starts the simulator and flight. The agent starts the tested application, observes its production path, collects evidence, and cleans up.

The existing application connects to the simulator when launched with `--source simconnect`.
It already provides optional JSONL diagnostics, HTTP/SSE, and embedded browser assets.
The Node/Playwright runtime test demonstrates process orchestration and browser inspection, but does not implement this live workflow.

This design preserves [native ownership](0006-simconnect-manual-ffi.md) and [application composition](0007-application-composition.md).
It adds development tooling around the application rather than another production acquisition path.

## Goals and boundaries

* Automate bounded observation of the actual SimConnect-to-browser path after each build.
* Identify the tested executable, process, endpoint, source, and diagnostic session automatically.
* Distinguish HTTP availability, flight readiness, fresh telemetry, and browser consumption.
* Preserve independently unavailable fields and report unmet scenario prerequisites separately from application failures.
* Keep normal checks independent of the simulator, SDK, and DLL. Retain explicit Demo runs and controlled fixtures.
* Preserve the human's simulator, flight, aircraft controls, and unrelated processes.

Attaching to an existing application instance, debugger attachment, simulator launch, and automated piloting are outside this design.
Silently checking an older process or replacing a failed live run with Demo would invalidate the result.

## Approaches

| Option | Benefits | Costs and disposition |
| --- | --- | --- |
| Existing manual launch and diagnostics | Retains the current architecture without tooling | Does not provide autonomous repeatable checks or automatic evidence collection; insufficient for #111 |
| Node.js harness around the production application | Reuses native acquisition, diagnostics, SSE, and existing Playwright practice | Requires orchestration, evidence correlation, and focused diagnostic improvements; selected direction |
| Rust helper reusing the provider | Allows focused provider inspection without HTTP | Adds an executable and cannot alone establish server or browser behavior; deferred |
| Independent native SimConnect client or bridge | Supplies a separate acquisition observation | Adds native integration and maintenance; cannot prove production-path correctness; deferred until a demonstrated diagnostic need |

The launch-and-observe approach was selected in discussion.
The design uses Node.js for orchestration and Playwright for browser observation.

## Responsibilities

| Component | Responsibility |
| --- | --- |
| Harness | Launch the selected EXE, manage observation deadlines, collect evidence, evaluate checks, and clean up owned resources |
| Production Rust application | Own the single SimConnect connection, native lifecycle, validation, normalization, latest state, and HTTP/SSE |
| Provider diagnostics | Expose relevant raw acquisition and mapping/publication decisions from the existing provider |
| Harness SSE observer | Record actual published source, state, sequence, age, and field availability |
| Playwright browser | Load the application's embedded assets and observe received telemetry and relevant rendered indications |

Observing application HTTP or diagnostics does not open another SimConnect connection.
No simulator control endpoints, application framework, or production service are added.
Node.js and Playwright remain development dependencies.

## Observation procedure

1. Select an explicit executable and record its hash, path, build context, and invocation in an ignored local run directory.
2. Select a loopback endpoint and start that executable with `--source simconnect` and a dedicated diagnostic journal.
3. Record the child PID and start time. Verify endpoint ownership against that child before accepting observations; a busy port fails the run.
4. Probe HTTP with a deadline, then observe SSE and diagnostics for a bounded readiness period.
5. Open the embedded EFI in Playwright and execute checks applicable to the current flight during a bounded observation period.
6. Write machine-readable results and supporting observations, including unmet prerequisites and evidence gaps.
7. Close browser and SSE subscriptions, request application shutdown, and record the cleanup outcome before another run.

Each rebuild requires a new application run.
Recheck the executable identity after startup; a changed file makes build attribution inconclusive.
The harness must not rely on a successful `/health` response as proof of endpoint ownership or simulator readiness.

## Evidence and checks

| Evidence | Required interpretation |
| --- | --- |
| Executable hash, PID, start time, endpoint, and invocation | Identify the tested artifact and owned process; record source revision and dirty state separately from binary identity |
| Diagnostic session and provider observations | Identify acquisition and mapping decisions; absent or failed diagnostics leave those claims unverified |
| SSE observations | Establish actual publication with SimConnect source, sequence, state, age, and independent field availability |
| Browser observations | Establish received data and relevant display behavior; an independent harness subscription does not prove browser receipt |
| Timing and correlation | Join by explicit identifiers where available; otherwise record timing bounds and ambiguity rather than claim exact sample identity |
| Scenario checks | Define prerequisites, tolerances, observation windows, expected behavior, and outcome per check |

* Reject Demo or replayed input as live evidence. Record live and Demo as separate runs.
* Distinguish pass, application failure, unavailable prerequisite, and inconclusive observation in machine-readable results.
* Treat changing sequence and fresh acquisitions separately from changing aircraft values. A steady flight need not change every value.
* Test missing-field propagation independently from numeric correctness. Agreement between layers does not independently verify simulator accuracy.
* Continue applicable checks when a maneuver or aircraft condition is unavailable. Record loss and recovery only when actually observed.
* Bound retained data and diagnostic volume. Detect missing diagnostic intervals and report incomplete correlation.

## Lifecycle and failure behavior

| Condition | Required behavior |
| --- | --- |
| Invalid configuration, missing DLL, busy port, or application exit | Report the specific failure with captured diagnostics and a bounded completion |
| Simulator unavailable, no active flight, pause/menu, disconnect, or stale data | Report observed source state and unmet prerequisites; do not infer distinctions unsupported by diagnostics |
| Unsupported or independently missing field | Record field availability; apply only checks whose prerequisites hold |
| Timeout or cancellation | Close observers, request shutdown of the owned application, and record cleanup independently from check outcomes |
| Native shutdown remains blocked | Record the cleanup timeout; permit forced termination only of the owned child process and report that graceful native cleanup was not established |

The application continues to wait for its native worker under ADR 0007.
A harness deadline bounds the development run; it does not establish a native-call cancellation guarantee.
Windows process signaling and endpoint ownership checks require focused verification before relying on them.

## Consequences and remaining builder choices

* Keep one production provider and latest-state delivery. Extend diagnostics only where existing records cannot support the required observations.
* Prefer diagnostic correlation over extending the public telemetry contract. Any required wire change must update producers, consumers, and fixtures together.
* Keep raw sessions, machine-specific configuration, DLL paths, and SDK material in ignored local storage.
* The builder selects command names, module layout, result schema, default deadlines, retention limits, and concrete Windows signaling mechanisms.
* The builder must verify correlation coverage before promising exact acquisition-to-render matching. Unsupported correlations remain explicit evidence gaps.
* Operational documentation belongs with SimConnect usage and shared testing procedures; agent workflow selection belongs in agent guidance.

This ADR does not supersede existing native, delivery, freshness, or shutdown requirements.
It narrows #111 to launching a fresh application for each run; its optional existing-instance attachment alternative is excluded by the agreed scope.

## Validation required before implementation acceptance

* Test orchestration, SSE parsing, deadlines, failure classification, cancellation, repeat runs, and process ownership with controlled fixtures.
* Verify Windows startup, port conflicts, executable identity, graceful shutdown, and owned-process termination against the compiled application.
* Demonstrate live acquisition, publication, and browser consumption in a real flight. Record supported updates, missing fields, and any available loss/recovery scenarios.
* Use Demo or fixtures for unavailable scenarios without counting those results as live acceptance.
* Apply the [validation scope matrix](../testing/index.md#validation-by-change-scope). Application changes require a complete rebuild and applicable Windows x64 MSVC and JavaScript checks.
* Browser integration changes require both desktop browser scripts, screenshot tests, and inspection of affected captures.
* Check documentation links, requirements, and terminology. Desktop observations do not establish real-phone acceptance.

No implementation or simulator experiment is established by this decision record.

## Reconsideration

Revisit the design if production diagnostics cannot resolve a demonstrated acquisition discrepancy, or if observing an existing application becomes an agreed requirement.
Compare an independent reader only against that specific evidence need and preserve its separate validation scope.
