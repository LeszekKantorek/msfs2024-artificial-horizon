---
description: Apply the owner's preference for concise technical documentation. Use when editing docs or reviewing the local documentation prototype.
status: active
---

* Keep repository documentation in English, as required by [repository rules](repository-instructions.md#repository-rules).
* Use short technical sentences and consistent terms.
* Use `*` bullets for parallel rules, numbered procedures for ordered steps, tables for comparisons, and `>` notes for constraints.
* Format commands and identifiers as code, with fenced blocks for runnable commands.
* Use diagrams when they clarify flow or dependencies.
* Use checklists for procedures and acceptance criteria, not duplicated issue status.
* Preserve requirements, uncertainty, numeric limits, and distinctions between implemented and planned behavior.
* Apply the user-selected `asd-ste100` skill when available without claiming certified dictionary compliance.
* Treat the documentation rewrite, including the root guides, as a local prototype pending owner review.

## Document responsibilities

| Source | Owns |
| --- | --- |
| [README](../README.md) | First run, phone access, available behavior, user-facing limitations |
| [CONTRIBUTING](../CONTRIBUTING.md) | Developer environment setup, required tools, skill installation sources, contribution workflow |
| [Project brief](../docs/project-brief.md) | Requirements, scope, assumptions, success criteria |
| [Architecture](../docs/architecture.md) | Target design, component boundaries, library API |
| [Telemetry contract](../docs/telemetry-contract.md) | Wire interface, units, validity, freshness |
| [Testing](../docs/testing.md) | Check commands, procedures, acceptance criteria, evidence templates |
| [Roadmap](../docs/roadmap.md) | Delivery order and dependencies |
| [ADRs](../docs/adr/) | Architectural decisions and their lifecycle |
| GitHub Issues / PRs | Task status, execution results, acceptance evidence |

* Keep implementation status and completed/pending task lists in GitHub Issues.
* Do not duplicate command catalogs or task status across documents.
* Link to the document that owns the detail.
* Distinguish planned behavior from implemented behavior.
* ADR status describes a decision, not task completion.

## Source

* Owner's documentation request on 2026-10-09 explicitly selected `asd-ste100` and requested this presentation style.
* The owner prohibited branch and GitHub task creation for this prototype and initially prohibited commits.
* The owner later authorized a local checkpoint commit of the prototype without publishing it.
* The follow-up request on 2026-10-09 extended the prototype to README, AGENTS, and CONTRIBUTING and requested clearer first-use guidance.
* The owner requested moving rules and project context from CONTRIBUTING into knowledge entries on 2026-10-09.
* The owner clarified that CONTRIBUTING should retain workflow and practical environment setup, including required tools and skill sources.
