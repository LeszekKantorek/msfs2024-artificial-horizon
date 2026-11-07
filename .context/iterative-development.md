---
description: Execute Subtasks, verify context, consult Sol, preserve accepted decisions, and report findings in the main conversation.
status: active
---

The [work organization model](../docs/work-organization.md) defines the hierarchy and responsibilities.

Select the procedure that matches the requested result:

* Use [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) to assess material structural choices against the current outcome and next agreed use.
* Use [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) to turn the accepted choice into executable Subtasks.

Apply these scope and component-boundary requirements:

* Keep speculative frameworks and later issues outside current scope.
* Do not postpone useful structural changes solely because the current skeleton works.
* Keep the [separate altitude tape](../docs/efi/altitude.md#rationale-and-regression-lessons) unless actual shared requirements justify reconsidering its scale, sign, digit, selection, and alert boundaries.
* Use the [internal seams](../docs/efi/composition.md#visual-object-interfaces-and-ownership) first instead of inferring shared rules from a planned instrument.
* Apply the [PFD composition rules](../docs/efi/composition.md#rationale-and-regression-lessons) for calculation ownership and extraction to actual shared consumers.

## Accepted plans and handoffs

Prepare and preserve handoffs under these requirements:

* Use a self-contained Subtask as the primary execution handoff. Read its complete instructions, decisions, invariants, and applicable amendments before edits or delegation.
* Preserve supplied accepted plans verbatim and locally, including their original language. Retain existing plans and their amendments without requiring future handoffs to use `.local/plans/`.
* If a supplied plan governs the Subtask, preserve its requirements in the handoff and identify the governing source and amendments.
* Record subsequent corrections explicitly, with the date and the requirements they supersede. Do not silently rewrite accepted decisions or discard earlier evidence.
* Keep optional local indexes and notes as supplements. Another session must not depend on their continued availability to understand a Subtask.
* Publish Subtask instructions and delivery evidence only within authorized GitHub work. Do not commit supplied plan originals or use Issues as a running progress notebook.

### Prepare Subtask context

Delegate [context-retrieve](../.agents/skills/context-retrieve/SKILL.md) to a Luna High subagent under the [retrieval delegation rules](../AGENTS.md#context-selection) when preparing implementation context. Describe the requirements, decisions, code boundaries, or validation information needed, with the known issue identifiers and constraints. Use its verified references and missing-information report before dependent implementation; the implementing agent remains responsible for reading and interpreting the governing sources.
For work without an issue, derive the retrieval question from the requested outcome. For a Subtask, prepare the context in this order:

1. Read the Subtask and its Task through [issue and artifact retrieval](repository-instructions.md#read-github-issue-context).
   Use context-retrieve to locate the source sections required for its scope, then read those passages. Establish the intended result, constraints, dependencies, and validation. A source-location report does not replace complete Subtask instructions.
2. Check the current revision and relevant changes. Verify cited files, sections, line ranges, and consequential assumptions against current code and contracts.
3. If lines have moved, locate the intended passage by meaning and record a consequential mismatch. Line references remain valid handoff tools when checked.
4. Resolve routine implementation details within scope. Use accepted decisions without redesigning them merely because the Subtask entered a new session.
5. If missing information or a contradiction requires broader analysis, use [Consult Sol](#consult-sol). Continue independent authorized work while the dependent part remains unresolved.

### Consult Sol

Use consultation when any of these conditions requires broader analysis:

* A requirement or consequential technical decision is missing from the Subtask.
* A contract is ambiguous or the required sources conflict.
* An unexpected dependency requires analysis beyond routine implementation.

Start the consultation when the need is established, without waiting for a fixed number of failed attempts:

1. State the problem, affected criterion, relevant code and source sections, evidence, attempted checks, and one concrete question.
2. Start a Sol Medium subagent when active instructions authorize consultation delegation and the environment supports it:
   * Check model-selection capabilities before requesting a model.
   * Do not claim that a model or reasoning setting was selected when the host cannot select it.
3. Sol compares viable solutions and chooses the best supported technical solution within the accepted scope. Routine technical decisions do not require human confirmation.
4. If a solution conflicts with approved requirements or changes project direction, apply these requirements:
   * Record the conflict and obtain the human decision before dependent work.
   * Continue possible work within the accepted scope.
   * Keep unmet criteria unconfirmed.
5. Terra records the question, considered options, chosen solution, rationale, and subsequent result in the main conversation, then continues execution.

If delegation or the requested model is unavailable, apply these requirements:

* Report the unavailable capability without silently substituting another model.
* Continue independent work and routine decisions within the accepted scope.
* Record unresolved dependent work explicitly. Do not claim that an unavailable consultation or incomplete Subtask was completed.

### Record execution findings

Preserve execution findings under these requirements:

* Record missing Subtask information, context gaps, contradictions, and important independent decisions in the main conversation, including problems resolved successfully.
* Record consultation reasons, questions, options, decisions, rationale, and whether the resulting solution worked. A subagent transcript alone is insufficient.
* During long Subtasks, record consequential findings in the main conversation as they arise to reduce loss if execution is interrupted.
* An optional journal under Git-ignored `.local/` can hold the Subtask identifier, problem, consultation, decision, and result during execution.
* Before finishing, transfer all significant journal findings into the main conversation. Local files, test reports, and separate subagent sessions might be unavailable later.
* Keep full test logs local. Preserve the meaning of evidence and its limits in the conversation without copying entire logs or analyses.

This process requires no separate consultation database, synchronization, new skill, or automatic GitHub publication.

### Final Subtask summary

Finish in the main conversation with a self-contained summary covering every field below.

| Field | Required content |
| --- | --- |
| Work performed | What was implemented, changed, investigated, or established |
| Verification | Tests and review actually performed, results, reused evidence, and evidence limits |
| Discovered gaps | Significant deficiencies in the plan, requirements, context, or instructions, including resolved gaps |
| Sol consultations | Reasons, questions, considered solutions, selected decisions, and rationale |
| Terra decisions | Important ambiguities resolved independently and the reasons for those choices |
| Decision results | Whether the selected solutions worked and how they affected execution |
| Unresolved problems | Remaining limitations, risks, missing information, and unconfirmed criteria |
| Planning feedback | What similar Subtasks should include to prevent unnecessary analysis or consultation |

The summary must satisfy these completeness requirements:

* State explicitly when there were no significant gaps, consultations, independent decisions, or unresolved problems.
* Include the information in the table rather than substituting a completion claim, file list, or link to a local journal.
* Preserve enough information for JSON session review through the existing Context Skills without access to temporary files or subagent histories.

## Findings outside the current scope

Handle findings without expanding the authorized scope:

* Record the observation, expected behavior, supporting evidence, and status without expanding the authorized implementation scope.
* Distinguish accepted requirements from implemented behavior. A documented target does not establish that the code already meets it.
* Keep unresolved proposals separate from agreed follow-up work.
* Record the finding in the main conversation. Preserve reusable knowledge in its documentation owner through the applicable Context Skill.
* Before removing a local record, preserve agreed outstanding work in an authorized destination that does not depend on that record.
* Use [finding promotion](../CONTRIBUTING.md#promote-agreed-findings) for self-contained follow-up issues or further planning of an open Task. Publication still requires authorization.
