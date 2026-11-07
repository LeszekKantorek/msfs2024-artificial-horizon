# Existing error paths and proposed scenario coverage

Status: Static planning inventory at revision `9356e47e481d422c3b771205cd7f3e82f07789b0`.
The outcomes are predictions from current code, not runtime observations.
This is a coverage inventory, not a requirement to implement every failure in the first delivery. S5 records selected implemented cases and explicitly deferred cases; healthy playback and the flight narrative do not wait for exhaustive rare-error coverage.
Canonical native error semantics belong in `docs/sources/simconnect.md`; scenario purposes belong in the separate catalog.

## Native-source cases

| Trigger | Current source reaction | Application reaction | Planned input mechanism |
| --- | --- | --- | --- |
| DLL load, architecture, dependency, or missing-export failure | Permanent producer failure | Supervisor publishes disconnected; HTTP/SSE remains | Launch fixture selecting a missing/incompatible/incomplete DLL |
| Open error or null success handle | Interruptible bounded-backoff retry | HTTP remains available | `api_fault` HRESULT for ordinary retry; special null-output ABI test uses a dedicated DLL fixture |
| Primary attitude-definition failure | Permanent producer failure after cleanup | Disconnected; HTTP remains | Matched AddToDataDefinition failure |
| Required subscription failure | Permanent producer failure after cleanup | Disconnected; HTTP remains | Dedicated primary-subscribe scenario |
| Initial system-state query failure | Permanent producer failure after cleanup | Disconnected; HTTP remains | RequestSystemState call occurrence |
| Periodic system-state query native failure | Reconnect after successful cleanup; other non-native query errors still propagate | Disconnected during retry; HTTP remains | Later RequestSystemState occurrence |
| Primary periodic-request failure | Permanent producer failure after cleanup | Disconnected; HTTP remains | Matched primary RequestDataOnSimObject failure |
| Optional definition/request failure | Affected field absent; acquisition continues | Other fields and HTTP remain | Optional-definition/request scenarios |
| GetLastSentPacketID failure | Acquisition continues; correlation can be absent | HTTP remains; a later unmatched exception can be fatal | Separate call failure and call-failure-plus-exception cases |
| GetNextDispatch E_FAIL | Current receiver treats it as no packet; SDK documents generic failure without exhaustive conditions | Freshness still expires | Verify the empty-queue HRESULT/pointer/length tuple against the target runtime before claiming parity |
| Another failed GetNextDispatch HRESULT | Reconnect after successful cleanup | Disconnected during retry; HTTP remains | Dispatch-failure scenario |
| Truncated/invalid native layout | Permanent producer failure, not ordinary reconnect | Disconnected; HTTP remains | Owned raw-packet/null-dispatch fixture |
| Unknown receive ID or irrelevant routing | Decoder-specific ignored/rejected observation, not automatically fatal | Prior data expires unless replaced | Independent routing/unknown-ID fixture |
| Correlated optional asynchronous exception | Remove affected optional field | Other source data and HTTP remain | Exception linked to a real optional sent-packet ID |
| Uncorrelated asynchronous exception | Permanent producer failure | Disconnected; HTTP remains | Dedicated raw exception fixture with unmatched send ID |
| QUIT | Reconnect after cleanup/backoff | HTTP remains | Finite disconnect interval |
| No packets for watchdog duration | Reconnect | Primary data becomes stale first; HTTP remains | Explicit transport silence, including state-query responses |
| No fresh primary attitude | Stale when primary age reaches its limit | HTTP remains | Primary track gap; suppress the whole requested primary packet |
| Invalid primary attitude | Invalid state; recoverable with a new eligible sample | HTTP remains | Nonfinite/out-of-range input |
| Invalid/missing/expired optional value | Independent unavailability; dependent estimator/gates may reset | Other usable readings remain | Optional gap/value and target-gate scenarios |
| Pause, inactive Sim, flight change, or missing flight | Paused/waiting; cached primary eligibility cleared | HTTP remains | Normal lifecycle records |
| Close failure | Permanent producer failure; retain DLL/event mapping conservatively | HTTP remains; process exit releases retained state | Dedicated close-failure scenario in a child process |

The initial Open failure used by the flight tour is recoverable.
Do not mix the dedicated fatal cases into that tour.
A single scenario containing several terminal faults can reach only its first terminal failure.

## Host and infrastructure cases

This table identifies verification boundaries, not an instruction to build a new host-fault framework.
New loader/journal/host harness checks are proposed verification scope rather than additional simulator behavior.
Reuse existing checks and seams; report a missing case before expanding host infrastructure.

| Trigger | Current reaction | Correct verification boundary |
| --- | --- | --- |
| Invalid source/HTTP configuration | Startup error before acquisition; invalid configuration prevents binding | CLI/library tests |
| Busy HTTP port | Startup error; no producer starts | Application startup test |
| Journal open/protected destination/sharing/initial write failure | Producer fails before DLL load; HTTP remains | Launch/filesystem fixture |
| Later diagnostic write/flush failure | Disable diagnostics; continue acquisition and cleanup | Journal fault harness; native scenario supplies ordinary data |
| Stop/notification creation or Windows wait failure | Producer error; cleanup attempted | Host API seam/focused existing tests where available |
| Native worker Rust panic | WorkerPanic report; HTTP remains disconnected | Controlled Rust worker harness |
| Acquisition future panic | Supervisor catches it; HTTP remains disconnected | Existing supervision seam |
| HTTP service or caller shutdown error | Coordinate component shutdown and return application error | Application harness, not Demo instruction |
| Per-client socket timeout | Affected HTTP connection ends under server policy | HTTP tests; not a native fault |
| Permanently blocked native call | No guaranteed whole-process shutdown completion | Explicit existing limitation; do not add a hanging narrative |
| Access violation/abort/panic across FFI | Not safely covered by ordinary Rust supervision | Exclude deliberate unsafe-memory/process-crash injection |

A test harness can arrange these conditions without adding fake SimConnect exports or scenario opcodes for host failures.
A native error generally does not terminate the whole application under current source-failure isolation.
Assertions must check producer and HTTP outcomes separately.

## Additional mechanism required for watchdog coverage

A data-track gap does not imply total connection silence.
The receiver periodically requests system state; replies keep its packet watchdog alive.
The existing scenario controls therefore need one bounded transport-silence record:

```jsonl
{"type":"transport_silence","at_us":2000000,"duration_ms":17000}
```

During that interval, Open, registration, and query calls keep their configured synchronous result.
Suppress dispatch packets, including query replies and ordinary notifications.
GetNextDispatch returns the normal empty-queue result.
Do not emit QUIT merely because the interval started.

The scenario clock continues and the interval survives reconnects.
At interval end, supply current subscribed/query state and new eligible data.
This explicitly requested error coverage needs a new producer mechanism; no runnable implementation is claimed.

## Source references

| Area | Inspected code |
| --- | --- |
| Loading, dispatch buffers, Close | `src/providers/simconnect/native.rs:216–322, 469–505, 604–804` |
| Connections, required/optional configuration, packet correlation | `src/providers/simconnect/runtime.rs:293–419, 483–651` |
| Dispatch, exceptions, watchdog, waits | `src/providers/simconnect/runtime.rs:664–864` |
| Diagnostic failures | `src/providers/simconnect/journal.rs:39–175`; `runtime.rs:189–217` |
| Lifecycle/independent optional availability | `lifecycle.rs:49–132`; `instruments.rs:32–152` |
| HTTP-first source-failure isolation and application shutdown | `src/application.rs:64–114, 156–170` |

Refresh source locations before execution. No additional source fields or simulator-control functions were added by this inventory.
