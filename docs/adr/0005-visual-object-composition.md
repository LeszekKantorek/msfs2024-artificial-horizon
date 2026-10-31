# ADR 0005: Compose instruments from visual objects

Decision status: Accepted.

## Context

[ADR 0004](0004-modular-pfd-presentation.md) establishes instrument composition and one frame scheduler.
The visual-object decision addresses these internal boundaries:

* Combining unrelated creation, resize, and update operations obscures visual responsibilities.
* Visual-object boundaries preserve appearance, accessibility, telemetry, and lifecycle.
* Object-specific functions belong to their visual objects.
* Pure behavior and direct tests do not require a separate file.

## Decision

Use JavaScript factories, composition, and small object interfaces within the existing modular architecture.
The [presentation reference](../efi/composition.md#visual-object-interfaces-and-ownership) owns detailed interfaces, calculation locations, and resource ownership.

Visual ownership defines the interfaces:

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

`web/helpers.js` combines shared utilities under one import boundary:

* Export `clamp`, `svgElement`, and `setAttributes` by name.
* Keep actual shared utilities small without import-time DOM access.
* Let pure functions import clamp while SVG creation remains dormant until called.
* Preserve that import boundary as a condition of this JavaScript compromise.

## Rationale and consequences

Visual ownership keeps responsibilities and extensions local:

* Single responsibilities follow visual ownership, including each object's calculations and state transitions.
* Small interfaces hide child nodes and prevent parents from controlling unrelated paths or attributes.
* Extensions change the relevant object and its explicit composition site, consistent with OCP/SOLID.
* OCP does not require a plugin registry or closed composition list.

The decision limits abstraction and preserves presentation behavior:

* KISS keeps named range and trend operations inside the tape instead of creating modules for primitive shapes.
* DRY shares existing rules with actual callers and computes common transforms once.
* Meaningful pure tests import the owning modules in Node without DOM.
* Preserve SVG identity, paint order, clipping, and accessible meanings during internal refactoring.
* Visual objects add no frame cache, timer, subscription, or scheduler.

| Alternative | Decision |
| --- | --- |
| Continue centralized node bags and update operations | Reject because parents must coordinate child implementation details |
| Separate every pure function into a calculation file | Reject because purity alone does not establish an independent responsibility |
| Factory composition around existing visual objects | Select for explicit ownership and small operations without a technology migration |
| Base classes, a control framework, or a plugin registry | Defer because current composition requires none |
| Generic IAS/altitude tape | Defer until altitude supplies concrete scale, sign, digit, selection, and alert rules |

> Acceptance of this decision does not establish final validation or real-device acceptance.
