# ADR 0003: Extend the demo with a responsive read-only PFD

Decision status: Accepted.

## Context

* The [EFI scope](../efi/index.md#product-scope-and-reference) defines instrument coverage and exclusions.
* The G5 illustration guides behavior and visual hierarchy. Phone screen area differs from the physical instrument.
* Demo results, simulator observations, and device acceptance establish different evidence scopes.

## Decision

| Area | Rule |
| --- | --- |
| Delivery | Each feature includes typed Rust data, shared deterministic demo, SSE, JavaScript/SVG, tests, and documentation |
| Data | Demo supplies deterministic scenarios, including selected altitude/direction and barometric setting. SimConnect supplies available source data |
| Controls | Send no simulator commands. Permit the [local BARO display preference](#local-baro-display-preference) |
| Layout | No fixed aspect ratio. Size text, symbols, and scales independently. Preserve familiar relative positions |
| Geometry | Separate layout, instrument geometry, and DOM updates |
| SVG | Use explicit groups and clipping for moving elements, fixed references, markers, and warnings |
| Rendering | Retain the stack and latest-sample animation frames without interpolation |
| Structure | Add only what accepted feature requirements need |
| Contract | Add optional v1 fields. Preserve attitude-only publication, field meanings, and lifecycle semantics |
| Missing data | Show unavailable. Never fill missing live data with demo values |
| Invalid data | Isolate optional instrument failures. Source/transport loss invalidates the panel |
| Exclusions | No models or reserved space for battery, HSI, CDI, navigation course, ILS, GPS glidepath, or VNAV |

### Local BARO display preference

The browser provides these local display controls:

* Touching or clicking the BARO field cycles through inHg, hPa, and STD when valid numeric BARO data is available.
* Store the preference locally and redraw the current fresh frame.
* Use an in-memory preference when storage is unavailable.
* Send no setting or control command to the server or simulator.
* The local STD label does not indicate the simulator's standard mode.

Local selection provides unit choice without a transport or simulator dependency.
A supported universal cockpit display-unit source is not established.
This decision does not establish cockpit unit synchronization or actual-device usability.

## Alternatives and trade-offs

| Approach | Trade-off | Decision |
| --- | --- | --- |
| Scale one fixed 4:3 instrument | Simple geometry wastes phone space or makes all labels too small | Reject for mobile readability |
| Responsive SVG composition | Requires explicit sizing, clipping, and resize tests. Preserves the renderer and testable geometry | Select |
| Canvas or frontend framework | Adds migration/tooling before measurements establish SVG limits | Defer |
| Treat Demo acceptance as live acceptance | Synthetic data cannot establish simulator field behavior | Require separate simulator and device evidence |
| Fixed BARO display without local selection | Does not provide local unit choice | Use a local preference |
| Read cockpit display-unit preference | Could synchronize units, but no supported universal source is established | Use a local preference |

## Consequences and validation

* Assess the complete arrangement with development-only composition fixtures.
* Preserve phone readability when changing instruments.
* Introduce exact wire fields and fixtures with their owning feature, not this ADR.
* Demo profiles/results establish neither real-aircraft performance nor live PFD compatibility.
* Record real iOS Safari and Android Chrome evidence for the affected features.
* Record full-panel acceptance separately from individual instrument checks.

> ADR acceptance records a decision. It does not indicate implementation or test results.

Revisit for simulator controls, a supported cockpit unit source, interpolation, or another rendering technology supported by measurements.
