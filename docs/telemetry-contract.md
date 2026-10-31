# Telemetry contract v1

> Update this contract and its fixtures together when changing the HTTP/SSE interface.

## Model boundary

| Type / fixture | Contract |
| --- | --- |
| `telemetry::Attitude` | Validated attitude |
| `SlipSkid`, `TurnRate` | Independently validated optional indications |
| `IndicatedAirspeed`, `TrueAirspeed`, `GroundSpeed` | Independently validated finite nonnegative knots, with `new` and `knots` methods |
| `SelectedAltitude`, `FlightSample::with_selected_altitude` | Validate finite signed feet and add an independently optional source target |
| `BarometricAltitude` | Finite signed feet, with `new` and `feet` methods; no telemetry display limit |
| `VerticalSpeed` | Finite signed ft/min, with `new` and `feet_per_minute` methods. No telemetry display limit |
| `FlightSample::with_vertical_speed(speed)` | Add independently optional source-supplied vertical speed |
| `Heading`, `SelectedHeading` | Finite magnetic degrees in `[0, 360)`, with `new` and `degrees` methods |
| `FlightSample::with_heading(heading, selected)` | Add independently optional current and selected magnetic heading |
| `FlightSample::with_altitude` | Add independently optional altitude |
| `BaroSetting`, `FlightSample::with_baro_setting` | Add independently optional positive finite hPa pressure |
| `FlightSample` | One acquisition with attitude and independently optional indications |
| `FlightSample::with_speeds(ias, gs)` | Add independently optional speeds. The existing `new` constructor remains attitude/turn-only |
| `AirspeedProfile`, `VSpeedReferences` | Validated positive speed thresholds and four named references in knots |
| `IasAcceleration` | Validated finite IAS acceleration in kt/s. Negative values indicate deceleration |
| `FlightSample::with_true_airspeed(speed)` | Add independently optional true airspeed without changing `with_speeds` or the existing constructor |
| `FlightSample::with_airspeed_indications(profile, acceleration)` | Add independently optional configuration and acceleration without requiring IAS |
| `Publisher::publish_sample` | Atomically publish a fresh extended sample with shared sequence and age |
| `Publisher::publish(State)` | Preserve attitude-only and lifecycle publication; clear prior optional values |
| `State` | Typed lifecycle state. Only `State::Live` carries attitude |
| `Snapshot` | Serializable value. Callers cannot construct or mutate its fields |
| `Subscription` | Obtain a current snapshot immediately before serialization |
| `tests/fixtures/snapshots.json` | Independent wire examples |
| `tests/fixtures/attitudes.json` | Independent known poses |

The model exposes no HTTP endpoint.
`FlightSample` is cloneable rather than copyable. Publication owns its BARO text, and snapshot reads clone the sample.
The additive builders preserve the original attitude/turn constructor and attitude-only publication path.

## HTTP surface

