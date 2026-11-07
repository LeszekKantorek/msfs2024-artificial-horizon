# Independent architecture and Subtask review for Task #91

Architecture verdict: retain the proposed Demo DLL behind the existing SimConnect receiver. It meets the requested boundary without another normalization or publication path. No P1 architectural contradiction was established.

Readiness verdict: S1 and the S2 contract work are structurally ready for later authorization. Resolve the three P2 contract gaps below before treating S3 as implementation-ready. S4, S5, and S8 inherit that dependency. S6 is a valid bounded investigation; S7 remains deferred.

## Scope and evidence

* Reviewed the frozen `input/` snapshot, including all authored Markdown, seven Subtask bodies and first comments, nine JSONL files, original Task/Epic/accepted-comment records, and the remote metadata snapshot.
* Checked all 29 files against `input-manifest.json`: zero SHA256 mismatches. Local HEAD was `9356e47e481d422c3b771205cd7f3e82f07789b0`; `git status --short` returned no tracked changes.
* Compared relevant native registration, dispatch, lifecycle, instrument, and direct-Demo source with the drafts. Read the applicable context maps, architecture/planning procedures, writing procedures, and ADRs 0006 and 0007.
* Evaluated the design independently before reading the earlier `input/review.md`. That earlier review is historical evidence, not independent verification of this report.
* Used static inspection and read-only file/hash operations. No tests, builds, application/DLL execution, parser or scheduler prototypes, browsers, rendering, simulator checks, or GitHub mutations occurred. No subagents were spawned.
* Used the supplied frozen remote records rather than refreshing GitHub. The report therefore assesses that snapshot, not subsequent remote changes. Official API claims in the supplied evidence were not independently reverified online.

Snapshot locations below are relative to `input/`. Production source references are relative to the repository root at the inspected revision.

## Confirmed contradictions

No confirmed contradiction requires rejecting the selected architecture. The intended changes from YAML, direct normalized publication, full fictional profiles, and the previous CLI are explicitly recorded.

The HTTP-first scenario-error behavior is also explicitly identified as a proposed change from accepted pre-bind validation. It still needs acceptance as a requirement change; it is not a concealed implementation detail.

## Consequential unresolved contracts

### P2: State the returned-buffer lifetime across the emitter and dispatch threads

**Location:** `contracts.md:254`, `contracts.md:279`, and `subtasks/S3-demo-dll.md:41`.

The contract gives the DLL an emitter thread and receive buffers. It also requires refreshing pending scripted values at dispatch. It does not state when returned storage may be mutated or released.

**Failure scenario:** GetNextDispatch returns a pointer into an owned pending buffer. Before the receiver finishes copying it, the emitter refreshes or replaces that buffer. Owned allocation and valid lengths alone do not prevent a data race, inconsistent packet, or dangling pointer.

**Evidence:** `src/providers/simconnect/native.rs:481` relies on the SDK buffer remaining valid until the next dispatch and immediately calls `decode_received`. The consumer cannot hold an internal DLL lock while reading the returned bytes. Length checks do not address concurrent mutation.

**Minimal correction:** State that the returned byte range remains allocated and immutable until the next GetNextDispatch on that connection or Close. The emitter may modify pending state, but not the currently exposed allocation. Require the same rule for injected raw packets. Leave the storage and synchronization implementation to S3.

**Blocked work:** S3 buffer/emitter implementation and its dependent integration. S2 can resolve this directly from the existing receiver contract; no new architecture or runtime experiment is required.

This is a missing producer-side ownership guarantee, not evidence of an existing application memory defect.

### P2: Define lifecycle state and delivery order at a scenario repeat boundary

**Location:** `contracts.md:41`, `contracts.md:72`, and `contracts.md:286`.

“Restart the scenario timeline” defines time rebasing but leaves lifecycle reset behavior implicit. The contract distinguishes reconnect persistence and Configure-lifetime fault counters, but does not say how `initial` applies on subsequent cycles.

**Failure scenario:** A repeating scenario begins active and unpaused, then ends paused or with no loaded flight. Restarting only tracks and event offsets leaves the receiver paused or waiting in the next cycle. Resetting internal state without emitting corresponding native transitions leaves queries and the receiver temporarily inconsistent.

**Evidence:** `src/providers/simconnect/lifecycle.rs:27`, `:49`, `:57`, and `:67` retain Sim, pause, and loaded-flight state until native messages update them. New numeric observations alone cannot undo a pause. The current example ends with restored state, so it does not distinguish these interpretations.

