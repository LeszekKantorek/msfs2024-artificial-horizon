# ADR 0007: Application composition and independent source lifecycle

Decision status: Accepted.

## Context

HTTP must remain available before the simulator starts and after acquisition fails.
The existing receiver takes ownership of `Publisher`. Releasing its final watch sender would otherwise close SSE streams.
The application needs to retain unavailable state without adding another receiver, sample queue, or distribution layer.

| Responsibility | Governing decision |
| --- | --- |
| Producer ownership, joining, and source-failure behavior | This ADR |
| SSE delivery, channel-closure semantics, socket deadlines, and browser recovery | [ADR 0002](0002-sse-connection-lifecycle.md) |

## Decision

### Component boundaries

| Component or contract | Decision |
| --- | --- |
| Application composition | Select one source, validate configuration, create the channel, bind HTTP, and start the components |
| `Server::bind(config, subscription)` | Receive an existing subscription. The server neither owns a `Publisher` nor starts a provider |
| `Config` | Contain HTTP address and port only |
| `SourceSelection` | Belong to application composition |
| SimConnect | Keep the existing receiver, dedicated native worker, `RunReport`, and optional diagnostics |
| Telemetry and SSE | Keep the v1 contracts. Detailed source errors remain diagnostic output |

### Startup and source failure

Startup preserves these boundaries:

* Bind HTTP before starting acquisition. Invalid configuration or a failed bind prevents producer startup.
* DLL availability, exported functions, simulator connection, and flight readiness are producer concerns and do not delay HTTP startup.

Publication ownership follows this sequence:

1. Composition retains a nonpublishing handle to the same watch channel. Only the provider holds active publication while it runs.
2. After the provider releases its publisher, composition recovers the latest state and publishes `disconnected`.
3. The channel remains available to existing and new HTTP clients until application shutdown.

Recovery preserves source identity, sequence continuity, and the last accepted sample time.

| Source condition | Behavior |
| --- | --- |
| Transient simulator loss | Follow the existing bounded retry policy |
| Permanently failed producer | Report the failure without automatic restart or Demo fallback |
| Normal simulator states | Retain `waiting`, `live`, `paused`, `stale`, `disconnected`, and `invalid` |
| Recovery to live indications | Require a new eligible sample |

### Shutdown

* Use one application coordinator with a shared Tokio `watch<bool>` shutdown signal and task handles.
* Ctrl+C, HTTP failure, or an application infrastructure failure initiates shutdown of both components.
* Producer failure alone does not initiate HTTP shutdown.
* Signal both components before waiting for their independent results.
* HTTP retains the existing five-second connection grace and 30-second write-stall deadline.
* Wait for native cleanup and worker completion. Never abort the native worker.
* Preserve the limitation that a permanently blocked native call has no guaranteed completion deadline.

## Alternatives and consequences

| Alternative | Assessment |
| --- | --- |
| Keep provider ownership inside `Server` | Couples HTTP to acquisition and makes source failure terminate the service |
| Clone an active `Publisher` | Allows conflicting sequence and freshness ownership |
| Add a queue or separate distribution layer | Duplicates the existing latest-value channel without a requirement |
| Add provider traits, registries, or a composition framework | Adds unnecessary interfaces for the two existing source choices |
| Retain the channel and recover publication after release | Preserves the existing distribution contract and one active publisher |

The selected boundary has these consequences:

* The server boundary supports another contract-compatible source through application composition without changing HTTP or browser code.
* Callers create the telemetry channel explicitly and pass its subscription to `Server::bind`.

## Validation and limits

| Evidence scope | Requirement or limit |
| --- | --- |
| Application lifecycle and delivery | Verify startup ordering, retained SSE after source termination, sequence and age continuity, multi-client delivery, and coordinated shutdown |
| Automated tests and independent review | Do not establish simulator or phone compatibility |
| Complete MSFS2024-to-EFI path | Requires separate observations on actual iOS Safari and Android Chrome |
| [Receiver and local-publication evidence](../sources/simconnect.md#live-publication-evidence) | Remains scoped to its recorded environment and does not establish integrated acceptance |
