# S2: Record the native Demo architecture and JSONL contracts

## Issue body

Parent: Task #91 under Epic #9. Prerequisite: S1.

Execution: one GPT-6.1 Terra developer at medium. Use Sol Medium consultation for consequential gaps.
Read Task #91, Epic #9, this body, its first context comment, and prerequisite results.
Refresh issue artifacts and repository state before execution.
Follow the repository execution/reporting procedure; report results and gaps in the main conversation.
This is a local draft. It grants no implementation or publication authorization.

### Outcome

Canonical documentation defines the revised native Demo contract and clearly distinguishes planned behavior from the current direct provider.

### Scope

* Record a new proposed/accepted ADR with the actual decision status and links to amended portions of ADRs 0006 and 0007.
* Update `docs/sources/demo.md` with JSONL header, tracks, holds, ramps, gaps, lifecycle, faults, parser rules, and default-data scope.
* Update `docs/sources/simconnect.md` with the common native consumer, used API subset, source identity, setup, and independent-oracle limits.
* Update shared architecture/build guidance and agent maps only where their boundaries change.
* Keep README startup commands current until S4 implements the new CLI.
* Add `docs/sources/demo-scenarios.md` as a purpose-only catalog linked from the Demo source guide.
* Keep exact native-error outcomes in the SimConnect feature owner; do not duplicate scenario values or timelines in the catalog.
* Exclude code changes and GitHub issue edits.

### Context

* Task #91's accepted comment and its explicit amendments from this discussion.
* ADRs 0006, 0007, and 0008; `docs/architecture.md`; both source guides.
* `src/providers/simconnect/native.rs` field catalog, ABI declarations, and decoder.
* `runtime.rs::configure`, `query_states`, and `session`; `lifecycle.rs`; `instruments.rs`.
* Official SimConnect definition, system-event, system-state, Open, units, and per-variable references.
* Current validation matrix and evidence-selection guidance.

### Implementation guidance

1. Preserve the explicit scope decisions: JSONL, DLL clock, native-only source fields, forward input/output verification, normal/active pause, flight active/inactive, and fault injection.
2. Define ordinary signal keys separately from the independent native name/unit catalog and caller-supplied request IDs.
3. Specify half-open segments, continuous-clock looping, missed-frame policy, native delivery evidence limits, and fresh recovery.
4. Record the Demo-only setup call and HTTP-first scenario failure semantics.
   The user accepts this ordering: invalid scenario setup returns E_FAIL with an explanation before Open, leaving HTTP available. Do not report parser errors through GetNextDispatch.
5. Classify catalog entries as independently documented or current-consumer compatibility assumptions.
6. Preserve existing presentation requirements even where the new Demo does not supply a field.
7. Replace earlier planned YAML/full-snapshot requirements explicitly; retain their rationale/history in the decision record.
8. Repair affected links and distinguish public documentation from agent-facing maps.

### Acceptance criteria

* [ ] Review covers the assigned outcome and the prerequisites supplied to the next consumer under the [tasks/tests/review plan](../tasks-tests-review.md). Record evidence, gaps, and a scoped readiness verdict.

* [ ] Five-second hold/ramp instructions have unambiguous timing, units, bounds, and independent track semantics.
* [ ] Native request periods and flags govern delivery. Document current SIM_FRAME/flags-0 emission of unchanged values, continuous hold observations, and signal-specific gap silence.
* [ ] Hold continuously emits unchanged requested values; gap emits no value for that signal.
* [ ] A scenario can supply any subset of signals, events only, or no signal values. Supported registration is independent of supplied values.
* [ ] Missing values produce no observations and default to null/unavailable in the server and UI. Preserve existing SSE omission of optional fields; never substitute zero.
* [ ] Playback returns to the first step after the last step while the clock keeps increasing. There is no end waiting mode.
* [ ] Initial scenario state applies once. Pause and flight state change only through scenario records, including across pass boundaries.
* [ ] Normal pause maps to Pause_EX1=1; active pause maps to 4; off maps to 0.
* [ ] Pause delivery is labeled as an implementation verification requirement: the current assumption repeats frozen values and continues messages; the application's paused publication gate is separate.
* [ ] DLL basename matching is case insensitive, including DEMO.dll. Accepted path restrictions do not change native export spelling.
* [ ] Sim-running, loaded flight, pause, disconnect, and fresh-attitude eligibility remain distinct.
* [ ] Every used field, index, unit, datatype, export, diagnostic definition shape, and request option has an explicit evidence classification.
* [ ] Native Bool semantics include nonzero true; current exact-1 behavior is recorded as a discrepancy rather than copied into the oracle.
* [ ] The docs state which prior requirements change and which remain valid.
* [ ] Recorded-session conversion and schema 4/5 import are removed; S6 owns forward scenario/output evidence mapping.
* [ ] Public documents contain no links into `.context/`; agent maps point to the correct owners.
* [ ] Scenario catalog entries state purpose, assumptions, expected completion, and a link, without copying the JSONL sequence or numbers.
* [ ] Native error documentation distinguishes optional failure, connection retry, producer termination, and whole-application failure.

