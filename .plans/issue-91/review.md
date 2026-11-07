# Independent planning review

Status: Static review completed by the clarification/proportionality subagent, separately from draft authorship.

Historical review: later explicit decisions remove recorded-session conversion and its S7, replace S6 with forward scenario/output evidence, accept HTTP-first validation and case-insensitive DLL path/name rules, and permit staged fault coverage. References below to converter gates or a 100 ms replay cutoff are superseded, not current assignments. See the [current decision index](README.md#discussion-decisions).

This report records earlier planning stages. The [current playback contract](contracts.md#scripted-signal-tracks) supersedes any earlier clock-origin or repeat-state proposal.
Playback loops its steps on a continuously increasing clock; no automatic pause or flight-state restoration is assigned to implementation.

## Scope and method

The reviewer inspected the architecture, contracts, plan, evidence, Task amendment, and all six Subtask drafts.
The review assessed correctness, executable context, and additions beyond the requested behavior.
No project code, tests, builds, DLL loading, browser, Markdown renderer, or simulator was run.

## Findings and disposition

| Finding | Assessment | Draft correction or remaining limit |
| --- | --- | --- |
| P1: Clock origin could consume observations during registration | Valid | Configure/Open do not start time. Start once at first GetNextDispatch after a successful periodic request; preserve origin across reconnects |
| P2: Fixed 100 ms replay abort lacks evidence and delivery boundary | Valid | Retained as an explicitly unselected proposal for S6 comparison. Removed from S3/S4 acceptance |
| P2: Half-open repeated ramp does not emit its endpoint | Valid | Added an endpoint hold and documented the two-segment boundary behavior |
| P2: S7 is not executable yet | Valid planning limit | Index/plan explicitly state six prepared handoffs plus deferred converter implementation after S6 |

The post-review raw fault vocabulary clarifies argument names and memory constraints.
It does not add another fault family or change user scope.
The independent reviewer rechecked the corrected passages and found no further material issues in that focused static check.
The deferred raw-replay policy and converter handoff remain explicit limitations.

## Added design choices retained for discussion

These are author proposals or technical mechanisms, not additional user requirements:

| Added choice | Reason | Status |
| --- | --- | --- |
| One workspace cdylib package | Produce a real native DLL while keeping synthetic code outside the application package | Recommended structural change; amends single-package guidance |
| Adjacent default demo.dll | Preserve existing simulator-free default startup | Recommended default |
| Only two supported basenames; reject renamed absolute DLLs | Keep source identity/configuration bounded without an identity API | Proposed restriction; arbitrary names are not technically impossible |
| Private ConfigureV1 setup export | Select external scenario and report parsing errors without abusing SimConnect arguments | Recommended mechanism, additional to the eight SDK exports |
| Concurrent named tracks and per-track repeats | Preserve existing independent cycles without duplicated global phases | Recommended scenario structure |
| Separate scripted/observations modes | Prevent converting cached recorded data into fresh repeated holds | Proposed schema constraint; final replay mode awaits S6 |
| Whole-file prevalidation and strict parser rules | Reject incomplete/ambiguous scenario data before simulated Open | Proposed parser policy; exact size caps remain implementation details |
| Milliseconds for author durations, microseconds for recorded/event times | Keep hold/ramp authoring simple while retaining existing log precision | Proposed schema units, explicitly named in fields |
| Parse after HTTP bind | Keep native-owned preparation and current HTTP-first isolation | Proposed change to the previous accepted pre-bind scenario rule |
| Dedicated notification emitter thread | Exercise the current native event/wait mechanism | Recommended implementation choice |
| 100 ms fatal replay lateness budget | Prevent delayed archived input appearing newly acquired | Unselected alternative; evidence absent, S6 must compare policies |
| Bool=-1 regression and focused correction | Independent native documentation exposes an exact-1 consumer assumption | Relevant discovered contract issue; no code correction performed |
| Controlled clocks and child-process Close-failure tests | Verify timing and deliberately retained native state at useful boundaries | Proposed verification methods for later implementation |

These choices remain visible instead of being silently removed after review.

## Explicit requested or inherited scope

The following items are not accidental additions:

* JSONL scenarios with hold/ramp duration instructions and a DLL-owned approximate clock.
* One native consumer for official and Demo DLL selections.
* Existing scenario authoring and recorded-session conversion.
* Restriction to the currently consumed real SimConnect data surface.
* API errors and malformed-packet scenarios.
* Normal pause, active pause, active flight, and inactive flight.
* Independent name/unit/request verification, source identity, lifecycle, freshness, and shutdown.
* SDK-free Demo and retained simulator/device evidence limits.

## Remaining limits

* Final raw-conversion mapping and supported schema-5 evidence remain unavailable.
* Some indexed SimVar interpretations remain compatibility assumptions.
* Architecture, private setup ABI, default/path rules, and parser policy are proposals, not implemented behavior.
* Static text inspection does not establish rendered Markdown/Mermaid or runnable JSONL correctness.

## Multi-scenario amendment review

The independent reviewer inspected the purpose-only catalog, existing-error inventory, all eight JSONL drafts, S8, and related plan/contract/S5 changes.
The review remained static; it did not run a parser, scheduler, application, test, or renderer.

| Finding | Disposition |
| --- | --- |
| Initial versus periodic state-query failure was incorrectly combined | Corrected against runtime.rs:844–850. Initial failure is fatal; later native query failure reconnects after successful cleanup |
| Catalog could duplicate executable data | Reviewer found no copied timeline or numeric phase table; details remain in JSONL |
| Flight narrative could imply unsupported native capabilities | Explicit Vx, stall-warning, runway, and ground-contact limits retained |
| Terminal faults could prevent the rest of the narrative | Separate error files retained; normal tour contains only recoverable connection cases |

Static inspection found aligned track durations and coherent authored altitude/VSI and heading/turn relationships.
This is not production-reader validation, aerodynamic evidence, or observed painted behavior.

Additional technical choices remain proposals:

* The representative suite size, filenames, exact trajectories, and initial one-call Open failure.
* A bounded transport_silence record to suppress state replies as well as data for watchdog coverage.
* The additional flight tour remains an explicit selection; the existing mapped default/boundary scenario keeps its role.
* Host/filesystem/panic/HTTP entries identify verification boundaries. New host-fault infrastructure is not implicitly authorized.
