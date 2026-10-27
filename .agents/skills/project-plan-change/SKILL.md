---
name: project-plan-change
description: Prepare a detailed implementation plan for an issue or coordinated project change, with scope, dependencies, invariants, and acceptance evidence. Do not use for a small direct edit, executing an already accepted plan, or recording an architecture decision alone.
---

# Plan a Project Change

## Goal

Produce a reviewable implementation plan whose steps preserve accepted scope and identify the evidence needed to establish completion.

## Scope and authorization

Preserve accepted decisions while resolving the current task.

* Read the complete accepted plan and its amendments before choosing a planning path.
* An instruction to implement an accepted plan authorizes its existing scope without another request for the same approval.
* Ask about unresolved requirements or material changes outside that scope before implementing dependent work.
* Use a short in-chat design for bounded changes and a detailed plan when requested or needed for coordination.
* Prepare plans directly from project sources and the [handoff rules](../../../.context/iterative-development.md#accepted-plans-and-handoffs).

## Steps

Use this procedure for a detailed plan, with depth proportional to the change.

1. Read the issue, reference images, applicable project knowledge, and canonical contracts.
2. Check the current implementation and identify the base revision.
3. Resolve replaced requirements from their authority and record which earlier requirements still apply.
4. State the intended result, accepted decisions, scope boundaries, and behavior that must remain unchanged.
5. Identify unresolved decisions that affect implementation or acceptance.
6. Ask about material unresolved choices before planning dependent work.
7. Trace the change through actual module responsibilities, geometry, rendering, data lifecycle, and integration.
8. Define acceptance evidence for each requirement before choosing implementation steps.
9. Order numbered steps by dependency, including baseline capture before visual changes.
10. Give each step concrete substeps, relevant files or interfaces, and an observable completion condition.
11. Include applicable validation, independent review, documentation, context maintenance, and PR acceptance as explicit steps.
12. Identify unavailable checks and the evidence needed to complete them.
13. Preserve the accepted plan under the [plan and handoff rules](../../../.context/iterative-development.md#accepted-plans-and-handoffs).

## Completion checks

Check the plan before presenting it.

* Each step must explain what changes and how its completion can be observed.
* Each acceptance criterion must map to a check, an expected result, and evidence to record.
* Add substeps where dependency, risk, or distinct verification requires them.
* Avoid adding steps merely to increase plan length.
* Link canonical procedures instead of copying unrelated checklists.
* Apply [evidence selection](../../../docs/testing/evidence.md#select-evidence-by-failure-risk) and [independent review guidance](../../../docs/testing/evidence.md#independent-subagent-review) when selecting evidence and preparing the review handoff.
* Link relevant project sources under [task guidance selection](../../../.context/task-guidance.md) for each implementation step.

## Supporting knowledge

Read the [planning rationale](../../../.context/implementation-planning.md#why-these-details-matter) when choosing planning detail or explaining a trade-off.
The [planning guidance](../../../.context/implementation-planning.md#sources-and-limits) describes evidence and review limits.
Planning does not authorize implementation or publication of a local plan. Finish with the plan and any material unresolved decisions.
