# EFI documentation guide

The compact electronic flight instrument (EFI) combines attitude, speed, altitude, heading, and coordinated-turn indications in one responsive, read-only panel.
The artificial horizon is its attitude indication.
The [project terminology](../project-brief.md#terminology) defines these terms and the `pfd` technical identifier.

Each feature page groups its requirements, presentation, implementation, testing, and rationale.
Shared pages define contracts used across instruments.

## Features

Module paths are relative to `web/`.

| Feature | Scope | Modules | Related documentation |
| --- | --- | --- | --- |
| [Attitude and coordinated turn](attitude.md) | Horizon, fixed references, slip/skid, turn rate | `horizon/`, `fixed-symbols/`, `slip-skid.js`, `turn-rate.js`  | [Rendering](rendering.md), [world transforms](attitude.md#world-transforms) |
| [IAS and GS](airspeed.md) | Speed tape, rolling digits, GS, composition | `airspeed/index.js`, `airspeed/ias.js`, `airspeed/readout.js`, `airspeed/ground-speed.js`  | [TAS](true-airspeed.md), [speed indications](airspeed-indications.md) |
| [TAS](true-airspeed.md) | Independent top-left speed readout | `airspeed/true-airspeed.js`  | [IAS allocation](airspeed.md) |
| [Speed ranges, references, and trend](airspeed-indications.md) | Profile bands, V-speeds, prediction, VNE | `airspeed/tape.js`, `airspeed/references.js`, `airspeed/ias.js`  | [IAS and GS](airspeed.md) |
| [Altitude and BARO](altitude.md) | Signed tape, drums, pressure, composition | `altitude/index.js`, `altitude/tape.js`, `altitude/readout.js`, `altitude/baro.js`  | [Selected altitude](selected-altitude.md), [VSI](vertical-speed.md) |
| [Selected altitude and alerts](selected-altitude.md) | Target, notched bug, approach/deviation state | `altitude/selected.js`  | [Altitude](altitude.md), [lifecycle](lifecycle.md) |
| [Vertical speed](vertical-speed.md) | Independent compact ft/min readout | `altitude/vsi.js`  | [Altitude](altitude.md), [altitude trend](altitude-trend.md) |
| [Altitude trend](altitude-trend.md) | Source-VSI prediction and overlap | `altitude/trend.js`  | [VSI](vertical-speed.md), [selected bug](selected-altitude.md) |
| [Current and selected heading](heading.md) | Heading scale, readout, selected bug | `heading.js`  | [Shared layout](rendering.md#layout-and-readable-regions) |

## Shared responsibilities

Select the shared boundary affected by a change:

| Need | Documentation |
| --- | --- |
| Add an instrument or change module responsibilities | [Component responsibilities](composition.md#component-responsibilities), [instrument composition](composition.md#instrument-composition) |
| Change private nodes, shared hosts and clips, or concrete child inputs | [Visual object interfaces and ownership](composition.md#visual-object-interfaces-and-ownership) |
| Change calculation ownership, shared consumers, or DOM-free imports | [Calculations and shared helpers](composition.md#calculations-and-shared-helpers) |
| Change embedded browser imports or feature integration | [Asset integration and extension constraints](composition.md#asset-integration-and-extension-constraints) |
| Change frame fields or panel operations | [Normalized frame contract](lifecycle.md#normalized-frame-contract), [panel interface](lifecycle.md#panel-interface) |
| Change sample acceptance, resize, or drawing | [Sample acceptance and drawing](lifecycle.md#sample-acceptance-and-drawing) |
| Change timed alerts, cancellation, expiry, or recovery | [Timed visual updates](lifecycle.md#timed-visual-updates), [invalidation and recovery](lifecycle.md#invalidation-and-recovery) |
| Change browser normalization or callback publication | [Snapshot validation](telemetry.md#snapshot-validation-and-optional-normalization), [sample publication](telemetry.md#sample-acceptance-and-callback-publication) |
| Change connection ownership, freshness, or callback lifecycle behavior | [Connection lifecycle and freshness](telemetry.md#connection-lifecycle-and-freshness), [callback reentrancy limits](telemetry.md#callback-reentrancy-limits) |
| Change global unavailability or accessible announcements | [Status module](status.md) |
| Change layers, screen-space clips, or opacity | [Layer allocation](rendering.md#layer-allocation), [clipping and transforms](rendering.md#clipping-and-transforms), [background opacity](rendering.md#background-opacity) |
| Change cross-module drawing order | [Cross-module paint order](rendering.md#cross-module-paint-order) |

Presentation components retain these responsibilities:

| Component | Responsibility |
| --- | --- |
| Panel | Pass one complete read-only frame to each instrument |
| Instrument composition | Select fields and delegate to visual objects |
| Visual object | Own geometry and calculations |
| Common scheduler | Own retained frames and timing |
| Telemetry client | Own the connection and freshness |

## Scope and governing decisions

| Subject | Governing source | Defines |
| --- | --- | --- |
| Product scope | [Project brief](../project-brief.md) | Coverage and exclusions |
| System integration | [Browser boundary](../architecture.md#browser-boundary) | Integration and delivery constraints |
| Original Demo extension | [ADR 0003](../adr/0003-responsive-pfd-demo.md) | Original extension scope and decision |
| Instrument composition | [ADR 0004](../adr/0004-modular-pfd-presentation.md) | Instrument boundaries and layers |
| Visual-object composition | [ADR 0005](../adr/0005-visual-object-composition.md) | Object boundaries and calculation ownership |
| Current SimConnect instruments | [SimConnect observations](../sources/simconnect.md#optional-instrument-observations) | Available optional instruments and observation limits |
| Deterministic source behavior | [Demo requirements](../sources/demo.md#requirements-and-usage) | Deterministic scenarios and source-selected values |
| Telemetry extensions | [Extension contract](../telemetry-contract.md#further-efi-extension-boundary) | Additive fields, validity, fixtures, and attitude-only compatibility |
| Delivery order | [Roadmap](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) | Work sequence and dependencies |

EFI extensions preserve these scope limits:

* Demo extensions add no SDK requirements.
* Performance #7 and packaging #8 remain separate work.
* Source data and browser presentation use the same normalized contract for Demo and SimConnect.

## Content ownership

Documentation follows reader tasks rather than requiring a page for every JavaScript file.
Closely coupled objects share a feature page. Independently changed behavior has its own page.

| Information | Destination |
| --- | --- |
| User-visible requirements and limits | The feature's Requirements and Presentation sections |
| Feature SVG geometry, local state, calculations, constants, helpers, and module interfaces | The feature's Implementation section |
| Cross-feature ownership and shared helpers | [Composition](composition.md) |
| Browser normalization, callback publication, and connection internals | [Telemetry client](telemetry.md) |
| Normalized frames, panel interfaces, scheduling, and cancellation | [Frame lifecycle](lifecycle.md) |
| Shared layers, clipping, opacity, and paint order | [Rendering](rendering.md) |
| Global availability and status internals | [Status](status.md) |
| Rationale, accepted refinements, and regression lessons | The affected feature or shared document, with public provenance and evidence limits. ADRs own accepted architectural decisions |
| Feature scenarios, controlled Demo inputs, expected results, screenshots, and device-specific checks | The feature's Testing section |
| Reusable commands, phone setup, and evidence templates | [Shared testing pages](../testing/index.md) |
| Wire fields, normalization contract, units, and compatibility | [Telemetry contract](../telemetry-contract.md) |
| Source scenario definitions and timing | [Demo provider](../sources/demo.md) |
| Product coverage and exclusions | [Project brief](../project-brief.md#scope-and-acceptance) |
| Execution results and acceptance status | Related GitHub issues and PRs |

Apply these ownership rules when updating documentation:

* Read the feature together with its linked wire contract when changing inputs.
* Apply shared lifecycle and rendering rules to every feature without copying those rules into each page.
* Keep local constants, SVG state, paint order, controlled Demo inputs, and regression scenarios on the owning module or feature page.
* Keep cross-module contracts and reusable procedures on shared pages, with links to local details.

## Validation and evidence limits

| Need | Source |
| --- | --- |
| Select required checks | [Validation scope matrix](../testing/index.md#validation-by-change-scope) |
| Check one instrument | Its Testing section in the [feature directory](#features) |
| Check shared geometry and integration | [Shared EFI checks](../testing/pfd.md) |
| Perform real-phone acceptance | [Device procedure](../testing/devices.md#real-phone-acceptance-procedure) |
| Record results, missing checks, and evidence | [Evidence rules](../testing/evidence.md), with results in the responsible issue or PR |

* Shared validation procedures supplement feature checks. They do not replace them.
* Record actual iOS Safari and Android Chrome evidence separately in the responsible feature issue.

> Implementation descriptions and validation procedures do not establish passing device acceptance.
