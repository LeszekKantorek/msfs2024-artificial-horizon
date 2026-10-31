---
name: project-prepare-pr
description: Prepare or open a pull request explaining agreed changes with optional issue linkage and mandatory local validation evidence in the PR, a related issue, or a durable linked record. Keep description-only requests local; do not use for merging, unrelated implementation, or writing a release announcement.
---

# Prepare a Project Pull Request

## Goal

Produce the requested PR description or published pull request with accurate scope, rationale, optional issue linkage, and local validation evidence.

## Steps

Start with the requested outcome, current branch and diff, any related issue, and available validation evidence.

1. Establish whether the request is for a local description or a published PR. Keep draft-only work local; a request to open a draft PR authorizes publication.
2. Inspect the current branch, base, working changes, and existing PRs. Preserve unrelated changes and use the branch already selected for the task. Use the [contribution workflow](../../../CONTRIBUTING.md#workflow) when starting a new branch.
3. Link a relevant issue when one exists and check applicable [roadmap dependencies](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9). Create an issue only when requested or needed to organize authorized work, never solely for PR publication. Omit issue linkage when none applies.
4. Review the diff against the agreed scope and [change requirements](../../../CONTRIBUTING.md#change-requirements). Check that local plans, credentials, SDK material, and build output are excluded.
5. Select checks through the [validation matrix](../../../docs/testing/index.md#validation-by-change-scope). Apply [evidence reuse](../../../docs/testing/index.md#evidence-reuse), run missing applicable checks locally, and record the tested revision, environment, results, and unavailable checks in the PR, a related issue, or a durable linked validation record. Correct established defects within scope and repeat affected checks.
6. Reconcile any supplied criteria through [acceptance tracking](../../../CONTRIBUTING.md#acceptance-criteria-and-additional-verification). Leave unconfirmed items unchecked and record remaining verification in the PR, related issue, or linked record. Keep incomplete work in draft status. Issue linkage and checklist presence are optional; visible unchecked PR tasks and unchecked criteria in closing-linked issues still block acceptance.
7. Prepare the description using the [PR template](../../../.github/pull_request_template.md). State the problem, resulting behavior, rationale, and important consequences. Use `Closes #N` only when completing a related issue; no issue is required. Do not require a test-status section, command list, or checklist. A closing keyword does not claim pending acceptance passed. For a description-only request, deliver this draft and stop.
8. For publication, inspect the exact staged diff, commit the intended changes, push the selected branch, and create or update its PR against the agreed base. Preserve an existing suitable commit; if the tree changes after validation, reassess the evidence before publishing.
9. Verify the saved PR's base, head, description, issue linkage, and draft status. Inspect the `Acceptance criteria` status and preserve unrelated repository protections. Application builds and test suites run locally; their GitHub status is not required. Report pending, failed, or unavailable results accurately. Return the PR URL and attach it to the task when the host supports PR attachments. Do not merge as part of this procedure.

Finish with the requested local draft or verified PR record and explicit remaining checks.
If publication or verification fails, report the completed local work and the blocking operation without claiming a PR was created.
