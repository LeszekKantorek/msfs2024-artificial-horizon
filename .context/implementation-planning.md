---
description: Build detailed implementation plans from requirements, current code, risks, and acceptance evidence. Use when decomposing an issue or explaining a plan.
status: active
---

## Scope and authorization

Preserve accepted decisions while resolving the current task.

* Read the complete accepted plan and its amendments before choosing a planning path.
* An instruction to implement an accepted plan authorizes its existing scope without another request for the same approval.
* Ask about unresolved requirements or material changes outside that scope before implementing dependent work.
* Use a short in-chat design for bounded changes and a detailed plan when requested or needed for coordination.
* Prepare plans directly from project sources and the [handoff rules](iterative-development.md#accepted-plans-and-handoffs).

## Planning procedure

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
13. Preserve the accepted plan under the [plan and handoff rules](iterative-development.md#accepted-plans-and-handoffs).

## Why these details matter

| Planning detail | Reason | Example from #63 |
| --- | --- | --- |
| Current code and source authority | Prevent stale knowledge from overriding a newer requirement | #63 replaced the separate trend lane while retaining six-second prediction |
| Decisions before decomposition | Prevent a layout preference from becoming an undocumented assumption | Return the removed 4 px lane to the horizon |
| Explicit invariants | Keep a visual change from changing unrelated behavior | Preserve freshness, alarm history, prediction, and SVG node identity |
| Actual module boundaries | Make the steps executable without speculative refactoring | Change allocation in layout and SVG creation order in altitude composition |
| Completion conditions | Define when a step produces its required result | The tape ends at the panel edge and has no external trend region |
| Scenario inputs and expected results | Turn requirements into reproducible checks | +1500 ft/min predicts +150 ft and draws 60 px upward |
| Evidence selected by failure risk | Prevent a passing test from answering the wrong question | Inspect painted overlap because bounding boxes cannot prove layer order |
| Baseline and matched captures | Make before/after comparisons attributable to the change | Repeat the same values, viewports, routes, and browser |
| Review and acceptance limits | Expose work that test results alone cannot establish | Separate independent review and actual phone checks from desktop validation |

## Plan quality checks

Check the plan before presenting it.

* Each step must explain what changes and how its completion can be observed.
* Each acceptance criterion must map to a check, an expected result, and evidence to record.
* Add substeps where dependency, risk, or distinct verification requires them.
* Avoid adding steps merely to increase plan length.
* Link canonical procedures instead of copying unrelated checklists.
* Apply [validation and review](validation-and-review.md) when selecting evidence and preparing the review handoff.
* Link relevant project sources under [task guidance selection](task-guidance.md) for each implementation step.

## Sources and limits

* Source: detailed-plan correction and accepted implementation plan for [#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63), 2026-10-11.
* Source: request to preserve planning rationale and testing/review lessons in project knowledge, 2026-10-11.
* The accepted #63 plan omitted independent subagent review despite including extensive validation.
* The review step above addresses that omission rather than describing a completed historical review.
* No verified evidence attributes the initial abbreviated plan to the removal of optional skills.
* Source: scope and authorization rules consolidated from task guidance during the full context review, 2026-10-11.
