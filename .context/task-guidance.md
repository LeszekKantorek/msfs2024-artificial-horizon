---
description: Select conditional sources when a task crosses knowledge areas or its direct route is unclear.
status: active
---

## Practical selection

This table selects sources for tasks that cross knowledge areas or require a condition-dependent route.
[Context selection and retrieval delegation](context-retrival-delegation.md) owns selection and the fallback to the index.
Use these routes to select sources by outcome. When sources are unknown or incomplete, use [context-retrieve](../.agents/skills/context-retrieve/SKILL.md) under the selection rules. The routes do not require reading every linked source.

Use these direct routes when their conditions apply:

* For instrument input changes, use the [wire contract](../docs/telemetry-contract.md#fields-and-conventions).
* For geometry or layer changes, use [layout validation](../docs/testing/pfd.md#geometry-and-paint-regression-checks).
* For cross-feature changes, use [shared responsibilities](../docs/efi/index.md#shared-responsibilities).

| Task | Read first | Extend only when relevant |
| --- | --- | --- |
| Locate requirements, decisions, code, or validation guidance for a concrete project question | [context-retrieve](../.agents/skills/context-retrieve/SKILL.md) | Use the affected component or decision map to narrow the search; retain verified references and unresolved source gaps |
| Select project context or delegate retrieval | [Context selection and retrieval delegation](context-retrival-delegation.md) | Follow the conditional routes and required readings for the work |
| Read or refresh GitHub issue context | [Issue and artifact retrieval](repository-instructions.md#read-github-issue-context) | Before issue-based planning or execution, and when another project procedure uses issue context |
| Preserve findings or review knowledge | The context skill matching the task and [knowledge conventions](agent-maintenance.md#knowledge-maintenance) | [Markdown structure](../.agents/skills/project-write-content/SKILL.md) when writing entries; [tool maintenance](agent-maintenance.md#project-procedure-skills) when changing skills |
| Plan a Task | [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) | [Planning rationale](implementation-planning.md#why-these-details-matter) when choosing detail; [work organization](../docs/work-organization.md) for hierarchy and publication boundaries |
| Prepare implementation context, with or without a Subtask | [Context selection](context-retrival-delegation.md), [Subtask context preparation](iterative-development.md#prepare-subtask-context) | For issue-based work, read the complete Subtask and [accepted plans and handoffs](iterative-development.md#accepted-plans-and-handoffs); select contracts and checks by the affected behavior |
| Consult Sol during Subtask execution | [Sol consultation](iterative-development.md#consult-sol) | Applicable contracts for the specific question; retain unavailable-model and authorization limits |
| Report Subtask results, gaps, and consultations | [Final Subtask summary](iterative-development.md#final-subtask-summary) | [Execution findings](iterative-development.md#record-execution-findings) for long Subtasks or an optional local journal |
| Change behavior across component boundaries, such as providers, telemetry, HTTP, or the browser | [Change map](component-changes.md) | [Governing decisions](architecture-decisions.md#find-the-governing-decision) when reconsidering accepted boundaries or behavior |
| Change IAS, GS, TAS, speed ranges, references, or trend | [Speed instrument map](speed.md) | — |
| Change altitude, BARO, selected altitude, vertical speed, or altitude trend | [Altitude instrument map](altitude.md) | — |
| Change current or selected heading | [Heading instrument map](heading.md) | — |
| Change attitude, coordinated turn, or shared presentation | [Presentation boundary map](../docs/efi/index.md#features) | — |
| Select checks or reuse evidence | [Validation scope](../docs/testing/index.md#validation-by-change-scope) | [Evidence reuse](../docs/testing/index.md#evidence-reuse) for prior runs; [test design](../docs/testing/evidence.md#select-evidence-by-failure-risk) for new checks; [review and device evidence limits](../docs/testing/evidence.md#sources-and-limits) when interpreting evidence |
| Investigate live field availability, SSE contract, or a human-started flight | [Bounded development observation](../docs/sources/simconnect.md#bounded-development-observation) | Inspect the captured records for fresh SimConnect telemetry. Select presentation validation separately for display questions |
| Diagnose an application, test, build, or environment failure | [project-diagnose-failure](../.agents/skills/project-diagnose-failure/SKILL.md) | [Diagnostic boundary map](error-diagnosis.md#select-the-diagnostic-boundary) for symptom-specific contracts and restrictions |
| Validate EFI geometry, layers, lifecycle, or browser module integration | [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) | Feature-specific checks selected by the procedure; actual phones when device acceptance is required |
| Review a diff or scoped implementation | [project-review-change](../.agents/skills/project-review-change/SKILL.md) | Applicable contracts and [change requirements](../CONTRIBUTING.md#change-requirements) |
| Prepare or publish a PR for agreed changes | [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md) | [Acceptance-control map](../docs/pr-acceptance.md) for validator failures or workflow changes |
| Start a contribution branch | [Contribution workflow](../CONTRIBUTING.md#workflow) | [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) only when Task planning or coordinated decomposition is needed |
| Create, revise, review, or move project text | [Technical writing rules](../.agents/skills/technical-writing-rules/SKILL.md) and [project conventions](../CONTRIBUTING.md#writing-guidance); [project-write-content](../.agents/skills/project-write-content/SKILL.md) for Markdown | [Documentation maintenance](../CONTRIBUTING.md#documentation-maintenance) for moves or consolidation; [entry conventions](agent-maintenance.md#local-entry-and-index-convention) for project knowledge |
| Review an exposure or permission change | [Security boundaries](security-review.md#project-boundaries) | [Focused checks](security-review.md#focused-checks) for the affected input, network, dependency, or CI boundary |
| Find the rationale or scope of an existing architecture choice | [Governing decisions](architecture-decisions.md#find-the-governing-decision) | [Change map](component-changes.md) when assessing affected contracts and checks |
| Make a material architecture decision | [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) | [Change map](component-changes.md) for affected contracts and validation |
