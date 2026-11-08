---
description: Select project context, delegate source retrieval, and pass its results to agents.
status: active
---

## Select sources and assign context preparation

Use these rules before reasoning or planning that depends on project knowledge. Apply them when preparing implementation context, including work without an issue.

| Situation | Responsible agent | Action |
| --- | --- | --- |
| Required sources are known and sufficient | Agent preparing context | Read the relevant sections directly, including the conditional readings below. |
| Relevant context or verified retrieval results remain applicable and unchanged | Agent preparing context | Reuse them without repeating retrieval. Read any still-missing governing sources. |
| Required sources are unknown or incomplete | Agent preparing context | Delegate a bounded question to one Luna High subagent under the rules below. |
| A dependency, conflict, missing fact, or scope change requires more context | Agent preparing context | Return to the maps. Retrieve additional context if the needed sources remain unknown or incomplete. |
| The requesting agent prepares a plan and execution context before delegating implementation | Requesting agent | Select and read sources. If retrieval is needed, wait for its result before starting the implementing agent. Pass the execution context and any retrieval report to that agent. |
| The user delegates the complete task or prompt to a subagent | Requesting agent and task subagent | The requesting agent passes the complete task and applicable instructions. The subagent selects and reads its sources and delegates retrieval when required. |
| Another agent's work depends on retrieval results | Requesting agent | Wait for the results before starting that agent and pass the report to it. |
| Retrieval and another task are independent | Requesting agent | They can run in parallel. Do not assume the other agent received results produced later. |

* Knowing a source path does not establish that it covers every applicable requirement. Read governing requirements and complete Subtask instructions; a retrieval report does not replace them.
* Do not independently repeat the same retrieval in both agents. The implementing or task subagent can retrieve additional context when its sources are insufficient.
* A subagent can ask the requesting agent whether retrieval was performed. If it was, the requesting agent passes the report. Include every report in full and exactly as received, with paths, line ranges, descriptions, gaps, and access limitations. Keep interpretations and additional instructions separate from the report.

## Delegate retrieval

1. Check that Luna High (`gpt-6-luna`, reasoning effort `high`) is available. If unavailable, report it instead of substituting another model.
2. Give one Luna High subagent a bounded source-location question. Specify the information needed, known identifiers and references, scope, and reading constraints.
3. Require the subagent to execute [context-retrieve](../.agents/skills/context-retrieve/SKILL.md) directly, without further delegation. Its work is read-only: it does not edit, run tests, implement, or choose the technical direction.
4. Use its verified paths, line ranges, brief descriptions, and missing or inaccessible information to select sources. Read the relevant passages yourself. You remain responsible for interpretation, planning, and implementation.

## Select maps and readings

| Need | Map |
| --- | --- |
| Change behavior across component boundaries | [Changes across components](component-changes.md) |
| Understand or reconsider an accepted architecture choice | [Governing decisions](architecture-decisions.md#find-the-governing-decision) |
| Find instrument sources or additional guidance for implementation, diagnosis, validation, review, documentation, or delivery | [Task guidance](task-guidance.md#practical-selection) |

Follow applicable map rows and read only the sections needed. Apply every relevant route for changes across boundaries, including validation, acceptance, and accepted-plan invariants. Use the [index](index.md) as the final fallback when the maps do not identify a required source.

Apply these conditional readings:

* Before issue-based planning or execution, read the [GitHub issue context and artifacts](repository-instructions.md#read-github-issue-context).
* For startup or user-facing behavior, read README. For changed boundaries, read architecture. For the applicable contribution stage, read CONTRIBUTING.
* When validating a change, select required checks from the [validation scope matrix](../docs/testing/index.md#validation-by-change-scope).
