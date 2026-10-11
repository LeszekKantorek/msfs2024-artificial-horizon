# PFD presentation

Instrument modules, panel composition, rendering lifecycle, and SVG layers.
System context: [architecture](architecture.md). Decisions: [instrument composition](adr/0004-modular-pfd-presentation.md) and [visual-object composition](adr/0005-visual-object-composition.md).
Delivery order: [roadmap](roadmap.md). Validation: [testing](testing.md#modular-and-layered-pfd-acceptance).

## Current implementation and planned changes

The panel implements attitude, slip/skid, and turn rate from #20, IAS and GS from #21, and ranges/trend from #22.
It also implements altitude/pressure from #23, selected-altitude alerts from #24, and vertical speed from #25.
Current heading and the selected-heading bug implement #26.
#36 separates rendering from scheduling. #37 implements layers and full-panel background coverage.
Maintenance [#54](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/54) introduced internal calculation and validation seams.
[#56](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/56) composes visual objects and places object-specific calculations with their implementing modules:

| Module | Implemented responsibility |
| --- | --- |
| `app.js` | Assemble telemetry, status, panel, resize observation, and page lifecycle |
| `telemetry/client.js` | Own EventSource, sequence handling, retries, expiry, status, and start/stop lifecycle |
| `telemetry/validation.js` | Pure snapshot validation and independent optional-value normalization |
| `frame-scheduler.js` | Retain the latest frame, coalesce draws, check expiry, and cancel invalid work |
| `panel.js` | Compose instruments, own the common attitude clip, and delegate resize/render/invalidation |
| `horizon/` | Compose world, pitch scale with chevrons, and bank pointer. Calculate common attitude transforms |
| `fixed-symbols/` | Compose fixed bank scale and aircraft reference with outline |
| `slip-skid.js`, `turn-rate.js` | Local geometry, readings, and indication availability |
| `airspeed/` | Compose IAS and independent GS, with SVG operations and calculations in each visual object |
| `altitude/` | Compose the tape, rolling readout, selected target/alerts, pressure setting and compact VSI |
| `altitude/vsi.js` | Compact numeric readout, local window geometry, and independent availability |
| `heading.js` | Magnetic-heading scale, pointer readout, selected bug, pure calculations and independent availability |
| `status.js` | Accessible status, and global unavailability |
| `helpers.js` | Named clamp, SVG creation, and attribute updates without import-time DOM access |

* Final background opacity acceptance requires real-phone evidence.
* Later feature issues add their instrument modules with real demo data.
* Implementation does not establish real-device acceptance.
* GitHub issues own real-device acceptance evidence.

## Module composition

```text
Panel
|-- Horizon
|   |-- World (sky, ground, horizon line)
|   |-- Pitch scale and chevrons
|   `-- Bank pointer
|-- Airspeed
|   |-- IAS
|   |   |-- Tape (scale, ranges, trend)
|   |   |-- Readout (window, digits, unit, warning)
|   |   `-- References (markers, leaders, fallback values)
|   `-- Ground speed
|-- Altitude
|   |-- Tape and rolling readout
|   |-- Pressure setting
|   |-- Selected altitude and alerts
|   `-- Vertical speed
|-- Heading (scale, current readout and selected bug)
|-- Slip/skid
|-- Turn rate
|-- Fixed symbols (bank scale, aircraft and outline)
`-- Status and data availability
```

* A module does not require one file per graphic element.
* Add future instruments in their feature issues, without empty implementations or production placeholder readings.

| Owner | Responsibility |
| --- | --- |
| Application assembly | Connect telemetry, panel, resize observation, and page lifecycle |
| Telemetry client | Validate messages, preserve source/transport status, and expire data without new samples |
| Frame scheduler | Retain the latest sample, coalesce redraws, check expiry, and cancel pending work |
| Panel | Compose instruments, assign layers, and resize their shared attitude clip |
| Layout | Allocate CSS-pixel regions, including the background extent and central readable area |
| Instrument composition | Select frame fields and delegate concrete geometry/readings to visual objects |
| Visual object | Own its nodes, geometry, calculations, and local availability |
| Fixed symbols | Draw references that do not move with the world |
| Status | Present source/transport status and global unavailability without owning the connection |

Keep data conversion and rendering responsibilities within these boundaries:

* Use local instrument coordinates and named dimensions.
* Keep telemetry units at the input and conversion to display coordinates inside the instrument.
* Keep pure envelope validation in `telemetry/validation.js` and sample-age enforcement in the telemetry client.
* Use source-supplied trend data based on source time, not differences between browser arrival times.
* Layout calculations do not mutate telemetry or impose a fixed aspect ratio.
* CSS owns page composition and instrument background opacity.
* Share helpers for matching rules with actual callers, such as mathematical `clamp` and SVG attribute updates.
* Do not introduce a general rendering framework or a second renderer.

## Internal module seams

The instrument factories retain their external interfaces:

| Factory | Returned operations |
| --- | --- |
| `createAirspeed(instrument, layers)` in `airspeed/index.js` | `resize(layout)`, `render(frame)` |
| `createHorizon(instrument, layers)` in `horizon/index.js` | `resize(layout)`, `render(frame)` |
| `createFixedSymbols(instrument, symbols)` in `fixed-symbols/index.js` | `resize(layout)` only |
| `createHeading(instrument, layers)` in `heading.js` | `resize(layout)`, `render(frame)`, `invalidate(reason)` |

Internal objects use these ownership rules:

* Instrument entry points receive the complete read-only frame.
* Internal objects receive concrete SVG hosts, geometry, and the readings their operations require.
* Factories select existing owned nodes or create dynamic nodes once.
* Returned objects expose operations without public node bags.
* Parents update their own shared hosts and clips before invoking child operations.
* Visual closures retain nodes and initialized geometry without a latest-frame cache or timer.
* Selected altitude also retains alert history.
* The frame scheduler owns its timing work.

### Airspeed objects

`airspeed/index.js` composes IAS and GS:

* Compute the shared GS panel position once per resize.
* Pass `layout.airspeed` and the shared position to IAS.
* Pass `{ x, y, width }` to GS.
* Select IAS/profile/acceleration and GS from the complete frame during render.
* Keep coherent IAS readings and child geometry in IAS.
* Keep GS state, rounding, and geometry in GS.

| Object / factory | Owned content | Resize input | Render input |
| --- | --- | --- | --- |
| IAS: `createIas(defs, parent, warnings)` | IAS root/background, shared tape host and tape clip, state/ARIA flags, child composition | IAS allocation and shared GS panel Y | IAS value, profile, acceleration |
| Tape: `createTape(defs, tapeHost)` | Barber pattern, scale/ticks/labels, named range rectangles, trend | Tape geometry | IAS reading, profile, prediction |
| Readout: `createReadout(defs, iasHost, warnings)` | Exclusive digit clip, window, columns, unit, IAS warning | Readout geometry and unit position | IAS reading and fallback-reference flag |
| References: `createReferences(tapeHost, iasHost)` | Markers/leaders and bottom group/unit/values | Tape geometry | IAS reading, profile, fallback-reference flag |
| GS: `createGroundSpeed(parent, warnings)` | Independent GS group/background/label/value/unit and warning | `{ x, y, width }` | GS value |

Ownership and availability remain local:

* IAS owns the clip shared by tape and reference markers.
* Readout owns its exclusive digit clip.
* References owns both tape markers/leaders and fallback values, keeping each reference in one record.
* Tape retains named `renderRanges` and `renderTrend` operations inside the same object.
* IAS, GS, profile, and acceleration availability remain independent.
* Every frame resets relevant warning, visibility, unit, fallback, and trend states, including missing data.

### Horizon and fixed objects

Horizon shares origins, clips, and transforms through composition:

* Horizon owns the common world/warning origins, background clip, and pitch clip.
* Horizon calculates `attitudeTransforms` once per frame from attitude and initialized local pitch geometry.
* World, pitch scale, and bank pointer receive the same transforms object.

| Object / factory | Owned content | Operations and concrete inputs |
| --- | --- | --- |
| World: `createWorld(rotationHost)` | World rotation/pitch hosts and sky/ground/horizon paths | `resize({ width, height, pitchScale })`, `render(transforms)` |
| Pitch: `createPitchScale(scaleRotationHost, warningRotationHost)` | Scale/warning rotation/pitch hosts, ticks, labels, chevrons | `resize({ pitchScale, labelSize })`, `render(transforms)` |
| Pointer: `createBankPointer(originHost)` | Exclusive pointer origin and path | `resize({ cx, cy, bankRadius })`, `render(transforms)` |
| Fixed composition | Common fixed-symbol origin and layer mounting | `resize(layout)` |
| Bank: `createBankScale(bankHost)` | Bank zero mark, ticks, and local constants | `resize(radius)` |
| Aircraft: `createAircraft(fixedHost)` | Aircraft shape/outline and common path calculation | `resize(width)` |

Panel and fixed-object boundaries remain explicit:

* The panel owns `#attitude-clip`, shared by horizon's bank pointer and fixed symbols.
* `composePanel(svg, instruments, status)` retains its API.
* Panel composition assigns existing `layout.attitude` before instrument resize.
* No instrument scale or path calculation moves into panel delegation.
* Fixed objects have no per-frame render operation.
* Aircraft calculates its common shape/outline path once per resize.

### Pure calculations and helpers

Object-specific calculations remain in the modules that implement their objects:

| Module | Calculations |
| --- | --- |
| `horizon/index.js` | `attitudeTransforms` |
| `airspeed/ias.js` | IAS readings, `speedIndications`, and child geometry |
| `airspeed/ground-speed.js` | GS state, rounding, and geometry |
| `airspeed/readout.js` | `rollingDigits` and local `DIGIT_HEIGHT` |
| `airspeed/tape.js` | `speedRanges`, `rangeBounds`, and local tick constants |
| `airspeed/references.js` | `referenceReadout` and `placeReferenceLabels` |

Pure calculations follow visual ownership:

* Named pure exports support direct behavioral tests in Node without DOM.
* IAS child geometry and GS geometry stay inside their owning resize operations.
* Purity does not require a separate calculation file.
* `airspeed/helpers.js` exports only `MAX_SPEED`, `speedState`, and `tapeOffset` for actual shared rules.

Shared speed rules have these consumers and units:

| Rule | Consumers / value |
| --- | --- |
| `speedState` | IAS and GS |
| `tapeOffset` | Tape and references, using 4 px/kt |
| `MAX_SPEED` | Speed classification, tape, and references |
| Tape steps | Tape owns 2 kt minor steps and 10 kt major steps |
| Digit height | Readout owns 22 px |

The mixed helper module preserves its import boundary:

* `helpers.js` exports `clamp(value, min, max)`, `svgElement(name, attributes, text)`, and `setAttributes(element, values)`.
* Clamp clips infinities and preserves `NaN` propagation without normalizing invalid telemetry.
* DOM access occurs when SVG creation executes, never during module import.

The [visual-object decision](adr/0005-visual-object-composition.md) records this mixed-helper compromise.

### Telemetry operations

The telemetry client preserves the connection lifecycle:

* `telemetry/client.js` exports only `createTelemetryClient`.
* Its private operations are `readMessage`, `acceptSample`, `publishSample`, and `expireSample`.
* The current-connection guard precedes parsing.
* Parsing catches message decoding failures without swallowing callback exceptions.

The adjacent validation module exposes these pure functions:

| Function | Result |
| --- | --- |
| `isValidSnapshot(value)` | Boolean envelope and attitude validity |
| `normalizeAirspeedProfile(value)` | Copied valid profile with separate reference values, or `null` |
| `normalizeOptionalIndications(value)` | Independently normalized optional values, including profile and acceleration |

Validation and publication retain these boundaries:

* Validation accepts finite nonnegative speeds above the display limit.
* The 999 kt limit, `OVR`, and six-second prediction belong to airspeed calculations.
* EventSource ownership, sequence handling, retry/age timers, expiry deadlines, and start/stop remain in the client.
* Both sample callbacks receive separate attitude copies and the same deadline.
* `onAttitude` retains its attitude-only shape.
* The application attaches the current copied status when accepting the complete sample into the scheduler.

Accepted live samples invoke callbacks in this order:

| Order | Callback |
| --- | --- |
| 1 | `onStatus` |
| 2 | `onAttitude` |
| 3 | `onSample` |

Lifecycle state stays with the client and scheduler:

* Expiry retains the original received time and deadline, including rescheduling an early timer.
* One client owns the subscription, retry timer, and age timer.
* One scheduler owns the retained frame and pending draw.
* Slip/skid, turn rate, layout, status, and scheduler remain small standalone modules.

Further generalization remains deferred:

* Altitude uses its own tape and drums because its scale, signed values, and 20-ft digit steps differ from IAS.
* Selected-altitude and alert rules extend the altitude instrument in #24 without introducing a general tape abstraction.
* The current implementation adds no instrument base class, general tape renderer, or shared lower-indication controller.

### Callback reentrancy limit

The current client has these limits when a callback changes its lifecycle:

| Callback action | Possible continuation |
| --- | --- |
| Call `stop()` inside `onAttitude` | Invoke `onSample` and schedule expiry for the same acceptance |
| Call `stop()` inside `onSample` | Schedule expiry after the callback returns |
| Accept a nested sample | Retain the original timer behavior |

These limits remain outside the cancellation guarantee:

* An acceptance operation already executing a callback may continue after `stop()`.
* The #56 stage-1 review preserved these legacy behaviors for refactor equivalence.
* Future callers must not rely on these limitations as stable behavior guarantees.
* Changes require separate behavior scope and regression checks for callback-triggered stop, restart, and nested acceptance.

## Panel interface and frame lifecycle

```js
panel.resize(layout);
panel.accept(frame, now);
panel.render(frame, now);
panel.invalidate(reason);
```

| Operation | Contract |
| --- | --- |
| `resize(layout)` | Update instrument geometry, including while unavailable. Do not renew data validity |
| `accept(frame, now)` | Advance optional instrument history for each accepted fresh sample, without DOM writes |
| `render(frame, now)` | Apply one fresh, coherent presentation frame. Return the earliest next monotonic visual deadline, or `null` |
| `invalidate(reason)` | Obscure invalid indications immediately. Do not substitute level flight or zero values |

| Frame field | Meaning |
| --- | --- |
| `attitude`, `slip_skid`, `turn_rate_dps` | Values from one accepted sample |
| `ias_kt`, `gs_kt` | Independently normalized speeds from the same accepted sample, or `null` |
| `airspeed_profile`, `ias_acceleration_ktps` | Independently normalized configuration and source acceleration, or `null` |
| `altitude_ft`, `baro_inhg` | Independently normalized signed altitude and positive pressure setting, or `null` |
| `selected_altitude_ft` | Independently normalized signed selected altitude, or `null` |
| `vertical_speed_fpm` | Independently normalized signed ft/min, or `null` |
| `heading_deg`, `selected_heading_deg` | Independently normalized magnetic degrees in `[0, 360)`, or `null` |
| `expiresAt` | Original monotonic expiry deadline |
| `status` | Copied `source`, `transport`, and `state` values |

Instruments share the frame under these rules:

* Every instrument receives the same complete frame and selects its own fields without mutating the frame or another instrument.
* Nested frame values are also read-only.
* Instruments consume normalized frames.
* Missing or invalid optional readings arrive as `null`, without invented zero values.
* `resize(layout)` must run before the first `render(frame)`.
* Initialization supplies real geometry without substitute defaults.
* Future feature issues define additions to the frame and wire contract separately.

> The frame is an internal presentation value, not a new SSE schema.

The planned full composition has this shape.
Only implemented instruments participate at each delivery step:

```js
function renderPanel(frame) {
  horizon.render(frame);
  airspeedTape.render(frame);
  altitudeTape.render(frame);
  verticalSpeed.render(frame);
  direction.render(frame);
  slipSkid.render(frame);
  turnRate.render(frame);
  status.render(frame);
}
```

| Component | Initialization | Resize | Render |
| --- | --- | --- | --- |
| Instruments | Create SVG nodes once | Update geometry attributes on existing nodes | Apply pure calculations to SVG using the complete frame |
| Fixed symbols | Create references once | Update reference geometry | No per-frame render method |
| Panel | Compose implemented modules | Assign shared attitude clip, then delegate geometry updates | Delegate the same frame without scale formulas or path construction |

* Adding a data dependency inside an instrument does not change its call in the panel.
* Adding an instrument requires explicit composition in the panel.

The scheduler coordinates samples, layout, and invalidation in this sequence:

1. Create instrument elements once during initialization.
2. On each fresh accepted sample, delegate `accept(frame, now)` before retaining the newest sample and requesting at most one animation frame.
3. On resize with a retained sample, keep the newest layout and request at most one animation frame.
4. Before drawing, check the original deadline, then apply the pending layout and render the fresh frame in the same callback.
5. On invalidation, cancel pending drawing, discard the sample, and invoke `panel.invalidate(reason)` immediately.
6. Apply any pending layout under the unavailable cover, updating geometry immediately when no sample remains.
7. After reconnect or resume, wait for a new valid sample before restoring indications.

Freshness and status handling follow these rules:

* Resize never renews the sample deadline.
* Transport expiry detection continues even when no new samples arrive.
* Fresh samples with unchanged numeric values still renew freshness.
* The application passes the same status module instance to the panel.
* `status.update(status)` updates labels from the telemetry callback without exposing old readings.
* Scheduler expiry updates the visible stale reason even if the transport timer has not executed.
* Optional failures affect only their indications.
* Source/transport loss invalidates the panel.
* Do not interpolate or extrapolate current readings between accepted samples.
* The source-based [airspeed trend](#ranges-references-and-trend) remains a separate prediction indication.

Selected-altitude alerts use the existing scheduling and cancellation path:

* Altitude owns alert history and pure phase calculations.
* Rendering and resize never advance sample history.
* Instrument render operations can return a monotonic next-update deadline.
* Panel composition selects the earliest.
* The scheduler accepts injectable `now`, `setTimer`, and `clearTimer` dependencies.
* One timer requests the common animation frame at the earlier of the visual deadline and sample expiry.
* New samples cancel obsolete timing work.
* Rendering checks the retained sample's original deadline.
* Acceptance resets expired retained history before processing fresh recovery data, including when expiry callbacks were delayed.
* Timed visual changes use the same panel scheduling path and an injectable monotonic clock.
* Invalidation cancels their pending work and prevents an apparently valid alert from remaining active.

## Layers and clipping

| Layer | Content | Opacity and clipping |
| --- | --- | --- |
| 0 | Sky, ground, horizon line, pitch scale | Opaque full-SVG background, with a separate central clip for pitch markings |
| 1 | Airspeed tape, ticks, ranges, readout | Translucent background with readable digits, ticks, and value window |
| 2 | Altitude tape and VSI | Translucent background with readable indications and value windows |
| 3 | Fixed aircraft reference, other scales and indications | Transparent space around symbols |
| 4 | Warnings and unavailability | Cover the affected invalid indications, including the expanded horizon on global loss |

Instrument backgrounds use these opacity rules:

* Apply opacity to background shapes, not whole instrument groups.
* `--instrument-background-opacity` defaults to `0.65` pending real-phone acceptance in #37.
* Follow the [opacity comparison procedure](testing.md#modular-and-layered-pfd-acceptance) on real iOS Safari and Android Chrome.

> This architecture does not prescribe a final numeric opacity value.

| Element | Layer / origin | Clip and owner |
| --- | --- | --- |
| Sky/ground | Horizon, layer 0, origin at attitude center | Horizon background clip uses `layout.background` across the full SVG |
| Pitch scale | Horizon, layer 0, origin at attitude center | Horizon pitch clip fixed relative to horizon origin |
| Bank pointer | Horizon, layer 3, origin at attitude center | Panel attitude clip in panel coordinates |
| Chevrons | Horizon, layer 4, origin at attitude center | Horizon pitch clip fixed relative to horizon origin |
| Fixed symbols | Layer 3, origin at attitude center | Same panel attitude clip uses `layout.attitude` |
| IAS scale/ranges/trend/reference markers | Layer 1, IAS origin | IAS shared tape clip |
| IAS rolling digits | Layer 1, IAS origin | Readout exclusive digit clip |
| Slip/skid and turn rate | Own translated origins | Panel bounds |

Layer ownership, clipping, and transforms follow these rules:

* A module can own elements in several layers.
* Instrument-specific warning geometry remains with its instrument.
* Background, attitude, and pitch clips stay fixed while the world rotates and translates.
* Pitch translation remains nested inside bank rotation.
* Transform only moving elements and preserve rotation centers during resize.
* Size text, symbols, and scales independently without distorting their proportions.
* The development fixture uses the same instrument modules and layer containers as production.

Keep indications readable across the full panel:

* Separate background coverage from the central area reserved for readable attitude symbols.
* Intentional background overlap is allowed.
* Collisions between digits, ticks, and unrelated symbols are not allowed.
* Check painted extents, including stroke clearance, rather than only allocated rectangles.
* Use the full 17-element development fixture when checking instrument geometry.
* Preserve responsive sizing, safe areas, and the minima in the [brief](project-brief.md#mobile-presentation).

## Current and selected heading

`heading.js` owns its SVG, formatting, geometry, availability, and pure calculations:

* The module occupies `layout.heading` in the existing 32 CSS px strip between IAS and altitude.
* It uses layer 3 (`symbols`) with its own screen-space clip. Horizon geometry remains unchanged.
* Resize updates geometry and grows the retained tick pool only when needed.
* Render consumes the complete frame. Invalidation clears availability, scale visibility, and bug position.
* Shared frame scheduling and the original expiry deadline apply without history, filtering, interpolation, or a new timer.

The scale and current readout use these dimensions:

| Element | Rule |
| --- | --- |
| Scale | Fixed 4 CSS px per degree. Minor ticks every degree, major ticks every 5 degrees |
| Labels | Every 10 degrees, three digits, monospace 11 px. North is `360` |
| Current value | Three digits and degree symbol, rounded to whole degrees, monospace 14 px. North is `360°` |
| Window | Centered 44 by 22 px rectangle, with a lower pointer 16 px wide and 8 px high |
| Colors | Existing text and value-window tokens, with a white 1 px border |
| Selected bug | Cyan 16 by 7 px block at the tape's bottom, with a complementary upper notch |

Geometry and paint order preserve capture and edge clearance:

* The pointer and notch share dimensions in one module.
* The scale paints first, then the bug, then the opaque black window, border, and current value.
* At capture, cyan remains visible around the pointer.
* Selected position uses the shortest angular difference in `[-180, 180)`. Exactly 180 degrees goes left.
* Clamp the bug center with an 8.5 px inset, preserving the whole symbol and its stroke at either edge.
* For current 359 degrees and selected 1 degree, the bug is 8 px right of center.
* Scale calculations use unrounded heading and wrap continuously across north in either direction.
* Hide entire labels that intersect the readout or either clip edge. Do not clip individual digits.

Availability remains local:

* Missing or invalid current heading shows red `X`, hiding the scale and bug.
* Missing or invalid selected heading hides only the bug.
* Missing altitude, pressure, or other optional fields does not affect a valid heading.
* Ground track does not substitute for heading. No separate selected-heading readout or control is provided.

Garmin G5 Guide 190-01112-12 Rev. 2, section 3.3.8, provides the presentation reference.
Fixed spacing, edge pinning, dimensions, and independent availability are application design decisions.

## Vertical speed

`altitude/index.js` composes the stateless `vsi.js` object inside the altitude group:

* A 51-by-27 CSS px rectangle sits below the current-altitude window, with aligned right borders and three pixels between painted strokes.
* The window shares altitude's monospace font family, text color, background and one-pixel border.
* VSI uses 13 CSS px text, one pixel smaller than altitude's 14 CSS px text.
* The value aligns with the right edge of altitude's digits, using feet per minute without a visible unit label or arrow.
* The altitude readout supplies that edge from the measured bounds of its two-digit monospace drum during resize.
* VSI accounts for glyph bounds when positioning text, including the wider unavailable `X`.
* Altitude keeps its existing tape width beside a four-pixel trend lane at the right edge.
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
| Raw value within -9999 to +9999 ft/min | Rounded whole ft/min, with `+` for positive and `-` for negative readings |
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

## Altitude trend

`altitude/index.js` composes the stateless `trend.js` object beside the tape:

* A four CSS px lane follows the tape's right edge without reducing tape or readout width.
* The magenta line shares the airspeed trend's two-pixel stroke and color.
* It extends from the current-altitude center toward the altitude predicted in six seconds.
* Source VSI supplies the rate without estimating browser altitude differences.
* Climb extends upward and descent downward at the tape's fixed 0.4 CSS px/ft spacing.
* The line stops one pixel inside either vertical edge when its prediction exceeds the visible tape.
* The altitude and VSI windows end before this lane, retaining aligned right text edges.

The prediction uses feet and feet per minute:

```text
predicted_altitude_ft = altitude_ft + vertical_speed_fpm / 10
```

At 10060 ft and +1500 ft/min, the endpoint represents 10210 ft, 60 CSS px above the center.
Finite rates beyond VSI's numeric display limit still produce a clipped trend.
Zero rate, missing or invalid rate, and unavailable or overflowing current altitude hide the line.
Pressure and selected-target availability do not affect it.
The common frame and original sample deadline control rendering and invalidation, with no additional timer or interpolation.

This six-second altitude trend is an application design decision.

## Selected altitude and alerts

The altitude composition adds `selected.js` in layer 2:

* The selected object owns its black value window, symbol, tape bug, and alert history.
* The readout occupies the existing 32 CSS px strip above the altitude allocation.
* The bug shares altitude's tape host and clip, using the existing 0.4 CSS px/ft scale.
* The bug stays fully inside either tape edge, including its stroke.
* The turquoise symbol is a seven-pixel vertical block with a right-side notch matching the altitude window's pointer.
* Its 28-pixel height matches that window, and the header uses the same path.
* Ordinary selected values use turquoise text.
* Deviation uses yellow text on black.
* The bug paints below the opaque current-altitude window, its border and digits.
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

## Unavailable status

Status preserves the full-panel presentation and accessible state:

* The panel has no top status bar or reserved header space.
* The production panel has no source badge or DEMO announcement.
* Only the development fixture overlays a two-line `LAYOUT FIXTURE` badge in the empty tape area above ground speed.
* The fixture badge does not reserve production layout space.
* Successful source and transport messages have no visible labels.
* Source and transport states remain separate internally.
* A polite live region announces only changed text.
* Waiting, reconnecting, stale, suspended, paused, disconnected, and invalid states show an opaque full-panel cover with the applicable reason.
* Only rendering a fresh frame removes the cover.
* A `live` status alone does not remove the cover.

## SVG and CSS interface

IDs, classes, node order, layer placement, clipping, and accessible meanings form the presentation interface.
[Layer rules](#layers-and-clipping) define clip coordinates and transform order.

SVG identity and paint order remain stable:

* SVG nodes retain their identity through samples, loss, recovery, and resize.
* Reference leaders precede markers so labels remain above their leaders.

| Scope | Order |
| --- | --- |
| Definitions | Background clip, attitude clip, pitch clip, IAS tape clip, digit clip, barber pattern |
| IAS tape | Scale, ranges, trend, references |
| IAS | Background, tape, window, digits, unit, fallback references |
| Symbols layer | Bank pointer, fixed symbols, heading, slip/skid, turn rate |
| Heading | Background, scale, selected bug, current window, current value |
| Heading definitions | Exclusive heading tape clip |
| Fixed children | Bank scale, aircraft outline, aircraft shape |
| Warnings layer | Pitch warnings, IAS warning, GS warning, slip warning, turn warning |

| State written by JavaScript | CSS or presentation effect |
| --- | --- |
| Instrument `data-available` | Global SVG visibility and opaque unavailable cover |
| Slip/skid and turn-rate `data-available` | Local unavailable marks and hiding of the affected indication |
| IAS `data-state` | Hide tape and digits unless the value is available |
| IAS `data-overspeed`, `data-approaching` | Red pointer window and yellow predictive-warning digits |
| Bottom references `data-visible` | Hide the development badge while fallback references occupy its area |
| Heading `data-state`, `data-selected-available` | Local red `X`, scale visibility and independent selected-bug visibility |

State updates preserve availability and accessible meanings:

* IAS root state updates run before its child indication updates on every frame.
* GS updates its state independently.
* IAS and GS reset warning colors, availability, local X/OVR marks, profile visibility, fallback flags, and unit visibility.
* Trend updates clear the path and prediction attribute when no prediction exists.
* Accessible speed labels retain current values or unavailable/overflow states.
* Reference labels retain exact configured values despite compact visible text.
* Status announcements change only when their text changes, as defined in [unavailable status](#unavailable-status).

## Altitude and barometric setting

The altitude module uses the existing right-side allocation and layer 2:

* `altitude/index.js` composes the tape, readout, and independent pressure setting with `resize(layout)` and `render(frame)`.
* The composition owns the shared tape clip, background, unit label, geometry, and local altitude state.
* `tape.js` owns a retained pool of ticks and labels covering the visible range plus edge clearance.
* The pool grows only when resize needs more rows and hides surplus rows on smaller panels.
* Rendering reuses existing nodes without creating or replacing them.
* `readout.js` owns its digit clip, fixed opaque pointer window, sign, rolling columns, and layer-4 X/OVR mark.
* `baro.js` owns its background, value, unit, formatting, and independent state.
* Pure calculations remain in the visual objects that implement them; no generic IAS/altitude tape is introduced.

The tape and readouts use these presentation rules:

* The tape remains centered on the current altitude, with 20-ft minor ticks and 100-ft major ticks and labels.
* Minor ticks are always 8 CSS px apart and major ticks are 40 CSS px apart.
* This matches the former 400-ft scale at a 160 CSS px tape height in minimum-height landscape.
* Taller tapes show a wider range and more labels without stretching the spacing or text.
* A 160 CSS px tape spans 400 ft and a 400 CSS px tape spans 1000 ft.
* Higher values appear above the pointer.
* The tape includes negative values and crosses zero without a direction change.
* The readout supports -9999 through 99999 ft; valid values beyond that range show `OVR` and hide the scale and digits.
* The lower two digits roll together through 00/20/40/60/80, using the full fractional magnitude.
* Higher columns carry during the final 20 ft before their decimal boundary, with leading zeros hidden.
* Negative values have a separate minus sign and magnitude drums; zero, including negative zero, has no minus.
* An adjacent drum row outside the display range is blank instead of wrapping to zero.
* Missing altitude shows `X` and hides the scale and digits without hiding valid pressure or other instruments.
* The pressure readout stays at the bottom of the right column, with a 32-pixel background and a two-pixel panel inset.
* Pressure uses two decimal places and a separate `inHg` unit; positive values above 99.99 show `OVR`.
* A small positive pressure can round to 0.00; zero telemetry itself is invalid and shows `X`.
* Pressure failure affects only its readout; source/transport loss covers the entire panel.
* Backgrounds use the existing opacity token; text, ticks, and the pointer window remain opaque.
* Accessible labels preserve the full received values or the unavailable/overflow state without per-sample live announcements.

Node and browser tests cover calculations, clips, stable nodes, painted glyph bounds, and state transitions.
Real-phone acceptance remains separate in [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23).

## Airspeed and ground speed

Composition and geometry preserve the existing panel allocation:

* `airspeed/index.js` composes objects once and exposes `resize(layout)` and `render(frame)`.
* IAS and GS use layer 1.
* Their local failure and overflow marks use layer 4.
* The existing airspeed rectangle defines tape geometry.
* The central attitude allocation remains unchanged.
* GS stays at the bottom of the panel, with the same left edge and width as IAS.
* The IAS background extends to GS without a gap.
* When IAS is available, the area below `KT` contains no readings or scale markings.

The tape and readouts use these display rules:

* Tape spacing is 4 CSS px/kt, with major labels every 10 kt and minor ticks every 2 kt.
* The scale and rolling digits use fractional received IAS without rounding the input to whole knots.
* Screen-space clipping hides readings outside the tape rectangle and its unit label.
* The fixed opaque window clips three rolling digit columns.
* Leading zeros are blank.
* Units roll from fractional IAS.
* Higher columns roll during the final knot before a decimal carry.
* GS uses a separate labelled readout in kt, rounded to the nearest integer with halves rounded up.

Availability and lifecycle remain independent:

* Display values span 0–999 kt.
* Above 999 kt, show `OVR`.
* Unavailable values show `X`.
* Neither missing nor overflowing IAS leaves a visible numeric tape.
* GS availability remains independent.
* No independent animation, timer, interpolation, or client-generated flight values are used.

### Ranges, references, and trend

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

* White and green use adjacent strips where they overlap.
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

* Missing or invalid IAS shows a local `X` and the configured references in four rows above GS, with a `KT` heading.
* Bottom reference values round to 0.1 kt.
* Positive values below 0.1 show `<0.1`.
* Values above 999 show `OVR`.
* The fallback replaces the usual IAS unit label.
* The development badge hides while fallback references occupy its area.

Trend uses source-supplied acceleration:

* Zero IAS is valid.
* `OVR` retains its existing meaning and hides the numeric tape and trend.
* The browser calculates the six-second prediction as `IAS + 6 * ias_acceleration_ktps` from source-supplied acceleration.
* The magenta vector extends from current IAS toward that prediction.
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

All new content stays inside the existing IAS allocation. GS and central attitude geometry retain their positions.

## Delivery boundaries

Delivery preserves production scope and asset integration:

* Each implementation issue updates its tests, documentation, and mobile evidence.
* Update embedded Rust asset routes and the fixture server when imports change.
* The full development fixture uses production modules for all 17 included illustration elements.
* No new telemetry fields, framework, build pipeline, Solid.js, three.js, or custom signals are required for #36 or #37.
