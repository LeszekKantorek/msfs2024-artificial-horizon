---
description: Plan or implement visual-object ownership, local calculations, telemetry validation, panel scheduling, or layered PFD composition.
status: active
---

Read the canonical contract for the boundary being changed:

| Boundary | Source |
| --- | --- |
| Object ownership, local calculations, helpers, telemetry validation | [Internal module seams](../docs/pfd-presentation.md#internal-module-seams) |
| Complete normalized frames, initialization, scheduling, invalidation | [Panel interface and frame lifecycle](../docs/pfd-presentation.md#panel-interface-and-frame-lifecycle) |
| Shared clips, paint order, stable nodes, state resets, accessibility | [Layers and clipping](../docs/pfd-presentation.md#layers-and-clipping), [SVG/CSS interface](../docs/pfd-presentation.md#svg-and-css-interface) |
| Composition rationale and helpers compromise | [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md), [ADR 0005](../docs/adr/0005-visual-object-composition.md) |
| Future instrument scope and dependencies | [Roadmap](../docs/roadmap.md) |

The refactor and earlier regressions establish these decision checks:

* Keep object-specific calculations with their visual owner because pure behavior and DOM-free tests do not require a separate file.
* Extract shared rules only for actual independent consumers, not a planned instrument or a test-only file boundary.
* Preserve coherent resize and fresh readings in one animation callback, including pending geometry under the unavailable cover.
* Check background clipping with browser hit testing and a narrowed-clip negative check because path geometry excludes ancestor clips.
* Distinguish intentional background overlap from collisions between painted indications.
* Keep implemented layers and composition separate from the real-device acceptance evidence in their feature issues.

Source: approved documentation and backlog plan for [#35](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/35), 2026-10-10.

Source: verified resize and clipping regressions and accepted maintenance scope in [#45](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/45), 2026-10-10.

Source: historical internal seams in [#54](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/54), reviewed on 2026-10-10, followed by visual-object composition in [#56](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/56).

Source: accepted calculation-ownership correction, 2026-10-11, and stage3b/R3b source review at `c2393379927c2cd5882ea7e0810bba56f81fd656`.

Evidence limit: stage reviews establish the reviewed source scope.
Final validation and real iOS Safari/Android Chrome acceptance remain separate.
