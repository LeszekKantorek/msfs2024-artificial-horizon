# Telemetry contract v1

> Update this contract and its fixtures together when changing the HTTP/SSE interface.

## Model boundary

| Type / fixture | Contract |
| --- | --- |
| `telemetry::Attitude` | Validated attitude |
| `SlipSkid`, `TurnRate` | Independently validated optional indications |
| `IndicatedAirspeed`, `GroundSpeed` | Independently validated finite nonnegative knots, with `new` and `knots` methods |
| `BarometricAltitude` | Finite signed feet, with `new` and `feet` methods; no telemetry display limit |
| `BarometricSetting` | Positive finite inHg, with `new` and `inches_of_mercury` methods |
| `FlightSample::with_altitude(altitude, setting)` | Add independently optional altitude and pressure setting |
| `FlightSample` | One acquisition with attitude and independently optional indications |
| `FlightSample::with_speeds(ias, gs)` | Add independently optional speeds. The existing `new` constructor remains attitude/turn-only |
| `AirspeedProfile`, `VSpeedReferences` | Validated positive speed thresholds and four named references in knots |
| `IasAcceleration` | Validated finite IAS acceleration in kt/s. Negative values indicate deceleration |
| `FlightSample::with_airspeed_indications(profile, acceleration)` | Add independently optional configuration and acceleration without requiring IAS |
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
| `ias_kt` | Optional finite nonnegative indicated airspeed in knots |
| `gs_kt` | Optional finite nonnegative ground speed in knots |
| `airspeed_profile` | Optional validated speed configuration, defined below |
| `ias_acceleration_ktps` | Optional finite source-supplied IAS acceleration in knots per second |
| `altitude_ft` | Optional finite signed barometric altitude in feet for the source pressure setting |
| `baro_inhg` | Optional finite positive pressure setting in inches of mercury |
| `selected_altitude_ft` | Optional finite signed selected altitude in feet, supplied by the source and read-only |

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

### Optional speed indications

* Rust omits absent speeds and both speed fields in non-live snapshots.
* Each speed is independently optional. Missing, `null`, malformed, negative, or non-finite browser values become unavailable.
* Speeds share the accepted sample's sequence, age, and expiry deadline with attitude.
* A new snapshot replaces both previous speeds. Missing values never become zero or retain the previous reading.
* `Publisher::publish(State)` clears speed fields, including in attitude-only live publications.
* Finite values above the 999 kt display limit remain valid telemetry without clamping.

