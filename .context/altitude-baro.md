---
description: Extend or validate altitude, barometric pressure, signed rolling drums, or the right-side PFD layout.
status: active
---

Read the canonical contract for the affected behavior:

| Contract | Source |
| --- | --- |
| Fields, independent validity, pressure meaning, and demo sweeps | [Altitude telemetry](../docs/telemetry-contract.md#optional-altitude-and-pressure-setting) |
| Tape, drums, display limits, opacity, and unavailable states | [Altitude presentation](../docs/pfd-presentation.md#altitude-and-barometric-setting) |
| Complete accepted requirements and delivery evidence | [#23 handoff](../docs/plans/23-altitude-baro/index.md) |

* Use the accepted -9999 to 99999 ft display range and 20-ft lower drum when extending altitude.
* Keep finite out-of-display-range values valid in telemetry.
* Keep altitude and pressure availability independent and preserve their shared sample deadline.
* Keep the separate altitude tape because its responsive span, signs, and drums differ from IAS.
* Check actual glyph bounds when changing the pressure readout because a 24-pixel background caused a 2-pixel text collision in Edge.
* The corrected pressure background is 32 pixels high; retain the gap between its value and unit.
* Replace future illustrations with real modules in the full layout fixture as each instrument arrives.

Source: accepted #23 implementation plan and selected range/drums, 2026-10-11.
Source: Edge 155.0.4283.45 geometry checks on 2026-10-11; real iOS Safari and Android Chrome acceptance remains separate.
