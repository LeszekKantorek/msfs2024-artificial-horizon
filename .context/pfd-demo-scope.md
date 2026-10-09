---
description: Plan or implement PFD demo slices without expanding live-simulator scope.
status: active
---

| Decision | Canonical source |
| --- | --- |
| Coverage, exclusions, units, mobile acceptance owner | [Project brief](../docs/project-brief.md) |
| Feature order and technical prerequisites | [Roadmap](../docs/roadmap.md) |
| Additive fields and preserved attitude-only provider | [ADR 0003](../docs/adr/0003-responsive-pfd-demo.md), [contract](../docs/telemetry-contract.md) |

* Preserve #5-#8 scope instead of adding live PFD integration.
* Apply the owner's final corrections: no battery, HSI, or navigation guidance, and responsive layout instead of fixed 4:3.
* Treat new features as planned until their issues provide implementation and acceptance evidence.
* Scope approval does not establish hardware acceptance.

Sources: owner-approved scope, 2026-10-08, recorded in [#19](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/19).
The owner-supplied `190-01112-12_02.pdf`, section 3.3, was visually inspected in that session: printed page 8/25 is PDF page 13.
