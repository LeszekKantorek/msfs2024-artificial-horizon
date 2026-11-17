---
description: Explain local planning rationale and evidence limits. Use project-plan-change for the procedure.
status: active
---

## Planning rationale

The [planning skill](../.agents/skills/project-plan-change/SKILL.md) owns the procedure. The [execution rules](iterative-development.md#accepted-plans-and-handoffs) own context checks and reporting.

| Planning detail | Reason |
| --- | --- |
| Current sources and code | Prevent stale knowledge from overriding accepted requirements |
| Decisions before dependent work | Prevent unconfirmed assumptions from becoming implementation decisions |
| Explicit invariants and boundaries | Preserve behavior outside the requested change |
| Scenario inputs and expected results | Make checks reproducible and relevant to the outcome |
| Evidence selected by failure risk | Prevent a passing check from answering the wrong question |
| Review and device evidence limits | Distinguish desktop checks from independent review and real-device acceptance |

## Sources and limits

Select evidence that addresses changed behavior and failure risks. Report independent review separately from validation. Instruction reviews and scenario walkthroughs establish document coverage; claims about agent behavior, model selection, or GitHub publication require observed results.
