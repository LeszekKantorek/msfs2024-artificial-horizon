# Vertical speed

The compact signed VSI readout below current altitude.

## Requirements

Round numeric VSI to 10 ft/min, with ties away from zero: `0`, `-20`, `-120`, `+400`, `+420`; never `-0`. Preserve source precision and calculations.

Display source-supplied signed ft/min. Valid VSI remains available when altitude or BARO is unavailable.
Use raw values for display-limit checks before rounding; do not estimate VSI from browser altitude differences.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-vertical-speed).

## Presentation

`altitude/index.js` composes the stateless `vsi.js` object inside the altitude group:

* A 51-by-27 CSS px rectangle sits below the current-altitude window, with aligned right borders and three pixels between painted strokes.
* The window shares altitude's monospace font family, text color, background and one-pixel border.
* VSI uses 13 CSS px text, one pixel smaller than altitude's 14 CSS px text.
* The value aligns with the right edge of altitude's digits, using feet per minute without a visible unit label or arrow.
* The altitude readout supplies that edge from the measured bounds of its two-digit monospace drum during resize.
* VSI accounts for glyph bounds when positioning text, including the wider unavailable `X`.
* Altitude keeps its existing tape width and places the trend inside its left-side tick lane.
* VSI owns its formatting, window geometry and availability, with altitude delegating resize, rendering and invalidation.
* The tape uses VSI's local window bounds to hide whole intersecting labels and shorten intersecting ticks with two pixels of clearance.
* The selected-altitude bug paints below both opaque windows and remains visible to their left.

The local window position uses altitude tape width `W` and height `H`:

```text
x = W - 1 - 51
y = H/2 + 14 + 1 + 3
```

Reading and availability follow these rules:

| Input | Presentation |
| --- | --- |
| Raw value within -9999 to +9999 ft/min | Rounded to the nearest 10 ft/min, ties away from zero, with `+` for positive and `-` for negative readings |
| Rounded positive or negative zero | `0` |
| Raw value beyond the numeric limit | White `+OVR` or `-OVR` in the same window |
| Missing or invalid value | Red `X` in the same window |
| Missing altitude or pressure | Valid VSI remains available |

Bounds and availability use raw values before rounding.
The accessible label includes the raw value and feet per minute.
VSI uses the common frame and expiry deadline without a timer, interpolation, filtering, or sample history.
The full layout fixture uses the same module as production.

Garmin G5 Guide 190-01112-12 Rev. 2, section 3.3.9, provides the vertical-speed reference.
The compact numeric window and its numeric limits are application design decisions.

## Implementation

`altitude/index.js` composes the stateless `altitude/vsi.js` object.
VSI owns formatting, window geometry, and local availability. Altitude delegates resize, rendering, and invalidation.
The readout supplies measured digit bounds during resize; the tape uses the VSI window bounds for tick and label clearance.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

### Vertical-speed acceptance

Issue [#25](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/25) owns VSI evidence.
Use the [contract](../telemetry-contract.md#optional-vertical-speed) and [presentation rules](#presentation) for units, ranges, and availability.

Automated acceptance covers these boundaries:

| Boundary | Checks |
| --- | --- |
| Rust model and SSE | Finite signed rates, zero, no telemetry limit, omission, state clearing, shared age, stale suppression, old snapshots, and matching clients |
| Scenario | Independent 3960-sample checkpoints, every jump and phase, analytic altitude/rate agreement, 25 ft/s producing +1500 ft/min, and unchanged other cycles |
| Node calculations | Signed zero, rounding, raw +/-9999 boundaries, invalid values, window geometry, recovered horizon width, and label/tick obstruction |
| Shared lifecycle | Latest coherent frame, resize without renewed freshness, expiry before drawing, pause/disconnect, reconnect and background/resume without duplicate subscriptions |
| SVG | Shared font family/color, 13-pixel VSI text, matching right text edges and 51-by-27 window. Three-pixel painted gap, text bounds, target bug, label/tick clearance, nodes and partial loss |
| Release runtime | Embedded altitude/VSI/trend modules with GET/HEAD and JavaScript MIME, actual Rust ft/min in SSE/SVG, six-second climb prediction, hold and restart recovery |

Use these checks and artifacts:

* Run the [automated Rust and Node commands](../testing/automated.md#automated-checks) and both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks).
* Retain the selected-altitude alert regressions, including expiry while flashing and delayed callback cancellation.
* Inspect `.local/pfd-browser/vsi-*` for the minimum viewport, narrow portrait, short landscape, negative-altitude capture and long signed values.
* Inspect `.local/pfd-runtime-vertical-speed.png` for the compiled demo's +1500 ft/min climb beside altitude and its selected target.

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #25, with these VSI-specific checks:

1. Watch the 198-second altitude/VSI cycle for `0` during holds, positive climb, and negative descent directly below altitude.
2. Inspect matching readout styles, the narrower rectangle, selected target/bug, and pressure against sky and ground.
3. Use [phone fixtures](../testing/devices.md#phone-fixture-setup) with the inputs below for signed limits, partial failures, and missing data.

Also run the [altitude trend prediction checks](altitude-trend.md#prediction-regression-and-screenshots) and [trend phone checks for #25](altitude-trend.md#vsi-feature-phone-acceptance).

Enter these JSON lines into the fixture terminal:

```json
{"vertical_speed_fpm":1500}
{"vertical_speed_fpm":-1500}
{"vertical_speed_fpm":0}
{"vertical_speed_fpm":2000.01}
{"vertical_speed_fpm":-2000.01}
{"vertical_speed_fpm":9999}
{"vertical_speed_fpm":-9999}
{"vertical_speed_fpm":-9999,"altitude_ft":-60,"selected_altitude_ft":-100}
{"vertical_speed_fpm":9999.01}
{"vertical_speed_fpm":-9999.01}
{"vertical_speed_fpm":null}
{"vertical_speed_fpm":1500,"altitude_ft":null,"baro_text":null}
{"vertical_speed_fpm":1500,"altitude_ft":8500,"baro_text":"29.92 inHg","hold":true}
{"hold":false}
```

### Full-panel geometry

Check VSI limits during full-panel acceptance.

## Rationale and regression lessons

Retain these implementation and validation lessons:

* Derive altitude and VSI from one segment definition. An independent 25 ft/s to +1500 ft/min regression detects conversion mistakes.
* Artificial jumps omit VSI rather than representing flight motion. Continuous boundaries use the outgoing phase rate under the telemetry contract.
* VSI composition belongs to altitude, but its availability remains independent of altitude and pressure.
* Align VSI text with the altitude drum's measured right edge during resize. Centering does not preserve the accepted alignment.
* Wait for raw accessible values in browser checks when successive samples have identical rounded text.

Evidence limit: desktop checks do not establish real iOS Safari or Android Chrome acceptance.
