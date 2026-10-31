# Panel composition and implementation

Instrument ownership, shared helpers, public operations, and delivery boundaries.

## Component responsibilities

| Component | Module or boundary | Responsibility |
| --- | --- | --- |
| Application assembly | `app.js` | Assemble telemetry, status, panel, layout, resize observation, and page lifecycle |
| Telemetry client | `telemetry/client.js` | Validate messages and own EventSource, sequence handling, retries, expiry, source/transport status, and start/stop |
| Validation | `telemetry/validation.js` | Pure envelope validation and independent optional-value normalization |
| Frame scheduler | `frame-scheduler.js` | Retain the latest frame, coalesce draws, check expiry, and cancel invalid work |
| Panel | `panel.js` | Compose instruments, assign layers, own and resize the common attitude clip, and delegate resize, acceptance, rendering, and invalidation |
| Layout | `layout.js` | Allocate CSS-pixel regions, including the background extent and central readable area |
| Instrument composition | Feature entry points | Select frame fields and delegate concrete geometry and readings to visual objects |
| Visual object | Feature objects | Own nodes, geometry, calculations, and local availability |
| Fixed symbols | `fixed-symbols/` | Draw references that do not move with the world |
| Status | `status.js` | Present accessible source/transport status and global unavailability without owning the connection |
| Shared helpers | `helpers.js` | Named clamp, SVG creation, and attribute updates without import-time DOM access |

Slip/skid, turn rate, layout, status, and scheduler remain small standalone modules.
The [telemetry client](telemetry.md#connection-lifecycle-and-freshness) enforces sample age even without new samples.
The [frame lifecycle](lifecycle.md) defines scheduling and cancellation.

## Instrument composition

The panel presents these indications:

* Attitude, slip/skid, and turn rate.
* IAS, GS, TAS, speed ranges, and trend.
* Altitude, pressure, selected-altitude alerts, and vertical speed.
* Heading and the selected-heading bug.

```text
Panel
|-- Horizon
|-- Airspeed
|-- Altitude
|-- Heading
|-- Slip/skid
|-- Turn rate
|-- Fixed symbols
`-- Status and data availability
```

* A module does not require one file per graphic element.
* Adding an instrument requires explicit composition in the panel.
* Adding a data dependency inside an instrument does not change its call in the panel.

Data conversion and rendering follow these rules:

* Use local instrument coordinates and named dimensions.
* Keep telemetry units at the input and conversion to display coordinates inside the instrument.
* Keep pure envelope validation in `telemetry/validation.js` and sample-age enforcement in the telemetry client.
* Use source-supplied trend data based on source time, not differences between browser arrival times.
* Layout calculations do not mutate telemetry or impose a fixed aspect ratio.
* CSS owns page composition and instrument background opacity.
* Share helpers for matching rules with actual callers, such as mathematical `clamp` and SVG attribute updates.
* Do not introduce a general rendering framework or a second renderer.

## Visual object interfaces and ownership

Each feature documents its factory interface in its Implementation section.

Internal objects use these ownership rules:

* Instrument entry points receive the complete read-only frame.
* Internal objects receive concrete SVG hosts, geometry, and the readings their operations require.
* Factories select existing owned nodes or create dynamic nodes once.
* Returned objects expose operations without public node bags.
* Parents update their own shared hosts and clips before invoking child operations.
* Visual closures retain nodes and initialized geometry without a latest-frame cache or timer.
* The frame scheduler owns its timing work.

## Calculations and shared helpers

### Object-specific calculations

Object-specific calculations remain in the modules that implement their objects.
Each feature's Implementation section documents its calculations, constants, and local helpers.

* Named pure exports support direct behavioral tests in Node without DOM.
* Purity does not require a separate calculation file.
* Feature-local helpers belong with their feature and serve actual shared rules.

### Shared helper contracts

`helpers.js` exposes these operations:

| Operation | Contract |
| --- | --- |
| `clamp(value, min, max)` | Clip infinities and preserve `NaN` propagation without normalizing invalid telemetry |
| `svgElement(name, attributes, text)` | Create an SVG element when called, with attributes and optional text |
| `setAttributes(element, values)` | Update named attributes on an existing element |

DOM access occurs when SVG creation executes, never during module import.
The [visual-object decision](../adr/0005-visual-object-composition.md) records this mixed-helper compromise.

## Asset integration and extension constraints

* Add instruments within the accepted product scope with deterministic Demo data, without empty implementations or production placeholder readings.
* Update tests, documentation, and applicable mobile evidence when changing an instrument.
* Update embedded Rust asset routes and the fixture server when imports change. The [server reference](../server.md#embedded-assets) defines asset registration constraints.
* Use the [production fixture scope](rendering.md#production-fixture-scope) for full-panel checks.
* Retain plain JavaScript, SVG, and SSE without an application framework, frontend build pipeline, or custom signals system.
* Composition changes do not establish telemetry changes. Field extensions follow the [telemetry contract](../telemetry-contract.md#further-efi-extension-boundary) and accepted feature scope.

## Rationale and regression lessons

* Keep object-specific calculations with their visual owner because pure behavior and DOM-free tests do not require a separate file.
* Extract shared rules only for actual independent consumers, not a planned instrument or a test-only file boundary.
* Preserve [coherent geometry and fresh readings](lifecycle.md#coalesced-frame-rendering) in one animation callback, including pending geometry under the unavailable cover.
* Apply the [painted-bound and clipping checks](rendering.md#painted-bounds-and-clipping-checks) to distinguish background overlap from indication collisions.
* Keep real-device acceptance evidence separate from desktop checks. GitHub issues own real-device acceptance evidence.

> Implementation and source review do not establish final validation or real iOS Safari and Android Chrome acceptance.
