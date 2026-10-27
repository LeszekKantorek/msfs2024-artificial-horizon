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
| Change an instrument | [Presentation boundary map](../docs/efi/index.md#features) | [Wire contract](../docs/telemetry-contract.md#fields-and-conventions) for input changes; [layout validation](../docs/testing/pfd.md#geometry-and-paint-regression-checks) for geometry or layers |
| Select checks or reuse evidence | [Validation scope](../docs/testing/index.md#validation-by-change-scope) | [Evidence reuse](../docs/testing/index.md#evidence-reuse) for prior runs; [test design](../docs/testing/evidence.md#select-evidence-by-failure-risk) for new checks |
| Diagnose an application, test, build, or environment failure | [project-diagnose-failure](../.agents/skills/project-diagnose-failure/SKILL.md) | [Diagnostic boundary map](error-diagnosis.md#select-the-diagnostic-boundary) for symptom-specific contracts and restrictions |
| Validate EFI geometry, layers, lifecycle, or browser module integration | [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) | Feature-specific checks selected by the procedure; actual phones when device acceptance is required |
| Prepare independent code review | [project-review-change](../.agents/skills/project-review-change/SKILL.md) | Applicable contracts and [change requirements](../CONTRIBUTING.md#change-requirements) for the reviewed diff |
| Prepare or publish a PR for agreed changes | [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md) | [Acceptance-control map](../docs/pr-acceptance.md) for validator failures or workflow changes |
| Start a contribution branch | [Contribution workflow](../CONTRIBUTING.md#workflow) | [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) only when a detailed implementation plan is needed |
| Write or relocate documentation | [Writing rules and document responsibilities](writing-style.md) | [Context authoring](context-authoring.md) for knowledge entries and map changes |
| Review an exposure or permission change | [Security boundaries](security-review.md#project-boundaries) | [Focused checks](security-review.md#focused-checks) for the affected input, network, dependency, or CI boundary |
| Find the rationale or scope of an existing architecture choice | [Governing decisions](architecture-decisions.md#find-the-governing-decision) | [Change map](component-changes.md) when assessing affected contracts and checks |
| Make a material architecture decision | [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) | [Change map](component-changes.md) for affected contracts and validation |

## Working approach

Select the currently installed skills through [AGENTS](../AGENTS.md#project-knowledge).
Planning scope is defined by [implementation planning](implementation-planning.md#scope-and-authorization); verification scope is defined by the [validation matrix](../docs/testing/index.md#validation-by-change-scope).
