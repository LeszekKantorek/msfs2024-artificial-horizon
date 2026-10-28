# ADR 0007: Application composition and independent source lifecycle

Decision status: Accepted.

## Context

HTTP must remain available before the simulator starts and after acquisition fails.
The existing receiver takes ownership of `Publisher`; releasing its final watch sender would otherwise close SSE streams.
The application needs to retain unavailable state without adding another receiver, sample queue, or distribution layer.

This decision supersedes ADR 0002 only for producer ownership, joining, and fatal-source-failure behavior.
Its SSE delivery, channel-closure semantics, socket deadlines, and browser recovery rules remain applicable.

## Decision

### Component boundaries

* Application composition selects one source, validates configuration, creates the channel, binds HTTP, and starts the components.
* `Server::bind(config, subscription)` receives an existing subscription. The server neither owns a `Publisher` nor starts a provider.
* `Config` contains HTTP address and port only. `SourceSelection` belongs to application composition.
* Keep the existing SimConnect receiver, dedicated native worker, `RunReport`, and optional diagnostics.
* Keep the v1 telemetry and SSE contracts. Detailed source errors remain diagnostic output.

### Startup and source failure

Bind HTTP before starting acquisition. Invalid configuration or a failed bind prevents producer startup.
DLL availability, exported functions, simulator connection, and flight readiness are producer concerns and do not delay HTTP startup.

Composition retains a nonpublishing handle to the same watch channel.
Only the provider holds active publication while it runs.
After the provider releases its publisher, composition recovers the latest state and publishes `disconnected`.
It preserves source identity, sequence continuity, and the last accepted sample time.
The channel remains available to existing and new HTTP clients until application shutdown.

Transient simulator loss follows the existing bounded retry policy.
A permanently failed producer is reported without automatic restart or Demo fallback.
Normal simulator states remain `waiting`, `live`, `paused`, `stale`, `disconnected`, and `invalid`.
Only a new eligible sample restores live indications.

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

The server boundary supports another contract-compatible source through application composition without changing HTTP or browser code.
The public server API changes; callers must create the telemetry channel explicitly.

## Validation and limits

Required evidence includes startup ordering, retained SSE after source termination, sequence and age continuity, multi-client delivery, and coordinated shutdown.
Automated tests and independent review do not establish simulator or phone compatibility.
The complete MSFS2024-to-EFI path requires separate observations on actual iOS Safari and Android Chrome.
The receiver evidence from #5 remains scoped to its recorded environment and does not establish this integrated acceptance.
