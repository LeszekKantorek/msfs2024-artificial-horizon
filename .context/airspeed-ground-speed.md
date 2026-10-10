---
description: Extend or validate IAS/GS, ranges, references, trend, rolling digits, or the synthetic profile.
status: active
---

* Read the [speed contract](../docs/telemetry-contract.md#optional-speed-indications) for fields, independent validity, and compatibility.
* Read [speed presentation](../docs/pfd-presentation.md#airspeed-and-ground-speed) for tape geometry and rolling digits.
* The accepted display range is 0–999 kt, with `OVR` above 999 kt and `X` for missing or invalid data.
* The range is a presentation limit, not a telemetry limit or an aircraft speed threshold.
* Preserve the existing constructor and attitude-only publication when adding later PFD fields.
* #22 extends the same module with optional profile and source acceleration. Read the [range/trend contract](../docs/telemetry-contract.md#optional-airspeed-ranges-and-trend) and [presentation rules](../docs/pfd-presentation.md#ranges-references-and-trend).
* Use the real IAS/GS module in the full development fixture instead of duplicate illustrated readouts.
* Keep GS at the panel bottom and extend the IAS background through the area below KT. That area stays empty while IAS is available.
* With missing IAS and a valid profile, configured references occupy the lower instrument. Hide the development badge to preserve their clearance.
* Keep configuration and acceleration independently optional. Profile loss removes ranges/references/VNE warnings. Acceleration loss preserves actual overspeed and valid IAS.
* Derive the six-second prediction only from source acceleration in the current frame. Irregular receipt intervals do not define acceleration.
* Check painted glyph bounds. A 10 px SVG font rendered with a 14 px bounding height in Edge 155. Fallback references need 16 px rows and tape markers need 8 px vertical clearance.
* Test accepted fractional, tiny, large, coincident, and nearby references. Compact text and displaced labels must retain exact accessible values and speed anchors.

Source: accepted implementation plan for [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21), 2026-10-10, including the explicit 0–999 kt choice.

Source: accepted layout clarification, 2026-10-10, to keep GS at the bottom and replace the IAS/GS background gap with empty tape background.

Source: accepted #22 implementation plan and measured SVG bounds on desktop Edge 155.0.4283.45, 2026-10-10. Real iOS Safari and Android Chrome acceptance remains separate.
