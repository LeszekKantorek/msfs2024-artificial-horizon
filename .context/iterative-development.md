---
description: Execute Tasks, verify context, consult Sol, preserve accepted decisions, and report findings in the main conversation.
status: active
---

Use [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) to assess material structural choices against the current outcome and next agreed use.
Use [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) to turn the accepted choice into executable Tasks.
The [work organization model](../docs/work-organization.md) defines the hierarchy and responsibilities.

* Keep speculative frameworks and later issues outside current scope.
* Do not postpone useful structural changes solely because the current skeleton works.
* Keep the [separate altitude tape](../docs/efi/altitude.md#rationale-and-regression-lessons) unless actual shared requirements justify reconsidering its scale, sign, digit, selection, and alert boundaries.
* Use the [internal seams](../docs/efi/composition.md#visual-object-interfaces-and-ownership) first instead of inferring shared rules from a planned instrument.
* Apply the [PFD composition rules](../docs/efi/composition.md#rationale-and-regression-lessons) for calculation ownership and extraction to actual shared consumers.

## Accepted plans and handoffs

* Use a self-contained Task as the primary execution handoff. Read its complete instructions, decisions, invariants, and applicable amendments before edits or delegation.
* Preserve supplied accepted plans verbatim and locally, including their original language. Retain existing plans and their amendments without requiring future handoffs to use `.local/plans/`.
* If a supplied plan governs the Task, preserve its requirements in the handoff and identify the governing source and amendments.
* Record subsequent corrections explicitly, with the date and the requirements they supersede. Do not silently rewrite accepted decisions or discard earlier evidence.
* Keep optional local indexes and notes as supplements. Another session must not depend on their continued availability to understand a Task.
* Publish Task instructions and delivery evidence only within authorized GitHub work. Do not commit supplied plan originals or use Issues as a running progress notebook.

### Prepare Task context

1. Read the Task, its Work Item, and the source sections required for its scope. Establish the intended result, constraints, dependencies, and validation.
2. Check the current revision and relevant changes. Verify cited files, sections, line ranges, and consequential assumptions against current code and contracts.
3. If lines have moved, locate the intended passage by meaning and record a consequential mismatch. Line references remain valid handoff tools when checked.
4. Resolve routine implementation details within scope. Use accepted decisions without redesigning them merely because the Task entered a new session.
5. If missing information or a contradiction requires broader analysis, use [Consult Sol](#consult-sol). Continue independent authorized work while the dependent part remains unresolved.

### Consult Sol

Consult when a missing requirement, ambiguous contract, conflicting source, unexpected dependency, or unresolved technical choice needs broader analysis.
Consult as soon as that need is clear. No fixed number of failed attempts is required.

1. State the problem, affected criterion, relevant code and source sections, evidence, attempted checks, and one concrete question.
2. Start a Sol Medium subagent when active instructions authorize consultation delegation and the environment supports it.
   Check model-selection capabilities before requesting a model. Do not claim that a model or reasoning setting was selected when the host cannot select it.
3. Sol compares viable solutions and chooses the best supported technical solution within the accepted scope. Routine technical decisions do not require human confirmation.
4. If a solution conflicts with approved requirements or changes project direction, record the conflict. Obtain the human decision before dependent work.
   Continue possible work within the accepted scope. Do not mark unmet criteria complete.
5. Terra records the question, considered options, chosen solution, rationale, and subsequent result in the main conversation, then continues execution.

If delegation or the requested model is unavailable, report the limitation without silently substituting a model.
Continue independent work and routine decisions within scope. Keep the unresolved dependent work explicit rather than claiming the consultation or Task completed.

### Record execution findings

* Record missing Task information, context gaps, contradictions, and important independent decisions in the main conversation, including problems resolved successfully.
* Record consultation reasons, questions, options, decisions, rationale, and whether the resulting solution worked. A subagent transcript alone is insufficient.
* During long Tasks, record consequential findings in the main conversation as they arise to reduce loss if execution is interrupted.
* An optional journal under Git-ignored `.local/` can hold the Task identifier, problem, consultation, decision, and result during execution.
* Before finishing, transfer all significant journal findings into the main conversation. Local files, test reports, and separate subagent sessions might be unavailable later.
* Keep full test logs local. Preserve the meaning of evidence and its limits in the conversation without copying entire logs or analyses.

This process requires no separate consultation database, synchronization, new skill, or automatic GitHub publication.

### Final Task summary

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
| Planning feedback | What similar Tasks should include to prevent unnecessary analysis or consultation |

State explicitly when there were no significant gaps, consultations, independent decisions, or unresolved problems.
A completion claim, file list, or link to a local journal does not replace this summary.
The main conversation must support later JSON session review through the existing Context Skills without access to temporary files or subagent histories.

## Findings outside the current scope

* Record the observation, expected behavior, supporting evidence, and status without expanding the authorized implementation scope.
* Distinguish accepted requirements from implemented behavior. A documented target does not establish that the code already meets it.
* Keep unresolved proposals separate from agreed follow-up work.
* Record the finding in the main conversation. Preserve reusable knowledge in its documentation owner through the applicable Context Skill.
* Before removing a local record, preserve agreed outstanding work in an authorized destination that does not depend on that record.
* Use [finding promotion](../CONTRIBUTING.md#promote-agreed-findings) for self-contained follow-up issues or further planning of an open Work Item. Publication still requires authorization.
