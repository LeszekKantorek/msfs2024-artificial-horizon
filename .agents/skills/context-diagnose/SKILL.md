---
name: context-diagnose
description: Use to diagnose agent mistakes or behavior flagged by user corrections and check targeted improvements.
---

# Diagnose Agent Behavior

## Goal

Identify why agent behavior fell short, apply a targeted correction, and check its effect using task evidence.

## Steps

1. **Establish the gap between expected and observed behavior.**

* Establish how observed behavior differs from expectations and how that affects the task.
* Use the user's request and corrections, applicable project decisions, and available work results.
* Use the questions below to establish the expected behavior, observations, and evidence limits.

| Question | Identify | Example |
| --- | --- | --- |
| What outcome or behavior was expected? | The expectation and the source that establishes it | Project guidance requires checking visual overlap after layout changes. |
| What did the agent actually do? | Observable actions, omissions, and results | The agent changed the layout and reported only unit test results. |
| Where did the behavior diverge? | The specific difference and its consequence | The result lacks evidence about visual overlap. |
| What does the available evidence establish? | Supported observations and missing material | The report omits a visual check; execution records are unavailable. |

* Add questions when the table does not establish the relevant difference.
* Continue when the expected behavior, specific gap, and supporting evidence or uncertainty are clear.

2. **Trace the context available at the point of failure.**

* Establish which knowledge and instructions could have influenced the behavior identified in Step 1.
* Examine relevant project documents, routes, skills, and available records of their use.
* Use the table to reconstruct the context available at the time.

| Question | Identify | Example |
| --- | --- | --- |
| Did the needed knowledge exist at the time? | Available guidance or a missing source | The visual validation procedure existed before the task began. |
| How could the agent reach it? | Selection conditions, links, and entry points | A layout route linked to the procedure only for changes described as resizing. |
| What evidence shows that it was read or used? | Records of retrieval and application, with their limits | A tool result contains the procedure, but the plan does not include its check. |
| What other instructions affected the decision? | Related, competing, or conflicting guidance | Another document describes unit tests as sufficient for all interface changes. |

* Examine other relationships when the table does not explain the relevant context.
* Continue when relevant context, access paths, and evidence of use are identified, with unavailable history explicit.

3. **Evaluate possible causes against the evidence.**

* Explain the gap from Step 1 using the context and evidence from Step 2.
* Compare plausible causes by what supports them, what contradicts them, and what would distinguish them.
* Use the table to investigate possible causes.

| Possible cause | Examine |
| --- | --- |
| Missing or incorrect knowledge | Did the agent have the information needed for a correct decision? |
| Ineffective routing | Did the situation lead to the relevant source? |
| Unclear instructions | Could the agent identify the applicable condition and required action? |
| Excessive or conflicting instructions | Did unnecessary requirements or contradictions interfere with the decision? |
| Failure to apply available guidance | Do available records show omission or incorrect application of guidance the agent read? |
| Execution or tool limitations | Did the problem occur despite a correct decision and sufficient context? |

* Investigate another cause when the listed explanations do not account for the observed gap.
* Retain competing explanations when the evidence cannot distinguish them.
* Continue when the cause is supported or the remaining hypotheses have checks that could distinguish them.

4. **Select and apply a targeted correction.**

* Select a response to the findings from Step 3 that supports the expected behavior from Step 1.
* Use the table to select a response suited to the diagnosis.

| Finding | Possible response |
| --- | --- |
| Missing or incorrect knowledge | Add or correct the needed information in the relevant source. |
| Ineffective routing | Correct the selection condition or link. |
| Unclear instructions | Clarify the affected condition or action. |
| Excessive or conflicting instructions | Remove, narrow, or reconcile the guidance. |
| Failure despite adequate guidance | Investigate how the instruction was applied before changing its content. |
| Execution or tool limitations | Address the execution constraint or identify the external change needed to proceed. |
| Uncertain cause | Perform a check that distinguishes the plausible explanations. |

* Adapt or combine responses when the listed options do not address the cause.
* Perform the selected check or apply the correction, identifying the expected result.
* Return to Step 3 when new evidence changes the diagnosis or leaves it unresolved.
* Continue when the correction is ready for a behavioral check. If progress is blocked, identify what is needed to proceed.

5. **Check whether the correction improves behavior.**

* Determine whether the correction from Step 4 improves the behavior identified in Step 1.
* Use a comparable task that exposes the original failure and a relevant case that could reveal unwanted effects.
* Compare observed behavior before and after the correction, accounting for differences that could affect the result.
* Use the table to assess the outcome and its limits.

| Check | Establish | Follow-up |
| --- | --- | --- |
| Original failure | Whether the expected behavior occurs after the correction | Revisit the diagnosis if the gap remains. |
| Applicability | Whether the correction works under the intended conditions | Adjust the correction or its scope where results differ from expectations. |
| Side effects | Whether the correction causes unnecessary reading, actions, or new errors | Adjust the correction to address the observed side effect. |
| Evidence limits | What was tested and what remains uncertain | Keep untested effectiveness and unresolved explanations explicit. |

* Add a check when another consequence could affect the task or interpretation of the result.
* Return to the diagnosis when results challenge the explanation, and repeat affected checks after further corrections.
* State the observed effect and its limits.
