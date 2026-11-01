---
name: project-plan-change
description: Plan a Work Item or coordinated project change as self-contained executable Tasks with decisions, context, dependencies, and acceptance evidence. Do not use for a small direct edit, executing accepted work, or recording an architecture decision alone.
---

# Plan a Project Change

## Goal

Prepare the currently justified, executable Tasks for Terra Medium, with complete drafts when GitHub publication is not authorized.

## Model and execution defaults

* Plan and consult with GPT-6.1 Sol at `medium`. Assign execution to GPT-6.1 Terra at `medium` unless explicitly overridden.
* State the assignments in the handoff. Skill instructions do not change the active model or its settings.
* Check host capabilities before selecting a subagent model. If the requested model is unavailable, report the limitation without silently substituting another model.
* Use one developer by default. Propose parallel developers only for independent scopes with separate ownership and useful integration boundaries.
* Keep independent review separate from authorship. Do not add agents merely to reread sources or run commands.

## Scope and authorization

The [work organization model](../../../docs/work-organization.md) defines Initiative, Work Item, and Task responsibilities.
Preserve accepted decisions while planning the requested outcome and its necessary prerequisites.

* Distinguish planning, publishing Tasks, and reorganizing existing Issues. A planning request alone does not authorize GitHub changes.
* When publication is not authorized, return complete Task drafts with intended parents and dependencies. Do not create Issues or labels.
* Read a supplied accepted plan and its amendments completely. Implementation of that plan is already authorized when explicitly requested.
* Resolve technical choices within accepted scope. Ask about changes to requirements or project direction before preparing dependent execution.
* Use a short in-chat design for bounded work. Do not require an Issue for every small direct change.
* Prefer conversational planning unless another interaction mode is requested. Question assumptions and discuss material trade-offs without turning routine details into an interview.
* Keep hypotheses, proposals, accepted decisions, and confirmed evidence distinct. Do not append unrelated refactoring, roadmaps, or follow-up stages.

## Steps

### 1. Establish the Work Item and current evidence

