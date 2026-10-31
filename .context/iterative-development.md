---
description: Choose module boundaries for agreed delivery and preserve accepted plans locally without publishing them to GitHub.
status: active
---

Use [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) to assess material structural choices against the current outcome and next agreed use.
Use [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) to turn the accepted choice into executable steps.

* Keep speculative frameworks and later issues outside current scope.
* Do not postpone useful structural changes solely because the current skeleton works.
* Keep the [separate altitude tape](../docs/efi/altitude.md#rationale-and-regression-lessons) unless actual shared requirements justify reconsidering its scale, sign, digit, selection, and alert boundaries.
* Use the [internal seams](../docs/efi/composition.md#visual-object-interfaces-and-ownership) first instead of inferring shared rules from a planned instrument.
* Apply the [PFD composition rules](../docs/efi/composition.md#rationale-and-regression-lessons) for calculation ownership and extraction to actual shared consumers.

## Accepted plans and handoffs

* Use the [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) for detailed steps, substeps, completion conditions, and acceptance evidence.
* Preserve the complete accepted plan verbatim, including supplied language, requirements, and invariants.
* Store implementation plans, amendments, and handoff indexes under Git-ignored `.local/plans/<issue>-<topic>/`.
* Do not commit implementation plans or publish them in GitHub issues or PRs unless explicitly requested.
* Keep abbreviated execution notes as supplements, never replacements for the accepted plan.
* Keep session progress and diagnostic work logs local. Use them to prepare the final summary; do not use GitHub Issues as a running progress notebook.
* Record subsequent corrections as separate dated amendments that identify the superseded requirements.
* Keep a local handoff index that links the original plan, all amendments in application order, and the latest delivery evidence.
* Record public acceptance evidence in the issue or PR without publishing the implementation plan.
* Read the complete plan and applicable amendments before edits or delegated work, including all invariants.
* Preserve prior-stage evidence when an amendment changes later implementation boundaries.

The local progress reporting rule does not restrict publication of a requested final result.

## Findings outside the current scope

* Record the observation, expected behavior, supporting evidence, and status without expanding the authorized implementation scope.
* Distinguish accepted requirements from implemented behavior. A documented target does not establish that the code already meets it.
* Keep unresolved proposals separate from agreed follow-up work.
* Before removing a local record, preserve agreed outstanding work in an authorized destination that does not depend on that record.
* Use [finding promotion](../CONTRIBUTING.md#promote-agreed-findings) for self-contained follow-up issues. Publication still requires authorization.
