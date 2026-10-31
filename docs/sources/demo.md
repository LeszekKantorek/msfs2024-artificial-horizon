# Demo provider

Demo publishes deterministic synthetic flight samples through the same telemetry channel and HTTP/SSE path as SimConnect.
All subscribers observe one producer's scenario state.

## Requirements and usage

* Select Demo at startup with `--source demo`, or omit the source option for the CLI default.
* Follow the [startup procedure](../../README.md#start-the-demo) and [private LAN setup](../../README.md#use-your-phone).
* Require Windows x64 MSVC, but no simulator, SDK headers, or SimConnect DLL.
* Keep acquisition, scenario values, selected targets, and the fictional speed profile in Rust.
* Keep the browser read-only. It receives values without generating flight data or sending setting/control commands.
* Preserve the [v1 field meanings and optional-value rules](../telemetry-contract.md).
* Never select Demo automatically after a SimConnect failure.

## Public API

The `providers::demo` module exposes these functions:

| Interface | Contract |
| --- | --- |
| `run(publisher, shutdown)` | Consume the supplied `Publisher` and await shutdown with output `()`. Return `Result<(), TelemetryError>` |
| `sample(index)` | Return an extended `FlightSample` for the deterministic sample index |
| `attitude(sample)` | Return the original seven-pose `Attitude` cycle, with 40 samples per pose |
| `speed_profile()` | Return validated fictional thresholds and named references |
| `normalize_degrees(pitch, roll)` | Wrap finite roll to `[-180, 180)` and validate attitude. Return `TelemetryError` for invalid input |

The runtime and its caller have these responsibilities:

* `run` owns separate sample and freshness intervals of 50 ms and skips missed ticks.
* Each sample tick publishes `sample(index)` and advances the index modulo 3960.
* Freshness ticks call `Publisher::expire()` even without sample publication.
* Shutdown takes priority over both ticks.
* The caller supplies a channel identified as `Source::Demo` and owns HTTP and process signals.

## Scenario timing

Playback follows these timing rules:

* Scenario times describe nominal playback at 20 Hz.
* The runtime advances by published sample index, so skipped ticks do not advance the scenario by elapsed wall time.

| Scenario | Samples | Nominal period |
| --- | --- | --- |
| Extended attitude, coordinated turn, speeds, heading, and pressure | 360 | 18 seconds |
| Altitude, selected altitude, and vertical speed | 3960 | 198 seconds |
| Standalone `attitude` helper | 280 | 14 seconds |

Scenario cycles preserve these relationships:

* The full producer cycle contains eleven short cycles.
* The standalone seven-pose helper remains compatible with its original fixtures.
* `sample` adds two extreme-pitch poses without changing that helper.

### Attitude and coordinated turn

Each row covers 40 samples, or two nominal seconds:

| Time | Pitch (degrees) | Roll (degrees) | Slip/skid | Turn rate (deg/s) |
| --- | --- | --- | --- | --- |
| 0–2 s | 0 | 0 | 0 | 0 |
| 2–4 s | 15 | 0 | -0.5 | 0 |
| 4–6 s | -15 | 0 | 0.5 | 0 |
| 6–8 s | 0 | -30 | 0 | -3 |
| 8–10 s | 0 | 30 | 0 | 3 |
| 10–12 s | 10 | 25 | 1 | 3 |
| 12–14 s | -10 | -25 | -1 | -3 |
| 14–16 s | 70 | 0 | 0 | 0 |
| 16–18 s | -50 | 0 | 0 | 0 |

The [attitude feature](../efi/attitude.md#demo-behavior) owns expected visual behavior and regression checks.

### Speed cycle and profile

IAS, TAS, and GS share one fraction while retaining independent values:

| Time | Fraction | IAS acceleration |
| --- | --- | --- |
| 0–2 s | Zero | 0 kt/s |
| 2–8 s | Linear increase from zero to one | +25 kt/s |
| 8–10 s | One | 0 kt/s |
| 10–16 s | Linear decrease to zero | -25 kt/s |
| 16–18 s | Zero | 0 kt/s |

Speed calculations use these values:

* Multiply the fraction by 150 kt for IAS, 160 kt for TAS, and 170 kt for GS.
* At sample `step = index % 360`, the increasing fraction is `(step - 40) / 120`.
* The decreasing fraction is `(320 - step) / 120`.

The demo supplies this fictional configuration:

> These values do not establish the performance of a real aircraft.

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

Acceleration and fixtures follow these rules:

* The demo acceleration is +25 kt/s during acceleration, zero during hold and rest, and -25 kt/s during deceleration.
* At each segment boundary, the new segment supplies the acceleration.
* Independent fixtures reside in `airspeed-profile.json` and `speed-indications.json`.

### Altitude and pressure

The first 18 seconds of the shared 198-second Rust altitude demo contain six three-second sweeps.
Each sweep includes both endpoints over 60 samples:

| Time | Start altitude (ft) | End altitude (ft) |
| --- | --- | --- |
| 0–3 s | -120 | 120 |
| 3–6 s | 980 | 1020 |
| 6–9 s | 9980 | 10020 |
| 9–12 s | 10020 | 9980 |
| 12–15 s | 1020 | 980 |
| 15–18 s | 120 | -120 |

Pressure, display references, and validation follow these rules:

* The producer cycles through `29.92 inHg`, `30.12 inHg`, and `29.72 inHg` at six-second intervals.
* Transitions between sweeps are synthetic test cases, not a physical flight trajectory or a pressure-response model.
* `tests/fixtures/altitude-samples.json` contains independent checkpoints and cycle boundaries.
* [Altitude presentation](../efi/altitude.md#presentation) defines the display limits and rolling drums.
* Convert scenario pressure to `baro_setting.pressure_hpa` with `HPA_PER_INHG`. The browser owns display-unit formatting.

### Selected-altitude phases

The shared altitude, target, and vertical-speed demo repeats after 3960 samples at 20 Hz:

| Time | Altitude / target |
| --- | --- |
| 0-18 seconds | Existing six signed sweeps, without selected altitude |
| 18-108 seconds | Approach 10000 ft from 8500 ft, hold, then depart to 10300 ft |
| 108-198 seconds | Approach 1000 ft from 2500 ft, hold, then depart to 700 ft |

Each 90-second approach uses this schedule:

| Duration | Altitude relative to target |
| --- | --- |
| 6 seconds | Hold 1500 ft away |
| 40 seconds | Move to 500 ft away |
| 20 seconds | Reach target |
| 6 seconds | Hold target |
| 12 seconds | Depart by 300 ft |
| 6 seconds | Hold deviation |

Related cycles and validation remain shared:

* Attitude, speeds, coordinated-turn values, and pressure retain their existing 18-second cycles.
* `tests/fixtures/selected-altitude-samples.json` contains independent checkpoints.
* See [selected-altitude presentation](../efi/selected-altitude.md#presentation) for alert rules and display validity.

### Vertical speed

The demo derives altitude and VSI from shared segment definitions:

| Segment | VSI |
| --- | --- |
| First six altitude sweeps | Analytic slope, approximately +/-4881.36 or +/-813.56 ft/min |
| Target approach and departure | +1500 ft/min upward, -1500 ft/min downward |
| Holds | Zero |
| First sample after artificial jumps | Omitted at cycle indices 0, 60, 120, 240, 300, 360, and 2160 |
| Continuous phase boundary | Rate of the phase beginning at that sample |

VSI calculations and validation use these rules:

* `tests/fixtures/vertical-speed-samples.json` supplies independent segment and jump checkpoints.
* See [VSI presentation](../efi/vertical-speed.md#presentation) for display limits and [VSI acceptance](../efi/vertical-speed.md#testing) for checks.
* The six sweeps use `(end - start) / 59` feet per sample to include both endpoints. Multiply that slope by 1200 samples per minute to obtain the analytic VSI.
* Target phases interpolate over their sample intervals and use the next phase's value at each boundary.

### Heading cycle

The heading demo repeats every 360 samples at 20 Hz, following source turn rate:

| Time | Current heading | Selected heading | Turn rate |
| --- | --- | --- | --- |
| 0-2 seconds | 354 degrees | 350 degrees | 0 deg/s |
| 2-4 seconds | 354 degrees | 90 degrees | 0 deg/s |
| 4-6 seconds | 354 degrees | 180 degrees | 0 deg/s |
| 6-8 seconds | 354 to 348 degrees | 350 degrees | -3 deg/s |
| 8-10 seconds | 348 to 354 degrees | 350 degrees | +3 deg/s |
| 10-12 seconds | 354 to 0 degrees | 0 degrees | +3 deg/s |
| 12-14 seconds | 0 to 354 degrees | 0 degrees | -3 deg/s |
| 14-16 seconds | 354 degrees | Unavailable | 0 deg/s |
| 16-18 seconds | 354 degrees | 354 degrees | 0 deg/s |

Heading boundaries and validation follow these rules:

* Phase boundaries use the starting phase's values.
* Normalization wraps calculations into `[0, 360)`.
* The producer still wraps after 3960 samples.
* The altitude cycle remains 198 seconds.
* `tests/fixtures/heading-samples.json` supplies independent checkpoints, including both north crossings.
* See [heading presentation](../efi/heading.md#presentation) and [heading acceptance](../efi/heading.md#testing).

## Validation

Select validation by scope:

* Use the [telemetry and Demo checks](../testing/runtime.md#telemetry-and-demo-checks) for publication, lifecycle, and fixture validation.
* Use [EFI feature pages](../efi/index.md#features) for instrument-specific rendering and acceptance scenarios.

| Boundary | Expected evidence |
| --- | --- |
| Original and extended attitude | `tests/fixtures/attitudes.json` and `tests/fixtures/pfd-samples.json`, including the seven-pose helper and nine-segment sample cycle |
| Speeds | `tests/fixtures/speed-samples.json`, including simultaneous IAS 150, TAS 160, and GS 170 kt |
| Profile and acceleration | `tests/fixtures/airspeed-profile.json` and `tests/fixtures/speed-indications.json` |
| Shared scenario | Independent altitude, target, VSI, and heading checkpoints referenced in the scenario sections |
| Multiple clients | Matching source samples from one acquisition channel, without per-client scenarios |
| Compatibility | Attitude-only publication clears optional indications; old v1 snapshots remain usable |

> Demo and desktop evidence do not establish simulator compatibility or actual iOS Safari and Android Chrome acceptance.

Keep those results separate under the [device procedure](../testing/devices.md#real-phone-acceptance-procedure).
