---
description: Plan or implement instrument modules, panel scheduling, or layered PFD composition.
status: active
---

* Read [panel presentation](../docs/pfd-presentation.md) for module ownership, target interfaces, frame lifecycle, and layers.
* Use [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md) for the accepted design and [roadmap](../docs/roadmap.md) for steps 1-9.
* Read the implemented #36 module interfaces in the presentation document.
* Use the implemented #37 layer containers; check #36 and #37 separately for real-device acceptance evidence.
* Pass the same complete, read-only frame to each instrument and keep field selection inside that module.
* Add future instruments in their owning feature issues instead of creating empty modules during the refactor.
* Distinguish intended background overlap from collisions between painted indication content.

Source: approved documentation and backlog plan for [#35](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/35), 2026-10-10.
