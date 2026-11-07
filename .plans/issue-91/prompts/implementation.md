# Implementation assignment

Start a chat with Terra Medium. Replace the angle-bracket fields and copy the prompt below. Choose the validation and delivery scope explicitly; drafting this template does not authorize execution.

```text
Implement Subtask <ISSUE_URL> in LeszekKantorek/msfs2024-artificial-horizon.

Validation: <required checks for the changed scope | static analysis only; no tests, builds, or application runs>.
Delivery: <local changes | commit | commit, push, and PR>.

Read AGENTS.md. Delegate context-retrieve to one Luna High subagent for bounded, read-only source retrieval. Read the relevant returned passages yourself. If that model is unavailable, report it rather than silently substituting another model.

Read the current Subtask description, first context comment, later amendments, artifacts, parent Task, and applicable Epic context. Read the predecessor handoff and the successor requirements that depend on your output. Use GitHub as the execution authority; do not depend on another agent's .local/ files. Follow the tasks-tests-review execution plan linked from the issues.

Implement the assigned outcome in a worktree under .worktrees/, on a codex/ branch from current main. Preserve unrelated work. Planning snapshots under .plans/ are references; do not include them in a PR targeting main.

Keep to accepted scope. Resolve routine implementation details independently. For consequential ambiguity, you may consult a Sol Medium subagent with a bounded question. A change to approved requirements needs a human decision; continue independent work while that decision is pending. Report useful additions beyond scope as proposals, without implementing or discarding them silently.

Select validation from the Subtask and repository scope matrix, within the authorization above. Reuse applicable evidence tied to the same relevant implementation and inputs. Repeat checks after changes or findings that invalidate that evidence, not merely because another phase began. If application code or build inputs change and execution is authorized, build the complete application.

For Task #91, S1-S6 have no screenshot or manual visual-approval checkpoint. S8 owns the integrated visual session. Keep earlier appearance claims unverified; retain applicable nonvisual checks. Inspect the next Subtask's prerequisites without implementing its scope early.

Do not merge or close issues. Publish only the artifacts authorized by Delivery. Keep any PR description short: changes, purpose, and material validation limits. Do not invent checklist items or merge gates to record skipped checks; describe them in prose without claiming success. Preserve existing acceptance criteria honestly.

Finish with the exact revision or identifiable patch, changes, validation actually performed, remaining gaps, and successor readiness. Record significant context gaps, consultations, decisions, and their outcomes in this chat, including resolved problems. State which requirements and artifacts the independent reviewer must inspect. Your own inspection is not independent review.
```
