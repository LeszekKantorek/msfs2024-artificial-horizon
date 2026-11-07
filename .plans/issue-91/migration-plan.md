# Publish the revised #91 handoff

Status: Proposed publication procedure. This request prepares the plan only; no remote changes have been made.

## Outcome and roles

GitHub contains every requirement, contract, scenario draft, execution context, and dependency needed to continue without this chat or `.local/`.
One GPT-6.1 Terra agent at high performs the later authorized migration. The root agent independently reads back GitHub and verifies completeness after Terra finishes.
The migration changes planning records and publishes a planning snapshot. It does not implement the application, execute scenarios/tests/builds, or merge documentation into main.

## Current remote evidence

Read-only retrieval on 2026-11-07 found the following state. All six separately queried issue-artifact lists were empty; each successful first page contained fewer than 100 entries.

| Issue | State and relationship | Updated at (UTC) |
| --- | --- | --- |
| #9 | Open Epic | 2026-11-06T21:07:05Z |
| #91 | Open Task under #9; original YAML/conversion scope | 2026-11-06T22:10:20Z |
| #123 | Open Subtask under #91; YAML migration | 2026-11-06T22:10:25Z |
| #124 | Open Subtask under #91; direct playback integration | 2026-11-06T21:51:00Z |
| #125 | Open Subtask under #91; conversion evidence | 2026-11-06T21:52:45Z |
| #126 | Open Subtask under #91; converter implementation | 2026-11-06T21:51:39Z |

The current first context comment on #91 is [6696677478](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91#issuecomment-6696677478), updated 2026-11-06T22:10:20Z.
Refresh all affected records, comments, artifacts, labels, parent relationships, and dependencies immediately before publication. A changed result must be reconciled before overwriting it.

## Content destinations

| Source | GitHub destination and purpose |
| --- | --- |
| `task-91-amendment.md` | #91 title and Description: result, requirements, exclusions, acceptance, and links to the active execution plan |
| `architecture.md`, `contracts.md`, current decisions | #91 first context comment: architectural summary, authoritative links, accepted decisions versus proposals, and remaining native evidence gaps |
| Active architecture/contracts, catalog, error inventory, evidence, JSONL examples and scenarios | Planning snapshot on `codex/issue-91-planning`, under `.plans/issue-91/`, linked by immutable commit SHA |
| Each `subtasks/*.md` issue-body section | Description of a new Subtask under #91 |
| Each Subtask's first-comment section | Its first execution-context comment, with precise published source/code references |
| `tasks-tests-review.md` and targeted reviewer sections | Commit-pinned plan linked from #91; preserve S1 -> S2, S2 -> S3, and S3 -> S4 readiness checks in their first context comments |
| `plan.md` dependency information | #91 execution table, actual parent/sub-issue relations, dependency relations, and prerequisite links in child descriptions |
| Original affected issue bodies/comments and frozen reviews | Clearly marked historical snapshot, linked only as history; never a competing execution contract |

The selected branch and .plans/issue-91/ directory store exact Markdown/JSONL files, not a new canonical documentation owner or an application delivery branch. S2 later promotes the accepted requirements into their existing project documentation owners through normal delivery.
Publish an index identifying active files and historical material. Rewrite relative links for the published layout and use commit-pinned GitHub URLs from issues. Exclude real logs, SDK paths/binaries, credentials, generated output, and unrelated local files.
Do not commit `.local/` itself. Do not introduce a permanent duplicate roadmap: delivery order lives in GitHub Issues. The snapshot can preserve the original plan as clearly historical planning evidence.

## Publication sequence

