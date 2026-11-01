---
description: Find contribution constraints, acceptance procedures, and follow-up guidance.
status: active
---

## Repository rules

Use these owners for repository requirements and procedures:

* Apply the project constraints in [AGENTS](../AGENTS.md#project-constraints).
* Apply the contribution requirements in [CONTRIBUTING](../CONTRIBUTING.md#change-requirements).
* Follow the [workflow](../CONTRIBUTING.md#workflow) for branch and PR preparation.
* Use [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) for evidence and pending checks.

## Follow-up findings

Use [finding promotion](../CONTRIBUTING.md#promote-agreed-findings) to continue Work Item planning from confirmed results or define an agreed follow-up.

## GitHub issue records

### Read GitHub issue context

Before planning or execution, read the issue and its required parent context:

1. Fetch current issue content, metadata, and relevant comments. Always list artifacts separately, even when the issue response does not mention them.
   Refresh this context for new execution sessions or relevant changes.
2. Check each response before continuing. Treat failed or incomplete reads as missing context, not an empty artifact list.
   Resolve missing context before dependent work.
   For pagination, request another page after each successful page containing 100 artifacts.
   Stop after a successful page containing fewer than 100 artifacts.
3. Read complete artifact content and relevant linked material. Retrieve individual artifacts when their content is missing or truncated.
   Reuse previously read content only when its identity and update time remain unchanged.
   Inspect HTML source without running embedded code.
4. Compare artifacts with the issue, current contracts, and accepted decisions.
   Resolve consequential conflicts before dependent work. Keep proposals distinct from accepted requirements.
5. Cite artifact URLs and `updated_at` values in plans and handoffs.

List artifacts through a supported reader or the connector's general fetch operation:

`GET /repos/{owner}/{repo}/issues/{issue_number}/artifacts?per_page=100&page=1`

See the [issue artifacts REST API](https://docs.github.com/en/rest/issues/artifacts) for individual retrieval and response details.

### Maintain issue records

Issue maintenance follows these authorization boundaries:

* Do not automatically correct historical or completed issues, including their text, checklists, links, or status.
* Change historical records only when the user explicitly requests that change.
* Maintain Initiatives, Work Items, and Tasks only within the authorized scope. Planning, publication, and reorganization are separate permissions.
* Return complete drafts when publication is not authorized. Verify labels and actual parent/sub-issue relationships after authorized publication.
* An open issue is not permission to rewrite it during documentation review.
* Preserve requirements in their canonical Markdown owner without editing the source issue.
* Report obsolete links or contradictory statements in historical issues instead of silently repairing them.
* Honor explicit exclusions for individual issues, even when they remain open.

Keep records in the owners defined by the [work organization model](../docs/work-organization.md):

* Keep requirements and contracts in documentation.
* Keep work, unresolved proposals, execution results, and acceptance evidence in Issues.
* Retain execution gaps and consultation details in the main conversation under [execution reporting](iterative-development.md#record-execution-findings).
* State requirements so they can be understood without opening an issue.
* Issue links can identify work or supporting evidence. They must not replace the requirement in its documentation owner.
* Keep accepted requirements for future features in documentation, explicitly marked as planned.
* An unresolved idea does not become a requirement merely because it appears in an issue.

Use the work organization model when checking completion or planning a separately authorized migration of existing Issues.
