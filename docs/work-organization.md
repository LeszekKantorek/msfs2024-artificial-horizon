# Work organization

This document defines the requirements for organizing project goals, planning, and execution in GitHub Issues.
[CONTRIBUTING](../CONTRIBUTING.md#workflow) defines the contribution workflow that applies these requirements.

## Issue hierarchy

Issues in this workflow must use the following levels, content, labels, and parent relationships:

| Level | Required content | Label | Parent |
| --- | --- | --- | --- |
| Epic | A project goal, intended result, overall scope, and delivery dependencies | `epic` | None within this hierarchy |
| Task | A concrete problem, feature, or result with requirements, constraints, and acceptance criteria | `task` | Epic |
| Subtask | An execution outcome, scope, context, implementation guidance, acceptance criteria, validation, and prerequisites | `subtask` | Task |

Issue publication and decomposition must meet these requirements:

* Each published Task or Subtask must have the actual parent relationship specified in the table.
  The publisher must verify the saved relationship using the supported [GitHub sub-issue operations](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues).
  A reference in the issue body does not confirm that relationship.
* Sol must define each Subtask so Terra can execute and verify its outcome after the stated prerequisites are satisfied.
  Changes across several files, their tests, and applicable review must remain together when they establish the same outcome.
* Functional, implementation, diagnostic, research, documentation, and infrastructure work must use the same three levels.
  The kind of work must not change the required parent or introduce another level.

## Responsibilities

The roles must provide the following decisions and results:

| Role | Required decisions and results |
| --- | --- |
| Human | Define goals and expected results, oversee Epics and selected Tasks, approve material requirement or direction changes, and confirm user acceptance |
| Sol Medium | Prepare Subtask instructions from Task requirements and available evidence, resolve technical choices within accepted scope, and define acceptance and validation |
| Terra Medium | Verify Subtask context, perform the assigned work and applicable checks, consult Sol on unresolved consequential choices, and report results and planning gaps |

Technical decisions and execution must meet these requirements:

* Sol must record the selected technical direction and its rationale in the Subtask instructions or consultation result.
  Technical choices within approved scope do not require separate human confirmation.
* A change to approved requirements or project direction must receive human approval before dependent work proceeds.
  Terra must retain unconfirmed criteria and continue work that remains possible within approved scope.
* Terra must check the supplied context against current sources and implementation before relying on it.

## Iterative planning

Sol must select each iteration's Subtasks according to the following decision rules:

| Available basis | Required planning result |
| --- | --- |
| A reported problem has no established cause | A diagnostic Subtask that states the question, observations needed to distinguish causes, and evidence required in its result |
| Evidence establishes a defect and supports a correction | A correction Subtask that identifies the finding, selected solution, preserved behavior, and checks that confirm the correction |
| Accepted requirements specify new behavior | A delivery Subtask that identifies the governing requirements, selected implementation direction, and checks of the expected behavior |
| Completed Subtasks leave Task criteria unconfirmed | The missing evidence or decisions for those criteria and the next executable Subtasks, with the Task remaining open |

Subtask preparation must meet these requirements before execution:

* Sol must base Subtask instructions on the Task's requirements, current implementation, and available results from preceding Subtasks.
* Sol must resolve the technical decisions needed to execute a Subtask before handing it to Terra.
  Dependent implementation Subtasks must remain deferred while required findings or decisions are unavailable.
* An investigation Subtask must define its question and the evidence to obtain.
  Its answer is an execution result, not a prerequisite for assigning the investigation.
* A correction Subtask based on an investigation must reference the confirmed finding and define the regression check for that defect.
* If an investigation leaves source availability unresolved, the next investigation Subtask must identify the observation needed to establish it.

## Completion and evidence

Issue closure must be supported by the result and evidence required at that level:

| Level | Evidence required before closure |
| --- | --- |
| Subtask | Its assigned result, required verification, observed outcomes, and explicit limits of the evidence |
| Task | Confirmation of its expected behavior or result against its own acceptance criteria, using relevant Subtask results |
| Epic | Confirmation of its project goal and intended result, including its own acceptance criteria and applicable user acceptance |

Completion records must satisfy these requirements:

* Each Subtask must include the tests and review applicable to its outcome. Their results form part of its completion evidence.
* A diagnostic or research Subtask must report what its evidence establishes and what remains unresolved.
  A confirmed diagnosis, experiment result, or technical limitation can satisfy its outcome without code changes or a PR.
* When required evidence is missing, the completion record must identify the unconfirmed criterion and remaining verification.
  The corresponding issue must remain open under [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).

## Issues and documentation

The following records must retain the information needed for their purpose:

| Record | Required information |
| --- | --- |
| Project documentation | Durable requirements, contracts, constraints, architecture decisions and rationale, expected behavior, and verification scenarios |
| GitHub Issues | Goals, work decomposition, execution instructions, dependencies, completion criteria, delivery evidence, and related PRs |
| Main agent conversation | Execution findings, planning gaps, consultations, decisions, rationale, results, and unresolved problems for later session analysis |

Handoffs and reporting must satisfy these requirements:

* Each Subtask must contain the decisions and constraints needed to execute its scope in another session.
  References must identify the relevant document sections, code, or earlier results instead of requiring reconstruction of the planning conversation.
* Accepted requirements and contracts must remain in their subject's documentation owner, including when an issue contains execution-specific details.
  Issue references may supply supporting evidence, but the documented requirement must be understandable without opening the issue.
* Terra's final report must retain the significant execution findings in the main conversation, including successfully resolved gaps and consultation results.
  Local journals and separate subagent histories cannot be its only record because they might be unavailable during later session analysis.
* Consultation reporting must preserve findings in the main conversation without requiring automatic updates to Issues or documentation.
  Later knowledge review selects reusable findings for their documentation owners.

## Adoption boundaries

[MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) is designated as the MVP Epic in the target hierarchy.

Adoption and migration must meet these requirements:

* Migration of existing issues, labels, and relationships must be authorized separately from adoption of these repository instructions.
* Delivery order and dependencies must be maintained in the relevant GitHub hierarchy. Project documentation describes the work model without duplicating that schedule.
* Before publishing with the issue forms, the publisher must verify that the `epic`, `task`, and `subtask` labels exist.
  Creation of missing labels must fall within the authorized GitHub scope.
* Migration must preserve existing relationships, completed records, and delivery evidence except where a specific change is explicitly authorized.
  Migration status must distinguish verified changes from operations not performed or not confirmed.
* Issue publication and reorganization must be included in the requested scope before either operation is performed.
  Without that authorization, planning must deliver complete drafts with their intended parents and dependencies.
