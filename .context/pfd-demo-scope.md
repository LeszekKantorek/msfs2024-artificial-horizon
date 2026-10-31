---
description: Find EFI scope, provider boundaries, scenario playback requirements, and validation guidance.
status: active
---

| Decision | Canonical source |
| --- | --- |
| Coverage, exclusions, and units | [EFI scope](../docs/efi/index.md#product-scope-and-reference) |
| Mobile layout and acceptance responsibilities | [Rendering](../docs/efi/rendering.md#layout-and-readable-regions), [feature acceptance](../docs/testing/pfd.md#feature-acceptance-ownership) |
| Feature order and technical prerequisites | [Roadmap](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) |
| Modules, layers, and instrument scope | [Panel presentation](../docs/efi/index.md), [ADR 0004](../docs/adr/0004-modular-pfd-presentation.md) |
| Optional fields and current attitude-only publication | [ADR 0003](../docs/adr/0003-responsive-pfd-demo.md), [contract](../docs/telemetry-contract.md) |
| Planned YAML playback and recorded-session conversion | [Demo playback requirements](../docs/sources/demo.md#planned-scenario-playback) |
| Performance measurements and evidence limits | [Measurements](../docs/testing/evidence.md#measurements-and-evidence), [device acceptance](../docs/testing/devices.md#real-phone-acceptance-procedure) |
| SDK distribution requirements | [SimConnect requirements](../docs/sources/simconnect.md#requirements) |

* The instrument scope includes Demo and available SimConnect data. See [SimConnect](../docs/sources/simconnect.md) and [ADR 0008](../docs/adr/0008-partial-airspeed-profile.md).
* Apply the scope: no battery, HSI, or navigation guidance, and responsive layout instead of fixed 4:3.
* Treat scope as planned until source and implementation evidence establish that it exists.
* Describe implemented behavior separately from device acceptance, which requires its own recorded evidence.
* Scope approval does not establish hardware acceptance.
* Demo results do not establish real-phone performance or simulator acceptance.
* A successful SimConnect run does not establish DLL redistribution rights.
* For heading: current magnetic heading and selected-heading bug only, excluding ground-track item 19 and a separate numeric selected-heading readout.

The locally supplied `190-01112-12_02.pdf`, section 3.3, was visually inspected during scope review: printed page 8/25 is PDF page 13.
