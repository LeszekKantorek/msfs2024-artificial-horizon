# ADR 0003: Extend the demo with a responsive read-only PFD

- Date: 2026-10-08
- Status: Accepted

## Context

The owner approved expanding the attitude demo into the PFD scope defined in the
[project brief](../project-brief.md), with seven feature slices under
[epic #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9).
The G5 illustration is a behavior and visual-hierarchy reference; the phone's
available screen area differs from the physical instrument. Existing simulator
integration, validation and distribution issues #5-#8 must retain their scope.

## Decision

- Deliver each PFD feature through typed Rust data, shared deterministic demo,
  SSE, JavaScript/SVG presentation, tests and documentation in one reviewable slice.
- Keep the GUI read-only and all new values demo-supplied, including selected
  altitude/direction and barometric setting; defer live PFD integration.
- Use responsive layout geometry without a fixed panel aspect ratio; size text,
  symbols and scales independently while preserving familiar relative positions.
- Separate layout, instrument geometry and DOM updates; use explicit SVG groups
  and clipping for moving elements, fixed references, markers and warnings.
- Retain the existing stack, latest-sample animation-frame rendering and no
  interpolation; add only the structure needed for the agreed next slices.
- Extend v1 additively with optional fields; preserve the attitude-only publication
  path, field meanings and lifecycle semantics.
- Missing new data remains unavailable; isolate optional instrument failures while
  source/transport loss invalidates the panel. Never fill missing live data from demo.
- Do not model or reserve space for battery, HSI, CDI, navigation course, ILS,
  GPS glidepath or VNAV.

## Alternatives and trade-offs

| Approach | Trade-off | Decision |
| --- | --- | --- |
| Scale one fixed 4:3 instrument | Simple geometry, but wastes phone space or makes all labels too small together | Reject; mobile readability takes priority |
| Responsive SVG composition | Requires explicit sizing, clipping and resize tests; preserves the existing renderer and testable geometry | Select |
| Replace presentation with Canvas or a frontend framework | Adds migration/tooling cost before existing SVG limits are demonstrated | Defer |
| Expand live integration at the same time | Couples UI delivery to unverified simulator fields and changes #5-#8 | Defer to separate future scope |

## Consequences and validation

The first slice assesses the complete intended arrangement with development-only
fixtures; later slices must preserve phone legibility. Exact wire fields and
fixtures are introduced with their owning feature, not predeclared by this ADR.
Demo profiles and results do not establish real aircraft performance or live
PFD compatibility. Real iOS Safari and Android Chrome evidence belongs to the new
feature issues, with accumulated full-panel acceptance in the final slice.

Revisit these decisions if live PFD data, user controls, interpolation or another
rendering technology becomes an explicitly accepted requirement supported by
measurements. ADR acceptance does not indicate feature implementation or test results.