Read the Work Item, its Initiative, supplied material, prior Task results, and applicable sources selected through [task guidance](../../../.context/task-guidance.md#practical-selection).
For a change without an Issue, use the supplied outcome. Identify the intended parent only if publication is requested.

* Identify the base revision, relevant local changes, current implementation, source authority, and replaced requirements.
* Establish the result, scope, constraints, invariants, existing dependencies, and acceptance criteria that remain unconfirmed.
* Read only relevant module boundaries, lifecycle, contracts, and presentation sources. Reuse unchanged context and evidence.

Continue when the requested result and current evidence are clear, with missing facts identified.

### 2. Resolve decisions or bound the investigation

Apply the [decision contract](#decision-contract) before assigning dependent execution.
Compare viable technical options where correctness, complexity, ownership, or compatibility changes.

| Evidence or scope condition | Planning action |
| --- | --- |
| Evidence supports a technical choice within accepted requirements | Select the best justified solution and record its rationale and constraints |
| A missing fact prevents a justified correction | Prepare a bounded diagnostic or research Task with a question, discriminating observations, and expected evidence |
| A choice changes approved requirements or project direction | Obtain the required human decision before preparing dependent execution |
| Earlier Task results expose more work | Use the results to prepare the next justified Tasks and retain unconfirmed parent criteria |

Do not turn an unconfirmed cause into detailed correction instructions.
Continue when each proposed Task has either resolved execution decisions or an explicit investigation outcome.

### 3. Decompose the currently ready work

Order small, complete outcomes by dependency. Establish contracts first, isolate risky mechanisms, and integrate observable results.
Prepare only Tasks supported by current knowledge. Future implementation can wait for investigation results.

* Use [Task content](#task-content) for each assignment. Keep technical choices that affect execution explicit without prescribing every routine function detail.
* Define acceptance evidence before implementation actions. Include scenario inputs, expected results, and preserved behavior.
* Keep testing and applicable review within the Task. Do not create separate Issues solely for routine validation or one per file or function.
* A diagnostic, research, documentation, or infrastructure Task can complete without application code or a PR.
* Use the [handoff rules](../../../.context/iterative-development.md#accepted-plans-and-handoffs) when a supplied plan governs execution.

Continue when Terra can execute each ready Task without reconstructing intent or choosing an unresolved consequential contract.

### 4. Check the handoff and validation

Check every draft against the Work Item, accepted decisions, and [completion checks](#completion-checks).
Apply [validation and acceptance](#validation-and-acceptance) to the changed behavior and its risks.
Use the [execution rules](#escalation-and-handoff) for preparation, consultation, and reporting rather than duplicating them in every Task.

Continue when each acceptance criterion has an expected result, a check, and evidence to record. Retain limitations explicitly.

### 5. Deliver drafts or verify authorized publication

| Authorization | Deliverable |
| --- | --- |
| Planning only | Complete Task drafts, intended parent, dependency order, accepted decisions, and remaining Work Item questions |
| Task publication included | Published Tasks with verified content, `task` labels, actual Work Item parents, and dependency references |
| Existing Issue reorganization requested | Only the explicitly authorized changes, preserving relationships, history, and delivery evidence |

1. Before publication, check current parent state, existing relationships, label availability, and supported GitHub operations.
2. After authorized publication, verify the saved relationship. An issue-body link does not establish a parent.
3. Report failed or unavailable operations without claiming that publication or migration completed.

Do not close a Work Item because all currently planned Tasks finished. Its own result and criteria govern completion.
Finish with the drafts or verified records, the next evidence needed for further planning, and unconfirmed parent criteria.

### Decision contract

Resolve these boundaries before assigning dependent implementation:

| Boundary | Record |
| --- | --- |
| Interfaces | Names, signatures, data shapes, units, and compatibility where affected |
| Ownership | Component responsibilities, mutation rights, task and resource lifetimes |
| Failure | Invalid inputs, unavailable data, retries, propagation, and cleanup where applicable |
| Completion | Observable behavior, preserved invariants, and expected check results |
| Remaining uncertainty | A bounded investigation, its output, and the dependent steps it blocks |

* Record technical choices and trade-offs within accepted scope. Human approval is required for changes to requirements or direction.
* Prefer the simplest solution meeting current requirements. Justify added complexity or abstractions with a concrete need, not hypothetical growth.
* Leave routine implementation details to the developer. Do not specify pseudocode for every function.
* Do not present unresolved alternatives as an implementation-ready step.

### Task content

Each Task identifies its Work Item parent and prerequisite Tasks, or states that there are no prerequisites.
Include all six fields, with detail proportional to complexity and risk:

| Field | Required content |
| --- | --- |
| Outcome | One concrete behavior, investigation result, or other deliverable |
| Scope | Affected modules or documents, exclusions, constraints, and preserved contracts or behavior |
| Context | Required document sections, useful checked line ranges, relevant code, earlier results, and source authority |
| Implementation guidance | Selected direction, decisions and assumptions, component responsibilities, dependencies, and actions needed for the outcome |
| Acceptance criteria | Observable completion conditions and expected results |
| Validation | Required checks and applicable review, evidence to record, and unavailable checks or equipment |

* Link canonical sources instead of copying entire documents. Include the decisions, requirements, and constraints needed to avoid reconstructing the plan's intent.
* Split assignments when outcomes are independent or a risky dependency needs a separate result.
* Integrate observable results during execution rather than postponing all integration until the end.

### Escalation and handoff

Use [Task context preparation](../../../.context/iterative-development.md#prepare-task-context), [Sol consultation](../../../.context/iterative-development.md#consult-sol), and [execution reporting](../../../.context/iterative-development.md#record-execution-findings).
These sections own the procedure, autonomy boundaries, model availability limits, and optional local journal.
The [final Task summary](../../../.context/iterative-development.md#final-task-summary) preserves gaps and decisions in the main conversation for later session review.

### Validation and acceptance

* Select focused checks by changed behavior and failure risk through the [validation matrix](../../../docs/testing/index.md#validation-by-change-scope).
* Run the complete applicable suite after integration. Include independent review for code changes under the [review guidance](../../../docs/testing/evidence.md#independent-subagent-review).
* After corrections, repeat affected checks and applicable review. Broaden checks only when a new risk or explicit requirement justifies it.
* Include necessary documentation, context updates, and PR preparation when authorized. Keep unavailable checks explicit.
* For visual changes, capture a baseline and matched inputs before implementation. Tests alone do not establish painted appearance or actual phone acceptance.
* Combine related manual acceptance at the end when the tested revision and affected behavior are identified. Do not require operator acceptance after every Task.
* Apply the [manual acceptance rules](../../../docs/testing/evidence.md#sources-and-limits) to confirmations, missing artifacts, and requests for another test.
* Keep full logs local and report meaningful results and evidence limits. Longer plans, more Tasks, agents, or repeated checks do not establish quality.

## Completion checks

| Check | Expected result |
| --- | --- |
| Work Item coverage | Ready Tasks serve its result; deferred work and unconfirmed criteria remain explicit |
| Executability | Each Task supplies the six fields, parent, prerequisites, decisions, and preserved invariants |
| Evidence | Each criterion maps to a check, expected result, and evidence record under [evidence selection](../../../docs/testing/evidence.md#select-evidence-by-failure-risk) |
| Uncertainty | Investigation precedes dependent corrections; hypotheses are not presented as confirmed causes |
| Context | Precise sources support execution without duplicated documents or dependence on temporary files |
| Proportionality | Tasks deliver coherent results; parallel work has a concrete benefit and validation belongs to the work |
| Authorization | Drafts, publication, and reorganization match the request; historical records remain protected |
| Reporting | Execution routes require the main-conversation summary, including successfully resolved gaps and consultation results |

Correct a failed check before marking dependent work ready. Retain unresolved matters with their needed decision or evidence.

## Supporting knowledge

Read the [planning rationale](../../../.context/implementation-planning.md#why-these-details-matter) when choosing planning detail or explaining a trade-off.
The [planning evidence limits](../../../.context/implementation-planning.md#sources-and-limits) distinguish validation, review, and acceptance.
Planning does not itself authorize implementation or publication.
