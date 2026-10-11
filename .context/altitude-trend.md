---
description: Extend altitude prediction, its left-side tick overlap, or the complementary selected-altitude bug.
status: active
---

Read [altitude trend presentation](../docs/pfd-presentation.md#altitude-trend) before changing the prediction or layout.
Read [altitude trend acceptance](../docs/testing.md#altitude-trend-placement-acceptance) for automated and device checks.

Apply these accepted constraints:

* Predict six seconds from source VSI without estimating browser altitude differences.
* Place the trend over the left-side ticks and below the selected-altitude bug and both opaque readout windows.
* Preserve tape width and return the former four-pixel outside allocation to the horizon and heading strip.
* Use the airspeed trend's magenta color and two-pixel stroke.
* Keep prediction, SVG geometry and invalidation in the altitude composition without another timer.
* Hide on zero rate or loss of required inputs, using the original shared sample deadline.
* Keep valid numeric VSI available when altitude is unavailable, even though the trend is hidden.
* Treat this altitude trend as a project design choice rather than documented Garmin behavior.
* Apply the shared [paint order checks](pfd-layout-validation.md#paint-order-checks) to trend, bug, and readout overlap.

Source: accepted altitude trend and notched-bug amendment, 2026-10-11, with its outside-lane requirement replaced by [#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63).
Source: accepted #63 implementation plan, 2026-10-11.
Evidence limit: desktop checks do not establish real iOS Safari or Android Chrome acceptance.
