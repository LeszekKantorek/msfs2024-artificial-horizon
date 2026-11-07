# Independent implementation review

Start a separate chat with Sol Medium, independent of the implementer. Replace the angle-bracket fields and copy the prompt below.

```text
Independently review implementation of Subtask <ISSUE_URL> in LeszekKantorek/msfs2024-artificial-horizon.

Review target: <PR_URL or exact BASE_SHA..HEAD_SHA; identify any uncommitted patch separately>.
Implementation handoff and evidence: <LINKS or pasted summary>.
Validation: <static review and existing evidence only | explicitly authorized targeted checks>.
Report destination: <this chat only | this chat and a PR review comment>.

Read AGENTS.md and retrieve the governing context through a Luna High context-retrieve subagent. Read the current issue description, first context comment, amendments, artifacts, parent requirements, predecessor handoff, and applicable successor instructions. Follow the tasks-tests-review plan linked from the issues. Do not depend on .local/ files or treat the implementation summary as proof.

Review the exact diff and relevant unchanged producers, consumers, contracts, and cleanup paths. Check correctness, regressions, failure handling, requirement coverage, and whether tests can detect the relevant failures independently of the implementation.

Check both the Subtask's outcome and the prerequisites it supplies to its successor. For #91 this includes S1 -> S2, S2 -> S3, S3 -> S4, and the other transitions in the accepted plan. Missing predecessor deliverables are findings; implementation intentionally owned by the next Subtask is not a predecessor defect.

Identify additions beyond the accepted requirements, including extra abstractions, features, restrictions, and validation obligations. Explain their purpose, cost, and whether the task needs them. Label proposals separately from defects. Do not remove or redesign anything during review.

Reuse relevant validation evidence, retaining its revision, attribution, and limits. Run only checks authorized above and needed to resolve a concrete uncertainty. For #91, do not run or request screenshots or visual approval for S1-S6. S8 owns the integrated visual evidence; missing earlier screenshots are not a defect.

Do not edit code, commit, merge, close issues, or change checklists. Publish a report only to the selected destination. Do not create an extra reviewer layer; perform this independent review yourself. Delegate only the bounded context retrieval described above.

Return actionable findings ordered by severity. Each finding needs a file and line or exact requirement reference, triggering situation, consequence, and supporting evidence. Distinguish confirmed defects, unresolved questions, and optional proposals. State explicitly if there are no actionable findings.

Finish with the reviewed base/head, evidence used, checks actually run, unreviewed scope, and a successor-readiness verdict: ready, blocked, or not verified. For every gap, name the affected prerequisite, responsible task, and evidence needed. Passing tests or no findings alone do not establish full Task acceptance.
```
