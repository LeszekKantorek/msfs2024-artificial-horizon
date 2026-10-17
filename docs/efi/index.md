# EFI feature guide

The compact electronic flight instrument (EFI) combines attitude, speed, altitude, heading, and coordinated-turn indications in one responsive, read-only panel.
The artificial horizon is its attitude indication; the [project terminology](../project-brief.md#terminology) distinguishes the instrument from its historical PFD naming.
This guide groups each feature's requirements, presentation, implementation, rationale, and tests on one page.

## Shared responsibilities

Read only the shared boundary affected by a change:

| Boundary | Documentation | Use when |
| --- | --- | --- |
| Module composition | [Composition and implementation](composition.md) | Adding an instrument, changing object ownership, or sharing calculations |
| Frames and scheduling | [Lifecycle](lifecycle.md) | Changing sample acceptance, expiry, resize, timed alerts, or invalidation |
| Browser telemetry | [Telemetry client](telemetry.md) | Changing normalization, publication callbacks, or connection ownership |
| Global status | [Status module](status.md) | Changing unavailable presentation or accessible announcements |
| Layout and drawing | [Rendering](rendering.md) | Changing panel layers, clip boundaries, opacity, or cross-module drawing order |
| Validation | [Testing guide](../testing/index.md) | Selecting automated, desktop, and actual-phone checks |

The panel passes one complete read-only frame to each instrument.
Instrument compositions select fields and delegate to visual objects.
Objects own their geometry and calculations.
The common scheduler owns retained frames and timing.
The telemetry client owns the connection and freshness.

System boundaries remain in [architecture](../architecture.md).
[ADR 0004](../adr/0004-modular-pfd-presentation.md) and [ADR 0005](../adr/0005-visual-object-composition.md) explain instrument and visual-object composition.
The [roadmap](../roadmap.md) defines delivery order.

## Features

Documentation follows reader tasks rather than requiring a page for every JavaScript file.
Closely coupled objects share a feature page; independently changed behavior has its own page.

| Feature | Scope and implementing modules | Related feature |
| --- | --- | --- |
| [Attitude and coordinated turn](attitude.md) | Horizon, fixed references, slip/skid, turn rate; `horizon/`, `fixed-symbols/`, `slip-skid.js`, `turn-rate.js` | Shared rendering |
| [IAS and GS](airspeed.md) | Speed tape, rolling digits, GS, composition; `airspeed/index.js`, `ias.js`, `readout.js`, `ground-speed.js` | TAS and speed indications |
| [TAS](true-airspeed.md) | Independent top-left speed readout; `airspeed/true-airspeed.js` | IAS allocation |
| [Speed ranges, references, and trend](airspeed-indications.md) | Profile bands, V-speeds, prediction, VNE; `airspeed/tape.js`, `references.js`, `ias.js` | IAS and GS |
| [Altitude and BARO](altitude.md) | Signed tape, drums, pressure, composition; `altitude/index.js`, `tape.js`, `readout.js`, `baro.js` | Selected altitude and VSI |
| [Selected altitude and alerts](selected-altitude.md) | Target, notched bug, approach/deviation state; `altitude/selected.js` | Altitude and shared lifecycle |
| [Vertical speed](vertical-speed.md) | Independent compact ft/min readout; `altitude/vsi.js` | Altitude and altitude trend |
| [Altitude trend](altitude-trend.md) | Source-VSI prediction and overlap; `altitude/trend.js` | VSI and selected bug |
| [Current and selected heading](heading.md) | Heading scale, readout, selected bug; `heading.js` | Shared layout |

## Content ownership

| Information | Destination |
| --- | --- |
| User-visible requirements and limits | The feature's Requirements and Presentation sections |
| Feature SVG geometry, local state, calculations, constants, helpers, and module interfaces | The feature's Implementation section |
| Cross-feature ownership, panel interfaces, scheduling, and shared helpers | Shared boundary pages; module internals belong on the telemetry or status page |
| Rationale, accepted refinements, regression lessons | The affected feature or shared validation page, with public provenance and evidence limits |
| Feature scenarios, expected results, screenshots, device-specific checks | The feature's Testing section |
| Reusable commands, phone setup, evidence templates | Shared testing pages |
| Wire fields, normalization, units, source demo schedules | [Telemetry contract](../telemetry-contract.md) |
| Product coverage and exclusions | [Project brief](../project-brief.md#scope-and-acceptance) |
| Execution results and acceptance status | Related GitHub issues and PRs |

Read the feature together with its linked wire contract when changing inputs.
Apply shared lifecycle and rendering rules to every feature without copying those rules into each page.
Shared validation procedures supplement feature checks; they do not replace them.
Keep local constants, SVG state, paint order, demo inputs, and regression scenarios on the owning module or feature page.
Shared pages describe cross-module contracts and reusable procedures, with links to local details.

Implementation descriptions and validation procedures do not establish passing device acceptance.
Record actual iOS Safari and Android Chrome evidence separately in the responsible feature issue.
