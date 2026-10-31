# Vertical speed

The compact signed VSI readout below current altitude.

## Requirements

* Display source-supplied signed ft/min without estimating VSI from browser altitude differences.
* Preserve source precision and calculations when rounding the displayed value.
* Use raw values for display-limit and availability checks before rounding.
* Keep valid VSI available when altitude or BARO is unavailable.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-vertical-speed).

## Presentation

* Place the compact numeric window below current altitude.
* Align its right border with the altitude window and its value with the right edge of altitude's digits.
* Display feet per minute without a visible unit label or arrow.

### Input availability, rounding, and display limits

Round numeric VSI to 10 ft/min, with ties away from zero. Example readings include `0`, `-20`, `-120`, `+400`, and `+420`.
Never display `-0`.

| Input | Presentation |
| --- | --- |
| Raw value within -9999 to +9999 ft/min | Rounded to the nearest 10 ft/min, ties away from zero, with `+` for positive and `-` for negative readings |
| Rounded positive or negative zero | `0` |
| Raw value beyond the numeric limit | White `+OVR` or `-OVR` in the same window |
| Missing or invalid value | Red `X` in the same window |
| Missing altitude or pressure | Valid VSI remains available |

Bounds and availability use raw values before rounding. A raw value within the limit can therefore round to a displayed magnitude of 10000.

## Implementation

### Composition and visual object interface

`altitude/index.js` composes the stateless `altitude/vsi.js` object inside the altitude group:

| Component | Responsibility |
| --- | --- |
| Altitude composition | Delegate resize, rendering, and invalidation to VSI |
| VSI | Own formatting, window geometry, and local availability |
| Altitude readout | Supply the measured right edge of the two-digit monospace drum during resize |
| Altitude tape | Use VSI's local window bounds for tick and label clearance |

`createVsi(parent)` exposes these methods:

| Method | Contract |
| --- | --- |
| `resize(allocation, altitudeTextRight)` | Set local window geometry and align text with the supplied altitude digit edge |
| `render(fpm)` | Format the raw rate, set local availability, and update text and the accessible label |
| `invalidate()` | Set the unavailable state, show `X`, and set the unavailable accessible label |

### Geometry, text alignment, and paint order

| Parameter | Requirement |
| --- | --- |
| Window size | 51-by-27 CSS px |
| Position | Below the current-altitude window, with aligned right borders |
| Painted gap | Three pixels between the altitude and VSI window strokes |
| Shared style | Altitude's monospace font family, text color, background, and one-pixel border |
| Text size | 13 CSS px, one pixel smaller than altitude's 14 CSS px text |
| Tape allocation | Preserve the existing altitude tape width and keep the trend inside its left-side tick lane |
| Tape obstruction | Hide whole intersecting labels and shorten intersecting ticks with two pixels of clearance |
| Selected-altitude bug | Paint below both opaque windows and remain visible to their left |

The local window position uses altitude tape width `W` and height `H`:

```text
x = W - 1 - 51
y = H/2 + 14 + 1 + 3
```

During resize, the altitude readout supplies the right edge measured from its two-digit monospace drum.
VSI corrects text positioning for glyph bounds, including the wider unavailable `X`, so the painted text edges align.

### Frame updates, SVG state, and accessibility

* Use the common frame and expiry deadline.
* Add no timer, interpolation, filtering, or sample history.
* Include the raw value and feet per minute in the accessible label for available and overflow readings.
* Use the same VSI module in the full layout fixture and production.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

Issue [#25](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/25) owns VSI evidence.

### Acceptance scenarios

Use the [contract](../telemetry-contract.md#optional-vertical-speed) and [presentation rules](#presentation) for units, ranges, and availability.
Check the raw limit independently from the rounded text:

| Raw input in ft/min | Expected indication |
| --- | --- |
| `9999` | `+10000` |
| `-9999` | `-10000` |
| `9999.01` | White `+OVR` |
| `-9999.01` | White `-OVR` |
| Positive or negative value rounding to zero | `0`, without a minus sign |
| Missing or invalid value | Red `X` |
| Valid VSI with unavailable altitude or BARO | Retain the VSI reading |

### Automated checks and independent fixtures

Run the [automated Rust and Node commands](../testing/automated.md#automated-checks).
Automated acceptance covers these boundaries:

| Boundary | Checks |
| --- | --- |
| Rust model and SSE | Finite signed rates, zero, no telemetry limit, omission, state clearing, shared age, stale suppression, old snapshots, and matching clients |
| Scenario | Independent 3960-sample checkpoints, every jump and phase, analytic altitude/rate agreement, 25 ft/s producing +1500 ft/min, and unchanged other cycles |
| Node calculations | Signed zero, rounding, raw +/-9999 boundaries, invalid values, window geometry, recovered horizon width, and label/tick obstruction |
| Shared lifecycle | Latest coherent frame, resize without renewed freshness, expiry before drawing, pause/disconnect, reconnect and background/resume without duplicate subscriptions |

Retain the selected-altitude alert regressions, including expiry while flashing and delayed callback cancellation.

### Desktop geometry and availability transitions

Run both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks).
Check these SVG properties:

* Shared font family and color, 13-pixel VSI text, and matching right text edges.
* A 51-by-27 window with a three-pixel painted gap and contained text bounds.
* Selected target bug, label and tick clearance, retained nodes, and partial loss.

Inspect `.local/pfd-browser/vsi-*` for the minimum viewport, narrow portrait, short landscape, negative-altitude capture, and long signed values.
Also run the [altitude trend prediction checks](altitude-trend.md#automated-checks-and-desktop-geometry).

### Real-phone checks

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #25, with these VSI-specific checks:

1. Watch the 198-second altitude/VSI cycle for `0` during holds, positive climb, and negative descent directly below altitude.
2. Inspect matching readout styles, the narrower rectangle, selected target/bug, and pressure against sky and ground.
3. Use [phone fixtures](../testing/devices.md#phone-fixture-setup) with the inputs below for signed limits, partial failures, and missing data.

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
{"vertical_speed_fpm":1500,"altitude_ft":null,"baro_setting":null}
{"vertical_speed_fpm":1500,"altitude_ft":8500,"baro_setting":{"pressure_hpa":1013.25},"hold":true}
{"hold":false}
```

4. Run the [trend phone checks for #25](altitude-trend.md#real-phone-checks).

> Desktop checks do not establish real iOS Safari or Android Chrome acceptance.

### Full-panel integration

Check these release-runtime results:

* Embedded altitude/VSI/trend modules support GET/HEAD and JavaScript MIME.
* Actual Rust ft/min reaches SSE and SVG.
* Six-second climb prediction, hold, and restart recovery work through the integrated application.

Inspect `.local/pfd-runtime-vertical-speed.png` for the compiled demo's +1500 ft/min climb beside altitude and its selected target.
Check VSI limits during full-panel acceptance.

## Rationale and regression lessons

Garmin G5 Guide 190-01112-12 Rev. 2, section 3.3.9, provides the vertical-speed reference.

> The compact numeric window and its numeric limits are application design decisions.

Retain these implementation and validation lessons:

* Derive altitude and VSI from one segment definition. An independent 25 ft/s to +1500 ft/min regression detects conversion mistakes.
* Artificial jumps omit VSI rather than representing flight motion. Continuous boundaries use the outgoing phase rate under the telemetry contract.
* VSI composition belongs to altitude, but its availability remains independent of altitude and pressure.
* Align VSI text with the altitude drum's measured right edge during resize. Centering does not preserve the accepted alignment.
* Wait for raw accessible values in browser checks when successive samples have identical rounded text.
