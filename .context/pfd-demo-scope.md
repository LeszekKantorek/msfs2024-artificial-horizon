---
description: Find PFD demo scope, its historical simulator boundary, and the YAML replay follow-up.
status: active
---

| Decision | Canonical source |
| --- | --- |
| Coverage, exclusions, units, mobile acceptance responsibilities | [Project brief](../docs/project-brief.md) |
| Feature order and technical prerequisites | [Roadmap](../docs/roadmap.md) |
| Modules, layers, and instrument scope | [Panel presentation](../docs/efi/index.md), [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md) |
| Additive fields and preserved attitude-only provider | [ADR 0003](../docs/adr/0003-responsive-pfd-demo.md), [contract](../docs/telemetry-contract.md) |

* The accepted full-instrument plan supersedes the original demo-only restriction; see [SimConnect](../docs/sources/simconnect.md) and [ADR 0008](../docs/adr/0008-partial-airspeed-profile.md). Performance #7 and packaging #8 remain separate.
* Apply the scope: no battery, HSI, or navigation guidance, and responsive layout instead of fixed 4:3.
* Treat scope as planned until source and implementation evidence establish that it exists.
* Describe implemented behavior separately from device acceptance, which requires its own recorded evidence.
* Scope approval does not establish hardware acceptance.
* For heading: current magnetic heading and selected-heading bug only, excluding ground-track item 19 and a separate numeric selected-heading readout.

The locally supplied `190-01112-12_02.pdf`, section 3.3, was visually inspected during scope review: printed page 8/25 is PDF page 13.

## Scenario playback follow-up

[Issue #91](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91) plans `demo_scenario.yaml` and `--source demo --scenario <path>`; neither is implemented. Preserve all Demo values and timing in one playback path. Log conversion selects one session and preserves gaps and unavailable data. Replay stays Demo without renewing stale samples. This replaces the earlier Demo-option restriction, not the SimConnect probe exclusion.
