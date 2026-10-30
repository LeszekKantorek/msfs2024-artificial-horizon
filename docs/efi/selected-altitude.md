# Selected altitude and alerts

Read-only selected target, notched tape bug, approach flashing, and deviation alerts.

## Requirements

* Use the source target in the same indicated-altitude reference as current altitude.
* Keep alert history in presentation state and use the common monotonic scheduler for timed drawing.
* Accept every fresh sample into alert history before coalescing draws.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-selected-altitude).

## Presentation

### Input availability and display limits

| Input condition | Selected readout | Tape bug | Alert history |
| --- | --- | --- | --- |
| Available target from -9999 through 99999 ft and available current altitude | Round the target to whole feet | Show at the target position, pinned inside either tape edge when needed | Process unrounded inputs |
| Unavailable target | `X` | Hide | Reset |
| Overflowing target outside the display limits | `OVR` | Hide | Reset |
| Missing or overflowing current altitude, with a valid target | Retain the valid selected readout | Hide | Reset |
| Pressure unavailable | Unaffected | Unaffected | Unaffected |

### Selected readout and tape bug

* Round finite selected targets to whole feet without rounding alert inputs.
* Use turquoise text for ordinary selected values and yellow text on black for deviation.
* Keep the selected readout above the altitude tape.
* At capture, fit the black current-altitude pointer into the bug's notch while retaining the turquoise strip on the left.

### Alert transitions

Alert error is the absolute difference between current altitude and selected altitude, using unrounded feet.
These transitions apply in both approach directions:

| Transition | Result |
| --- | --- |
| First valid pair after reset | Establish the current band without flashing. Arm deviation at error <=200 ft |
| First entry from >1000 to <=1000 ft | Flash the selected readout for five seconds |
| First entry from >200 to <=200 ft | Flash for five seconds and arm deviation |
| Departure from the armed +/-200 ft band | Flash yellow for five seconds, then retain steady yellow |
| Return to <=200 ft | Clear deviation and allow another departure alert |
| Target change or required input loss | Clear threshold history, deviation, and flash timing |

### Flash timing and precedence

| Timing parameter | Rule |
| --- | --- |
| Flash duration | Five seconds for each new alert |
| Phases | Alternate 250 ms visible and 250 ms hidden, starting visible |
| Completion | End visible at 5000 ms |
| Flashing content | Only the selected value |
| Stable content during flashing | Tape bug, current-altitude digits, and accessible description |

* Approach alerts occur once between resets.
* Holding or oscillating around a threshold does not repeat them.
* A sample entering <=200 ft directly consumes both approach thresholds and starts only the 200-ft alert.
* A new alert replaces the previous flash interval with its own five seconds.

## Implementation

### Composition and visual object interface

| Component | Responsibility |
| --- | --- |
| `altitude/selected.js` | Own the black value window, symbol, tape bug, alert history, and pure phase calculations |
| Altitude composition | Compose `selected.js` in layer 2 |
| Panel lifecycle | Delegate sample acceptance, rendering, and invalidation |
| Shared scheduler | Own visual timers and cancellation |

`createSelectedAltitude(parent, bugHost)` exposes these operations:

| Method | Contract |
| --- | --- |
| `resize(allocation)` | Update local geometry without advancing sample history |
| `accept(altitude, target, now)` | Advance alert history from the accepted sample |
| `render(altitude, target, now)` | Present the current state without advancing history and return `nextUpdateAt` |
| `invalidate()` | Reset alert history and local presentation state, restore value visibility, and hide the bug |

### Geometry, clipping, and paint order

| Parameter | Requirement |
| --- | --- |
| Readout allocation | Existing 32 CSS px strip above the altitude allocation |
| Tape host and clip | Share altitude's tape host and clip |
| Tape scale | Existing 0.4 CSS px/ft |
| Edge containment | Keep the whole bug, including its stroke, inside either tape edge |
| Symbol | Turquoise seven-pixel vertical block with a right-side notch matching the altitude window's pointer |
| Symbol height | 28 pixels, matching the altitude window |
| Header symbol | Use the same path as the tape bug |

Paint the overlapping elements in this order:

1. Altitude trend.
2. Selected-altitude bug.
3. Both opaque altitude/VSI windows, including their borders and text.

### Alert history and scheduler integration

