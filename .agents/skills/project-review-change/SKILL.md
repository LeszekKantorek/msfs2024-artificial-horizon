---
name: project-review-change
description: Review a project diff or scoped implementation for contract, lifecycle, maintainability, and evidence gaps. Coordinate independent review only when delegation is authorized. Exclude failure diagnosis, test execution alone, and agent-behavior diagnosis.
---

# Review a project change

## Goal

Produce assessed findings for an identified implementation, with supporting evidence, correction costs, coverage limits, and verification of any authorized fixes.

## Steps

### 1. Establish the scope and permissions

* Identify the requested outcome, checkout, and HEAD. Use the table to establish the reviewed state.

| Requested scope | Record |
| --- | --- |
| Committed diff | Exact base and head revisions, changed files, and relevant callers. |
| Scoped implementation | Selected mechanisms, their entry points, and excluded areas. |
| Uncommitted changes | A patch or equivalent snapshot identifying the included changes, in addition to the base revision. |

* Determine permission to edit separately from permission to execute tests, builds, browsers, or reproductions.
* For static-only work, inspect source and assertions without execution.
* Keep reviewed files unchanged during inspection. Review alone does not authorize corrections or GitHub publication.
* Keep product redesign outside a documentation-format review.
* Continue when the reviewed state, exclusions, and permitted actions are explicit.

### 2. Prepare the governing context and assignments

