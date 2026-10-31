---
description: Select conditional sources when a task crosses knowledge areas or its direct route is unclear.
status: active
---

## Practical selection

Use this table for tasks that need more than one source or a condition-dependent route.
[AGENTS](../AGENTS.md#context-selection) owns context selection and the fallback to the index.

* For instrument input changes, use the [wire contract](../docs/telemetry-contract.md#fields-and-conventions).
* For geometry or layer changes, use [layout validation](../docs/testing/pfd.md#geometry-and-paint-regression-checks).
* For cross-feature changes, use [shared responsibilities](../docs/efi/index.md#shared-responsibilities).

| Task | Read first | Extend only when relevant |
| --- | --- | --- |
| Preserve findings or review knowledge | The context skill matching the task and [knowledge conventions](agent-maintenance.md#knowledge-maintenance) | [Markdown structure](../.agents/skills/project-write-content/SKILL.md) when writing entries; [tool maintenance](agent-maintenance.md#project-procedure-skills) when changing skills |
| Prepare a detailed implementation plan | [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) | [Planning rationale](implementation-planning.md#why-these-details-matter) when choosing detail; [handoffs](iterative-development.md#accepted-plans-and-handoffs) for accepted plans and amendments |
| Change behavior across component boundaries, such as providers, telemetry, HTTP, or the browser | [Change map](component-changes.md) | [Governing decisions](architecture-decisions.md#find-the-governing-decision) when reconsidering accepted boundaries or behavior |
| Change IAS, GS, TAS, speed ranges, references, or trend | [Speed instrument map](speed.md) | — |
| Change altitude, BARO, selected altitude, vertical speed, or altitude trend | [Altitude instrument map](altitude.md) | — |
| Change current or selected heading | [Heading instrument map](heading.md) | — |
| Change attitude, coordinated turn, or shared presentation | [Presentation boundary map](../docs/efi/index.md#features) | — |
| Select checks or reuse evidence | [Validation scope](../docs/testing/index.md#validation-by-change-scope) | [Evidence reuse](../docs/testing/index.md#evidence-reuse) for prior runs; [test design](../docs/testing/evidence.md#select-evidence-by-failure-risk) for new checks; [review and device evidence limits](../docs/testing/evidence.md#sources-and-limits) when interpreting evidence |
| Diagnose an application, test, build, or environment failure | [project-diagnose-failure](../.agents/skills/project-diagnose-failure/SKILL.md) | [Diagnostic boundary map](error-diagnosis.md#select-the-diagnostic-boundary) for symptom-specific contracts and restrictions |
| Validate EFI geometry, layers, lifecycle, or browser module integration | [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) | Feature-specific checks selected by the procedure; actual phones when device acceptance is required |
| Prepare independent code review | [project-review-change](../.agents/skills/project-review-change/SKILL.md) | Applicable contracts and [change requirements](../CONTRIBUTING.md#change-requirements) for the reviewed diff |
| Prepare or publish a PR for agreed changes | [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md) | [Acceptance-control map](../docs/pr-acceptance.md) for validator failures or workflow changes |
| Start a contribution branch | [Contribution workflow](../CONTRIBUTING.md#workflow) | [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) only when a detailed implementation plan is needed |
| Create, revise, review, or move project text | [Technical writing rules](../.agents/skills/technical-writing-rules/SKILL.md) and [project conventions](../CONTRIBUTING.md#writing-guidance); [project-write-content](../.agents/skills/project-write-content/SKILL.md) for Markdown | [Documentation maintenance](../CONTRIBUTING.md#documentation-maintenance) for moves or consolidation; [entry conventions](agent-maintenance.md#local-entry-and-index-convention) for project knowledge |
| Review an exposure or permission change | [Security boundaries](security-review.md#project-boundaries) | [Focused checks](security-review.md#focused-checks) for the affected input, network, dependency, or CI boundary |
| Find the rationale or scope of an existing architecture choice | [Governing decisions](architecture-decisions.md#find-the-governing-decision) | [Change map](component-changes.md) when assessing affected contracts and checks |
| Make a material architecture decision | [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) | [Change map](component-changes.md) for affected contracts and validation |
