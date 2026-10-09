# Telemetry contract v1

> Update this contract and its fixtures together when changing the HTTP/SSE interface.

## Model boundary

| Type / fixture | Contract |
| --- | --- |
| `telemetry::Attitude` | Validated attitude |
| `SlipSkid`, `TurnRate` | Independently validated optional indications |
| `FlightSample` | One acquisition with attitude and independently optional indications |
| `Publisher::publish_sample` | Atomically publish a fresh extended sample with shared sequence and age |
| `Publisher::publish(State)` | Preserve attitude-only and lifecycle publication; clear prior optional values |
| `State` | Typed lifecycle state. Only `State::Live` carries attitude |
| `Snapshot` | Serializable value. Callers cannot construct or mutate its fields |
| `Subscription` | Obtain a current snapshot immediately before serialization |
| `tests/fixtures/snapshots.json` | Independent wire examples |
| `tests/fixtures/attitudes.json` | Independent known poses |

The model exposes no HTTP endpoint.

## HTTP surface

| Request | Response |
| --- | --- |
| `GET /` | Instrument HTML with relative same-origin asset URLs |
| `GET /health` | `200` JSON `{"status":"ok"}` when HTTP is serving |
| `GET /api/v1/events` | `200 text/event-stream`, one stream per page |

* There are no write/control endpoints.
* Health reports process liveness, not flight readiness.
* Send `Cache-Control: no-cache, no-transform` and `X-Accel-Buffering: no` for SSE.
* Do not compress or buffer SSE responses through a proxy.
* If you introduce a proxy, test streaming and timeout settings.

## Framing

```text
retry: 2000
event: telemetry
data: {"schema_version":1,"sequence":42,"source":"demo","state":"live","sample_age_ms":0,"attitude":{"pitch_deg":5.0,"roll_deg":15.0}}

```

1. Send a full named `telemetry` event on subscription.
2. Send a full event on each publication.
3. Send state transitions promptly.
4. End every event with an empty line.

| Timing | Initial value / rule |
| --- | --- |
| Valid samples | Proposed 20 Hz cadence |
| Reconnect hint | `retry: 2000` |
| Page retry | Close the failed EventSource. Create its replacement after 2,000 ms, including initial connection failures |
| Idle heartbeat | SSE comment every 10 seconds when otherwise idle |
| Stalled socket write | Close after 30 seconds without progress |
| Server shutdown | Stop acquisition and end streams. Allow five seconds for remaining connections |

These durations are initial tuning values.

* Keep only one connection and retry timer active per page.
* Omit SSE `id`. Ignore supplied `Last-Event-ID` and send latest state without history replay.
* Reset sequence comparisons for each new subscription.
* Pull from the latest-value channel. Intermediate publications may be skipped, with no sample queue.
* Compute age when producing each body frame. Finite HTTP/TCP buffers may still add transit delay.
* Do not limit session duration or treat an idle source as transport failure.
* A closed source channel ends the stream after pending state.

## Fields and conventions

| Field | Contract |
| --- | --- |
| `schema_version` | Integer `1`. Visibly reject unsupported major versions |
| `sequence` | Nonnegative, increasing publication number within one server run. JSON safe integer |
| `source` | `demo` or `simconnect`. Never silently change on failure |
| `state` | `waiting`, `live`, `paused`, `stale`, `disconnected`, or `invalid` |
| `sample_age_ms` | Nonnegative server-monotonic age of last accepted sample, or `null` before any sample |
| `attitude` | Object only for `live`, otherwise `null` |
| `attitude.pitch_deg` | Finite degrees, positive nose up, range [-90, 90] |
| `attitude.roll_deg` | Finite degrees, positive right wing down, normalized to [-180, 180) |
| `slip_skid` | Optional finite normalized ball displacement in [-1, 1]. Negative left, positive right, zero centered |
| `turn_rate_dps` | Optional finite degrees/second. Negative left, positive right. Standard rate is +/-3 deg/s |

