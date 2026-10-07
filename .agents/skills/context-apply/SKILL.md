---
name: context-apply
description: Read the .context index and relevant knowledge entries, then apply them to the current task. Use when planning, implementing, validating, or resuming work. Use context-gather to create, update, or review stored knowledge.
---

# Apply Context

## Step 1: Read the index

* **Start with `.context/INDEX.md`.** Reuse an existing project index if it has another name.
* Select entries by their `Use when` conditions and the current task.
* Open the linked entries before acting; the index is a navigation table, not the full knowledge.
* If the index is missing, inspect relevant `.context/` files directly and report the missing index.
* Exclude `.context/sessions/`; session files belong to `context-import-sessions`.

## Step 2: Resolve the applicable knowledge

* Read each selected entry's `description`, `status`, and relevant content.
* Reuse knowledge already read in this session when it still applies.
* Check important claims against current evidence when they may have changed.
* Preserve the distinction between an accepted decision, implemented behavior, and a hypothesis.
* If knowledge is missing, inspect the necessary project sources and state unresolved gaps.

| Status | Action |
| --- | --- |
| `active` | Apply within its stated scope; preserve any uncertainty. |
| `superseded` | Follow `superseded_by`, relative to the entry, to its replacement. |
| `deprecated` | Treat the entry as withdrawn. |

* Stop a replacement chain at a broken link or cycle and report the defect.
* Resolve conflicting claims from their evidence and authority; do not invent a missing decision.

## Step 3: Apply the knowledge

* **Turn each relevant rule into an action or check.** State the consequence in the task's plan, change, or result.
* Example: if reopening an export must preserve its original snapshot, implement and check that behavior.
* Continue the requested task without a separate context report.
* Distinguish checks actually run from checks merely planned.
* Use `context-gather` when durable learning should be saved or stored knowledge needs repair.

> Reading an entry is not enough. The resulting action must respect it.
