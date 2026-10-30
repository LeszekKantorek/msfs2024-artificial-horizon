# HTTP server

The Rust HTTP server serves embedded web assets, health, and telemetry over SSE. It receives a caller-owned subscription and does not select or start a source. The [architecture](architecture.md) defines its relationship with application composition and providers.

## Public server interfaces

`Config` and `Server` are re-exported from the library root. The caller creates the telemetry channel before binding a server.

| Interface | Contract |
| --- | --- |
| `Config` | HTTP listen address and port. Configuration does not read process arguments |
| `Server::bind(config, subscription)` | Validate HTTP configuration before binding. Use the supplied subscription without starting a provider |
| `Server::local_address()` | Return the bound address as `io::Result<SocketAddr>` |
| `Server::run(shutdown)` | Serve until the caller's shutdown future resolves, then begin graceful shutdown of active connections |
| `http::router()` | Build the embedded asset and health routes without telemetry |
| `http::router_with_telemetry(subscription)` | Add SSE using the supplied subscription, without starting acquisition |

`Server::run` accepts a shutdown future returning `()`. The caller handles process signals. The [architecture](architecture.md#application-api) describes application composition and its separate shutdown contract.

## Assets and routes

`src/http/assets.rs` maps each exact path to a MIME type and a body embedded at compile time. Its `assets::router()` registers those entries through one loop. `src/http/mod.rs` adds `/health` to the asset router. The telemetry router adds the SSE route to that base.

The [architecture](architecture.md#panel-boundary) defines asset compatibility and routing constraints. When browser imports change, update both embedded routes and the development fixture server. The [asset integration checks](testing/pfd.md#modular-and-layered-efi-acceptance) describe the required checks.

## Telemetry delivery and shutdown

Each SSE client receives the latest snapshot from its own subscription. The first event reads and acknowledges the current publication atomically. Later events wait for changes; intermediate publications can be skipped. The [subscription contract](architecture.md#library-telemetry-interface) defines the read operations. The [telemetry contract](telemetry-contract.md#framing) defines framing and freshness on the wire.

`/health` reports HTTP liveness independently of source readiness. The connection layer enforces the [socket deadlines](adr/0002-sse-connection-lifecycle.md#socket-deadlines). Server shutdown closes HTTP connections; acquisition and its shutdown remain caller-owned. Use the [runtime checks](testing/runtime.md) to validate startup, delivery, and shutdown.
