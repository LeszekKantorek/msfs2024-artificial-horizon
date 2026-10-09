# ADR 0001: One Rust server with a lightweight SSE client

| Date | Status |
| --- | --- |
| 2026-10-05 | Accepted as the initial implementation baseline |

## Context

* Phone browsers need current simulator attitude.
* Rust and a lightweight web client are requirements.
* The client needs updates without a simulator command channel.
* There is no persistent domain data.

## Decision

| Boundary | Choice |
| --- | --- |
| Application | One Rust library package with Tokio, Axum, and Serde |
| Executables | Thin targets under `src/bin/`, starting with `src/bin/main.rs` |
| CLI | clap parsing, typed configuration passed to the library |
| HTTP | Embedded assets and same-origin SSE |
| Browser | Native EventSource, HTML/CSS, vanilla JavaScript, SVG |
| Telemetry | Normalized latest-value snapshots |
| Source | Explicit demo provider or Windows SimConnect provider |
| Platform | Windows x64 MSVC for library, binaries, demo, and CI |
| SDK | Optional dependencies. Default Windows checks require no MSFS |

Axum provides [SSE responses and keep-alives](https://docs.rs/axum/latest/axum/response/sse/).
EventSource supports reconnecting streams. The [MDN SSE guide](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events) describes framing and heartbeat comments.

* Resolve dependency versions at bootstrap.
* Record the tested toolchain.
* Commit `Cargo.lock`.

> This ADR does not pin untested versions.

## Alternatives and trade-offs

| Option | Trade-off | Decision |
| --- | --- | --- |
| SSE | Native receive API, text frames, long-lived HTTP | Select for one-way telemetry |
| WebSocket | Bidirectional control and binary frames require more protocol code | Unnecessary for MVP |
| Polling | Simple requests add repetition and sampling delay | Reject for continuous attitude |
| Rust/WASM frontend | Shared language adds build/runtime integration | Keep the lightweight client |
| Separate telemetry service | Independent deployment adds a process/protocol | Unnecessary at this scale |

## Consequences

* Other binaries can reuse the library without process-argument coupling.
* Demo development and HTTP tests require neither MSFS nor its SDK.
* Handle freshness, reconnects, mobile suspension, and slow clients explicitly.
* Use one SSE connection per page. Browsers may limit concurrent HTTP/1.x SSE connections.
* Test the intended small client count.
* Measure JSON bandwidth on actual phones at the proposed rate.
* One process simplifies installation. Simulator integration must not block HTTP.
* Defer FFI selection until the Windows spike establishes compatibility.

## Validation of the choice

* The design serves a small project and separates unverified SDK integration.
* It supports stated requirements and isolated model/HTTP tests.
* It assumes no team experience.

Revisit for remote access, bidirectional commands, persistent history, or substantially larger client counts.
