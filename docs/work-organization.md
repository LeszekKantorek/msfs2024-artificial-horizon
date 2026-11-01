# Work organization

This document defines the requirements for organizing project goals, planning, and execution in GitHub Issues.
[CONTRIBUTING](../CONTRIBUTING.md#workflow) defines the contribution workflow that applies these requirements.

## Issue hierarchy

Issues in this workflow must use the following levels, content, labels, and parent relationships.

| Level | Required content | Label | Parent |
| --- | --- | --- | --- |
| Initiative | A project goal, intended result, overall scope, and delivery dependencies | `initiative` | None within this hierarchy |
| Work Item | A concrete problem, feature, or result with requirements, constraints, and acceptance criteria | `work-item` | Initiative |
| Task | An execution outcome, scope, context, implementation guidance, acceptance criteria, validation, and prerequisites | `task` | Work Item |

A published Work Item or Task must show its intended parent in GitHub's parent/sub-issue relationship.
The publisher must verify that relationship after saving the issue, rather than treating a parent reference in its body as confirmation.
The supported operations are described in [GitHub sub-issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues).

Sol must divide a Work Item into Tasks with outcomes that Terra can execute and verify after their prerequisites are satisfied.
Changes across several files, their tests, and applicable review belong to one Task when they establish the same outcome.

Functional, implementation, diagnostic, research, documentation, and infrastructure work use the same hierarchy.
The kind of work does not change the required parent or introduce another level.

## Responsibilities

The roles must provide the following decisions and results.

| Role | Required decisions and results |
| --- | --- |
| Human | Define goals and expected results, oversee Initiatives and selected Work Items, approve material requirement or direction changes, and confirm user acceptance |
| Sol Medium | Prepare Task instructions from Work Item requirements and available evidence, resolve technical choices within accepted scope, and define acceptance and validation |
| Terra Medium | Verify Task context, perform the assigned work and applicable checks, consult Sol on unresolved consequential choices, and report results and planning gaps |

* Sol must record the selected technical direction and its rationale in the Task instructions or consultation result.
  Technical choices within approved scope do not require separate human confirmation.
* A change to approved requirements or project direction must receive human approval before dependent work proceeds.
  Terra must retain unconfirmed criteria and continue work that remains possible within approved scope.
* Terra must check the supplied context against current sources and implementation before relying on it.
  Execution results, significant gaps, consultations, and decision outcomes must remain in the main conversation for later session analysis.

## Iterative planning

Each planning iteration must use the Work Item's requirements, current implementation, and available results from preceding Tasks.
Sol must select the next executable outcomes according to what those inputs establish.

| Available basis | Required planning result |
| --- | --- |
| A reported problem has no established cause | A diagnostic Task that states the question, observations needed to distinguish causes, and evidence required in its result |
| Evidence establishes a defect and supports a correction | A correction Task that identifies the finding, selected solution, preserved behavior, and checks that confirm the correction |
| Accepted requirements specify new behavior | A delivery Task that identifies the governing requirements, selected implementation direction, and checks of the expected behavior |
| Completed Tasks leave Work Item criteria unconfirmed | The missing evidence or decisions for those criteria and the next executable Tasks, with the Work Item remaining open |

A Task is ready for execution only when its instructions resolve the decisions needed to perform its assigned outcome.
An investigation meets this requirement by defining the question and the evidence to obtain, without requiring its answer in advance.
Sol must defer dependent implementation Tasks until the missing findings or decisions are available.

For example, a Task investigating a missing airspeed band can establish where its expected value is lost.
A confirmed mapping defect then supplies the basis for a correction Task and its regression check.
If the investigation leaves source availability unresolved, the next Task must identify the observation needed to resolve it.

## Completion and evidence

Issue closure must be supported by the result and evidence required at that level.

| Level | Evidence required before closure |
| --- | --- |
| Task | Its assigned result, required verification, observed outcomes, and explicit limits of the evidence |
| Work Item | Confirmation of its expected behavior or result against its own acceptance criteria, using relevant Task results |
| Initiative | Confirmation of its project goal and intended result, including its own acceptance criteria and applicable user acceptance |

* Each Task must include the tests and review applicable to its outcome. Their results form part of its completion evidence.
* A diagnostic or research Task must report what its evidence establishes and what remains unresolved.
  A confirmed diagnosis, experiment result, or technical limitation can satisfy its outcome without code changes or a PR.
* When required evidence is missing, the completion record must identify the unconfirmed criterion and remaining verification.
  The corresponding issue must remain open under [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).

## Issues and documentation

The following records must retain the information needed for their purpose.

| Record | Required information |
| --- | --- |
| Project documentation | Durable requirements, contracts, constraints, architecture decisions and rationale, expected behavior, and verification scenarios |
| GitHub Issues | Goals, work decomposition, execution instructions, dependencies, completion criteria, delivery evidence, and related PRs |
| Main agent conversation | Execution findings, planning gaps, consultations, decisions, rationale, results, and unresolved problems for later session analysis |

* Each Task must contain the decisions and constraints needed to execute its scope in another session.
  References must identify the relevant document sections, code, or earlier results instead of requiring reconstruction of the planning conversation.
* Accepted requirements and contracts must remain in their subject's documentation owner, including when an issue contains execution-specific details.
  Issue references may supply supporting evidence, but the documented requirement must be understandable without opening the issue.
* Terra's final report must retain the significant execution findings in the main conversation, including successfully resolved gaps and consultation results.
  Local journals and separate subagent histories cannot be its only record because they might be unavailable during later session analysis.

GitHub Issues retain delivery results and acceptance evidence. The main conversation retains the detailed course of execution and consultations.
Later knowledge review selects reusable findings for documentation. Individual consultations do not require automatic documentation or issue updates.

## Adoption boundaries

[MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) is designated as the MVP Initiative in the target hierarchy.
It currently describes itself as the roadmap. Migration of existing issues, labels, and relationships requires separate authorization.

* Delivery order and dependencies must be maintained in the relevant GitHub hierarchy. Project documentation describes the work model without duplicating that schedule.
* Before publishing with the issue forms, the publisher must verify that the `initiative`, `work-item`, and `task` labels exist.
  Creation of missing labels must fall within the authorized GitHub scope.
* Migration must preserve existing relationships, completed records, and delivery evidence except where a specific change is explicitly authorized.
  Migration status must distinguish verified changes from operations not performed or not confirmed.
* Issue publication and reorganization must be included in the requested scope before either operation is performed.
  Without that authorization, planning must deliver complete drafts with their intended parents and dependencies.
