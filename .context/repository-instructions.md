---
description: Apply repository constraints when changing runtime behavior, dependencies, delivery boundaries, or preparing a contribution. Select source sections for the affected scope.
status: active
---

## Steps

1. Apply the [project constraints](../AGENTS.md#project-constraints) and identify the behavior or contribution stage being changed.
2. Select source sections from the conditional routes below, reusing applicable content already read.
3. Distinguish target design and SDK assumptions from verified behavior.
4. Stop discovery when requirements, constraints, and validation are clear, expanding only for relevant gaps or dependencies.

## Conditional sources

| Affected scope | Read the relevant section |
| --- | --- |
| Startup, phone access, or user-facing instructions | [README](../README.md) |
| Ownership, lifecycle, module boundaries, or deployment | [Architecture](../docs/architecture.md) and the ADR for the changed decision |
| Telemetry fields, units, compatibility, validity, or freshness | [Telemetry contract](../docs/telemetry-contract.md) |
| Check selection or acceptance evidence | [Validation by change scope](../docs/testing.md#validation-by-change-scope), then applicable procedures |
| Branch or PR preparation | [Contribution workflow](../CONTRIBUTING.md#workflow) |
| Issue or PR acceptance checklists | [Acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) |
| Preserving an accepted plan or amendments | [Local handoff rules](iterative-development.md#accepted-plans-and-handoffs) |

These routes do not require reading complete documents or following every link within a selected source.

## Repository rules

* When writing project text, apply the relevant [writing rules](writing-style.md).
* When changing dependencies, commit `Cargo.lock` and exclude dependencies without distribution rights.
* When making material architecture changes, record the decision in `docs/adr/`.
* When adding tests, check behavior rather than repeating implementation details.
* When changing runtime lifecycle, preserve cancellation cleanup under the [HTTP startup checks](../docs/testing.md#http-startup-checks).
* When preparing a PR, follow the contribution workflow and its manual checklist requirement for new features.
* Include `Closes #N` in implementation PRs, including drafts, under the acceptance tracking rules.
* Read [PR acceptance](pr-acceptance.md) when changing or activating the checklist control.

## Definition of done

Apply only the criteria relevant to the task, using every applicable row of the validation scope matrix.

* [ ] Acceptance criteria have supporting evidence.
* [ ] Relevant [automated checks](../docs/testing.md#automated-checks) pass.
* [ ] Manual results identify the environment and observations separately from unavailable hardware checks.
* [ ] Runtime changes address errors, source loss, and reconnects.
* [ ] Telemetry changes include compatible fixtures and consumer updates.
* [ ] Documentation distinguishes planned behavior, implemented behavior, and confirmed device acceptance.

## Follow-up findings

* Collect reusable implementation findings in [#9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9), referencing the originating task.
* Review those findings after the originating task closes before defining follow-up scope.
* Use [finding promotion](../CONTRIBUTING.md#promote-agreed-findings) when creating an agreed follow-up issue.

Sources: repository CONTRIBUTING rules, moved here by an accepted documentation decision on 2026-10-09, and the linked project documents.

Source: accepted PR checklist requirement and corrections, 2026-10-09. CONTRIBUTING owns this workflow rule.

Source: cancellation cleanup verified in maintenance [#45](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/45), 2026-10-10.

Source: user correction, 2026-10-10, to include automatic issue closure in implementation PRs and record pending verification separately.

Source: accepted acceptance-tracking rule and simplified control, 2026-10-10: mark verified issue criteria, record remaining verification in the PR, and check both lists without comparing wording.

Source: accepted conditional routing change, 2026-10-11, replacing mandatory full-document reads with scope-specific source sections.
