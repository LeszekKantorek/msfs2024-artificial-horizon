# PR acceptance control

The `PR acceptance` workflow runs on `ubuntu-latest` and publishes an `Acceptance criteria` commit status for open PRs targeting `main`.
The main ruleset must require that status to block merge.

## Prepare an issue and PR

1. Give the issue an `Acceptance criteria` heading with Markdown task items.
2. Mark criteria confirmed by unit tests or automated validation as checked and record the evidence.
3. Add a closing reference to the PR, such as `Closes #21`.
4. List pending PR verification as unchecked tasks. Wording can differ from the issue to reflect the PR context.
5. Describe the verification steps and expected results.
6. Complete verification and check every issue Acceptance criterion and every PR task.

Use a separate closing keyword for each issue, for example `Closes #21. Fixes #48.`
Same-repository qualified references and GitHub issue URLs are also supported.
Cross-repository references, PR numbers, and issues without an acceptance checklist fail validation.
Use Markdown headings such as `## Acceptance criteria`; heading matching ignores case.
The validator reads GFM tokens with a pinned [Marked parser](https://marked.js.org/using_advanced); it does not render HTML.
The control checks checkbox states independently and does not match or require copied text.

Only tasks under the issue's `Acceptance criteria` heading are required, including nested subsections.
Tasks elsewhere in the issue are outside this control.
Every visible unchecked PR task blocks the status, regardless of its section.
Fenced or indented code examples, HTML comments, and quoted task lists do not satisfy checklist requirements.
A PR with no task items can pass when all linked issue criteria are checked; no dummy checkbox is required.

> A checked box records a verification claim; the workflow cannot establish whether a physical-device test was performed.

## Understand the result

| Status | Meaning | Action |
| --- | --- | --- |
| `pending` | A PR event started fresh validation | Wait for the result |
| `failure` | References, issue Acceptance criteria or PR tasks are incomplete | Read the workflow log and correct the listed items |
| `error` | Current GitHub data could not be read or validation failed | Resolve the API failure and rerun the workflow |
| `success` | Linked criteria and the PR verification checklist satisfy the control | Other required checks still apply |

PR creation, description edits, new commits, reopening, closure, and marking ready trigger validation.
Issue edits, reopening, and closure also trigger validation of open PRs.
Manual dispatch and a twice-hourly schedule provide recovery after missed events or token-generated edits.
Scheduled jobs can be delayed by GitHub.
Identical results are not republished, avoiding repeated commit-status growth.

The status belongs to the current PR head commit.
If multiple open PRs share that commit, they must all satisfy acceptance before the shared status succeeds.
Runs are serialized and read current PR/issue bodies rather than validating an old event payload.
Metadata and shared-head PR membership are read again before granting success.

GitHub events and API reads are asynchronous; an edit can briefly precede its updated status.
API failures block every discovered head. If an outage prevents discovering any PR heads or writing statuses, existing statuses cannot be revoked.
Rerun the workflow after recovery and wait for a current result before merging.

## Activate merge blocking

1. Merge the workflow and validator into `main` through the existing required checks.
2. Open **Actions → PR acceptance → Run workflow**, using `main`.
3. In **Settings → Rules → Rulesets → main**, edit the existing required status checks.
4. Preserve the existing `check` requirement and add `Acceptance criteria`, with GitHub Actions as its source.
5. Save the ruleset and verify that it remains active, targets `main`, and has no unintended bypass actors.
6. Confirm that a PR with missing or unchecked verification cannot merge, then check the remaining items and confirm a successful acceptance status.

> Activate the new requirement after the trusted workflow reaches `main`; enabling it first would block the bootstrap PR that installs it.

## Validate changes to the control

Run the focused or full Node checks from the repository root:

```powershell
npm ci --prefix .github/scripts --ignore-scripts
node --test .github/scripts/acceptance-check.test.mjs
```

The focused tests cover reference parsing, independent checkbox checks, hidden examples, issue section boundaries, API errors, pagination, concurrent metadata edits, new heads, shared commits and repeated runs.
`Acceptance validator tests` runs these tests on `ubuntu-latest` with read-only permissions.
The existing browser controller tests and Rust checks remain in Windows CI.

The privileged workflow uses `pull_request_target`, checks out only `main`, and grants read access to contents/issues/PRs plus write access to commit statuses.
The acceptance workflow never executes code from the PR head, persists checkout credentials, or interpolates PR/issue text into shell commands.
Dependencies are locked and installed with `--ignore-scripts`; they are used only by CI tooling and are not bundled into the app.
Do not change that trust boundary when extending the control.
