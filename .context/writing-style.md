---
description: Write concise, impersonal project text, including documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs. Choose document responsibilities.
status: active
---

* Apply these writing rules to documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Use short English sentences and consistent terms.
* Use `*` bullets for parallel rules and numbered steps for sequences.
* Use tables for comparisons, `>` notes for constraints, and diagrams for flows.
* Format identifiers as inline code and commands as fenced code blocks.
* Use `[ ]` checklists for procedures or criteria, not duplicate task status.
* Preserve requirements, units, limits, uncertainty, and planned/implemented distinctions.
* Apply `asd-ste100` when available without claiming certified dictionary compliance.
* Link to the canonical source instead of repeating commands or rules.
* For each PR that adds a feature, include a short checklist tailored to the implemented behavior and the manual checks it needs.
* Name concrete actions or expected results in checklist items, without generic placeholders or detailed steps.

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
| [CONTRIBUTING](../CONTRIBUTING.md) | Developer setup, tools, skill sources, workflow |
| [Brief](../docs/project-brief.md) | Requirements, scope, assumptions, success criteria |
| [Architecture](../docs/architecture.md) | Target design, boundaries, library API |
| [Contract](../docs/telemetry-contract.md) | Wire interface, units, validity, freshness |
| [Testing](../docs/testing.md) | Check commands, procedures, criteria, evidence templates |
| [Roadmap](../docs/roadmap.md) | Order and dependencies |
| [ADRs](../docs/adr/) | Decisions and decision lifecycle, not implementation status |
| GitHub Issues / PRs | Task status, execution results, acceptance evidence |

Source: accepted writing style, document responsibilities, and setup decisions, 2026-10-09.

Source for manual PR checklists: accepted requirement and correction, 2026-10-09, to write feature-specific checks instead of generic template items.
