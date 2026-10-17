---
description: Plan or implement visual-object ownership, local calculations, telemetry validation, panel scheduling, or layered PFD composition.
status: active
---

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
