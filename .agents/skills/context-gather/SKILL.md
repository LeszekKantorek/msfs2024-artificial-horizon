---
name: context-gather
description: Create and maintain .context knowledge from the whole available session and context. Use to save learning, build initial memory, review existing entries, or repair stale and duplicated knowledge. Create and maintain the index. Use context-apply to use knowledge and context-import-sessions to load saved sessions.
---

# Gather Context

## Step 1: Collect knowledge from the session

* **Review the whole available session and context, not only the last message.** Include user corrections, decisions, supplied sources, and verified results.
* Use imported session material when provided by `context-import-sessions`.
* Keep knowledge that can improve a future task; leave temporary progress in the task's plan.
* Keep source pointers and evidence limits with each candidate.
* Check consequential claims against accessible sources; do not turn an agent's assertion into an accepted decision.
* State missing evidence; never invent content from unavailable parts of a session.
* Leave session discovery and checkpoint updates to `context-import-sessions`.

* Use the table to spot useful knowledge; its examples are illustrative, not project facts.

| Look for | Ask | A useful entry could capture |
| --- | --- | --- |
| Decisions and rationale | What was chosen? Why? When should it be reconsidered? | Why exports use snapshots and when a new request is required. |
| Constraints and invariants | What must remain true? What would violate it? | Reopening an export must preserve the originally accepted snapshot. |
| Project concepts and relationships | What does a local term mean? How do the parts interact? | The difference between an export request, a job, and its result. |
| Local conventions | Which practice is required here? What establishes that requirement? | Where integration tests belong and which command runs them. |
| User corrections and preferences | What did the user correct? Does it apply beyond this task? | The user's preference for concise instructions and numbered procedures in project docs. |
| Repeatable procedures | Which steps worked? What are the preconditions and success checks? | How to reproduce a local build and verify its output. |
| Failures and recovery | What failed? Under which conditions? What fixed it? | A confirmed stale-cache failure with diagnosis and recovery steps. |
| Verified findings and limits | What did we observe? What remains unproven? | A batch-size experiment with its measured result and environment limits. |
| Reusable artifacts and source locations | What would save future work? Where is the canonical version? | A release checklist, configuration snippet, or link to the authoritative procedure. |
| Unresolved questions and hypotheses | What uncertainty could affect future work? What evidence would resolve it? | An unconfirmed retry-duplication risk with a concrete verification step. |

* The table contains only examples of potential categories; other kinds of useful knowledge are welcome.
* Keep a candidate when you can name a future situation where it changes an action, decision, or check.
* Capture the useful rule or finding with its scope and source; avoid a chronological session summary.
* If project files already explain something, link to the canonical source instead of copying its content into `.context/`.
* Add a short entry only when it helps locate or use that source; state when to read it and retain only missing context.
* Use these prompts selectively; do not create an entry for every row.

## Step 2: Write concise entries

* **Create new entries from the collected knowledge before reviewing existing context.** Leave duplicate checks and corrections to Step 3.
* Create `.context/` if needed; use new filenames without overwriting existing files.
* If the session contains no durable knowledge, continue to Step 3 without creating filler entries.
* Choose and adapt the shape that fits the content using [entry examples](references/entry-templates.md).
* Require `description` and `status` in YAML frontmatter.
* Limit `description` to three short sentences.
* Use exactly `active`, `superseded`, or `deprecated` for `status`.
* Require `superseded_by` only for `superseded`; resolve its path relative to the entry.
* Preserve useful existing relationships as links in the body.
* Use single-sentence bullets; split long sentences and paragraphs into smaller points.
* Use numbered lists for steps that must be followed in order.
* Add sections only when they help separate content.
* Use tables, snippets, reusable templates, or Mermaid and ASCII diagrams when useful.
* Preserve sources and rationale in short bullets; do not copy raw transcripts or secrets.

## Step 3: Review existing context

* **Now compare the new entries with existing context.** Read the index and inspect entry descriptions, including files missing from the index.
* Open entries that overlap with the new material or the requested review scope.
* For a full collection review, inspect every knowledge entry; otherwise report material coverage limits.
* Check accuracy, duplication, conflicting claims, status, and links.
* Use available task evidence when checking whether stored knowledge was applied correctly.
* Keep uncertain claims explicit; age alone does not make knowledge obsolete.
* Exclude `.context/sessions/` from knowledge entries.

| Finding | Action |
| --- | --- |
| A new entry covers a distinct topic | Keep the new entry. |
| New material adds evidence or corrects an existing topic | Merge it into the existing entry. |
| Entries duplicate the same knowledge | Merge into a clear canonical home and repair links. |
| A whole entry has an evidenced replacement | Set `status: superseded` and link it with `superseded_by`. |
| An entry is withdrawn without a replacement | Set `status: deprecated` and explain why. |
| Accepted claims still conflict | Preserve the conflict and identify the unresolved decision. |
| Existing knowledge fully covers a new entry | Keep the existing entry and remove the newly created redundant copy. |

* Remove a newly created copy after merging only when its useful content is preserved in the canonical entry.
* Fix affected incoming and outgoing links after moving or merging entries.
* Keep useful replacement history; do not create broken links or replacement cycles.

## Step 4: Create or update the index

* **Create `.context/INDEX.md` if the project has no context index.** Create `.context/` if needed.
* If an index already exists, update it in place; do not create a competing index.
* **The index contains only the table below.** Do not add frontmatter, headings, instructions, or prose.
* Move useful prose from an older index into entries before converting it to the table.
* Replace the example with links to real active entries and concrete `Use when` conditions.
* Use paths relative to the index; include each active entry once.
* Add new entries and update rows after renames or changes in scope.
* Remove rows for withdrawn or replaced entries; link to the active replacement where applicable.
* Exclude session records and the index itself.
* If no entries exist, keep only the table header and separator; do not invent a note to fill it.

```markdown
| Entry | Use when |
| --- | --- |
| [Repository instructions](repository-instructions.md) | Use when changing code or validating changes in this repository. |
```

## Step 5: Verify and report

* Check frontmatter, index coverage, link targets, and replacement chains.
* Try a concrete future question against the index and the affected entries.
* Report entries created, updated, merged, or withdrawn, plus any unresolved conflicts.
* Say when no durable knowledge changed; still repair a missing or outdated index within scope.

> Keep report-only requests read-only. Propose entry and index changes instead of writing them.
