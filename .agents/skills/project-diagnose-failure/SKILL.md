---
name: project-diagnose-failure
description: Diagnose an application, test, build, or local environment failure using reproduction and discriminating evidence. Use for an observed failure, not agent-behavior corrections, general code review, or a new feature request.
---

# Diagnose a Project Failure

## Goal

Establish a reproducible failure mechanism or a bounded unresolved diagnosis, then verify any authorized correction against the original scenario.

## Steps

Follow this sequence within the affected task scope.

1. Record the expected behavior and the observed failure separately.
2. Identify the revision, relevant local changes, environment, exact command or interaction, inputs, and native exit code.
3. Preserve relevant logs, screenshots, and artifact provenance without exposing credentials or unrelated data.
4. Select the [diagnostic boundary](../../../.context/error-diagnosis.md#select-the-diagnostic-boundary). Check its environment recovery and execution restrictions before reproducing the failure.
5. Reproduce the failure with the smallest scenario that retains its trigger.
6. Locate the first boundary where the observed behavior differs from its contract.
7. State a candidate cause and an observation that would distinguish it from competing causes.
8. Change one relevant condition or add a focused observation to test that candidate.
9. Select a fix only when the evidence explains the failure mechanism. For diagnosis-only requests, retain it as a proposal and proceed to reporting. If a correction is authorized, apply it within that scope.
10. After an applied correction, check the original failing scenario and affected regression behavior. If the cause remains uncertain, report alternatives and the next discriminating observation instead of claiming a fix.
11. Record the established cause, fix, evidence, and remaining uncertainty in the requested report. Update an issue or PR only when publication is authorized.

If reproduction is intermittent, record its conditions and frequency before drawing a causal conclusion.

## Evidence and recovery limits

Keep diagnosis separate from speculation and recovery attempts.

* Use the actual telemetry-to-display boundaries to narrow application failures.
* Compare receipt, normalization, freshness, scheduling, and painting where the symptom requires them.
* Preserve timestamps and deadlines rather than refreshing stale data to make a test pass.
* Report "cause not established" when observations cannot distinguish the candidate causes.
* Do not broaden permissions, disable security controls, or stop unrelated processes as diagnostic shortcuts.
* Use the [validation procedure](../../../docs/testing/evidence.md#select-evidence-by-failure-risk) for a regression check that detects the original failure.
* Reuse unaffected evidence under the [evidence-reuse rules](../../../docs/testing/index.md#evidence-reuse).
* Save established reusable findings through [context authoring](../../../.context/context-authoring.md), rather than storing raw debugging progress.

## Completion checks

Compare the outcome with the original expected behavior. Record confirmed causes separately from hypotheses, performed checks, and missing evidence.
For a diagnosis-only request, return the diagnosis and proposed correction without editing application code. Apply fixes only within the authorized scope.
When evidence cannot distinguish causes, report the unresolved alternatives and the next discriminating observation.
Use [context-diagnose](../context-diagnose/SKILL.md) for agent mistakes instead of application or environment failures.
The [source record](../../../.context/error-diagnosis.md#sources-and-limits) preserves the origins and limits of this procedure.
