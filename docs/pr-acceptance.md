# PR acceptance control

The acceptance control uses this workflow configuration:

| Property | Value |
| --- | --- |
| Workflow | `PR acceptance` |
| Runner | `ubuntu-latest` |
| Scope | Open PRs targeting `main`; planning-file check on pushes to `main` |
| Published commit status | `Acceptance criteria` |

> The main ruleset must require `Acceptance criteria` to block merge. The `Build & Test` status must not be required.

## Prepare a PR

Follow [CONTRIBUTING](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) for author responsibilities, evidence, and pending verification.
The sections below define how the control interprets those records and publishes its status.

### Issue references

Issue linkage is optional. Link a related issue when available; use a closing keyword when the PR completes it, for example `Closes #21. Fixes #48.`

| Reference | Result |
| --- | --- |
| Non-closing reference such as `Refs #21` | Informational; does not select an issue for validation |
| Issue number in the same repository | Supported |
| Qualified reference to an issue in the same repository | Supported |
| GitHub issue URL for the same repository | Supported |
| Cross-repository reference | Validation failure |
| PR number instead of an issue | Validation failure |

### Checklist scope

The validator checks issues selected by closing references and their `Acceptance criteria` task items when present. It also checks visible PR task items.

| Location or format | Treatment |
| --- | --- |
| Tasks supplied under a closing-linked issue's `Acceptance criteria` heading, including nested subsections | All supplied tasks must be checked |
| Tasks elsewhere in the issue | Outside this control |
| Visible unchecked PR tasks in any section | Block the status |
| Fenced or indented code, HTML comments, and quoted task lists | Excluded from task validation |

Checklist parsing and wording follow these rules:

