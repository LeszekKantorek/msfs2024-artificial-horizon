---
name: project-write-content
description: Structure and review project Markdown with lists, tables, headings, code blocks, and Mermaid diagrams while preserving meaning. Apply when creating, revising, or moving Markdown, including documentation, context entries, agent instructions, and issue or PR text. Use technical-writing-rules for language and non-Markdown text. Exclude ordinary chat.
---

# Write Project Markdown

## Goal

Produce Markdown whose structure makes instructions, relationships, and limits explicit without changing their meaning.

## Steps

### 1. Select the writing task

* Identify the requested artifact, source material, and permitted edits. In review-only work, return findings without changing the artifact.
* Apply [technical-writing-rules](../technical-writing-rules/SKILL.md) for language and technical accuracy within its scope.
* Use [project writing conventions](../../../CONTRIBUTING.md#writing-guidance) for local requirements.
* Reuse the destination and findings already established by the task. For unresolved ownership, follow [context selection](../../../AGENTS.md#context-selection).
* Scale the procedure to the changed text. A short PR description does not require a document-wide reorganization.

Continue when the text to create or assess and its scope are clear.

### 2. Choose the Markdown structure

| Condition | Required form | Example |
| --- | --- | --- |
| Independent obligations, findings, results, or limitations need separate assessment; order does not matter | Bullets grouped by subject | [Independent rules](references/markdown-examples.md#independent-rules) |
| Actions must occur in order | Numbered steps, with one main action per step | [Ordered procedure](references/markdown-examples.md#ordered-procedure) |
| Conditions select between alternative actions | A condition/action table | [Conditions and alternatives](references/markdown-examples.md#conditions-and-alternatives) |
| Elements have common fields for comparison or lookup | A table with consistent columns | [Meaning across table rows](references/markdown-examples.md#meaning-across-table-rows) |
| Validation results share fields such as check, result, and limitation | A table; use grouped bullets when the fields differ | [PR validation results](references/markdown-examples.md#pr-validation-results) |
| A condition, explanation, example, or qualification belongs to one instruction or result | Keep them in the same item or row | [PR validation results](references/markdown-examples.md#pr-validation-results) |
| Sentences explain one mechanism or relationship without enumerating independent obligations, findings, results, or limitations | A short paragraph | [Connected explanation](references/markdown-examples.md#connected-explanation) |
| The artifact presents a flow structure, interaction sequence, state transitions, or dependency network | A fenced `mermaid` diagram; retain required instructions and limits in text | [Diagram scope](references/markdown-examples.md#diagram-scope) |

* Read the linked examples for the forms being changed.
* A shared topic does not permit combining independent obligations, findings, results, or limitations into a paragraph.
* Group evidence by claim or check. Keep each result with its explanation and qualification; do not create one item per sentence.
* Single newlines do not substitute for a list.
* A single relationship mentioned in a short text does not require a diagram.
* When no row fits, choose a form that preserves the relationship and reduces reading effort. Do not override a matching row.
* For feature pages, compare one representative page of the same type. Preserve justified differences in sections and structure.

Continue when each changed passage has a form suited to its information.

### 3. Construct the content

| Element | Rule |
| --- | --- |
| Headings | Preserve hierarchy and the scope of subordinate content. See [Heading scope](references/markdown-examples.md#heading-scope). |
| Lists | Use `*` bullets. Group by subject; keep explanations with the item they qualify. |
| Introductions | Introduce long lists or distinct groups with a short topic sentence. A clear heading can introduce a list without a redundant sentence. |
| Tables | Use consistent fields. Preserve conditions shared across rows explicitly; do not rely on the original sentence order. |
| Paragraphs | Keep one connected explanation together. Move independent obligations, findings, results, or limitations to lists or tables using Step 2. |
| Notes | Use `>` for constraints; keep required actions beside the affected step. Do not move a mandatory procedural limit into an optional note. |
| Identifiers and commands | Use inline code for identifiers and language-labelled fences for commands. Preserve their exact contents. |
| Checklists | Use `[ ]` for procedural checks or criteria, not duplicated task status. |
| Links | Use descriptive labels. Repair affected paths and anchors when content moves. |
| Mermaid | Preserve ordering, branches, skipped events, and limits. Do not imply guarantees absent from the source. |

Continue when the draft or review findings preserve the source meaning in the selected forms.

### 4. Check meaning and rendered structure

| Check | Required action |
| --- | --- |
| Coverage | Compare conditions, exceptions, values, units, obligation levels, and uncertainty against the source. Restore omissions. |
| Relationships | Check words such as “also”, “only”, and “unless” when splitting sentences. Make their scope explicit in the new form. |
| Paragraphs | Inspect every changed paragraph for independent obligations, findings, results, or limitations requiring separate assessment. Convert violations using Step 2; retain explanations and qualifications with their item. |
| Headings | Read each changed section with its subordinate content. Check whether warnings or qualifications acquired a narrower or broader scope. |
| Rendering | Inspect rendered Markdown for heading hierarchy, list boundaries, table columns, and code blocks. Valid syntax alone does not establish a suitable structure. |
| Diagrams | Compare every depicted relationship with the source. Check Mermaid rendering separately; a displayed code fence is not a rendered diagram. |
| Navigation | Check affected links and anchors, including incoming links to moved sections. |
| Review scope | Inspect changed sections and their surrounding text. For a recurring problem reported across a document, inspect the whole requested document. |

* Correct authorized edits and repeat affected checks. For review-only work, report locations and proposed corrections.
* Report checks actually performed and material limits, including unavailable rendering.
* Distinguish text inspection from observed execution of the skill.