`GET /api/v1/events` delivers the v1 contract as `200 text/event-stream`.
The [server reference](server.md#http-routes) owns asset routes, health, and response behavior.
There are no write/control endpoints.

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
| Valid samples | Source cadence: [Demo](sources/demo.md#scenario-timing) uses 20 Hz; [SimConnect attitude](sources/simconnect.md#attitude-and-freshness) requests each simulator frame |
| Reconnect hint | `retry: 2000` |
| Page retry | Close the failed EventSource. Create its replacement after 2,000 ms, including initial connection failures |
| Idle heartbeat | SSE comment every 10 seconds when otherwise idle |
| Stalled socket write | Close after 30 seconds without progress |
| Server shutdown | End HTTP connections with a five-second grace. Application composition separately stops acquisition |

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
| `tas_kt` | Optional finite nonnegative true airspeed in knots, supplied by the source |
| `gs_kt` | Optional finite nonnegative ground speed in knots |
| `airspeed_profile` | Optional validated speed configuration, defined below |
| `ias_acceleration_ktps` | Optional finite source-supplied IAS acceleration in knots per second |
| `altitude_ft` | Optional finite signed barometric altitude in feet for the source pressure setting |
| `baro_setting` | Optional object with positive finite `pressure_hpa` |
| `selected_altitude_ft` | Optional finite signed selected altitude in feet, supplied by the source and read-only |
| `vertical_speed_fpm` | Optional finite signed feet per minute. Positive means climb, negative means descent, zero means level |
| `heading_deg` | Optional finite magnetic heading in `[0, 360)`. Zero means north |
| `selected_heading_deg` | Independently optional finite selected magnetic heading in `[0, 360)` |

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

* Rust omits absent speeds and all three speed fields in non-live snapshots.
* Each speed is independently optional. Missing, `null`, malformed, negative, or non-finite browser values become unavailable.
* Speeds share the accepted sample's sequence, age, and expiry deadline with attitude.
* A new snapshot replaces all previous speeds. Missing values never become zero or retain the previous reading.
* `Publisher::publish(State)` clears speed fields, including in attitude-only live publications.
* Finite values above the 999 kt display limit remain valid telemetry without clamping.

[Speed presentation](efi/airspeed.md#presentation) owns formatting, rounding, overflow, and unavailable marks.

Speed fields extend the live example above:

```json
{"ias_kt": 99.5, "tas_kt": 110.5, "gs_kt": 137.6}
```

`tests/fixtures/speed-samples.json` defines independent speed-profile checkpoints.

### Optional altitude and pressure setting

* Altitude is a source-supplied indication for its pressure setting; the browser does not calculate pressure corrections.
* Altitude and BARO are independently optional and share the sample sequence, age, and expiry deadline.
* Rust omits absent values and BARO fields outside `live`. Attitude-only publication clears them.
* Missing, null, nonnumeric, or non-finite altitude becomes unavailable independently of pressure and other indications.
* The optional `baro_setting` contains positive finite `pressure_hpa`. Missing, null, or invalid pressure is unavailable independently of altitude.
* The SimConnect provider omits `baro_setting` if pressure is unavailable.
* The browser formats pressure using the local display unit. Do not substitute source-formatted text for missing numeric pressure.
* Each sample replaces the previous altitude and pressure values. Missing values never become zero or retain a previous reading.
* Finite altitude values outside the presentation limits remain valid telemetry.
* Source or transport loss invalidates the entire panel.

Example additions to a live v1 snapshot:

```json
{"altitude_ft": -120.0, "baro_setting": {"pressure_hpa": 1013.25}}
```

See the [Demo provider](sources/demo.md#altitude-and-pressure) for the deterministic scenario and fixture references.

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

See the [Demo provider](sources/demo.md#selected-altitude-phases) for the deterministic scenario and fixture references.

### Optional magnetic headings

Current and selected headings follow the shared optional-field lifecycle:

* Rust validates each finite value in `[0, 360)`. Negative values and 360 are invalid.
* Zero is valid north. The browser formats north as `360` without changing telemetry.
* Missing, null, nonnumeric, non-finite, or out-of-range browser values normalize independently to `null`.
* Each field shares the sample sequence, age, and expiry deadline.
* Rust omits absent fields and both fields outside `live`.
* A new sample replaces both values. Publishing `State` clears both, including attitude-only live publications.
* No ground-track data or fallback is defined. Both headings are source-supplied and read-only.

Example additions to a live v1 snapshot:

```json
{"heading_deg": 359.0, "selected_heading_deg": 1.0}
```

See the [Demo provider](sources/demo.md#heading-cycle) for the deterministic scenario and fixture references.

### Optional vertical speed

Vertical speed follows the optional-field lifecycle:

* `vertical_speed_fpm` shares the sample sequence, age, and expiry deadline.
* Rust omits absent values and the field outside `live`. Attitude-only publication clears the previous value.
* Missing, null, nonnumeric, or non-finite browser values normalize to `null` independently of altitude and pressure.
* Finite signed values beyond presentation limits remain valid telemetry, including negative and zero values.
* Each snapshot replaces the previous indication. Missing values never become zero or retain the previous reading.
* The browser uses source-supplied ft/min without calculating differences between received altitudes.

Example addition to a live v1 snapshot:

```json
{"vertical_speed_fpm": 1500.0}
```

See the [Demo provider](sources/demo.md#vertical-speed) for the deterministic scenario and fixture references.

### Optional airspeed ranges and trend

The profile contains these fields:

| Field | Validation / meaning |
| --- | --- |
| `vs0_kt`, `vs1_kt`, `vfe_kt`, `vno_kt`, `vne_kt` | Independently optional finite positive knots; each rendered interval requires ordered bounds |
| `references.vr_kt`, `references.vx_kt`, `references.vy_kt`, `references.vg_kt` | Independently optional finite positive references; availability does not require VNE |
| `redline_kt` (inside profile) | Optional finite positive current maximum speed in knots, independent of fixed VNE |
| `ias_acceleration_ktps` | Finite kt/s, including negative and zero. Optional independently of the profile |

Rust types expose validated constructors. Profile fields remain private. The wire names identify their units and reference meanings.

These extensions follow the existing sample lifecycle:

* Both fields share the sample sequence, monotonic age, and expiry deadline. Providers must supply acceleration from the same fresh acquisition as IAS.
* Rust omits absent fields and all extensions outside `live`. Attitude-only publication clears both fields.
* A new browser sample replaces the previous configuration and acceleration. Missing values never select a demo default.
* Missing, null, or non-object profiles become unavailable independently of valid IAS, GS, attitude, and acceleration. Malformed members affect only dependent indications.
* Missing, null, or malformed acceleration becomes unavailable independently of the profile and current readings.
* Unknown additive profile fields are ignored. Each known threshold and reference is independently optional. Missing, null, or invalid members are omitted independently; an empty profile is unavailable. No global ordering or reference-to-VNE constraint applies.
* Source/transport loss invalidates the panel. Local IAS loss does not invalidate a valid profile.

The browser never computes acceleration from browser receipt intervals.
[Ranges, references, and trend](efi/airspeed-indications.md#presentation) defines the prediction, clipping, warnings, and fallback presentation.

See the [Demo provider](sources/demo.md#speed-cycle-and-profile) for the deterministic scenario and fixture references.

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
| Add, remove, or change a field, units, or states | Update producers, consumers, and fixtures together. Rebuild and deliver the complete application |
| Earlier application versions or mixed server/browser versions | Unsupported. Provide no legacy adapter or fallback to previous formats or values |
| Unknown field | Client ignores it |
| Malformed message, unknown state, non-finite/out-of-range attitude | Client rejects it |

The server, contract, and embedded Web follow the [application delivery contract](architecture.md#application-delivery-contract).
The endpoint and schema identifier describe the current application contract, not a promise of backward compatibility.
Changes do not require parallel endpoints or retained legacy schemas solely to support earlier application versions.

* `baro_inhg` is unsupported and has no adapter.
* The current `baro_setting.pressure_hpa` field is distinct from `baro_inhg`.
* Update consumers with the delivered server and browser.

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

## Further EFI extension boundary

> The fields above define current v1. The [EFI scope](efi/index.md#product-scope-and-reference) defines coverage and exclusions. Planned additions belong to [roadmap issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9).

Each feature slice must:

* Define its optional normalized fields, units, independent validity, and freshness handling.
* Update Rust and browser fixtures with the fields.
* Preserve attitude fields, state meanings, and pitch/bank-only publication.
* Keep attitude-only publications valid under the current contract. Missing optional fields remain unavailable.
* Invalidate only the affected indication for malformed optional instrument data.
* Preserve existing envelope/attitude validation and global source/transport invalidation.

Constraints:

* Do not advertise unimplemented fields as current contract fields.
* Never substitute zero or demo values for missing data.
* SimConnect publishes available optional instrument data through the shared telemetry contract.
* Selected values remain source-supplied and read-only.
* Add no control endpoint.
* Document wire fields, units, and validation in this contract. Add matching Rust and browser fixtures.
