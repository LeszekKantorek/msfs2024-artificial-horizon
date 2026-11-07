# Local amendment draft for Task #91

Status: Proposed replacement direction for future authorized publication. Nothing was sent to GitHub.

## Proposed title

Exercise SimConnect acquisition through a JSONL-driven Demo DLL

## Outcome

Select `--source demo.dll` or `--source SimConnect.dll`.
Both use the same application-native receiver, normalization, lifecycle gates, Publisher, HTTP/SSE, and EFI.
Demo runs without the simulator or official SDK runtime.

## Requirements

* Build a project-owned Windows x64 Demo DLL implementing the currently consumed SimConnect API.
* Keep the existing direct official DLL loading path. Do not introduce a wrapper DLL for the real runtime.
* Replace the separate source enum and --dll CLI with selected DLL paths.
* Keep Demo source identity and simulator-free default startup.
* Store the default scenario in `demo_scenario.jsonl` and embed it in Demo DLL.
* Support an explicit scenario path without recompilation, with actionable validation failure and no fallback.
* Use JSONL instructions for holds, ramps, gaps, lifecycle, and faults.
* Let Demo DLL calculate approximate monotonic timing and deliver observations at the requested cadence.
* Match SimConnect request flags and periods. With the current SIM_FRAME/flags-0 requests, send unchanged values throughout hold; gap sends no observations for that signal.
* Keep the playback clock continuously increasing. After the last step, return to the first step without resetting time or waiting at the file boundary.
* Treat Demo DLL as a playback mechanism only. Initialize from the scenario once; change pause and flight state only through explicit scenario records.
* Do not restore initial state or synthesize lifecycle events when the step sequence loops.
* Cover every currently consumed signal, unit, index, datatype, request key, and lifecycle name against an independent catalog.
* Cover the used healthy API across the suite, without requiring every scenario to supply every signal. Rare fault coverage may be staged with explicit deferred cases.
* Accept scenarios with a signal subset, lifecycle events only, or no signal values. Missing values produce no observations or registration failure.
* Default unreceived instrument values to null/unavailable in the server and UI. Demo sends no implicit zeros; absent optional SSE fields remain omitted.
* Restrict Demo data to the current real native API surface. Preserve separate presentation-fixture coverage for additional display fields.
* Simulate normal pause, active pause, active flight, inactive flight, loaded-flight changes, gaps, disconnect, and fresh-data recovery.
* Retain deterministic API errors, asynchronous exceptions, malformed packets, invalid numbers, and cleanup failures in the coverage inventory. Implement an explicit first subset and record deferred cases; do not require every fault initially.
* Supply several scenarios: a narrated flight, focused lifecycle/availability inputs, and isolated potentially terminating errors.
* Demonstrate runway acceleration, rotation, climb, speed loss/stall-like descent, recovery, turns, approach/landing, both pause modes, and simulator availability.
* Use existing native fields for that narrative; add no Vx/stall/ground-contact input or flight dynamics.
* Keep the mapped default/boundary scenario unless its role is explicitly changed. The flight tour is an additional explicit selection.
* Maintain `docs/sources/demo-scenarios.md` with purposes, assumptions, expected completion, and file links only.
* Keep scenario details in JSONL and independent expected checkpoints in tests/fixtures.
* Verify scenario -> DLL -> application -> log/SSE, relating checked output values to authored input and existing derivations.
* Exclude recorded-session conversion, diagnostic schema 4/5 import, and reconstruction of the original scenario from output.
* Preserve one receiver, one active publisher, current estimator, latest-state distribution, independent availability, freshness, and coordinated shutdown.
* Rebuild/deliver the application EXE and project-owned Demo DLL together.
* Keep official SDK artifacts and generated real-session data outside commits.

## Source and timing boundaries

A five-second hold sends successive fresh observations.
A gap sends no observations.
The next scenario pass uses the same increasing clock and later deadlines.

Demo emits native units and lifecycle messages.
The current consumer computes normalized telemetry, age, optional gates, and IAS acceleration.
The receiver assigns receipt times; compare actual results using independent expected values and justified timing tolerances.