* When supplying an issue acceptance checklist, use a Markdown heading such as `## Acceptance criteria`.
* Heading matching ignores case.
* The pinned [Marked parser](https://marked.js.org/using_advanced) reads GFM tokens without rendering HTML.
* The control checks issue and PR checkboxes independently.
* PR wording may reflect the PR context without matching or copying issue text.
* Track required automated and manual validation evidence in the PR, a related issue, or a durable linked record under [CONTRIBUTING](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).

> A checked box records a verification claim. The workflow cannot establish whether a physical-device test occurred.

### Planning files

Files under repository-root `.plans/` may be committed on working branches. They must be absent from the final tree proposed for `main`, including `.gitkeep` or other placeholders.

* For each PR targeting `main`, the validator reads the immutable head commit and its root Git tree through the API. Any root `.plans` entry fails the existing `Acceptance criteria` status, including a directory, file, symlink, or submodule.
* The check reads the complete root tree rather than only changed files. Plans retained from an earlier commit still block; plans removed from the final tree do not block because they remain in history.
* Ordinary branch commits do not trigger this rule unless proposed for `main`. Plan links can point to a preserved planning commit.
* A push to `main` checks the pushed commit even when no PR is open. This detects a violation after the push; preventing bypass pushes still depends on the repository's existing branch protections.
* Unreadable, invalid, or truncated tree responses prevent a successful check. The workflow never executes or checks out PR code.

## Understand the result

| Status | Meaning | Action |
| --- | --- | --- |
| `pending` | A PR event started fresh validation | Wait for the result |
| `failure` | Supplied closing references are invalid, supplied issue criteria or PR tasks are unchecked, `.plans` is present in the proposed main tree, or metadata changed during validation | Read the workflow log and correct the listed items |
| `error` | Current GitHub data could not be read or validation failed | Resolve the API failure and rerun the workflow |
| `success` | PR acceptance checks pass, including absence of `.plans`; a push-only result confirms the planning-file rule for that main commit | Mandatory local validation and other repository protections still apply |

### Validation triggers

These events refresh acceptance status:

| Trigger | Events or purpose |
| --- | --- |
| PR events | Creation, description edits, new commits, reopening, closure, and marking ready |
| Push to `main` | Check the pushed tree for planning files and refresh open-PR acceptance |
| Issue events | Edits, reopening, and closure trigger validation of open PRs |
| Manual dispatch | Recovery after missed events or token-generated edits |
| Twice-hourly schedule | Recovery after missed events or token-generated edits |

> GitHub can delay scheduled jobs.

### Commit status consistency

The control applies these consistency rules:

* Status belongs to the current PR head commit.
* After an issue edit, check the current `Acceptance criteria` status; older failed runs can remain visible after success.
* All open PRs sharing that commit must satisfy acceptance before its shared status succeeds.
* Runs execute serially.
* Validation reads current PR and issue bodies instead of an old event payload.
* The control rereads metadata and PR membership for the shared commit before granting success.
* Identical results are not published again, limiting repeated commit-status growth.

### API failures and recovery

Asynchronous events and API access impose these limits:

* An edit can briefly precede its updated status.
* API failures block every discovered PR head.
* An outage that prevents head discovery or status writes also prevents revocation of existing statuses.

After API access recovers:

1. Rerun the workflow.
2. Wait for a current result before merging.

## Activate merge blocking

1. Keep the trusted [acceptance workflow](../.github/workflows/acceptance.yml) and validator on `main`.
2. Open **Actions → PR acceptance → Run workflow**, using `main`.
3. In **Settings → Rules → Rulesets → main**, edit the existing required status checks.
4. Remove `Build & Test` if required. Remove `check` only if it is obsolete.
5. Retain or add `Acceptance criteria`, with GitHub Actions as its source. Preserve every unrelated protection and valid status requirement.
6. Save the ruleset.
7. Check that it remains active, targets `main`, and has no unintended bypass actors.
8. Confirm that valid supplied references and completed tasks pass, while invalid closing references or supplied unchecked tasks block acceptance.
9. Complete remaining verification in the PR, related issue, or durable linked record and update any supplied criteria.
10. Confirm a successful acceptance status.

> Activate `Acceptance criteria` after the trusted workflow reaches `main` so the workflow can publish the required status.

## Validate changes to the control

Run the acceptance validator suite from the repository root:

```powershell
npm ci --prefix .github/scripts --ignore-scripts
node --test .github/scripts/acceptance-check.test.mjs
```

The suite covers these boundaries:

| Area | Cases |
| --- | --- |
| Parsing | Optional references and checklist presence, supplied checkbox checks, hidden examples, and issue section boundaries |
| API access | Failures and pagination |
| Concurrent changes | Metadata edits, new heads, and shared commits |
| Repeated execution | Repeated runs |
| Planning files | Main-bound PRs, root entry types, removal from the final tree, API failures, and main versus working-branch pushes |

Run this suite locally when changing the validator or its workflow.
The `PR acceptance` workflow validates supplied closing references, issue criteria, PR tasks, and the planning-file rule. It does not run this suite, application tests, or builds.

### Verify deployment

1. Merge the reviewed validator change into trusted `main` before testing the new behavior in the hosted workflow.
2. Inspect the active rules for `main`. Retain the required `Acceptance criteria` status and unrelated protections.
3. With authorization to publish test PRs, verify representative metadata cases from the validator suite against the deployed workflow.
4. Confirm that the hosted workflow publishes the expected `Acceptance criteria` status for each tested head.
5. Record the PR, head revision, workflow run, and result in the delivery evidence. Local tests alone do not establish this deployed result.

### Trust boundary

Preserve these workflow permissions and execution limits:

| Property | Required boundary |
| --- | --- |
| Events | `pull_request_target`, main pushes, issue updates, scheduled runs, and manual dispatch |
| Checkout | Only `main` |
| Read permissions | Repository contents, issues, and PRs |
| Write permissions | Commit statuses |
| Dependencies | Locked and installed with `--ignore-scripts` |
| Dependency use | Acceptance tooling only, without bundling into the application |

The acceptance workflow must never:

* Execute code from the PR head.
* Persist checkout credentials.
* Interpolate PR or issue text into shell commands.

> Extensions must preserve this trust boundary.
