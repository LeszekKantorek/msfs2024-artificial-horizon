---
description: Select conditional sources when a task crosses knowledge areas or its direct route is unclear.
status: active
---

## Practical selection

Use this table for tasks that need more than one source or a condition-dependent route.
[AGENTS](../AGENTS.md#context-selection) owns context selection and the fallback to the index.

| Task | Read first | Extend only when relevant |
| --- | --- | --- |
| Preserve findings or review knowledge | [Context authoring](context-authoring.md) and the matching [context skill](../AGENTS.md#project-knowledge) | [Tool maintenance](agent-maintenance.md) when changing skills or hooks |
| Prepare a detailed implementation plan | [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) | [Planning rationale](implementation-planning.md#why-these-details-matter) when choosing detail; [handoffs](iterative-development.md#accepted-plans-and-handoffs) for accepted plans and amendments |
| Change behavior across providers, telemetry, HTTP, or the browser | [Change map](component-changes.md) | [Governing decisions](architecture-decisions.md#find-the-governing-decision) when reconsidering accepted boundaries or behavior |
| Change an instrument | [Presentation boundary map](../docs/pfd-presentation.md#find-the-affected-boundary) | [Wire contract](../docs/telemetry-contract.md#fields-and-conventions) for input changes; [layout validation](pfd-layout-validation.md) for geometry or layers |
| Select checks or reuse evidence | [Validation scope](../docs/testing.md#validation-by-change-scope) | [Evidence reuse](../docs/testing.md#evidence-reuse) for prior runs; [test design](validation-and-review.md#select-evidence-by-failure-risk) for new checks |
| Diagnose an application, test, build, or environment failure | [project-diagnose-failure](../.agents/skills/project-diagnose-failure/SKILL.md) | [Diagnostic boundary map](error-diagnosis.md#select-the-diagnostic-boundary) for symptom-specific contracts and restrictions |
| Validate PFD geometry, layers, lifecycle, or browser module integration | [pfd-validate-presentation](../.agents/skills/pfd-validate-presentation/SKILL.md) | Feature-specific checks selected by the procedure; actual phones when device acceptance is required |
| Prepare independent code review | [project-review-change](../.agents/skills/project-review-change/SKILL.md) | Applicable contracts and [change requirements](../CONTRIBUTING.md#change-requirements) for the reviewed diff |
| Prepare or publish a PR for agreed changes | [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md) | [Acceptance-control map](pr-acceptance.md) for validator failures or workflow changes |
| Start a contribution branch | [Contribution workflow](../CONTRIBUTING.md#workflow) | [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) only when a detailed implementation plan is needed |
| Write or relocate documentation | [Writing rules and document responsibilities](writing-style.md) | [Context authoring](context-authoring.md) for knowledge entries and map changes |
| Review an exposure or permission change | [Security boundaries](security-review.md#project-boundaries) | [Focused checks](security-review.md#focused-checks) for the affected input, network, dependency, or CI boundary |
| Find the rationale or scope of an existing architecture choice | [Governing decisions](architecture-decisions.md#find-the-governing-decision) | [Change map](component-changes.md) when assessing affected contracts and checks |
| Make a material architecture decision | [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) | [Change map](component-changes.md) for affected contracts and validation |

## Working approach

The expert-skill selection workflow retired on 2026-10-11 remains replaced by project procedures.
Do not restore that former set as a prerequisite for project work.
Select the currently installed skills through [AGENTS](../AGENTS.md#project-knowledge).
The [historical selection entry](skill-selection.md) retains the replacement record.

## Sources

* Source: accepted planning scope and evidence-reuse adaptations, 2026-10-11.
* Source: planning and testing knowledge request after #63, 2026-10-11, including the missing independent subagent review.
* Source: explicit migration from optional expert skills to project context and existing Markdown documentation, 2026-10-11.
* The migration replaces the skill recommendations and plan skill annotations accepted on 2026-10-10.
* Source: request to create diagnostic, security review, and architecture decision entries from existing repository sources, 2026-10-11.
* Source: accepted conditional routing proposal and five-scenario validation request, 2026-10-11.
