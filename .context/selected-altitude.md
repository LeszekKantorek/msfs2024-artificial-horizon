---
description: Extend selected-altitude targets, alert history, timed panel updates, or the longer altitude demo.
status: active
---

Read the canonical rules for the affected boundary:

| Boundary | Source |
| --- | --- |
| Optional target, freshness, and shared altitude demo | [Selected-altitude telemetry](../docs/telemetry-contract.md#optional-selected-altitude) |
| Capture, baseline, thresholds, flashing, and yellow deviation | [Selected-altitude presentation](../docs/pfd-presentation.md#selected-altitude-and-alerts) |
| Per-sample acceptance and timed rendering | [Panel lifecycle](../docs/pfd-presentation.md#panel-interface-and-frame-lifecycle) |
| Automated and actual-device checks | [Selected-altitude acceptance](../docs/testing.md#selected-altitude-and-alert-acceptance) |

* Process every accepted fresh sample before coalescing draws so transitions between animation frames remain observable.
* Keep alert history with the selected visual object and timers with the common scheduler.
* Reset retained alert history before accepting recovery data if the previous sample expired, even when its expiry callback has not run.
* Paint the selected bug after the current-altitude window and keep its stroke clear of the minus sign.
* Use raw feet for alert thresholds and rounded feet only for display.
* Treat entry into +/-200 ft, silent initialization, 2 Hz flashing, and persistent yellow as project decisions, not fully specified Garmin behavior.
* Preserve the 198-second altitude cycle separately from other instruments' 18-second cycles.
* Compare SVG matrix positions with a subpixel tolerance because fractional values use single-precision matrices.

Source: accepted #24 plan and implementation, 2026-10-11.
Source: accepted #25 plan, 2026-10-11, extending the altitude cycle from 90 to 198 seconds for coherent VSI.
Evidence limit: desktop and automated results do not establish real iOS Safari or Android Chrome acceptance.
