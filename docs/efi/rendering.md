# Panel layout and layers

Panel-wide geometry, layer allocation, cross-module paint order, and shared SVG/CSS rules.

## Layout and readable regions

* Separate background coverage from the central area reserved for readable attitude symbols.
* Intentional background overlap is allowed.
* Collisions between digits, ticks, and unrelated symbols are not allowed.
> No fixed aspect ratio, including 4:3. Adapt geometry to available width and height.

| Area | Layout rule |
| --- | --- |
| Portrait | Attitude center, IAS left, altitude right. Compact VSI below altitude. Altitude trend over the left ticks and below the selected-altitude bug. Heading above |
| Landscape | Use extra width and reduced height. Retain familiar instrument positions |
| Instrument sizing | Size digits, symbols, tapes, and horizon independently. Preserve undistorted symbols and readable central attitude |
| Page chrome | Minimize bezel, headings, and margins |
| Status | Do not show a source badge. Show unavailable reasons on the full-panel cover. Retain accessible source/transport state |
| Viewport | Account for safe areas, cutouts, and browser bars. Fit without scrolling from 320 CSS px in both orientations |
| Minimum usable height | 240 CSS px after browser bars and safe areas, including short landscape |


* Target iOS Safari and Android Chrome in portrait and landscape.
* The first slice (#20) required assessing all intended elements with development-only fixtures before later additions reduced horizon space.

## Layer allocation

| Layer | Content | Opacity and clipping |
| --- | --- | --- |
| 0 | Sky, ground, horizon line, pitch scale | Opaque full-SVG background, with a separate central clip for pitch markings |
| 1 | Airspeed tape, ticks, ranges, readout | Translucent background with readable digits, ticks, and value window |
| 2 | Altitude tape and VSI | Translucent background with readable indications and value windows |
| 3 | Fixed aircraft reference, other scales and indications | Transparent space around symbols |
| 4 | Warnings and unavailability | Cover the affected invalid indications, including the expanded horizon on global loss |

* The opaque sky/ground background fills side, heading, and supplemental regions as well as the central area.
* A module can own elements in several layers.
* Instrument-specific warning geometry remains with its instrument.

## Clipping and transforms

* Transform only moving elements and preserve rotation centers during resize.
* Size text, symbols, and scales independently without distorting their proportions.
* Use each feature's Implementation section for local clip geometry and origins.

The [attitude geometry](attitude.md#implementation) defines horizon transforms and clipping.
Other instrument pages own their local geometry and internal paint order.

## Background opacity

* Apply opacity to background shapes, not whole instrument groups.
* `--instrument-background-opacity` defaults to `0.65` pending real-phone acceptance in #37.
* CSS owns page composition and instrument background opacity.

> This architecture does not prescribe a final numeric opacity value. Final background opacity acceptance requires real-phone evidence.

## Cross-module paint order

The following orders connect multiple modules. Each feature documents its internal node order.

| Scope | Order |
| --- | --- |
| Definitions | Background clip, attitude clip, pitch clip, IAS tape clip, digit clip, barber pattern, altitude tape clip, altitude trend clip, altitude digit clip, heading tape clip |
| Symbols layer | Bank pointer, fixed symbols, heading, slip/skid, turn rate |
| Warnings layer | Pitch warnings, IAS warning, GS warning, altitude warning, slip warning, turn warning |

## SVG and CSS interface

* IDs, classes, node order, layer placement, clipping, and accessible meanings form the presentation interface.
* SVG nodes retain their identity through samples, loss, recovery, and resize.
* Local state attributes and their CSS effects belong to the feature that writes them.
* The [status module](status.md) owns global visibility and announces only changed status text.

## Validation and evidence limits

### Painted bounds and clipping checks

* Check painted extents, including stroke clearance, rather than only allocated rectangles.
* Check background clipping with browser hit testing and a narrowed-clip negative check because path geometry excludes ancestor clips.
* Distinguish intentional background overlap from collisions between painted indications.

Use the [geometry and paint regression checks](../testing/pfd.md#geometry-and-paint-regression-checks).

### Production fixture scope

* The development fixture uses the same instrument modules and layer containers as production.
* Use the full 17-element development fixture when checking instrument geometry.

The [heading fixture reference](heading.md#full-panel-integration) records the 17 illustration IDs and the command to count them.
A declared item count does not establish painted coverage or visual acceptance.

### Real-phone opacity acceptance

Follow the [opacity comparison procedure](../testing/pfd.md#modular-and-layered-efi-acceptance) on real iOS Safari and Android Chrome.
Record real-device acceptance evidence in the owning GitHub issue.

> Keep real-device evidence separate from desktop checks. Implementation and source review do not establish final validation or actual phone acceptance.

## Composition provenance

* #36 separated existing modules without changing the image.
* #37 introduced layered composition without a top status bar.
* Those steps added no telemetry fields or functional tape readings. Background opacity remains subject to mobile acceptance.
