# S6: Establish scenario-to-output evidence

## Issue body

Parent: Task #91 under Epic #9. Prerequisite: S2. S4 supplies integrated evidence for the resulting mapping; S8 uses its checkpoints.

Execution: one GPT-6.1 Terra developer at medium. Consult Sol Medium for consequential gaps.
Read the amended Task, Epic, first context comment, prerequisite results, and current issue artifacts before execution.
Follow repository execution/reporting rules and report results and gaps in the main conversation.
This local draft grants no implementation or publication authorization.

### Outcome

A bounded field/checkpoint mapping lets implementation verify that an observed log/SSE result follows from the selected Demo scenario.
This assignment defines reusable acceptance evidence; routine test execution stays with S3–S5 and S8.

### Scope

* Map authored native values to receipt diagnostics and normalized output, including units, signs, gates, expiry, and existing derived values.
* Define independent expected checkpoints and evidence sufficient to identify the scenario, run, and relevant segment/pass window.
* Use existing diagnostic records and SSE capture. Exclude recorded-flight conversion, schema 4/5 import support, reverse reconstruction, a new recording protocol, and private native timestamp fields.

### Context

* The amended #91 requirement is forward verification only; the earlier converter requirement is explicitly withdrawn.
* S2's signal catalog, continuous-clock schedule, hold/gap rules, and native API contract.
* Current native decoding, instrument normalization/estimator, lifecycle gates, journal record stages, and SSE full-snapshot publication.
* Existing independent fixtures and scenario data ownership.

### Implementation guidance

1. Identify which existing log records show received input and which show final output. Use SSE capture where no final-output log exists.
2. Map direct fields and computed outputs separately. Explain why a ramp's intermediate value or normalized angle need not occur literally in JSONL.
3. Specify a small set of independently calculated checkpoints: hold, ramp, gap, pause/resume, and another pass on the increasing clock.
4. Record scenario hash, application/DLL identity, run start reference, and relevant time window in acceptance evidence. Keep this outside the official ABI.
5. Separate scheduler/receipt tolerance from numeric tolerances; derive justified bounds during implementation rather than inheriting the withdrawn 100 ms recording-replay proposal.
6. Give S4/S8 the mapping and expected observations. Reuse existing test seams; do not build a general tracing framework.

### Acceptance criteria

* [ ] Review covers the assigned outcome and the prerequisites supplied to the next consumer under the [tasks/tests/review plan](../tasks-tests-review.md). Record evidence, gaps, and a scoped readiness verdict.

* [ ] Each checked final value can be related to the authored input or an explicit existing derivation.
* [ ] Expected values are independent of the parser/normalizer being checked.
* [ ] Raw receipt evidence is distinguished from final log/SSE evidence.
* [ ] Repeated equal hold observations refresh only their own field; gaps create no fabricated values.
* [ ] Missing readings remain unavailable under existing lifecycle and attitude gates.
* [ ] Looping preserves increasing time; pass identification in evidence does not reset playback state.
* [ ] No one-to-one native-packet/SSE count or exact wall-clock equality is assumed.
* [ ] No converter, diagnostic-schema migration, or original-scenario reconstruction is assigned.

### Validation

Review the mapping against current code and independently calculated examples. Integrated checks belong to the implementation outcomes that consume it.
Report missing observability and timing evidence explicitly. This planning session performs static inspection only, with no tests, builds, or execution.

## First comment: execution context

Reviewer: apply the [S6 transition row](../tasks-tests-review.md#tasks-checks-and-handoff-reviews) against S4/S8 verification requirements. Confirm that the mapping supplies usable independent expected results and identifies the observation stage and evidence gaps. S4 supplies later integrated evidence; it is not a prerequisite for defining this mapping.

The user wants to see a value in the resulting log and establish that it came from the Demo input. The required direction is scenario -> DLL -> application -> result.
For example, an IAS hold at 100 should explain received IAS=100; a ramp from 100 to 150 over five seconds should explain intermediate values in the appropriate time window. Existing normalization may change units or signs, and IAS acceleration is derived from received history.

Latest-state SSE can coalesce updates, while diagnostics may record raw inputs that the lifecycle gate does not publish. Therefore a native receipt log alone cannot certify a visible value. Keep source availability and eligible fresh attitude in expected results, without requiring filler values in sparse scenarios.

Earlier schema 4/5 work concerned importing recorded flights. It is historical context only. This assignment neither reconstructs a source file from SSE nor creates a replacement converter.
