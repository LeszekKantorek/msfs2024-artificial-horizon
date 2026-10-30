# Demo provider

Demo publishes deterministic synthetic telemetry through the same channel used by SimConnect. Its source identity is `demo`. It does not load SimConnect or require the simulator SDK, and it is never selected as a fallback for a failed provider.

## Requirements

* Share one producer and scenario among subscribers.
* Publish fresh samples even when numeric values are unchanged.
* Evaluate freshness independently of sample publication and skip missed ticks.
* Keep synthetic values identifiable as Demo. The fictional speed profile does not describe a real aircraft.
* Preserve attitude-only publication and independent availability of optional indications under the [telemetry contract](../telemetry-contract.md).

## Implementation

`src/providers/demo.rs` builds validated `FlightSample` values from a sample index. `demo::run` owns the sample and freshness intervals, both at 50 ms. It publishes a sample on each sample tick, calls `Publisher::expire()` on each freshness tick, and skips missed ticks. The caller supplies the publisher and shutdown signal. Publication errors return to application supervision.

The sample sequence includes attitude, coordinated turn, speed, altitude, pressure, vertical speed, and heading. The original `demo::attitude` helper retains its seven-pose cycle. `demo::sample` adds two extreme pitch poses to the nine-segment attitude cycle.

## Scenario overview

At the nominal 50 ms sample interval, the complete scenario repeats after 3,960 samples, or 198 seconds. Attitude, coordinated turn, speeds, heading, and pressure repeat within that sequence every 18 seconds. Missed sample ticks are skipped, so elapsed wall time can differ from the nominal schedule.

The scenario combines these indications in each sample:

| Indication | Scenario | Detailed expectations |
| --- | --- | --- |
| Attitude and coordinated turn | Nine two-second poses include level, pitch, bank, combined attitudes, and extreme pitch. Slip/skid and turn-rate values exercise both directions | [Attitude and turn demo](../efi/attitude.md#demo-behavior) |
| IAS, TAS, and GS | Independently valued speeds accelerate, hold, and decelerate over 18 seconds. The source also supplies IAS acceleration and a fictional speed profile | [IAS/GS cycle](../efi/airspeed.md#demo-behavior), [TAS acceptance](../efi/true-airspeed.md#true-airspeed-acceptance), and [profile and acceleration](../telemetry-contract.md#optional-airspeed-ranges-and-trend) |
| Heading and selected heading | Current heading follows the demo turn rate and crosses north in both directions. The selected heading changes and is absent in one phase | [Heading phases](../telemetry-contract.md#optional-magnetic-headings) and [heading acceptance](../efi/heading.md#heading-acceptance) |
| Altitude, selected altitude, and VSI | Six altitude sweeps are followed by approaches to two selected altitudes, holds, and departures over the 198-second sequence | [Altitude and pressure](../telemetry-contract.md#optional-altitude-and-pressure-setting), [selected-altitude schedule](../telemetry-contract.md#optional-selected-altitude), and [VSI segments](../telemetry-contract.md#optional-vertical-speed) |
| BARO | The source cycles three fictional pressure settings at six-second intervals, repeating every 18 seconds | [Altitude and pressure](../telemetry-contract.md#optional-altitude-and-pressure-setting) and [BARO presentation](../efi/altitude.md#presentation) |

The altitude and VSI values share segment definitions. VSI is omitted for the first sample after an artificial altitude jump. These transitions exercise display behavior and do not represent a physical flight trajectory. The fictional profile and pressure values do not establish real-aircraft performance or a pressure-response model.

### Altitude and target sequence

| Nominal time | Expected scenario |
| --- | --- |
| 0–18 seconds | Six three-second altitude sweeps with no selected altitude |
| 18–108 seconds | Approach 10,000 ft from 8,500 ft, hold, and depart to 10,300 ft |
| 108–198 seconds | Approach 1,000 ft from 2,500 ft, hold, and depart to 700 ft |

Each approach includes a hold away from the target, movement toward it, a target hold, and a departure. The [selected-altitude schedule](../telemetry-contract.md#optional-selected-altitude) specifies phase durations and independent checkpoints. The [VSI contract](../telemetry-contract.md#optional-vertical-speed) specifies rates and omitted samples at discontinuities.

### Use the scenario for verification

* Observe a complete cycle when checking both target approaches and the wrap to the initial sweeps. Allow at least 198 seconds at nominal cadence.
* Use the shorter cycles to inspect attitude, heading, speed, and pressure changes without waiting for the altitude sequence.
* Compare source values with the linked independent fixtures and feature expectations. Do not infer real simulator behavior from synthetic values.
* Use controlled fixtures for unavailable or invalid fields outside the built-in scenario. The default sequence does not cover every failure case.

## Testing

Use the [telemetry and demo checks](../testing/runtime.md#telemetry-and-demo-checks) for the shared producer. Apply the affected feature's scenarios for changes to its synthetic values. Results from Demo do not establish simulator or real-device acceptance.
