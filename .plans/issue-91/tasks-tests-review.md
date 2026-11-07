# Task, test, and review execution plan

Status: Planned execution and review requirements for Task #91. No implementation checks or reviews are claimed by this document.

## Review the result and its next use

Each Subtask review checks two outcomes: the assigned work satisfies its own requirements, and its deliverables satisfy the prerequisites that the next Subtask needs from it. Read the complete successor instructions to identify those prerequisites, including its first context comment and applicable amendments.

The successor's implementation is not required early. For example, S1 supplies the actual workspace and artifact facts needed to document the system in S2; it does not implement S2's scenario contract. S3 supplies a DLL that can be integrated in S4; its independent DLL-client evidence is not yet production HTTP/SSE evidence.

Use the [delivery dependencies](plan.md#assignments-and-dependencies) for order. Tests and review remain part of each Subtask; this plan creates no additional task or review layer. S6 can define the evidence mapping after S2 while S3 proceeds. S4 consumes that mapping for its output verification; S6 does not wait for S4 to implement its mapping, so there is no circular prerequisite.

## Execution sequence for each Subtask

1. Read the parent requirements, current Subtask, predecessor handoff, and successor instructions. Use context-retrieve through Luna High under the repository's AGENTS.md to locate the relevant sources; the implementing agent remains responsible for interpreting them.
2. Identify which successor requirements depend on this Subtask. Record the expected artifact or contract and the evidence that can establish it at this stage. Distinguish later implementation obligations from missing prerequisites.
3. Implement only the assigned outcome. Run its required checks during authorized implementation and record the exact revision, inputs, outputs, and missing evidence. Documentation outcomes use static checks rather than application builds.
4. Request independent review of the exact diff and integrated state. Include the parent, current and successor requirements, predecessor results, test evidence, known gaps, and the transition checks below. The reviewer inspects relevant unchanged code where it consumes or constrains the changed boundary.
5. Resolve findings in their owning scope, repeat affected checks, and review material corrections. Record proposed requirement changes for decision instead of silently expanding implementation.
6. Publish the review and handoff result in the owning Subtask: current acceptance, successor readiness, evidence, blockers, and deferred checks. At successor startup, verify the actual predecessor revision and result rather than relying only on a closed issue or passing check.

The current preparation remains static only: this sequence describes future execution and does not authorize running tests now.

## Visual verification schedule

The explicit Task #91 amendment consolidates screenshots and visual verification into one integrated session in S8. Do not request screenshots, run screenshot-producing browser suites, or require manual visual approval after every Subtask.

* S1-S6 retain the unit, boundary, integration, build, and static review checks applicable to their changes. Review of prerequisites is a code/contract/evidence review, not a visual acceptance checkpoint.
* S4 updates browser integration and runner assumptions but hands visual verification to S8. S5 adds affected display states to that same final scope rather than scheduling another capture session.
* S8 runs the applicable browser scripts against the integrated delivery and inspects representative matched captures. Cover the default, selected scenario, and relevant availability/lifecycle states; do not require a screenshot for every scenario step or API fault.
* If the final session reveals a defect, correct it and repeat the affected visual checks. Do not repeat the whole visual session after an unrelated change without a new reason.
* Before S8, label appearance as unverified. Earlier Subtasks may hand off their supported nonvisual outcomes, but their reviews must not claim visual acceptance; #91's final visual criterion remains pending.

This is a Task-specific scheduling amendment to the earlier per-change capture instructions. It changes when visual evidence is collected, not the expected final display behavior or the separate limits of simulator/device evidence.

## Tasks, checks, and handoff reviews

| Task and next consumer | Tests or other evidence owned by this task | Reviewer checks the handoff |
| --- | --- | --- |
| S1 -> S2 | SDK-free workspace/build evidence; EXE and DLL names, target/profile paths, identities, and unchanged application startup | S2 can document the actual package boundary, artifact delivery, build commands, and applicable constraints. An empty DLL is identified as preparation, not working emulation |
| S2 -> S3 | Static contract/ADR review, source and API references, examples, links, and classification of unresolved native behavior | S3 has a consistent implementable JSONL/timing/ABI contract. Missing decisions are distinguished from bounded implementation investigations; each unresolved behavior has an owner and identifies the dependent work it blocks |
| S3 -> S4 | Parser/timeline checks; independent native client with real DLL loading, registration, dispatch, notification, and cleanup; applicable workspace checks | The actual artifact exposes the required setup and eight native exports, uses the declared ABI, and supplies the data/lifecycle responses required by the current receiver. S4 needs no Demo-specific decoder or second Publisher to make it work |
| S4 -> S5 and S8 | Real loader/CLI, HTTP/SSE lifecycle, complete application delivery, and S6's output checkpoints; visual verification assigned to S8 | Fault scenarios can reach the production receiver; healthy scenarios reach the delivered application. Errors preserve the intended source/HTTP separation. Changes to the application have not weakened or bypassed the S2/S3 contract; appearance remains unverified |
| S5 -> error scenarios in S8 | Selected deterministic API/packet faults, safe buffers, retry/terminal/cleanup outcomes, and HTTP availability | Every selected error scenario has a supported trigger and independently asserted outcome. Deferred rare cases are named. Healthy S8 work does not wait for exhaustive fault coverage |
| S6 -> verification in S4 and S8 | Static mapping and independently calculated expected checkpoints for input, receipt, normalization, and output | The integration checks can relate results to the authored input and existing transformations without reconstructing a scenario, adding a converter, or using the implementation under test as its own oracle |
| S8 -> Task #91 acceptance | Narrated flight and focused scenarios through the real application, independent input/output checkpoints, catalog consistency, and applicable integrated/browser evidence | The integrated result meets #91's own criteria across the full path. Completed children do not hide missing parent evidence; simulator compatibility and device acceptance retain their separate evidence limits |

## Required transitions

### S1 to S2

The reviewer compares S1's result with all S2 requirements and identifies the parts S2 depends on S1 to supply:

* Workspace/package ownership and actual `demo.dll` target naming agree with the proposed architecture. Build and output-location evidence is concrete enough for S2's documentation.
* SDK-free Windows x64 MSVC delivery, MIT licensing, external SDK binaries, and existing application behavior remain intact.
* Deployment paths and loader-test paths are distinguished. S2 must not document `cargo run` as building the DLL unless that behavior is actually established.
* Later scenario, ABI, and CLI work remains explicitly planned. S1 has not introduced stubs or extra framework decisions that S2 would have to endorse or undo.

### S2 to S3

The reviewer compares the documented contract with all inputs needed by S3:

* JSONL header/records, units, grouping, sparse signals, missing values, holds/ramps/gaps, and request-driven unchanged samples are mutually consistent.
* Clock origin, continuously increasing time, looping cursor, and explicit lifecycle changes contain no reset or automatic state-restoration requirement.
* Used exports, setup timing/errors, native layouts, IDs, registration, event notification, returned-memory ownership, and shutdown provide a consistent boundary for implementation.
* Empty-queue results, pause delivery, overflow, and uncertain SimVar spellings have explicit evidence limits and bounded verification work. A guess is not labeled SDK behavior; a consequential unresolved choice does not receive an unconditional readiness verdict.
* The documented native-only scope, case-insensitive DLL selection, HTTP-first setup, absence of a converter, and staged fault coverage agree with the parent and later integration requirements.

### S3 to S4

The reviewer compares the produced DLL and its evidence with the real consumer that S4 will integrate:

* The delivered DLL can be loaded at its documented path and exposes the exact native/setup signatures. Setup validates before Open, reports actionable failure, and does not start emission early.
* Registrations and requests use caller-supplied IDs, order, units, and cadence. Normal and diagnostic attitude definitions are compatible with the receiver; the catalog is not copied from the receiver merely to make the checks pass.
* Notification, queued packet retrieval, readable immutable buffers, empty-queue behavior, and Close satisfy the agreed ownership contract. Identify any remaining official-runtime comparison gap separately.
* Fresh equal holds, gaps, sparse values, lifecycle queries/events, and repeated passes supply the input semantics required by the application without fabricated values or hidden resets.
* Independent DLL-client results are present, with known limitations. Real CLI, HTTP-first startup, SSE, and browser integration remain S4 checks; their absence at S3 is not misreported as a passed end-to-end test.

## Review result and transition record

Use this compact record in the Subtask's review/result comment; do not copy entire contracts into it:

| Field | Required content |
| --- | --- |
| Reviewed state | Exact base/head or preserved patch identity; relevant artifact and scenario identities |
| Current outcome | Criteria supported by evidence, failures, and checks not performed |
| Successor requirements checked | References to the next Subtask's required inputs and the matching delivered artifacts/contracts |
| Transition verdict | Ready, blocked, or not verified; scope the verdict when only part of the successor is ready |
| Gap and owner | Specific missing input, contradiction, or evidence; who resolves it and which dependent work must wait |
| Remaining integration evidence | Checks intentionally owned by the successor, without claiming they already passed |

A missing required prerequisite blocks the affected downstream work. A planned successor implementation is not itself a predecessor defect. Known evidence gaps must not disappear behind a generic approval or be treated as proof of compatibility. Review must also flag additions beyond the accepted scope.

## GitHub publication

Link this plan from #91 and its context comment using a commit-pinned URL. Preserve the targeted reviewer sections in the first context comments of S1, S2, and S3; every Subtask links its applicable transition row. Put review results and actual dependency status in GitHub during execution, not in a second mutable roadmap.
During publication, replace local S-identifiers in execution links with actual issue references while preserving this mapping. Keep this file on the planning branch; it must not be merged into main under the `.plans/` rule.
