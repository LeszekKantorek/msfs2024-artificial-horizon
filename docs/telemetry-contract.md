# Telemetry contract v1

Status: proposed implementation contract. Update this document and its fixtures
together if implementation evidence requires a change.

## HTTP surface

| Request | Response |
| --- | --- |
| `GET /` | Instrument HTML; relative same-origin asset URLs |
| `GET /health` | `200` JSON `{"status":"ok"}` when HTTP is serving |
| `GET /api/v1/events` | `200 text/event-stream`; one stream per page |

There are no write/control endpoints. Health reports process liveness, not flight readiness.
For SSE use `Cache-Control: no-cache, no-transform`; avoid response compression and
proxy buffering. If a proxy is introduced, test streaming and timeout configuration.

## Framing

Send a full named `telemetry` event on subscription and on each publication. A
proposed 20 Hz cadence applies while receiving valid samples; state transitions
must also be sent promptly. Use `retry: 1000` as a reconnect hint and an SSE comment
every 10 seconds when otherwise idle. These durations are initial tuning values.

```text
retry: 1000
event: telemetry
data: {"schema_version":1,"sequence":42,"source":"demo","state":"live","sample_age_ms":0,"attitude":{"pitch_deg":5.0,"roll_deg":15.0}}

```

End every event with an empty line. Omit SSE `id`: history replay is not supported.
Ignore `Last-Event-ID` if supplied and send the latest state. Browser reconnection
starts a new subscription; sequence comparisons reset on that subscription.

## Fields and conventions

| Field | Contract |
| --- | --- |
| `schema_version` | Integer `1`; reject unsupported major versions visibly |
| `sequence` | Nonnegative, increasing publication number within one server run; JSON safe integer |
| `source` | `demo` or `simconnect`; never silently changed on failure |
| `state` | `waiting`, `live`, `paused`, `stale`, `disconnected`, or `invalid` |
| `sample_age_ms` | Nonnegative server-monotonic age of last accepted sample, or `null` before any sample |
| `attitude` | Object only for `live`; otherwise `null` |
| `attitude.pitch_deg` | Finite degrees, positive nose up; range [-90, 90] |
| `attitude.roll_deg` | Finite degrees, positive right wing down; normalized to [-180, 180) |

The provider converts SDK values to these conventions; never infer a SimVar's
units from its name alone. Verify conversion in the SimConnect spike. Handle the
roll wrap via the shortest angular distance if interpolating. Do not clamp
non-finite or invalid values into apparently valid telemetry.

Example unavailable state:

```json
{"schema_version":1,"sequence":43,"source":"simconnect","state":"disconnected","sample_age_ms":1250,"attitude":null}
```

An additive optional field is compatible with v1. Removing/changing field meaning,
units, or states requires a new version and endpoint. Clients ignore unknown fields
but reject malformed messages, unknown states, and non-finite/out-of-range attitude.

## Freshness and status

- `waiting`: connected source without an active flight/usable sample yet.
- `live`: usable current sample and an active, unpaused source.
- `paused`: simulator explicitly reports pause; invalidate the displayed live attitude.
- `stale`: no accepted sample for 1,000 ms while otherwise expecting live data.
- `disconnected`: simulator connection is unavailable/lost.
- `invalid`: current source data cannot be normalized or validated.

Explicit unavailable states override `live`. Only a fresh valid sample from an
active unpaused flight can restore `live`. Repeated equal values with fresh source
callbacks are valid samples; re-emitting a cached value does not reset its age.

Compute age using a monotonic server clock immediately before sending. The browser
adds elapsed monotonic time since receipt to `sample_age_ms`; at 1,000 ms it flags
stale attitude. Heartbeats, unknown/malformed events, and connection-open events do
not refresh valid-sample age. This estimate excludes network transit time, so it is
not an end-to-end latency measurement or a substitute for bounded server buffers.

Browser transport states (`connecting` / `reconnecting`) are separate from source
state. On errors, timeout, background/resume, or unavailable source, obscure/flag
the attitude instead of resetting to a credible level horizon. Keep a DEMO indicator
visible in demo mode even while source state is `live`.
