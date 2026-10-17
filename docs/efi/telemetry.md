# Browser telemetry client

Connection ownership, snapshot normalization, callback publication, and reentrancy limits.

## Requirements

Validate snapshots and optional fields independently. Preserve the original freshness deadline and one connection per client.
Use the [wire freshness contract](../telemetry-contract.md#freshness-and-status) and the [shared frame lifecycle](lifecycle.md).

## Implementation

### Telemetry operations

The telemetry client preserves the connection lifecycle:

* `telemetry/client.js` exports only `createTelemetryClient`.
* Its private operations are `readMessage`, `acceptSample`, `publishSample`, and `expireSample`.
* The current-connection guard precedes parsing.
* Parsing catches message decoding failures without swallowing callback exceptions.

The adjacent validation module exposes these pure functions:

| Function | Result |
| --- | --- |
| `isValidSnapshot(value)` | Boolean envelope and attitude validity |
| `normalizeAirspeedProfile(value)` | Copied valid profile with separate reference values, or `null` |
| `normalizeOptionalIndications(value)` | Independently normalized optional values, including profile and acceleration |

Validation and publication retain these boundaries:

* EventSource ownership, sequence handling, retry/age timers, expiry deadlines, and start/stop remain in the client.
* Both sample callbacks receive separate attitude copies and the same deadline.
* `onAttitude` retains its attitude-only shape.
* The application attaches the current copied status when accepting the complete sample into the scheduler.

Accepted live samples invoke callbacks in this order:

| Order | Callback |
| --- | --- |
| 1 | `onStatus` |
| 2 | `onAttitude` |
| 3 | `onSample` |

Lifecycle state stays with the client and scheduler:

* Expiry retains the original received time and deadline, including rescheduling an early timer.
* One client owns the subscription, retry timer, and age timer.
* One scheduler owns the retained frame and pending draw.

### Callback reentrancy limit

The current client has these limits when a callback changes its lifecycle:

| Callback action | Possible continuation |
| --- | --- |
| Call `stop()` inside `onAttitude` | Invoke `onSample` and schedule expiry for the same acceptance |
| Call `stop()` inside `onSample` | Schedule expiry after the callback returns |
| Accept a nested sample | Retain the original timer behavior |

These limits remain outside the cancellation guarantee:

* An acceptance operation already executing a callback may continue after `stop()`.
* The #56 stage-1 review preserved these legacy behaviors for refactor equivalence.
* Future callers must not rely on these limitations as stable behavior guarantees.
* Changes require separate behavior scope and regression checks for callback-triggered stop, restart, and nested acceptance.

## Testing

Use the [automated commands](../testing/automated.md) and [HTTP/SSE integration checks](../testing/runtime.md).

### Client and validation suites

| Suite | Coverage / method |
| --- | --- |
| Browser controller | Independent fixtures, one EventSource/retry timer, two-second retry, sequence reset, freshness, malformed messages, suspend/resume |
| Telemetry validation | Direct internal-module tests, independent optional fields, old snapshots, copied profile/reference values, and unbounded finite speed input |
| Sample callbacks | Current status before attitude before full sample, with separate attitude copies, original attitude-only shape, and matching deadlines |

### Retired asset routes

| Retired route | Current owner |
| --- | --- |
| `/telemetry-client.js` | `/telemetry/client.js` |
