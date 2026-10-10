# ADR 0005: Compose instruments from visual objects

| Date | Status |
| --- | --- |
| 2026-10-11 | Accepted |

## Context

[ADR 0004](0004-modular-pfd-presentation.md) establishes instrument composition and one frame scheduler.
Large internal creation, resize, and update operations still combine several visual responsibilities.
The next boundary follows concrete visual objects while preserving appearance, accessibility, telemetry, and lifecycle.

An initial split placed pure calculations in separate files.
The accepted correction places object-specific functions with their implementing visual objects.
Pure behavior and direct tests do not require a separate file.

## Decision

Use JavaScript factories, composition, and small object interfaces within the existing modular architecture.
The [presentation reference](../pfd-presentation.md#internal-module-seams) owns detailed interfaces, calculation locations, and resource ownership.

* Keep complete read-only frames at instrument entry points and select required fields inside each instrument.
* Give internal objects concrete hosts, geometry, and readings, with private nodes and their own operations.
* Keep shared hosts and clips with their immediate common parent, including the panel's common attitude clip.
* Keep both tape and fallback reference presentations in one reference object.
* Calculate common attitude transforms once and delegate the same result to moving objects.
* Keep fixed objects resize-only, without mandatory empty render methods.
* Keep object-specific calculations and constants in their implementing modules.
* Extract shared rules only for actual independent consumers.

The three shared speed exports have concrete callers:

| Shared rule | Consumers |
| --- | --- |
| `MAX_SPEED` | Speed classification, tape extent, reference formatting |
| `speedState` | IAS and GS |
| `tapeOffset` | Tape and references |

`web/helpers.js` combines `clamp`, `svgElement`, and `setAttributes` as named exports.
This JavaScript compromise keeps actual shared utilities small without import-time DOM access.
Pure functions can import clamp while SVG creation remains dormant until called.
The compromise requires preserving that import boundary.

## Rationale and consequences

Single responsibilities follow visual ownership, including each object's calculations and state transitions.
Small interfaces hide child nodes and prevent parents from controlling unrelated paths or attributes.
Extensions change the relevant object and its explicit composition site, consistent with OCP/SOLID.
OCP does not require a plugin registry or closed composition list.

KISS keeps named range and trend operations inside the tape instead of creating modules for primitive shapes.
DRY shares existing rules with actual callers and computes common transforms once.
Meaningful pure tests import the owning modules in Node without DOM.
SVG identity, paint order, clipping, and accessible meanings remain compatibility constraints.
Visual objects add no frame cache, timer, subscription, or scheduler.

| Alternative | Decision |
| --- | --- |
| Continue centralized node bags and update operations | Reject because parents must coordinate child implementation details |
| Separate every pure function into a calculation file | Reject because purity alone does not establish an independent responsibility |
| Factory composition around existing visual objects | Select for explicit ownership and small operations without a technology migration |
| Base classes, a control framework, or a plugin registry | Defer because current composition requires none |
| Generic IAS/altitude tape | Defer until altitude supplies concrete scale, sign, digit, selection, and alert rules |

Decision source: approved [#56](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/56) continuation of [PR #55](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/pull/55), including the calculation-ownership correction of 2026-10-11.
ADR 0004 remains the historical instrument-level decision.
Acceptance of this decision does not establish final validation or real-device acceptance.
