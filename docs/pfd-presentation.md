# PFD presentation

Instrument modules, panel composition, rendering lifecycle, and SVG layers.
System context: [architecture](architecture.md). Decision: [ADR 0004](adr/0004-modular-pfd-presentation.md).
Delivery order: [roadmap](roadmap.md). Validation: [testing](testing.md#modular-and-layered-pfd-acceptance).

## Current implementation and planned changes

The panel implements attitude, slip/skid, and turn rate from #20.
#36 separates their rendering from frame scheduling:

| Module | Implemented responsibility |
| --- | --- |
| `app.js` | Assemble telemetry, status, panel, resize observation, and page lifecycle |
| `frame-scheduler.js` | Retain the latest frame, coalesce draws, check expiry, and cancel invalid work |
| `panel.js` | Compose instruments and delegate resize, render, and invalidation |
| `horizon.js` | Sky/ground, pitch scale, chevrons, and moving bank pointer |
| `fixed-symbols.js` | Fixed bank scale and aircraft references |
| `slip-skid.js`, `turn-rate.js` | Local geometry, readings, and indication availability |
| `status.js` | Accessible status, and global unavailability |
| `svg.js` | Shared SVG creation and attribute helpers |

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
|-- Altitude tape and selected-altitude alerts (#23, extended by #24)
|-- Vertical speed                       (#25)
|-- Heading and track                    (#26)
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
* Preserve envelope validation and sample-age rules in the telemetry client.
* Use source-supplied trend data based on source time, not differences between browser arrival times.
* Layout calculations do not mutate telemetry or impose a fixed aspect ratio.
* CSS owns page composition and instrument background opacity.
* Share helpers only when an implemented instrument and the next agreed feature need them.
* Do not introduce a general rendering framework or a second renderer.

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
| `expiresAt` | Original monotonic expiry deadline |
| `status` | Copied `source`, `transport`, and `state` values |

Instruments share the frame under these rules:

* Every instrument receives the same complete frame and selects its own fields without mutating the frame or another instrument.
* Nested frame values are also read-only.
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
* Only the development fixture uses a two-line `LAYOUT FIXTURE` badge, in the lower left above future ground speed.
* Successful source and transport messages have no visible labels.
* Source and transport states remain separate internally; a polite live region announces only changed text.
* Waiting, reconnecting, stale, suspended, paused, disconnected, and invalid states show an opaque full-panel cover with the applicable reason.
* Only rendering a fresh frame removes the cover; a `live` status alone does not.

## Delivery boundaries

* Each implementation issue updates its tests, documentation, and mobile evidence.
* Update embedded Rust asset routes and the fixture server when imports change.
* Tape fixtures remain development-only until #21 and #23 deliver their instruments.
* No new telemetry fields, framework, build pipeline, Solid.js, three.js, or custom signals are required for #36 or #37.
