# Backend implementation

The Rust backend selects one telemetry source, retains its latest state, and serves the instrument and telemetry to browsers. The [architecture](architecture.md) defines system boundaries and invariants. This guide explains how the current backend components collaborate.

## Components and data flow

```text
CLI -> application composition -> selected provider -> Publisher
Publisher -> latest state -> Subscription -> HTTP/SSE -> browser
```

| Component | Responsibility |
| --- | --- |
| `src/bin/main.rs` | Parse CLI options, select the source, and supply process shutdown |
| `src/application.rs` | Validate configuration, create the channel, bind HTTP, supervise the provider, and coordinate shutdown |
| `src/providers/` | Produce normalized samples or unavailable states using the supplied publisher |
| `src/telemetry.rs` | Validate model values, retain the latest state, track sequence and age, and provide subscriptions |
| `src/http/` | Serve embedded assets and health, and serialize subscription snapshots as SSE |

The application creates one channel for the selected provider. HTTP receives a subscription and does not start acquisition. Each SSE client consumes the latest state independently. Slow clients can skip intermediate publications without blocking the provider.

[ADR 0007](adr/0007-application-composition.md) records the composition decision and failure behavior. [ADR 0002](adr/0002-sse-connection-lifecycle.md) defines SSE delivery and socket deadlines.

## Public application interfaces

The library root re-exports `Config`, `Source`, and `Server`. The CLI parses process arguments and supplies the shutdown signal. The [SimConnect guide](sources/simconnect.md#command-line-options) lists the CLI options.

| Interface | Contract |
| --- | --- |
| `Config` | HTTP listen address and port only. Library configuration does not read process arguments |
| `Source` | Channel source identity, independent of HTTP configuration |
| `application::SourceSelection` | Select `Demo` or `SimConnect(SimConnectConfig)` |
| `application::run(config, selection, shutdown)` | Validate configuration, create the channel, bind HTTP, start components, and collect their results |
| `Server::bind(config, subscription)` | Validate HTTP configuration before binding. Use the supplied subscription without starting a provider |
| `Server::local_address()` | Return the bound address as `io::Result<SocketAddr>` |
| `Server::run(shutdown)` | Serve HTTP until caller-owned shutdown, then begin graceful shutdown of active connections |
| `http::router()` | Build the embedded asset and health routes without telemetry |
| `http::router_with_telemetry(subscription)` | Add SSE using the supplied subscription, without starting acquisition |
| `providers::simconnect::SimConnectConfig` | Optional absolute DLL override and optional exact log path. It contains no HTTP configuration |
| `providers::simconnect::run(config, publisher, shutdown)` | Acquire using the supplied publisher. Return a `RunReport` with separate acquisition and diagnostic outcomes |

`application::run` and `simconnect::run` accept shutdown futures returning `io::Result<()>`. `Server::run` accepts a shutdown future returning `()`. The caller owns process signals. The library does not install signal handlers.

The [telemetry contract](telemetry-contract.md#model-boundary) describes the model and publication interfaces. The [architecture](architecture.md#library-telemetry-interface) defines subscription acknowledgment behavior.

## Source and lifecycle

The [telemetry sources guide](sources/index.md) lists the available sources and their documentation.

| Source | Implementation | Details |
| --- | --- | --- |
| Demo | Synthetic samples from a deterministic scenario | [Demo provider](sources/demo.md) |
| SimConnect | A native worker acquires and normalizes simulator data | [SimConnect guide](sources/simconnect.md) and [ADR 0006](adr/0006-simconnect-manual-ffi.md) |

Composition validates configuration and binds HTTP before starting acquisition. On producer failure, it retains the channel and publishes `disconnected`. HTTP remains available until shutdown. A permanently failed producer is not restarted and Demo is not selected automatically.

The caller supplies the shutdown signal. Composition signals both components and collects their results. It waits for native cleanup, which has no guaranteed deadline if a native call blocks. The [runtime checks](testing/runtime.md) cover startup, failure, and shutdown behavior.

## Telemetry and HTTP

`telemetry::channel(Source)` returns a publisher and a cloneable subscription. The source identity remains fixed for the channel lifetime. The provider publishes validated samples or lifecycle states. A freshness tick expires stale samples even when no new sample arrives.

A subscription reads the latest snapshot rather than a history queue. HTTP reads and acknowledges the initial snapshot atomically, then waits for changes. The [library telemetry interface](architecture.md#library-telemetry-interface) describes the public read operations. The [telemetry contract](telemetry-contract.md) owns field meanings, validation, freshness, and SSE framing.

The server also serves embedded assets and `/health`. Health reports HTTP liveness, not simulator readiness. [Runtime checks](testing/runtime.md) describe the relevant verification procedures.
