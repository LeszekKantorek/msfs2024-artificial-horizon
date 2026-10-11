---
description: Extend or validate IAS/GS, ranges, references, trend, rolling digits, or the synthetic profile.
status: active
---

Use the canonical sources for the contract being extended:

| Contract | Source |
| --- | --- |
| Speed fields, independent validity, attitude-only compatibility | [Speed contract](../docs/telemetry-contract.md#optional-speed-indications) |
| Optional profile and source acceleration | [Range/trend contract](../docs/telemetry-contract.md#optional-airspeed-ranges-and-trend) |
| Tape, rolling digits, GS placement, unavailable/overflow states | [Speed presentation](../docs/pfd-presentation.md#airspeed-and-ground-speed) |
| Reference placement, fallback values, colors, prediction | [Ranges, references, and trend](../docs/pfd-presentation.md#ranges-references-and-trend) |

Keep these pitfalls in scope when extending the instrument:

* The accepted 0–999 kt range is a presentation limit, not a telemetry limit or an aircraft speed threshold.
* Keep profile and acceleration loss independent because acceleration loss must preserve actual overspeed and valid IAS.
* Use source acceleration for the six-second prediction because irregular receipt intervals do not define acceleration.
* Check painted glyph bounds because a 10 px SVG font produced a 14 px bounding height in Edge 155.
* Preserve the resulting 16 px fallback rows and 8 px marker clearance when changing reference geometry.
* Preserve exact accessible values and speed anchors when compacting or displacing fractional, tiny, large, coincident, or nearby references.
* Exercise the real IAS/GS module in the full fixture, including fallback references that occupy the development badge area.
* Measure fallback reference glyphs against both the IAS warning and persistent `[KT]` when changing footer positions.
* Keep unit-label margins primary when choosing font size, and measure their clearance from the tape clip and footer.
* Reuse the speed-trend magenta token for the GS border and the opaque BARO background for both footers.

Source: accepted implementation plan for [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21), 2026-10-10, including the explicit 0–999 kt choice.

Source: accepted layout clarification, 2026-10-10, to keep GS at the bottom and replace the IAS/GS background gap with empty tape background.

Source: accepted #22 implementation plan and measured SVG bounds on desktop Edge 155.0.4283.45, 2026-10-10. Real iOS Safari and Android Chrome acceptance remains separate.

Source: accepted #65 plan and Edge glyph measurements, 2026-10-11, preserving 16 px fallback row spacing while clearing `[KT]`.

Source: accepted #65 correction, 2026-10-11; Edge 155 measures 1 CSS px above and below both labels at 10 CSS px centered at panel height minus 40 CSS px.
