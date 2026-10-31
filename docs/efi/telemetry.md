# Browser telemetry client

Connection ownership, snapshot normalization, callback publication, and reentrancy limits.

## Requirements

* Validate snapshots and optional fields independently.
* Preserve the original freshness deadline and one connection per client.
* Keep EventSource ownership, sequence handling, retry/age timers, expiry deadlines, and start/stop in the client.

Use the [wire freshness contract](../telemetry-contract.md#freshness-and-status) and the [shared frame lifecycle](lifecycle.md).

## Implementation

### Client interface and ownership

`telemetry/client.js` exports only `createTelemetryClient`.
Its private operations are `readMessage`, `acceptSample`, `publishSample`, and `expireSample`.

| Interface | Contract |
| --- | --- |
| `onStatus` | Required callback receiving a copied status |
| `onAttitude`, `onSample` | Optional callbacks, defaulting to no operation |
| `makeEventSource`, `now`, `schedule`, `cancel` | Injectable connection, monotonic clock, and timer dependencies |
| `start()` | Activate the client and connect. Repeated calls while active do nothing |
| `stop()` | Close the connection, clear retry and age timers, and report suspended/waiting status, subject to [callback reentrancy limits](#callback-reentrancy-limits) |

One client owns the subscription, retry timer, and age timer.
The [scheduler](lifecycle.md#sample-acceptance-and-drawing) owns the retained frame and pending draw.

### Snapshot validation and optional normalization

The adjacent validation module exposes these pure functions:

| Function | Result |
| --- | --- |
| `isValidSnapshot(value)` | Boolean envelope and attitude validity |
| `normalizeAirspeedProfile(value)` | Copied valid profile with separate reference values, or `null` |
| `normalizeOptionalIndications(value)` | Independently normalized optional values, including profile and acceleration |

### Sample acceptance and callback publication

* Check the current connection before parsing a message.
* Catch message decoding failures without swallowing callback exceptions.
* Give both sample callbacks separate attitude copies and the same deadline.
* Preserve the attitude-only shape of `onAttitude`.
* Attach the current copied status in the application before accepting the complete sample into the scheduler.

Accepted live samples invoke callbacks in this order:

1. `onStatus`.
2. `onAttitude`.
3. `onSample`.

### Connection lifecycle and freshness

| Event or condition | Client behavior |
| --- | --- |
| New connection | Open `/api/v1/events` and reset the sequence to `-1` |
| Connection opens | Report connected/waiting status |
| Connection fails or construction throws | Close the connection, clear the age timer, report reconnecting/waiting, and retry after 2000 ms while active |
| Malformed or invalid snapshot | Clear the age timer and report invalid state |
| Valid snapshot with sequence no greater than the retained sequence | Ignore the snapshot |
| Accepted live snapshot | Subtract `sample_age_ms` from 1000 ms, using zero age when absent. Report stale if no freshness remains |
| Fresh live snapshot | Publish with the deadline `receivedAt + remaining` and schedule expiry |
| Early expiry timer | Reschedule using the original received time and remaining duration |
| Expiry deadline reached | Report stale even without another sample |
| Hidden page or `pagehide` | Application calls `stop()` |
| Visible page or visible `pageshow` | Application calls `start()` |

### Callback reentrancy limits

The current client has these limits when a callback changes its lifecycle:

| Callback action | Possible continuation |
| --- | --- |
| Call `stop()` inside `onAttitude` | Invoke `onSample` and schedule expiry for the same acceptance |
| Call `stop()` inside `onSample` | Schedule expiry after the callback returns |
| Accept a nested sample | Retain the original timer behavior |

These limits remain outside the cancellation guarantee:

* An acceptance operation already executing a callback may continue after `stop()`.
* Future callers must not rely on these limitations as stable behavior guarantees.
* Changes require separate behavior scope and regression checks for callback-triggered stop, restart, and nested acceptance.

## Testing

Use the [automated commands](../testing/automated.md) and [HTTP/SSE integration checks](../testing/runtime.md).
Verify the [asset registration contract](../server.md#embedded-assets): unregistered paths return `404` without compatibility aliases.

### Client and validation coverage

| Suite | Coverage and method |
| --- | --- |
| Browser controller | Independent fixtures, one EventSource/retry timer, two-second retry, sequence reset, freshness, malformed messages, suspend/resume |
| Telemetry validation | Direct internal-module tests, independent optional fields, attitude-only snapshots, copied profile/reference values, and unbounded finite speed input |
| Sample callbacks | Current status before attitude before full sample, with separate attitude copies, attitude-only shape, and matching deadlines |

### Evidence limits

> Client and validation tests do not establish integrated simulator-to-browser or real-phone acceptance. Use the [device procedure](../testing/devices.md#real-phone-acceptance-procedure) for device evidence.
