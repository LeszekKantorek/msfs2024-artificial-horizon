---
description: Find governing architecture decisions, their scope, and evidence limits. Use project-decide-architecture to compare options and record a new decision.
status: active
---

## Find the governing decision

Select decisions by the proposed change, then read their rationale and current contracts.
The scope column identifies current responsibilities and evidence limits.
An accepted decision does not establish implementation, simulator compatibility, or device acceptance.

| Proposed change or question | Decision and rationale | Current scope and related constraints |
| --- | --- | --- |
| Change the Rust/browser stack, transport, or process boundaries | [ADR 0001 decision](../docs/adr/0001-rust-http-sse.md#decision), [alternatives](../docs/adr/0001-rust-http-sse.md#alternatives-and-trade-offs) | Use the [current boundaries](../docs/architecture.md#components-and-execution) and [build policy](build-validation.md) govern platform and package features. [Reconsideration triggers](../docs/adr/0001-rust-http-sse.md#validation-of-the-choice) include remote access, commands, history, and scale. |
| Change subscriptions, backpressure, deadlines, or reconnects | [ADR 0002 decision](../docs/adr/0002-sse-connection-lifecycle.md#decision), [alternatives and limits](../docs/adr/0002-sse-connection-lifecycle.md#alternatives-and-consequences) | Defines SSE delivery and connection lifecycle. Use the [framing contract](../docs/telemetry-contract.md#framing) for wire behavior. Socket progress does not prove phone receipt. |
| Change PFD scope, responsive layout, controls, or live-data integration | [ADR 0003 decision](../docs/adr/0003-responsive-pfd-demo.md#decision), [alternatives](../docs/adr/0003-responsive-pfd-demo.md#alternatives-and-trade-offs) | Instrument scope, local BARO display preference, and [product scope](../docs/efi/index.md#product-scope-and-reference), [acceptance ownership](../docs/testing/pfd.md#feature-acceptance-ownership). [Evidence limits](../docs/adr/0003-responsive-pfd-demo.md#consequences-and-validation) separate demo results from live compatibility. |
| Change instrument composition, shared frames, or layers | [ADR 0004 decision](../docs/adr/0004-modular-pfd-presentation.md#decision), [rationale](../docs/adr/0004-modular-pfd-presentation.md#alternatives-and-consequences) | Instrument composition with internal visual-object boundaries defined by [ADR 0005](../docs/adr/0005-visual-object-composition.md#decision). The [presentation map](../docs/efi/index.md#features) leads to current interfaces and layer rules. |
| Move calculations, shared helpers, child nodes, or clips between visual objects | [ADR 0005 decision](../docs/adr/0005-visual-object-composition.md#decision), [rationale and consequences](../docs/adr/0005-visual-object-composition.md#rationale-and-consequences) | Places object-specific calculations with their visual objects. Use [current module seams](../docs/efi/composition.md#visual-object-interfaces-and-ownership). The decision adds no scheduler or telemetry ownership. |
| Change application composition, producer failure, startup order, or shutdown coordination | [ADR 0007 decision](../docs/adr/0007-application-composition.md#decision), [alternatives](../docs/adr/0007-application-composition.md#alternatives-and-consequences) | Defines producer ownership and source-failure isolation. Retains one active publisher, the latest-state channel, SSE delivery, and native shutdown limits. |
| Change SimConnect bindings, native ownership, loading, or diagnostic assumptions | [ADR 0006 decision](../docs/adr/0006-simconnect-manual-ffi.md#decision), [alternatives](../docs/adr/0006-simconnect-manual-ffi.md#alternatives-and-consequences) | [SimConnect scope and evidence](#simconnect-scope-and-evidence) and [evidence limits](../docs/adr/0006-simconnect-manual-ffi.md#evidence-limits). |
| Change application interfaces, wire fields, units, or meaning | [Application delivery contract](../docs/architecture.md#application-delivery-contract), [wire contract](../docs/telemetry-contract.md#compatibility) | Server, contract, and embedded Web form one application. Rebuild everything after every change. Update consumers and fixtures together without backward compatibility or legacy fallbacks. |

For current SimConnect implementation boundaries and remaining evidence, use [SimConnect integration](../docs/sources/simconnect.md).
ADR 0006 defines native ownership and live-publication conditions. Receiver evidence does not establish integrated simulator or mobile acceptance.
Use the [change map](component-changes.md) to locate affected contracts and validation after selecting the decision.

## SimConnect scope and evidence

ADR 0006 selects these native integration boundaries:

* Use minimal manual FFI and runtime loading in one worker within the existing package.
* Run acquisition in the main application.
* Select the DLL beside the EXE by default, with an optional absolute override.
* Support optional exact-file append diagnostics.
* Apply the [live-publication conditions](../docs/adr/0006-simconnect-manual-ffi.md#live-publication), including lifecycle, numeric, and freshness checks.

Evidence has these limits:

* Exact readiness timing and quantitative cockpit-angle agreement are not established.
* Broader simulator compatibility and full CPU coverage remain unverified.
* Receiver evidence does not establish the integrated HTTP/SSE-to-EFI path or actual mobile behavior.
* Interpret stored logs through the [diagnostic schema rules](../docs/sources/simconnect.md#diagnostic-schema-interpretation).
* Reuse applicable evidence. Rerun checks for an affected condition or contradictory observation.

## Application delivery rules

* Treat the server, telemetry contract, and embedded Web as one application under the [delivery contract](../docs/architecture.md#application-delivery-contract).
* Rebuild and deliver the complete application after every change.
* Update producers, consumers, and fixtures together without backward compatibility guarantees.
* Add no legacy adapters or fallbacks to previous formats or values.
* Keep documentation links out of application source code and runtime output.
* Optional readings can be unavailable within the current contract. Do not confuse this with support for previous formats.

## Decision procedure

The project-decide-architecture skill owns the [decision procedure](../.agents/skills/project-decide-architecture/SKILL.md#steps).

## Comparison criteria

The project-decide-architecture skill owns the [comparison criteria](../.agents/skills/project-decide-architecture/SKILL.md#comparison-criteria).

## ADR and delivery records

The project-decide-architecture skill owns the [adr and delivery records](../.agents/skills/project-decide-architecture/SKILL.md#adr-and-delivery-records).

## Sources and limits

* Existing decisions favor concrete needs and actual consumers over speculative frameworks or separate services.
