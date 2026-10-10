# PFD presentation

Target module boundaries, panel composition, and rendering lifecycle.
System context: [architecture](architecture.md). Decision: [ADR 0004](adr/0004-modular-pfd-presentation.md).
Delivery order: [roadmap](roadmap.md). Validation: [testing](testing.md#modular-and-layered-pfd-acceptance).

## Current implementation and planned changes

The current panel implements attitude, slip/skid, and turn rate from #20.
`pfd-view.js` builds their SVG geometry and updates their elements.
`horizon.js` combines attitude transforms with frame scheduling.
`app.js` connects telemetry, layout, status presentation, and page lifecycle.

* [#36](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/36) introduces the modules and panel interface below without changing the image.
* [#37](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/37) introduces the layered composition after that refactor.
* Later feature issues add their instrument modules with real demo data.

> The interfaces and layers below are planned, not implemented by this documentation change.

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

Sky/ground and the pitch scale can remain internal parts of the horizon module.
A module does not require one file per graphic element.
Add future instruments in their feature issues, without empty implementations or production placeholder readings.

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

Use local instrument coordinates and named dimensions.
Keep telemetry units at the input and conversion to display coordinates inside the instrument.
Preserve envelope validation and sample-age rules in the telemetry client.
Use source-supplied trend data based on source time, not differences between browser arrival times.
Layout calculations do not mutate telemetry or impose a fixed aspect ratio.
CSS owns page composition and compact source/transport status.
Share helpers only when an implemented instrument and the next agreed feature need them.
Do not introduce a general rendering framework or a second renderer.

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

`frame` is an internal presentation value, not a new SSE schema.
It combines one accepted sample, its original monotonic expiry deadline, and the applicable status/availability information.
Future feature issues define their own additions to this value and the wire contract separately.

The final composition has this shape; only implemented instruments participate at each delivery step:

```js
function renderPanel(frame) {
  horizon.render(frame.attitude);
  airspeedTape.render(frame.airspeed);
  altitudeTape.render(frame.altitude);
  verticalSpeed.render(frame.verticalSpeed);
  direction.render(frame.direction);
  slipSkid.render(frame.slipSkid);
  turnRate.render(frame.turnRate);
  status.render(frame.status);
}
```

Fixed symbols are built at initialization and positioned during resize.
The panel coordinator contains no scale formulas or SVG path construction.
Instrument calculations are pure; applying their results to SVG changes DOM state.

1. Create instrument elements once during initialization.
2. On an accepted sample, retain that sample and request at most one animation frame.
3. On resize, update geometry and schedule a redraw of the latest sample without changing its deadline.
4. Before drawing, check the original deadline and render only a fresh frame.
5. On invalidation, the scheduler cancels pending drawing and invokes `panel.invalidate(reason)` immediately.
6. After reconnect or resume, wait for a new valid sample before restoring indications.

Transport expiry detection continues even when no new samples arrive.
Fresh samples with unchanged numeric values still renew freshness.
Status changes remain visible when no valid instrument frame exists.
Optional failures affect only their indications; source/transport loss invalidates the panel.
Use no interpolation or extrapolation.

Selected-altitude alert timing belongs to #24.
Timed visual changes use the same panel scheduling path and an injectable monotonic clock.
Invalidation cancels their pending work and prevents an apparently valid alert from remaining active.

## Layers and clipping

| Layer | Content | Opacity and clipping |
| --- | --- | --- |
| 0 | Sky, ground, horizon line, pitch scale | Opaque background behind side tapes; pitch markings have a separate central clip |
| 1 | Airspeed tape, ticks, ranges, readout | Translucent background; readable digits, ticks, and value window |
| 2 | Altitude tape and VSI | Translucent background; readable indications and value windows |
| 3 | Fixed aircraft reference, other scales and indications | Transparent space around symbols |
| 4 | Warnings and unavailability | Cover the affected invalid indications, including the expanded horizon on global loss |

Apply opacity to background shapes, not whole instrument groups.
Select and record the background opacity in #37 through mobile visual acceptance.
No numeric opacity value is prescribed by this architecture.

A module can own elements in several layers.
Instrument-specific warning geometry remains with its instrument, even when placed in a warning container.
Fixed screen-space clips must not rotate with the world.
Pitch translation remains nested inside bank rotation.
Transform only moving elements and preserve rotation centers during resize.
Size text, symbols, and scales independently without distorting their proportions.

Separate background coverage from the central area reserved for readable attitude symbols.
Intentional background overlap is allowed; collisions between digits, ticks, and unrelated symbols are not.
Check painted extents, including stroke clearance, rather than only allocated rectangles.
Use the full 18-element development fixture before later instruments reduce usable space.
Preserve responsive sizing, safe areas, and the minima in the [brief](project-brief.md#mobile-presentation).

## Delivery boundaries

Each implementation issue updates its tests, documentation, and mobile evidence.
The modular refactor also updates embedded Rust asset routes and the fixture server when imports change.
The layer change uses development-only tape fixtures until #21 and #23 deliver their instruments.
No new telemetry fields, framework, build pipeline, Solid.js, three.js, or custom signals are required for #36 or #37.
