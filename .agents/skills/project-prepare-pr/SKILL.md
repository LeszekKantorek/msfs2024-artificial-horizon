---
name: project-prepare-pr
description: Prepare or open a pull request explaining agreed project changes and linking their issue, with mandatory local validation and acceptance evidence maintained in the issue or a linked record. Keep description-only requests local; do not use for merging, unrelated implementation, or writing a release announcement.
---

# Prepare a Project Pull Request

## Goal

Produce the requested PR description or published pull request with an accurate scope, rationale, and issue linkage, supported by local validation and acceptance records.

## Steps

Start with the requested outcome, current branch and diff, related issue, and available validation evidence.

1. Establish whether the request is for a local description or a published PR. Keep draft-only work local; a request to open a draft PR authorizes publication.
2. Inspect the current branch, base, working changes, and existing PRs. Preserve unrelated changes and use the branch already selected for the task. Use the [contribution workflow](../../../CONTRIBUTING.md#workflow) when starting a new branch.
3. Select the issue covering the implemented outcome and check [roadmap dependencies](../../../docs/roadmap.md). For publication, create a scoped issue if none fits. For a local draft, identify a missing issue without publishing one.
4. Review the diff against the agreed scope and [change requirements](../../../CONTRIBUTING.md#change-requirements). Check that local plans, credentials, SDK material, and build output are excluded.
5. Select checks through the [validation matrix](../../../docs/testing/index.md#validation-by-change-scope). Apply [evidence reuse](../../../docs/testing/index.md#evidence-reuse), run missing applicable checks locally, and record the tested revision, environment, results, and unavailable checks in the issue or a linked validation record. Correct established defects within scope and repeat affected checks.
6. Reconcile issue criteria through [acceptance tracking](../../../CONTRIBUTING.md#acceptance-criteria-and-additional-verification). Missing evidence remains unchecked in the issue, with remaining verification described there or in the linked record. Keep incomplete work in draft status. If the PR has task items, reconcile them too; unchecked tasks block acceptance.
7. Prepare the description using the [PR template](../../../.github/pull_request_template.md). State the problem and resulting behavior, why the change is needed, and important design choices or consequences. Link the issue with `Closes #N`. Do not require a test-status section, command list, or testing checklist. The closing keyword does not claim pending acceptance passed. For a description-only request, deliver this draft and stop.
8. For publication, inspect the exact staged diff, commit the intended changes, push the selected branch, and create or update its PR against the agreed base. Preserve an existing suitable commit; if the tree changes after validation, reassess the evidence before publishing.
9. Verify the saved PR's base, head, description, issue linkage, and draft status. Inspect the `Acceptance criteria` status and preserve unrelated repository protections. Application builds and test suites run locally; their GitHub status is not required. Report pending, failed, or unavailable results accurately. Return the PR URL and attach it to the task when the host supports PR attachments. Do not merge as part of this procedure.

Finish with the requested local draft or verified PR record and explicit remaining checks.
If publication or verification fails, report the completed local work and the blocking operation without claiming a PR was created.
