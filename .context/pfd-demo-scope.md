---
description: Apply the accepted PFD demo expansion and its boundaries when planning or implementing the new feature slices.
status: active
---

- Read the [project brief](../docs/project-brief.md) for canonical PFD coverage, exclusions, units and mobile acceptance ownership.
- Follow the [roadmap](../docs/roadmap.md) for feature order and technical dependencies; do not silently widen #5-#8 to live PFD integration.
- The owner's final corrections removed battery, HSI and navigation guidance and replaced the earlier 4:3 proposal with responsive phone layout.
- Treat new functionality as planned until its issue provides implementation and acceptance evidence; document/backlog approval is not hardware evidence.
- Use [ADR 0003](../docs/adr/0003-responsive-pfd-demo.md) and the [telemetry contract](../docs/telemetry-contract.md) to preserve the attitude-only provider path during additive demo work.

## Sources

- Owner-approved plan and corrections in the 2026-10-08 scope-planning session, implemented as scope/backlog work in [#19](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/19).
- Garmin reference `190-01112-12_02.pdf`, section 3.3: printed Page 8 of 25 is physical PDF page 13; supplied locally by the owner and visually inspected in that session.
