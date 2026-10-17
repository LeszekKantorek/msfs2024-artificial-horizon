---
description: Locate selected-altitude rules and preserve alert-history, scheduling, and SVG validation lessons.
status: active
---

Select the source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Optional target, freshness, and shared altitude demo | [Selected-altitude telemetry](../docs/telemetry-contract.md#optional-selected-altitude) |
| Capture, bug geometry, thresholds, flashing, and deviation | [Selected-altitude presentation](../docs/pfd-presentation.md#selected-altitude-and-alerts) |
| Per-sample acceptance and timed rendering | [Panel lifecycle](../docs/pfd-presentation.md#panel-interface-and-frame-lifecycle) |
| Automated and actual-device checks | [Selected-altitude acceptance](../docs/testing.md#selected-altitude-and-alert-acceptance) |

These lessons explain the lifecycle and validation constraints:

* Advance history before coalescing draws so transitions between animation frames remain observable.
* Reset expired history before accepting recovery data. The expiry callback may not have run yet.
* Keep history with the selected visual object and timing with the common scheduler to preserve one cancellation path.
* Entry into +/-200 ft, silent initialization, 2 Hz flashing, and persistent yellow are project decisions, not fully specified Garmin behavior.
* Compare SVG matrix positions with a subpixel tolerance because fractional values use single-precision matrices.

Source: accepted #24 plan and implementation, 2026-10-11.
Source: accepted #25 plan, 2026-10-11, extending the altitude cycle from 90 to 198 seconds for coherent VSI.
Source: accepted notched-bug and separate trend-lane amendment, 2026-10-11, replacing the earlier bug paint order.
Source: accepted [#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63) plan, 2026-10-11, replacing the outside trend lane with tick overlap below the bug.
Evidence limit: desktop and automated results do not establish real iOS Safari or Android Chrome acceptance.
