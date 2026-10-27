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
| Move calculations, shared helpers, child nodes, or clips between visual objects | [ADR 0005 decision](../docs/adr/0005-visual-object-composition.md#decision), [rationale and consequences](../docs/adr/0005-visual-object-composition.md#rationale-and-consequences) | Refines ADR 0004 and corrects the earlier separation of object-specific calculations. Use [current module seams](../docs/efi/composition.md#internal-module-seams); it adds no scheduler or telemetry ownership. |
| Change SimConnect bindings, native ownership, loading, or diagnostic assumptions | [ADR 0006 decision](../docs/adr/0006-simconnect-manual-ffi.md#decision), [alternatives](../docs/adr/0006-simconnect-manual-ffi.md#alternatives-and-rationale), [evidence limits](../docs/adr/0006-simconnect-manual-ffi.md#experimental-choices-and-evidence-limits) | Accepts manual minimal FFI/runtime loading in one existing-package worker, refining ADR 0001. The 2026-10-27 amendment selects the main app, DLL beside EXE by default, optional absolute override, and optional exact-file append diagnostics. [SimConnect scenarios](../docs/simconnect.md#repeatable-simulator-scenarios) own live evidence requirements; [basic sign directions](../docs/simconnect.md#direction-evidence) have paired log/operator support. Full lifecycle, dispatch/wait, reconnect and cleanup acceptance remains incomplete; candidate live is published as waiting. #5 owns reception, #6 connects the same module to HTTP/SSE/EFI, and #8 owns release packaging. |
| Remove a wire field or change its meaning or units | [Compatibility rule and accepted BARO exception](../docs/telemetry-contract.md#compatibility), [BARO decision provenance](../docs/efi/altitude.md#rationale-and-regression-lessons) | The #65 exception applies only to that replacement. It does not authorize future breaking changes within v1. |

For current SimConnect implementation boundaries and remaining evidence, use [integration uncertainty](../docs/architecture.md#integration-uncertainty).
ADR 0006 records accepted structural decisions separately from the unresolved hypotheses in #5.
Use the [change map](component-changes.md) to locate affected contracts and validation after selecting the decision.

## Decision procedure

The project-decide-architecture skill owns the [decision procedure](../.agents/skills/project-decide-architecture/SKILL.md#steps).

## Comparison criteria

The project-decide-architecture skill owns the [comparison criteria](../.agents/skills/project-decide-architecture/SKILL.md#comparison-criteria).

## ADR and delivery records

The project-decide-architecture skill owns the [adr and delivery records](../.agents/skills/project-decide-architecture/SKILL.md#adr-and-delivery-records).

## Sources and limits

* Sources: [architecture](../docs/architecture.md), [iteration rules](iterative-development.md), and ADRs 0001, 0002, 0005, and 0006 linked above.
* Source: request to create project-based architecture decision guidance, 2026-10-11.
* Existing decisions favor concrete needs and actual consumers over speculative frameworks or separate services.
* An accepted ADR does not establish successful implementation, simulator compatibility, or real-device acceptance.
