---
description: Diagnose application, test, and environment failures from reproducible observations. Use before selecting a fix or attributing a failure to its cause.
status: active
---

## Diagnostic procedure

Follow this sequence within the affected task scope.

1. Record the expected behavior and the observed failure separately.
2. Identify the revision, relevant local changes, environment, exact command or interaction, inputs, and native exit code.
3. Preserve relevant logs, screenshots, and artifact provenance without exposing credentials or unrelated data.
4. Check whether an existing environment recovery or execution restriction applies.
5. Reproduce the failure with the smallest scenario that retains its trigger.
6. Locate the first boundary where the observed behavior differs from its contract.
7. State a candidate cause and an observation that would distinguish it from competing causes.
8. Change one relevant condition or add a focused observation to test that candidate.
9. Select a fix only when the evidence explains the failure mechanism.
10. Check the original failing scenario and affected regression behavior after the fix.
11. Record the established cause, fix, evidence, and remaining uncertainty in the issue or PR.

If reproduction is intermittent, record its conditions and frequency before drawing a causal conclusion.

## Select the diagnostic boundary

Use the source that owns the failing boundary.

| Symptom | First source or check | Avoid this inference |
| --- | --- | --- |
| Rust tool or DLL blocked, SAC error 4551 | [Windows validation](windows-validation.md) and its required stop procedure | A dependency change or retry necessarily resolves the block |
| Executable replacement fails with error 5 | [Executable replacement diagnosis](local-validation-environment.md#executable-replacement-fails) | Error 5 proves a file lock or SAC failure |
| Browser cannot launch or reach a fixture | [Local validation environment](local-validation-environment.md) | A launch failure and a connection timeout have the same cause |
| Health succeeds but indications are unavailable | [Session and lifecycle](../docs/architecture.md#session-and-lifecycle), [freshness contract](../docs/telemetry-contract.md#freshness-and-status) | HTTP liveness proves source freshness |
| Wrong displayed value or delayed recovery | [Architecture boundaries](../docs/architecture.md#boundaries), [telemetry contract](../docs/telemetry-contract.md), [PFD composition](pfd-module-plan.md) | The display proves what the provider originally supplied |
| Overlap, clipping, or resize failure | [PFD layout validation](pfd-layout-validation.md), [paint order checks](pfd-layout-validation.md#paint-order-checks) | Bounding boxes or a fresh sample prove correct painted geometry |
| Screenshot or executable differs | [Screenshot comparison](../docs/testing.md#screenshot-comparisons), [build provenance](../docs/testing.md#build-provenance) | Different bytes establish their cause or source revision |

## Evidence and recovery limits

Keep diagnosis separate from speculation and recovery attempts.

* Use the actual telemetry-to-display boundaries to narrow application failures.
* Compare receipt, normalization, freshness, scheduling, and painting where the symptom requires them.
* Preserve timestamps and deadlines rather than refreshing stale data to make a test pass.
* Report "cause not established" when observations cannot distinguish the candidate causes.
* Do not broaden permissions, disable security controls, or stop unrelated processes as diagnostic shortcuts.
* Use the [validation procedure](validation-and-review.md#select-evidence-by-failure-risk) for a regression check that detects the original failure.
* Reuse unaffected evidence under the [evidence-reuse rules](../docs/testing.md#evidence-reuse).
* Save established reusable findings through [context authoring](context-authoring.md), rather than storing raw debugging progress.

## Sources and limits

* Sources: the linked environment recoveries, architecture boundaries, telemetry contracts, and testing evidence procedures.
* Source: request to create project-based diagnostic guidance, 2026-10-11.
* This procedure combines existing diagnostic lessons without claiming a new reproduced failure or an established universal cause.
