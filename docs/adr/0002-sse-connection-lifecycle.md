# ADR 0002: Pull-driven SSE with socket lifecycle deadlines

Decision status: Accepted. [ADR 0007](0007-application-composition.md) defines application composition and producer ownership.

## Context

* Clients need independent subscriptions without replay or acquisition backpressure.
* A latest-value watch channel already owns freshness.
* HTTP polls the SSE body only when it needs more data.
* A timeout inside that body cannot reliably close a socket with stalled writes.
* Long-lived responses must end before graceful shutdown can complete.

## Decision

### Delivery

* Keep Axum's server and SSE framing.
* Pull snapshots directly from each subscriber.
* Acknowledge the initial publication and serialize age immediately.
* Add no per-client acquisition tasks, queues, or cached wire snapshots.
* Keep provider state separate from HTTP liveness.
* End streams when their source channel closes.
* Application composition owns producer startup and joining. The server owns HTTP delivery and connection closure.

### Socket deadlines

Wrap accepted sockets through Axum's `Listener` interface.

| Condition | Behavior |
| --- | --- |
| Pending write/flush without progress | Fail after 30 seconds |
| Successful progress | Clear the write deadline |
| Idle, no pending write | No time limit |
| Shutdown | Use one shared monotonic timestamp. Allow five seconds to finish |
| Shutdown deadline reached | Pending reads/writes return an I/O error |

Deadlines operate below the SSE body and preserve Axum's accept-error handling.

### Browser lifecycle

1. Keep one EventSource per page.
2. Close a failed EventSource before scheduling its replacement.
3. Retry after two seconds, including initial failures.
4. Suspend subscriptions while the page is hidden.
5. Require a fresh snapshot after resumption.

* Send `retry: 2000` for other EventSource consumers.
* Closing the failed object prevents competing native and application retries.
* Stale source data does not trigger reconnect.

> This mechanism recovers connections. It does not poll telemetry.

## Alternatives and consequences

| Alternative | Trade-off / decision |
| --- | --- |
| Per-client queues | Preserve history but increase memory and stale delivery. Select current state and skip intermediate publications |
| Body-only timeouts | Simpler, but cannot enforce shutdown under socket backpressure. Use an adapter with deterministic I/O deadline tests |
| Custom Hyper task supervisor | More control, but duplicates connection ownership unnecessarily |

* The write deadline measures socket progress, not phone receipt.
* Finite HTTP/TCP buffers can add delay. Measure it on real devices using the [measurement procedure](../testing/evidence.md#measurements-and-evidence).
* Producer failure alone does not stop HTTP. Application composition reports the failure and retains unavailable telemetry.
* Recoverable source failures produce explicit telemetry states without demo fallback.
* Record real-phone lifecycle and long-duration memory evidence separately from automated socket checks.
