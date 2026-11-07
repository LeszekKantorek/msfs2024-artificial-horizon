# S4: Select either DLL through the common native receiver

## Issue body

Parent: Task #91 under Epic #9. Prerequisite: S3.

Execution: one GPT-6.1 Terra developer at medium. Use Sol Medium consultation for consequential gaps.
Read Task #91, Epic #9, this body, its first context comment, and prerequisite results.
Refresh issue artifacts and repository state before execution.
Follow the repository execution/reporting procedure; report results and gaps in the main conversation.
This is a local draft. It grants no implementation or publication authorization.

### Outcome

Default Demo, explicit Demo scenarios, and the official SimConnect DLL use one native receiver through the requested DLL-path CLI.

### Scope

* Change CLI, typed source configuration, application composition, native loading/setup, diagnostics identity, dependent tests, and current usage documentation.
* Remove the direct Demo acquisition branch from production.
* Preserve existing HTTP/SSE, telemetry normalization, independent presentation capability, and shutdown.
* Exclude provider registries, compatibility aliases, UI controls, estimator redesign, and converter implementation.

### Context

* S2 contracts and S3 DLL/artifact result.
* `src/bin/main.rs`, `src/application.rs`, `src/providers/mod.rs`, and `providers/simconnect/{runtime,native,journal}.rs`.
* Old direct-Demo callers in Rust tests and both browser scripts.
* README, source guides, architecture composition/lifecycle, validation matrix, and ADR 0007.

### Implementation guidance

1. Parse `--source <dll>`; accept the two supported basenames and absolute paths retaining them.
2. Resolve bare names beside EXE; preserve restricted dependency loading and no fallback.
3. Fix typed Demo/SimConnect identity before channel creation; call the same receiver for both.
4. Accept `--scenario` only for Demo, pass its absolute path through the private setup export, and keep arguments out of provider internals.
5. Keep HTTP-first startup; make Demo parser/setup failure a reported source failure without closing HTTP.
6. Remove production direct-Demo scheduling. Replace helper-dependent tests with appropriate independent fixtures or native integration checks.
7. Preserve full-profile presentation tests without treating their fixture values as available native source data.
8. Update browser runner CLI assumptions and deployed startup examples.
   Document explicit selection of another scenario file without a browser selector, registry, or new source type.
9. Keep Demo diagnostics identified as synthetic; never claim the Demo binary is an official SDK runtime.

### Acceptance criteria

* [ ] Review covers the assigned outcome and the prerequisites supplied to the next consumer under the [tasks/tests/review plan](../tasks-tests-review.md). Record evidence, gaps, and a scoped readiness verdict.

* [ ] No source option selects adjacent demo.dll; explicit demo.dll and SimConnect.dll use the same native acquisition code.
* [ ] Missing/invalid explicit DLLs and scenarios never fall back to another file or source.
* [ ] Legacy source enums and --dll are rejected; help/version open no DLL/log.
* [ ] Source identity remains Demo for emulated data and SimConnect for the real selection.
* [ ] Default, explicit-file, and deployed-outside-repository Demo reach real HTTP/SSE and serve the embedded EFI. Screenshot and visual verification belongs to S8's integrated session.
* [ ] Changing external JSONL changes the next launch without rebuilding.
* [ ] Two clients share one progression; slow delivery does not control acquisition.
* [ ] With eligible lifecycle and fresh attitude, successive equal IAS observations keep IAS available beyond 1000 ms. An IAS gap expires IAS independently of continuing ALT observations.
* [ ] Normal/active pause and inactive flight suppress live; resume waits for new eligible data.
* [ ] This publication gate does not force Demo to stop native delivery. Paused packets cannot authorize later resume.
* [ ] DEMO.dll and other case variants select the same source identity. Export lookup remains exact.
* [ ] HTTP binds before scenario validation; a private setup E_FAIL stops the source with an actionable error and leaves HTTP/SSE available. It is not passed through the empty-dispatch path.
* [ ] Use S6's input/output mapping to distinguish raw receipt evidence from final log/SSE results.
* [ ] Bind failure prevents native startup; producer failure keeps existing/new HTTP clients available.
* [ ] Caller shutdown joins native work and HTTP according to existing cleanup limits.
* [ ] Accepted wire, independent optional availability, and estimator behavior retain their required checks. Preserve browser capabilities; final appearance verification is explicitly handed to S8.

### Validation

Later execution: CLI/library and real-loader tests, HTTP/SSE lifecycle coverage, and workspace-wide checks/release build.
Update the browser scripts for integration, but run screenshot-producing suites and inspect captures only in S8's consolidated visual session under the [visual verification schedule](../tasks-tests-review.md#visual-verification-schedule).
Hand off representative scene inputs and intentional source-field differences. Record visual acceptance as pending S8, without requiring an intermediate screenshot or manual appearance review.
Obtain independent review of the exact implementation diff.
No execution occurred while drafting.

## First comment: execution context

Reviewer: apply the [S4 transition row](../tasks-tests-review.md#tasks-checks-and-handoff-reviews) against S5 and S8's required inputs. Confirm the production receiver and observable source/HTTP behavior are available to fault and scenario checks, using S6's independent output mapping. Report missing downstream prerequisites separately from work assigned to those successors.

The requested source syntax replaces separate --source simconnect and --dll selection.
A bare supported DLL filename resolves beside the application EXE, independently of working directory.
An explicit DLL path must be absolute and preserve its supported basename.
No PATH search, SDK discovery, extension guessing, or legacy alias is selected.

The typed source kind is configuration metadata, not evidence that a filename authenticates its bytes.
No extra identity handshake is necessary for this two-source scope.

Demo alone uses the proposed ConfigureV1 setup export.
Do not add scenario fields to the official SimConnect calls or misuse its application-name argument.

The earlier planned pre-bind scenario validation is deliberately replaced:
the native DLL is loaded only after HTTP binds, then parses the scenario before simulated Open.
Failure remains actionable while HTTP serves unavailable state.
Help/version and syntactically invalid paths still avoid native startup.

Several tests currently call demo::sample/speed_profile/attitude directly.
Preserve their independent assertions by selecting the right test boundary.
Do not keep a hidden second production Demo solely to satisfy those helpers.

Keep the existing native normalization and current optional data gates.
The independent Bool=-1 discrepancy is not permission for unrelated cleanup; handle only the contract mismatch exposed by applicable evidence.

Scenarios may supply IAS alone, only lifecycle events, or no signal values. The application must not require Demo to fill other fields.
Unreceived instrument values default to null/unavailable, not zero. The earlier initial-zero preference is withdrawn.
Preserve existing SSE serialization: absent optional fields are omitted; attitude and sample age use null under their respective contracts.
Retain current unavailable presentation and lifecycle/freshness gates. Sparse scenario support does not by itself authorize independent instrument display without eligible attitude.
