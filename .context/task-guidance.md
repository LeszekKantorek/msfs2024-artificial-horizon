---
description: Resolve unclear routes or tasks that cross several areas. Select only the project entries and source sections needed for the affected behavior.
status: active
---

## Practical selection

Use this table when the index does not already identify a clear route.
Select by the behavior being changed, not merely by the file extension or directory.

| Task | Read first | Extend only when relevant |
| --- | --- | --- |
| Project knowledge | [Context authoring](context-authoring.md) when editing knowledge | [Context maintenance](agent-maintenance.md) for changes to context tools or hooks |
| Planning and module design | [Implementation planning](implementation-planning.md) for a requested detailed plan | [Handoffs](iterative-development.md#accepted-plans-and-handoffs) for accepted plans, [PFD composition](pfd-module-plan.md) for ownership, or the affected [architecture](../docs/architecture.md) section |
| Interface development | The affected instrument entry from the [index](index.md) | [PFD layout validation](pfd-layout-validation.md) for geometry or layers, [browser presentation](browser-presentation.md) for shared behavior, and affected [presentation sections](../docs/pfd-presentation.md) |
| Test implementation and diagnosis | [Test design](validation-and-review.md#select-evidence-by-failure-risk) for new tests, or [error diagnosis](error-diagnosis.md) for failures | [Local validation environment](local-validation-environment.md) for executable or browser failures, [Windows validation](windows-validation.md) for SAC or E0463 symptoms |
| Test strategy and acceptance | [Validation scope](../docs/testing.md#validation-by-change-scope) | Applicable procedures in [testing](../docs/testing.md), [evidence limits](validation-and-review.md) for uncertain coverage, or [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) for checklists |
| Review | [Independent subagent review](validation-and-review.md#independent-subagent-review) when preparing delegated review | The contracts affected by the diff and applicable [repository rules](repository-instructions.md) |
| Verification | [Validation scope](../docs/testing.md#validation-by-change-scope) | [Evidence reuse](../docs/testing.md#evidence-reuse) for previous results, or [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) for issue or PR criteria |
| Writing | Applicable [writing rules](writing-style.md) and the text being changed | [Document responsibilities](writing-style.md#document-responsibilities) when placing requirements, [context authoring](context-authoring.md) when editing knowledge |
| Security | [Security review](security-review.md) for changed exposure, inputs, dependencies, or privileges | [Deployment](../docs/architecture.md#deployment) for network changes, [CI trust boundary](../docs/pr-acceptance.md#trust-boundary) for workflow permissions, or affected [repository rules](repository-instructions.md) |
| Major architecture decision | [Architecture decisions](architecture-decisions.md) for material structural choices | The affected [architecture](../docs/architecture.md) section and ADR, such as [visual composition](../docs/adr/0005-visual-object-composition.md) only when changing that boundary |

## Working approach

* Open only entries and source sections needed to establish the outcome, constraints, or validation method.
* Links are references, not recursive reading requirements.
* Apply all matching routes when a change affects several boundaries.
* Stop discovery once the outcome, constraints, and validation method are clear.
* Expand discovery for a dependency, conflict, missing fact, or changed scope.
* Reuse content already read while its source and applicability remain unchanged.
* Follow the accepted scope and applicable repository constraints without imposing an arbitrary file-count limit.
* Optional expert skills are no longer part of the project workflow.
* Do not install or restore them as prerequisites for planning, implementation, testing, review, or writing.
* Keep the context retrieval and maintenance workflow defined in [AGENTS](../AGENTS.md).
* Use verified learning to extend guidance through the [context authoring rules](context-authoring.md).
* Conditional reading does not waive required checks, accepted plan invariants, or evidence limits.

## Sources

* Source: accepted planning scope and evidence-reuse adaptations, 2026-10-11.
* Source: planning and testing knowledge request after #63, 2026-10-11, including the missing independent subagent review.
* Source: explicit migration from optional expert skills to project context and existing Markdown documentation, 2026-10-11.
* The migration replaces the skill recommendations and plan skill annotations accepted on 2026-10-10.
* Source: request to create diagnostic, security review, and architecture decision entries from existing repository sources, 2026-10-11.
* Source: accepted conditional routing proposal and five-scenario validation request, 2026-10-11.
