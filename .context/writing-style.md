---
description: Write concise, impersonal project text, including documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs. Choose document responsibilities.
status: active
---

* Apply these writing rules to documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Use short English sentences and consistent terms.
* Keep supplied accepted plans local and verbatim in their original language under the [plan-preservation rule](iterative-development.md#accepted-plans-and-handoffs).
* Use `*` bullets for parallel rules and numbered steps for sequences.
* Introduce long lists or distinct groups of bullets with one short sentence that states their topic or purpose.
* A heading can introduce a list when its scope is clear, but a short lead-in can add useful context.
* Separate groups by subject instead of creating one uninterrupted list or repeating the heading in a filler sentence.
* Use tables for comparisons, `>` notes for constraints, and diagrams for flows.
* Format identifiers as inline code and commands as fenced code blocks.
* Use `[ ]` checklists for procedures or criteria, not duplicate task status.
* Preserve requirements, units, limits, uncertainty, and the distinction between plans, implementation, and device acceptance.
* Before removing moved or shortened text, verify where each requirement, constraint, and caveat remains in the canonical documentation.
* Check sentence length, ambiguity, and consistent terminology directly using these project rules.
* Check Markdown grouping, procedures, and tables separately from sentence clarity.
* Check rendered paragraph boundaries because single newlines do not separate Markdown paragraphs.
* Link to the canonical source instead of repeating commands or rules.

## Impersonal project records

* Describe results, decisions, procedures, and evidence without personal narration.
* Apply this rule to all project text covered above.
* Avoid references such as "the owner", "owner-assisted", "the user reported", or "the assistant verified".
* Use wording such as "Tested on...", "Confirmed using...", "Based on documentation...", and "Accepted for this iteration...".
* Use "Reported..." for supplied observations and "Verified..." only for checks directly performed.
* Preserve sources, dates, environments, missing details, uncertainty, and approval requirements.
* Keep technical ownership terms when they describe resource management or responsibilities.

Source: writing requirement recorded in [issue #31](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/31), 2026-10-09.

## Document responsibilities

| Source | Owns |
| --- | --- |
| [README](../README.md) | First run, phone access, available behavior, limitations |
| [CONTRIBUTING](../CONTRIBUTING.md) | Developer setup, context tooling, contribution workflow |
| [Brief](../docs/project-brief.md) | Requirements, scope, assumptions, success criteria |
| [Architecture](../docs/architecture.md) | Target design, boundaries, library API |
| [Panel presentation](../docs/pfd-presentation.md) | Instrument interfaces, frame lifecycle, layer ownership, clipping |
| [Contract](../docs/telemetry-contract.md) | Wire interface, units, validity, freshness |
| [Testing](../docs/testing.md) | Check commands, procedures, criteria, evidence templates |
| [Roadmap](../docs/roadmap.md) | Order and dependencies |
| [ADRs](../docs/adr/) | Decisions and decision lifecycle, not implementation status |
| GitHub Issues / PRs | Task status, execution results, acceptance evidence |

Source: accepted writing style, document responsibilities, and setup decisions, 2026-10-09.

Source: accepted documentation correction, 2026-10-10, to introduce long bullet groups with short sentences that explain their scope.

Source: documentation structure correction for #56 and PR #55, 2026-10-11, separating Markdown checks from language checks.

Source: formatting correction for `docs/testing.md` in PR #60, 2026-10-11, to group independent checks instead of stacking prose lines.