The [panel lifecycle](lifecycle.md#sample-acceptance-and-drawing) connects sample history to shared scheduling:

* Every fresh accepted sample advances history before drawing is coalesced.
* Rendering and resize never advance sample history.
* Resize and timed redraws preserve history and the original sample deadline.
* Delayed renders calculate the current phase from monotonic time without replaying missed phases.

Before accepting fresh recovery data, the scheduler invalidates expired retained data through the panel.
This resets the selected object's history even when expiry callbacks were delayed.
The object returns the next visual deadline. Only the shared scheduler owns the timer.

Invalidation clears history through the panel and cancels pending timer and frame work through the scheduler.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

Issue [#24](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) owns selected-altitude and alert evidence.

### Acceptance scenarios

Use the [telemetry contract](../telemetry-contract.md#optional-selected-altitude) and [presentation rules](#presentation).

| Scenario | Expected behavior |
| --- | --- |
| Initialization and recovery | Establish the current band silently from fresh data |
| Approach thresholds | Apply exact unrounded 1000-ft and 200-ft transitions in both directions, including skipped bands |
| Capture, departure, and return | Arm deviation, flash yellow on departure, retain steady yellow, and allow another departure after return |
| Target change or required input loss | Reset threshold history, deviation, and flash timing |
| Pressure-only loss | Preserve the selected target and alerts |
| Coalesced drawing | Retain transitions from every fresh accepted sample |

### Automated checks and controlled time

The [automated Rust and Node commands](../testing/automated.md#automated-checks) include these tests:

| Boundary | Checks |
| --- | --- |
| Rust model and demo | Finite signed targets, independent omission, shared freshness, old publications, two SSE clients, 198-second checkpoints and unchanged 18-second cycles |
| Alert transitions | Both approach directions, exact thresholds, unrounded values, skipped bands, sustained values, target changes, capture, departure, return and repeat departure |
| Controlled time | 250-ms flash phases, completion at 5000 ms, delayed/early callbacks, identical fresh samples, resize, coalesced samples and expiry before draw |
| Invalidation | Local missing/overflow values, paused/stale/disconnected/waiting/invalid states, suspension, timed alert cancellation under a controlled monotonic clock, stale callbacks and silent recovery |

Controlled clocks check five-second altitude alerts, target-change reset, freshness, and expiry before pending draw during full-panel acceptance.
Use no wall-clock sleeps.

### Desktop geometry and availability transitions

Both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) are required for this feature.
The selected SVG checks use real panel modules with injected monotonic time:

* Check matching notched symbols in header and tape, readout above the bug, signed capture, and edge clearance.
* Check colors, availability, and stable nodes.
* Check bug paint visibility and minus-sign clearance at positive and negative capture without five-second wall-clock waits.

Inspect screenshots at `.local/pfd-browser/selected-altitude-*`.

### Real-phone checks

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #24, with these alert-specific checks:

1. Watch the 198-second demo for both approach directions, full flash intervals, target changes, and yellow deviation.
2. Check the pinned bug and selected value against sky and ground in portrait and short landscape, including 320 CSS px width.
3. Use [phone fixtures](../testing/devices.md#phone-fixture-setup) for target loss, malformed targets, both overflow boundaries, current-altitude loss, pressure-only loss, and return into capture.
4. Perform the shared loss and resume scenarios during an alert. Recovery requires fresh data and silent initialization.

> Desktop and automated results do not establish real iOS Safari or Android Chrome acceptance.

### Full-panel integration

* Check the embedded selected module, actual Rust target in SSE/SVG, pinned bug, and recovery after restart.
* Check selected-altitude bugs outside the visible tape during full-panel acceptance.
* Inspect `.local/pfd-runtime-selected-altitude.png`.

## Rationale and regression lessons

The reference is Garmin G5 Guide 190-01112-12 Rev. 2, sections 3.3.4 and 3.3.6.
The graphics remain original application assets.

These lessons explain the lifecycle and validation constraints:

* Advance history before coalescing draws so transitions between animation frames remain observable.
* Reset expired history before accepting recovery data. The expiry callback may not have run yet.
* Keep history with the selected visual object and timing with the common scheduler to preserve one cancellation path.
* Entry into +/-200 ft as capture, silent initialization, 2 Hz flashing, and persistent yellow are project decisions for #24. Garmin does not fully specify these behaviors.
* Compare SVG matrix positions with a subpixel tolerance because fractional values use single-precision matrices.