HTTP binds before loading the selected DLL.
Scenario setup validates before simulated Open; setup failure leaves HTTP available with unavailable source data.
This explicitly changes the earlier planned pre-bind scenario-validation requirement.

## Acceptance criteria

* [ ] Each Subtask review checks its own outcome and the prerequisites it supplies downstream under the [tasks/tests/review execution plan](tasks-tests-review.md). S1 -> S2, S2 -> S3, and S3 -> S4 have explicit reviewer instructions and readiness evidence; final acceptance covers the integrated Task result.
* [ ] A workspace build produces the application EXE and demo.dll without simulator/SDK dependencies.
* [ ] Both DLL selections use the actual same native receiver and fixed source identity.
* [ ] Default selection and explicit absolute DLL selection work independently of working directory.
* [ ] Help/version and invalid syntax perform no native acquisition; legacy --dll/source aliases are rejected.
* [ ] External JSONL can change playback after restart without rebuilding.
* [ ] Hold/ramp/gap timing has controlled-clock coverage and real DLL integration evidence.
* [ ] Equal values produce successive native observations under flags 0. A healthy hold keeps IAS fresh; an IAS gap allows IAS expiry without refreshing it through unrelated fields.
* [ ] The next pass uses later clock values. Pause and flight state persist across the boundary until explicit scenario steps change them.
* [ ] All used fields, names, units, indexes, request options, and diagnostic variants have independent coverage with assumptions identified.
* [ ] IAS-only and event-only scenarios work without filler signals. Supported parameters absent from the scenario register normally and remain silent.
* [ ] Missing readings remain null/unavailable rather than zero. Preserve current SSE serialization and lifecycle gates.
* [ ] Normal/active pause and active/inactive flight behave distinctly from sample silence and source disconnect.
* [ ] Resume/flight transition waits for a new eligible attitude.
* [ ] API and malformed-packet scenarios exercise isolated optional failure, retry, fatal decoder failure, and cleanup safely.
* [ ] Existing additional native error paths have explicit source, HTTP, and application expectations; host failures use launch/harness conditions.
* [ ] The narrated flight reaches its intended completion, while terminal native-error cases remain isolated in separate files.
* [ ] The scenario catalog summarizes coverage without duplicating scenario values, timing, or event sequence.
* [ ] Two HTTP/SSE clients share one producer and source failure does not close service.
* [ ] An evidence mapping relates checked scenario inputs and existing transformations to observed output.
* [ ] Log/SSE checks identify the exact scenario/run and distinguish native receipt from final publication, without reconstructing a source file.
* [ ] Complete application checks, release delivery, and independent code review are recorded. Applicable screenshots and visual inspection are consolidated into one integrated S8 session, with affected reruns only when needed; intermediate Subtasks do not require separate visual approval.
* [ ] Documentation reflects implemented behavior and distinguishes emulator, live simulator, and device evidence.

## Prior requirements changed by this proposal

| Earlier requirement | Revised requirement |
| --- | --- |
| YAML complete normalized events | JSONL authored native-input instructions |
| Exact every fictional default output | Mapped native scenario values and phases; unsupported display fields remain independently fixture-tested |
| Direct Demo Publisher path | Demo DLL behind the same native consumer |
| Source enum plus optional DLL | DLL-path source selection |
| No native DLL in Demo | Project-owned Demo DLL; no official SDK/runtime dependency |
| Pure absolute replay with historical acquisition ages | Approximate native observation timing with new receiver receipt times |
| Recorded-session conversion, including schemas 4/5 | Removed by explicit user decision; retain historical evidence and verify authored scenario -> result only |
| One package/executable delivery | Existing application package plus one Demo cdylib workspace member |

## Publication constraints

Retain the accepted comment and original Subtask history.
Refresh #123–#126 before deciding their disposition.
Do not close, rewrite, relabel, or replace them without explicit authorization.
Task completion requires the amended scenario-to-output criteria, not a converter. Finished DLL Subtasks alone do not establish the Task result.
