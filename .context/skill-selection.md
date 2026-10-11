---
description: Select complementary skills for planning, development, testing, review, documentation, security, and architecture decisions.
status: active
---

## Practical selection

| Task | Suggested selection |
| --- | --- |
| Project knowledge | `context-apply` to retrieve rules; `context-gather` to save durable learning; `context-import-sessions` with `context-gather` to analyze earlier sessions. |
| Planning and module design | `brainstorming` for unresolved scope and design choices; `codebase-design` for module boundaries and interfaces. |
| Interface development | `frontend-expert` for implementation and rendering; `ux-expert` for readability and phone use. |
| Test implementation and diagnosis | `designing-tests` for behavioral checks; `systematic-debugging` for failed tests and other faults. |
| Test strategy and acceptance | `quality-expert` to select tests by risk and define acceptance criteria. |
| Review | `requesting-code-review` for independent review; `receiving-code-review` to assess feedback; `quality-expert` to assess test coverage and failure scenarios. |
| Verification | `verification-before-completion` for evidence applicable to the tested state; `quality-expert` to assess evidence against acceptance criteria. |
| Writing | `asd-ste100` for precise language; `diataxis` for document purpose and structure. |
| Security | `security-expert` for threats, network access, dependencies, and trust boundaries. |
| Major architecture decision | `engineering-expert` to compare options and tradeoffs, or `clean-architecture` to design boundaries and dependencies. |

* For each implementation-plan step, name the skills that support that step.
* Select skills for their distinct roles in the task; the table does not require loading every skill in a row.
* Follow the selected skill instructions within the accepted task scope and [repository rules](repository-instructions.md).
* Use [agent maintenance](agent-maintenance.md) for installation and hook changes.

## Planning and verification scope

These project adaptations preserve the accepted task scope:

* Read the accepted plan and its amendments before choosing a planning path.
* An explicit instruction to implement that plan authorizes its existing scope without another request for the same approval.
* Ask about unresolved requirements or material changes outside that scope before implementing the dependent work.
* Use a short in-chat design for bounded changes and a written plan when architectural coordination requires one.
* If `writing-plans` is unavailable, prepare the necessary plan directly using the [plan and handoff rules](iterative-development.md#accepted-plans-and-handoffs).
* A missing optional planning skill does not require installing dependencies or stopping an otherwise authorized task.
* Select checks using [validation by change scope](../docs/testing.md#validation-by-change-scope).
* Apply [evidence reuse](../docs/testing.md#evidence-reuse) before repeating checks because a new message alone does not invalidate evidence.
* Preserve an explicit requirement for a fresh final run and any task restriction on execution.
* Report unperformed checks and evidence limits without claiming a pass.

The local brainstorming and verification skills implement these adaptations.
[Agent maintenance](agent-maintenance.md) describes their availability and reinstall limits.

Source: accepted project skill selection, including testing, review, verification, security, architecture, and writing additions, 2026-10-10.

Source: implementation-plan skill annotations requested for #36, 2026-10-10.

Source: accepted project-maintenance review, 2026-10-11, covering existing approvals, missing planning dependencies, and applicable verification evidence.
