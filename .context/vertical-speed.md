---
description: Locate VSI contracts and preserve analytic-scenario, text-alignment, and availability validation lessons.
status: active
---

Select the source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Signed ft/min, optional fields, freshness, and analytic scenario | [VSI telemetry](../docs/telemetry-contract.md#optional-vertical-speed) |
| Window geometry, styles, limits, rounding, and local availability | [VSI presentation](../docs/pfd-presentation.md#vertical-speed) |
| Automated coverage and real-phone procedures | [VSI acceptance](../docs/testing.md#vertical-speed-acceptance) |
| Prediction and overlap with the selected bug | [Altitude trend](altitude-trend.md) |

Retain these implementation and validation lessons:

* Derive altitude and VSI from one segment definition. An independent 25 ft/s to +1500 ft/min regression detects conversion mistakes.
* Artificial jumps omit VSI rather than representing flight motion. Continuous boundaries use the outgoing phase rate under the telemetry contract.
* VSI composition belongs to altitude, but its availability remains independent of altitude and pressure.
* Align VSI text with the altitude drum's measured right edge during resize. Centering does not preserve the accepted alignment.
* Wait for raw accessible values in browser checks when successive samples have identical rounded text.

Source: accepted #25 plan and implementation, 2026-10-11.
Source: accepted compact-window and altitude composition amendment, 2026-10-11, replacing the scale and arrow presentation.
Source: accepted right-alignment correction, 2026-10-11.
Source: accepted one-pixel reduction in VSI window width, height and font size, 2026-10-11.
Source: accepted [#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63) plan, 2026-10-11, placing the trend over ticks and below the selected bug.
Evidence limit: desktop checks do not establish real iOS Safari or Android Chrome acceptance.