### Validation

Documentation-only diff, links, anchors, terminology, preserved requirements, and static source/API comparison.
No application build is needed for this outcome.
Report any unavailable rendering.
This planning session did not run rendering or tests.

## First comment: execution context

### Reviewer: readiness for S3

Read [S3's complete requirements and context](S3-demo-dll.md) alongside S2's documented contract, S1's result, and the parent requirements. Apply the [S2-to-S3 transition review](../tasks-tests-review.md#s2-to-s3).

* Check every S3 requirement for a contract prerequisite: records, native names/units/layouts, registration/request semantics, clock/lifecycle behavior, setup errors, and resource ownership.
* Identify contradictions or omissions that would force S3 to invent a consequential rule. Distinguish accepted decisions, technical proposals, documented SDK facts, and implementation verification gaps.
* For each native evidence gap, require a bounded investigation and identify the dependent behavior that cannot yet be claimed compatible. Documentation approval does not establish runtime parity.
* Report whether S3 has sufficient input to implement the resolved portions and investigate the named gaps. Do not approve readiness solely because the Markdown is well formed.

### Contract context

The selected JSONL format is an instruction document, not a line-by-line normalized telemetry dump.
Use a required format/version header and whole-file prevalidation.
One complete JSON object occupies each physical line.

Loop semantics are decided, not an open state-reset design task.
Demo DLL is a playback mechanism only. Never reset its clock or restore initial lifecycle state at a pass boundary.
Schedule the same steps at later absolute deadlines and preserve state until an explicit step changes it.
Do not synthesize resume, flight activation, or flight-load events to make the next pass look like the first.

Sparse scenarios are also decided. Do not reopen a requirement to supply every registered field.
An IAS-only scenario has no obligation to provide attitude or altitude; event-only scenarios need no instrument tracks.
Cover the complete native catalog across the suite rather than demanding complete values in each file.

The current application requests periodic observations with flags 0, not change-only delivery.
Record that equal values are separate acquisitions and refresh freshness; only absent observations allow the per-field timeout to expire.
The synthetic frame period represents simulation cadence, not a replacement for native request semantics or the application's dispatch frequency.

The intended ordinary records are scenario header, signal declaration, hold, ramp, gap, pause, flight activation, flight-loaded change, and disconnect interval.
API-failure and malformed-packet records are explicit opt-in fixtures.
Reject unknown fields/types, duplicate keys, invalid numbers, and overflowing timelines.

The Demo DLL consumes the selected scenario through a private versioned setup export before Open.
The official DLL exposes only the existing required SimConnect API; never call Demo setup on it.
Setup must not publish or create another acquisition path.

The new default supports current native inputs only.
It no longer supplies fictional VFE/VNO/VNE or VX/VY/VG and no longer injects analytical IAS acceleration.
Acceleration remains the existing receiver estimator.
Existing full-profile presentation fixtures retain their purpose.

Independent-source gaps found during planning:

* AUTOPILOT target slot 0 is described by slot documentation, but the exact indexed target reads are not conclusively established by the variable tables.
* INDICATED ALTITUDE is shown unindexed in the inspected reference. The existing :1 suffix is not independently established.
* KOHLSMAN HG:1 is explicitly documented as the first altimeter.
* Bool documentation permits nonzero true, including -1; current AUTOPILOT AVAILABLE handling accepts only 1.

Retain uncertain spellings as labeled compatibility assumptions until stronger evidence exists.
Do not describe an emulator passing those assumed spellings as proof of SDK conformance.
No simulator experiment is authorized during current planning.
