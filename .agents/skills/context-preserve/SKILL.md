---
name: context-preserve
description: Use to preserve project decisions and findings from available sessions or supplied sources. Record established knowledge with its scope, rationale, and evidence.
---

# Preserve Project Context

## Goal

Preserve project decisions and findings with their meaning, scope, rationale, and evidence intact.

## Steps

1. **Identify what the available material establishes.**

* Identify knowledge that can guide future project work.
* Review the whole available session and supplied sources within the requested scope.
* Use the questions below to identify knowledge worth preserving.

| Question | Identify | Example |
| --- | --- | --- |
| What understanding or decision emerged or changed? | Goals, explanations, decisions, requirements, and corrections | The user clarified which component owns retries. |
| What did the work teach us? | Findings, effective procedures, failures, and their conditions | Repeated diagnosis revealed a useful sequence of checks. |
| Why does this knowledge apply, and what supports it? | Rationale, sources, scope, and evidence | A decision defines intended behavior. A test shows observed behavior. |
| What remains unresolved? | Proposals, uncertainty, and conflicting claims | The cause remains uncertain despite a successful recovery. |
| When could this knowledge help again? | Future situations where the knowledge affects work | A similar failure can use the established diagnostic sequence. |

* Identifying useful knowledge matters more than matching the table. Add or combine questions to identify knowledge the table does not fully cover.
* Continue when you understand which knowledge merits preservation, what it establishes, and what limits its use.

2. **Identify the relationships needed to preserve meaning.**

* Identify relationships that future readers need to understand and use the knowledge.
* Use the knowledge from Step 1 and directly related sources.
* Use the table to connect relevant statements with their purpose, consequences, and evidence where sources support these relationships.

| Aspect | Questions | Example |
| --- | --- | --- |
| Intent | Which goal or need explains this knowledge? Which priorities or exclusions do the sources connect with that goal? | Explicit unavailable states help users distinguish missing data from zero. |
| Behavior | Which expected behavior does it define or constrain? Under which conditions does that behavior change? | A missing reading must remain visibly unavailable. |
| Architecture | Which responsibility or dependency does it establish or affect? Which other components depend on that responsibility? | The provider reports availability. The browser presents it. |
| Contracts | Which boundary guarantees does it depend on or establish? Which assumptions must callers and providers share? | An absent value must not become zero during transport. |
| Implementation | Which code or configuration realizes or contradicts it? Which implementation details explain the observed result? | The decoder retains absent values instead of supplying defaults. |
| Evidence | Which observations support or challenge these relationships? What do these observations leave untested or uncertain? | A controlled test showed an unavailable reading. Mobile readability remains untested. |

* Add or adapt questions when the table does not reveal relationships needed to understand or apply the knowledge.
* Keep gaps and disagreements explicit, including differences between intended behavior and observed results.
* Continue when you understand which relationships must accompany the knowledge and what supports or limits them.

3. **Record the knowledge for future use.**

* Record the knowledge so future readers can understand and use it without reconstructing the original conversation.
* Use the knowledge from Step 1 and the relationships from Step 2.
* Inspect directly related content in known project locations.
* Use the table to choose the location, organization, and form of the record.

| Decision | Question | Action |
| --- | --- | --- |
| Location | Where does this knowledge belong? Does related content already preserve it? | Reuse or update existing content. Create a focused entry when no known location fits. |
| Grouping | Which statements must readers understand together? | Keep those statements together. Link to related knowledge that belongs elsewhere. |
| Form | Which form makes the knowledge and its relationships clear? | Use explanations, tables, diagrams, or ordered steps according to the content. |
| Context | Which context should appear beside the statements, and which context can readers reach through links? | Include its purpose, rationale, scope, conditions, and exceptions where relevant. |
| Support and status | How should the record show supporting sources, evidence limits, and unresolved matters? | Retain source links and evidence limits. Keep decisions, observations, proposals, and unresolved matters distinguishable. |

* Preserve the selected knowledge when these choices do not fit. Adapt its form or location to retain its meaning.
* Apply the selected changes where the existing content does not preserve the knowledge and its relationships.
* Continue when the record contains the selected knowledge, its relationships, and the context needed for future use.

4. **Check the record against the selected knowledge and its intended use.**

* Check that the record preserves the selected knowledge and supports its intended future use.
* Compare the record from Step 3 with the knowledge from Step 1 and the relationships from Step 2.
* Use a future situation from Step 1 to assess whether the record supplies the necessary context.
* Use the table to identify omissions, unsupported additions, and changes in meaning.

| Check | Questions | Correction |
| --- | --- | --- |
| Coverage | Does the record preserve the selected knowledge? Can readers reach knowledge retained in existing sources? | Restore missing content or links to existing sources. |
| Meaning | Does the record say what the sources establish? Does it preserve the distinction between decisions, observations, and proposals? | Correct changed meanings and unsupported claims. |
| Context and relationships | Does the record retain the reasons, conditions, and relationships needed to understand the knowledge? | Restore missing rationale, scope, exceptions, or connections. |
| Evidence and uncertainty | Do claims retain their supporting evidence and its limits? Do unresolved matters remain explicit? | Correct unsupported certainty. Restore evidence, limitations, or unresolved disagreements. |
| Future use | Can readers use the record in the identified situation? Are the wording, organization, and links sufficient? | Clarify the wording, adjust the organization, or correct links using the established knowledge. |

* Add a check when another omission or change could affect interpretation or use.
* Correct the affected content.
* Repeat the affected checks after corrections.

