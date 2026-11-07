---
name: context-retrieve
description: Use to locate project sources relevant to a question before reasoning or planning.
metadata:
  version: "1.1.0"
---

# Retrieve Project Sources

Ask where to find information about a specific subject, decision, or problem.
Describe the information you need and include known names, references, or constraints.

Example question:

> Where can I find the reasons for changing the phase two delivery date and confirmation of its approval?
> Locate the decision record and the revised schedule associated with REV-03.

## Goal

Locate relevant sources and return verified references with brief descriptions and missing information identified.

## Steps

1. **Extract search terms from the question.**

* Use the questions below to derive search phrases, identifiers, and simple synonyms.

| Question | Identify | Example |
| --- | --- | --- |
| What does the question concern? | Named subjects that provide starting points for the search | A question about a delivery date suggests `delivery date`, rather than the entire project name. |
| What does the requesting agent need to know about that subject? | Properties, actions, relationships, or reasons that distinguish the requested information | A question about why a date changed suggests `date change`, `reason`, and `decision`. |
| What limits the question's scope? | Conditions, dates, versions, or other qualifications supplied in the question | A question about an approved date for phase two suggests `approved` and `phase two`. |
| Which exact references does the question provide? | Names, identifiers, quotations, or paths to search literally | A supplied `REV-03` reference provides an exact search term. |
| What other wording expresses the same subject? | Alternative phrases that preserve the meaning of the question | `date change` and `changed date` provide alternative searches for the same subject. |
| What would a relevant source establish? | The contribution needed from a source | A decision record can explain a chosen date. A completion record can show whether the date was met. |

* Extend the questions to cover other details that affect source selection.
* Prioritize exact references supplied in the question. Use descriptive terms to locate material that the question does not name.

2. **Grep & Follow relevant sources.**

* Use existing project maps, indexes, and headings to select sources.
* Use the table to select the next action from the findings.

| Finding | Action | Example |
| --- | --- | --- |
| Matches concern the subject but differ in scope | Add a qualification from the question to narrow the search. | Add `phase two` to distinguish its delivery date from other phases. |
| The selected sources contain no matches | Try alternative wording from Step 1, then expand to related sources through maps and references. | Search `changed date` as well as `date change`, then inspect the decisions index. |
| A source supplies a relevant name or reference | Follow the reference or search the supplied name. | A schedule cites decision `D-17` as the reason for a changed date. |
| A section answers only part of the question | Retain the relevant section and search for the missing information. | A decision explains the changed date but leaves approval status unresolved. |
| A match provides neither requested information nor a relevant reference | Exclude the match from further reading. | A meeting agenda lists the delivery date but contains no discussion or reference to the change. |
| A source cannot be opened or read | Retain its location and the access problem. Continue through other available sources. | An index points to a decision record that is unavailable. |

* Use gaps in the selected sources to choose further searches, including cases not covered by the table.
* Continue when the sources cover the requested information or the available terms and references provide no further relevant leads.

3. **Report verified source locations.**

* Return the report as a message following this fictional example:

```text
- decisions/delivery-date.md:12-19 — Reasons for changing the phase two delivery date.
- plans/schedule.md:30-38 — Revised dates and the reference to decision D-17.
Not found: confirmation that the revised date was approved.
Unavailable: records/meeting-notes.md — access denied.
```

* Verify project-relative paths and exact line ranges against the opened sources.
