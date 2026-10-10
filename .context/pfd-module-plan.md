---
description: Plan or implement instrument modules, internal calculation and validation seams, panel scheduling, or layered PFD composition.
status: active
---

* Read [panel presentation](../docs/pfd-presentation.md) for module ownership, target interfaces, frame lifecycle, and layers.
* Use [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md) for the accepted design and [roadmap](../docs/roadmap.md) for steps 1-9.
* Read the implemented #36 module interfaces in the presentation document.
* Read [internal module seams](../docs/pfd-presentation.md#internal-module-seams) before extending airspeed calculations or telemetry normalization.
* Keep pure calculations and validation separate from SVG operations and connection lifecycle, as implemented in maintenance #54.
* Preserve the [normalized-input, initialization, and frame contracts](../docs/pfd-presentation.md#panel-interface-and-frame-lifecycle) and [SVG/CSS interface](../docs/pfd-presentation.md#svg-and-css-interface).
* Use the implemented #37 layer containers; check #36 and #37 separately for real-device acceptance evidence.
* Pass the same complete, read-only frame to each instrument and keep field selection inside that module.
* Add future instruments in their owning feature issues instead of creating empty modules during the refactor.
* Distinguish intended background overlap from collisions between painted indication content.
* Keep resize geometry and fresh readings coherent in one animation callback; apply pending geometry under the cover after invalidation.
* Validate background clipping with browser hit testing, including a narrowed-clip negative check; path fill geometry alone does not include ancestor clips.

Source: approved documentation and backlog plan for [#35](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/35), 2026-10-10.

Source: verified resize and clipping regressions and accepted maintenance scope in [#45](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/45), 2026-10-10.

Source: implemented internal seams in [#54](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/54), reviewed steps 1–4 on 2026-10-10. Device acceptance remains separate.
