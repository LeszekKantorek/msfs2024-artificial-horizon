---
description: Select diagnostic boundaries, environment restrictions, and supporting sources. Use project-diagnose-failure for the diagnosis procedure.
status: active
---

## Diagnostic procedure

The project-diagnose-failure skill owns the [diagnostic steps](../.agents/skills/project-diagnose-failure/SKILL.md#steps).

## Select the diagnostic boundary

Use the source that owns the failing boundary.

| Symptom | First source or check | Avoid this inference |
| --- | --- | --- |
| Rust tool or DLL blocked, SAC error 4551 | [Windows validation](windows-validation.md) and its required stop procedure | A dependency change or retry necessarily resolves the block |
| Executable replacement fails with error 5 | [Executable replacement diagnosis](local-validation-environment.md#executable-replacement-fails) | Error 5 proves a file lock or SAC failure |
| Browser cannot launch or reach a fixture | [Local validation environment](local-validation-environment.md) | A launch failure and a connection timeout have the same cause |
| Health succeeds but indications are unavailable | [Session and lifecycle](../docs/architecture.md#session-and-lifecycle), [freshness contract](../docs/telemetry-contract.md#freshness-and-status) | HTTP liveness proves source freshness |
| Wrong displayed value or delayed recovery | [Architecture boundaries](../docs/architecture.md#boundaries), [telemetry contract](../docs/telemetry-contract.md), [PFD composition](../docs/efi/composition.md#rationale-and-regression-lessons) | The display proves what the provider originally supplied |
| Overlap, clipping, or resize failure | [PFD layout validation](../docs/testing/pfd.md#geometry-and-paint-regression-checks), [paint order checks](../docs/testing/pfd.md#paint-order-checks) | Bounding boxes or a fresh sample prove correct painted geometry |
| Screenshot or executable differs | [Screenshot comparison](../docs/testing/evidence.md#screenshot-comparisons), [build provenance](../docs/testing/evidence.md#build-provenance) | Different bytes establish their cause or source revision |

## Evidence and recovery limits

The project-diagnose-failure skill owns the [evidence and recovery limits](../.agents/skills/project-diagnose-failure/SKILL.md#evidence-and-recovery-limits).

## Sources and limits

* Sources: the linked environment recoveries, architecture boundaries, telemetry contracts, and testing evidence procedures.
* Source: request to create project-based diagnostic guidance, 2026-10-11.
* This procedure combines existing diagnostic lessons without claiming a new reproduced failure or an established universal cause.
