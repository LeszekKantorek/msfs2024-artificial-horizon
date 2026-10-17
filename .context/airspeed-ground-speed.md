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
* For #64, preserve the single 6 CSS px range lane and white 3 CSS px left overlay. Paint white last because green must not hide the flap range where their intervals overlap.
* Airspeed and altitude trends share a 3 CSS px stroke and magenta color on their existing axes under the accepted thickness amendment.
* The #64 scope replaces the earlier request to thicken every PFD line. Airspeed bands and both trends become thicker; ticks and labels move 2 CSS px left for clearance.
* The #64 airspeed allocation gains 1 CSS px, including TAS and GS, to contain three-digit scale labels after the 2 px shift. The horizon loses 1 CSS px and altitude keeps its allocation. Scale labels retain their 12 px font.

Source: accepted implementation plan for [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21), 2026-10-10, including the explicit 0–999 kt choice.

Source: accepted layout clarification, 2026-10-10, to keep GS at the bottom and replace the IAS/GS background gap with empty tape background.

Source: accepted #22 implementation plan and measured SVG bounds on desktop Edge 155.0.4283.45, 2026-10-10. Real iOS Safari and Android Chrome acceptance remains separate.

Source: accepted #65 plan and Edge glyph measurements, 2026-10-11, preserving 16 px fallback row spacing while clearing `[KT]`.

Source: accepted #65 correction, 2026-10-11; Edge 155 measures 1 CSS px above and below both labels at 10 CSS px centered at panel height minus 40 CSS px.

Source: corrected [#64](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/64) and accepted implementation plan, 2026-10-17. The selected widths are 6/3/3 CSS px, with white on the left half. Actual iOS Safari and Android Chrome acceptance remains separate from desktop validation.

Source: accepted #64 width amendment, 2026-10-17, adding 1 CSS px to the airspeed column instead of bounding the label anchor. Edge 155 measured a negative left bound for labels 400–490 before this allocation change.

Source: accepted #64 trend-thickness amendment, 2026-10-17, increasing the altitude trend from 2 to 3 CSS px to match airspeed. This replaces the earlier altitude-width exclusion without broadening other PFD strokes.
