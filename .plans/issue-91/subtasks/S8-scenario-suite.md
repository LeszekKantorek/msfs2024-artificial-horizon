# S8: Deliver a flight narrative and focused Demo scenarios

## Issue body

Parent: Task #91 under Epic #9. Prerequisites: S4, S6's evidence mapping, and the selected S5 fault subset for error scenarios. Healthy flight coverage does not wait for exhaustive rare-fault coverage.

Execution: one GPT-6.1 Terra developer at medium; use Sol Medium for consequential gaps.
Read the parent, Epic, body, first context comment, and prerequisite results.
This is a local planning draft, not implementation or publication authorization.

### Outcome

A committed scenario suite demonstrates the requested flight progression and verifies source lifecycle, availability, and isolated failure outcomes through the real application.

### Scope

* Add `demo-scenarios/flight-tour.jsonl`, focused lifecycle/availability files, and dedicated error files.
* Preserve repository-root `demo_scenario.jsonl` as the mapped default/boundary scenario unless a later decision explicitly changes that role.
* Integrate scenarios into production-path tests and maintain the purpose-only catalog at `docs/sources/demo-scenarios.md`.
* Keep detailed values, durations, order, and fault triggers in JSONL; keep independent expected checkpoints in tests/fixtures.
* Retain calculation/boundary tests and presentation fixtures beyond the native field subset.
* Exclude flight dynamics, new Vx/stall/ground-contact SimVars, browser scenario controls, and OS simulator launch/termination.

### Context

* S2 scenario schema/catalog policy, S3 timing/ABI, S4 actual DLL/SSE path, and S5 fault mechanisms/results.
* Current native signal catalog, `Lifecycle`, `Instruments`, and independent existing instrument fixtures.
* The accepted discussion outcome: runway acceleration, rotation, climb, speed loss/stall-like descent, recovery, turns, approach/landing, both pause modes, and simulator start/stop.
* Native source error semantics, runtime validation, and presentation testing rules.

### Implementation guidance

1. Promote the local scenario drafts into their selected repository owners after validating them against the implemented reader.
2. Treat narrative values as fictional demonstration inputs, not aircraft-performance acceptance.
3. Exercise all existing native fields using independently documented names/units and explicit compatibility assumptions.
4. Use recoverable startup/connection unavailability in the flight tour; put permanent source failures in separate scenario files.
5. Add independent observations for coherent IAS/TAS/GS, pitch/roll, altitude/VSI, heading/turn/ball, available speed references, BARO, and target gates.
6. Separate inactive flight from absent/disconnected simulator and from pause.
7. Assert a fresh eligible observation after resume, reconnect, and flight changes.
8. Use the narrative for compiled application integration; keep focused fixtures for exact boundaries and source-unavailable cases.
9. Document only scenario purpose, assumptions, expected completion, and links in the catalog.
10. Record malformed/parser/API and infrastructure outcomes at the boundary that actually controls them.

### Acceptance criteria

* [ ] Review covers the assigned outcome and the prerequisites supplied to the next consumer under the [tasks/tests/review plan](../tasks-tests-review.md). Record evidence, gaps, and a scoped readiness verdict.

* [ ] Checked log/SSE results map to scenario input and documented transformations using S6's independent checkpoints. No recording converter or source-file reconstruction is required.
* [ ] Normal and active pause follow verified native behavior. Baseline paused intervals use authored holds; the clock keeps advancing. Do not classify ordinary paused native messages as injected faults.

* [ ] The flight tour demonstrates the requested progression without triggering a terminal native failure.
* [ ] The tour returns to its first step after the last step on the same increasing clock. Explicit scenario records control all pause and flight changes.
* [ ] Both pause modes and flight/simulator availability changes reach the real receiver and HTTP/SSE.
* [ ] A recoverable startup failure reaches later narrative data; permanent-failure files stop only their intended producer path.
* [ ] Every current native field has independent coverage; Vx/stall/runway narrative does not invent unavailable source fields.
* [ ] Coverage is collective across the suite. Focused IAS-only, event-only, and no-signal cases require no unrelated filler values.
* [ ] Primary gaps suppress the complete grouped attitude packet; no cached component becomes fresh.
* [ ] Focused cases distinguish a constant-value hold from a gap: the hold supplies repeated acquisitions under flags 0; the gap supplies none for that signal.
* [ ] Native failures are asserted separately from HTTP liveness and whole-application exit.
* [ ] Exact boundary tests and additional presentation-field coverage remain meaningful.
* [ ] The scenario catalog contains no copied timeline, numeric phase table, or duplicate scenario definition.
* [ ] Each committed JSONL file parses through the production reader and selected scenarios reach the compiled EFI.
* [ ] Matched captures, required application checks/release build, and independent review identify the exact tested files and artifacts.

### Validation

Later implementation: controlled-clock checkpoints, real-loader/source/HTTP assertions, complete applicable Windows checks/build, both desktop browser scripts, and capture inspection.
This is the single planned integrated screenshot/visual session for #91. Collect the relevant S4/S5 display cases here under the [visual verification schedule](../tasks-tests-review.md#visual-verification-schedule); inspect representative states, not every scenario step or native error. Repeat affected visual checks only if the session reveals defects or later changes invalidate its evidence.
Record scenario identities, independent expected checkpoints, actual source/HTTP/application outcomes, and unavailable checks.
Do not claim a passing narrative certifies aerodynamic behavior, real simulator compatibility, or device acceptance.
No scenarios were executed or tested during this planning session.

## First comment: execution context

Reviewer: apply the [S8 transition row](../tasks-tests-review.md#tasks-checks-and-handoff-reviews) to the parent #91 criteria. Review the combined scenario -> DLL -> receiver -> log/SSE/EFI result, not just added scenario files. Preserve outstanding simulator/device evidence and deferred rare faults without claiming that completed child issues establish full parent acceptance.

The scenario suite was explicitly requested after the initial six Subtask drafts.
A narrated flight and a malformed packet serve different outcomes.
The first fatal producer error would prevent the remaining flight narrative from executing.

The tour can include an initial Open failure and finite QUIT/unavailable interval because those use existing retry.
A required subscription error, malformed native buffer, or failed Close belongs in a separate file.
HTTP must remain usable when a native producer ends under the accepted isolation rule.

Initial Open faults use matching call occurrence before the scenario clock begins.
The first later successful Open starts time once; reconnects retain it.
Do not add another availability clock merely to implement that startup case.

Playback loops until shutdown or a terminal source failure. There is no wait-at-end policy.
State left by the final steps persists into the next pass until explicit records change it.
Any desired start-of-pass pause or flight transition must be authored in JSONL, never inferred by the playback mechanism.

The current native surface has no Vx, stall-warning/AOA, runway position, or weight-on-wheels input.
Represent the narrative through existing values; do not add a private SimVar or calculated flight model.
Minimum rotation is the existing project reference rather than an aircraft-certified takeoff speed.

The catalog selects scenarios by purpose and states expected termination/recovery.
Scenario contents remain the sole owner of durations, phase values, and fault parameters.
Independent test expectations are separate evidence and must not be generated by replaying the same parser/engine.

The existing default/boundary scenario remains in its agreed root file.
The flight tour is an additional explicit scenario.
Changing the default is a distinct decision; creating a catalog does not silently make that change.

The current local drafts are concrete planning examples, not validated production inputs.
S5 records implemented and deferred cases from the full error inventory; representative draft files do not require exhaustive fault coverage.
Host failures need launch/harness conditions rather than fictional native opcodes.
