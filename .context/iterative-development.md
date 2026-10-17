---
description: Choose module boundaries for agreed delivery and preserve accepted plans locally without publishing them to GitHub.
status: active
---

Use [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) to assess material structural choices against the current outcome and next agreed use.
Use [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) to turn the accepted choice into executable steps.

* Keep speculative frameworks and later issues outside current scope.
* Do not postpone useful structural changes solely because the current skeleton works.
* Keep the [separate altitude tape](altitude-baro.md) unless actual shared requirements justify reconsidering its scale, sign, digit, selection, and alert boundaries.
* Use the implemented [internal seams](../docs/pfd-presentation.md#internal-module-seams) first instead of inferring shared rules from a planned instrument.
* Apply the [PFD composition rules](pfd-module-plan.md) for calculation ownership and extraction to actual shared consumers.

## Accepted plans and handoffs

* Use the [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) for detailed steps, substeps, completion conditions, and acceptance evidence.
* Preserve the complete accepted plan verbatim, including supplied language, requirements, and invariants.
* Store implementation plans, amendments, and handoff indexes under Git-ignored `.local/plans/<issue>-<topic>/`.
* Do not commit implementation plans or publish them in GitHub issues or PRs unless explicitly requested.
* Keep abbreviated execution notes as supplements, never replacements for the accepted plan.
* Record subsequent corrections as separate dated amendments that identify the superseded requirements.
* Keep a local handoff index that links the original plan, all amendments in application order, and the latest delivery evidence.
* Record public acceptance evidence in the issue or PR without publishing the implementation plan.
* Read the complete plan and applicable amendments before edits or delegated work, including all invariants.
* Preserve prior-stage evidence when an amendment changes later implementation boundaries.

Source: accepted development approach from issue #1 review, 2026-10-06.

Source: conscious generic-tape deferral in maintenance [#54](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/54), 2026-10-10.

Source: accepted full-plan preservation correction, 2026-10-10, and the separate calculation-ownership amendment for [#56](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/56), 2026-10-11.

Source: approved context cleanup, 2026-10-11, after a later asset-registration amendment was missing from the earlier handoff navigation.

Source: accepted correction for #23, 2026-10-11, to remove `docs/plans/23-altitude-baro` from the repository and PR and keep future implementation plans local.

Source: accepted detailed-plan correction for #63, 2026-10-11.

Evidence limit: no verified causal link connects plan abbreviation with earlier defects.
