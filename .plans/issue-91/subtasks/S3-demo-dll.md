# S3: Implement JSONL-driven healthy SimConnect emulation

## Issue body

Parent: Task #91 under Epic #9. Prerequisite: S2.

Execution: one GPT-6.1 Terra developer at medium. Use Sol Medium consultation for consequential gaps.
Read Task #91, Epic #9, this body, its first context comment, and prerequisite results.
Refresh issue artifacts and repository state before execution.
Follow the repository execution/reporting procedure; report results and gaps in the main conversation.
This is a local draft. It grants no implementation or publication authorization.

Apply the [post-review receive-path clarification](../architecture.md#receive-path-clarification-after-the-astra-review).
Exercise notification-driven receipt without the application's polling timeout, immutable returned memory, and queued packet order.
Resolve the documented empty-queue distinction before treating one return shape as the only compatible behavior.

### Outcome

The project-owned Demo DLL reads compact JSONL instructions and returns healthy native SimConnect responses through the actual used ABI.

### Scope

* Implement the Demo package's parser, track evaluator, monotonic scheduler, default scenario, setup export, and eight SimConnect exports.
* Cover all currently consumed native fields and both two-field and six-field attitude definitions.
* Include normal/active pause, flight active/inactive, loaded-flight changes, gaps, continuous-clock looping, and reconnect state.
* Keep the application CLI integration in S4 and deliberate fault injection in S5.
* Retain the mapped default/boundary scenario. S8 delivers the additional narrated flight after the needed fault/lifecycle mechanisms exist.
* Exclude official SDK dependencies, a second telemetry publisher, arbitrary expressions, hot reload, and browser controls.
* Implement authored instructions only. Recorded-session import and observations mode are outside #91.

### Context

* S2's durable ADR and source-contract sections; S1's build/artifact result.
* `native.rs` exports, receive layouts, field definitions, and `Connection::next`.
* `runtime.rs::configure`, `session`, `query_states`; `event.rs`; `lifecycle.rs`.
* Current default phases in `src/providers/demo.rs` and independent existing fixtures.

### Implementation guidance

1. Embed repository-root `demo_scenario.jsonl` in the DLL and use the same parser for explicit files.
2. Implement whole-file typed validation with bounded line/file/record sizes and checked time arithmetic.
3. Represent each signal's sequential hold/ramp/gap track; evaluate concurrent tracks on one monotonic timeline.
4. Match actual AddToDataDefinition names/units/types against the independent catalog.
5. Store caller-supplied definitions, field order, requests, event subscriptions, and state-query IDs.
   Honor request cadence and flags; current SIM_FRAME/flags-0 requests emit equal values without deduplication.
6. Generate packed receive buffers from those registrations, including logging-enabled diagnostic probes.
7. Own notification emission and buffers inside the DLL; join emission before successful Close.
8. Maintain current scenario state and one-shot counters across reconnects.
9. Preserve mapped default phases and checkpoints. Document intentional native availability/estimator differences.

### Acceptance criteria

* [ ] Review covers the assigned outcome and the prerequisites supplied to the next consumer under the [tasks/tests/review plan](../tasks-tests-review.md). Record evidence, gaps, and a scoped readiness verdict.

* [ ] A 100-kt hold lasts five logical seconds and sends fresh requested observations; its following ramp reaches the expected midpoint after 2.5 seconds.
* [ ] A healthy hold longer than the server expiry threshold produces successive equal native observations at the requested cadence, rather than one initial packet.
* [ ] Generation follows synthetic frames and registered requests independently of dispatch calls. Flags 0 never applies a value-change filter.
* [ ] An explicit endpoint hold emits 150 at the ramp boundary; the test distinguishes a half-open ramp from guaranteed endpoint delivery.
* [ ] A gap sends no observations and allows existing freshness to expire.
* [ ] Independent tracks advance together and repeat at their declared periods.
* [ ] After the last step, playback returns to the first step at a later absolute time without resetting or pausing the clock.
* [ ] A pass ending paused or inactive remains in that state until an explicit scenario step changes it. Looping itself emits no lifecycle transition.
* [ ] Every catalog field is exercised across independent cases; wrong names, units, or unsupported request options fail rather than producing default values.
* [ ] Supported parameters register successfully even when absent from a scenario. Missing values generate no samples and no implicit zeros.
* [ ] IAS-only, event-only, and no-signal scenarios are accepted without filler tracks. Other requested fields remain silent.
* [ ] Altering client IDs or definition order changes returned routing/order correctly.
* [ ] Diagnostic logging receives six correctly ordered attitude fields; normal mode receives two.
* [ ] Pause, active pause, inactive flight, flight change, and resume/query behavior match the documented states.
* [ ] Verify normal and active pause delivery against SimConnect during implementation. Treat repeating frozen values as the current assumption, not established SDK evidence. Do not suppress packets merely because the application hides live data while paused.
* [ ] Invalid scenario setup returns E_FAIL and an actionable bounded explanation before Open. No emitter starts; S4 verifies HTTP remains alive.
* [ ] Reconnect obtains current lifecycle and fresh samples; it does not restart the same disconnect endlessly.
* [ ] Close releases all successful native ownership. No Rust panic crosses the FFI boundary.
* [ ] Invalid explicit input reports its path/line/field and never selects the embedded default.

### Validation

Later execution: controlled-clock parser/evaluator tests, an independent DLL client using real Windows loading, authoritative layout checks, and workspace application checks.
Include a negative control that changes only a client spelling or unit.
Expected values must not call the consumer's normalization functions.
Record EXE/DLL and scenario identities plus missing evidence.
No tests/builds/loads were run for this draft.

## First comment: execution context

### Reviewer: readiness for S4

Read [S4's complete requirements and context](S4-application.md) alongside the actual DLL, S3's evidence, S2's contract, and the current application receiver. Apply the [S3-to-S4 transition review](../tasks-tests-review.md#s3-to-s4).

* Check every S4 requirement for a DLL prerequisite: loadable artifact, exact exports/setup ABI, native layouts/routing, notification/dispatch ownership, lifecycle responses, error reporting, and cleanup.
* Verify that the shared receiver can consume the delivered behavior without a Demo-specific decoder, private telemetry route, or compensating fabricated samples.
* Check evidence for continuous-clock looping, sparse inputs, equal-value holds, and gaps. Report unverified SDK parity separately from tested DLL behavior.
* Give an explicit S4 readiness verdict with any missing prerequisite and owner. CLI/HTTP/SSE/browser integration remains S4 work; independent DLL-client checks do not establish it early.

### Native playback context

A five-second hold means repeated fresh samples. It does not mean retaining a single received sample for five seconds.
For the current SIM_FRAME request with flags 0, emit on each due frame even when the value is unchanged.
A gap emits no samples for its signal; other requests and events continue under their own scenario instructions.
The server timeout measures time since receipt, not time since the numeric value last changed.
Ramps calculate values from elapsed monotonic time rather than sleeping once per expected packet.
Queued packets retain their original contents. Scheduler lateness and queue overflow require an explicit fidelity policy.

Definitions come from the client. Normal scenario data must not hard-code request 1 or optional requests 100–116.
The actual client currently uses those IDs, but emulator behavior must also work with changed IDs.

Separate native catalog support from scenario values. Do not require a track for every registered field.
Preserve native packet layouts when a definition groups fields; missing components suppress that packet, not independent requests or events.
Do not fabricate data to satisfy application readiness or presentation expectations.

Start the monotonic scenario clock on the first successful Open, independently of dispatch.
Registration overlaps scenario time. Configure alone does not start the clock.
Retain the origin across reconnects and keep one-shot fault counters separate from initial registration.

Demo DLL is a playback mechanism only. After the last step, move the playback cursor to the first step.
Keep absolute time, connection state, pending packets, and call-occurrence counters intact at that boundary.
Apply header initial state once. Subsequent pause and flight changes come only from explicit scenario records.
There is no automatic lifecycle restoration or finite-end waiting mode.

The DLL must return current Sim/FlightLoaded query responses and initial known Pause_EX1 state.
Normal pause uses flag 1 and active pause uses flag 4.
After any lifecycle transition, the receiver requires a newly eligible attitude.
This rule belongs to the current consumer; the DLL supplies the events and data needed to exercise it.

Use source-native units: radians for primary attitude, raw ball position, inHg for native KOHLSMAN, and the documented units for other signals.
Numeric BARO is produced by the existing consumer. Add no legacy text fallback.

The root application still uses the old provider until S4.
The independent DLL client establishes ABI behavior at this stage; it is not final application integration evidence.
The multi-scenario flight narrative does not require adding Vx, stall-warning, ground contact, or a physics model to this package.