[Speed presentation](pfd-presentation.md#airspeed-and-ground-speed) owns formatting, rounding, overflow, and unavailable marks.

Speed fields extend the live example above:

```json
{"ias_kt": 99.5, "gs_kt": 137.6}
```

`tests/fixtures/speed-samples.json` defines independent speed-profile checkpoints.

### Optional altitude and pressure setting

* Altitude is a source-supplied indication for its pressure setting; the browser does not calculate pressure corrections.
* Both fields are independently optional and share the sample sequence, age, and expiry deadline.
* Rust omits absent values and both fields outside `live`; attitude-only publication clears both.
* Missing, null, nonnumeric, or non-finite altitude becomes unavailable independently of pressure and other indications.
* Missing, null, nonnumeric, non-finite, or nonpositive pressure becomes unavailable independently of altitude.
* Each sample replaces both previous values; missing values never become zero or retain a previous reading.
* Finite values outside the presentation limits remain valid telemetry.
* Source or transport loss invalidates the entire panel.

Example additions to a live v1 snapshot:

```json
{"altitude_ft": -120.0, "baro_inhg": 29.92}
```

The first 18 seconds of the shared 90-second Rust altitude demo contain six three-second sweeps.
Their endpoints are -120/120, 980/1020, 9980/10020, 10020/9980, 1020/980, and 120/-120 ft.
Each sweep includes both endpoints over 60 samples.
Pressure changes every six seconds through 29.92, 30.12, and 29.72 inHg.
Transitions between sweeps are synthetic test cases, not a physical flight trajectory or a pressure-response model.

`tests/fixtures/altitude-samples.json` contains independent checkpoints and cycle boundaries.
[Altitude presentation](pfd-presentation.md#altitude-and-barometric-setting) defines the display limits and rolling drums.

### Optional selected altitude

* `selected_altitude_ft` is independently optional and shares the sample sequence, age, and expiry deadline.
* Rust exposes `SelectedAltitude::new(feet)` and `FlightSample::with_selected_altitude` without changing existing constructors.
* Rust omits absent targets and the field outside `live`.
* Attitude-only publication clears the target.
* Missing, null, nonnumeric, or non-finite targets normalize to `null` without invalidating other indications.
* Every sample replaces the previous target.
* Missing targets never retain a value or become zero.
* Finite signed targets beyond display limits remain valid telemetry.
* Targets use the same indicated-altitude reference as `altitude_ft`.
* The browser applies no pressure correction.
* Alert history, flash phases, and monotonic visual deadlines remain browser presentation state, outside the wire schema.

Example addition to a live v1 snapshot:

```json
{"selected_altitude_ft": 10000.0}
```

The altitude demo repeats after 1800 samples at 20 Hz:

| Time | Altitude / target |
| --- | --- |
| 0-18 seconds | Existing six signed sweeps, without selected altitude |
| 18-54 seconds | Approach 10000 ft from 8500 ft, hold, then depart to 10300 ft |
| 54-90 seconds | Approach 1000 ft from 2500 ft, hold, then depart to 700 ft |

Each 36-second approach uses this schedule:

| Duration | Altitude relative to target |
| --- | --- |
| 6 seconds | Hold 1500 ft away |
| 8 seconds | Move to 500 ft away |
| 8 seconds | Reach target |
| 6 seconds | Hold target |
| 2 seconds | Depart by 300 ft |
| 6 seconds | Hold deviation |

Attitude, speeds, coordinated-turn values, and pressure retain their existing 18-second cycles.
`tests/fixtures/selected-altitude-samples.json` contains independent checkpoints.
See [selected-altitude presentation](pfd-presentation.md#selected-altitude-and-alerts) for alert rules and display validity.

### Optional airspeed ranges and trend

The profile contains these fields:

| Field | Validation / meaning |
| --- | --- |
| `vs0_kt`, `vs1_kt`, `vfe_kt`, `vno_kt`, `vne_kt` | Finite positive knots, strictly increasing in this order |
| `references.vr_kt`, `references.vx_kt`, `references.vy_kt`, `references.vg_kt` | Four finite positive configured references, each at or below VNE |
| `ias_acceleration_ktps` | Finite kt/s, including negative and zero. Optional independently of the profile |

Rust types expose validated constructors. Profile fields remain private. The wire names identify their units and reference meanings.

The demo supplies this fictional configuration. These values do not establish the performance of a real aircraft.

```json
{
  "airspeed_profile": {
    "vs0_kt": 40.0, "vs1_kt": 50.0, "vfe_kt": 85.0,
    "vno_kt": 110.0, "vne_kt": 140.0,
    "references": {"vr_kt": 55.0, "vx_kt": 65.0, "vy_kt": 75.0, "vg_kt": 90.0}
  },
  "ias_acceleration_ktps": 25.0
}
```

These extensions follow the existing sample lifecycle:

* Both fields share the sample sequence, monotonic age, and expiry deadline. Providers must supply acceleration from the same fresh acquisition as IAS.
* Rust omits absent fields and all extensions outside `live`. Attitude-only publication clears both fields.
* A new browser sample replaces the previous configuration and acceleration. Missing values never select a demo default.
* Missing, null, or malformed profiles become unavailable independently of valid IAS, GS, attitude, and acceleration.
* Missing, null, or malformed acceleration becomes unavailable independently of the profile and current readings.
* Unknown additive profile fields are ignored. All five known thresholds and four named references are required when a profile is supplied.
* Source/transport loss invalidates the panel. Local IAS loss does not invalidate a valid profile.

The browser never computes acceleration from browser receipt intervals.
[Ranges, references, and trend](pfd-presentation.md#ranges-references-and-trend) defines the prediction, clipping, warnings, and fallback presentation.

The demo acceleration is +25 kt/s during acceleration, zero during hold and rest, and -25 kt/s during deceleration.
At each segment boundary, the new segment supplies the acceleration. Independent fixtures reside in `airspeed-profile.json` and `speed-indications.json`.

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
* Keep source identity in telemetry and internal client state; show no DEMO label or announcement.

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
