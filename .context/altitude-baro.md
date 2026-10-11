---
description: Extend or validate altitude, barometric pressure, signed rolling drums, or the right-side PFD layout.
status: active
---

Read the canonical contract for the affected behavior:

| Contract | Source |
| --- | --- |
| Fields, independent validity, pressure meaning, and demo sweeps | [Altitude telemetry](../docs/telemetry-contract.md#optional-altitude-and-pressure-setting) |
| Tape, drums, display limits, opacity, and unavailable states | [Altitude presentation](../docs/pfd-presentation.md#altitude-and-barometric-setting) |
| Task scope and public acceptance evidence | [Issue #23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23), [PR #59](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/pull/59) |

* Use the accepted -9999 to 99999 ft display range and 20-ft lower drum when extending altitude.
* Keep finite out-of-display-range values valid in telemetry.
* Keep altitude and pressure availability independent and preserve their shared sample deadline.
* Keep the separate altitude tape because its responsive span, signs, and drums differ from IAS.
* Validate actual glyph bounds when changing the pressure readout because a 24-pixel background caused a 2-pixel text collision in Edge.
* Keep the corrected pressure background 32 pixels high.
* Retain the gap between the pressure value and its unit.
* Replace future illustrations with real modules in the full layout fixture as each instrument arrives.

Source: accepted #23 implementation plan and selected range/drums, 2026-10-11.
Source: Edge 155.0.4283.45 geometry validation on 2026-10-11.
Evidence limit: real iOS Safari and Android Chrome acceptance remains separate.
