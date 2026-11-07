# S3: Implement JSONL-driven healthy SimConnect emulation

## Issue body

Parent: Task #91 under Epic #9. Prerequisite: S2.

Execution: one GPT-6.1 Terra developer at medium. Use Sol Medium consultation for consequential gaps.
Read Task #91, Epic #9, this body, its first context comment, and prerequisite results.
Refresh issue artifacts and repository state before execution.
Follow the repository execution/reporting procedure; report results and gaps in the main conversation.
This is a local draft. It grants no implementation or publication authorization.

### Outcome

The project-owned Demo DLL reads compact JSONL instructions and returns healthy native SimConnect responses through the actual used ABI.

### Scope

* Implement the Demo package's parser, track evaluator, monotonic scheduler, default scenario, setup export, and eight SimConnect exports.
* Cover all currently consumed native fields and both two-field and six-field attitude definitions.
* Include normal/active pause, flight active/inactive, loaded-flight changes, gaps, finite completion, and reconnect state.
* Keep the application CLI integration in S4 and deliberate fault injection in S5.
* Retain the mapped default/boundary scenario. S8 delivers the additional narrated flight after the needed fault/lifecycle mechanisms exist.
* Exclude official SDK dependencies, a second telemetry publisher, arbitrary expressions, hot reload, and browser controls.
* Implement scripted mode only. Raw observations-mode support remains deferred until S6 resolves its contract.

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
6. Generate packed receive buffers from those registrations, including logging-enabled diagnostic probes.
7. Own notification emission and buffers inside the DLL; join emission before successful Close.
8. Maintain current scenario state and one-shot counters across reconnects.
9. Preserve mapped default phases and checkpoints. Document intentional native availability/estimator differences.

### Acceptance criteria

* [ ] A 100-kt hold lasts five logical seconds and sends fresh requested observations; its following ramp reaches the expected midpoint after 2.5 seconds.
* [ ] An explicit endpoint hold emits 150 at the ramp boundary; the test distinguishes a half-open ramp from guaranteed endpoint delivery.
* [ ] A gap sends no observations and allows existing freshness to expire.
* [ ] Independent tracks advance together and repeat at their declared periods.
* [ ] Every catalog field is exercised; wrong names, units, or unsupported request options fail rather than producing default values.
* [ ] Altering client IDs or definition order changes returned routing/order correctly.
* [ ] Diagnostic logging receives six correctly ordered attitude fields; normal mode receives two.
* [ ] Pause, active pause, inactive flight, flight change, and resume/query behavior match the documented states.
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

A five-second hold means repeated fresh samples. It does not mean retaining a single received sample for five seconds.
Ramps calculate values from elapsed monotonic time rather than sleeping once per expected packet.
Obsolete scripted frames are skipped instead of accumulated into a fresh-looking backlog.

Definitions come from the client. Normal scenario data must not hard-code request 1 or optional requests 100–116.
The actual client currently uses those IDs, but emulator behavior must also work with changed IDs.

Start the monotonic scenario clock on the first GetNextDispatch after a successful periodic data request.
The current consumer completes its initial registrations before that call.
Configure/Open alone do not consume the beginning of the scenario.
Retain the origin across reconnects and keep one-shot fault counters separate from initial registration.

The DLL must return current Sim/FlightLoaded query responses and initial known Pause_EX1 state.
Normal pause uses flag 1 and active pause uses flag 4.
After any lifecycle transition, the receiver requires a newly eligible attitude.
This rule belongs to the current consumer; the DLL supplies the events and data needed to exercise it.

Use source-native units: radians for primary attitude, raw ball position, inHg for native KOHLSMAN, and the documented units for other signals.
Numeric BARO is produced by the existing consumer. Add no legacy text fallback.

The root application still uses the old provider until S4.
The independent DLL client establishes ABI behavior at this stage; it is not final application integration evidence.
The multi-scenario flight narrative does not require adding Vx, stall-warning, ground contact, or a physics model to this package.