* Providers convert SDK values to these conventions.
* Check conversion in the SimConnect spike. Do not infer SimVar units from names alone.
* If you introduce interpolation, use the shortest angular distance across roll wrap.
* Do not clamp invalid or non-finite values into apparently valid telemetry.

### Optional coordinated-turn indications

* Rust omits absent optional fields and all extensions in non-live snapshots.
* New clients accept missing or `null` fields as unavailable indications.
* Validate each optional value separately from the envelope and attitude.
* An invalid or missing field hides its moving indication and shows a local unavailable mark.
* Never retain a prior optional value when the next snapshot omits or invalidates it.
* Each indication shares the accepted sample time and expires with attitude.
* Source/transport loss invalidates all indications.
* The normalized ball displacement defines this demo's presentation, not future SDK units.
* The turn display spans +/-6 deg/s with +/-3 deg/s marks and edge arrows beyond the display range.
* Finite rates beyond the display range remain valid telemetry; clipping is a display decision.

Extended live example:

```json
{
  "schema_version": 1,
  "sequence": 42,
  "source": "demo",
  "state": "live",
  "sample_age_ms": 0,
  "attitude": {"pitch_deg": 10.0, "roll_deg": 25.0},
  "slip_skid": 1.0,
  "turn_rate_dps": 3.0
}
```

`tests/fixtures/pfd-samples.json` defines independent extended demo values.
`tests/fixtures/snapshots.json` retains unchanged attitude-only v1 examples.

Unavailable state example:

```json
{
  "schema_version": 1,
  "sequence": 43,
  "source": "simconnect",
  "state": "disconnected",
  "sample_age_ms": 1250,
  "attitude": null
}
```

### Compatibility

| Change / input | Required behavior |
| --- | --- |
| Add an optional field | Compatible with v1 |
| Remove or change field meaning, units, or states | Introduce a new version and endpoint |
| Unknown field | Client ignores it |
| Malformed message, unknown state, non-finite/out-of-range attitude | Client rejects it |

## Freshness and status

| Source state | Meaning |
| --- | --- |
| `waiting` | Source connected, but no active flight or usable sample yet |
| `live` | Usable current sample from an active, unpaused source |
| `paused` | Simulator explicitly reports pause. Invalidate displayed live attitude |
| `stale` | No accepted sample for 1,000 ms while expecting live data |
| `disconnected` | Simulator connection unavailable or lost |
| `invalid` | Current source data cannot be normalized or validated |

> Explicit unavailable states override `live`. Only a fresh valid sample from an active, unpaused flight restores `live`.

### Age calculation

```text
estimated age = server sample_age_ms + browser monotonic time since receipt
stale         = estimated age >= 1,000 ms
```

* Compute server age with a monotonic clock immediately before sending.
* Repeated equal values from fresh callbacks are valid samples.
* Publishing a cached value again does not reset age.
* Heartbeats, unknown/malformed events, and connection-open events do not refresh valid-sample age.

> Estimated age excludes network transit time. It does not measure end-to-end latency or replace bounded server buffers.

### Browser indication

* Keep transport states (`connecting` / `reconnecting`) separate from source state.
* Obscure or flag attitude after errors, timeout, background/resume, or unavailable source.
* Never replace unavailable attitude with a credible level horizon.
* Keep DEMO visible in demo mode, including while source state is `live`.

## Further PFD extension boundary

> The fields above define current v1. The remaining [PFD scope](project-brief.md#pfd-reference-and-coverage) defines planned additions.

Each feature slice must:

* Define its optional normalized fields, units, independent validity, and freshness handling.
* Add compatible Rust/browser fixtures with the fields.
* Preserve attitude fields, state meanings, and pitch/bank-only publication.
* Keep old snapshots usable for attitude.
* Invalidate only the affected indication for malformed optional instrument data.
* Preserve existing envelope/attitude validation and global source/transport invalidation.

Constraints:

* Never substitute zero or demo values for missing data.
* The attitude-only SimConnect integration in #6 need not populate PFD extensions.
* Selected values remain source-supplied and read-only.
* Add no control endpoint.
* Feature issues own exact schema additions.
