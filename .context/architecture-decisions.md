---
description: Compare architecture options against agreed needs and record material decisions in ADRs. Use when changing boundaries, ownership, protocols, dependencies, or deployment.
status: active
---

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
