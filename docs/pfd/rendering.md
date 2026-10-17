# Panel layout and layers

Panel-wide geometry, layer allocation, cross-module paint order, and shared SVG/CSS rules.

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
* Follow the [opacity comparison procedure](../testing/pfd.md#modular-and-layered-pfd-acceptance) on real iOS Safari and Android Chrome.

> This architecture does not prescribe a final numeric opacity value.

Local clip geometry and origins are documented in each feature's Implementation section.

Layer ownership, clipping, and transforms follow these rules:

* A module can own elements in several layers.
* Instrument-specific warning geometry remains with its instrument.
* Transform only moving elements and preserve rotation centers during resize.
* Size text, symbols, and scales independently without distorting their proportions.
* The development fixture uses the same instrument modules and layer containers as production.

Keep indications readable across the full panel:

* Separate background coverage from the central area reserved for readable attitude symbols.
* Intentional background overlap is allowed.
* Collisions between digits, ticks, and unrelated symbols are not allowed.
* Check painted extents, including stroke clearance, rather than only allocated rectangles.
* Use the full 17-element development fixture when checking instrument geometry.
* Preserve responsive sizing, safe areas, and the minima in the [brief](../project-brief.md#mobile-presentation).

## SVG and CSS interface

IDs, classes, node order, layer placement, clipping, and accessible meanings form the presentation interface.
[Layer rules](#layers-and-clipping) define clip coordinates and transform order.

SVG nodes retain their identity through samples, loss, recovery, and resize.
The following orders connect multiple modules. Each feature documents its internal node order.

| Scope | Order |
| --- | --- |
| Definitions | Background clip, attitude clip, pitch clip, IAS tape clip, digit clip, barber pattern |
| Symbols layer | Bank pointer, fixed symbols, heading, slip/skid, turn rate |
| Warnings layer | Pitch warnings, IAS warning, GS warning, slip warning, turn warning |

Local state attributes and their CSS effects belong to the feature that writes them.

The [status module](status.md) owns global visibility and announces only changed status text.
