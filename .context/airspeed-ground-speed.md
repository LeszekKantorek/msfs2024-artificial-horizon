---
description: Extend or validate IAS/GS, rolling speed digits, or the synthetic speed profile.
status: active
---

* Read the [speed contract](../docs/telemetry-contract.md#optional-speed-indications) for fields, independent validity, and compatibility.
* Read [speed presentation](../docs/pfd-presentation.md#airspeed-and-ground-speed) for tape geometry and rolling digits.
* The accepted display range is 0–999 kt, with `OVR` above 999 kt and `X` for missing or invalid data.
* The range is a presentation limit, not a telemetry limit or an aircraft speed threshold.
* Preserve the existing constructor and attitude-only publication when adding later PFD fields.
* Keep speed ranges, V-speeds, trends, and VNE behavior in #22.
* Use the real IAS/GS module in the full development fixture instead of duplicate illustrated readouts.
* Keep GS directly below the IAS tape; fixture-only labels must not reserve space in the production layout.

Source: accepted implementation plan for [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21), 2026-10-10, including the explicit 0–999 kt choice.

Source: accepted layout correction, 2026-10-10, to remove the IAS/GS gap and keep the development badge independent of production geometry.
