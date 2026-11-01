# Work organization

GitHub Issues organize project goals, planning, and execution through Initiative, Work Item, and Task levels.
This document defines the model. [CONTRIBUTING](../CONTRIBUTING.md#workflow) defines the contribution workflow.

## Issue hierarchy

| Level | Responsibility | Label | Parent |
| --- | --- | --- | --- |
| Initiative | A project goal, intended result, overall scope, and delivery dependencies | `initiative` | None within this hierarchy |
| Work Item | A concrete problem, feature, or result with requirements, constraints, and acceptance criteria | `work-item` | Initiative |
| Task | A self-contained execution assignment with context, decisions, acceptance criteria, and validation | `task` | Work Item |

Use actual GitHub parent/sub-issue relationships. A link or checklist in an issue body does not establish the relationship.
See [GitHub sub-issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues) for the supported hierarchy operations.

The level describes responsibility and detail. Functional, implementation, diagnostic, research, documentation, and infrastructure work can appear at either Work Item or Task level.

* Keep the three levels regardless of the kind of work.
* Group work by coherent results rather than creating one issue per file, function, or test.

## Responsibilities

| Role | Responsibility |
| --- | --- |
| Human | Define goals and expected results, oversee Initiatives and selected Work Items, approve material requirement or direction changes, and confirm user acceptance |
| Sol Medium | Plan Work Items, resolve technical decisions within accepted scope, prepare executable Tasks, define evidence, and consult during execution |
| Terra Medium | Execute Tasks, verify context, implement or investigate, perform applicable validation and review, consult Sol, and report results and planning gaps |

Planning resolves consequential decisions before dependent execution. Terra still reasons about implementation and checks assumptions against current evidence.
Technical autonomy does not permit silent changes to approved requirements or claims that unmet acceptance criteria passed.

## Iterative planning

Plan the Tasks justified by current knowledge. A Work Item does not need a complete set of Tasks in one planning session.
When the cause remains uncertain, prepare diagnostic or research Tasks before dependent correction Tasks.

For example, a missing-indication Work Item can first investigate acquisition and mapping.
The results then support planning a confirmed correction, further investigation, or an explicit source limitation.
Do not prepare detailed correction instructions from an unconfirmed hypothesis.

## Completion and evidence

* A Task delivers its stated result and the verification appropriate to that result. It can produce a diagnosis, experiment result, or confirmed limitation without code changes.
* Tests and review belong to the Task when applicable. Do not create separate issues solely for routine testing or review.
* Close a Work Item only when evidence confirms its own acceptance criteria. Completing all currently planned Tasks can instead reveal more work.
* Close an Initiative only when its intended result and acceptance criteria are confirmed. Child completion alone does not establish that result.
* Keep unavailable checks and unresolved limitations explicit under [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).

## Issues and documentation

| Record | Owns |
| --- | --- |
| Project documentation | Durable requirements, contracts, constraints, architecture decisions and rationale, expected behavior, and verification scenarios |
| GitHub Issues | Goals, work decomposition, execution instructions, dependencies, completion criteria, delivery evidence, and related PRs |
| Main agent conversation | Execution findings, planning gaps, consultations, decisions, rationale, results, and unresolved problems for later session analysis |

An issue includes the decisions and constraints needed to execute its work and links the relevant documentation sections.
It must not become the sole durable source of project requirements or copy entire documents where a precise reference suffices.

Conversation history retains consultation details. Issues confirm work and results without becoming a running consultation notebook.
Later knowledge review can preserve reusable findings in their documentation owners. It does not require automatic updates after every consultation.

## Adoption boundaries

[MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) is designated as the MVP Initiative in the target hierarchy.
It currently describes itself as the roadmap. Migration of existing issues, labels, and relationships is a separate authorized change.
These repository instructions and forms do not establish that migration has occurred.

* Keep delivery order and dependencies in the relevant GitHub hierarchy. Do not maintain a competing roadmap document.
* Before using the forms, verify that the `initiative`, `work-item`, and `task` labels exist. Create missing labels only within authorized GitHub work.
* Preserve existing relationships, completed records, and delivery evidence during an authorized migration. Do not mass-rewrite historical issues.
* A planning request authorizes issue publication or reorganization only when its stated scope includes those operations. Otherwise, return complete drafts.
* After authorized publication, verify saved labels and actual parent relationships. A form's parent field is only an instruction to establish that relationship.
