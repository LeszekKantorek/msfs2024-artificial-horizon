---
description: Extend or check source-supplied VSI, its compact window in the altitude composition, or the shared altitude scenario.
status: active
---

Read the canonical source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Signed ft/min, optional fields, shared freshness, and analytic scenario | [VSI telemetry](../docs/telemetry-contract.md#optional-vertical-speed) |
| Window geometry, shared styles, limits, rounding, and local availability | [VSI presentation](../docs/pfd-presentation.md#vertical-speed) |
| Automated coverage and real-phone procedures | [VSI acceptance](../docs/testing.md#vertical-speed-acceptance) |

Apply these constraints when extending the feature:

* Derive demo altitude and VSI from one segment definition, with an independent 25 ft/s to +1500 ft/min regression.
* Omit the rate for one sample at artificial jumps instead of treating the jump as flight motion.
* Use the outgoing phase rate at continuous boundaries.
* Keep VSI independent of altitude and pressure availability, without calculating differences in browser receipt time.
* Preserve the 198-second altitude/target/VSI cycle and separate 18-second cycles for other indications.
* Compose the compact VSI through altitude while keeping rate availability independent of altitude and pressure.
* Keep VSI geometry and formatting with its visual object, and pass its window bounds to the tape for obstruction handling.
* Share font family, text color, window fill and border with the main altitude readout, with VSI text one pixel smaller.
* Align VSI text to the altitude drum's measured right edge during resize, rather than centering it in the window.
* Preserve altitude's fixed pixel spacing and keep both opaque windows above the selected bug.
* Apply [altitude trend](altitude-trend.md) when using VSI for the six-second magenta prediction.
* Wait for raw accessible values in browser checks when successive samples have identical rounded text.

Source: accepted #25 plan and implementation, 2026-10-11.
Source: accepted compact-window and altitude composition amendment, 2026-10-11, replacing the scale and arrow presentation.
Source: accepted right-alignment correction, 2026-10-11.
Source: accepted one-pixel reduction in VSI window width, height and font size, 2026-10-11.
Evidence limit: desktop checks do not establish real iOS Safari or Android Chrome acceptance.
