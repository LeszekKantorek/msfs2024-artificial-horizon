---
description: Plan or implement visual-object ownership, local calculations, telemetry validation, panel scheduling, or layered PFD composition.
status: active
---

* Read [panel presentation](../docs/pfd-presentation.md) for implemented object ownership, interfaces, frame lifecycle, and layers.
* Use [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md) for instrument composition and [ADR 0005](../docs/adr/0005-visual-object-composition.md) for visual objects and the helpers compromise.
* Use [roadmap](../docs/roadmap.md) for steps 1-9 and future instrument scope.
* Read [internal module seams](../docs/pfd-presentation.md#internal-module-seams) before extending airspeed calculations or telemetry normalization.
* Keep object-specific pure calculations in their implementing visual modules and test meaningful behavior by importing those modules without DOM.
* Extract shared calculation rules only for actual independent consumers, not solely to create a pure test file.
* Keep snapshot validation independent of the telemetry client's connection lifecycle.
* Preserve the [normalized-input, initialization, and frame contracts](../docs/pfd-presentation.md#panel-interface-and-frame-lifecycle) and [SVG/CSS interface](../docs/pfd-presentation.md#svg-and-css-interface).
* Use the implemented #37 layer containers; check #36 and #37 separately for real-device acceptance evidence.
* Pass the same complete, read-only frame to each instrument and keep field selection inside that module.
* Pass concrete hosts, geometry, and required readings to children, with private nodes and real object operations.
* Keep shared hosts and clips with their immediate common parent, including the panel's shared attitude clip.
* Preserve both tape markers/leaders and fallback values inside references, and independent GS availability.
* Calculate common attitude transforms once and add no frame cache, timer, or empty render method to visual objects.
* Add future instruments in their owning feature issues instead of creating empty modules during the refactor.
* Distinguish intended background overlap from collisions between painted indication content.
* Keep resize geometry and fresh readings coherent in one animation callback; apply pending geometry under the cover after invalidation.
* Validate background clipping with browser hit testing, including a narrowed-clip negative check; path fill geometry alone does not include ancestor clips.

Source: approved documentation and backlog plan for [#35](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/35), 2026-10-10.

Source: verified resize and clipping regressions and accepted maintenance scope in [#45](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/45), 2026-10-10.

Source: historical internal seams in [#54](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/54), reviewed on 2026-10-10, followed by visual-object composition in [#56](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/56).

Source: accepted calculation-ownership correction, 2026-10-11, and stage3b/R3b source review at `c2393379927c2cd5882ea7e0810bba56f81fd656`.
Detailed current boundaries belong in the presentation reference.

Evidence limit: stage reviews establish the reviewed source scope.
Final validation and real iOS Safari/Android Chrome acceptance remain separate.
