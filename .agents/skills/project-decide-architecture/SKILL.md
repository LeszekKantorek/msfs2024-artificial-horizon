---
name: project-decide-architecture
description: Compare viable architecture options for a material project choice and record the recommendation or accepted decision in an ADR with rationale and evidence limits. Use for changes to ownership, boundaries, protocols, dependencies, or deployment, not routine implementation detail, lookup of an existing decision, or delivery planning after a choice is settled.
---

# Decide Project Architecture

## Goal

Produce an evidence-based architecture comparison and ADR with accurate decision status, consequences, unresolved questions, and conditions for reconsideration.

## Steps

Use this sequence for a material architecture choice rather than routine implementation detail.

1. Define the problem, current outcome, and next agreed use from the issue and roadmap.
2. Use the [governing-decision map](../../../.context/architecture-decisions.md#find-the-governing-decision) to select applicable ADRs, then read the current architecture and contracts for the affected boundary.
3. Check the implementation separately from target design and unverified integration assumptions.
4. List required invariants and the evidence that would make the change necessary.
5. Compare viable options, including retaining the existing design when it can meet the requirement.
6. Evaluate each option against the criteria below and identify unresolved evidence.
7. Use a focused experiment within the authorized scope when a material uncertainty prevents a supported choice. If the required evidence is unavailable, retain the unresolved alternatives and identify the evidence needed to choose.
8. Recommend the smallest structural change supported by evidence that serves the agreed use, and explain its costs. If evidence is insufficient, record the decision as unresolved rather than inventing certainty.
9. Record the material choice or unresolved comparison in an ADR with its actual decision status. A recommendation remains proposed until acceptance is established; do not mark it accepted merely because this skill produced it.
10. Update affected contracts and the implementation plan after the decision enters the accepted scope.
11. Identify the validation needed for later implementation. If implementation is already authorized and performed, validate its outcome separately from the architectural rationale; this decision procedure does not itself authorize implementation.

## Comparison criteria

Use criteria that affect the actual outcome.

| Criterion | Question | Existing example |
| --- | --- | --- |
| Need and scope | Which agreed requirement needs this change? | [ADR 0001](../../../docs/adr/0001-rust-http-sse.md) selects SSE for receive-only telemetry |
| Ownership and boundaries | Which component owns state, calculations, resources, and cleanup? | [ADR 0005](../../../docs/adr/0005-visual-object-composition.md) keeps calculations with their visual objects |
| Complexity and deployment | Which new processes, dependencies, protocols, or installation steps does the option require? | [ADR 0001 alternatives](../../../docs/adr/0001-rust-http-sse.md#alternatives-and-trade-offs) compare simpler and larger designs |
| Lifecycle and failure behavior | How does the option handle stale data, slow clients, cancellation, and shutdown? | [ADR 0002](../../../docs/adr/0002-sse-connection-lifecycle.md) rejects body-only socket timeouts |
| Compatibility and verification | Which wire, visual, accessibility, or module contract must remain valid? | [PFD boundary map](../../../.context/pfd-presentation.md#composition) links frame, clipping, ownership, and SVG contracts |
| Exposure and distribution | Does the option change trust boundaries, network access, SDK obligations, or permissions? | [Security review](../../../.context/security-review.md), [SimConnect integration](../../../docs/sources/simconnect.md) |
| Reconsideration | What future requirement or evidence would invalidate the choice? | [ADR 0001](../../../docs/adr/0001-rust-http-sse.md) identifies remote access, commands, history, and scale as triggers |

## ADR and delivery records

Follow the existing ADR examples and [document responsibilities](../../../.context/writing-style.md#document-responsibilities).

* Record the decision status, problem context, selected option, alternatives, rationale, and consequences.
* Write the status as a plain line. Do not require a date or use a metadata table.
* Retain evidence limits, deferred options, and conditions for reconsidering the decision.
* Link the originating issue and canonical contracts rather than duplicating their full contents.
* Explain which earlier decision changes and preserve unaffected requirements when a later decision replaces part of it.
* Distinguish proposed choices, accepted decisions, implemented behavior, and verified acceptance.
* Keep delivery status and execution results in the issue or PR.
* Preserve accepted plans and amendments under the [local handoff rules](../../../.context/iterative-development.md#accepted-plans-and-handoffs).
* Apply [project-plan-change](../project-plan-change/SKILL.md) and [evidence selection](../../../docs/testing/evidence.md#select-evidence-by-failure-risk) and [independent review guidance](../../../docs/testing/evidence.md#independent-subagent-review) after resolving the architectural choice.

## Completion checks

Check that the ADR identifies the problem, viable options including the current design, governing constraints, comparison, rationale, consequences, and actual status.
Retain uncertainties, unavailable experiments, superseded portions of earlier decisions, and conditions for reconsideration.
Use the [change map](../../../.context/component-changes.md) to identify affected contracts and validation without duplicating their contents in the ADR.
An accepted ADR does not establish implementation, simulator compatibility, or real-device acceptance.
The [decision guidance](../../../.context/architecture-decisions.md#sources-and-limits) explains applicable boundaries and evidence limits.
