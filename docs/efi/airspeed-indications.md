# Airspeed ranges, references, and trend

Configured speed bands, V-speed markers, six-second prediction, and VNE warnings.

## Requirements

Use the explicit source profile and source acceleration. Missing configuration must not select a demo default.
Keep profile loss and acceleration loss independent. Predictions must not replace current IAS.
The demo profile is fictional and does not establish real-aircraft performance.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-airspeed-ranges-and-trend).

## Presentation

The same module consumes the optional profile and acceleration from the complete frame.
It creates all new SVG nodes once. Resize and render retain the existing panel lifecycle.
An unavailable profile hides ranges, references, and VNE warnings while preserving valid IAS, GS, attitude, and acceleration-based trend.
Unavailable acceleration hides the trend and predictive warning while preserving valid ranges, references, and actual overspeed.

| Range | Bounds in configured knots | Demo bounds |
| --- | --- | --- |
| Red low speed | `[0, VS0)` | 0-40 |
| White flap range | `[VS0, VFE)` | 40-85 |
| Green normal range | `[VS1, VNO)` | 50-110 |
| Yellow caution range | `[VNO, VNE)` | 110-140 |
| Red/white overspeed | `[VNE, infinity)` | From 140 |

Range and readout geometry use these rules:

* Red, green, yellow, and red/white striped ranges share one 6 CSS px lane at local `x = tape width - 18`.
* The white flap range overlays the left half of that lane at 3 CSS px width. Green remains visible on the right where both ranges apply.
* The white range paints after the other range rectangles. Each range retains its configured speed interval, including the white-only interval below VS1.
* Ticks and scale labels sit 2 CSS px farther left to clear the wider lane. The airspeed allocation is 1 CSS px wider to contain three-digit labels on the narrowest tape.
* Tick lengths, stroke widths, and label sizes remain unchanged.
* All ranges use the IAS scale's 4 CSS px/kt spacing and screen-space clip.
* The fixed window remains opaque.
* Digits use 17 px text to preserve three readable columns beside the new indications at minimum width.

Reference letters map to configured speeds:

| Letter | Reference | Synthetic demo value |
| --- | --- | --- |
| R | VR | 55 kt |
| X | VX | 65 kt |
| Y | VY | 75 kt |
| G | VG | 90 kt |

Tape references preserve their anchors and readable labels:

* Each reference sits to the right of the scale at its configured speed.
* Markers outside the visible tape stay hidden, without an edge substitute.
* An 8 px vertical inset preserves each marker's painted text and stroke at the tape boundaries.
* Coincident or nearby references retain their exact speed anchors.
* Displaced labels use leader lines and 16 px spacing inside the tape.
* Accessible reference labels retain exact configured values.
* Marker positions use those exact values.

Fallback references preserve configured values when IAS is unavailable:

* Missing or invalid IAS shows a local `X` and the configured references in four rows above GS, with the persistent `[KT]` footer label.
* Bottom reference values round to 0.1 kt.
* Positive values below 0.1 show `<0.1`.
* Values above 999 show `OVR`.
* The development badge hides while fallback references occupy its area.

Trend uses source-supplied acceleration:

* Zero IAS is valid.
* `OVR` retains its existing meaning and hides the numeric tape and trend.
* The browser calculates the six-second prediction as `IAS + 6 * ias_acceleration_ktps` from source-supplied acceleration.
* The magenta vector extends from current IAS toward that prediction.
* Its 3 CSS px stroke remains centered at local `x = tape width - 10`, leaving 0.5 CSS px between its painted edge and the range lane.
* Its stroke stays inside the tape clip.
* A negative prediction remains a calculation result and clips at the tape boundary.
* Constant speed, missing acceleration, or a non-finite prediction hides the vector.
* Missing configuration does not prevent a valid trend calculation.
* The prediction never replaces or extrapolates the current IAS reading.

VNE changes the current-speed window and digits independently:

* At or above VNE, the pointer window becomes red, including when the readout shows `OVR`.
* Below VNE, a rising prediction at or above VNE makes the actual-speed digits yellow.
* A new frame resets colors when the corresponding condition or required data is absent.
* Profile loss removes both VNE warnings.
* Acceleration loss removes only the predictive warning.

All range and trend content stays inside the IAS allocation.
The airspeed column, including TAS and GS, gains 1 CSS px. Its width is `clamp(panel width * 0.16, 50, 84) + 1`.
The central attitude region starts 1 CSS px farther right and is 1 CSS px narrower. The altitude allocation remains unchanged.

## Implementation

