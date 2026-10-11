---
description: Choose module boundaries for agreed delivery and preserve accepted plans locally without publishing them to GitHub.
status: active
---

1. Identify the current outcome and its next agreed use in the [roadmap](../docs/roadmap.md).
2. Assess boundaries against those uses and the [architecture](../docs/architecture.md).
3. Choose small structural changes that simplify the next agreed step.
4. Explain each choice's immediate cost and next use.
5. Check that the current outcome still works.

* Keep speculative frameworks and later issues outside current scope.
* Do not postpone useful structural changes solely because the current skeleton works.
* Reassess a general IAS/altitude tape during actual altitude implementation against its scale, sign, digit, selection, and alert rules.
* Use the implemented [internal seams](../docs/pfd-presentation.md#internal-module-seams) first instead of inferring shared rules from a planned instrument.
* Keep calculations with their visual objects unless actual independent consumers justify extraction, as recorded in [ADR 0005](../docs/adr/0005-visual-object-composition.md).

## Accepted plans and handoffs

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

Evidence limit: no verified causal link connects plan abbreviation with earlier defects.
