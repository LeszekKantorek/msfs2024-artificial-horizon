---
name: project-plan-change
description: Prepare a detailed implementation plan for an issue or coordinated project change, with scope, dependencies, invariants, and acceptance evidence. Do not use for a small direct edit, executing an already accepted plan, or recording an architecture decision alone.
---

# Plan a Project Change

## Goal

Produce a decision-complete plan for Terra medium implementation, with small steps, bounded context, and observable completion conditions.

## Model and execution defaults

* Plan with GPT-6.1 Sol at `medium`; assign implementation to GPT-6.1 Terra at `medium` unless explicitly overridden.
* State these assignments in the plan. Skill instructions do not change the active model or its settings.
* If the requested model is unavailable, report the limitation without silently substituting another model.
* Use one developer by default. Add parallel developers only for independent scopes with separate file ownership and useful integration boundaries.
* Keep independent review separate from authorship. Do not add agents merely to reread the same sources or run commands.

## Scope and authorization

Preserve accepted decisions while resolving the current task.

* Read the complete accepted plan and its amendments before choosing a planning path.
* An instruction to implement an accepted plan authorizes its existing scope without another request for the same approval.
* Ask about unresolved requirements or material changes outside that scope before implementing dependent work.
* Use a short in-chat design for bounded changes and a detailed plan when requested or needed for coordination.
* Prefer conversational planning in chat over a dedicated planning mode or multiple-choice forms, unless explicitly requested otherwise.
* Use a Socratic approach: question assumptions, identify contradictions, and offer alternative interpretations. Do not automatically agree with proposed ideas.
* Use open questions for material uncertainties and discuss trade-offs. Do not turn routine implementation details into an interview.
* Consolidate agreed decisions into the plan after discussion. Do not treat exploration as approval.
* Prepare plans directly from project sources and the [handoff rules](../../../.context/iterative-development.md#accepted-plans-and-handoffs).
* Plan only the requested outcome and its necessary prerequisites. Do not append unsolicited roadmaps, features, refactoring, or follow-up stages.

## Steps

Use this procedure for a detailed plan, with depth proportional to the change.

1. Read the issue, supplied documents, applicable reference images, and contracts selected through project maps.
2. Check the current implementation and identify the base revision.
3. Resolve replaced requirements from their authority and record which earlier requirements still apply.
4. State the intended result, accepted decisions, scope boundaries, and behavior that must remain unchanged.
5. Resolve implementation decisions using the decision contract below. Compare alternatives only where they affect correctness, complexity, or compatibility.
6. Ask about material choices outside accepted scope. For missing technical evidence, plan a bounded investigation before dependent implementation.
7. Trace affected module boundaries, data lifecycle, integration, and presentation. Do not inspect unrelated subsystems.
8. Define acceptance evidence for each requirement before choosing implementation steps.
9. Order small, complete steps by dependency. Establish contracts first, isolate risky mechanisms, then integrate observable vertical slices.
10. Describe each step using the step fields below. Include baseline capture before visual changes.
11. Schedule focused checks during implementation and the complete applicable suite after integration. Follow the validation rules below.
12. Include independent review, necessary documentation, context updates, and authorized PR preparation. Identify unavailable checks without inventing results.
13. Preserve the full accepted plan and a short execution index under the [handoff rules](../../../.context/iterative-development.md#accepted-plans-and-handoffs).

### Decision contract

Resolve these boundaries before assigning dependent implementation:

| Boundary | Record |
| --- | --- |
| Interfaces | Names, signatures, data shapes, units, and compatibility where affected |
| Ownership | Component responsibilities, mutation rights, task and resource lifetimes |
| Failure | Invalid inputs, unavailable data, retries, propagation, and cleanup where applicable |
| Completion | Observable behavior, preserved invariants, and expected check results |
| Remaining uncertainty | A bounded investigation, its output, and the dependent steps it blocks |

* Present options and trade-offs for material design choices; obtain approval before marking dependent steps ready. Resolve routine implementation details autonomously.
* Prefer the simplest solution meeting current requirements. Justify added complexity or abstractions with a concrete need, not hypothetical growth.
* Leave routine implementation details to the developer. Do not specify pseudocode for every function.
* Do not present unresolved alternatives as an implementation-ready step.

### Implementation step fields

Use these fields for each numbered step:

| Field | Required content |
| --- | --- |
| Goal | One coherent behavior or prerequisite delivered by the step |
| Scope | Files or modules to change, interfaces to preserve, and developer assignment |
| Dependencies | Required earlier outputs and the relevant source sections |
| Actions | Concrete substeps, without unrelated refactoring |
| Completion | Observable result and the focused check that establishes it |

* Split a step when it combines independent outcomes or leaves a risky boundary implicit.
* Avoid file-by-file steps that postpone all integration until the end.
* Link shared contracts once and reference them by name in dependent steps.

### Escalation and handoff

Include these execution rules in the plan:

* After two attempts on the same problem yield no new evidence, stop repeating them and escalate that problem to Sol medium.
* Provide the symptom, relevant code, attempted checks, results, and one specific question.
* Sol resolves the bounded problem; Terra resumes implementation with that decision recorded in the handoff.
* Escalation does not require replanning the whole task. Independent work can continue.
* Keep a short index in `.local/`: current step, decisions, completed checks, and open problems, with links to detailed evidence.
* Preserve the full accepted plan verbatim. Record changes as amendments, never as silent rewrites.
* Use the index to locate the plan and applicable amendments, not to replace their required reading.
* Reuse unchanged sources and evidence. Inspect relevant sections and log excerpts instead of repeatedly dumping whole documents or diffs.

### Validation and acceptance

* Select automated checks by changed behavior and failure risk. Run focused checks during implementation.
* Run the complete applicable suite after integration, then obtain independent review.
* After corrections, repeat affected checks and review. Broaden them only when a new risk or explicit requirement justifies it.
* Schedule one combined manual acceptance at the end. Do not require operator acceptance after each feature.
* Apply the [manual acceptance rules](../../../docs/testing/evidence.md#sources-and-limits) to confirmations, missing artifacts, and requests for another test.
* Keep full logs local and report concise results. Do not treat longer plans, more agents, or repeated checks as evidence of quality.

## Completion checks

Check the plan before presenting it.

* Each step must explain what changes and how its completion can be observed.
* Each acceptance criterion must map to a check, an expected result, and evidence to record.
* Add substeps where dependency, risk, or distinct verification requires them.
* Avoid adding steps merely to increase plan length.
* Link canonical procedures instead of copying unrelated checklists.
* Apply [evidence selection](../../../docs/testing/evidence.md#select-evidence-by-failure-risk) and [independent review guidance](../../../docs/testing/evidence.md#independent-subagent-review) when selecting evidence and preparing the review handoff.
* Link relevant project sources under [task guidance selection](../../../.context/task-guidance.md) for each implementation step.
* Confirm that Terra can execute each ready step without choosing an unresolved interface, ownership rule, or failure policy.
* Confirm that parallel work has a concrete benefit and that escalation, handoff, and final acceptance are specified.

## Supporting knowledge

Read the [planning rationale](../../../.context/implementation-planning.md#why-these-details-matter) when choosing planning detail or explaining a trade-off.
The [planning guidance](../../../.context/implementation-planning.md#sources-and-limits) describes evidence and review limits.
Planning does not authorize implementation or publication of a local plan. Finish with the plan and any material unresolved decisions.
