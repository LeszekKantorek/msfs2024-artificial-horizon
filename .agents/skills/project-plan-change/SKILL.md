---
name: project-plan-change
description: Plan a coordinated project change locally with decisions, context, dependencies, and acceptance evidence. Do not use for a small direct edit, executing accepted work, or recording an architecture decision alone.
---

# Plan a Project Change

## Goal

Prepare an executable local plan for the requested outcome, with decisions and evidence appropriate to its scope.

## Steps

### 1. Establish the outcome and evidence

* Read the supplied request, relevant issue when present, accepted plans and amendments, and sources selected through [task guidance](../../../.context/task-guidance.md#practical-selection). Use [issue retrieval](../../../.context/repository-instructions.md#read-github-issue-context) for issue-based work.
* Establish the base revision, local changes, current implementation, source authority, scope, constraints, dependencies, and user acceptance criteria. Distinguish facts, hypotheses, proposals, and accepted decisions.
* Resolve routine technical choices within accepted scope. Obtain a human decision for changes to requirements or direction.

### 2. Resolve decisions and order work

* Compare viable options and their trade-offs for consequential boundaries, including ownership, interfaces, lifecycle, failure, and compatibility where affected. Select the simplest supported solution within scope and record its rationale.
* When evidence is missing, define a bounded investigation with discriminating observations and the decisions it blocks. Do not present an unconfirmed cause as established.
* Order coherent outcomes by dependency. Keep validation and applicable review with the work they verify. Defer dependent actions until their prerequisites are established.
* Select checks by failure risk, including expected results, preserved behavior, applicable manual acceptance, and evidence limits. Follow the [validation matrix](../../../docs/testing/index.md#validation-by-change-scope).
* Plan the complete applicable suite after integration and independent review for code changes under [review guidance](../../../docs/testing/evidence.md#independent-subagent-review). Repeat affected checks after corrections; broaden them only when a new risk or requirement justifies it.
* For visual changes, plan a baseline and matched inputs before implementation. Distinguish painted appearance and actual phone acceptance from automated tests. Apply the [manual acceptance rules](../../../docs/testing/evidence.md#sources-and-limits).

### 3. Prepare and check the local plan

Store plans and analyses under `.plans/<number>-<description>/` in the task worktree, using a descriptive name for work without an issue. Follow the [contribution workflow](../../../CONTRIBUTING.md#additional-working-trees). Do not create GitHub subtasks or put agent context into stories or bugs.

Include the outcome, scope, selected sources and relevant code, decisions and rationale, constraints, ordered actions, acceptance checks, dependencies, and unresolved questions at the detail needed for execution. Preserve supplied accepted plans and amendments under [execution rules](../../../.context/iterative-development.md#accepted-plans-and-handoffs).

Compare the plan with the request and current sources. Verify that each consequential decision is supported, each outcome has an appropriate check, and deferred work and evidence limits are explicit. A planning request does not authorize GitHub publication or implementation. Deliver the plan and unresolved decisions without claiming checks were performed.
