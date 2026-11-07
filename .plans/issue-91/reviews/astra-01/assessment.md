# Root assessment of the Astra architecture review

## Later playback correction

The subsequent explicit requirement resolves the repeat-state question: the clock keeps increasing and playback returns to the first step after the last step.
Pause and flight state change only through scenario records. Header initial state applies once, without automatic restoration between passes.
The former proposal to restore initial lifecycle state is withdrawn and is not an S2 or S3 assignment.
The [current playback contract](../../contracts.md#scripted-signal-tracks) governs execution.
The independent report and frozen input remain historical evidence of the earlier draft; they do not override this correction.

The subsequent sparse-scenario clarification also resolves registration versus missing values.
Supported parameters register independently of scenario coverage. A missing value produces no observation and no implicit zero.
IAS-only, event-only, and no-signal scenarios are valid. Do not reopen a requirement for complete scenario data.
The [current sparse-scenario contract](../../contracts.md#registration-and-sparse-scenarios) governs this decision.
Duplicate-binding validation remains a separate schema detail; it is not a reason to reject absent values.

## Result

Retain the shared-receiver Demo DLL architecture.
The independent review found no P1 architectural blocker.
The review originally identified three P2 contract gaps, not observed defects in implemented Demo code.
The repeat-state and missing-value questions are resolved by the explicit corrections above; other findings retain their separately recorded dispositions.

The plan is not fully implementation-ready.
S1 is structurally scoped, and S2 can resolve the contract gaps after the relevant architecture/requirement decisions and execution authorization.
S3 is gated by those resolutions. S4, S5, and S8 inherit that dependency.
The latest explicit decision removes conversion from #91. S6 now establishes forward scenario-to-output evidence; S7 is removed. Frozen review inputs and the independent report retain their historical wording.

## Review identity and state

| Field | Record |
| --- | --- |
| Requested reviewer | Astra Medium |
| Selected model | `gpt-6-astra`, reasoning effort `medium` |
| Independent report | [Astra report](astra-report.md) |
| Review request | [Scope and constraints](request.md) |
| Reviewed drafts | [Frozen index](input/README.md) |
| File identities | [SHA256 manifest](input-manifest.json) |
| Repository source | `9356e47e481d422c3b771205cd7f3e82f07789b0` |
| Scope | All 29 input files, including seven Subtask bodies/comments, eight scenario drafts, the introductory example, and original issue evidence |

Root rechecked file identities after receiving the report.
The 29 frozen files and all original draft files still match the manifest.
The tracked working tree remains unchanged.
Review outputs were written only to this local review directory.

## Finding assessment

| Finding | Root disposition | Source-supported consequence | Proposed resolution and owner |
| --- | --- | --- | --- |
| Returned-buffer lifetime/immutability | Accepted P2 | The current receiver copies returned bytes after the FFI call. A concurrent emitter must not replace or mutate that exposed storage | S2 states immutable allocated bytes until next dispatch on that connection or Close; S3 implements synchronization/storage under that rule |
| Lifecycle behavior at repeat | Resolved by subsequent explicit requirement | State intentionally persists between passes until an explicit scenario record changes it | S2 documents continuous-clock looping; S3 must not restore header state or synthesize lifecycle transitions. Call-occurrence counters retain their scope |
| Supported registration versus omitted scenario tracks | Resolved by subsequent explicit requirement | Scenarios may supply any subset or no signal values; missing values are intentional | S2/S3 accept supported registrations independently of scenario coverage and emit no invented values. Duplicate bindings remain a separate schema detail |

The report's [three findings](astra-report.md#consequential-unresolved-contracts) contain precise snapshot lines, failure scenarios, and production-code evidence.
No extra Subtask or architecture layer is needed to address them.

## Remaining decisions and qualifications

* Native-input conversion and schema 4/5 support are withdrawn by explicit user decision. Their evidence gaps no longer gate this Task.
* Native-invalid values and invalid JSON/schema values need distinct admissibility rules for S5. Fault inputs must reach the native boundary rather than fail prematurely as malformed authoring.
* The default migration needs an explicit preserved/changed checkpoint classification. Immediate direct-Demo VSI omissions do not automatically match native expiry behavior.
* Indexed SimVar spellings remain compatibility assumptions where independent evidence is incomplete.
* HTTP-first scenario validation was explicitly accepted after this review. Private setup failure before Open leaves HTTP available; E_FAIL is not sent through GetNextDispatch for parser errors.
* Path/name restrictions were explicitly accepted, with case-insensitive basename recognition (including DEMO.dll). Exact parser/ABI choices remain technical proposals.

These items are planning gaps or qualification requirements.
They do not establish a parser, timing, native-memory, simulator, or browser failure observed at runtime.

## Additions beyond explicit requirements

The report's [choice inventory](astra-report.md#added-choices-and-restrictions) distinguishes mechanisms from optional design choices.

| Category | Examples | Root assessment |
| --- | --- | --- |
| Mechanisms needed for requested behavior | Request/definition routing, bounded buffers, a scenario setup channel, transport silence for the watchdog case | Justified by the selected native boundary or explicit error coverage |
| Optional technical choices | Exact ConfigureV1 ABI, one workspace member, dedicated emitter thread, concurrent tracks | Proportionate proposals; not additional user product requirements |
| Added restrictions | Only two basenames, absolute directory paths, no renamed DLLs, exact parser rejection policy | Path/name restrictions are now explicitly accepted, case insensitive; exact parser policy remains a proposal |
| Accepted behavior change | Parse the scenario after HTTP binds | User accepted this after review; failures leave HTTP available |
| Withdrawn behavior | Fixed 100 ms raw-replay cutoff | Removed with recorded-session conversion; no remaining S6 assignment |
| Proposed verification expansion | Host/filesystem/panic/HTTP cases outside DLL control | Reuse existing seams; do not create a general fault framework implicitly |
| Scenario authoring choices | File names, exact trajectories, durations, initial one-call Open failure | Editable examples satisfying the requested narrative, not aircraft-performance requirements |

The review found no hidden second publisher, provider framework, recording protocol, browser controls, physics model, or official DLL redistribution.
Vx, stall, and ground-contact limits are consistent with the explicitly accepted native-only field scope.

## Work performed and limits

* Received and assessed a complete independent Astra Medium report.
* Compared material findings with the draft contracts and current receiver/lifecycle/instrument source.
* Preserved original drafts and frozen inputs; no suggested correction was applied during this review-only request.
* Performed static file/hash inspection only. No application, DLL, scenario reader, test suite, build, browser, renderer, simulator, or device was run.
* The reviewer used frozen issue records and did not refresh remote state. The review applies to the identified snapshot.
* Earlier review reports remain historical evidence. This assessment records the new whole-plan review without rewriting those records.

## Minimal follow-up

1. Carry the explicit continuous-clock playback rule into S2/S3; resolve the remaining contract findings under their current dispositions.
2. Resolve remaining native evidence gaps, especially pause/empty-queue/overflow behavior. Conversion is no longer a gate.
3. Keep remaining optional additions labeled. Subsequent user instructions authorize local corrections, while implementation and GitHub publication remain outside this static update.

This review is complete. Implementation readiness remains qualified as stated above.
