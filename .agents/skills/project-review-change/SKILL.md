---
name: project-review-change
description: Arrange an independent review of a specific project diff, assess the returned findings, and verify authorized corrections. Use after implementation and applicable checks, not for test execution alone, self-review presented as independent review, or diagnosing agent behavior. Delegation requires authorization from the active instructions.
---

# Review a Project Change

## Goal

Obtain and assess an independent review report for an exact diff, with corrections, reviewed revisions, and unreviewed scope recorded separately from test evidence.

## Steps

1. Establish the requested review outcome, exact base and head revisions, checkout, and diff scope. If uncommitted changes are included, preserve a patch or equivalent snapshot in the local handoff so the reviewer can identify the reviewed state.
2. Check whether active instructions authorize subagent delegation and whether it is available. Selecting this skill does not itself grant that authorization. If delegation is unauthorized or unavailable, report independent review as unperformed and identify the missing prerequisite; do not replace it with a claim of self-review independence.
3. After implementation and applicable validation, prepare the handoff. Supply the complete accepted plan and amendments through the [local handoff index](../../../.context/iterative-development.md#accepted-plans-and-handoffs), applicable contracts, preserved invariants, validation evidence, and known limitations. When there is no accepted plan, state that explicitly and use the agreed task scope.
4. Request a separate read-only review of that state within the requested review scope. For an implementation review, inspect correctness, regressions, scope changes, and test or acceptance gaps. A documentation-format review does not authorize redesigning the product. Permit relevant source inspection and checks without concurrent edits to the reviewed files.
5. Require actionable findings with severity, file and line, failure scenario, and supporting reasoning. Wait for the returned report. If review is interrupted, fails, or returns no report, record it as incomplete; a request or passing tests do not establish completed independent review.
6. Assess each finding against requirements and current code before making changes. Distinguish confirmed defects, unsupported findings, and unresolved questions with reasons. For review-only work, report proposed corrections without editing the implementation.
7. When corrections are authorized, resolve established defects and rerun affected checks. Request review of material fixes under the same delegation conditions. Supply the original scope, all findings, their dispositions, and subsequent accepted decisions. Disclose a reviewer change without requiring the same reviewer. Obtain the report and record the resulting reviewed revision. Mark later unreviewed changes explicitly.
8. Report findings, their disposition, the reviewed state, and remaining unreviewed scope separately from test results. Publish the report to an issue or PR only when authorized.

## Completion and evidence limits

Finish with the received report and assessed findings, or an explicit incomplete-review result with its blocking condition.
Do not describe self-review, passing tests, or a review request without a returned report as completed independent review.
An independent review does not establish absence of defects or replace tests and actual device acceptance.
Earlier results supplied in a handoff remain attributed evidence, not checks independently performed by the receiving reviewer.
Keep unavailable device checks separate under [acceptance tracking](../../../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).
Use [evidence selection](../../../docs/testing/evidence.md#select-evidence-by-failure-risk) when findings require additional checks.
The [evidence guidance](../../../docs/testing/evidence.md#sources-and-limits) explains review and device evidence limits.
