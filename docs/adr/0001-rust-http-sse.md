# ADR 0001: One Rust server with a lightweight SSE client

- Date: 2026-10-05
- Status: Accepted as the initial implementation baseline

## Context

A small simulator companion needs to deliver current aircraft attitude to phone
browsers. Rust and a lightweight web client are requirements. The client needs
updates but no simulator command channel. There is no persistent domain data.

## Decision

Use one Rust binary crate with Tokio, Axum, and Serde; serve embedded static assets
and a same-origin SSE endpoint. Use native EventSource, HTML/CSS, vanilla JavaScript,
and SVG in the browser. Publish normalized latest-value snapshots from either an
explicit demo provider or a Windows SimConnect provider.

Axum has [SSE response and keep-alive support](https://docs.rs/axum/latest/axum/response/sse/).
Native EventSource supports reconnecting streams; the
[MDN SSE guide](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
documents framing and heartbeat comments. Resolve dependency versions at bootstrap,
record the tested toolchain, and commit Cargo.lock; this ADR does not pin untested versions.

## Alternatives and trade-offs

| Option | Trade-off | Decision |
| --- | --- | --- |
| SSE | Simple native receive API; text frames and long-lived HTTP connection | Fits one-way telemetry |
| WebSocket | Bidirectional control and binary frames; more connection protocol code | Unnecessary for MVP |
| Polling | Easy request model; repeated requests and sampling delay | Reject for continuous attitude |
| Rust/WASM frontend | Shared language; extra build/runtime integration | Keep the requested lightweight client |
| Separate telemetry service | Independent deployment; additional process/protocol | Unnecessary at this scale |

## Consequences

- Demo development and HTTP tests do not require MSFS or its SDK.
- Freshness, reconnects, mobile suspension, and slow clients need explicit handling.
- Browsers may limit concurrent HTTP/1.x SSE connections; use one per page and test
  the intended small client count.
- JSON bandwidth at the proposed rate must be measured on actual phones.
- One process is simple to install, but simulator integration must not block HTTP.
- FFI selection remains open until the Windows spike validates compatibility.

## Validation of the choice

The design matches a small project, separates the unverified SDK integration,
supports all stated requirements, and admits isolated model and HTTP tests.
No team experience has been assumed. Review this decision if remote access,
bidirectional commands, persistent history, or much larger client counts become requirements.
