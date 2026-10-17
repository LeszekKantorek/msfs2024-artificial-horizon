---
description: Locate heading contracts and preserve north-wrap, capture, and full-panel validation lessons.
status: active
---

Select the source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Magnetic units, validity, optional fields, freshness, and demo phases | [Heading contract](../docs/telemetry-contract.md#optional-magnetic-headings) |
| Geometry, clip, paint order, notch, formatting, and availability | [Heading presentation](../docs/pfd-presentation.md#current-and-selected-heading) |
| Automated checks, screenshot procedure, and hardware evidence | [Heading acceptance](../docs/testing.md#heading-acceptance) |

Preserve these validation lessons and decision boundaries:

* North displays as `360`, but raw 360 is invalid. Distinguish source validation from display formatting.
* The 359/1-degree capture case and 180-degree left tie detect wrap and direction errors.
* Check actual cyan paint at capture and whole-label hiding at both the current window and clip edges.
* The accepted #26 scope excludes ground-track fallback and a separate numeric selected heading.
* Use the full 17-element fixture for accumulated acceptance. Keep real-phone evidence separate from desktop checks.

Source: accepted #26 plan and Garmin section 3.3.8, 2026-10-11.
The original plan remains local under `.local/plans/26-heading/`.

Source: accepted #65 correction, 2026-10-11, removing the degree symbol from the current heading window.

Source: #63 reclaimed four pixels for the central allocation, as reflected in `web/layout.js`, reviewed 2026-10-11.
