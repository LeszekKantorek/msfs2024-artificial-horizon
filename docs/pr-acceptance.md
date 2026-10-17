# PR acceptance control

The acceptance control uses this workflow configuration:

| Property | Value |
| --- | --- |
| Workflow | `PR acceptance` |
| Runner | `ubuntu-latest` |
| PR scope | Open PRs targeting `main` |
| Published commit status | `Acceptance criteria` |

> The main ruleset must require `Acceptance criteria` to block merge.

## Prepare an issue and PR

Follow [CONTRIBUTING](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) for author responsibilities, evidence, and pending verification.
The sections below define how the control interprets those records and publishes its status.

### Issue references

Use one closing keyword per issue, for example `Closes #21. Fixes #48.`

| Reference | Result |
| --- | --- |
| Issue number in the same repository | Supported |
| Qualified reference to an issue in the same repository | Supported |
| GitHub issue URL for the same repository | Supported |
| Cross-repository reference | Validation failure |
| PR number instead of an issue | Validation failure |
| Issue without an acceptance checklist | Validation failure |

### Checklist scope

The validator applies these selection rules:

| Location or format | Treatment |
| --- | --- |
| Tasks under the issue's `Acceptance criteria` heading, including nested subsections | Required |
| Tasks elsewhere in the issue | Outside this control |
| Visible unchecked PR tasks in any section | Block the status |
| Fenced or indented code, HTML comments, and quoted task lists | Do not satisfy checklist requirements |
| PR without task items | Can pass when every linked issue criterion is checked |

Checklist parsing and wording follow these rules:

* Use Markdown headings such as `## Acceptance criteria`.
* Heading matching ignores case.
* The pinned [Marked parser](https://marked.js.org/using_advanced) reads GFM tokens without rendering HTML.
* The control checks issue and PR checkboxes independently.
* PR wording may reflect the PR context without matching or copying issue text.
* A PR without task items needs no dummy checkbox to satisfy the validator.
* New features still require the [manual checklist](../CONTRIBUTING.md#workflow).

> A checked box records a verification claim. The workflow cannot establish whether a physical-device test occurred.

## Understand the result

| Status | Meaning | Action |
| --- | --- | --- |
| `pending` | A PR event started fresh validation | Wait for the result |
| `failure` | References, issue Acceptance criteria or PR tasks are incomplete | Read the workflow log and correct the listed items |
| `error` | Current GitHub data could not be read or validation failed | Resolve the API failure and rerun the workflow |
| `success` | Linked criteria and the PR verification checklist satisfy the control | Other required checks still apply |

### Validation triggers

These events refresh acceptance status:

| Trigger | Events or purpose |
| --- | --- |
| PR events | Creation, description edits, new commits, reopening, closure, and marking ready |
| Issue events | Edits, reopening, and closure trigger validation of open PRs |
| Manual dispatch | Recovery after missed events or token-generated edits |
| Twice-hourly schedule | Recovery after missed events or token-generated edits |

> GitHub can delay scheduled jobs.

### Commit status consistency

The control applies these consistency rules:

* Status belongs to the current PR head commit.
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

1. Merge the workflow and validator into `main` through the existing required checks.
2. Open **Actions → PR acceptance → Run workflow**, using `main`.
3. In **Settings → Rules → Rulesets → main**, edit the existing required status checks.
4. Compare required checks with names published by the current workflows, including [Windows CI](../.github/workflows/ci.yml) status `Build & Test`.
5. Replace an obsolete `check` requirement if present while preserving other valid requirements.
6. Add `Acceptance criteria`, with GitHub Actions as its source.
7. Save the ruleset.
8. Check that it remains active, targets `main`, and has no unintended bypass actors.
9. Confirm that a PR with missing or unchecked verification cannot merge.
10. Complete the remaining verification and check its items.
11. Confirm a successful acceptance status.

> Activate the requirement after the trusted workflow reaches `main`. Earlier activation would block the bootstrap PR that installs it.

## Validate changes to the control

Run the acceptance validator suite from the repository root:

```powershell
npm ci --prefix .github/scripts --ignore-scripts
node --test .github/scripts/acceptance-check.test.mjs
```

The suite covers these boundaries:

| Area | Cases |
| --- | --- |
| Parsing | References, independent checkbox checks, hidden examples, and issue section boundaries |
| API access | Failures and pagination |
| Concurrent changes | Metadata edits, new heads, and shared commits |
| Repeated execution | Repeated runs |

CI keeps acceptance tooling separate from application checks:

* `Acceptance validator tests` runs this suite on `ubuntu-latest` with read-only permissions.
* Browser controller tests and Rust checks remain in Windows CI.

### Trust boundary

Preserve these workflow permissions and execution limits:

| Property | Required boundary |
| --- | --- |
| Event | `pull_request_target` |
| Checkout | Only `main` |
| Read permissions | Repository contents, issues, and PRs |
| Write permissions | Commit statuses |
| Dependencies | Locked and installed with `--ignore-scripts` |
| Dependency use | CI tooling only, without bundling into the application |

The acceptance workflow must never:

* Execute code from the PR head.
* Persist checkout credentials.
* Interpolate PR or issue text into shell commands.

> Extensions must preserve this trust boundary.
