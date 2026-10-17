# Altitude trend

Six-second source-based altitude prediction over the left-side tape ticks.

## Requirements

Use current altitude and source VSI for the six-second prediction. Zero rate or unavailable required data hides the line.
BARO and selected-target availability do not affect a valid prediction.
The altitude trend is an application design choice, not a claim about documented Garmin behavior.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-vertical-speed).

## Presentation

`altitude/index.js` composes the stateless `trend.js` object over the tape's left-side ticks:

* The trend uses local `x = 2` inside a four CSS px clip at the tape's left edge.
* The magenta line shares the airspeed trend's color and 3 CSS px stroke. Its four-pixel clip leaves 0.5 CSS px on each side of the painted line.
* It extends from the current-altitude center toward the altitude predicted in six seconds.
* Source VSI supplies the rate without estimating browser altitude differences.
* Climb extends upward and descent downward at the tape's fixed 0.4 CSS px/ft spacing.
* The line stops one pixel inside either vertical edge when its prediction exceeds the visible tape.

The altitude composition sets the paint order:

1. Paint the tape ticks and labels.
2. Paint the magenta trend over intersecting ticks.
3. Paint the turquoise selected-altitude bug over the trend, including capture and either pinned edge.
4. Paint the opaque current-altitude window, border and digits, then the opaque VSI window and text.

Removing the outside four-pixel reservation returns that width to the horizon and heading strip.
The unchanged altitude tape width reaches the panel's right edge.
The selected header, column background and BARO use that width without an additional trend allocation.
The trend object owns its clip and receives local geometry from the altitude composition.
This placement replaces the outside-lane requirement under [#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63).

The prediction uses feet and feet per minute:

```text
predicted_altitude_ft = altitude_ft + vertical_speed_fpm / 10
```

At 10060 ft and +1500 ft/min, the endpoint represents 10210 ft, 60 CSS px above the center.
Finite rates beyond VSI's numeric display limit still produce a clipped trend.
Zero rate, missing or invalid rate, and unavailable or overflowing current altitude hide the line.
Pressure and selected-target availability do not affect it.
The common frame and original sample deadline control rendering and invalidation, with no additional timer or interpolation.

## Implementation

The stateless `altitude/trend.js` object owns prediction, its SVG path, and clip.
Altitude supplies local geometry and establishes the paint order with the tape, selected bug, and opaque readouts.
Use the original frame deadline without another timer or interpolation.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

### Altitude trend placement acceptance

[#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63) owns the corrected placement and paint order.
Use both [desktop browser scripts](../testing/pfd.md#responsive-pfd-layout-and-desktop-checks) and inspect the production page and full layout fixture.

The automated checks cover these visible results:

| Scenario | Expected result |
| --- | --- |
| 10060 ft, +1500 / -1500 ft/min | Six-second prediction at 10210 / 9910 ft, with endpoints 60 CSS px above / below center |
| Tick crossing | The magenta 3 CSS px stroke at local `x = 2` covers the tick stroke and retains 0.5 CSS px clearance inside either side of its four-pixel clip |
| Positive and negative capture, either rate sign | The turquoise strip covers the trend and the black pointer fits the notch |
| Selected target above / below the scale, +10000 / -10000 ft/min | The clipped trend stays below the whole pinned bug at either edge |
| Readout overlap | The opaque altitude window and its border cover the trend and bug |
| VSI readout | Aligned text edges, contained glyphs, unchanged window dimensions and three-pixel painted gap |
| Invalid inputs and recovery | Zero or missing required data hides the trend. BARO/target loss does not hide it |
| Resize | Tape width and pixel spacing remain unchanged. The outside four pixels return to the horizon and heading |

Paint checks use transformed points that belong to both strokes or fills, including ancestor clips.
Negative checks temporarily reverse the relevant SVG order and require the same point to reveal the wrong object.
Restore the original order before the next check.
Wait for the SVG `viewBox` to match its CSS bounds before measuring resized geometry.

Screenshot and device acceptance use this procedure:

1. Capture matching before/after scenes at 320x480 portrait, 568x240 short landscape and the 320x240 minimum.
2. Include climb, descent, capture in both directions, negative-altitude capture and both pinned edges on production and fixture pages.
3. Record the input values, viewport, tested commit and browser version with the screenshots.
4. Repeat on actual iOS Safari and Android Chrome, including rotation, browser bars and safe areas.
5. Confirm readable digits and strokes, correct layering and recovery after source or connection loss.
6. Record device, OS/browser versions, commit, observations and screenshots in #63 or its PR.

Keep the device criterion unchecked until both real-device records exist.
Desktop viewport checks do not establish phone acceptance.

### Prediction regression and screenshots

| Boundary | Checks |
| --- | --- |
| Altitude trend | Source +1500 ft/min predicts +150 ft over six seconds and extends 60 px upward. Descent, clipping, zero, missing inputs and independent pressure |

* Inspect `.local/pfd-browser/altitude-trend-*` for upward/downward paint over the left ticks and below the turquoise bug.

### VSI feature phone acceptance

For #25, apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) and these trend checks:

1. Check that the magenta trend extends up for climb, down for descent, and disappears during holds.
2. Check that the trend covers intersecting ticks and that the notched turquoise bug covers the trend.

## Rationale and regression lessons

Apply these accepted constraints:

* Predict six seconds from source VSI without estimating browser altitude differences.
* Place the trend over the left-side ticks and below the selected-altitude bug and both opaque readout windows.
* Preserve tape width and return the former four-pixel outside allocation to the horizon and heading strip.
* Use the airspeed trend's magenta color and 3 CSS px stroke. The four-pixel clip leaves 0.5 CSS px beside each painted edge.
* Keep prediction, SVG geometry and invalidation in the altitude composition without another timer.
* Hide on zero rate or loss of required inputs, using the original shared sample deadline.
* Keep valid numeric VSI available when altitude is unavailable, even though the trend is hidden.
* Treat this altitude trend as a project design choice rather than documented Garmin behavior.
* Apply the shared [paint order checks](../testing/pfd.md#paint-order-checks) to trend, bug, and readout overlap.

Source: accepted altitude trend and notched-bug amendment, 2026-10-11, with its outside-lane requirement replaced by [#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63).

Source: accepted #63 implementation plan, 2026-10-11.

Source: accepted #64 amendment, 2026-10-17, increasing altitude trend thickness from 2 to 3 CSS px to match the airspeed trend. The existing axis, clip, prediction, and layer order remain unchanged.

Evidence limit: desktop checks do not establish real iOS Safari or Android Chrome acceptance.
