---
description: Extend or validate magnetic-heading telemetry, north wrapping, selected-bug capture, or the top-strip SVG geometry.
status: active
---

Use the canonical heading documents:

| Boundary | Source |
| --- | --- |
| Magnetic units, validity, optional fields, freshness and demo phases | [Heading contract](../docs/telemetry-contract.md#optional-magnetic-headings) |
| Geometry, clip, paint order, notch, formatting and availability | [Heading presentation](../docs/pfd-presentation.md#current-and-selected-heading) |
| Automated checks, screenshot procedure and hardware evidence | [Heading acceptance](../docs/testing.md#heading-acceptance) |

Preserve these decision checks:

* Keep SVG and pure calculations together in `heading.js`, with no timer or history.
* Use raw canonical `[0, 360)` magnetic degrees and format north as `360` without accepting raw 360.
* Show only the three-digit number in the current heading window, retaining degree units in accessibility descriptions.
* At current 359 and selected 1 degrees, expect the bug 8 CSS px right of center.
* Use the shortest angular difference. Exactly 180 degrees goes left.
* Keep the notch and pointer dimensions shared and check actual cyan paint at capture.
* Hide whole labels near the current window and both clip edges.
* Preserve the 32 px top strip and derive horizontal geometry from the current panel allocation.
* Exclude ground-track data, fallback, graphics, and a separate numeric selected heading.
* Validate the full 17-element fixture, keeping real-phone evidence separate from desktop checks.

Source: accepted #26 plan and Garmin section 3.3.8, 2026-10-11.
The original plan remains local under `.local/plans/26-heading/`.

Source: accepted #65 correction, 2026-10-11, removing the degree symbol from the current heading window.

Source: #63 reclaimed four pixels for the central allocation, as reflected in `web/layout.js`, reviewed 2026-10-11.
