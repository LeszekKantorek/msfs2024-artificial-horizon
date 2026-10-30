# Telemetry sources

The application selects one source at startup. Each source publishes through the shared telemetry channel. A source failure never selects Demo automatically.

| Source | Data | Documentation |
| --- | --- | --- |
| Demo | Deterministic synthetic samples | [Demo provider](demo.md) |
| SimConnect | Simulator observations acquired by a native worker | [SimConnect integration](simconnect.md) |

The [backend implementation](../backend-implementation.md) describes composition and delivery. The [telemetry contract](../telemetry-contract.md) defines shared fields, validity, freshness, and wire behavior. The [README](../../README.md) describes startup and source selection.
