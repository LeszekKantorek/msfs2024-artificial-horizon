---
description: Compare architecture options against agreed needs and record material decisions in ADRs. Use when changing boundaries, ownership, protocols, dependencies, or deployment.
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
| Change instrument composition, shared frames, or layers | [ADR 0004 decision](../docs/adr/0004-modular-pfd-presentation.md#decision), [rationale](../docs/adr/0004-modular-pfd-presentation.md#alternatives-and-consequences) | Instrument-level baseline, refined internally by [ADR 0005](../docs/adr/0005-visual-object-composition.md#decision). The [presentation map](../docs/pfd-presentation.md#find-the-affected-boundary) leads to current interfaces and layer rules. |
| Move calculations, shared helpers, child nodes, or clips between visual objects | [ADR 0005 decision](../docs/adr/0005-visual-object-composition.md#decision), [rationale and consequences](../docs/adr/0005-visual-object-composition.md#rationale-and-consequences) | Refines ADR 0004 and corrects the earlier separation of object-specific calculations. Use [current module seams](../docs/pfd-presentation.md#internal-module-seams); it adds no scheduler or telemetry ownership. |
| Remove a wire field or change its meaning or units | [Compatibility rule and accepted BARO exception](../docs/telemetry-contract.md#compatibility), [BARO decision provenance](altitude-baro.md) | The #65 exception applies only to that replacement. It does not authorize future breaking changes within v1. |

For SimConnect binding, signs, units, or distribution, use [integration uncertainty](../docs/architecture.md#integration-uncertainty).
The existing architecture leaves these choices dependent on evidence from #5.
Use the [change map](component-changes.md) to locate affected contracts and validation after selecting the decision.

## Decision procedure

Use this sequence for a material architecture choice rather than routine implementation detail.

1. Define the problem, current outcome, and next agreed use from the issue and roadmap.
2. Read the current architecture, applicable ADRs, and contracts for the affected boundary.
3. Check the implementation separately from target design and unverified integration assumptions.
4. List required invariants and the evidence that would make the change necessary.
5. Compare viable options, including retaining the existing design when it can meet the requirement.
6. Evaluate each option against the criteria below and identify unresolved evidence.
7. Use a focused experiment when a material uncertainty prevents a supported choice.
8. Select the smallest structural change that serves the agreed use and explain its costs.
9. Record a material choice in an ADR with its actual decision status.
10. Update affected contracts and the implementation plan after the decision enters the accepted scope.
11. Validate the implemented outcome separately from the architectural rationale.

## Comparison criteria

Use criteria that affect the actual outcome.

| Criterion | Question | Existing example |
| --- | --- | --- |
| Need and scope | Which agreed requirement needs this change? | [ADR 0001](../docs/adr/0001-rust-http-sse.md) selects SSE for receive-only telemetry |
| Ownership and boundaries | Which component owns state, calculations, resources, and cleanup? | [ADR 0005](../docs/adr/0005-visual-object-composition.md) keeps calculations with their visual objects |
| Complexity and deployment | Which new processes, dependencies, protocols, or installation steps does the option require? | [ADR 0001 alternatives](../docs/adr/0001-rust-http-sse.md#alternatives-and-trade-offs) compare simpler and larger designs |
| Lifecycle and failure behavior | How does the option handle stale data, slow clients, cancellation, and shutdown? | [ADR 0002](../docs/adr/0002-sse-connection-lifecycle.md) rejects body-only socket timeouts |
| Compatibility and verification | Which wire, visual, accessibility, or module contract must remain valid? | [PFD composition](pfd-module-plan.md) links frame, clipping, ownership, and SVG contracts |
| Exposure and distribution | Does the option change trust boundaries, network access, SDK obligations, or permissions? | [Security review](security-review.md), [integration uncertainty](../docs/architecture.md#integration-uncertainty) |
| Reconsideration | What future requirement or evidence would invalidate the choice? | [ADR 0001](../docs/adr/0001-rust-http-sse.md) identifies remote access, commands, history, and scale as triggers |

## ADR and delivery records

Follow the existing ADR examples and [document responsibilities](writing-style.md#document-responsibilities).

* Record the date, decision status, problem context, selected option, alternatives, rationale, and consequences.
* Retain evidence limits, deferred options, and conditions for reconsidering the decision.
* Link the originating issue and canonical contracts rather than duplicating their full contents.
* Explain which earlier decision changes and preserve unaffected requirements when a later decision replaces part of it.
* Distinguish proposed choices, accepted decisions, implemented behavior, and verified acceptance.
* Keep delivery status and execution results in the issue or PR.
* Preserve accepted plans and amendments under the [local handoff rules](iterative-development.md#accepted-plans-and-handoffs).
* Apply [implementation planning](implementation-planning.md) and [validation and review](validation-and-review.md) after resolving the architectural choice.

## Sources and limits

* Sources: [architecture](../docs/architecture.md), [iteration rules](iterative-development.md), and ADRs 0001, 0002, and 0005 linked above.
* Source: request to create project-based architecture decision guidance, 2026-10-11.
* Existing decisions favor concrete needs and actual consumers over speculative frameworks or separate services.
* An accepted ADR does not establish successful implementation, simulator compatibility, or real-device acceptance.
