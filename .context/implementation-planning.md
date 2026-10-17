---
description: Explain planning detail, preserved lessons, and evidence limits. Use project-plan-change to execute the planning procedure.
status: active
---

## Scope and authorization

The project-plan-change skill owns the [scope and authorization](../.agents/skills/project-plan-change/SKILL.md#scope-and-authorization).

## Planning procedure

The project-plan-change skill owns the [planning steps](../.agents/skills/project-plan-change/SKILL.md#steps).

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

The project-plan-change skill owns the [completion checks](../.agents/skills/project-plan-change/SKILL.md#completion-checks).

## Sources and limits

* Source: detailed-plan correction and accepted implementation plan for [#63](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/63), 2026-10-11.
* Source: request to preserve planning rationale and testing/review lessons in project knowledge, 2026-10-11.
* The accepted #63 plan omitted independent subagent review despite including extensive validation.
* The review step above addresses that omission rather than describing a completed historical review.
* No verified evidence attributes the initial abbreviated plan to the removal of optional skills.
* Source: scope and authorization rules consolidated from task guidance during the full context review, 2026-10-11.
