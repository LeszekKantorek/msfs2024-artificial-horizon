---
description: Prepare execution context, preserve accepted decisions, consult on consequential gaps, and report results.
status: active
---

The [work organization model](../docs/work-organization.md) defines issue types. The [contribution workflow](../CONTRIBUTING.md#workflow) defines worktrees and local plan locations.

## Scope and component boundaries

* Keep speculative frameworks and later work outside the current scope. Do not postpone useful structural changes solely because the current skeleton works.
* Keep the [separate altitude tape](../docs/efi/altitude.md#rationale-and-regression-lessons) unless shared requirements justify reconsidering its scale, sign, digit, selection, and alert boundaries.
* Use the [internal seams](../docs/efi/composition.md#visual-object-interfaces-and-ownership) first instead of inferring shared rules from a planned instrument. Apply the [PFD composition rules](../docs/efi/composition.md#rationale-and-regression-lessons) for calculation ownership and extraction to actual shared consumers.

## Accepted plans and handoffs

* Preserve supplied accepted plans verbatim and locally, including their original language and amendments. Record later corrections with their date and the requirements they supersede.
* Keep plans, analyses, and agent handoffs under `.plans/<number>-<description>/` in the working tree; use a descriptive name for work without an issue. Remove all tracked `.plans/` entries from the final PR tree targeting `main`.
* Read governing plans, decisions, invariants, and amendments before dependent work. Preserve durable requirements in their documentation owners.
* Do not create GitHub subtasks for execution or place AI agent context in stories or bugs. Publish issue changes and evidence only within authorized GitHub work.

### Prepare execution context

Select sources under [context selection and retrieval delegation](context-retrival-delegation.md). For issue-based work, read the issue through [issue retrieval](repository-instructions.md#read-github-issue-context). For work without an issue, derive context from the requested outcome.

Check current code, contracts, relevant decisions, prerequisites, and validation against the plan. Resolve routine details within scope. Record consequential mismatches and use consultation when broader analysis is needed; continue independent authorized work.

### Consult Sol

Consult when a consequential decision or requirement is missing, sources conflict, or an unexpected dependency needs broader analysis:

1. State the problem, affected outcome, relevant sources, evidence, attempted checks, and a concrete question.
2. If active instructions authorize delegation and the host supports it, start a Sol Medium subagent. Report an unavailable model or delegation capability without silently substituting another.
3. Compare viable solutions and select a supported technical solution within accepted scope. Obtain a human decision before work that changes requirements or direction; continue independent work.
4. Record the question, options, decision, rationale, and result in the main conversation. Retain unresolved dependent work explicitly.

### Record execution findings

Record consequential context gaps, contradictions, decisions, and consultation results in the main conversation, including those resolved successfully. Local notes, test logs, and subagent transcripts may not be available during later session analysis. Distinguish observations, hypotheses, and evidence limits. Do not publish findings to GitHub without authorization.

### Final execution summary

Finish in the main conversation with the work performed, checks and evidence limits, significant gaps, consultations and decisions with their results, and unresolved problems. State when no significant gaps or consultations arose. Include planning feedback when it can prevent a recurring problem.

## Findings outside the current scope

Record the observation, expected behavior, evidence, and status without expanding authorized scope. Distinguish accepted requirements from implementation and proposals. Preserve reusable knowledge in its documentation owner through the applicable Context Skill. Before removing a local record, preserve agreed outstanding work in an authorized destination. Use [finding promotion](../CONTRIBUTING.md#promote-agreed-findings) for an agreed follow-up; publication requires authorization.
