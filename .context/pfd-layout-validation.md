---
description: Preserve responsive PFD geometry and use the full arrangement fixture when adding the next instrument.
status: active
---

| Need | Source |
| --- | --- |
| Run presentation validation | [pfd-validate-presentation](../.agents/skills/pfd-validate-presentation/SKILL.md) |
| Check responsive geometry and painted bounds | [Geometry and paint regression checks](../docs/testing/pfd.md#geometry-and-paint-regression-checks) |
| Trace bank-zero clipping evidence | [Attitude geometry provenance](../docs/pfd/attitude.md#geometry-evidence-provenance) |
| Trace hidden-label clipping and full-fixture scope | [Heading provenance](../docs/pfd/heading.md#clipping-and-scope-provenance) |

## Paint order checks

Use the [shared paint-order procedure](../docs/testing/pfd.md#paint-order-checks) for layer changes and the [altitude-trend scenarios](../docs/pfd/altitude-trend.md#altitude-trend-placement-acceptance) for #63 overlap checks.
