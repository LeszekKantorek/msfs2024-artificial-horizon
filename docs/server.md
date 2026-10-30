# HTTP and SSE server

The server delivers embedded browser assets, HTTP liveness, and current telemetry from a caller-supplied subscription.
Ownership is divided between the server and application composition:

* The server owns the listening socket and client connections.
* Application composition owns acquisition and coordinated shutdown.

## Public API

The public interfaces have these entry points:

* The crate root re-exports `Server`, `ServerError`, `Config`, and `ConfigError`.
* Routers are public through the `http` module.

| Interface | Contract |
| --- | --- |
| `Server::bind(config, subscription)` | Validate HTTP configuration before acquiring a listener. Return `Result<Server, ServerError>` |
| `Server::local_address()` | Return the bound `SocketAddr` as `io::Result<SocketAddr>` |
| `Server::run(shutdown)` | Consume the server and await caller-owned shutdown with output `()`. Return `io::Result<()>` |
| `http::router()` | Build embedded asset and health routes without telemetry or acquisition |
| `http::router_with_telemetry(subscription)` | Add `/api/v1/events` using the supplied channel, without starting a provider |
| `ServerError::Config` | Invalid HTTP settings, rejected before binding |
| `ServerError::Bind` | Listener failure with the requested address and underlying I/O error |

Server execution follows these constraints:

* The shutdown future for `Server::run` must be `Send + 'static`.
* Only the CLI handles process signals.
* A router alone does not install socket deadlines. `Server::run` wraps accepted sockets with the connection adapter.

### Configuration

`Config` contains HTTP settings only and never reads process arguments:

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| `listen_address` | `IpAddr` | `127.0.0.1` | Reject multicast and `255.255.255.255`. Accept interface or wildcard addresses subject to OS binding |
| `port` | `u16` | `8080` | Accept `1` through `65535`; reject zero |

Configuration follows these rules:

* `Config::validate()` applies the same checks to library and CLI callers.
* `Config::socket_address()` combines the address and port.
* Provider selection is separate from HTTP configuration.
* Use the [deployment constraints](architecture.md#deployment) before exposing the server to a LAN.

## HTTP routes

| Request | Response |
| --- | --- |
| `GET /` | Instrument HTML with relative same-origin asset URLs |
| `GET` for a registered asset | Embedded body and explicit MIME type |
| `HEAD` for a registered asset | Matching response headers without the body |
| `GET /health` | `200` JSON `{"status":"ok"}` when HTTP is serving |
| `GET /api/v1/events` | `200 text/event-stream` when using the telemetry router |
| Unregistered asset path | `404`, without a compatibility facade |

Routes and streaming responses follow these constraints:

* Health reports HTTP liveness, independently of simulator connection, source freshness, or flight readiness.
* Static routes remain independent of telemetry. The static router does not expose SSE.
* There are no write/control endpoints.
* SSE sends `Cache-Control: no-cache, no-transform` and `X-Accel-Buffering: no`.
* Do not compress or buffer SSE responses through a proxy.
* If a proxy is introduced, test its streaming and timeout settings.

## Embedded assets

| Module | Responsibility |
| --- | --- |
| `src/http/assets.rs` | Explicit literal path, MIME type, and compile-time body mappings for HTML, CSS, and current JavaScript modules |
| `assets::router()` | Register the table through one loop |
| `src/http/mod.rs` | Compose assets and health, then add telemetry when supplied |

Asset delivery follows these constraints:

* Embed assets in the binary. Runtime delivery requires no working-directory files or frontend build.
* Preserve paths, MIME types, status codes, response bodies, and GET/HEAD behavior when reorganizing asset code.
* Replaced root and calculation-module routes return 404 without compatibility facades.
* Add no wildcard routes, directory serving, runtime file access, or dependencies for asset registration.
* Update embedded routes and the development fixture server together when browser imports change.

The [EFI composition guide](efi/composition.md#delivery-boundaries) owns browser delivery boundaries and feature integration.

## SSE delivery

Each request owns a clone of the subscription to the single acquisition channel.
Delivery follows this sequence:

1. The first body frame uses `snapshot_and_update()` to read and acknowledge the current state atomically. This prevents duplicate delivery of a pending initial publication.
2. Later frames await `changed()` and serialize the newest snapshot.

Publication and transport remain separate:

* Add no per-client acquisition tasks, sample queues, or cached wire snapshots.
* Pull from the latest-value channel when the HTTP body is polled. Slow consumers can skip intermediate publications.
* Recompute sample age immediately before serialization and suppress expired live data even before the producer's freshness tick.
* Send the latest state on reconnect without replay. Ignore `Last-Event-ID`.
* Keep source state separate from transport state. An idle or unavailable source does not end a session.
* A closed source channel ends the stream after pending state.

Framing and application lifecycle follow these contracts:

* The [wire framing contract](telemetry-contract.md#framing) defines the event name, heartbeat, retry, and envelope rules.
* Application composition retains the channel after producer termination, so existing streams stay open with unavailable data.
* A new client receives the same current unavailable state.
* See the [application lifecycle](architecture.md#application-lifecycle) for startup ordering and failure supervision.

### Connection shutdown

The connection adapter enforces deadlines below the SSE body because a body timeout cannot close a socket blocked on writes.

| Condition | Behavior |
| --- | --- |
| Pending write or flush without progress | Fail after 30 seconds |
| Successful write progress | Clear the write deadline |
| Idle socket with no pending write | No time limit |
| Server shutdown | Stop accepting and allow five seconds from the first shared monotonic shutdown timestamp |
| Shutdown deadline reached | Pending reads and writes return an I/O error |
| Repeated shutdown signal | Preserve the first deadline |
| Dropped or aborted running server future | Signal connection shutdown through the shutdown guard |

Shutdown ownership and evidence have these limits:

* Server shutdown closes HTTP connections without owning or stopping an externally supplied publisher.
* Application shutdown signals both server and provider and collects their outcomes independently.
* Native worker joining has separate constraints. The five-second HTTP grace does not bound a blocked native call.
* Socket progress does not establish phone receipt. Finite HTTP/TCP buffers can still add transit delay.

The lifecycle decisions have these scopes:

* [ADR 0002](adr/0002-sse-connection-lifecycle.md) records the socket design and alternatives.
* [ADR 0007](adr/0007-application-composition.md) supersedes ADR 0002's original producer ownership and fatal-source-failure behavior.

## Validation

Use the [validation scope matrix](testing/index.md#validation-by-change-scope) to select checks for the affected boundary.

| Boundary | Checks |
| --- | --- |
| Configuration, startup, source failure, cancellation, and port reuse | [HTTP startup checks](testing/runtime.md#http-startup-checks) |
| Initial snapshots, clients, framing, and stalled sockets | [SSE checks](testing/runtime.md#sse-and-browser-controller-checks) |
| Embedded routes and browser imports | [EFI validation](testing/pfd.md) |
| Phone recovery and delivery observations | [Real-phone procedure](testing/devices.md#real-phone-acceptance-procedure) |

> Automated socket and fixture checks do not establish actual phone latency, integrated simulator acceptance, or long-duration memory behavior.
