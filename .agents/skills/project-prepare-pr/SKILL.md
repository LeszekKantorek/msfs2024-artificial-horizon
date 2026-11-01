---
name: project-prepare-pr
description: Prepare or open a pull request explaining agreed changes with accurate scope, rationale, and local validation evidence. Keep description-only requests local; do not use for merging, unrelated implementation, or writing a release announcement.
---

# Prepare a Project Pull Request

## Goal

Produce the requested PR description or published pull request supported by accurate scope, rationale, and local validation evidence.

## Steps

Use the requested outcome, current branch and diff, related issues, and available validation evidence to prepare the PR:

1. Establish whether the request is for a local description or a published PR. Keep draft-only work local; a request to open a draft PR authorizes publication.
2. Inspect the current branch, base, working changes, and existing PRs under these constraints:
   * Preserve unrelated changes and use the branch already selected for the Task.
   * When starting a new branch, follow the [contribution workflow](../../../CONTRIBUTING.md#workflow).
3. Establish the scope and prerequisites from the available records:
   * Read the Task when available, its Work Item, and the relevant Initiative under the [work organization model](../../../docs/work-organization.md).
   * For work without an issue, use the agreed outcome.
4. Review the diff against the agreed scope and [change requirements](../../../CONTRIBUTING.md#change-requirements).
   Exclude local plans, credentials, SDK material, and build output.
5. Establish applicable validation evidence under these requirements:
   * Select checks through the [validation matrix](../../../docs/testing/index.md#validation-by-change-scope) and apply [evidence reuse](../../../docs/testing/index.md#evidence-reuse).
   * Run missing applicable checks locally. Correct established defects within scope and repeat affected checks.
   * Record the tested revision, environment, results, and unavailable checks in the PR, related issue, or durable linked validation record.
6. Reconcile completion under [acceptance tracking](../../../CONTRIBUTING.md#acceptance-criteria-and-additional-verification):
   * Check the Task's criteria, or related issue criteria when there is no Task, and reconcile supplied PR tasks.
   * Leave unconfirmed items unchecked and record remaining verification with the evidence. Keep incomplete work in draft status.
   * Require separate evidence before claiming parent completion.
7. Prepare the description using the [PR template](../../../.github/pull_request_template.md) and these content requirements:
   * State the problem, resulting behavior, rationale, and important consequences.
   * Link the Task when available and reference its Work Item for context. A Task can use several PRs or no PR.
   * Use `Closes #N` only when the PR completes that issue's criteria. A closing keyword does not establish pending acceptance.
   * Keep a test-status section, command list, and checklist optional.
   * For a description-only request, deliver the draft and stop before publication.
8. For authorized publication, perform these actions in order:
   1. Inspect the exact staged diff and commit the intended changes, preserving an existing suitable commit.
   2. If the tree changed after validation, reassess the evidence before publishing.
   3. Push the selected branch and create or update its PR against the agreed base.
9. Verify the published result under these requirements:
   * Check the saved base, head, description, issue linkage, and draft status.
   * Inspect the `Acceptance criteria` status and preserve unrelated repository protections.
   * Run application builds and test suites locally when applicable. Their GitHub status is not required.
   * Report pending, failed, or unavailable results accurately.
   * Return the PR URL and attach it to the task when the host supports PR attachments.
   * Leave merging outside this procedure.

The final result must satisfy these reporting requirements:

* Deliver the requested local draft or verified PR record with explicit remaining checks.
* If publication or verification fails, report completed local work and the blocking operation.
  Claim that a PR was created only when that result is verified.
