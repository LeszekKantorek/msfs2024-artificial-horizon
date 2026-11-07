# Integrated Task acceptance

Use a separate Sol Medium chat for the acceptance assessment. This model choice is a template recommendation, not a new repository requirement. Replace the angle-bracket fields and copy the prompt below.

```text
Assess delivery of Task <TASK_URL; for the current work, https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91> as a complete integrated result.

Delivery to assess: <INTEGRATED_SHA and PR/artifact references>.
Implementation, review, and validation evidence: <LINKS>.
Validation: <static assessment and existing evidence only | explicitly authorized missing integrated checks>.
Report destination: <this chat only | this chat and a Task comment>.

Read AGENTS.md and retrieve relevant sources through a Luna High context-retrieve subagent. Read the current Task and applicable Epic requirements, amendments, artifacts, Subtask outcomes, independent review findings and dispositions, and the linked tasks-tests-review execution plan. Read the governing passages yourself; do not rely on local planning files being available.

Assess the integrated revision against the Task's own criteria. Closed Subtasks, merged PRs, passing statuses, or reviewer approval do not establish the parent outcome. Check that all delivered components and artifacts belong to the assessed revision and work through the agreed production boundaries.

For #91, trace scenario -> demo.dll -> shared SimConnect receiver -> normalization/publisher -> log/SSE -> UI. Check the accepted DLL selection, used API registrations and requests, JSONL playback, continuously increasing clock and looping steps, sparse values, unchanged hold samples, gaps, explicit pause/flight events, selected fault coverage, and scenario catalog. Use the published contracts for exact semantics rather than inventing new ones. Confirm that removed converter work has not returned.

Check evidence across Subtask boundaries, including S6's independent scenario-to-output mapping. Distinguish implemented behavior from assumptions requiring comparison with the official SimConnect runtime. Retain simulator and physical-device evidence limits. Named deferred rare faults must be assessed against the accepted staged scope, not treated automatically as missing required work.

Reuse valid evidence rather than repeating every Subtask's tests. S8 owns the consolidated visual session. Verify its existing evidence; run missing integrated or visual checks only when explicitly authorized above. Repeat affected checks only when a later change, defect, or evidence gap makes that necessary.

Flag extra features, abstractions, restrictions, or acceptance obligations beyond the approved scope. Keep these observations separate from failures of agreed criteria. Do not fix code, redesign the solution, create new issues, merge, close the Task, or alter acceptance checklists. Publish only the selected report.

Return a compact criterion-to-evidence table: requirement, delivered result, evidence/revision, and satisfied, failed, or not verified. Name unresolved review findings, integration gaps, and their owning Subtasks. Do not add blocking checkboxes merely to record skipped checks.

Finish with one verdict: acceptance supported, acceptance not supported because of identified defects, or acceptance not established because evidence is missing. Explain the specific remaining work. Keep this assessment separate from authorization to merge or close issues, and do not claim checks that were not performed.
```
