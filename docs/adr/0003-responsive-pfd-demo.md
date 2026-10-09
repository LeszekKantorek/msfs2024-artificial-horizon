# ADR 0003: Extend the demo with a responsive read-only PFD

| Date | Status |
| --- | --- |
| 2026-10-08 | Accepted |

## Context

* The [PFD scope](../project-brief.md) was approved as seven feature slices under [epic #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9).
* The G5 illustration guides behavior and visual hierarchy. Phone screen area differs from the physical instrument.
* Existing simulator integration, validation, and distribution issues #5-#8 retain their scope.

## Decision

| Area | Rule |
| --- | --- |
| Delivery | Each slice includes typed Rust data, shared deterministic demo, SSE, JavaScript/SVG, tests, and documentation |
| Data | Demo supplies all new values, including selected altitude/direction and barometric setting. Defer live PFD integration |
| Controls | Keep the GUI read-only |
| Layout | No fixed aspect ratio. Size text, symbols, and scales independently. Preserve familiar relative positions |
| Geometry | Separate layout, instrument geometry, and DOM updates |
| SVG | Use explicit groups and clipping for moving elements, fixed references, markers, and warnings |
| Rendering | Retain the stack and latest-sample animation frames without interpolation |
| Structure | Add only what the agreed next slices need |
| Contract | Add optional v1 fields. Preserve attitude-only publication, field meanings, and lifecycle semantics |
| Missing data | Show unavailable. Never fill missing live data with demo values |
| Invalid data | Isolate optional instrument failures. Source/transport loss invalidates the panel |
| Exclusions | No models or reserved space for battery, HSI, CDI, navigation course, ILS, GPS glidepath, or VNAV |

## Alternatives and trade-offs

| Approach | Trade-off | Decision |
| --- | --- | --- |
| Scale one fixed 4:3 instrument | Simple geometry wastes phone space or makes all labels too small | Reject for mobile readability |
| Responsive SVG composition | Requires explicit sizing, clipping, and resize tests. Preserves the renderer and testable geometry | Select |
| Canvas or frontend framework | Adds migration/tooling before measurements establish SVG limits | Defer |
| Concurrent live integration | Depends on unverified simulator fields and changes #5-#8 | Defer to separate scope |

## Consequences and validation

* Assess the complete arrangement in the first slice with development-only fixtures.
* Preserve phone readability through later slices.
* Introduce exact wire fields and fixtures with their owning feature, not this ADR.
* Demo profiles/results establish neither real-aircraft performance nor live PFD compatibility.
* Record real iOS Safari and Android Chrome evidence in each feature issue.
* Record accumulated full-panel acceptance in the final slice.

> ADR acceptance records a decision. It does not indicate implementation or test results.

Revisit for explicitly accepted live PFD data, user controls, interpolation, or another rendering technology supported by measurements.
