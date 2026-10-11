---
description: Extend or check source-supplied VSI, its stationary scale, or the shared altitude scenario.
status: active
---

Read the canonical source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Signed ft/min, optional fields, shared freshness, and analytic scenario | [VSI telemetry](../docs/telemetry-contract.md#optional-vertical-speed) |
| Scale, arrow, limits, rounding, and local availability | [VSI presentation](../docs/pfd-presentation.md#vertical-speed) |
| Automated coverage and real-phone procedures | [VSI acceptance](../docs/testing.md#vertical-speed-acceptance) |

Apply these constraints when extending the feature:

* Derive demo altitude and VSI from one segment definition, with an independent 25 ft/s to +1500 ft/min regression.
* Omit the rate for one sample at artificial jumps instead of treating the jump as flight motion.
* Use the outgoing phase rate at continuous boundaries.
* Keep VSI independent of altitude and pressure availability, without calculating differences in browser receipt time.
* Preserve the 198-second altitude/target/VSI cycle and separate 18-second cycles for other indications.
* Keep the VSI scale linear over +/-2000 ft/min at every tape height.
* Preserve altitude's fixed pixel spacing rather than sharing the VSI scale mapping.
* Wait for raw accessible values in browser checks when successive samples have identical rounded text.

Source: accepted #25 plan and implementation, 2026-10-11.
Evidence limit: desktop checks do not establish real iOS Safari or Android Chrome acceptance.
