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

The [attitude feature](../efi/attitude.md#demo-behavior) owns pose expectations. The [telemetry contract](../telemetry-contract.md) and relevant [EFI feature pages](../efi/index.md#features) describe individual values and scenarios. The [runtime demo checks](../testing/runtime.md#demo-behavior) define shared cadence, expiry, and source-failure expectations.

## Testing

Use the [telemetry and demo checks](../testing/runtime.md#telemetry-and-demo-checks) for the shared producer. Apply the affected feature's scenarios for changes to its synthetic values. Results from Demo do not establish simulator or real-device acceptance.
