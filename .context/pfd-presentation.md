---
description: Select PFD composition and visual validation sources.
status: active
---

## Composition

Read the canonical contract for the boundary being changed:

| Boundary | Source |
| --- | --- |
| Object ownership, local calculations, and shared helpers | [Internal module seams](../docs/efi/composition.md#visual-object-interfaces-and-ownership) |
| Normalized frame fields and read-only values | [Frame contract](../docs/efi/lifecycle.md#normalized-frame-contract) |
| Initialization, sample acceptance, scheduling, and invalidation | [Frame lifecycle](../docs/efi/lifecycle.md#initialization-and-geometry-updates), [invalidation and recovery](../docs/efi/lifecycle.md#invalidation-and-recovery) |
| Shared clips, paint order, stable nodes, state resets, accessibility | [Clipping and transforms](../docs/efi/rendering.md#clipping-and-transforms), [paint order](../docs/efi/rendering.md#cross-module-paint-order), [SVG/CSS interface](../docs/efi/rendering.md#svg-and-css-interface) |
| Composition rationale and helpers compromise | [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md), [ADR 0005](../docs/adr/0005-visual-object-composition.md) |
| Future instrument scope and dependencies | [Roadmap](../docs/roadmap.md) |
| Calculation ownership and extraction rationale | [Composition lessons](../docs/efi/composition.md#rationale-and-regression-lessons) |
| Resize coherence and clipping regressions | [Coalesced rendering](../docs/efi/lifecycle.md#coalesced-frame-rendering), [painted-bound checks](../docs/efi/rendering.md#painted-bounds-and-clipping-checks) |
| Telemetry validation, callbacks, and reentrancy | [Telemetry operations](../docs/efi/telemetry.md#implementation), [callback limits](../docs/efi/telemetry.md#callback-reentrancy-limits) |

## Visual validation

| Need | Source |
| --- | --- |
| Run presentation validation | [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) |
| Check responsive geometry and painted bounds | [Geometry and paint regression checks](../docs/testing/pfd.md#geometry-and-paint-regression-checks) |
| Trace bank-zero clipping evidence | [Attitude geometry provenance](../docs/efi/attitude.md#rationale-and-regression-lessons) |
| Check hidden-label clipping and full-fixture scope | [Heading clipping and scope](../docs/efi/heading.md#desktop-geometry-and-availability-transitions) |

## Paint order checks

Use the [shared paint-order procedure](../docs/testing/pfd.md#paint-order-checks) for layer changes and the [altitude-trend scenarios](../docs/efi/altitude-trend.md#acceptance-scenarios) for #63 overlap checks.
