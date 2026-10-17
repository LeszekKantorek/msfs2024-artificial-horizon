---
description: Locate altitude and BARO contracts, and preserve lessons from tape and glyph validation.
status: active
---

Select the source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Fields, independent validity, pressure meaning, and demo sweeps | [Altitude telemetry](../docs/telemetry-contract.md#optional-altitude-and-pressure-setting) |
| Tape spacing, drums, display limits, BARO formatting, and availability | [Altitude presentation](../docs/pfd-presentation.md#altitude-and-barometric-setting) |
| Automated and device checks | [Altitude acceptance](../docs/testing.md#altitude-and-pressure-acceptance), [footer and BARO checks](../docs/testing.md#tape-footers-and-source-formatted-baro) |
| Public evidence and accepted scope | [Issue #23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23), [PR #59](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/pull/59), [footer revision #65](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/65) |

Retain these reasons and validation lessons when changing the presentation:

* Altitude has a separate tape because its responsive span, signed values, and 20-ft drums differ from IAS.
* The accepted resize correction keeps landscape pixel spacing and reveals more altitude on taller tapes. Do not restore a fixed 400-ft span.
* A 24-pixel pressure background caused a 2-pixel text collision in Edge. The corrected height is 32 pixels.
* Check actual pressure glyph bounds and the centered text anchor separately. Font side bearings can produce a subpixel visual offset.
* The #65 BARO replacement deliberately ships server and embedded Web together. Its [versioning exception](../docs/telemetry-contract.md#compatibility) is limited to that replacement.

Source: accepted #23 implementation plan and selected range/drums, 2026-10-11.
Source: Edge 155.0.4283.45 geometry validation on 2026-10-11.
Source: accepted portrait/landscape correction for #23, 2026-10-11, to keep landscape pixel spacing and show more altitude on taller tapes.
Evidence limit: real iOS Safari and Android Chrome acceptance remains separate.

Source: accepted #65 implementation plan, 2026-10-11, superseding the separate pressure value and unit layout.
