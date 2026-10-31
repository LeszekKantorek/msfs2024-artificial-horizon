---
description: Write concise, impersonal project text, including documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs. Choose document responsibilities.
status: active
---

* Apply these writing rules to documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Use short English sentences and consistent terms.
* Keep supplied accepted plans local and verbatim in their original language under the [plan-preservation rule](iterative-development.md#accepted-plans-and-handoffs).
* Select lists, tables, steps, and prose through [Markdown structure](#markdown-structure).
* Introduce long lists or distinct groups of bullets with one short sentence that states their topic or purpose.
* A heading can introduce a list when its scope is clear, but a short lead-in can add useful context.
* Separate groups by subject instead of creating one uninterrupted list or repeating the heading in a filler sentence.
* Use `>` notes for constraints and diagrams for flows.
* Format identifiers as inline code and commands as fenced code blocks.
* Use `[ ]` checklists for procedures or criteria, not duplicate task status.
* Preserve requirements, units, limits, uncertainty, and the distinction between plans, implementation, and device acceptance.
* Before removing moved or shortened text, verify where each requirement, constraint, and caveat remains in the canonical documentation.
* Check sentence length, ambiguity, and consistent terminology directly using these project rules.
* Check [Markdown structure](#markdown-structure) separately from sentence clarity and technical accuracy.
* Link to the canonical source instead of repeating commands or rules.

## Markdown structure

* For feature pages, follow the section structure and Markdown conventions of one representative page of the same type.

Choose the construct by the information it carries:

| Information | Markdown structure |
| --- | --- |
| Independent requirements, rules, or constraints | `*` bullets, grouped by subject |
| Parameters, states, variants, comparisons, or mappings | A table with consistent fields |
| Actions that must occur in order | A numbered list with one main action per step |
| An introduction, mechanism, or rationale that needs connected reasoning | A short paragraph |

Review the choice of structure independently from the wording:

* Review each created or changed section as a whole, including unchanged surrounding text affected by the edit.
* Classify independent facts, mappings, ordered actions, and connected explanations before choosing their Markdown structure.
* For feature pages, compare the document with the representative page, including lists, tables, steps, and paragraph grouping.
* Keep reference facts and independent rules in lists or tables instead of narrative blocks.
* Inspect rendered Markdown. Headings and short sentences alone do not establish format consistency.
* Do not substitute one sentence per source line for a list or table. Single newlines remain one rendered paragraph.
* Retain prose when it explains a necessary relationship. Do not convert connected reasoning into arbitrary bullet fragments.

## Impersonal project records

* Describe results, decisions, procedures, and evidence without personal narration.
* Apply this rule to all project text covered above.
* Avoid references such as "the owner", "owner-assisted", "the user reported", or "the assistant verified".
* Use wording such as "Tested on...", "Confirmed using...", "Based on documentation...", and "Accepted for this iteration...".
* Use "Reported..." for supplied observations and "Verified..." only for checks directly performed.
* Preserve sources, relevant environments, missing details, uncertainty, and approval requirements. Keep change dates and work status in the owning issue or evidence record.
* Keep technical ownership terms when they describe resource management or responsibilities.

## Document responsibilities

| Source | Owns |
| --- | --- |
| [README](../README.md) | First run, phone access, available behavior, limitations |
| [CONTRIBUTING](../CONTRIBUTING.md) | Developer setup, context tooling, contribution workflow |
| [EFI scope](../docs/efi/index.md#product-scope-and-reference) | Product coverage, exclusions, reference illustration, and terminology |
| [Architecture](../docs/architecture.md) | End-to-end flow, boundaries, application lifecycle, deployment, and fundamental library contracts |
| [Server](../docs/server.md) | HTTP API, routers, embedded assets, health, SSE delivery, and connection shutdown |
| [Sources](../docs/sources/index.md) | Shared acquisition responsibilities and provider navigation |
| [Demo](../docs/sources/demo.md) | Deterministic scenarios, source API, timing, and validation references |
| [SimConnect](../docs/sources/simconnect.md) | Native configuration, API, diagnostics, implementation, tests, and evidence limits |
| [Panel presentation](../docs/efi/index.md) | Feature requirements, presentation, implementation, rationale, and acceptance scenarios; shared composition, lifecycle, and rendering contracts |
| [Contract](../docs/telemetry-contract.md) | Wire interface, units, validity, freshness |
| [Testing](../docs/testing/index.md) | Shared check commands, runtime and device procedures, validation selection, and evidence templates; feature scenarios remain on feature pages |
| [Roadmap](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) | Order and dependencies |
| [ADRs](../docs/adr/) | Decisions and decision lifecycle, not implementation status |
| GitHub Issues / PRs | Task status, execution results, acceptance evidence |

## Instrument terminology

Use [project terminology](../docs/efi/index.md#terminology): EFI for the complete instrument, artificial horizon for the attitude indication.
Keep technical identifiers consistent with their consumers.
