# Altitude trend

Six-second source-based altitude prediction over the left-side tape ticks.

## Requirements

* Use current altitude and source VSI for the six-second prediction without estimating browser altitude differences.
* Hide the line when the rate is zero or required data is unavailable.
* Keep valid predictions independent of BARO and selected-target availability.

> The altitude trend is an application design choice, not a claim about documented Garmin behavior.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-vertical-speed).

## Presentation

### Input availability and partial loss

Required inputs and independent indications follow these rules:

| Input condition | Result |
| --- | --- |
| Available current altitude and finite nonzero source VSI | Display the six-second prediction |
| Finite rate beyond VSI's numeric display limit | Retain the trend and clip it to the visible tape |
| Zero, missing, or invalid rate | Hide the trend |
| Unavailable or overflowing current altitude | Hide the trend. Keep valid numeric VSI available |
| Unavailable BARO or selected target | Preserve a valid trend |

### Six-second prediction

The prediction uses feet and feet per minute:

```text
predicted_altitude_ft = altitude_ft + vertical_speed_fpm / 10
```

The line extends from the current-altitude center toward the predicted altitude. Climb extends upward and descent downward.
At 10060 ft and +1500 ft/min, the prediction adds 150 ft. The endpoint represents 10210 ft, 60 CSS px above center.

## Implementation

### Composition and visual object interface

| Module | Responsibility |
| --- | --- |
| `altitude/index.js` | Compose the stateless trend object, supply local geometry, and establish paint order with the tape, selected bug, and opaque readouts |
| `altitude/trend.js` | Own prediction, SVG path, clip, and local invalidation within the altitude composition |

`createAltitudeTrend(defs, parent)` creates the visual object:

| Method | Contract |
| --- | --- |
| `resize(lane)` | Store local geometry and update the clip from `lane.x`, `lane.width`, and `lane.height` |
| `render(altitude, fpm)` | Calculate the prediction and update the path, or invalidate unavailable results |
| `invalidate()` | Hide the line and remove `d` and `data-prediction-ft` |

### Geometry, clipping, and paint order

The trend overlays the tape's left-side ticks:

| Parameter | Requirement |
| --- | --- |
| Horizontal position | Local `x = 2` at the tape's left edge |
| Clip width | 4 CSS px |
| Line style | Airspeed trend's magenta color and 3 CSS px stroke |
| Horizontal clearance | 0.5 CSS px on each side of the painted line inside the clip |
| Tape spacing | Fixed 0.4 CSS px/ft |
| Vertical clipping | Stop one pixel inside either vertical edge when the prediction exceeds the visible tape |
| Tape allocation | Preserve the altitude tape width and extend it to the panel's right edge |
| Shared column width | Selected header, column background, and BARO use the tape width without an additional trend allocation |

The altitude composition sets the paint order:

1. Paint the tape ticks and labels.
2. Paint the magenta trend over intersecting ticks.
3. Paint the turquoise selected-altitude bug over the trend, including capture and either pinned edge.
4. Paint the opaque current-altitude window, border, and digits, then the opaque VSI window and text.

### Frame updates and invalidation

* Use the common frame and original sample deadline for rendering and invalidation.
* Add no timer or interpolation.
* Hide and clear the trend when required inputs become unavailable or the rate becomes zero.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

The [geometry and paint-order contract](#geometry-clipping-and-paint-order) defines placement and overlap.

### Acceptance scenarios

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
| Resize | Preserve tape width and pixel spacing. Keep the trend inside the altitude allocation without a separate outside lane |

### Automated checks and desktop geometry

Use both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) and inspect the production page and full layout fixture.
Apply the shared [paint order checks](../testing/pfd.md#paint-order-checks) to trend, bug, and readout overlap.

Paint checks use transformed points that belong to both strokes or fills, including ancestor clips.
Verify that the negative checks detect incorrect layering:

1. Temporarily reverse the relevant SVG order.
2. Require the same point to reveal the wrong object.
3. Restore the original order before the next check.

Wait for the SVG `viewBox` to match its CSS bounds before measuring resized geometry.
Check trend placement in both compositions:

1. Capture current scenes at 320x480 portrait, 568x240 short landscape, and the 320x240 minimum.
2. Include climb, descent, capture in both directions, negative-altitude capture, and both pinned edges on production and fixture pages.
3. Record the input values, viewport, tested commit, and browser version with the screenshots.
4. Inspect `.local/pfd-browser/altitude-trend-*` for upward/downward paint over the left ticks and below the turquoise bug.

### Real-phone checks

Repeat the placement scenarios on actual iOS Safari and Android Chrome:

1. Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure).
2. Repeat the [desktop scenes](#automated-checks-and-desktop-geometry), including rotation, browser bars, and safe areas.
3. Confirm readable digits and strokes, correct layering, and recovery after source or connection loss.
4. Record device, OS/browser versions, commit, observations, and screenshots with the acceptance results.

Include these trend checks in the shared phone procedure:

* Check that the magenta trend extends up for climb, down for descent, and disappears during holds.
* Check that the trend covers intersecting ticks and that the notched turquoise bug covers the trend.

> Keep missing device evidence explicit until both real-device records exist. Desktop viewport checks do not establish iOS Safari or Android Chrome acceptance.

## Rationale and regression lessons

* The trend overlays the altitude tape's left-side ticks without an additional width allocation.
* Preserve the altitude tape, attitude, and heading allocations.

* Keep prediction based on source VSI. Browser altitude differences do not supply the rate.
* Preserve independent availability: missing altitude hides the trend but does not invalidate a valid numeric VSI indication.
* Check trend, bug, and opaque readout overlap at capture and both pinned edges. A visible trend alone does not establish correct layering.
