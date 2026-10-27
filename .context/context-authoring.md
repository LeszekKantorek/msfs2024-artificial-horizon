---
description: Keep project knowledge concise and selective. Use when editing .context entries or their index.
status: active
---

* Retain decisions, explanations, findings, and rules that help a future task, with their rationale and conditions of use.
* Link to canonical documents instead of copying their content.
* Remove session progress and expired task-specific permissions from reusable knowledge.
* Preserve conditions, uncertainty, and evidence limits. Omit the history and provenance of documentation changes.
* Use concise technical English and the [writing style](writing-style.md).
* Use tables or diagrams only when they reduce reading effort.
* Keep independent topics separate so agents can read only relevant entries.
* Put direct routes in the index and conditional routes in [task guidance](task-guidance.md).
* Keep procedures in their owning sources.
* For compiled procedures, follow the [skill ownership table](agent-maintenance.md#project-procedure-skills). Keep publicly required rationale and evidence in documentation; edit steps only in their designated owner.
* Retain scoped evidence limits when consolidating duplicate guidance.
* Select the appropriate [context skill](../AGENTS.md#project-knowledge) for preservation, maintenance, mapping, diagnosis, or session review.

## Local entry and index convention

These formats are project conventions. The installed context skills do not prescribe them.

| Field / record | Project rule |
| --- | --- |
| `description` | State the entry's subject and when its knowledge is useful |
| `status` | Use `active`, `superseded`, or `deprecated` |
| `superseded_by` | Required for `superseded`; resolve the replacement path relative to the entry |
| Active entry | Apply within its stated scope, retaining evidence limits and uncertainty |
| Superseded entry | Follow the replacement; report broken paths or cycles instead of guessing |
| Deprecated entry | Treat as withdrawn; retain the reason and useful history |
| `.context/index.md` | Keep a table-only map with each active entry once and concrete selection conditions |
| Direct documentation routes | Include recurring tasks that can go directly to a canonical document or section |

Exclude session metadata and transcripts under [AGENTS](../AGENTS.md#project-knowledge).
When moving knowledge, update incoming links, index rows, and replacement paths.
Check that representative tasks still reach the applicable requirements, rationale, and evidence limits.

## Documentation boundary

Follow [documentation ownership](../AGENTS.md#documentation-ownership) for the boundary between agent knowledge and project/user documentation.
Use the [PFD content ownership map](../docs/efi/index.md#content-ownership) to place feature and shared information.
