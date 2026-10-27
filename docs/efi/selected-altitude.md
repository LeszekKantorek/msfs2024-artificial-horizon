# Selected altitude and alerts

Read-only selected target, notched tape bug, approach flashing, and deviation alerts.

## Requirements

Use the source target in the same indicated-altitude reference as current altitude.
Keep alert history in presentation state and use the common monotonic scheduler for timed drawing.
Accept every fresh sample into alert history before coalescing draws.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-selected-altitude).

## Presentation and alert behavior

The altitude composition adds `selected.js` in layer 2:

* The selected object owns its black value window, symbol, tape bug, and alert history.
* The readout occupies the existing 32 CSS px strip above the altitude allocation.
* The bug shares altitude's tape host and clip, using the existing 0.4 CSS px/ft scale.
* The bug stays fully inside either tape edge, including its stroke.
* The turquoise symbol is a seven-pixel vertical block with a right-side notch matching the altitude window's pointer.
* Its 28-pixel height matches that window, and the header uses the same path.
* Ordinary selected values use turquoise text.
* Deviation uses yellow text on black.
* The bug paints above the altitude trend and below both opaque altitude/VSI windows, their borders and text.
* At capture, the black pointer fits its notch while the turquoise strip remains visible on the left.
* The readout rounds finite targets to whole feet without rounding the alert inputs.
* Available selected values span -9999 through 99999 ft, with `X` for unavailable and `OVR` beyond these limits.
* Missing or overflowing targets hide the bug and reset alert history.
* Missing or overflowing current altitude retains a valid selected readout, hides the bug, and resets alerts.
* Pressure availability does not affect the selected target or alerts.

Alert behavior follows these project rules:

| Transition | Result |
| --- | --- |
| First valid pair after reset | Establish the current band without flashing. Arm deviation at error <=200 ft |
| First entry from >1000 to <=1000 ft | Flash the selected readout for five seconds |
| First entry from >200 to <=200 ft | Flash for five seconds and arm deviation |
| Departure from the armed +/-200 ft band | Flash yellow for five seconds, then retain steady yellow |
| Return to <=200 ft | Clear deviation and allow another departure alert |
| Target change or required input loss | Clear threshold history, deviation, and flash timing |

Timing and precedence remain deterministic:

* Approach alerts occur once between resets.
* Holding or oscillating around a threshold does not repeat them.
* A sample entering <=200 ft directly consumes both approach thresholds and starts only the 200-ft alert.
* A new alert replaces the previous flash interval with its own five seconds.
* Flash phases alternate 250 ms visible and 250 ms hidden, starting visible and ending visible at 5000 ms.
* Only the selected value flashes.
* The bug, current-altitude digits, and accessible description remain stable.
* Delayed renders calculate the current phase from monotonic time without replaying missed phases.
* Every fresh accepted sample advances history before drawing is coalesced.
* Resize and timed redraws preserve history and the original sample deadline.
* Invalidation clears history and cancels pending timer and frame work.

The reference is Garmin G5 Guide 190-01112-12 Rev. 2, sections 3.3.4 and 3.3.6.
Entry into +/-200 ft as capture, silent initialization, 2 Hz flashing, and persistent yellow are explicit project choices for #24.
The graphics remain original application assets.

## Implementation

`altitude/selected.js` owns the selected visual object and alert history.
Altitude composes the object; the [panel lifecycle](lifecycle.md#panel-interface-and-frame-lifecycle) delegates acceptance, rendering, and invalidation.
Only the shared scheduler owns visual timers and cancellation.

### Alert history and scheduler integration

* The selected-altitude object owns alert history and pure phase calculations.
* Rendering and resize never advance sample history.
* Acceptance resets expired retained history before processing fresh recovery data, including when expiry callbacks were delayed.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

### Selected altitude and alert acceptance

Issue [#24](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) owns selected-altitude and alert evidence.
Use the [telemetry contract](../telemetry-contract.md#optional-selected-altitude) and [presentation rules](#presentation-and-alert-behavior).

Automated coverage exercises these boundaries:

| Boundary | Checks |
| --- | --- |
| Rust model and demo | Finite signed targets, independent omission, shared freshness, old publications, two SSE clients, 198-second checkpoints and unchanged 18-second cycles |
| Alert transitions | Both approach directions, exact thresholds, unrounded values, skipped bands, sustained values, target changes, capture, departure, return and repeat departure |
| Controlled time | 250-ms flash phases, completion at 5000 ms, delayed/early callbacks, identical fresh samples, resize, coalesced samples and expiry before draw |
| Invalidation | Local missing/overflow values, paused/stale/disconnected/waiting/invalid states, suspension, timed alert cancellation under a controlled monotonic clock, stale callbacks and silent recovery |
| SVG | Matching notched symbols in header and tape, readout above bug, signed capture, edge clearance, colors, availability and stable nodes |
| Release runtime | Embedded selected module, actual Rust target in SSE/SVG, pinned bug and recovery after restart |

Use these checks and artifacts for automated acceptance:

* The [automated Rust and Node commands](../testing/automated.md#automated-checks) include these tests.
* Both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) are required for this feature.
* The selected SVG checks use real panel modules with injected monotonic time.
* They check bug paint visibility and minus-sign clearance at positive and negative capture without five-second wall-clock waits.
* Inspect screenshots at `.local/pfd-browser/selected-altitude-*` and `.local/pfd-runtime-selected-altitude.png`.

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #24, with these alert-specific checks:

1. Watch the 198-second demo for both approach directions, full flash intervals, target changes, and yellow deviation.
2. Check the pinned bug and selected value against sky and ground in portrait and short landscape, including 320 CSS px width.
3. Use [phone fixtures](../testing/devices.md#phone-fixture-setup) for target loss, malformed targets, both overflow boundaries, current-altitude loss, pressure-only loss, and return into capture.
4. Perform the shared loss and resume scenarios during an alert. Recovery requires fresh data and silent initialization.

### Full-panel timing scenarios

| Area | Scenarios and acceptance |
| --- | --- |
| Timing | Controlled clocks check five-second altitude alerts, target-change reset, freshness, and expiry before pending draw. No wall-clock sleeps |

### Full-panel geometry

Check selected-altitude bugs outside the visible tape during full-panel acceptance.

## Rationale and regression lessons

These lessons explain the lifecycle and validation constraints:

* Advance history before coalescing draws so transitions between animation frames remain observable.
* Reset expired history before accepting recovery data. The expiry callback may not have run yet.
* Keep history with the selected visual object and timing with the common scheduler to preserve one cancellation path.
* Entry into +/-200 ft, silent initialization, 2 Hz flashing, and persistent yellow are project decisions, not fully specified Garmin behavior.
* Compare SVG matrix positions with a subpixel tolerance because fractional values use single-precision matrices.

Evidence limit: desktop and automated results do not establish real iOS Safari or Android Chrome acceptance.
