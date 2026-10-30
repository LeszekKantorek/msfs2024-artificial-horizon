# EFI feature guide

The compact electronic flight instrument (EFI) combines attitude, speed, altitude, heading, and coordinated-turn indications in one responsive, read-only panel.
The artificial horizon is its attitude indication; the [project terminology](../project-brief.md#terminology) defines the instrument terminology, including the `pfd` technical identifier.
This guide groups each feature's requirements, presentation, implementation, rationale, and tests on one page.

## Extension scope

| Scope or decision | Source |
| --- | --- |
| Product scope | [Project brief](../project-brief.md) |
| Original Demo extension decision | [ADR 0003](../adr/0003-responsive-pfd-demo.md) |
| Current SimConnect optional instruments | [SimConnect observations](../sources/simconnect.md#optional-instrument-observations) |
| Deterministic Demo scenarios and source-selected values | [Demo requirements](../sources/demo.md#requirements-and-usage) |
| Additive fields, validity, fixtures, and attitude-only compatibility | [Telemetry extension contract](../telemetry-contract.md#further-efi-extension-boundary) |

EFI extensions preserve these scope limits:

* Demo extensions add no SDK requirements.
* Performance #7 and packaging #8 remain separate work.
* Source data and browser presentation use the same normalized contract for Demo and SimConnect.

## Shared responsibilities

Read only the shared boundary affected by a change:

| Boundary | Documentation | Use when |
| --- | --- | --- |
| Module composition | [Composition and implementation](composition.md#panel-composition) | Adding an instrument or changing module responsibilities |
| Internal composition | [Internal module seams](composition.md#internal-module-seams) | Changing private nodes, shared hosts/clips, or concrete child inputs |
| Calculations and helpers | [Pure calculations and helpers](composition.md#pure-calculations-and-helpers) | Changing calculation ownership, shared consumers, or DOM-free imports |
| Asset integration | [Delivery boundaries](composition.md#delivery-boundaries) | Adding future instruments or changing embedded browser imports |
| Frames and scheduling | [Lifecycle](lifecycle.md#panel-interface-and-frame-lifecycle) | Changing sample acceptance, expiry, resize, timed alerts, or invalidation |
| Browser telemetry | [Telemetry client](telemetry.md#telemetry-operations) | Changing normalization, publication callbacks, or connection ownership |
| Global status | [Status module](status.md) | Changing unavailable presentation or accessible announcements |
| Layout and drawing | [Rendering](rendering.md) | Changing panel layers, screen-space clips, opacity, or cross-module drawing order |
| Attitude transforms | [World transforms](attitude.md#world-transforms) | Changing pitch translation, bank rotation, or their relation to fixed clips |
| Validation | [Testing guide](../testing/index.md) | Selecting automated, desktop, and actual-phone checks |

Shared presentation ownership follows these rules:

* The panel passes one complete read-only frame to each instrument.
* Instrument compositions select fields and delegate to visual objects.
* Objects own their geometry and calculations.
* The common scheduler owns retained frames and timing.
* The telemetry client owns the connection and freshness.

| Cross-component boundary or decision | Source |
| --- | --- |
| System integration and delivery constraints | [Browser boundary](../architecture.md#browser-boundary) |
| Instrument composition and layers | [ADR 0004](../adr/0004-modular-pfd-presentation.md) |
| Visual-object composition and calculation ownership | [ADR 0005](../adr/0005-visual-object-composition.md) |
| Delivery order | [Roadmap](../roadmap.md) |

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
| Wire fields, normalization, units, and compatibility | [Telemetry contract](../telemetry-contract.md) |
| Source scenario definitions and timing | [Demo provider](../sources/demo.md) |
| Product coverage and exclusions | [Project brief](../project-brief.md#scope-and-acceptance) |
| Execution results and acceptance status | Related GitHub issues and PRs |

Read the feature together with its linked wire contract when changing inputs.
Apply shared lifecycle and rendering rules to every feature without copying those rules into each page.
Shared validation procedures supplement feature checks; they do not replace them.
Keep local constants, SVG state, paint order, demo inputs, and regression scenarios on the owning module or feature page.
Shared pages describe cross-module contracts and reusable procedures, with links to local details.

Implementation descriptions and validation procedures do not establish passing device acceptance.
Record actual iOS Safari and Android Chrome evidence separately in the responsible feature issue.