The [airspeed composition](airspeed.md#implementation) supplies IAS readings and child geometry.
Tape and reference interfaces are defined on this page.
The tape owns named `renderRanges` and `renderTrend` operations.
References owns markers, leaders, and fallback values, keeping each reference in one record.
IAS selects coherent readings, profile, and prediction for its children.

Calculations and constants remain with their visual objects:

| Module | Calculations and constants |
| --- | --- |
| `airspeed/tape.js` | `speedRanges`, `rangeBounds`, and local tick constants: 2 kt minor steps and 10 kt major steps |
| `airspeed/references.js` | `referenceReadout` and `placeReferenceLabels` |

Tape and references share `tapeOffset` at 4 px/kt and use `MAX_SPEED` from `airspeed/helpers.js`.
The [shared speed helpers](airspeed.md#shared-speed-helpers) section lists all consumers.

### SVG and CSS state

| State written by JavaScript | CSS or presentation effect |
| --- | --- |
| Bottom references `data-visible` | Hide the development badge while fallback references occupy its area |

### Reference and trend state

* Reference leaders precede markers so labels remain above their leaders.
* Trend updates clear the path and prediction attribute when no prediction exists.
* Reference labels retain exact configured values despite compact visible text.

### Visual object interfaces

| Object / factory | Owned content | Resize input | Render input |
| --- | --- | --- | --- |
| Tape: `createTape(defs, tapeHost)` | Barber pattern, scale/ticks/labels, named range rectangles, trend | Tape geometry | IAS reading, profile, prediction |
| References: `createReferences(tapeHost, iasHost)` | Markers/leaders and bottom group/unit/values | Tape geometry | IAS reading, profile, fallback-reference flag |

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

### Airspeed ranges, references, and trend acceptance

Run the automated checks and both browser scripts for range, reference, or trend implementation changes.
The #22 checks use the existing demo and fixture transport.
The [contract](../telemetry-contract.md#optional-airspeed-ranges-and-trend) defines the fictional profile and source acceleration.

| Area | Expected behavior |
| --- | --- |
| Profile | Five positive, strictly increasing thresholds and four named positive references at or below VNE. Invalid configuration affects only its indications |
| Ranges | Red below 40, white 40-85, green 50-110, yellow 110-140, red/white from 140 kt. White overlays the left half of green where their intervals overlap |
| Range clipping | Unbounded overspeed ranges clip to the tape. Literal expectations cover infinite bounds and preserve `NaN` coordinates without zero substitution |
| Range lane (#64) | Red, green, yellow, and striped bands share `x = tape width - 18` and 6 CSS px width. White uses the same left edge and 3 CSS px width, painted above green |
| Range paint (#64) | At IAS 65 kt, inspect 72 kt above the readout: white covers the left half and green remains visible on the right. Reversing their SVG order must fail the same paint check; restore the original order afterward |
| Trend thickness (#64) | Airspeed and altitude trends share a 3 CSS px stroke and the same magenta at their existing axes. Check positive/negative prediction and both clip boundaries |
| Scale clearance (#64) | Ticks and numbers move 2 CSS px left within the 1 px wider airspeed allocation. Original tick lengths, 1 px stroke, 12 px labels, reference speed anchors, and opaque IAS pointer remain readable |
| Boundaries | Values below, at, and above every threshold retain coherent geometry. Marker positions match configured speeds |
| Trend | Six-second prediction comes from source acceleration. Positive, negative, zero, missing, malformed, and overflowing calculations have independent expectations |
| VNE | IAS 140 kt makes the pointer red. IAS 110 with +5 kt/s predicts exactly 140 and makes digits yellow. Current readings never become predictions |
| References | R/X/Y/G appear at their scale positions. Off-tape markers hide without an edge substitute. Missing IAS shows R 55, X 65, Y 75, G 90 above GS |
| Reference edge cases | Fractional, tiny, and above-display-range values retain exact accessible values with bounded fallback text. Coincident and nearby labels stay separate with exact leader anchors |
| Partial loss | Invalid profile retains IAS/GS and a valid trend. Invalid acceleration retains profile and actual overspeed. Old snapshots retain attitude |
| Freshness | Irregular message receipt does not change source acceleration. Equal fresh samples renew validity. Expiry before painting and resize preserve the original deadline |
| Integration | Two SSE clients receive matching profile and acceleration. The release binary demonstrates a predictive warning, actual overspeed, hold with hidden trend, and restart recovery |
| Layout | Three IAS columns fit the window. Marker strokes stay within the tape. Fallback rows stay below local X and above GS at the minimum viewport |

The browser checks use independent expected positions and inspect computed colors, SVG bounds, clipping edges, and node identity.
Screenshots include `speed-approaching-vne.png`, `speed-at-vne.png`, and fallback views under `.local/pfd-browser/`.

`checkAirspeedAvailabilityTransitions` in `tests/browser/checks/airspeed-browser-checks.mjs` checks 40 loss/resize/recovery transitions through `pfd-browser.mjs`:

| Scenario | Coverage |
| --- | --- |
| Ten-step sequence | IAS loss, profile loss, combined loss with IAS recovering first, and combined loss with profile recovering first |
| Initial states | Actual overspeed and predictive warning |
| Pages | `/` and `/layout.html` |
| Resize | 844x320 to 326x246, alternating with 320x480 during recovery |

Transition assertions preserve independent availability and state resets:

* Check computed colors/visibility, state resets, exact replacement-profile ARIA, fresh fractional digits/tape offsets, and fallback/unit/badge behavior.
* GS remains independently available with nonzero painted bounds inside the panel and viewport.
* Production SVG node identity remains stable.
* The development fixture reuses the same production nodes and geometry operations.

#### Real-phone checks for #22

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #22.
Inspect ranges, references, acceleration/deceleration, yellow predictive digits, red overspeed pointer, and constant-speed trend removal.
Use [phone fixtures](../testing/devices.md#phone-fixture-setup) for the controlled cases below in both `/` and `/layout.html`.

Enter these JSON lines into the fixture terminal. The fixture initially supplies the configured profile.

```json
{"ias_kt":55,"ias_acceleration_ktps":0}
{"ias_kt":65,"ias_acceleration_ktps":0}
{"ias_kt":75,"ias_acceleration_ktps":0}
{"ias_kt":90,"ias_acceleration_ktps":0}
{"ias_kt":110,"ias_acceleration_ktps":5}
{"ias_kt":139.99,"ias_acceleration_ktps":0}
{"ias_kt":140,"ias_acceleration_ktps":0}
{"ias_kt":140.01,"ias_acceleration_ktps":-5}
{"ias_kt":null,"gs_kt":137}
{"ias_kt":123,"ias_acceleration_ktps":null}
{"ias_kt":123,"ias_acceleration_ktps":5,"airspeed_profile":null}
{"attitude_only":true}
```

Restart the fixture to restore its initial profile after the partial-loss checks.
Check immediately below and above the five thresholds with the same input mechanism.
Real-phone acceptance requires passing records from both devices. Desktop checks provide separate evidence.

#### Wider airspeed bands acceptance (#64)

Preserve matched before/after screenshots for `/` and `/layout.html` at 320x240, 320x480, and 568x240 CSS px.
Use IAS 25, 65, 95, 123, and 150 kt to cover low speed, white/green overlap, normal, caution, and striped overspeed.
Include rising and falling trends, range thresholds, readout overlap, and clipping at both tape edges.
Inspect painted bounds, labels, and V-speed references after resize settles.
Confirm that other EFI stroke widths remain unchanged.
The IAS/TAS/GS column gains 1 CSS px, the horizon loses 1 CSS px, and the altitude allocation remains unchanged.

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) on actual iOS Safari and Android Chrome.
Inspect the 6 px range lane, its 3 px white left overlay, and matching 3 px airspeed and altitude trends.
Check portrait/landscape, rotation, browser bars, safe areas, connection loss, and lock/background recovery.
Record the tested revision, device model, OS/browser versions, date, viewport, observations, and pass/fail.
Leave device acceptance unconfirmed until both records pass.

### Independent fixtures

| Fixture / suite | Coverage |
| --- | --- |
| `tests/fixtures/airspeed-profile.json` | Independent fictional thresholds and named reference configuration |
| `tests/fixtures/speed-indications.json` | Source acceleration and six-second predictions at segment boundaries |

### Full-panel geometry

Check the six-second speed trend during full-panel acceptance.

## Rationale and regression lessons

* Keep profile and acceleration loss independent because acceleration loss must preserve actual overspeed and valid IAS.
* Use source acceleration for the six-second prediction because irregular receipt intervals do not define acceleration.
* Check painted glyph bounds because a 10 px SVG font produced a 14 px bounding height in Edge 155.
* Preserve the resulting 16 px fallback rows and 8 px marker clearance when changing reference geometry.
* Preserve exact accessible values and speed anchors when compacting or displacing fractional, tiny, large, coincident, or nearby references.
* For #64, preserve the single 6 CSS px range lane and white 3 CSS px left overlay. Paint white last because green must not hide the flap range where their intervals overlap.
* Airspeed and altitude trends share a 3 CSS px stroke and magenta color on their existing axes under the accepted thickness amendment.
* The #64 scope replaces the earlier request to thicken every EFI line. Airspeed bands and both trends become thicker; ticks and labels move 2 CSS px left for clearance.
* The #64 airspeed allocation gains 1 CSS px, including TAS and GS, to contain three-digit scale labels after the 2 px shift. The horizon loses 1 CSS px and altitude keeps its allocation. Scale labels retain their 12 px font.

Source: accepted #22 implementation plan and measured SVG bounds on desktop Edge 155.0.4283.45, 2026-10-10. Real iOS Safari and Android Chrome acceptance remains separate.

Source: corrected [#64](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/64) and accepted implementation plan, 2026-10-17. The selected widths are 6/3/3 CSS px, with white on the left half. Actual iOS Safari and Android Chrome acceptance remains separate from desktop validation.

Source: accepted #64 width amendment, 2026-10-17, adding 1 CSS px to the airspeed column instead of bounding the label anchor. Edge 155 measured a negative left bound for labels 400–490 before this allocation change.

Source: accepted #64 trend-thickness amendment, 2026-10-17, increasing the altitude trend from 2 to 3 CSS px to match airspeed. This replaces the earlier altitude-width exclusion without broadening other EFI strokes.
