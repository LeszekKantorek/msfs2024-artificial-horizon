---
description: Choose document ownership and write concise technical documentation.
status: active
---

* Use short English sentences and consistent terms.
* Use `*` bullets for parallel rules and numbered steps for sequences.
* Use tables for comparisons, `>` notes for constraints, and diagrams for flows.
* Format identifiers as inline code and commands as fenced code blocks.
* Use `[ ]` checklists for procedures or criteria, not duplicate task status.
* Preserve requirements, units, limits, uncertainty, and planned/implemented distinctions.
* Apply `asd-ste100` when available without claiming certified dictionary compliance.
* Link to the canonical source instead of repeating commands or rules.

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

Source: owner's style, document ownership, and setup decisions, 2026-10-09.
