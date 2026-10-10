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
| `status.js` | Source badge, accessible status, and global unavailability |
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
CSS owns page composition and instrument background opacity.
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
The implemented frame contains `attitude`, `slip_skid`, `turn_rate_dps`, `expiresAt`, and a copied `status` value.
The status value contains `source`, `transport`, and `state`.
Every instrument receives the same frame object and treats it, including nested values, as read-only.
Future feature issues define additions to this value and the wire contract separately.

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

Fixed symbols are built at initialization and positioned during resize.
Every render method receives the same complete frame as read-only input.
Each module selects its required fields without mutating the frame or another module.
Adding a data dependency inside an instrument does not change its call in the panel coordinator.
Adding a new instrument still requires explicit composition in the panel.
The panel coordinator contains no scale formulas or SVG path construction.
Fixed symbols have no per-frame render method. Their geometry updates during resize.
Instrument calculations are pure; applying their results to SVG changes DOM state.

Horizon and fixed-symbol groups use local origins at the attitude center.
Slip/skid and turn rate use their own translated origins.
The attitude clip stays in panel coordinates. The pitch clip stays fixed relative to the horizon origin.
SVG nodes are created once. Resize updates attributes without replacing nodes.

1. Create instrument elements once during initialization.
2. On an accepted sample, retain that sample and request at most one animation frame.
3. On resize, update geometry and schedule a redraw of the latest sample without changing its deadline.
4. Before drawing, check the original deadline and render only a fresh frame.
5. On invalidation, the scheduler cancels pending drawing and invokes `panel.invalidate(reason)` immediately.
6. After reconnect or resume, wait for a new valid sample before restoring indications.

Transport expiry detection continues even when no new samples arrive.
Fresh samples with unchanged numeric values still renew freshness.
Unavailable reasons remain visible on the opaque full-panel overlay when no valid instrument frame exists.
`status.update(status)` updates labels directly from the telemetry callback without exposing old readings.
The application passes the same status module instance to the panel.
Only a fresh scheduled render makes the panel available.
Scheduler expiry also updates the visible stale reason when the transport timer has not yet executed.
Optional failures affect only their indications; source/transport loss invalidates the panel.
Use no interpolation or extrapolation.

Selected-altitude alert timing belongs to #24.
Timed visual changes use the same panel scheduling path and an injectable monotonic clock.
Invalidation cancels their pending work and prevents an apparently valid alert from remaining active.

## Layers and clipping

| Layer | Content | Opacity and clipping |
| --- | --- | --- |
| 0 | Sky, ground, horizon line, pitch scale | Opaque background covering the entire SVG; pitch markings have a separate central clip |
| 1 | Airspeed tape, ticks, ranges, readout | Translucent background; readable digits, ticks, and value window |
| 2 | Altitude tape and VSI | Translucent background; readable indications and value windows |
| 3 | Fixed aircraft reference, other scales and indications | Transparent space around symbols |
| 4 | Warnings and unavailability | Cover the affected invalid indications, including the expanded horizon on global loss |

Apply opacity to background shapes, not whole instrument groups.
The named `--instrument-background-opacity` token starts at `0.65`. Compare `0.50`, `0.65`, and `0.80` on the development fixture. Select and record the final value in #37 through real iOS Safari and Android Chrome visual acceptance.
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

## Source badge and unavailable status

The panel has no top status bar or reserved header space.
A small `DEMO` badge appears in the lower left above the future ground-speed indication only for the demo source.
The development fixture uses a two-line `LAYOUT FIXTURE` badge at the same position.
Successful source and transport messages have no visible labels.
Source and transport remain separate internal states and are announced through a polite live region only when their text changes.
Waiting, reconnecting, stale, suspended, paused, disconnected, and invalid states obscure the entire panel with the applicable reason.
A `live` status alone does not remove this cover; only rendering a fresh frame does so.

`layout.background` covers the full SVG; `layout.attitude` defines the central reference area.
The horizon owns sky/ground in layer 0, the bank pointer in layer 3, and chevrons in layer 4.
Background, attitude, and pitch clips remain fixed while the world rotates and translates.
The development fixture puts future tape and heading elements in the same layer containers that their later modules will receive.

## Delivery boundaries

Each implementation issue updates its tests, documentation, and mobile evidence.
The modular refactor also updates embedded Rust asset routes and the fixture server when imports change.
The layer change uses development-only tape fixtures until #21 and #23 deliver their instruments.
No new telemetry fields, framework, build pipeline, Solid.js, three.js, or custom signals are required for #36 or #37.