1. **Prepare and freeze the payload locally.** Refresh remote state; save originals and an explicit file/issue manifest. Finalize the parent Description, first-comment text, seven child bodies/comments, old-issue disposition comments, and required links. Preserve the supplied originals and identify content hashes. Resolve stale YAML/converter/reset requirements before publishing.
2. **Publish the file snapshot.** Use codex/issue-91-planning and create a reviewed snapshot commit containing .plans/issue-91/. Record its SHA. Read back every required file and resolve links before changing the Task. Use supported GitHub file operations; if a required capability is unavailable, report that specific missing operation instead of claiming upload success.
3. **Update #91.** Replace its YAML/conversion title and Description with the current outcome. Update its first context comment after preserving the exact previous text in the historical snapshot. Include an explicit scope-change note and links to that original, the current architecture/contracts, scenarios, and unresolved evidence. Leave #91 open.
4. **Create the seven replacement Subtasks.** Create S1, S2, S3, S4, S5, S6, and S8 with `subtask` labels and first context comments. Preserve local identifiers in the publication manifest; use actual GitHub numbers in published references. There is no S7 converter. Remove local-only planning disclaimers from execution instructions while retaining planned/unverified behavior labels.
5. **Connect the work.** Establish and verify actual #91 parent relationships. Preserve #91's existing parent #9 without migrating the Epic. Add dependencies and replace placeholders with real issue URLs. The chain is S1 -> S2 -> S3 -> S4 -> S5; S6 follows S2 and provides the input/output mapping; S8 uses S4, S6, and the selected S5 fault subset. Healthy narrative work must not wait for exhaustive rare-error coverage. Do not create a circular S4/S6 dependency: S6 defines the mapping, S4 supplies execution evidence.
6. **Retire conflicting old assignments.** Proposed migration scope includes adding a disposition comment to #123–#126 and closing them as not planned under the revised scope. #123/#124 point to their replacement work; #125/#126 explain that recorded-session conversion was withdrawn. Preserve their descriptions, comments, relationships, and evidence; do not report their old implementation criteria as completed. Record the existing #125 mapping as historical work, not missing new work. Perform this only after the new handoff is complete.
7. **Terra reports the publication result.** Return the snapshot SHA, file manifest, parent/comment URLs, old/new issue mapping, saved relationships/dependencies, operation failures, and remaining evidence gaps. Do not close #91 or claim implementation acceptance.
8. **Root verifies GitHub independently.** Fetch the actual saved issues, comments, artifact lists, files, and relationships. Compare them with the accepted local payload and checklist below. Root reports omissions; Terra corrects publication defects if needed, followed by another readback. Root does not silently redesign the plan during verification.

Before starting this sequence, the publication instruction must include the concrete proposed mutations: a planning branch, updating #91 and its first comment, creating/linking seven children, and commenting/closing #123–#126 as superseded or withdrawn. The current request authorizes this plan, not those writes.

## Verification checklist

* [ ] Another agent can start from #91 and retrieve every required file without `.local/` or chat access.
* [ ] The parent Description and first context comment agree with all child instructions.
* [ ] The tasks/tests/review plan is accessible. Each review checks the current outcome and downstream prerequisites; S1-S3 have concrete successor checks in their first context comments.
* [ ] Screenshots and visual inspection are assigned once to the integrated S8 session. S1-S6 retain applicable nonvisual checks and do not require separate appearance approval; earlier visual evidence remains explicitly pending.
* [ ] JSONL/native DLL acquisition replaces YAML/direct publication; both DLLs use the same receiver.
* [ ] The clock always increases; only the cursor loops. No implicit state restoration remains.
* [ ] Sparse scenarios, null/unavailable defaults, repeated hold samples, and gap silence are explicit.
* [ ] No converter, schema 4/5 import, observations mode, or S7 delivery requirement remains active.
* [ ] Scenario-to-result evidence distinguishes native receipt, normalization, and final log/SSE output.
* [ ] Pause delivery, empty-queue HRESULT/message details, overflow, and other unverified SDK behavior remain named implementation evidence gaps, not confirmed facts.
* [ ] HTTP-first setup failure and case-insensitive DLL basenames are accepted decisions.
* [ ] Rare fault coverage is staged, with implemented/deferred cases explicit and terminal scenarios separate.
* [ ] Scenario documentation gives purpose and links without duplicating JSONL timelines.
* [ ] Optional technical proposals remain labeled; publication does not promote them into user requirements.
* [ ] Parent relations and dependencies are verified through GitHub, not inferred from body links alone.
* [ ] All replacement links resolve, originals remain accessible, and old assignments cannot be mistaken for active work.
* [ ] No application implementation, tests, builds, simulator run, or main-branch merge is reported or performed by this migration.

## Branch-only planning directory

The selected location is .plans/issue-91/. Planning files may be committed on working branches, but .plans/ must contain no tracked entries in the final main tree. Preserve the planning commit and its immutable links; promote durable requirements to their owners before removing plan files from a merge candidate. The acceptance-control change is prepared separately from application implementation and must reach trusted main before hosted enforcement is active.