**Minimal correction:** Specify whether each cycle restores header `initial` state, which subscribed transitions precede the new cycle's data, and whether unchanged state needs an event. State that ordinary timeline events repeat while Configure-lifetime call counters retain their documented scope. Keep new-cycle data behind any required lifecycle reset. Add one later focused repeat-boundary check with different terminal and initial states.

**Blocked work:** S3 repeat/lifecycle behavior and dependent S4 acceptance. This does not block S1 or the S6 investigation.

### P2: Separate supported field registration from scenario data availability

**Location:** `contracts.md:92`, `subtasks/S3-demo-dll.md:39`, and `scenarios/errors/optional-definition.jsonl:2`.

The independent catalog defines recognized fields, and gaps define temporary silence. The contract does not explicitly define a recognized field that has no signal track in the selected file. Most focused drafts intentionally omit most catalog fields.

**Failure scenario:** A catalog lookup is implemented by searching the scenario's declared tracks. The supplied optional-definition scenario then fails registrations for every omitted optional field, not only the injected IAS failure. Filling omitted fields with default values would instead fabricate observations. Both interpretations undermine isolated error checks.

**Evidence:** `src/providers/simconnect/runtime.rs:626` attempts all 17 optional definitions for every connection. `scenarios/errors/optional-definition.jsonl` declares only pitch, bank, and IAS. `scenarios/errors/transport-silence.jsonl` declares only attitude. These are legitimate sparse scenarios, not complete native field catalogs.

**Minimal correction:** Define registration against the independent supported catalog regardless of authored tracks. A recognized field without a track supplies no observations; it does not receive an implicit value or registration error. Apply the existing grouped-packet suppression rule when a requested component is absent. Reserve definition/request failures for unsupported client arguments or explicit faults. Define rejection of conflicting duplicate signal bindings so one native field has one ordinary value source.

**Blocked work:** S3 scenario-to-registration behavior; S5 isolated optional-failure assertions; S8 focused-scenario acceptance. No additional scenario feature is needed.

## Other decisions and limits

| Priority | Location | Assessment and required disposition |
| --- | --- | --- |
| P2 gate, already acknowledged | `subtasks/S6-conversion-evidence.md:45`; `plan.md`, deferred converter assignment | Original accepted conversion supports schemas 4 and 5. Native replay invalidates the old sufficiency argument, so S6 is justified. Missing schema-5 evidence must leave support unconfirmed; it must not silently redefine Task completion as schema-4-only. If support proves infeasible, return that requirement change for a decision before preparing S7. S1–S5 and S8 need not wait. |
| P3 clarification | `contracts.md:47`, `contracts.md:185`, and `contracts.md:220` | Separate invalid JSON/schema numbers from deliberately invalid native payloads. S5 can use owned raw packets for nonfinite input, but ordinary out-of-range native values need a stated admissibility rule. Do not copy normalized telemetry validation into all native scenario values and thereby reject the requested fault before dispatch. Resolve in S2/S5; no new value language is necessary. |
| P3 preservation check | `subtasks/S3-demo-dll.md:44`; `contracts.md:85` | “Mapped default phases and checkpoints” needs a concise migration classification during S3. Existing immediate VSI omissions and target changes may differ because native fields expire or are gated. The draft already allows intentional availability/estimator differences; record those differences beside preserved checkpoints rather than silently weakening assertions. Exact full-profile equivalence is expressly superseded. |
| Nonblocking | `subtasks/S2-documentation.md:89` | Indexed SimVar ambiguities are correctly labeled compatibility assumptions. S2 must retain that classification. Passing the emulator cannot independently establish that the simulator accepts those spellings. |

## Ownership, dependencies, and proportionality

* S1 has a concrete repository outcome and preserves application behavior. An initially nonfunctional DLL artifact is appropriate for that stage and is labeled accurately.
* S2 owns durable contracts before dependent code. The three P2 clarifications belong there; they do not justify extra Subtasks.
* S3 owns the healthy DLL, S4 the common application receiver, and S5 deliberate faults. Their scopes are coherent. S3 can test reconnect state through its independent client without taking ownership of S5's fault language.
* S6 can proceed after S2 and does not need completed DLL implementation to identify recording information loss. Runtime fidelity questions can remain explicit if prohibited experiments prevent resolution.
* S7 has no implementation-ready body and must stay deferred. Conversion remains part of Task completion.
* S8 appropriately follows S5 because the requested tour uses recoverable Open/QUIT behavior. The catalog remains purpose-oriented; numeric timelines stay in JSONL.
* The narrative uses only the native subset and separates fatal cases. Static inspection supports its intended rotation/climb/descent/recovery/turn sequence, not aerodynamic correctness or observable browser timing.
* Complete build and browser evidence requirements are assigned to later implementation. Controlled-clock checks and real-loader checks serve different purposes. No additional general host-fault framework is justified.

