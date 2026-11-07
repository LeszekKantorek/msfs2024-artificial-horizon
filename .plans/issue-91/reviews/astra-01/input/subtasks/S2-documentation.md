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
* Update `docs/sources/demo.md` with JSONL header, tracks, holds, ramps, gaps, observations, lifecycle, faults, parser rules, and default-data scope.
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

1. Preserve the explicit scope decisions: JSONL, DLL clock, native-only source fields, conversion, normal/active pause, flight active/inactive, and fault injection.
2. Define ordinary signal keys separately from the independent native name/unit catalog and caller-supplied request IDs.
3. Specify half-open segments, repeat/end behavior, missed-frame policy, observation timing limits, and fresh recovery.
4. Record the Demo-only setup call and HTTP-first scenario failure semantics.
5. Classify catalog entries as independently documented or current-consumer compatibility assumptions.
6. Preserve existing presentation requirements even where the new Demo does not supply a field.
7. Replace earlier planned YAML/full-snapshot requirements explicitly; retain their rationale/history in the decision record.
8. Repair affected links and distinguish public documentation from agent-facing maps.

### Acceptance criteria

* [ ] Five-second hold/ramp instructions have unambiguous timing, units, bounds, and independent track semantics.
* [ ] Hold, gap, and one-shot recorded observation cannot be confused.
* [ ] Normal pause maps to Pause_EX1=1; active pause maps to 4; off maps to 0.
* [ ] Sim-running, loaded flight, pause, disconnect, and fresh-attitude eligibility remain distinct.
* [ ] Every used field, index, unit, datatype, export, diagnostic definition shape, and request option has an explicit evidence classification.
* [ ] Native Bool semantics include nonzero true; current exact-1 behavior is recorded as a discrepancy rather than copied into the oracle.
* [ ] The docs state which prior requirements change and which remain valid.
* [ ] Conversion support remains pending its new raw-input mapping result; no exact historical-publication guarantee is invented.
* [ ] Public documents contain no links into `.context/`; agent maps point to the correct owners.
* [ ] Scenario catalog entries state purpose, assumptions, expected completion, and a link, without copying the JSONL sequence or numbers.
* [ ] Native error documentation distinguishes optional failure, connection retry, producer termination, and whole-application failure.

### Validation

Documentation-only diff, links, anchors, terminology, preserved requirements, and static source/API comparison.
No application build is needed for this outcome.
Report any unavailable rendering.
This planning session did not run rendering or tests.

## First comment: execution context

The selected JSONL format is an instruction document, not a line-by-line normalized telemetry dump.
Use a required format/version header and whole-file prevalidation.
One complete JSON object occupies each physical line.

The intended ordinary records are scenario header, signal declaration, hold, ramp, gap, pause, flight activation, flight-loaded change, disconnect interval, and recorded observation.
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
