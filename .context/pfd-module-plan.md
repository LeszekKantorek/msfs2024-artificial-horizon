---
description: Plan or implement instrument modules, panel scheduling, or layered PFD composition.
status: active
---

* Read [panel presentation](../docs/pfd-presentation.md) for module ownership, target interfaces, frame lifecycle, and layers.
* Use [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md) for the accepted design and [roadmap](../docs/roadmap.md) for steps 1-9.
* Treat #36 and #37 as planned until their implementation and acceptance evidence exists.
* Pass the same complete, read-only frame to each instrument and keep field selection inside that module.
* Add future instruments in their owning feature issues instead of creating empty modules during the refactor.
* Distinguish intended background overlap from collisions between painted indication content.

Source: approved documentation and backlog plan for [#35](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/35), 2026-10-10.
