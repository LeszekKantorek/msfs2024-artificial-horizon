# PFD presentation

Instrument modules, panel composition, rendering lifecycle, and SVG layers.
System context: [architecture](architecture.md). Decision: [ADR 0004](adr/0004-modular-pfd-presentation.md).
Delivery order: [roadmap](roadmap.md). Validation: [testing](testing.md#modular-and-layered-pfd-acceptance).

## Current implementation and planned changes

The panel implements attitude, slip/skid, and turn rate from #20, plus IAS and GS from #21 and ranges/trend from #22.
#36 separates rendering from scheduling. Maintenance [#54](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/54) organizes internals within the accepted design:

| Module | Implemented responsibility |
| --- | --- |
| `app.js` | Assemble telemetry, status, panel, resize observation, and page lifecycle |
| `telemetry-client.js` | Own EventSource, sequence handling, retries, expiry, status, and start/stop lifecycle |
| `telemetry/validation.js` | Pure snapshot validation and independent optional-value normalization |
| `frame-scheduler.js` | Retain the latest frame, coalesce draws, check expiry, and cancel invalid work |
| `panel.js` | Compose instruments and delegate resize, render, and invalidation |
| `horizon.js` | Sky/ground, pitch scale, chevrons, and bank pointer, with private world/pitch/bank resize operations |
| `fixed-symbols.js` | Fixed bank scale and aircraft references |
| `slip-skid.js`, `turn-rate.js` | Local geometry, readings, and indication availability |
| `airspeed.js` | Airspeed factory, private SVG creation/resize, state resets, and indication updates |
| `airspeed/calculations.js` | Pure speed states, ranges, trend, digits, reference formatting/placement, and local geometry |
| `status.js` | Accessible status, and global unavailability |
| `svg.js` | Shared SVG creation and attribute helpers |
| `math.js` | Shared mathematical clamp for layout, turn rate, IAS range bounds, and trend clipping |

* #37 adds full-panel background coverage, explicit layers, and translucent instrument backgrounds. Final opacity acceptance requires real-phone evidence.
* Later feature issues add their instrument modules with real demo data.
* Implementation does not establish real-device acceptance. GitHub issues own that evidence.

## Module composition

```text
Panel
|-- Horizon
|   |-- Sky and ground
|   `-- Pitch scale
|-- Airspeed tape                         (#21, extended by #22)
|   `-- Pure airspeed calculations        (internal)
|-- Altitude tape and selected-altitude alerts (planned #23/#24)
|-- Vertical speed                       (planned #25)
|-- Heading and track                    (planned #26)
|-- Slip/skid
|-- Turn rate
|-- Fixed symbols
`-- Status and data availability
```

* Sky/ground and the pitch scale can remain internal parts of the horizon module.
* A module does not require one file per graphic element.
* Add future instruments in their feature issues, without empty implementations or production placeholder readings.

| Owner | Responsibility |
| --- | --- |
| Application assembly | Connect telemetry, panel, resize observation, and page lifecycle |
| Telemetry client | Validate messages, preserve source/transport status, and expire data without new samples |
| Frame scheduler | Retain the latest sample, coalesce redraws, check expiry, and cancel pending work |
| Panel | Compose instrument modules and assign their layer containers |
| Layout | Allocate CSS-pixel regions, including the background extent and central readable area |
| Instrument | Own its SVG and pure calculations for scales, positions, labels, and local availability |
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

The public airspeed module exports only `createAirspeed` and returns `resize(layout)` and `render(frame)`.
Its closure retains the created SVG nodes and latest geometry. It retains no second telemetry frame or timer.

| Internal implementation | Responsibility |
| --- | --- |
| Private SVG operations in `airspeed.js` | Create nodes once, apply resize geometry, reset states, and update tape, ranges, trend, references, digits, and GS |
| `airspeed/calculations.js` | Pure speed classification, readings, ranges, six-second prediction, rolling digits, reference text/placement, tape conversion, and geometry |
| Reference records | Associate each key, symbol, accessible name, leader, marker, and fallback label without parallel node arrays |
| Named range rectangles | Associate each SVG rectangle with its range name |
| Geometry groups | Separate `tape`, `readout`, and `groundSpeed`, with explicit panel coordinates for warnings |

Scale counts and positions derive from one 4 px/kt conversion, 2/10 kt steps, and the 999 kt maximum.
The digit clip uses `DIGIT_HEIGHT`. Other dimensions retain their distinct meanings.
Calculation tests import the internal module directly. The factory adds no calculation re-exports.
The calculation module has no DOM, transport, or timer dependencies. It imports only mathematical `clamp`.

The telemetry client exports only `createTelemetryClient`. Its internal validation module exposes these pure functions:

| Function | Result |
| --- | --- |
| `isValidSnapshot(value)` | Boolean envelope and attitude validity |
| `normalizeAirspeedProfile(value)` | Copied valid profile with separate reference values, or `null` |
| `normalizeOptionalIndications(value)` | Independently normalized optional values, including profile and acceleration |

Validation accepts finite nonnegative speeds above the display limit. The 999 kt limit, `OVR`, and six-second prediction belong to airspeed calculations.
EventSource ownership, sequence handling, retry/age timers, expiry deadlines, and start/stop remain in the client.
For an accepted live sample, `onStatus` precedes `onAttitude`, which precedes `onSample`.
Both sample callbacks receive separate attitude copies and the same deadline. `onAttitude` retains its attitude-only shape.
The application attaches the current copied status when accepting the complete sample into the scheduler.

`math.js` supplies only `clamp(value, min, max)`. It clips infinities and preserves `NaN` propagation without normalizing invalid telemetry.
Slip/skid and turn rate retain separate availability and limit rules. Existing SVG helpers cover their related attribute writes.
Fixed symbols, panel delegation, frame scheduling, application assembly, status, and SVG helpers retain their responsibilities.

A general IAS/altitude tape remains deferred until actual altitude implementation.
Compare concrete scale, sign, digit, selection, and alert rules before sharing an operation.
The current implementation adds no instrument base class, general tape renderer, or shared lower-indication controller.

## Panel interface and frame lifecycle

```js
panel.resize(layout);
panel.render(frame);
panel.invalidate(reason);
```

| Operation | Contract |
| --- | --- |
| `resize(layout)` | Update instrument geometry, including while unavailable. Do not renew data validity |
| `render(frame)` | Apply one fresh, coherent presentation frame to existing instrument elements |
| `invalidate(reason)` | Obscure invalid indications immediately. Do not substitute level flight or zero values |

| Frame field | Meaning |
| --- | --- |
| `attitude`, `slip_skid`, `turn_rate_dps` | Values from one accepted sample |
| `ias_kt`, `gs_kt` | Independently normalized speeds from the same accepted sample, or `null` |
| `airspeed_profile`, `ias_acceleration_ktps` | Independently normalized configuration and source acceleration, or `null` |
| `expiresAt` | Original monotonic expiry deadline |
| `status` | Copied `source`, `transport`, and `state` values |

Instruments share the frame under these rules:

* Every instrument receives the same complete frame and selects its own fields without mutating the frame or another instrument.
* Nested frame values are also read-only.
* Instruments consume normalized frames. Missing or invalid optional readings arrive as `null`, without invented zero values.
* `resize(layout)` must run before the first `render(frame)`. Initialization supplies real geometry without substitute defaults.
* Future feature issues define additions to the frame and wire contract separately.

> The frame is an internal presentation value, not a new SSE schema.

The planned full composition has this shape; only implemented instruments participate at each delivery step:

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
| Panel | Compose implemented modules | Delegate geometry updates | Delegate the same frame without scale formulas or path construction |

* Adding a data dependency inside an instrument does not change its call in the panel.
* Adding an instrument requires explicit composition in the panel.

The scheduler coordinates samples, layout, and invalidation in this sequence:

1. Create instrument elements once during initialization.
2. On an accepted sample, retain that sample and request at most one animation frame.
3. On resize with a retained sample, keep the newest layout and request at most one animation frame.
4. Before drawing, check the original deadline, then apply the pending layout and render the fresh frame in the same callback.
5. On invalidation, cancel pending drawing, discard the sample, and invoke `panel.invalidate(reason)` immediately.
6. Apply any pending layout under the unavailable cover; with no sample, resize updates geometry immediately.
7. After reconnect or resume, wait for a new valid sample before restoring indications.

Freshness and status handling follow these rules:

* Resize never renews the sample deadline.
* Transport expiry detection continues even when no new samples arrive.
* Fresh samples with unchanged numeric values still renew freshness.
* The application passes the same status module instance to the panel.
* `status.update(status)` updates labels from the telemetry callback without exposing old readings.
* Scheduler expiry updates the visible stale reason even if the transport timer has not executed.
* Optional failures affect only their indications; source/transport loss invalidates the panel.
* Use no interpolation or extrapolation.

Future timed alerts use the existing scheduling and cancellation path:

* Selected-altitude alert timing belongs to #24.
* Timed visual changes use the same panel scheduling path and an injectable monotonic clock.
* Invalidation cancels their pending work and prevents an apparently valid alert from remaining active.

## Layers and clipping

| Layer | Content | Opacity and clipping |
| --- | --- | --- |
| 0 | Sky, ground, horizon line, pitch scale | Opaque background covering the entire SVG; pitch markings have a separate central clip |
| 1 | Airspeed tape, ticks, ranges, readout | Translucent background; readable digits, ticks, and value window |
| 2 | Altitude tape and VSI | Translucent background; readable indications and value windows |
| 3 | Fixed aircraft reference, other scales and indications | Transparent space around symbols |
| 4 | Warnings and unavailability | Cover the affected invalid indications, including the expanded horizon on global loss |

Instrument backgrounds use these opacity rules:

* Apply opacity to background shapes, not whole instrument groups.
* `--instrument-background-opacity` defaults to `0.65` pending real-phone acceptance in #37.
* Follow the [opacity comparison procedure](testing.md#modular-and-layered-pfd-acceptance) on real iOS Safari and Android Chrome.

> This architecture does not prescribe a final numeric opacity value.

| Element | Layer / origin | Clip |
| --- | --- | --- |
| Sky/ground | Horizon, layer 0, origin at attitude center | `layout.background` covers the full SVG |
| Pitch scale | Horizon, layer 0, origin at attitude center | Pitch clip fixed relative to horizon origin |
| Bank pointer | Horizon, layer 3, origin at attitude center | Attitude clip in panel coordinates |
| Chevrons | Horizon, layer 4, origin at attitude center | Pitch clip fixed relative to horizon origin |
| Fixed symbols | Layer 3, origin at attitude center | `layout.attitude` defines the central reference area |
| Slip/skid and turn rate | Own translated origins | Panel bounds |

Layer ownership, clipping, and transforms follow these rules:

* A module can own elements in several layers; instrument-specific warning geometry remains with its instrument.
* Background, attitude, and pitch clips stay fixed while the world rotates and translates.
* Pitch translation remains nested inside bank rotation.
* Transform only moving elements and preserve rotation centers during resize.
* Size text, symbols, and scales independently without distorting their proportions.
* The development fixture uses the layer containers intended for future tape and heading modules.

Keep indications readable across the full panel:

* Separate background coverage from the central area reserved for readable attitude symbols.
* Intentional background overlap is allowed; collisions between digits, ticks, and unrelated symbols are not.
* Check painted extents, including stroke clearance, rather than only allocated rectangles.
* Use the full 18-element development fixture before later instruments reduce usable space.
* Preserve responsive sizing, safe areas, and the minima in the [brief](project-brief.md#mobile-presentation).

## Unavailable status

* The panel has no top status bar or reserved header space.
* The production panel has no source badge or DEMO announcement.
* Only the development fixture overlays a two-line `LAYOUT FIXTURE` badge in the empty tape area above ground speed; it does not reserve production layout space.
* Successful source and transport messages have no visible labels.
* Source and transport states remain separate internally; a polite live region announces only changed text.
* Waiting, reconnecting, stale, suspended, paused, disconnected, and invalid states show an opaque full-panel cover with the applicable reason.
* Only rendering a fresh frame removes the cover; a `live` status alone does not.

## SVG and CSS interface

IDs, classes, node order, layer placement, clipping, and accessible meanings form the presentation interface.
[Layer rules](#layers-and-clipping) define clip coordinates and transform order.
SVG nodes retain their identity through samples, loss, recovery, and resize.
Within the IAS tape, scale, ranges, trend, and references retain their paint order.
Reference leaders precede markers so labels remain above their leaders.

| State written by JavaScript | CSS or presentation effect |
| --- | --- |
| Instrument `data-available` | Global SVG visibility and opaque unavailable cover |
| Slip/skid and turn-rate `data-available` | Local unavailable marks and hiding of the affected indication |
| IAS `data-state` | Hide tape and digits unless the value is available |
| IAS `data-overspeed`, `data-approaching` | Red pointer window and yellow predictive-warning digits |
| Bottom references `data-visible` | Hide the development badge while fallback references occupy its area |

Speed state updates run before conditional indication updates on every frame.
They reset warning colors, availability, local X/OVR marks, profile visibility, fallback flags, and unit visibility.
Trend updates clear the path and prediction attribute when no prediction exists.
Accessible speed labels retain current values or unavailable/overflow states. Reference labels retain exact configured values despite compact visible text.
Status announcements change only when their text changes, as defined in [unavailable status](#unavailable-status).

## Airspeed and ground speed

* `airspeed.js` creates SVG once and exposes `resize(layout)` and `render(frame)`.
* IAS and GS use layer 1. Their local failure and overflow marks use layer 4.
* The existing airspeed rectangle defines tape geometry. The central attitude allocation remains unchanged.
* Tape spacing is 4 CSS px/kt, with major labels every 10 kt and minor ticks every 2 kt.
* The scale translates from received IAS. Screen-space clipping hides readings outside the tape rectangle and its unit label.
* The fixed opaque window clips three rolling digit columns. Leading zeros are blank.
* Units roll from fractional IAS. Higher columns roll during the final knot before a decimal carry.
* GS uses a separate labelled readout in kt, rounded to the nearest integer.
* GS stays at the bottom of the panel, with the same left edge and width as IAS.
* The IAS background extends to GS without a gap. When IAS is available, the area below `KT` contains no readings or scale markings.
* Display values span 0–999 kt. Above 999 kt, show `OVR`. Unavailable values show `X`.
* Neither missing nor overflowing IAS leaves a visible numeric tape. GS availability remains independent.
* No independent animation, timer, interpolation, or client-generated flight values are used.

### Ranges, references, and trend

The same module consumes the optional profile and acceleration from the complete frame.
It creates all new SVG nodes once. Resize and render retain the existing panel lifecycle.

| Range | Bounds in configured knots | Demo bounds |
| --- | --- | --- |
| Red low speed | `[0, VS0)` | 0-40 |
| White flap range | `[VS0, VFE)` | 40-85 |
| Green normal range | `[VS1, VNO)` | 50-110 |
| Yellow caution range | `[VNO, VNE)` | 110-140 |
| Red/white overspeed | `[VNE, infinity)` | From 140 |

White and green use adjacent strips where they overlap. All ranges use the IAS scale's 4 CSS px/kt spacing and screen-space clip.
The fixed window remains opaque. Digits use 17 px text to preserve three readable columns beside the new indications at minimum width.

References and trend use these rules:

* R, X, Y, and G denote VR, VX, VY, and VG. The synthetic demo references are 55, 65, 75, and 90 kt.
* Each reference sits to the right of the scale at its configured speed. Markers outside the visible tape stay hidden, without an edge substitute.
* An 8 px vertical inset preserves each marker's painted text and stroke at the tape boundaries.
* Coincident or nearby references retain their exact speed anchors. Displaced labels use leader lines and 16 px spacing inside the tape.
* Missing or invalid IAS shows a local `X` and the configured references in four rows above GS, with a `KT` heading.
* Bottom reference values round to 0.1 kt. Positive values below 0.1 show `<0.1`, and values above 999 show `OVR`.
* Accessible reference labels retain exact configured values. Marker positions use those exact values.
* The fallback replaces the usual IAS unit label. The development badge hides while fallback references occupy its area.
* Zero IAS is valid. `OVR` retains its existing meaning and hides the numeric tape and trend.
* The magenta vector extends from current IAS toward the source-supplied six-second prediction. Its stroke stays inside the tape clip.
* Constant speed, missing acceleration, or a non-finite prediction hides the vector. Missing configuration does not prevent a valid trend calculation.

VNE changes the current-speed window and digits independently:

* At or above VNE, the pointer window becomes red, including when the readout shows `OVR`.
* Below VNE, a rising prediction at or above VNE makes the actual-speed digits yellow.
* A new frame resets colors when the corresponding condition or required data is absent.
* Profile loss removes both VNE warnings. Acceleration loss removes only the predictive warning.

All new content stays inside the existing IAS allocation. GS and central attitude geometry retain their positions.

## Delivery boundaries

* Each implementation issue updates its tests, documentation, and mobile evidence.
* Update embedded Rust asset routes and the fixture server when imports change.
* The full development fixture uses the implemented IAS/GS module. Remaining instrument illustrations stay development-only until their feature slices arrive.
* No new telemetry fields, framework, build pipeline, Solid.js, three.js, or custom signals are required for #36 or #37.