* Use existing context maps to select requirements and accepted decisions for the mechanisms from Step 1.
* Read applicable owners: `docs/telemetry-contract.md` for field semantics, `docs/efi/` for presentation, `docs/sources/` for acquisition, and `docs/server.md` for delivery and shutdown.
* Check governing ADRs before challenging intentional boundaries. Retain conflicting requirements for assessment in Step 5.
* For issue-based work, use [issue and artifact retrieval](../../../.context/repository-instructions.md#read-github-issue-context). Include relevant artifact references and unresolved reads.
* Prepare the [Subtask handoff](../../../.context/iterative-development.md#accepted-plans-and-handoffs) when applicable. Include accepted decisions, amendments, preserved invariants, validation evidence, and known limitations.
* Include a governing supplied plan when one exists. State which inputs are absent and use the agreed scope when no Subtask or accepted plan applies.
* Attribute existing validation to its original evidence. Missing checks do not prevent static inspection.

Choose how to perform the review:

| Condition | Action |
| --- | --- |
| Delegation is authorized and available | Supply reviewers with the scope, context, permitted methods, and Steps 3–5. Require findings and coverage evidence. |
| Delegation is unauthorized or unavailable | Perform the authorized local review and identify it as self-review. Selecting this skill does not authorize delegation. |
| Several reviewers are authorized | Assign complementary mechanisms, such as semantic flow and resource lifecycle. Require a local maintainability pass in each assignment. |
| Independent discovery is requested | Withhold previous findings until reports arrive. |
| Regression review is requested | Supply earlier findings and require a disposition for each. |

* Continue when each selected mechanism has governing context or a recorded gap, and responsibility for inspection is assigned.

### 3. Trace behavior across boundaries

* Follow relevant callers beyond changed files. Compare implementation with the requirements selected in Step 2.
* For telemetry, trace acquisition, normalization, publication, SSE, browser acceptance, and presentation as applicable to the scope.
* For resources, trace acquisition, partial startup, operation, failure, cleanup, and final reporting.
* Use the table to select boundary cases and distinguish superficially similar behavior.

| Trigger | Trace and compare | Distinction to preserve |
| --- | --- | --- |
| Optional indication or representation | Follow valid, omitted, null, invalid, expired, and recovered values through producers, wire fields, normalized frames, and rendering. Inspect fixtures too. | Missing numeric data must not become plausible legacy text or zero. Independent optional absence differs from version compatibility. |
| Time, gap, or reconnect | Identify timestamp ownership, units, inclusive endpoints, overlapping intervals, and reset rules. Trace optional updates and later publications. | No native observation differs from immediate field invalidation. Cached publication differs from renewed acquisition age. |
| Owned process, native handle, socket, stream, or timer | Locate the cleanup boundary from first acquisition. Follow startup failure and every awaited dependency during cancellation. | Aborting HTTP does not necessarily interrupt output drain or final flush. Successful startup cleanup does not cover failed startup. |
| Primary failure followed by cleanup failure | Follow both errors through return values, console output, and enabled or disabled logging. | Loss of the reported primary error differs from loss of all diagnostics. Preserve intentional native-resource retention after unconfirmed Close. |
| Test or acceptance claim | Trace fixture creation to the implementation actually exercised. Compare assertions with an independent requirement. | Synthetic normalized samples do not prove native receiver timing. A presentation fixture can remain useful without proving acquisition behavior. |

* Investigate other mechanisms when their evidence warrants it. The table does not define exhaustive coverage.
* For timing claims, distinguish reachable publications from guaranteed browser visibility. SSE and drawing can coalesce intermediate values.
* Use concrete boundary sequences, including repeated and overlapping events. Label unexecuted sequences as static scenarios, not reproductions.
* Record candidates and counterevidence for assessment in Step 5.
* Continue when each selected trace has supporting evidence or an explicit unresolved boundary.

### 4. Identify concrete maintenance costs

* Inspect the mechanisms traced in Step 3 even when behavioral defects already exist.
* Compare names, types, comments, and actual reads. Check whether a time parameter supplies a timestamp or merely signals an event.
* Expose hidden units, inclusive progress boundaries, instrument identities, and availability decisions when their meaning requires reconstructing distant code.
* Find rules requiring synchronized edits across catalog order, conversions, history, gates, and assertions. Distinguish them from independent test data.
* Compare module comments with current source selection and composition. Historical future-work comments can misstate current dependencies.
* Inspect compound cleanup statements and reused state for hidden ordering or error precedence.
* Retain helpers and forwarding layers that own lifecycle, geometry, ABI, or dependency contracts.
* For each candidate, describe a concrete reading, change, or verification cost. A runtime defect is not required for a maintenance finding.
* Do not demand a rename when callers already communicate the distinction, or split modules based on size alone.
* Propose the smallest correction that removes the demonstrated cost. Justify an enum, registry, or framework through that cost before proposing it.
* Continue when material maintenance candidates have evidence and a proposed correction, or an explicit reason for exclusion or uncertainty.

### 5. Assess candidates and reconcile reports

* For delegated work, wait for returned reports. Mark interrupted, failed, or unreturned assignments as incomplete.
* Check candidates from Steps 3–4 against current source, governing decisions, and counterevidence. Historical reports supply questions, not proof of current defects.
* Require severity, file and line, evidence, consequence, smallest correction, and correction cost for each supported finding.
* Distinguish implementation defects, maintenance costs, contradictory requirements, and missing evidence. Use the table to determine their disposition.

| Evidence | Disposition and next action |
| --- | --- |
| Source and requirements support the candidate | Retain the finding and explain its consequence. Use Step 6 to determine whether to report or correct it. |
| Counterevidence refutes the candidate | Exclude it and explain the decisive evidence. |
| Requirements conflict or evidence cannot distinguish explanations | Retain the unresolved question and identify the missing decision or evidence. Do not select new product behavior implicitly. |
| Reports describe the same cause | Reconcile them into one finding while preserving distinct consequences and evidence limits. |
| Reviewers disagree | Compare their evidence and assumptions. Finding counts and repeated votes do not determine severity. |

* Record other evidence states explicitly when these rows do not describe them.
* For supplied earlier findings, record rediscovered, excluded with counterevidence, unresolved, or not examined. Silence does not establish resolution.
* Continue when material candidates have dispositions and incomplete assignments remain visible. Zero findings is valid.

### 6. Deliver findings or verify authorized corrections

* Use the assessed findings from Step 5 to select the next action.

| Authorization and state | Action |
| --- | --- |
| Review only | Report findings and proposed corrections without changing the implementation. |
| Corrections are authorized | Apply established corrections and perform affected checks within the execution permission from Step 1. Select checks through [evidence selection](../../../docs/testing/evidence.md#select-evidence-by-failure-risk). |
| Material fixes have been made | Repeat the affected review steps under the original delegation conditions. Supply findings, dispositions, and subsequent accepted decisions. |
| A follow-up uses another reviewer | Disclose the reviewer change and provide the same review context. |
| Required checks or follow-up review cannot complete | Report the unresolved condition and its effect on the result. Do not claim verification. |

* Deliver the local report or received independent reports with assessed findings, actual mechanisms traced, exclusions, and remaining gaps.
* Identify the resulting reviewed revision or snapshot. Mark later unreviewed changes explicitly.
* Separate static reasoning, executed tests, supplied evidence, and actual device acceptance under the [evidence limits](../../../docs/testing/evidence.md#sources-and-limits).
* Keep unavailable device checks visible under [acceptance tracking](../../../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).
* Claim completed independent review only after receiving its report. Self-review, passing tests, or a review request alone do not establish it.
* Publish to an issue or PR only when authorized.
* Finish when the scope has an assessed report and authorized corrections have verification evidence or explicit gaps. Review does not establish the absence of defects.
