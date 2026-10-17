---
description: Preserve responsive PFD geometry and use the full arrangement fixture when adding the next instrument.
status: active
---

| Need | Source |
| --- | --- |
| Run presentation validation | [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) |
| Check responsive geometry and painted bounds | [Geometry and paint regression checks](../docs/testing/pfd.md#geometry-and-paint-regression-checks) |
| Trace bank-zero clipping evidence | [Attitude geometry provenance](../docs/efi/attitude.md#geometry-evidence-provenance) |
| Trace hidden-label clipping and full-fixture scope | [Heading provenance](../docs/efi/heading.md#clipping-and-scope-provenance) |

## Paint order checks

Use the [shared paint-order procedure](../docs/testing/pfd.md#paint-order-checks) for layer changes and the [altitude-trend scenarios](../docs/efi/altitude-trend.md#altitude-trend-placement-acceptance) for #63 overlap checks.
