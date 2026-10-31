# EFI documentation guide

The compact electronic flight instrument (EFI) combines attitude, speed, altitude, heading, and coordinated-turn indications in one responsive, read-only panel.
The artificial horizon is its attitude indication.
The [project terminology](#terminology) defines these terms and the `pfd` technical identifier.

Each feature page groups its requirements, presentation, implementation, testing, and rationale.
Shared pages define contracts used across instruments.

## Terminology

* Electronic Flight Instrument (EFI) names the complete compact instrument in this project.
* Artificial horizon names its attitude indication and remains part of the application name.
* The intended presentation resembles a small standby instrument rather than a complete integrated cockpit display. This does not imply an independent backup data source.
* Primary Flight Display (PFD) describes a display role and is not exclusive to the Garmin G1000. Garmin describes the G5 as an [electronic flight instrument usable as a primary or backup instrument](https://www.garmin.com/en-US/blog/aviation/garmin-introduces-electronic-flight-instrument-experimental-aircraft/).

Test scripts, fixture variables, and ADRs use the technical identifier `pfd`.
Their names do not expand the product scope. Use EFI in new descriptions of the project.

## Product scope and reference

* Preserve mobile readability, low delay, explicit unavailable states, and G5-inspired hierarchy with original assets.
* Keep simulator access read-only. Do not add menus, knobs, simulator editing, or synchronization commands.
* The [accepted local BARO preference](../adr/0003-responsive-pfd-demo.md#local-baro-unit-preference-amendment) refines the original prohibition on browser controls. It sends no simulator commands.
* The original simulator MVP covered pitch/bank, scales, aircraft references, and source/connection status under #5-#8.
* Additional live EFI data was deferred in that original scope. Current [SimConnect capabilities and evidence](../sources/simconnect.md#live-publication-evidence) include optional instruments.
* Attitude-only publications still leave optional readings unavailable. Source failure must never select Demo silently.
* Demo scenarios supply selected altitude, magnetic heading, selected-heading bugs, and barometric setting through separate feature slices.

| Reference | Location |
| --- | --- |
| Reference guide | Garmin G5 Part 23 AML STC Pilot's Guide |
| File / revision | `190-01112-12_02.pdf`, document 190-01112-12 Rev. 2 |
| Illustration | Section 3.3, printed **Page 8 of 25**, physical PDF page **13** |
| Behavior | Subsequent subsections |

Use original graphics. Do not embed PDF screenshots or copy the bezel.

| Included illustration items | EFI capability | Feature issue |
| --- | --- | --- |
| 2, 3, 5, 7, 10 | Attitude, pitch scale, aircraft reference, slip/skid, turn rate | [#20](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/20) |
| 1, 4, 8 | Airspeed tape, current IAS, ground speed | [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21) |
| 21, extends 1 | V-speed references, speed ranges, six-second trend, VNE indications | [#22](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/22) |
| 9, 13, 15 | Barometric setting, current altitude, altimeter tape | [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) |
| 11, 16 | Selected-altitude bug/readout, approach/deviation alerts | [#24](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) |
| 12 | Vertical speed | [#25](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/25) |
| 18, 20 | Current heading and selected-heading bug | [#26](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/26) |

**Coverage: 17 of 22 illustration items.**

| Exclusion | Rule |
| --- | --- |
| CDI (6), vertical navigation guidance (14), navigation course (17), ground track (19), battery (22) | No UI space, model fields, or demo scenarios |
| HSI, ILS, GPS glidepath, VNAV | Outside scope. Current and selected heading remain EFI indications |
| Flight director, autopilot controls, recording, internet hosting | Outside scope |
| Full G5 replica | Outside scope |

* Original default units: kt, ft, ft/min, inHg, and degrees. The [BARO preference](altitude.md) adds local hPa and STD presentation.
* Demo speed thresholds belong to an explicit synthetic profile, not assumed real-aircraft performance. Live profile availability follows the [SimConnect observations](../sources/simconnect.md#optional-instrument-observations).

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
| Product scope | [Product scope and reference](#product-scope-and-reference) | Coverage and exclusions |
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
| Product coverage and exclusions | [Product scope and reference](#product-scope-and-reference) |
| Execution results and acceptance status | Related GitHub issues and PRs |

Apply these ownership rules when updating documentation:

* Read the feature together with its linked wire contract when changing inputs.
* Apply shared lifecycle and rendering rules to every feature without copying those rules into each page.
* Keep local constants, SVG state, paint order, controlled Demo inputs, and regression scenarios on the owning module or feature page.
* Keep cross-module contracts and reusable procedures on shared pages, with links to local details.

## Validation and evidence limits

[Feature acceptance ownership](../testing/pfd.md#feature-acceptance-ownership) defines shared criteria and the scope of accumulated Demo evidence.

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
