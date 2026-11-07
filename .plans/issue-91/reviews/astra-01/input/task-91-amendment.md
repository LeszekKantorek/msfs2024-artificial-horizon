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
* Cover every currently consumed signal, unit, index, datatype, request key, and lifecycle name against an independent catalog.
* Restrict Demo data to the current real native API surface. Preserve separate presentation-fixture coverage for additional display fields.
* Simulate normal pause, active pause, active flight, inactive flight, loaded-flight changes, gaps, disconnect, and fresh-data recovery.
* Include deterministic API errors, asynchronous exceptions, malformed packets, invalid numbers, and cleanup failures.
* Supply several scenarios: a narrated flight, focused lifecycle/availability inputs, and isolated potentially terminating errors.
* Demonstrate runway acceleration, rotation, climb, speed loss/stall-like descent, recovery, turns, approach/landing, both pause modes, and simulator availability.
* Use existing native fields for that narrative; add no Vx/stall/ground-contact input or flight dynamics.
* Keep the mapped default/boundary scenario unless its role is explicitly changed. The flight tour is an additional explicit selection.
* Maintain `docs/sources/demo-scenarios.md` with purposes, assumptions, expected completion, and file links only.
* Keep scenario details in JSONL and independent expected checkpoints in tests/fixtures.
* Retain offline conversion of explicitly selected recorded sessions into the same native-input JSONL format.
* Re-establish conversion support and fidelity at this changed boundary before implementing the converter.
* Preserve one receiver, one active publisher, current estimator, latest-state distribution, independent availability, freshness, and coordinated shutdown.
* Rebuild/deliver the application EXE and project-owned Demo DLL together.
* Keep official SDK artifacts and generated real-session data outside commits.

## Source and timing boundaries

A five-second hold sends successive fresh observations.
A gap sends no observations.
Recorded observations are scheduled once instead of converted into continuously refreshed holds.

Demo emits native units and lifecycle messages.
The current consumer computes normalized telemetry, age, optional gates, and IAS acceleration.
Its receipt clock means exact original sample ages, estimator outputs, and SSE publication history are not promised.

HTTP binds before loading the selected DLL.
Scenario setup validates before simulated Open; setup failure leaves HTTP available with unavailable source data.
This explicitly changes the earlier planned pre-bind scenario-validation requirement.

## Acceptance criteria

* [ ] A workspace build produces the application EXE and demo.dll without simulator/SDK dependencies.
* [ ] Both DLL selections use the actual same native receiver and fixed source identity.
* [ ] Default selection and explicit absolute DLL selection work independently of working directory.
* [ ] Help/version and invalid syntax perform no native acquisition; legacy --dll/source aliases are rejected.
* [ ] External JSONL can change playback after restart without rebuilding.
* [ ] Hold/ramp/gap timing has controlled-clock coverage and real DLL integration evidence.
* [ ] All used fields, names, units, indexes, request options, and diagnostic variants have independent coverage with assumptions identified.
* [ ] Normal/active pause and active/inactive flight behave distinctly from sample silence and source disconnect.
* [ ] Resume/flight transition waits for a new eligible attitude.
* [ ] API and malformed-packet scenarios exercise isolated optional failure, retry, fatal decoder failure, and cleanup safely.
* [ ] Existing additional native error paths have explicit source, HTTP, and application expectations; host failures use launch/harness conditions.
* [ ] The narrated flight reaches its intended completion, while terminal native-error cases remain isolated in separate files.
* [ ] The scenario catalog summarizes coverage without duplicating scenario values, timing, or event sequence.
* [ ] Two HTTP/SSE clients share one producer and source failure does not close service.
* [ ] Required recorded-session schemas and fidelity have an explicit evidence-based mapping.
* [ ] The implemented converter preserves the supported observations, event order, gaps, and availability through converted-file playback.
* [ ] Complete application checks, release delivery, applicable browser captures, and independent code review are recorded.
* [ ] Documentation reflects implemented behavior and distinguishes emulator, live simulator, and device evidence.

## Prior requirements changed by this proposal

| Earlier requirement | Revised requirement |
| --- | --- |
| YAML complete normalized events | JSONL authored instructions or recorded native observations |
| Exact every fictional default output | Mapped native scenario values and phases; unsupported display fields remain independently fixture-tested |
| Direct Demo Publisher path | Demo DLL behind the same native consumer |
| Source enum plus optional DLL | DLL-path source selection |
| No native DLL in Demo | Project-owned Demo DLL; no official SDK/runtime dependency |
| Pure absolute replay with historical acquisition ages | Approximate native observation timing with new receiver receipt times |
| Schema 4/5 normalized conversion mapping already sufficient | New raw-input mapping required; preserve earlier result as historical evidence |
| One package/executable delivery | Existing application package plus one Demo cdylib workspace member |

## Publication constraints

Retain the accepted comment and original Subtask history.
Refresh #123–#126 before deciding their disposition.
Do not close, rewrite, relabel, or replace them without explicit authorization.
Task completion still requires conversion and the Task's own criteria; finished DLL Subtasks alone do not close #91.