## Added choices and restrictions

These choices should remain visible for discussion. Their inclusion in a proposal does not make each a direct user requirement.

| Choice | Classification | Assessment |
| --- | --- | --- |
| Native registration tables, caller IDs, packet encoding, lifecycle state, bounded native buffers | Necessary mechanisms | Required to exercise the used SimConnect consumer accurately |
| External-scenario setup channel | Necessary mechanism; exact ABI is optional | ConfigureV1 is a focused solution. Its exact signature and versioning are author choices |
| Dedicated emitter thread | Optional implementation proposal | Fits native notification testing; monotonic ownership does not itself require this exact threading design |
| One workspace cdylib member | Optional structural proposal | Justified by the second delivered artifact; explicitly amends single-package guidance |
| Adjacent default demo.dll and embedded default JSONL | Proposed deployment choices with inherited intent | Preserve simulator-free default startup and deployment independence |
| Two accepted basenames, absolute directory paths only, rejection of renamed DLLs | Optional restrictions | Simplify identity and resolution. They are not required by the requested `--source <dll>` syntax |
| HTTP-first parsing and source failure on malformed scenario | Proposed behavior change | Changes accepted pre-bind failure semantics and needs explicit acceptance before dependent execution |
| Concurrent tracks and individual repeats | Optional schema design | Proportionate to independent existing cycles; avoids enumerated snapshots |
| Scripted/observations mode exclusivity | Optional restriction | Reasonable protection against accidental refresh; final observations contract belongs to S6/S7 |
| Whole-file parsing, strict keys, blank-line rejection, finite budgets, ms/us fields | Parser and authoring choices | Deterministic rejection is useful. Exact syntax restrictions and budgets are not separate product requirements |
| Fixed 100 ms observation lateness cutoff | Unselected proposal | Correctly retained for comparison rather than imposed on S3/S4 |
| Bool=-1 check and focused correction | Discovered contract work | Relevant to the explicit native oracle goal; not permission for broader instrument changes |
| Transport silence including state replies | Necessary mechanism for the selected watchdog check | Ordinary signal gaps cannot trigger that condition reliably |
| Exact suite names, trajectories, timing, one-call startup failure | Optional scenario choices | Satisfy the requested narrative but remain editable author decisions |
| Additional loader, journal, host, panic, and HTTP tests | Proposed verification expansion | Reuse existing seams. Missing seams should be reported before adding infrastructure |
| Replacing direct-Demo helper-dependent tests | Consequential migration choice | Removing the production branch is necessary. The disposition of public helpers and independent fixture assertions should remain explicit |

No hidden provider framework, second publisher, recording protocol, browser control, physics model, or official DLL redistribution was found in the proposal.

## Rejected or unsupported concerns

* A separate wrapper for the official DLL is unnecessary: the current receiver already resolves the eight exports dynamically.
* The native DLL does not need fictional Vx, stall-warning, or ground-contact fields to demonstrate the requested narrative. Those exclusions match the accepted native subset.
* Exact original ages, estimator values, or SSE histories cannot be assumed from new native receipt times. The draft appropriately limits that promise instead of adding a timestamp backchannel.
* The earlier clock-origin and half-open endpoint issues are addressed in the frozen contract. They are not repeated as open findings.
* The 100 ms replay policy is not currently a selected acceptance requirement.
* Sparse scenarios are not intrinsically defective; the missing contract is how their absent fields interact with registration.
* No crash, parser failure, timing failure, or simulator incompatibility was observed. This report identifies planning risks and source-supported consequences only.

## Minimal next changes

1. Add returned-buffer immutability/lifetime, repeat lifecycle reset, and absent-track registration semantics to the S2 contract handoff.
2. Keep the S6 schema-support decision gate explicit and preserve S7's deferred status.
3. Retain the added-choice inventory and obtain the relevant requirement decisions, particularly HTTP-first parse failure, before execution.
4. During later S3/S5 implementation, distinguish parser rejection from native fault values and record the default availability mapping.

No additional package, Subtask, runtime feature, or present-session execution is recommended. The report received static text and structure inspection only; Markdown rendering was not performed.
