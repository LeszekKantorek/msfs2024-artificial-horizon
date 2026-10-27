---
description: Select PFD composition and visual validation sources.
status: active
---

## Composition

Read the canonical contract for the boundary being changed:

| Boundary | Source |
| --- | --- |
| Object ownership, local calculations, and shared helpers | [Internal module seams](../docs/efi/composition.md#internal-module-seams) |
| Complete normalized frames, initialization, scheduling, invalidation | [Panel interface and frame lifecycle](../docs/efi/lifecycle.md#panel-interface-and-frame-lifecycle) |
| Shared clips, paint order, stable nodes, state resets, accessibility | [Layers and clipping](../docs/efi/rendering.md#layers-and-clipping), [SVG/CSS interface](../docs/efi/rendering.md#svg-and-css-interface) |
| Composition rationale and helpers compromise | [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md), [ADR 0005](../docs/adr/0005-visual-object-composition.md) |
| Future instrument scope and dependencies | [Roadmap](../docs/roadmap.md) |
| Refactor rationale, resize/clipping regressions, and evidence limits | [Composition lessons](../docs/efi/composition.md#rationale-and-regression-lessons) |
| Telemetry validation, callbacks, and reentrancy | [Telemetry operations](../docs/efi/telemetry.md#telemetry-operations), [callback limits](../docs/efi/telemetry.md#callback-reentrancy-limit) |

## Visual validation

| Need | Source |
| --- | --- |
| Run presentation validation | [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) |
| Check responsive geometry and painted bounds | [Geometry and paint regression checks](../docs/testing/pfd.md#geometry-and-paint-regression-checks) |
| Trace bank-zero clipping evidence | [Attitude geometry provenance](../docs/efi/attitude.md#geometry-evidence-provenance) |
| Check hidden-label clipping and full-fixture scope | [Heading clipping and scope](../docs/efi/heading.md#clipping-and-scope) |

## Paint order checks

Use the [shared paint-order procedure](../docs/testing/pfd.md#paint-order-checks) for layer changes and the [altitude-trend scenarios](../docs/efi/altitude-trend.md#altitude-trend-placement-acceptance) for #63 overlap checks.
