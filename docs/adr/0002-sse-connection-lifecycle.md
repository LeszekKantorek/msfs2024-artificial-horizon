# ADR 0002: Pull-driven SSE with socket lifecycle deadlines

- Date: 2026-10-08
- Status: Accepted

## Context

Issue #3 needs independent browser subscriptions without replay or acquisition
backpressure. A latest-value watch channel already owns telemetry freshness.
An SSE body is only polled when HTTP wants more data; a timeout inside that body
cannot reliably close a socket whose writes are stalled. Long-lived responses
must also terminate before graceful server shutdown can complete.

## Decision

Keep Axum's server and SSE framing. Pull snapshots directly from each subscriber,
acknowledging the initial publication and serializing age immediately. Do not add
per-client acquisition tasks, queues, or cached wire snapshots. Keep provider
states separate from HTTP liveness, and end streams when their source channel closes.

Wrap accepted sockets through Axum's Listener interface. A pending write/flush
has a 30-second no-progress deadline; successful progress clears it. Idle time
without a pending write is unrestricted. A shared monotonic shutdown timestamp
allows five seconds for connections to finish, then pending reads and writes
return an I/O error. Deadlines operate below the SSE body and retain Axum's
accept-error handling. Producer ownership and joining remain in Server::run.

The browser owns one EventSource and retries failed connections after two seconds,
including initial failures. Explicitly close the failed object before scheduling
its replacement, avoiding native retries competing with application retries.
Send retry: 2000 for other EventSource consumers. Suspend subscriptions while the
page is hidden and require a fresh snapshot after resumption. This is connection
recovery, not telemetry polling; stale source data does not trigger reconnect.

## Alternatives and consequences

- Per-client queues would preserve history but increase memory and stale delivery;
  current-state delivery deliberately skips intermediate publications.
- Body-only timeouts are simpler but cannot enforce socket shutdown under write
  backpressure; the adapter adds focused I/O code with deterministic deadline tests.
- Replacing Axum's server with a custom Hyper task supervisor gives more control
  but duplicates connection ownership unnecessarily for this requirement.
- The write timeout measures progress into the socket, not phone receipt; finite
  HTTP/TCP buffers can still add delay and require real-device measurement in #7.
- Fatal producer-task failure shuts HTTP down and returns an error; recoverable
  source failures are explicit telemetry states and never trigger demo fallback.
- Real phone lifecycle and long-duration memory evidence remain part of #7.
