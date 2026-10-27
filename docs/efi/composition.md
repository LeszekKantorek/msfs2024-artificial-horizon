# Panel composition and implementation

Instrument ownership, shared helpers, public operations, and delivery boundaries.

## Panel composition

The panel presents attitude, slip/skid, turn rate, IAS, GS, TAS, speed ranges and trend, altitude, pressure, selected-altitude alerts, vertical speed, heading, and the selected-heading bug.
Rendering and scheduling have separate responsibilities. Visual objects own their object-specific calculations:

| Module | Responsibility |
| --- | --- |
| `app.js` | Assemble telemetry, status, panel, resize observation, and page lifecycle |
| `telemetry/client.js` | Own EventSource, sequence handling, retries, expiry, status, and start/stop lifecycle |
| `telemetry/validation.js` | Pure snapshot validation and independent optional-value normalization |
| `frame-scheduler.js` | Retain the latest frame, coalesce draws, check expiry, and cancel invalid work |
| `panel.js` | Compose instruments, own the common attitude clip, and delegate resize/render/invalidation |
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
|-- Airspeed
|-- Altitude
|-- Heading
|-- Slip/skid
|-- Turn rate
|-- Fixed symbols
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

Slip/skid, turn rate, layout, status, and scheduler remain small standalone modules.

## Internal module seams

Each feature documents its factory interface in its Implementation section.

Internal objects use these ownership rules:

* Instrument entry points receive the complete read-only frame.
* Internal objects receive concrete SVG hosts, geometry, and the readings their operations require.
* Factories select existing owned nodes or create dynamic nodes once.
* Returned objects expose operations without public node bags.
* Parents update their own shared hosts and clips before invoking child operations.
* Visual closures retain nodes and initialized geometry without a latest-frame cache or timer.
* The frame scheduler owns its timing work.

## Pure calculations and helpers

Object-specific calculations remain in the modules that implement their objects.
Each feature's Implementation section documents its calculations, constants, and local helpers.
Shared ownership rules apply across features:

* Named pure exports support direct behavioral tests in Node without DOM.
* Purity does not require a separate calculation file.
* Feature-local helpers belong with their feature and serve actual shared rules.

The mixed helper module preserves its import boundary:

* `helpers.js` exports `clamp(value, min, max)`, `svgElement(name, attributes, text)`, and `setAttributes(element, values)`.
* Clamp clips infinities and preserves `NaN` propagation without normalizing invalid telemetry.
* DOM access occurs when SVG creation executes, never during module import.

The [visual-object decision](../adr/0005-visual-object-composition.md) records this mixed-helper compromise.

## Delivery boundaries

Delivery preserves production scope and asset integration:

* Each implementation issue updates its tests, documentation, and mobile evidence.
* Update embedded Rust asset routes and the fixture server when imports change.
* The full development fixture uses production modules for all 17 included illustration elements.
* No new telemetry fields, framework, build pipeline, Solid.js, three.js, or custom signals are required for #36 or #37.

## Rationale and regression lessons

Apply these composition and regression checks:

* Keep object-specific calculations with their visual owner because pure behavior and DOM-free tests do not require a separate file.
* Extract shared rules only for actual independent consumers, not a planned instrument or a test-only file boundary.
* Preserve coherent resize and fresh readings in one animation callback, including pending geometry under the unavailable cover.
* Check background clipping with browser hit testing and a narrowed-clip negative check because path geometry excludes ancestor clips.
* Distinguish intentional background overlap from collisions between painted indications.
* Keep real-device acceptance evidence separate from desktop checks.

Evidence limit: a source review does not establish final validation or real iOS Safari and Android Chrome acceptance.
