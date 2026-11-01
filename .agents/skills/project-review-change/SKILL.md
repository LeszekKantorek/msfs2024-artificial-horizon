---
name: project-review-change
description: Arrange an independent review of a specific project diff, assess the returned findings, and verify authorized corrections. Use after implementation and applicable checks, not for test execution alone, self-review presented as independent review, or diagnosing agent behavior. Delegation requires authorization from the active instructions.
---

# Review a Project Change

## Goal

Obtain and assess an independent review report for an exact diff, with corrections, reviewed revisions, and unreviewed scope recorded separately from test evidence.

## Steps

Review the agreed diff and assess the returned findings in this order:

1. Establish the requested review outcome, exact base and head revisions, checkout, and diff scope.
   If uncommitted changes are included, preserve a patch or equivalent snapshot that identifies the reviewed state.
2. Check delegation under these authorization and availability requirements:
   * Confirm that active instructions authorize subagent delegation and that the environment supports it. Selecting this skill does not grant authorization.
   * If delegation is unauthorized or unavailable, report independent review as unperformed and identify the missing prerequisite.
   * Report self-review separately from independent review.
3. After implementation and applicable validation, prepare the [Task handoff](../../../.context/iterative-development.md#accepted-plans-and-handoffs) with these inputs:
   * Supply the Task, accepted decisions and amendments, applicable contracts, preserved invariants, validation evidence, and known limitations.
   * Establish issue-based requirements through [issue and artifact retrieval](../../../.context/repository-instructions.md#read-github-issue-context). Include the relevant artifact references and unresolved reads in the review handoff.
   * Include a governing supplied plan when one exists. A local handoff index is optional.
   * For work without a Task or accepted plan, state that explicitly and use the agreed scope.
4. Request a separate read-only review of that state under these scope requirements:
   * For implementation review, inspect correctness, regressions, scope changes, and test or acceptance gaps.
   * Keep product redesign outside a documentation-format review.
   * Permit relevant source inspection and checks without concurrent edits to the reviewed files.
5. Obtain the review report under these evidence requirements:
   * Require actionable findings with severity, file and line, failure scenario, and supporting reasoning.
   * Wait for the returned report. If review is interrupted, fails, or returns no report, record it as incomplete.
   * Treat the report, rather than a review request or passing tests, as evidence that review completed.
6. Assess each finding against requirements and current code under these rules:
   * Distinguish confirmed defects, unsupported findings, and unresolved questions, with reasons for each disposition.
   * For review-only work, report proposed corrections without editing the implementation.
7. When corrections are authorized, resolve established defects and rerun affected checks under these review requirements:
   * Request review of material fixes under the same delegation conditions.
   * Supply the original scope, all findings, their dispositions, and subsequent accepted decisions.
   * Disclose a reviewer change. The follow-up review need not use the same reviewer.
   * Obtain the report and record the resulting reviewed revision. Mark later unreviewed changes explicitly.
8. Deliver the review result under these reporting requirements:
   * Report findings, their disposition, the reviewed state, and remaining unreviewed scope separately from test results.
   * Publish the report to an issue or PR only when authorized.

## Completion and evidence limits

Apply these completion requirements and evidence limits:

* Deliver the received report and assessed findings, or identify the blocking condition that left review incomplete.
* Claim completed independent review only after receiving its report. Self-review, passing tests, or a review request alone are insufficient.
* Report independent review separately from tests and actual device acceptance. Review does not establish the absence of defects.
* Attribute results supplied in a handoff to their original evidence. Do not claim that the receiving reviewer independently repeated them.
* Keep unavailable device checks separate under [acceptance tracking](../../../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).
* Use [evidence selection](../../../docs/testing/evidence.md#select-evidence-by-failure-risk) when findings require additional checks.
* Apply the review and device evidence limits in the [evidence guidance](../../../docs/testing/evidence.md#sources-and-limits).
