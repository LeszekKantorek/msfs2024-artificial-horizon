# S5: Exercise API failures and malformed packets through Demo DLL

## Issue body

Parent: Task #91 under Epic #9. Prerequisite: S4.

Execution: one GPT-6.1 Terra developer at medium. Use Sol Medium consultation for consequential gaps.
Read Task #91, Epic #9, this body, its first context comment, and prerequisite results.
Refresh issue artifacts and repository state before execution.
Follow the repository execution/reporting procedure; report results and gaps in the main conversation.
This is a local draft. It grants no implementation or publication authorization.

### Outcome

Authored JSONL fault scenarios exercise the actual receiver's API errors, native validation, lifecycle recovery, and cleanup safely.

### Scope

* Add deterministic API-result rules, asynchronous exceptions, raw malformed dispatch fixtures, bad values, and connection-loss scenarios.
* Include every used export and source-health/HTTP behavior.
* Map the additional existing error paths to field loss, retry, terminal producer failure, diagnostics failure, or application failure.
* Supply separate JSONL files for independent terminal faults; provide launch/harness conditions for failures outside the loaded DLL.
* Correct only confirmed defects exposed within these native contracts; preserve unrelated behavior.
* Exclude inaccessible-memory pointers, permanently blocked calls, general scripting, load/performance testing, and simulator-control features.

### Context

* S2 fault contract; S3 emitter/ownership; S4 production-path results.
* `native.rs::decode_received`, `decode_with_attitude_count`, `Connection::next`, and close ownership.
* `runtime.rs::session`, reconnect, optional sent-packet correlation, and supervision.
* `instruments.rs`, `lifecycle.rs`, official units reference, and existing native tests.

### Implementation guidance

1. Match faults by export, one-based matching call occurrence, and whitelisted argument selectors.
2. Preserve fault occurrence state across retries so recovery scenarios actually recover.
3. Correlate asynchronous exceptions with real sent-packet IDs captured from matched calls.
4. Return owned buffers for malformed shape/size/count/ID cases; keep advertised readable length within allocation.
5. Separate empty E_FAIL from native dispatch failure and decoder/ABI failure.
6. Add primary-invalid, optional-invalid, missing, repeated equal, paused-input, and post-resume cases.
7. Run failed-Close cases in a child process and preserve conservative retain-mapping behavior.
8. Retain default healthy behavior and source-failure isolation.
9. Cover initial and later system-state query failure, sent-packet correlation failure, startup retry, and the packet-silence watchdog.
10. Implement bounded transport silence that suppresses query replies as well as data; a signal gap alone is insufficient.
11. Verify loader/export and diagnostics I/O outcomes with the appropriate launch/filesystem harness.
12. Reuse existing seams for host errors/panics. Record missing checks rather than inventing SimConnect opcodes or a general host-fault framework.
13. Keep each fatal case independent so the first producer failure does not hide later cases.

### Acceptance criteria

* [ ] Failure of each used export has a deterministic trigger and independently asserted outcome.
* [ ] Optional definition/request/exception failure does not remove attitude or independent optional fields.
* [ ] Correlated and uncorrelated exceptions follow their distinct existing paths.
* [ ] Empty dispatch E_FAIL does not fabricate disconnect; other native dispatch errors follow retry.
* [ ] Truncated headers/payloads and invalid sizes/counts are rejected without unsafe memory reads.
* [ ] Unknown receive IDs and irrelevant routing are handled according to the current decoder contract.
* [ ] Invalid/nonfinite attitude suppresses live; optional invalid values remain independently unavailable.
* [ ] Bool=-1 is exercised against the independent native true contract; any correction preserves target availability semantics.
* [ ] Normal/active pause and inactive-flight input cannot be retained to authorize resume.
* [ ] Stop remains responsive during gaps, retry, dense queued traffic, and fault handling.
* [ ] Failed Close neither unloads still-retained native state nor leaves Demo's emitter running.
* [ ] HTTP remains available after permanent producer failure, with unavailable telemetry and no Demo fallback.
* [ ] Required initial state-query failure terminates the producer; a later periodic native query failure reconnects after successful cleanup. HTTP remains available in both cases.
* [ ] Sent-packet-ID failure and a later unrecognized exception exercise their distinct nonfatal/fatal consequences.
* [ ] Total transport silence triggers watchdog retry; a data-only gap with healthy state replies does not falsely establish that case.
* [ ] Loader/export and early/later diagnostic failures have the correct launch/filesystem harness and separate acquisition/diagnostic expectations.
* [ ] Each dedicated file reports its own trigger, expected source outcome, HTTP liveness, and whole-application outcome.
* [ ] Available host/supervision checks are retained; missing fault seams are reported without unrelated architectural changes.

### Validation

Later execution: isolated DLL-client tests, memory-safe malformed buffers, real-loader child-process tests, focused native regressions, complete workspace checks/build, and independent diff review.
Repeat affected compiled-browser checks when corrected behavior changes displayed states.
Record fault scenario identity, trigger, native outcome, normalized state, recovery, cleanup, and missing checks.
No fault injection or project execution was performed during planning.

## First comment: execution context

Broad API and malformed-packet simulation was explicitly requested in the discussion.
It is not an accidental expansion from the healthy Demo.

Keep authored fault data separate from malformed scenario syntax.
Scenario syntax errors fail setup; a valid fault scenario intentionally produces bad native input later.

Representative raw faults include header size smaller than the header, declared size beyond readable bytes, missing payload values, mismatched definition/counts, unknown receive IDs, and invalid numeric bits.
The DLL must still honor the memory contract for its output pointer.
Do not simulate arbitrary unmapped memory or buffer use-after-free.

The current consumer treats decoder errors as fatal and native dispatch HRESULT errors as reconnectable.
Preserve those distinctions unless concrete evidence supports a focused correction.

Independent SDK units documentation permits nonzero Bool true, including -1.
The current consumer requires AP available == 1. A copied oracle would conceal that discrepancy.
Keep this case visible and scoped to the used native field contract.

The later discussion explicitly requests additional existing SimConnect error coverage and several scenario files.
Use independent files for required-subscription failure, malformed dispatch, uncorrelated exception, and failed Close.
These terminate the producer under current code; they must not interrupt the normal flight tour.

Do not equate source termination with whole-process termination.
The application retains HTTP/SSE and publishes unavailable data after a permanent native producer failure.
Configuration/bind/HTTP-service failures have different application outcomes and belong in the application harness.

The watchdog measures all native packets, including system-state replies.
Add transport_silence on the established scenario clock, with a finite duration surviving reconnects.
Keep API call results separate from dispatch silence; do not synthesize QUIT for this case.

Loader failure cannot be requested by a script inside a DLL that never loaded.
Missing/incomplete DLLs, journal destination failures, host event failures, and worker panic require the appropriate existing launch/test seam.
Do not add fake SDK calls, unsafe memory faults, or a general dependency-injection framework to make every condition a JSONL instruction.
