---
description: Find governing architecture decisions, their scope, and evidence limits. Use project-decide-architecture to compare options and record a new decision.
status: active
---

## Find the governing decision

Select decisions by the proposed change, then read their rationale and current contracts.
The scope column distinguishes later refinements and exceptions from the original decision.
An accepted decision does not establish implementation, simulator compatibility, or device acceptance.

| Proposed change or question | Decision and rationale | Current scope and related constraints |
| --- | --- | --- |
| Change the Rust/browser stack, transport, or process boundaries | [ADR 0001 decision](../docs/adr/0001-rust-http-sse.md#decision), [alternatives](../docs/adr/0001-rust-http-sse.md#alternatives-and-trade-offs) | Initial baseline; [current boundaries](../docs/architecture.md#boundaries) and [build policy](build-validation.md) govern platform and package features. [Reconsideration triggers](../docs/adr/0001-rust-http-sse.md#validation-of-the-choice) include remote access, commands, history, and scale. |
| Change subscriptions, backpressure, deadlines, or reconnects | [ADR 0002 decision](../docs/adr/0002-sse-connection-lifecycle.md#decision), [alternatives and limits](../docs/adr/0002-sse-connection-lifecycle.md#alternatives-and-consequences) | Refines the SSE baseline. Use the [framing contract](../docs/telemetry-contract.md#framing) for wire behavior; socket progress does not prove phone receipt. |
| Change PFD scope, responsive layout, controls, or live-data integration | [ADR 0003 decision](../docs/adr/0003-responsive-pfd-demo.md#decision), [alternatives](../docs/adr/0003-responsive-pfd-demo.md#alternatives-and-trade-offs) | Read-only demo scope and [product acceptance](../docs/project-brief.md#scope-and-acceptance). [Evidence limits](../docs/adr/0003-responsive-pfd-demo.md#consequences-and-validation) separate demo results from live compatibility. |
| Change instrument composition, shared frames, or layers | [ADR 0004 decision](../docs/adr/0004-modular-pfd-presentation.md#decision), [rationale](../docs/adr/0004-modular-pfd-presentation.md#alternatives-and-consequences) | Instrument-level baseline, refined internally by [ADR 0005](../docs/adr/0005-visual-object-composition.md#decision). The [presentation map](../docs/efi/index.md#features) leads to current interfaces and layer rules. |
| Move calculations, shared helpers, child nodes, or clips between visual objects | [ADR 0005 decision](../docs/adr/0005-visual-object-composition.md#decision), [rationale and consequences](../docs/adr/0005-visual-object-composition.md#rationale-and-consequences) | Places object-specific calculations with their visual objects. Use [current module seams](../docs/efi/composition.md#internal-module-seams); it adds no scheduler or telemetry ownership. |
| Change application composition, producer failure, startup order, or shutdown coordination | [ADR 0007 decision](../docs/adr/0007-application-composition.md#decision), [alternatives](../docs/adr/0007-application-composition.md#alternatives-and-consequences) | Supersedes ADR 0002 producer ownership and fatal-source failure only. Retains one active publisher, latest-state channel, SSE delivery and native shutdown limits. |
| Change SimConnect bindings, native ownership, loading, or diagnostic assumptions | [ADR 0006 decision](../docs/adr/0006-simconnect-manual-ffi.md#decision), [alternatives](../docs/adr/0006-simconnect-manual-ffi.md#alternatives-and-consequences) | [SimConnect scope and evidence](#simconnect-scope-and-evidence); [evidence limits](../docs/adr/0006-simconnect-manual-ffi.md#evidence-limits). |
| Remove a wire field or change its meaning or units | [Compatibility rule and accepted BARO exception](../docs/telemetry-contract.md#compatibility), [BARO decision provenance](../docs/efi/altitude.md#rationale-and-regression-lessons) | The #65 exception applies only to that replacement. It does not authorize future breaking changes within v1. |

For current SimConnect implementation boundaries and remaining evidence, use [SimConnect integration](../docs/architecture.md#simconnect-integration).
ADR 0006 separates the original structural acceptance and guard origin from the later bounded live-publication decision and remaining evidence limits in #5.
Use the [change map](component-changes.md) to locate affected contracts and validation after selecting the decision.

## SimConnect scope and evidence

Accepts manual minimal FFI/runtime loading in one existing-package worker, refining ADR 0001. The decision selects the main app, DLL beside EXE by default, optional absolute override and optional exact-file append diagnostics. [Live publication](../docs/adr/0006-simconnect-manual-ffi.md#live-publication-amendment) permits live publication within the [documented limits](../docs/sources/simconnect.md#live-publication-evidence), retaining lifecycle/numeric/freshness checks. Historical schemas 1–3 retain the guard; schema 4 enables live publication. Exact readiness timing, quantitative angle agreement, broader compatibility and full CPU coverage remain limits. Reuse applicable evidence; rerun for a concrete affected condition or contradiction.

## Decision procedure

The project-decide-architecture skill owns the [decision procedure](../.agents/skills/project-decide-architecture/SKILL.md#steps).

## Comparison criteria

The project-decide-architecture skill owns the [comparison criteria](../.agents/skills/project-decide-architecture/SKILL.md#comparison-criteria).

## ADR and delivery records

The project-decide-architecture skill owns the [adr and delivery records](../.agents/skills/project-decide-architecture/SKILL.md#adr-and-delivery-records).

## Sources and limits

* Existing decisions favor concrete needs and actual consumers over speculative frameworks or separate services.
