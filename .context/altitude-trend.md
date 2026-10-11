---
description: Extend altitude prediction, its narrow lane, or the complementary selected-altitude bug.
status: active
---

Read [altitude trend presentation](../docs/pfd-presentation.md#altitude-trend) before changing the prediction or layout.
Read [VSI acceptance](../docs/testing.md#vertical-speed-acceptance) for automated and device checks.

Apply these accepted constraints:

* Predict six seconds from source VSI without estimating browser altitude differences.
* Keep the four-pixel trend lane outside the unchanged altitude tape and readout widths.
* Use the airspeed trend's magenta color and two-pixel stroke.
* Keep prediction, SVG geometry and invalidation in the altitude composition without another timer.
* Hide on zero rate or loss of required inputs, using the original shared sample deadline.
* Keep valid numeric VSI available when altitude is unavailable, even though the trend is hidden.
* Treat this altitude trend as a project design choice rather than documented Garmin behavior.

Source: accepted altitude trend and notched-bug amendment, 2026-10-11.
Evidence limit: desktop checks do not establish real iOS Safari or Android Chrome acceptance.
