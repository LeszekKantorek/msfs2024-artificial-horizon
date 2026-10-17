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
| Prepare a detailed implementation plan | [Implementation planning](implementation-planning.md) | [Handoffs](iterative-development.md#accepted-plans-and-handoffs) for accepted plans and amendments |
| Change behavior across providers, telemetry, HTTP, or the browser | [Change map](component-changes.md) | [Governing decisions](architecture-decisions.md#find-the-governing-decision) when reconsidering accepted boundaries or behavior |
| Change an instrument | [Presentation boundary map](../docs/pfd-presentation.md#find-the-affected-boundary) | [Wire contract](../docs/telemetry-contract.md#fields-and-conventions) for input changes; [layout validation](pfd-layout-validation.md) for geometry or layers |
| Select checks or reuse evidence | [Validation scope](../docs/testing.md#validation-by-change-scope) | [Evidence reuse](../docs/testing.md#evidence-reuse) for prior runs; [test design](validation-and-review.md#select-evidence-by-failure-risk) for new checks |
| Diagnose a failure | [Error diagnosis](error-diagnosis.md#select-the-diagnostic-boundary) | Follow the symptom-specific environment or rendering route in that table |
| Prepare independent code review | [Review procedure](validation-and-review.md#independent-subagent-review) | Applicable contracts and [change requirements](../CONTRIBUTING.md#change-requirements) for the reviewed diff |
| Prepare a branch or PR | [Contribution workflow](../CONTRIBUTING.md#workflow) | [Acceptance-control map](pr-acceptance.md) for validator failures or workflow changes |
| Write or relocate documentation | [Writing rules and document responsibilities](writing-style.md) | [Context authoring](context-authoring.md) for knowledge entries and map changes |
| Review an exposure or permission change | [Security boundaries](security-review.md#project-boundaries) | [Focused checks](security-review.md#focused-checks) for the affected input, network, dependency, or CI boundary |
| Find the rationale or scope of an existing architecture choice | [Governing decisions](architecture-decisions.md#find-the-governing-decision) | [Change map](component-changes.md) when assessing affected contracts and checks |
| Make a material architecture decision | [Decision procedure](architecture-decisions.md#decision-procedure), [governing decisions](architecture-decisions.md#find-the-governing-decision) | [Change map](component-changes.md) for affected contracts and validation |

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
